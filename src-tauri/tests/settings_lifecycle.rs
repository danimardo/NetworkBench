use networkbench_lib::errors::ErrorCode;
use networkbench_lib::logging::LogLevel;
use networkbench_lib::settings::{SettingsStore, ThemeMode};
use std::fs;

#[test]
fn test_settings_lifecycle_missing_file_creates_clean_default() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_lc_missing_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");

    assert!(!prefs_file.exists());
    let store = SettingsStore::new(prefs_file.clone());
    let prefs = store.get();

    assert_eq!(prefs.theme, ThemeMode::Dark);
    assert_eq!(prefs.locale, "es");
    assert!(!prefs.reduce_motion);
    assert_eq!(prefs.log_level, LogLevel::Warn);
    assert!(!prefs.auto_accept_trusted);
    assert!(!prefs.autostart);
    assert!(!prefs.minimize_to_tray);
    assert!(prefs.mdns_enabled);
    assert!(prefs_file.exists());

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_settings_lifecycle_corrupt_file_quarantine() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_lc_corrupt_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");
    let _ = fs::create_dir_all(&tmp_dir);

    fs::write(&prefs_file, "INVALID_NON_JSON_CONTENT{{{:::").unwrap();

    let store = SettingsStore::new(prefs_file.clone());
    let prefs = store.get();
    assert_eq!(prefs.theme, ThemeMode::Dark);

    // Debe existir un fichero con .corrupt.
    let corrupt_found = fs::read_dir(&tmp_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().contains("corrupt"));
    assert!(
        corrupt_found,
        "El archivo corrupto debe aislarse en cuarentena"
    );

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_settings_lifecycle_schema_migration_preserves_existing() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_lc_mig_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");
    let _ = fs::create_dir_all(&tmp_dir);

    // JSON heredado de versión anterior sin campos de Fase 9 (autostart, minimizeToTray, mdnsEnabled)
    let legacy_json = r#"{
        "schemaVersion": 1,
        "theme": "light",
        "locale": "en",
        "reduceMotion": true,
        "logLevel": "info",
        "autoAcceptTrusted": true,
        "customControlPort": 5202
    }"#;
    fs::write(&prefs_file, legacy_json).unwrap();

    let store = SettingsStore::new(prefs_file.clone());
    let prefs = store.get();

    assert_eq!(prefs.theme, ThemeMode::Light);
    assert_eq!(prefs.locale, "en");
    assert!(prefs.reduce_motion);
    assert_eq!(prefs.log_level, LogLevel::Info);
    assert!(prefs.auto_accept_trusted);
    assert_eq!(prefs.custom_control_port, Some(5202));
    // Defaults para campos nuevos
    assert!(!prefs.autostart);
    assert!(!prefs.minimize_to_tray);
    assert!(prefs.mdns_enabled);

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_settings_lifecycle_future_schema_returns_memory_defaults_and_preserves_disk() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_lc_future_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");
    let _ = fs::create_dir_all(&tmp_dir);

    let future_json = r#"{
        "schemaVersion": 999,
        "theme": "light",
        "locale": "fr",
        "reduceMotion": true,
        "futureFeaturePayload": { "enabled": true }
    }"#;
    fs::write(&prefs_file, future_json).unwrap();

    let store = SettingsStore::new(prefs_file.clone());
    let prefs = store.get();
    // Retorna defaults seguros en memoria sin romper
    assert_eq!(prefs.schema_version, 1);
    assert_eq!(prefs.locale, "es");

    // En disco el archivo sigue intacto con schemaVersion 999
    let disk_content = fs::read_to_string(&prefs_file).unwrap();
    assert!(disk_content.contains("999"));
    assert!(disk_content.contains("futureFeaturePayload"));

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_settings_lifecycle_session_active_rejects_critical_changes() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_lc_active_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");

    let store = SettingsStore::new(prefs_file.clone());

    // 1. Intentar cambiar custom_control_port durante sesión activa
    let res = store.validate_and_update(true, |p| {
        p.custom_control_port = Some(9000);
    });
    assert!(
        res.is_err(),
        "Debe rechazar cambio de puerto durante sesión activa"
    );
    let err = res.unwrap_err();
    assert_eq!(err.code, ErrorCode::PeerBusy);

    // 2. Intentar cambiar mdns_enabled durante sesión activa
    let res2 = store.validate_and_update(true, |p| {
        p.mdns_enabled = false;
    });
    assert!(
        res2.is_err(),
        "Debe rechazar cambio de mDNS durante sesión activa"
    );

    // 3. Intentar cambiar auto_accept_trusted durante sesión activa
    let res3 = store.validate_and_update(true, |p| {
        p.auto_accept_trusted = true;
    });
    assert!(
        res3.is_err(),
        "Debe rechazar cambio de autoaceptación durante sesión activa"
    );

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_settings_lifecycle_session_active_allows_presentation_changes() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_lc_pres_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");

    let store = SettingsStore::new(prefs_file.clone());

    // Cambiar tema, idioma, reducir movimiento y minimizar a bandeja durante sesión activa debe ser permitido
    let res = store.validate_and_update(true, |p| {
        p.theme = ThemeMode::Light;
        p.locale = "en".to_string();
        p.reduce_motion = true;
        p.minimize_to_tray = true;
    });

    assert!(
        res.is_ok(),
        "Cambios visuales y de bandeja deben permitirse durante sesión activa"
    );
    let current = store.get();
    assert_eq!(current.theme, ThemeMode::Light);
    assert_eq!(current.locale, "en");
    assert!(current.reduce_motion);
    assert!(current.minimize_to_tray);

    let _ = fs::remove_dir_all(&tmp_dir);
}
