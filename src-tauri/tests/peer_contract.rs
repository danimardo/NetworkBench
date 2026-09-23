use networkbench_lib::model::{
    decode_frame_length, encode_frame, BenchmarkDirection, BenchmarkPlan, BenchmarkProtocol,
    CancelPayload, HeartbeatPayload, HelloPayload, PairRequestPayload, PairResultPayload, Peer,
    ProtocolEnvelope, ProtocolMessageType, RequestPayload, ResponsePayload, TrustState,
    MAX_FRAME_SIZE_BYTES,
};
use uuid::Uuid;

#[test]
fn test_trust_state_variants_are_distinct() {
    let states = [
        TrustState::Unknown,
        TrustState::Known,
        TrustState::Trusted,
        TrustState::TrustedAutoAccept,
    ];

    // Verificar que los cuatro estados son distintos y serializan como camelCase
    let serialized: Vec<String> = states
        .iter()
        .map(|s| serde_json::to_string(s).unwrap())
        .collect();

    assert_eq!(serialized[0], "\"unknown\"");
    assert_eq!(serialized[1], "\"known\"");
    assert_eq!(serialized[2], "\"trusted\"");
    assert_eq!(serialized[3], "\"trustedAutoAccept\"");

    // Deserialización inversa exacta
    for (idx, json_str) in serialized.iter().enumerate() {
        let de: TrustState = serde_json::from_str(json_str).unwrap();
        assert_eq!(de, states[idx]);
    }
}

#[test]
fn test_peer_validation_and_normalization() {
    let id = Uuid::new_v4();
    let fp = "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855";
    let name = " Sobremesa-Laboratorio ";

    let peer = Peer::new(id, name.to_string(), fp.to_string(), vec!["192.168.1.10:7411".into()])
        .expect("Peer debe ser válido");

    assert_eq!(peer.display_name, "Sobremesa-Laboratorio");
    assert_eq!(
        peer.fingerprint,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(peer.trust_state, TrustState::Unknown);
    assert!(!peer.auto_accept);

    // Nombre inválido
    assert!(Peer::new(id, "".to_string(), fp.to_string(), vec![]).is_err());
    assert!(Peer::new(id, "a".repeat(49), fp.to_string(), vec![]).is_err());
    assert!(Peer::new(id, "Equipo\u{0000}Invalido".to_string(), fp.to_string(), vec![]).is_err());
    assert!(Peer::new(id, "Equipo\u{202E}Invalido".to_string(), fp.to_string(), vec![]).is_err());

    // Huella inválida
    assert!(Peer::new(id, "PC".to_string(), "corto".to_string(), vec![]).is_err());
    assert!(Peer::new(id, "PC".to_string(), "z".repeat(64), vec![]).is_err());
}

#[test]
fn test_benchmark_plan_limits() {
    let mut plan = BenchmarkPlan::new_standard_tcp(7412);
    assert!(plan.validate().is_ok());

    // Streams fuera de rango
    plan.streams = 0;
    assert!(plan.validate().is_err());
    plan.streams = 65;
    assert!(plan.validate().is_err());

    // 'both' dirección limita streams a 32
    plan.direction = BenchmarkDirection::Both;
    plan.streams = 33;
    assert!(plan.validate().is_err());
    plan.streams = 32;
    assert!(plan.validate().is_ok());

    // Medición
    plan.streams = 1;
    plan.measure_seconds = 4;
    assert!(plan.validate().is_err());
    plan.measure_seconds = 301;
    assert!(plan.validate().is_err());
    plan.measure_seconds = 10;
    assert!(plan.validate().is_ok());

    // Puerto reservado < 1024
    plan.port = 80;
    assert!(plan.validate().is_err());
    plan.port = 1024;
    assert!(plan.validate().is_ok());
}

#[test]
fn test_protocol_envelope_and_framing() {
    let env = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Hello,
        id: Uuid::new_v4(),
        session_id: None,
        ts: "2026-09-21T20:00:00Z".to_string(),
        in_reply_to: None,
        payload: HelloPayload {
            protocol_version: 1,
            protocol_min: 1,
            app_version: "0.1.0".to_string(),
            instance_id: Uuid::new_v4(),
            display_name: "Equipo-Dani".to_string(),
            platform: "windows".to_string(),
            is_busy: false,
        },
    };

    let json_bytes = serde_json::to_vec(&env).expect("Serializar envelope");
    let frame = encode_frame(&json_bytes).expect("Codificar frame");

    assert_eq!(frame.len(), 4 + json_bytes.len());
    let mut header = [0u8; 4];
    header.copy_from_slice(&frame[0..4]);
    assert_eq!(decode_frame_length(header), json_bytes.len());

    // Rechazar frames mayores a 1 MiB
    let oversized = vec![0u8; MAX_FRAME_SIZE_BYTES + 1];
    assert!(encode_frame(&oversized).is_err());
}

#[test]
fn test_pair_and_request_messages() {
    let req = RequestPayload {
        plan: BenchmarkPlan::new_standard_tcp(7412),
        estimate_seconds: 12,
        suggested_interface: Some("Ethernet".to_string()),
    };
    let json = serde_json::to_string(&req).expect("Serializar request");
    let de: RequestPayload = serde_json::from_str(&json).expect("Deserializar request");
    assert_eq!(de.estimate_seconds, 12);
    assert_eq!(de.plan.protocol, BenchmarkProtocol::Tcp);

    let pair_req = PairRequestPayload {
        pairing_code: "123456".to_string(),
    };
    assert_eq!(pair_req.pairing_code.len(), 6);

    let pair_res = PairResultPayload {
        accepted: true,
        reason: None,
    };
    assert!(pair_res.accepted);

    let resp = ResponsePayload {
        accepted: false,
        reason: Some("Rechazado".into()),
        interface_name: None,
    };
    assert!(!resp.accepted);

    let cancel = CancelPayload {
        reason: "Cancelado".into(),
        code: Some("NB-CONN-004".into()),
    };
    assert_eq!(cancel.code.as_deref(), Some("NB-CONN-004"));

    let hb = HeartbeatPayload { nonce: 999 };
    assert_eq!(hb.nonce, 999);
}
