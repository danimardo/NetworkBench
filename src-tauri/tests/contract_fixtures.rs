//! Fixtures de contrato **generados por Rust** (T160, T011: «checks de contratos»).
//!
//! Hasta ahora los contratos del frontend (`src/lib/contracts/*`) se probaban con JSON
//! escrito a mano, y las pruebas de Rust solo comprobaban trozos de su propia salida. Nadie
//! había parseado con el esquema de TypeScript lo que Rust **realmente serializa**, así que
//! una diferencia de nombre, de mayúsculas o de formato entre los dos lados solo habría
//! aparecido en ejecución, como un error de la interfaz al recibir datos reales.
//!
//! Este test es la mitad de Rust: construye valores reales con el código real y los compara
//! con ficheros de `tests/contracts/fixtures/rust/`. Esos mismos ficheros los parsea con Zod
//! `tests/contracts/rust-fixtures.test.ts`. Los dos lados quedan atados al mismo fichero.
//!
//! Si cambia un contrato a propósito, se regeneran:
//!
//! ```text
//! NB_UPDATE_FIXTURES=1 cargo test --manifest-path src-tauri/Cargo.toml --test contract_fixtures
//! ```
//!
//! y el cambio del fichero aparece en la revisión, junto con lo que rompa en TypeScript.

use networkbench_lib::control::engine_port::MotorDeLaboratorio;
use networkbench_lib::control::orquestador::{
    Direccion, EntradaSesion, MedidasDireccion, Orquestador,
};
use networkbench_lib::engine::ntttcp::parser::NtttcpRole;
use networkbench_lib::errors::{AppError, ErrorCode};
use networkbench_lib::ipc::snapshot::AppSnapshot;
use networkbench_lib::model::peer::{Peer, TrustState};
use networkbench_lib::model::plan::{BenchmarkPlan, BenchmarkProtocol};
use networkbench_lib::model::result::PeerSnapshot;
use networkbench_lib::sampling::SampleCollector;
use networkbench_lib::sampling::aggregate::SampleBatcher;
use networkbench_lib::sampling::samples::SamplePoint;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

fn carpeta() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("contracts")
        .join("fixtures")
        .join("rust")
}

/// Compara con el fichero, o lo escribe si `NB_UPDATE_FIXTURES=1`.
fn fijar<T: Serialize>(nombre: &str, valor: &T) {
    let actual = serde_json::to_string_pretty(valor).unwrap() + "\n";
    let ruta = carpeta().join(format!("{nombre}.json"));

    if std::env::var("NB_UPDATE_FIXTURES").as_deref() == Ok("1") {
        std::fs::create_dir_all(carpeta()).unwrap();
        std::fs::write(&ruta, &actual).unwrap();
        return;
    }
    let esperado = std::fs::read_to_string(&ruta)
        .unwrap_or_else(|_| panic!("Falta {ruta:?}; regenera con NB_UPDATE_FIXTURES=1"))
        .replace("\r\n", "\n");
    assert_eq!(
        actual, esperado,
        "El JSON que emite Rust para '{nombre}' cambió. Si es intencionado, regenera con \
         NB_UPDATE_FIXTURES=1 y revisa también el esquema de TypeScript."
    );
}

/// UUID v4 válido y determinista (bits de versión y variante fijados).
fn id(n: u128) -> Uuid {
    let con_version = (n & !(0xf_u128 << 76)) | (4_u128 << 76);
    let con_variante = (con_version & !(0x3_u128 << 62)) | (2_u128 << 62);
    Uuid::from_u128(con_variante)
}

fn snapshot(nombre: &str, n: u128) -> PeerSnapshot {
    PeerSnapshot {
        instance_id: id(n),
        display_name: nombre.to_string(),
        fingerprint: format!("{n:064x}"),
        address: "192.168.1.10:7411".to_string(),
    }
}

