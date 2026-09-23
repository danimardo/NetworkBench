use networkbench_lib::control::transport::{recv_envelope, send_envelope};
use networkbench_lib::control::{SessionState, SessionStateMachine};
use networkbench_lib::model::plan::BenchmarkPlan;
use networkbench_lib::model::protocol::{
    CancelPayload, HelloPayload, PairRequestPayload, PairResultPayload, ProtocolEnvelope,
    ProtocolMessageType, RequestPayload, ResponsePayload,
};
use networkbench_lib::pairing::derive_pairing_code;
use tokio::net::TcpListener;
use uuid::Uuid;

#[tokio::test]
async fn test_two_peers_happy_path() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let peer_a_id = Uuid::new_v4();
    let peer_b_id = Uuid::new_v4();
    let fp_a = "1111111111111111111111111111111111111111111111111111111111111111";
    let fp_b = "2222222222222222222222222222222222222222222222222222222222222222";
    let session_id = Uuid::new_v4();

    // Servidor / Peer B
    let server_task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let (mut reader, mut writer) = socket.split();

        // 1. Recibir HELLO de A
        let hello_a: ProtocolEnvelope<HelloPayload> = recv_envelope(&mut reader).await.unwrap();
        assert_eq!(hello_a.msg_type, ProtocolMessageType::Hello);
        assert_eq!(hello_a.payload.instance_id, peer_a_id);

        // 2. Responder con HELLO de B
        let hello_b = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Hello,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-21T20:00:00Z".into(),
            in_reply_to: Some(hello_a.id),
            payload: HelloPayload {
                protocol_version: 1,
                protocol_min: 1,
                app_version: "0.1.0".into(),
                instance_id: peer_b_id,
                display_name: "Equipo-B".into(),
                platform: "windows".into(),
                is_busy: false,
            },
        };
        send_envelope(&mut writer, &hello_b).await.unwrap();

        // 3. Recibir PAIR_REQUEST
        let pair_req: ProtocolEnvelope<PairRequestPayload> = recv_envelope(&mut reader).await.unwrap();
        let expected_code = derive_pairing_code(fp_a, fp_b, &session_id).unwrap();
        assert_eq!(pair_req.payload.pairing_code, expected_code);

        // 4. Responder PAIR_RESULT aceptado
        let pair_res = ProtocolEnvelope {
            msg_type: ProtocolMessageType::PairResult,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-21T20:00:01Z".into(),
            in_reply_to: Some(pair_req.id),
            payload: PairResultPayload {
                accepted: true,
                reason: None,
            },
        };
        send_envelope(&mut writer, &pair_res).await.unwrap();

        // 5. Recibir REQUEST con plan
        let req: ProtocolEnvelope<RequestPayload> = recv_envelope(&mut reader).await.unwrap();
        assert_eq!(req.payload.plan.streams, 1);

        // 6. Responder RESPONSE aceptado
        let resp = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Response,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-21T20:00:02Z".into(),
            in_reply_to: Some(req.id),
            payload: ResponsePayload {
                accepted: true,
                reason: None,
                interface_name: Some("Ethernet 1".into()),
            },
        };
        send_envelope(&mut writer, &resp).await.unwrap();
    });

    // Cliente / Peer A (Iniciador)
    let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
    let (mut reader, mut writer) = socket.split();
    let mut sm_a = SessionStateMachine::new();
    sm_a.start_session(session_id, peer_b_id).unwrap();

    // 1. Enviar HELLO de A
    let hello_a = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Hello,
        id: Uuid::new_v4(),
        session_id: Some(session_id),
        ts: "2026-09-21T20:00:00Z".into(),
        in_reply_to: None,
        payload: HelloPayload {
            protocol_version: 1,
            protocol_min: 1,
            app_version: "0.1.0".into(),
            instance_id: peer_a_id,
            display_name: "Equipo-A".into(),
            platform: "windows".into(),
            is_busy: false,
        },
    };
    send_envelope(&mut writer, &hello_a).await.unwrap();

    // 2. Recibir HELLO de B
    let hello_b: ProtocolEnvelope<HelloPayload> = recv_envelope(&mut reader).await.unwrap();
    assert_eq!(hello_b.payload.instance_id, peer_b_id);
    assert!(!hello_b.payload.is_busy);
    sm_a.transition_to(SessionState::HelloPending, session_id).unwrap();

    // 3. Enviar PAIR_REQUEST
    let pairing_code = derive_pairing_code(fp_a, fp_b, &session_id).unwrap();
    let pair_req = ProtocolEnvelope {
        msg_type: ProtocolMessageType::PairRequest,
        id: Uuid::new_v4(),
        session_id: Some(session_id),
        ts: "2026-09-21T20:00:01Z".into(),
        in_reply_to: None,
        payload: PairRequestPayload { pairing_code },
    };
    send_envelope(&mut writer, &pair_req).await.unwrap();
    sm_a.transition_to(SessionState::Pairing, session_id).unwrap();

    // 4. Recibir PAIR_RESULT
    let pair_res: ProtocolEnvelope<PairResultPayload> = recv_envelope(&mut reader).await.unwrap();
    assert!(pair_res.payload.accepted);

    // 5. Enviar REQUEST
    let req = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Request,
        id: Uuid::new_v4(),
        session_id: Some(session_id),
        ts: "2026-09-21T20:00:02Z".into(),
        in_reply_to: None,
        payload: RequestPayload {
            plan: BenchmarkPlan::new_standard_tcp(7412),
            estimate_seconds: 12,
            suggested_interface: None,
        },
    };
    send_envelope(&mut writer, &req).await.unwrap();
    sm_a.transition_to(SessionState::Requesting, session_id).unwrap();

    // 6. Recibir RESPONSE
    let resp: ProtocolEnvelope<ResponsePayload> = recv_envelope(&mut reader).await.unwrap();
    assert!(resp.payload.accepted);

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_two_peers_user_rejection() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let session_id = Uuid::new_v4();
    let _peer_b_id = Uuid::new_v4();

    // Servidor que rechaza
    let server_task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let (mut reader, mut writer) = socket.split();

        let _req: ProtocolEnvelope<RequestPayload> = recv_envelope(&mut reader).await.unwrap();

        let resp = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Response,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-21T20:00:00Z".into(),
            in_reply_to: None,
            payload: ResponsePayload {
                accepted: false,
                reason: Some("El usuario rechazó la solicitud en el equipo remoto".into()),
                interface_name: None,
            },
        };
        send_envelope(&mut writer, &resp).await.unwrap();
    });

    let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
    let (mut reader, mut writer) = socket.split();

    let req = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Request,
        id: Uuid::new_v4(),
        session_id: Some(session_id),
        ts: "2026-09-21T20:00:00Z".into(),
        in_reply_to: None,
        payload: RequestPayload {
            plan: BenchmarkPlan::new_standard_tcp(7412),
            estimate_seconds: 12,
            suggested_interface: None,
        },
    };
    send_envelope(&mut writer, &req).await.unwrap();

    let resp: ProtocolEnvelope<ResponsePayload> = recv_envelope(&mut reader).await.unwrap();
    assert!(!resp.payload.accepted);
    assert_eq!(
        resp.payload.reason.as_deref(),
        Some("El usuario rechazó la solicitud en el equipo remoto")
    );

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_two_peers_cancellation() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let session_id = Uuid::new_v4();

    // Servidor que procesa cancelación
    let server_task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let (mut reader, mut writer) = socket.split();

        let cancel_msg: ProtocolEnvelope<CancelPayload> = recv_envelope(&mut reader).await.unwrap();
        assert_eq!(cancel_msg.msg_type, ProtocolMessageType::Cancel);
        assert_eq!(cancel_msg.payload.code.as_deref(), Some("NB-CONN-004"));

        // Responder CANCEL_ACK
        let ack = ProtocolEnvelope {
            msg_type: ProtocolMessageType::CancelAck,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-21T20:00:01Z".into(),
            in_reply_to: Some(cancel_msg.id),
            payload: serde_json::json!({ "acknowledged": true }),
        };
        send_envelope(&mut writer, &ack).await.unwrap();
    });

    let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
    let (mut reader, mut writer) = socket.split();

    let mut sm = SessionStateMachine::new();
    sm.start_session(session_id, Uuid::new_v4()).unwrap();
    sm.cancel(session_id).unwrap();

    let cancel_msg = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Cancel,
        id: Uuid::new_v4(),
        session_id: Some(session_id),
        ts: "2026-09-21T20:00:00Z".into(),
        in_reply_to: None,
        payload: CancelPayload {
            reason: "Cancelado por el usuario".into(),
            code: Some("NB-CONN-004".into()),
        },
    };
    send_envelope(&mut writer, &cancel_msg).await.unwrap();

    let ack: ProtocolEnvelope<serde_json::Value> = recv_envelope(&mut reader).await.unwrap();
    assert_eq!(ack.msg_type, ProtocolMessageType::CancelAck);

    sm.complete_cancellation();
    assert_eq!(sm.current_state(), SessionState::Cancelled);

    server_task.await.unwrap();
}
