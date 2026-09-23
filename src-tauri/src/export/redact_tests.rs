use crate::export::redact::*;
use crate::model::result::SessionResult;

#[test]
fn test_redact_individual_identifiers() {
    let raw_ipv4 = "Conectando a 192.168.1.50 desde 10.0.0.12 en el puerto 5001";
    let redacted_ipv4 = redact_ips(raw_ipv4);
    assert!(!redacted_ipv4.contains("192.168.1.50"));
    assert!(!redacted_ipv4.contains("10.0.0.12"));
    assert!(redacted_ipv4.contains(REDACTED_IP_PLACEHOLDER));

    let raw_ipv6 = "Dirección link-local: fe80::1ff:fe23:4567:890a detectada";
    let redacted_ipv6 = redact_ips(raw_ipv6);
    assert!(!redacted_ipv6.contains("fe80::1ff:fe23:4567:890a"));
    assert!(redacted_ipv6.contains(REDACTED_IP_PLACEHOLDER));

    let raw_mac = "MAC encontrada: 00:1A:2B:3C:4D:5E y secundaria A4-BB-6D-80-01-22";
    let redacted_mac = redact_macs(raw_mac);
    assert!(!redacted_mac.contains("00:1A:2B:3C:4D:5E"));
    assert!(!redacted_mac.contains("A4-BB-6D-80-01-22"));
    assert!(redacted_mac.contains(REDACTED_MAC_PLACEHOLDER));

    let raw_fp = "Huella cert: a1b2c3d4e5f60123456789abcdef0123456789abcdef0123456789abcdef0123 verificada";
    let redacted_fp = redact_fingerprints(raw_fp);
    assert!(!redacted_fp.contains("a1b2c3d4e5f60123456789abcdef0123456789abcdef0123456789abcdef0123"));
    assert!(redacted_fp.contains(REDACTED_FINGERPRINT_PLACEHOLDER));
}

#[test]
fn test_redact_session_fixture_with_canaries_no_leaks() {
    let fixture_str = include_str!("../../../tests/fixtures/export/session_with_canaries.json");
    let session: SessionResult = serde_json::from_str(fixture_str).expect("Deserializar fixture con canarios");

    // Lista explícita de canarios sensibles
    let canaries = vec![
        "192.168.1.105",
        "192.168.1.200",
        "fe80::1ff:fe23:4567:890a",
        "00:1A:2B:3C:4D:5E",
        "A4-BB-6D-80-01-22",
        "a1b2c3d4e5f60123456789abcdef0123456789abcdef0123456789abcdef0123",
        "b2c3d4e5f6a10123456789abcdef0123456789abcdef0123456789abcdef0123",
    ];

    // Verificar que el fixture original contiene todos los canarios
    for canary in &canaries {
        assert!(
            fixture_str.contains(canary),
            "El fixture original debe contener el canario {}",
            canary
        );
    }

    // Redactar la sesión
    let redacted_session = redact_session_result(&session, true);
    let serialized_redacted = serde_json::to_string(&redacted_session).expect("Serializar sesión redactada");

    // Criterio AC-NB-12: ningún canario identificativo debe aparecer en el JSON resultante
    for canary in &canaries {
        assert!(
            !serialized_redacted.contains(canary),
            "FUGA DETECTADA: El resultado anonimizado contiene el canario {}",
            canary
        );
    }

    // Comprobar que los bloques crudos no saneables se sustituyeron por la declaración de omisión
    assert!(serialized_redacted.contains(RAW_OMITTED_PLACEHOLDER));

    // Comprobar que las huellas y direcciones se anonimizaron
    assert_eq!(redacted_session.initiator.fingerprint, REDACTED_FINGERPRINT_PLACEHOLDER);
    assert_eq!(redacted_session.responder.fingerprint, REDACTED_FINGERPRINT_PLACEHOLDER);
    assert!(redacted_session.initiator.address.contains(REDACTED_IP_PLACEHOLDER));
    assert!(redacted_session.responder.address.contains(REDACTED_IP_PLACEHOLDER));

    // Las métricas numéricas esenciales (throughput, bytes, etc.) deben conservarse intactas
    assert_eq!(redacted_session.status, "completed");
    assert_eq!(
        redacted_session.directions[0].official_bps.as_deref(),
        Some("948000000")
    );
}

#[test]
fn test_unredacted_session_preserves_original_values() {
    let fixture_str = include_str!("../../../tests/fixtures/export/session_with_canaries.json");
    let session: SessionResult = serde_json::from_str(fixture_str).expect("Deserializar fixture");

    let preserved = redact_session_result(&session, false);
    assert_eq!(preserved.initiator.address, session.initiator.address);
    assert_eq!(preserved.initiator.fingerprint, session.initiator.fingerprint);
    assert_eq!(preserved.responder.address, session.responder.address);
    assert_eq!(preserved.directions[0].sender.as_ref().unwrap().raw, session.directions[0].sender.as_ref().unwrap().raw);
}
