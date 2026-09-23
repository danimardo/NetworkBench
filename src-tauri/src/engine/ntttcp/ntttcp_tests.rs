use super::*;
use crate::model::plan::BenchmarkPlan;
use std::path::Path;

#[test]
fn test_parse_real_receiver_xml_fixture() {
    let xml = include_str!("../../../../tests/fixtures/ntttcp/receiver_success.xml");
    let res = parse_ntttcp_xml(xml).expect("Parsear XML de receptor exitoso");

    assert_eq!(res.role, NtttcpRole::Receiver);
    assert_eq!(res.total_bytes, 1_186_201_600);
    assert_eq!(res.total_buffers, 18_100);
    assert_eq!(res.throughput_bps, 948_771_000);
    assert_eq!(res.errors_count, 0);
    assert!(res.realtime_seconds >= 10.0);
    assert_eq!(res.cpu_percent, Some(4.25));
}

#[test]
fn test_parse_real_sender_xml_fixture() {
    let xml = include_str!("../../../../tests/fixtures/ntttcp/sender_success.xml");
    let res = parse_ntttcp_xml(xml).expect("Parsear XML de emisor exitoso");

    assert_eq!(res.role, NtttcpRole::Sender);
    assert_eq!(res.total_bytes, 1_191_444_480);
    assert_eq!(res.total_buffers, 18_180);
    assert_eq!(res.throughput_bps, 953_060_000);
    assert_eq!(res.errors_count, 0);
    assert!(res.realtime_seconds >= 10.0);
    assert_eq!(res.cpu_percent, Some(6.80));
}

#[test]
fn test_parse_corrupted_and_empty_xml() {
    let corrupted = include_str!("../../../../tests/fixtures/ntttcp/corrupted.xml");
    let res_corr = parse_ntttcp_xml(corrupted);
    assert!(res_corr.is_err());
    assert!(matches!(res_corr.unwrap_err(), NtttcpParseError::MalformedXml(_) | NtttcpParseError::MissingField(_)));

    let empty = include_str!("../../../../tests/fixtures/ntttcp/empty.xml");
    let res_empty = parse_ntttcp_xml(empty);
    assert_eq!(res_empty, Err(NtttcpParseError::EmptyXml));
}

#[test]
fn test_build_allowlisted_args_receiver() {
    let plan = BenchmarkPlan::new_standard_tcp(7412);
    let output_path = Path::new("C:\\temp\\output.xml");

    let args = build_ntttcp_args(NtttcpRole::Receiver, &plan, None, output_path)
        .expect("Construir args receptor");

    assert!(args.contains(&"-r".to_string()));
    assert!(!args.contains(&"-s".to_string()));
    assert!(args.contains(&"-m".to_string()));
    assert!(args.contains(&"(1,0,7412)".to_string()));
    assert!(args.contains(&"-l".to_string()));
    assert!(args.contains(&"65536".to_string()));
    assert!(args.contains(&"-t".to_string()));
    assert!(args.contains(&"10".to_string()));
    assert!(args.contains(&"-wu".to_string()));
    assert!(args.contains(&"1".to_string()));
    assert!(args.contains(&"-cd".to_string()));
    assert!(args.contains(&"1".to_string()));
    assert!(args.contains(&"-xml".to_string()));
    assert!(args.contains(&"C:\\temp\\output.xml".to_string()));
}

#[test]
fn test_build_allowlisted_args_sender_and_injection_prevention() {
    let plan = BenchmarkPlan::new_standard_tcp(7412);
    let output_path = Path::new("C:\\temp\\output.xml");

    // Éxito con host válido
    let args = build_ntttcp_args(NtttcpRole::Sender, &plan, Some("192.168.1.50"), output_path)
        .expect("Construir args emisor");

    assert!(args.contains(&"-s".to_string()));
    assert!(args.contains(&"(1,*,192.168.1.50,7412)".to_string()));

    // Rechazo si falta host para el emisor
    assert!(build_ntttcp_args(NtttcpRole::Sender, &plan, None, output_path).is_err());

    // Rechazo ante inyección de argumentos o caracteres no permitidos en el host
    let malicious_host = "192.168.1.50; rm -rf";
    assert!(build_ntttcp_args(NtttcpRole::Sender, &plan, Some(malicious_host), output_path).is_err());
}

#[test]
fn test_parse_udp_sender_and_receiver_fixtures() {
    let sender_xml = include_str!("../../../../tests/fixtures/ntttcp/udp_sender.xml");
    let sender_res = parse_ntttcp_xml(sender_xml).expect("Parsear XML UDP emisor");
    assert_eq!(sender_res.role, NtttcpRole::Sender);
    assert_eq!(sender_res.total_buffers, 100_000);
    assert_eq!(sender_res.packets_sent, Some(100_000));
    assert_eq!(sender_res.packets_received, None);

    let receiver_xml = include_str!("../../../../tests/fixtures/ntttcp/udp_receiver.xml");
    let receiver_res = parse_ntttcp_xml(receiver_xml).expect("Parsear XML UDP receptor");
    assert_eq!(receiver_res.role, NtttcpRole::Receiver);
    assert_eq!(receiver_res.total_buffers, 99_500);
    assert_eq!(receiver_res.packets_sent, None);
    assert_eq!(receiver_res.packets_received, Some(99_500));
}

#[test]
fn test_build_allowlisted_args_udp() {
    let mut plan = BenchmarkPlan::new_standard_tcp(5001);
    plan.protocol = crate::model::plan::BenchmarkProtocol::Udp;
    plan.udp_packet_size_bytes = Some(1472);

    let output_path = Path::new("C:\\temp\\output_udp.xml");
    let args = build_ntttcp_args(NtttcpRole::Sender, &plan, Some("192.168.1.50"), output_path)
        .expect("Construir args emisor UDP");

    assert!(args.contains(&"-u".to_string()));
    assert!(args.contains(&"-l".to_string()));
    assert!(args.contains(&"1472".to_string()));
}

