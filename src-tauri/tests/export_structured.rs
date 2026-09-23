use networkbench_lib::export::csv::{
    export_samples_csv, export_summary_csv, sanitize_csv_cell, SampleCsvRow, UTF8_BOM,
};
use networkbench_lib::export::json::export_sessions_to_json;
use networkbench_lib::model::result::SessionResult;
use uuid::Uuid;

fn load_sample_session() -> SessionResult {
    let fixture_str = include_str!("../../tests/fixtures/export/session_with_canaries.json");
    serde_json::from_str(fixture_str).expect("Cargar fixture de prueba")
}

#[test]
fn test_json_single_session_exact_units_and_schema() {
    let session = load_sample_session();
    let json_output = export_sessions_to_json(&[session.clone()], false).expect("Exportar JSON individual");

    // Debe ser un objeto JSON (no un array)
    assert!(json_output.trim_start().starts_with('{'));
    assert!(json_output.trim_end().ends_with('}'));

    let parsed: serde_json::Value = serde_json::from_str(&json_output).expect("Parsear JSON generado");
    assert_eq!(parsed["schemaVersion"], 1);
    assert_eq!(parsed["sessionId"], session.session_id.to_string());
    assert_eq!(parsed["status"], "completed");

    // Cifras en unidades base intactas
    assert_eq!(parsed["directions"][0]["officialBps"], "948000000");
    assert_eq!(parsed["capacity"]["refBps"], 1000000000);
}

#[test]
fn test_json_multiple_sessions_array_output() {
    let mut session1 = load_sample_session();
    session1.session_id = Uuid::new_v4();
    let mut session2 = load_sample_session();
    session2.session_id = Uuid::new_v4();

    let json_output = export_sessions_to_json(&[session1.clone(), session2.clone()], false)
        .expect("Exportar JSON múltiple");

    // Debe ser un array JSON
    assert!(json_output.trim_start().starts_with('['));
    assert!(json_output.trim_end().ends_with(']'));

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_output).expect("Parsear array JSON");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["sessionId"], session1.session_id.to_string());
    assert_eq!(parsed[1]["sessionId"], session2.session_id.to_string());
}

#[test]
fn test_csv_bom_and_locale_separators_spanish_and_english() {
    let session = load_sample_session();

    // 1. Español: delimitador ';' y decimal ','
    let csv_es = export_summary_csv(&[session.clone()], false, ';', ',')
        .expect("Exportar CSV español");

    assert!(csv_es.starts_with(UTF8_BOM), "El CSV debe comenzar con UTF-8 BOM");
    let lines_es: Vec<&str> = csv_es.lines().collect();
    assert!(lines_es.len() >= 2);
    assert!(lines_es[0].contains("sessionId;fecha;equipoLocal;equipoRemoto;direccion;protocolo;velocidadMbit"));
    // Verificar decimal ',' en los valores formateados
    assert!(lines_es[1].contains("948,00") || lines_es[1].contains("1000,00"));

    // 2. Inglés: delimitador ',' y decimal '.'
    let csv_en = export_summary_csv(&[session.clone()], false, ',', '.')
        .expect("Exportar CSV inglés");

    assert!(csv_en.starts_with(UTF8_BOM), "El CSV debe comenzar con UTF-8 BOM");
    let lines_en: Vec<&str> = csv_en.lines().collect();
    assert!(lines_en[0].contains("sessionId,fecha,equipoLocal,equipoRemoto,direccion,protocolo,velocidadMbit"));
    // Verificar decimal '.' en los valores formateados
    assert!(lines_en[1].contains("948.00") || lines_en[1].contains("1000.00"));
}

#[test]
fn test_csv_formula_injection_sanitization() {
    // Verificar neutralización directa de fórmulas en celdas
    assert_eq!(sanitize_csv_cell("=CMD|'/c calc'!A0"), "'=CMD|'/c calc'!A0");
    assert_eq!(sanitize_csv_cell("+12345"), "'+12345");
    assert_eq!(sanitize_csv_cell("-SUM(A1:A10)"), "'-SUM(A1:A10)");
    assert_eq!(sanitize_csv_cell("@HYPERLINK('evil')"), "'@HYPERLINK('evil')");
    assert_eq!(sanitize_csv_cell("Normal Text"), "Normal Text");

    // Verificar en una sesión con nombres maliciosos
    let mut malicious_session = load_sample_session();
    malicious_session.initiator.display_name = "=SUM(1+1)".to_string();
    malicious_session.responder.display_name = "+34600000000".to_string();

    let csv = export_summary_csv(&[malicious_session], false, ';', ',')
        .expect("Exportar CSV con nombres maliciosos");

    assert!(csv.contains("'=SUM(1+1)"));
    assert!(csv.contains("'+34600000000"));
    assert!(!csv.contains(";=SUM(1+1);"));
    assert!(!csv.contains(";+34600000000;"));
}

#[test]
fn test_csv_samples_export() {
    let session_id = Uuid::new_v4();
    let samples = vec![
        SampleCsvRow {
            session_id,
            direction: "forward".to_string(),
            endpoint: "DESKTOP-CANARY1".to_string(),
            timestamp_ms: 500,
            rx_bps: Some(945_000_000),
            tx_bps: Some(950_000_000),
            cpu_percent: Some(4.5),
        },
        SampleCsvRow {
            session_id,
            direction: "forward".to_string(),
            endpoint: "DESKTOP-CANARY1".to_string(),
            timestamp_ms: 1000,
            rx_bps: Some(948_000_000),
            tx_bps: Some(948_000_000),
            cpu_percent: Some(4.8),
        },
    ];

    let csv_samples = export_samples_csv(&samples, ';', ',').expect("Exportar muestras CSV");
    assert!(csv_samples.starts_with(UTF8_BOM));
    let lines: Vec<&str> = csv_samples.lines().collect();
    assert_eq!(lines[0], format!("{UTF8_BOM}sessionId;direccion;equipo;tMs;rxMbit;txMbit;cpuPct"));
    assert!(lines[1].contains(";forward;DESKTOP-CANARY1;500;945,00;950,00;4,5"));
    assert!(lines[2].contains(";forward;DESKTOP-CANARY1;1000;948,00;948,00;4,8"));
}
