use super::*;
use crate::model::plan::BenchmarkPlan;
use std::path::Path;

// Salida auténtica de NTTTCP 5.40, capturada el 2026-09-23 en una medición en bucle
// local (`-m 1,*,127.0.0.1 -t 3`). Los fixtures `receiver_success.xml` y
// `sender_success.xml`, pese a llamarse «real» las pruebas que los usan, son
// sintéticos y contienen un elemento `<role>` que el motor no emite.

#[test]
fn test_xml_autentico_de_receptor() {
    let xml = include_str!("../../../../tests/fixtures/ntttcp/real_5.40_receiver.xml");
    let res = parse_ntttcp_xml(xml).expect("parsear salida real de receptor");

    // El rol viene del elemento raíz `<ntttcpr>`, no de ningún `<role>`.
    assert_eq!(res.role, NtttcpRole::Receiver);
    // `total_bytes` llega en MB con decimales: 5215.3125 MB × 1048576.
    assert_eq!(res.total_bytes, 5_468_651_520);
    assert_eq!(res.total_buffers, 83_445);
    // `realtime` aparece dos veces; vale la del total, no la del hilo (0.000).
    assert!((res.realtime_seconds - 3.018212).abs() < 0.0001);
    // `throughput` aparece con cinco métricas distintas; la buena es Bps -> bits.
    assert_eq!(res.throughput_bps, 14_495_074_024);
    assert_eq!(res.cpu_percent, Some(20.664));
    assert_eq!(res.errors_count, 2);
    assert_eq!(res.packets_sent, Some(234_871));
    assert_eq!(res.packets_received, Some(234_707));
}

#[test]
fn test_xml_autentico_de_emisor() {
    let xml = include_str!("../../../../tests/fixtures/ntttcp/real_5.40_sender.xml");
    let res = parse_ntttcp_xml(xml).expect("parsear salida real de emisor");

    assert_eq!(res.role, NtttcpRole::Sender);
    assert_eq!(res.total_buffers, 83_446);
    assert_eq!(res.errors_count, 2);
    assert!(res.throughput_bps > 0);
}

#[test]
fn test_las_metricas_repetidas_no_se_confunden() {
    // `<throughput>` aparece cinco veces con métricas distintas y `<realtime>` dos
    // veces a distinta profundidad. Un parser que tomara la última ocurrencia leería
    // `buffers/s` como caudal y `0.000` como duración.
    let xml = include_str!("../../../../tests/fixtures/ntttcp/real_5.40_receiver.xml");
    let res = parse_ntttcp_xml(xml).expect("parsear");

    // buffers/s valía 27647.160: si se hubiera colado, el caudal sería absurdo.
    assert_eq!(res.throughput_bps, 14_495_074_024);
    // El `<realtime>` del hilo valía 0.000.
    assert!(res.realtime_seconds > 3.0);
}

#[test]
fn test_parse_corrupted_and_empty_xml() {
    let corrupted = include_str!("../../../../tests/fixtures/ntttcp/corrupted.xml");
    let res_corr = parse_ntttcp_xml(corrupted);
    assert!(res_corr.is_err());
    assert!(matches!(
        res_corr.unwrap_err(),
        NtttcpParseError::MalformedXml(_) | NtttcpParseError::MissingField(_)
    ));

    let empty = include_str!("../../../../tests/fixtures/ntttcp/empty.xml");
    let res_empty = parse_ntttcp_xml(empty);
    assert_eq!(res_empty, Err(NtttcpParseError::EmptyXml));
}

#[test]
fn test_build_allowlisted_args_receiver() {
    let plan = BenchmarkPlan::new_standard_tcp(7412);
    let output_path = Path::new("C:\\temp\\output.xml");

    // El receptor también necesita dirección: la interfaz local a la que se liga.
    assert!(
        build_ntttcp_args(NtttcpRole::Receiver, &plan, None, output_path).is_err(),
        "sin dirección no se puede construir el mapeo"
    );

    let args = build_ntttcp_args(NtttcpRole::Receiver, &plan, Some("127.0.0.1"), output_path)
        .expect("Construir args receptor");

    assert!(args.contains(&"-r".to_string()));
    assert!(!args.contains(&"-s".to_string()));
    assert!(args.contains(&"-m".to_string()));
    // Forma real de NTTTCP 5.40: sin paréntesis y con el puerto en `-p`.
    assert!(args.contains(&"1,*,127.0.0.1".to_string()));
    assert!(args.contains(&"-p".to_string()));
    assert!(args.contains(&"7412".to_string()));
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
    assert!(args.contains(&"1,*,192.168.1.50".to_string()));
    assert!(args.contains(&"-p".to_string()));

    // Rechazo si falta host para el emisor
    assert!(build_ntttcp_args(NtttcpRole::Sender, &plan, None, output_path).is_err());

    // Rechazo ante inyección de argumentos o caracteres no permitidos en el host
    let malicious_host = "192.168.1.50; rm -rf";
    assert!(
        build_ntttcp_args(NtttcpRole::Sender, &plan, Some(malicious_host), output_path).is_err()
    );
}

#[test]
fn test_los_contadores_udp_son_los_del_motor() {
    // Antes `packets_sent` y `packets_received` se rellenaban con `total_buffers`
    // según el rol, no se leían. Eso hacía que la pérdida UDP (V-02) se calculase
    // sobre una cifra inventada. El motor sí emite ambos contadores, en los dos roles.
    let emisor = parse_ntttcp_xml(include_str!(
        "../../../../tests/fixtures/ntttcp/real_5.40_udp_sender.xml"
    ))
    .expect("parsear UDP emisor");
    assert_eq!(emisor.role, NtttcpRole::Sender);
    assert!(emisor.packets_sent.is_some());
    // Un emisor real también informa de lo recibido: no es `None` por ser emisor.
    assert!(emisor.packets_received.is_some());
    assert_ne!(emisor.packets_sent, Some(emisor.total_buffers));

    let receptor = parse_ntttcp_xml(include_str!(
        "../../../../tests/fixtures/ntttcp/real_5.40_udp_receiver.xml"
    ))
    .expect("parsear UDP receptor");
    assert_eq!(receptor.role, NtttcpRole::Receiver);
    assert_eq!(receptor.packets_sent, Some(174_440));
    assert_eq!(receptor.packets_received, Some(174_476));
    assert_eq!(receptor.packets_retransmitted, Some(0));
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
