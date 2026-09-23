use networkbench_lib::logging::{
    format_madrid_human, is_madrid_dst, sanitize_value, LogEvent, LogLevel, LogOrigin, Logger,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[test]
fn test_log_level_parsing_and_invalid_values() {
    assert_eq!(LogLevel::parse("trace"), Some(LogLevel::Trace));
    assert_eq!(LogLevel::parse("DEBUG"), Some(LogLevel::Debug));
    assert_eq!(LogLevel::parse("info"), Some(LogLevel::Info));
    assert_eq!(LogLevel::parse("warn"), Some(LogLevel::Warn));
    assert_eq!(LogLevel::parse("warning"), Some(LogLevel::Warn));
    assert_eq!(LogLevel::parse("error"), Some(LogLevel::Error));

    // Valores inválidos deben fallar y devolver None
    assert_eq!(LogLevel::parse("unknown"), None);
    assert_eq!(LogLevel::parse(""), None);
    assert_eq!(LogLevel::parse("123"), None);
}

#[test]
fn test_log_level_filtering() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_log_{}", uuid::Uuid::new_v4()));
    let logger = Logger::new(tmp_dir.clone(), LogLevel::Warn);

    let mk_event = |level: LogLevel, msg: &str| LogEvent {
        schema_version: 1,
        timestamp: "2026-09-21 12:00:00.000 +02:00".to_string(),
        level,
        origin: LogOrigin::Backend,
        module: "test".to_string(),
        event_code: "EVT".to_string(),
        message: msg.to_string(),
        error_code: None,
        duration_ms: None,
        diagnostic_id: None,
        safe_params: HashMap::new(),
    };

    // Debug e Info deben ser filtrados
    logger.log(mk_event(LogLevel::Debug, "mensaje debug filtrado"));
    logger.log(mk_event(LogLevel::Info, "mensaje info filtrado"));

    let log_file = tmp_dir.join("networkbench.log");
    assert!(!log_file.exists(), "No debe crear el archivo si todos los logs son filtrados");

    // Warn y Error deben escribirse
    logger.log(mk_event(LogLevel::Warn, "alerta visible"));
    logger.log(mk_event(LogLevel::Error, "error visible"));

    assert!(log_file.exists());
    let content = fs::read_to_string(&log_file).unwrap();
    assert!(!content.contains("mensaje debug filtrado"));
    assert!(!content.contains("mensaje info filtrado"));
    assert!(content.contains("alerta visible"));
    assert!(content.contains("error visible"));

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_sensitive_data_redaction() {
    assert_eq!(sanitize_value("secretKey", "xyz123"), "[REDACTED]");
    assert_eq!(sanitize_value("authToken", "token_abc"), "[REDACTED]");
    assert_eq!(sanitize_value("userPassword", "p@ssword"), "[REDACTED]");
    assert_eq!(sanitize_value("sessionId", "session-789"), "[REDACTED]");
    assert_eq!(sanitize_value("peerFingerprint", "sha256:abcd"), "[REDACTED]");
    assert_eq!(sanitize_value("privateCert", "cert_content"), "[REDACTED]");

    // Parámetros seguros permitidos
    assert_eq!(sanitize_value("interfaceName", "Ethernet0"), "Ethernet0");
    assert_eq!(sanitize_value("durationSec", "10"), "10");
}

#[test]
fn test_madrid_timezone_and_dst_transitions() {
    // 2026 no bisiesto.
    // 15 de enero de 2026 a las 12:00:00 UTC (invierno -> CET UTC+1)
    // 1768478400 segundos unix
    let winter_secs = 1768478400u64;
    assert!(!is_madrid_dst(winter_secs));
    let winter_time = SystemTime::UNIX_EPOCH + Duration::from_secs(winter_secs);
    let winter_str = format_madrid_human(winter_time);
    assert!(winter_str.contains("+01:00"));
    assert!(winter_str.contains("13:00:00")); // 12h UTC + 1h = 13h CET

    // 15 de julio de 2026 a las 12:00:00 UTC (verano -> CEST UTC+2)
    // 1784116800 segundos unix
    let summer_secs = 1784116800u64;
    assert!(is_madrid_dst(summer_secs));
    let summer_time = SystemTime::UNIX_EPOCH + Duration::from_secs(summer_secs);
    let summer_str = format_madrid_human(summer_time);
    assert!(summer_str.contains("+02:00"));
    assert!(summer_str.contains("14:00:00")); // 12h UTC + 2h = 14h CEST

    // Cambio de hora marzo 2026: domingo 29 de marzo de 2026
    // 00:59:59 UTC -> aún CET UTC+1
    let march_before = 1774745999u64;
    assert!(!is_madrid_dst(march_before));
    let str_before_m = format_madrid_human(SystemTime::UNIX_EPOCH + Duration::from_secs(march_before));
    assert!(str_before_m.contains("+01:00"));

    // 01:00:00 UTC -> CEST UTC+2
    let march_after = 1774746000u64;
    assert!(is_madrid_dst(march_after));
    let str_after_m = format_madrid_human(SystemTime::UNIX_EPOCH + Duration::from_secs(march_after));
    assert!(str_after_m.contains("+02:00"));

    // Cambio de hora octubre 2026: domingo 25 de octubre de 2026
    // 00:59:59 UTC -> aún CEST UTC+2
    let oct_before = 1792889999u64;
    assert!(is_madrid_dst(oct_before));
    let str_before_o = format_madrid_human(SystemTime::UNIX_EPOCH + Duration::from_secs(oct_before));
    assert!(str_before_o.contains("+02:00"));

    // 01:00:00 UTC -> CET UTC+1
    let oct_after = 1792890000u64;
    assert!(!is_madrid_dst(oct_after));
    let str_after_o = format_madrid_human(SystemTime::UNIX_EPOCH + Duration::from_secs(oct_after));
    assert!(str_after_o.contains("+01:00"));
}

#[test]
fn test_rotation_and_max_files() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_rot_{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&tmp_dir);

    let logger = Logger::new(tmp_dir.clone(), LogLevel::Info);

    // Simular rotación con límite bajo para prueba (max 5 archivos, 500 bytes totales)
    for i in 0..15 {
        let rotated_file = tmp_dir.join(format!("networkbench-{}.log", 1000 + i));
        fs::write(&rotated_file, "log content line\n").unwrap();
    }

    let _ = logger.rotate_if_needed(&tmp_dir, 5, 500);

    let entries: Vec<_> = fs::read_dir(&tmp_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("networkbench-"))
        .collect();

    // No debe haber más de 5 archivos rotados
    assert!(entries.len() <= 5);

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_sink_failure_resilience_and_dropped_counter() {
    // Si la ruta no se puede escribir (ej. apuntar a un archivo existente como directorio)
    let invalid_path = PathBuf::from("NUL_OR_INVALID_DEVICE:\\nonexistent_dir\\test");
    let logger = Logger::new(invalid_path, LogLevel::Trace);

    let event = LogEvent {
        schema_version: 1,
        timestamp: "2026-09-21 12:00:00.000 +02:00".to_string(),
        level: LogLevel::Error,
        origin: LogOrigin::Backend,
        module: "sink_test".to_string(),
        event_code: "ERR".to_string(),
        message: "sink test failure".to_string(),
        error_code: None,
        duration_ms: None,
        diagnostic_id: None,
        safe_params: HashMap::new(),
    };

    // Esto no debe lanzar pánico ni abortar
    logger.log(event);
    assert_eq!(logger.dropped_count(), 1);
}