/// Una sesión de dos direcciones completas, con los datos que produce el orquestador real.
fn sesion(protocolo: BenchmarkProtocol) -> networkbench_lib::model::result::SessionResult {
    let o = Orquestador::new(Arc::new(MotorDeLaboratorio::con_respuestas(vec![])));
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);
    plan.protocol = protocolo;

    let direccion = |sentido: Direccion, enviados: u64, recibidos: u64, retrans: u64| {
        let mut emisor = MotorDeLaboratorio::resultado(NtttcpRole::Sender, 950_000_000, 10.0);
        emisor.packets_sent = Some(enviados);
        emisor.packets_retransmitted = Some(retrans);
        let mut receptor = MotorDeLaboratorio::resultado(NtttcpRole::Receiver, 948_000_000, 10.0);
        receptor.packets_received = Some(recibidos);
        o.resultado_de_direccion(&MedidasDireccion {
            direccion: sentido,
            protocolo,
            emisor: Some(emisor),
            receptor: Some(receptor),
            muestras: SampleCollector::new(),
        })
    };

    let mut s = o.ensamblar(EntradaSesion {
        session_id: id(0x5e55),
        started_at: "2026-09-25T10:00:00.000Z",
        finished_at: "2026-09-25T10:00:30.000Z",
        plan: &plan,
        initiator: snapshot("Equipo A", 0xa),
        responder: snapshot("Equipo B", 0xb),
        direcciones: vec![
            direccion(Direccion::Forward, 10_000, 9_990, 12),
            direccion(Direccion::Reverse, 10_000, 9_850, 3),
        ],
        capacidad: None,
        engine_version: "5.40".to_string(),
    });
    s.result_source = "initiator".to_string();
    s
}

#[test]
fn resultado_de_sesion_tcp() {
    fijar("session-result-tcp", &sesion(BenchmarkProtocol::Tcp));
}

/// Con pérdida UDP viajando en `retransmission` (T184): el frontend debe poder leerlo.
#[test]
fn resultado_de_sesion_udp() {
    fijar("session-result-udp", &sesion(BenchmarkProtocol::Udp));
}

#[test]
fn plan_tcp_y_udp() {
    let tcp = BenchmarkPlan::new_standard_tcp(5001);
    let mut udp = BenchmarkPlan::new_standard_tcp(5201);
    udp.protocol = BenchmarkProtocol::Udp;
    udp.udp_target_rate_bps = Some("100000000".into());
    udp.udp_packet_size_bytes = Some(1472);
    fijar("plan-tcp", &tcp);
    fijar("plan-udp", &udp);
}

#[test]
fn equipos_en_cada_estado_de_confianza() {
    let mut v = Vec::new();
    for (n, estado) in [
        TrustState::Unknown,
        TrustState::Known,
        TrustState::Trusted,
        TrustState::TrustedAutoAccept,
    ]
    .into_iter()
    .enumerate()
    {
        let mut p = Peer::new(
            id(0x100 + n as u128),
            format!("Equipo {n}"),
            format!("{:064x}", 0x100 + n as u128),
            vec!["192.168.1.20:7411".to_string()],
        )
        .unwrap();
        p.trust_state = estado;
        p.auto_accept = estado == TrustState::TrustedAutoAccept;
        p.last_seen = "2026-09-25T10:00:00.000Z".to_string();
        v.push(p);
    }
    fijar("peers", &v);
}

#[test]
fn instantanea_de_la_aplicacion() {
    fijar(
        "app-snapshot",
        &AppSnapshot {
            revision: 3,
            app_version: "0.1.0".into(),
            locale: "es".into(),
            theme: "dark".into(),
            instance_id: id(0xa).to_string(),
            instance_name: "Equipo A".into(),
            is_session_active: true,
            active_session_id: Some(id(0x5e55).to_string()),
            peers_count: 2,
        },
    );
}

#[test]
fn lote_de_muestras() {
    let mut b = SampleBatcher::new(id(0x5e55).to_string(), "forward");
    let mut lote = None;
    for i in 0..3u64 {
        lote = b.push(SamplePoint {
            t_ms: i * 500,
            direction: "forward".into(),
            bps: 900_000_000 + i,
            cpu_percent: if i == 1 { None } else { Some(12.5) },
            gap: i == 1,
        });
    }
    fijar("sample-batch", &lote.expect("un lote"));
}

#[test]
fn errores_de_aplicacion() {
    let v: Vec<AppError> = [
        ErrorCode::PortControlInUse,
        ErrorCode::FirewallUacRejected,
        ErrorCode::PermissionDenied,
        ErrorCode::InternalError,
    ]
    .into_iter()
    .map(AppError::from_code)
    .collect();
    fijar("app-errors", &v);
}
