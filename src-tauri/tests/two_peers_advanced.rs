use networkbench_lib::control::ports::allocate_ports_for_plan;
use networkbench_lib::control::transport::{recv_envelope, send_envelope};
use networkbench_lib::control::{SessionState, SessionStateMachine};
use networkbench_lib::diagnostic::{evaluate_udp_diagnostics, UdpLossLevel};
use networkbench_lib::model::plan::{BenchmarkDirection, BenchmarkPlan, BenchmarkProtocol};
use networkbench_lib::model::protocol::{
    HelloPayload, PairRequestPayload, PairResultPayload, ProtocolEnvelope,
    ProtocolMessageType, RequestPayload, ResponsePayload,
};
use networkbench_lib::pairing::derive_pairing_code;
use tokio::net::TcpListener;
use uuid::Uuid;

#[tokio::test]
async fn test_two_peers_unidirectional_forward() {
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

        let _hello_a: ProtocolEnvelope<HelloPayload> = recv_envelope(&mut reader).await.unwrap();
        let hello_b = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Hello,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-22T08:00:00Z".into(),
            in_reply_to: None,
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

        let pair_req: ProtocolEnvelope<PairRequestPayload> = recv_envelope(&mut reader).await.unwrap();
        let pair_res = ProtocolEnvelope {
            msg_type: ProtocolMessageType::PairResult,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-22T08:00:01Z".into(),
            in_reply_to: Some(pair_req.id),
            payload: PairResultPayload {
                accepted: true,
                reason: None,
            },
        };
        send_envelope(&mut writer, &pair_res).await.unwrap();

        // Recibir plan unidireccional (Forward)
        let req: ProtocolEnvelope<RequestPayload> = recv_envelope(&mut reader).await.unwrap();
        assert_eq!(req.payload.plan.direction, BenchmarkDirection::Forward);
        assert_eq!(req.payload.plan.streams, 4);

        let resp = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Response,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-22T08:00:02Z".into(),
            in_reply_to: Some(req.id),
            payload: ResponsePayload {
                accepted: true,
                reason: None,
                interface_name: Some("Ethernet 10G".into()),
            },
        };
        send_envelope(&mut writer, &resp).await.unwrap();
    });

    // Cliente / Peer A (Iniciador)
    let client_task = tokio::spawn(async move {
        let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
        let (mut reader, mut writer) = socket.split();

        let mut sm_a = SessionStateMachine::new();
        sm_a.start_session(session_id, peer_b_id).unwrap();

        // HELLO
        let hello_a = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Hello,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-22T08:00:00Z".into(),
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
        let _hello_b: ProtocolEnvelope<HelloPayload> = recv_envelope(&mut reader).await.unwrap();

        // PAIRING
        sm_a.transition_to(SessionState::HelloPending, session_id).unwrap();
        sm_a.transition_to(SessionState::Pairing, session_id).unwrap();

        let code = derive_pairing_code(fp_a, fp_b, &session_id).unwrap();
        let pair_req = ProtocolEnvelope {
            msg_type: ProtocolMessageType::PairRequest,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-22T08:00:01Z".into(),
            in_reply_to: None,
            payload: PairRequestPayload {
                pairing_code: code,
            },
        };
        send_envelope(&mut writer, &pair_req).await.unwrap();
        let _pair_res: ProtocolEnvelope<PairResultPayload> = recv_envelope(&mut reader).await.unwrap();

        // REQUEST con plan unidireccional
        sm_a.transition_to(SessionState::Requesting, session_id).unwrap();
        let mut plan = BenchmarkPlan::new_standard_tcp(5001);
        plan.direction = BenchmarkDirection::Forward;
        plan.streams = 4;

        let req = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Request,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-22T08:00:02Z".into(),
            in_reply_to: None,
            payload: RequestPayload {
                plan,
                estimate_seconds: 12,
                suggested_interface: Some("Ethernet 10G".into()),
            },
        };
        send_envelope(&mut writer, &req).await.unwrap();
        let resp: ProtocolEnvelope<ResponsePayload> = recv_envelope(&mut reader).await.unwrap();
        assert!(resp.payload.accepted);

        // Transición unidireccional: Preparing -> RunningSend -> Analyzing -> Completed (omitiendo reverse)
        sm_a.transition_to(SessionState::Preparing, session_id).unwrap();
        sm_a.transition_to(SessionState::RunningSend, session_id).unwrap();
        sm_a.transition_to(SessionState::Analyzing, session_id).unwrap();
        sm_a.transition_to(SessionState::Completed, session_id).unwrap();
        assert_eq!(sm_a.current_state(), SessionState::Completed);
    });

    tokio::try_join!(server_task, client_task).unwrap();
}

