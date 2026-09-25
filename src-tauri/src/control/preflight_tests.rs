use crate::control::preflight::{PreflightCheckType, PreflightEvaluator, PreflightStatus};
use crate::errors::ErrorCode;
use crate::firewall::RuleStatus;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, TcpListener};

#[test]
fn test_preflight_engine_missing() {
    let dummy_path = env::temp_dir().join("nonexistent_ntttcp_binary_xyz123.exe");
    let check = PreflightEvaluator::check_engine(&dummy_path, "anyhash");
    assert_eq!(check.status, PreflightStatus::Failed);
    assert_eq!(check.check_type, PreflightCheckType::Engine);
    assert_eq!(check.error.unwrap().code, ErrorCode::EngineNotFound);
}

#[test]
fn test_preflight_engine_hash_mismatch() {
    let temp_file = env::temp_dir().join("test_ntttcp_fake.exe");
    fs::write(&temp_file, b"fake engine content").unwrap();

    let check = PreflightEvaluator::check_engine(
        &temp_file,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    let _ = fs::remove_file(&temp_file);

    assert_eq!(check.status, PreflightStatus::Failed);
    assert_eq!(check.error.unwrap().code, ErrorCode::EngineHashMismatch);
}

#[test]
fn test_preflight_engine_valid() {
    let temp_file = env::temp_dir().join("test_ntttcp_valid.exe");
    let content = b"valid engine binary mock";
    fs::write(&temp_file, content).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash_bytes = hasher.finalize();
    let expected_hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    let check = PreflightEvaluator::check_engine(&temp_file, &expected_hash);
    let _ = fs::remove_file(&temp_file);

    assert_eq!(check.status, PreflightStatus::Passed);
    assert!(check.error.is_none());
}

#[test]
fn test_preflight_nic_localhost() {
    let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let check = PreflightEvaluator::check_nic(localhost);
    assert_eq!(check.status, PreflightStatus::Passed);
    assert_eq!(check.check_type, PreflightCheckType::NetworkInterface);
}

#[test]
fn test_preflight_ports_available() {
    // Escoger un puerto libre dinámico
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let check = PreflightEvaluator::check_ports(Some(port), port + 10, 2);
    assert_eq!(check.status, PreflightStatus::Passed);
    assert_eq!(check.check_type, PreflightCheckType::Ports);
}

#[test]
fn test_preflight_ports_control_in_use() {
    let listener = TcpListener::bind("0.0.0.0:0").unwrap();
    let occupied_port = listener.local_addr().unwrap().port();

    let check = PreflightEvaluator::check_ports(Some(occupied_port), 5100, 2);
    assert_eq!(check.status, PreflightStatus::Failed);
    assert_eq!(check.error.unwrap().code, ErrorCode::PortControlInUse);
}

#[test]
fn test_preflight_ports_sin_control_port_no_lo_comprueba() {
    // Quien pregunta ya está escuchando en su propio puerto de control: pasarlo
    // comprobaría que un puerto ocupado por uno mismo está ocupado, lo cual es un
    // sinsentido, no una comprobación (T173).
    //
    // El puerto de datos se toma de un `bind(0)` recién soltado, no de un desplazamiento
    // arbitrario: con las pruebas corriendo en paralelo, un puerto elegido a ciegas
    // (p. ej. `+1000`) puede coincidir con el que otra prueba tiene abierto en ese
    // instante, y la prueba falla de forma intermitente sin que el código cambie.
    let base = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let check = PreflightEvaluator::check_ports(None, base, 1);
    assert_eq!(check.status, PreflightStatus::Passed);
}

#[test]
fn test_preflight_disk_space() {
    let temp_dir = env::temp_dir();
    // Umbral de 1 MB: debe pasar con éxito en cualquier equipo normal
    let check_pass = PreflightEvaluator::check_disk_space(&temp_dir, 1);
    assert_eq!(check_pass.status, PreflightStatus::Passed);

    // Umbral gigante (1000 TB): debe generar aviso
    let check_warn = PreflightEvaluator::check_disk_space(&temp_dir, 1_000_000_000);
    assert_eq!(check_warn.status, PreflightStatus::Warning);
    assert_eq!(check_warn.error.unwrap().code, ErrorCode::DiskSpaceLow);
}

#[test]
fn test_preflight_version_check() {
    let check_ok = PreflightEvaluator::check_version(1, 1);
    assert_eq!(check_ok.status, PreflightStatus::Passed);

    let check_mismatch = PreflightEvaluator::check_version(1, 2);
    assert_eq!(check_mismatch.status, PreflightStatus::Failed);
    assert_eq!(
        check_mismatch.error.unwrap().code,
        ErrorCode::VersionIncompatible
    );
}

#[test]
fn test_preflight_firewall_diagnostics() {
    // 1. Canal de control no responde -> NB-FW-001
    let check1 = PreflightEvaluator::check_firewall(false, false, RuleStatus::Missing);
    assert_eq!(check1.status, PreflightStatus::Failed);
    assert_eq!(
        check1.error.unwrap().code,
        ErrorCode::FirewallBlockedControl
    );

    // 2. Canal de control OK pero sondeo falla y regla ausente -> NB-FW-002
    let check2 = PreflightEvaluator::check_firewall(true, false, RuleStatus::Missing);
    assert_eq!(check2.status, PreflightStatus::Failed);
    assert_eq!(check2.error.unwrap().code, ErrorCode::FirewallBlockedNtttcp);

    // 3. Canal de control OK pero sondeo falla y regla deshabilitada -> NB-FW-002
    let check3 = PreflightEvaluator::check_firewall(true, false, RuleStatus::Disabled);
    assert_eq!(check3.status, PreflightStatus::Failed);
    assert_eq!(check3.error.unwrap().code, ErrorCode::FirewallBlockedNtttcp);

    // 4. Canal de control OK pero sondeo falla y regla presente -> NB-FW-005 (bloqueo intermedio)
    let check4 = PreflightEvaluator::check_firewall(true, false, RuleStatus::Present);
    assert_eq!(check4.status, PreflightStatus::Failed);
    assert_eq!(
        check4.error.unwrap().code,
        ErrorCode::FirewallExternalBlocked
    );

    // 5. Todo OK -> Passed
    let check5 = PreflightEvaluator::check_firewall(true, true, RuleStatus::Present);
    assert_eq!(check5.status, PreflightStatus::Passed);
    assert!(check5.error.is_none());
}
