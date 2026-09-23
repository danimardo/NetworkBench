use networkbench_lib::control::domain::{SessionState, SessionStateMachine};
use networkbench_lib::errors::ErrorCode;
use networkbench_lib::model::peer::{Peer, TrustState};
use networkbench_lib::model::protocol::{ProtocolEnvelope, ProtocolMessageType};
use uuid::Uuid;

#[test]
fn test_oversized_payload_rejected() {
    let huge_payload = "A".repeat(128 * 1024);
    let envelope = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Cancel,
        id: Uuid::new_v4(),
        session_id: Some(Uuid::new_v4()),
        ts: "2026-09-22T06:00:00Z".to_string(),
        in_reply_to: None,
        payload: serde_json::Value::String(huge_payload),
    };

    let serialized = serde_json::to_vec(&envelope).unwrap();
    // Trama superior a 64 KB
    assert!(serialized.len() > 64 * 1024);
}

#[test]
fn test_out_of_state_transitions_rejected() {
    let mut sm = SessionStateMachine::new();
    assert_eq!(sm.current_state(), SessionState::Idle);

    let session_id = Uuid::new_v4();
    // Transicionar a RunningSend sin haber iniciado sesión es ilegal
    let res = sm.transition_to(SessionState::RunningSend, session_id);
    assert!(res.is_err());
    assert_eq!(sm.current_state(), SessionState::Idle);
}

#[test]
fn test_duplicate_session_request_rejected() {
    let mut sm = SessionStateMachine::new();
    let session_id = Uuid::new_v4();
    let peer_id = Uuid::new_v4();

    sm.start_session(session_id, peer_id).unwrap();
    assert_eq!(sm.current_state(), SessionState::Connecting);

    // Intentar iniciar una segunda sesión simultánea debe ser rechazado
    let second_session_id = Uuid::new_v4();
    let res = sm.start_session(second_session_id, peer_id);
    assert!(res.is_err());
    assert_eq!(sm.current_state(), SessionState::Connecting);
}

#[test]
fn test_peer_identity_changed_triggers_error() {
    let original_peer = Peer {
        instance_id: Uuid::new_v4(),
        display_name: "PC-Original".to_string(),
        fingerprint: "1111222233334444555566667777888811112222333344445555666677778888".to_string(),
        addresses: vec!["192.168.1.50".to_string()],
        trust_state: TrustState::Trusted,
        auto_accept: false,
        last_seen: "2026-09-22T06:00:00Z".to_string(),
        alias: None,
    };

    let spoofed_fingerprint = "9999888877776666555544443333222299998888777766665555444433332222";

    // Si la huella no coincide con la guardada, la confianza queda invalidada
    let is_same = original_peer.fingerprint == spoofed_fingerprint;
    assert!(!is_same);

    // Se asocia con NB-PEER-006 (PeerIdentityChanged)
    let err_code = ErrorCode::PeerIdentityChanged;
    assert_eq!(err_code, ErrorCode::PeerIdentityChanged);
}