#[tokio::test]
async fn test_two_peers_simultaneous_running_both_and_cancellation() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);
    plan.direction = BenchmarkDirection::Both; // simultáneo
    plan.streams = 32;

    // Verificar asignación de puertos simultáneos sin solape
    let alloc = allocate_ports_for_plan(&plan).unwrap();
    assert!(alloc.is_simultaneous);
    assert!(!alloc.has_overlap());
    assert_eq!(alloc.forward_range, (5001, 5032));
    assert_eq!(alloc.reverse_range, (5033, 5064));

    // Máquina de estados en RUNNING_BOTH
    let session_id = Uuid::new_v4();
    let peer_id = Uuid::new_v4();
    let mut sm = SessionStateMachine::new();
    sm.start_session(session_id, peer_id).unwrap();
    sm.transition_to(SessionState::HelloPending, session_id).unwrap();
    sm.transition_to(SessionState::Requesting, session_id).unwrap();
    sm.transition_to(SessionState::Preparing, session_id).unwrap();

    // Entrar en RUNNING_BOTH
    sm.transition_to(SessionState::RunningBoth, session_id).unwrap();
    assert_eq!(sm.current_state(), SessionState::RunningBoth);
    assert!(sm.current_state().is_active());

    // Cancelar durante RUNNING_BOTH
    sm.cancel(session_id).unwrap();
    assert_eq!(sm.current_state(), SessionState::Cancelling);
    sm.complete_cancellation();
    assert_eq!(sm.current_state(), SessionState::Cancelled);
    assert!(sm.current_state().is_terminal());
}

#[tokio::test]
async fn test_two_peers_udp_diagnostics_and_loss() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);
    plan.protocol = BenchmarkProtocol::Udp;
    plan.udp_target_rate_bps = Some("100000000".to_string()); // 100 Mbit/s
    plan.udp_packet_size_bytes = Some(1472);
    assert!(plan.validate().is_ok());

    // Evaluación diagnóstica de la prueba UDP
    let sent_packets = 50_000u64;
    let received_packets = 49_800u64; // 200 perdidos = 0.4% pérdida (moderada)
    let emitted_bps = 100_000_000u64;
    let received_bps = 99_600_000u64;

    let diag = evaluate_udp_diagnostics(
        sent_packets,
        received_packets,
        emitted_bps,
        received_bps,
        Some(100_000_000),
        Some(1_000_000_000),
        None,
    );

    assert_eq!(diag.loss_level, UdpLossLevel::Moderate);
    assert_eq!(diag.packets_lost, 200);
    assert!((diag.loss_percent - 0.4).abs() < 1e-6);
    assert!(!diag.is_target_exceeded);
}

#[tokio::test]
async fn test_two_peers_malicious_out_of_bounds_plan_rejected() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let session_id = Uuid::new_v4();

    // Servidor que recibe un plan malicioso
    let server_task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let (mut reader, mut writer) = socket.split();

        let req: ProtocolEnvelope<RequestPayload> = recv_envelope(&mut reader).await.unwrap();

        // El receptor revalida bilateralmente el plan
        let is_valid = req.payload.plan.validate().is_ok();
        assert!(!is_valid, "El plan fuera de límites debe ser inválido");

        // Responder con rechazo tipado params_invalid
        let resp = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Response,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-22T08:00:03Z".into(),
            in_reply_to: Some(req.id),
            payload: ResponsePayload {
                accepted: false,
                reason: Some("params_invalid".into()),
                interface_name: None,
            },
        };
        send_envelope(&mut writer, &resp).await.unwrap();
    });

    let client_task = tokio::spawn(async move {
        let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
        let (mut reader, mut writer) = socket.split();

        // Enviar plan malicioso con 33 streams en simultáneo (límite es 32)
        let mut invalid_plan = BenchmarkPlan::new_standard_tcp(5001);
        invalid_plan.direction = BenchmarkDirection::Both;
        invalid_plan.streams = 33; // Ilegal

        let req = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Request,
            id: Uuid::new_v4(),
            session_id: Some(session_id),
            ts: "2026-09-22T08:00:02Z".into(),
            in_reply_to: None,
            payload: RequestPayload {
                plan: invalid_plan,
                estimate_seconds: 10,
                suggested_interface: None,
            },
        };
        send_envelope(&mut writer, &req).await.unwrap();

        let resp: ProtocolEnvelope<ResponsePayload> = recv_envelope(&mut reader).await.unwrap();
        assert!(!resp.payload.accepted);
        assert_eq!(resp.payload.reason, Some("params_invalid".into()));
    });

    tokio::try_join!(server_task, client_task).unwrap();
}
