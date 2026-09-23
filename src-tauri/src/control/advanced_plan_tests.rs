use crate::model::plan::{BenchmarkDirection, BenchmarkPlan, BenchmarkProtocol};

#[test]
fn test_sequential_streams_limits_and_boundaries() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);
    plan.direction = BenchmarkDirection::Forward;

    // Límites válidos 1..64
    plan.streams = 1;
    assert!(plan.validate().is_ok());
    plan.streams = 64;
    assert!(plan.validate().is_ok());

    // Fuera de rango
    plan.streams = 0;
    assert!(plan.validate().is_err());
    plan.streams = 65;
    assert!(plan.validate().is_err());

    // Mismos límites para reverse y both_sequential
    plan.direction = BenchmarkDirection::Reverse;
    plan.streams = 64;
    assert!(plan.validate().is_ok());

    plan.direction = BenchmarkDirection::BothSequential;
    plan.streams = 64;
    assert!(plan.validate().is_ok());
    plan.streams = 65;
    assert!(plan.validate().is_err());
}

#[test]
fn test_simultaneous_streams_limits_1_to_32() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);
    plan.direction = BenchmarkDirection::Both;

    // Límites válidos 1..32
    plan.streams = 1;
    assert!(plan.validate().is_ok());
    plan.streams = 32;
    assert!(plan.validate().is_ok());

    // 33 debe ser rechazado en simultáneo
    plan.streams = 33;
    assert!(plan.validate().is_err());
}

#[test]
fn test_timing_parameters_warmup_measure_cooldown() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);

    // Warmup 0..10 s
    plan.warmup_seconds = 0;
    assert!(plan.validate().is_ok());
    plan.warmup_seconds = 10;
    assert!(plan.validate().is_ok());
    plan.warmup_seconds = 11;
    assert!(plan.validate().is_err());
    plan.warmup_seconds = 2;

    // Cooldown 0..10 s
    plan.cooldown_seconds = 0;
    assert!(plan.validate().is_ok());
    plan.cooldown_seconds = 10;
    assert!(plan.validate().is_ok());
    plan.cooldown_seconds = 11;
    assert!(plan.validate().is_err());
    plan.cooldown_seconds = 2;

    // Measure 5..300 s
    plan.measure_seconds = 5;
    assert!(plan.validate().is_ok());
    plan.measure_seconds = 300;
    assert!(plan.validate().is_ok());
    plan.measure_seconds = 4;
    assert!(plan.validate().is_err());
    plan.measure_seconds = 301;
    assert!(plan.validate().is_err());
}

#[test]
fn test_port_boundaries_and_64_block_fit() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);

    // 1024..65000
    plan.port = 1024;
    assert!(plan.validate().is_ok());
    plan.port = 65000;
    assert!(plan.validate().is_ok());

    // Fuera de rango
    plan.port = 1023;
    assert!(plan.validate().is_err());
    plan.port = 65001;
    assert!(plan.validate().is_err());
}

#[test]
fn test_buffer_size_powers_of_two_and_ranges() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);

    // Válidos: potencias de 2 entre 4 KB (4096) y 4 MB (4194304)
    plan.buffer_size_bytes = Some("4096".to_string());
    assert!(plan.validate().is_ok());
    plan.buffer_size_bytes = Some("65536".to_string());
    assert!(plan.validate().is_ok());
    plan.buffer_size_bytes = Some("4194304".to_string());
    assert!(plan.validate().is_ok());

    // Inválidos: fuera de rango o no potencia de 2
    plan.buffer_size_bytes = Some("2048".to_string());
    assert!(plan.validate().is_err());
    plan.buffer_size_bytes = Some("8388608".to_string());
    assert!(plan.validate().is_err());
    plan.buffer_size_bytes = Some("5000".to_string());
    assert!(plan.validate().is_err());
    plan.buffer_size_bytes = Some("invalid".to_string());
    assert!(plan.validate().is_err());
}

#[test]
fn test_udp_specific_parameters_and_ranges() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);
    plan.protocol = BenchmarkProtocol::Udp;

    // Tasa objetivo: 1 Mbit/s..100 000 Mbit/s (1_000_000..100_000_000_000 bps)
    plan.udp_target_rate_bps = Some("1000000".to_string());
    assert!(plan.validate().is_ok());
    plan.udp_target_rate_bps = Some("100000000000".to_string());
    assert!(plan.validate().is_ok());
    plan.udp_target_rate_bps = Some("999999".to_string());
    assert!(plan.validate().is_err());
    plan.udp_target_rate_bps = Some("100000000001".to_string());
    assert!(plan.validate().is_err());
    plan.udp_target_rate_bps = None;

    // Tamaño de datagrama: 64..65 507 bytes
    plan.udp_packet_size_bytes = Some(64);
    assert!(plan.validate().is_ok());
    plan.udp_packet_size_bytes = Some(1472);
    assert!(plan.validate().is_ok());
    plan.udp_packet_size_bytes = Some(65507);
    assert!(plan.validate().is_ok());
    plan.udp_packet_size_bytes = Some(63);
    assert!(plan.validate().is_err());
    plan.udp_packet_size_bytes = Some(65508);
    assert!(plan.validate().is_err());
}

#[test]
fn test_expected_capacity_parameter_range() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);

    // 1 Mbit/s..400 000 Mbit/s
    plan.expected_capacity_bps = Some("1000000".to_string());
    assert!(plan.validate().is_ok());
    plan.expected_capacity_bps = Some("400000000000".to_string());
    assert!(plan.validate().is_ok());
    plan.expected_capacity_bps = Some("999999".to_string());
    assert!(plan.validate().is_err());
    plan.expected_capacity_bps = Some("400000000001".to_string());
    assert!(plan.validate().is_err());
}

#[test]
fn test_direction_serialization_and_aliases() {
    let json_seq = "\"both_sequential\"";
    let dir_seq: BenchmarkDirection = serde_json::from_str(json_seq).unwrap();
    assert_eq!(dir_seq, BenchmarkDirection::BothSequential);

    let json_sim = "\"both_simultaneous\"";
    let dir_sim: BenchmarkDirection = serde_json::from_str(json_sim).unwrap();
    assert_eq!(dir_sim, BenchmarkDirection::Both);

    let json_both = "\"both\"";
    let dir_both: BenchmarkDirection = serde_json::from_str(json_both).unwrap();
    assert_eq!(dir_both, BenchmarkDirection::Both);
}
