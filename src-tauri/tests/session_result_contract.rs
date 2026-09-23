use networkbench_lib::diagnostic::{
    AsymmetryStats, CapacityReference, CapacitySource, CpuLevel, DiagnosticItem,
    RetransmissionLevel, RetransmissionStats, SessionVerdict, StabilityLevel, StabilityStats,
    THRESHOLDS_HASH, VerdictLevel,
};
use networkbench_lib::model::plan::BenchmarkPlan;
use networkbench_lib::model::result::{
    DirectionResult, EngineResult, PeerSnapshot, ResultVersions, SessionResult,
};
use uuid::Uuid;

#[test]
fn test_session_result_serde_roundtrip_completed() {
    let initiator = PeerSnapshot {
        instance_id: Uuid::new_v4(),
        display_name: "DESKTOP-LOCAL".into(),
        fingerprint: "sha256:112233".into(),
        address: "192.168.1.10".into(),
    };
    let responder = PeerSnapshot {
        instance_id: Uuid::new_v4(),
        display_name: "SERVER-REMOTE".into(),
        fingerprint: "sha256:445566".into(),
        address: "192.168.1.20".into(),
    };

    let plan = BenchmarkPlan::new_standard_tcp(5001);

    let capacity = CapacityReference {
        ref_bps: Some(1_000_000_000),
        ref_source: CapacitySource::Negotiated,
        cap_a_bps: Some(1_000_000_000),
        cap_b_bps: Some(1_000_000_000),
    };

    let sender = EngineResult {
        role: "sender".into(),
        total_bytes: "2375000000".into(),
        realtime_seconds: 20.0,
        throughput_bps: "950000000".into(),
        cpu_percent: Some(12.5),
        buffers_count: Some(36240),
        errors_count: 0,
        raw: None,
    };
    let receiver = EngineResult {
        role: "receiver".into(),
        total_bytes: "2370000000".into(),
        realtime_seconds: 20.0,
        throughput_bps: "948000000".into(),
        cpu_percent: Some(8.2),
        buffers_count: Some(36160),
        errors_count: 0,
        raw: None,
    };

    let mut direction = DirectionResult::new_completed("forward", sender, receiver, 40, 0);
    direction.utilization = Some(0.948);
    direction.stability = Some(StabilityStats {
        level: StabilityLevel::VeryStable,
        mean_bps: 948_000_000.0,
        std_dev_bps: 9_480_000.0,
        cv: 0.01,
        min_bps: 940_000_000.0,
        max_bps: 955_000_000.0,
        p5_bps: 942_000_000.0,
        p50_bps: 948_000_000.0,
        p95_bps: 954_000_000.0,
        drops_count: 0,
        samples_count: 40,
        gaps_count: 0,
    });
    direction.retransmission = Some(RetransmissionStats {
        level: RetransmissionLevel::Normal,
        ratio: Some(0.0001),
        packets_sent: Some(160000),
        packets_retransmitted: Some(16),
    });

    let verdict = SessionVerdict {
        rules_version: "1.0.0".into(),
        level: VerdictLevel::Ok,
        title_key: "verdict.ok_title".into(),
        performance_level: Some(VerdictLevel::Ok),
        stability_level: StabilityLevel::VeryStable,
        asymmetry_level: Some(VerdictLevel::Ok),
        retransmission_level: RetransmissionLevel::Normal,
        cpu_level: CpuLevel::Low,
        facts: vec![DiagnosticItem {
            rule_id: "FACT_SPEED".into(),
            message_key: "facts.speed_measured".into(),
            safe_params: Some(serde_json::json!({ "speedBps": "948000000" })),
            evidence_refs: vec!["officialBps".into()],
        }],
        observations: vec![],
        possible_causes: vec![],
        actions: vec![],
    };

    let session = SessionResult {
        schema_version: 1,
        session_id: Uuid::new_v4(),
        started_at: "2026-09-22T05:00:00.000Z".into(),
        finished_at: "2026-09-22T05:01:00.000Z".into(),
        status: "completed".into(),
        initiator,
        responder,
        plan,
        capacity: Some(capacity),
        directions: vec![direction],
        asymmetry: Some(AsymmetryStats {
            ratio: 0.02,
            is_asymmetric: false,
        }),
        verdict: Some(verdict),
        result_source: "initiator".into(),
        versions: ResultVersions {
            app_version: "0.1.0".into(),
            protocol_version: 1,
            engine_version: "5.40".into(),
            schema_version: 1,
            thresholds_hash: THRESHOLDS_HASH.into(),
        },
    };

    let json = serde_json::to_string(&session).expect("serialización fallida");
    assert!(json.contains("\"officialBps\":\"948000000\""));
    assert!(json.contains(
        "\"thresholdsHash\":\"b65f73615249f16c22fa3d63685f792b7f94661eed4f7e0eea454d1dba8923c0\""
    ));

    let deserialized: SessionResult = serde_json::from_str(&json).expect("deserialización fallida");
    assert_eq!(session, deserialized);
    assert!(deserialized.is_fully_completed());
}

#[test]
fn test_session_result_incomplete_no_official_bps() {
    let initiator = PeerSnapshot {
        instance_id: Uuid::new_v4(),
        display_name: "DESKTOP-LOCAL".into(),
        fingerprint: "sha256:112233".into(),
        address: "192.168.1.10".into(),
    };
    let responder = PeerSnapshot {
        instance_id: Uuid::new_v4(),
        display_name: "SERVER-REMOTE".into(),
        fingerprint: "sha256:445566".into(),
        address: "192.168.1.20".into(),
    };
    let plan = BenchmarkPlan::new_standard_tcp(5001);
    let direction = DirectionResult::new_incomplete("forward");

    let session = SessionResult {
        schema_version: 1,
        session_id: Uuid::new_v4(),
        started_at: "2026-09-22T05:00:00.000Z".into(),
        finished_at: "2026-09-22T05:00:10.000Z".into(),
        status: "incomplete".into(),
        initiator,
        responder,
        plan,
        capacity: None,
        directions: vec![direction],
        asymmetry: None,
        verdict: None,
        result_source: "local".into(),
        versions: ResultVersions {
            app_version: "0.1.0".into(),
            protocol_version: 1,
            engine_version: "5.40".into(),
            schema_version: 1,
            thresholds_hash: THRESHOLDS_HASH.into(),
        },
    };

    let json = serde_json::to_string(&session).expect("serialización fallida");
    assert!(!json.contains("\"officialBps\""));
    assert!(!session.is_fully_completed());
}
