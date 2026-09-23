use networkbench_lib::logging::LogLevel;
use networkbench_lib::settings::{SettingsStore, ThemeMode};
use std::fs;

#[test]
fn test_settings_initial_defaults() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_prefs_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");

    let store = SettingsStore::new(prefs_file.clone());
    let prefs = store.get();

    // Verificación constitucional y FR-037: Tema oscuro por defecto
    assert_eq!(prefs.theme, ThemeMode::Dark);
    assert_eq!(prefs.locale, "es");
    assert_eq!(prefs.log_level, LogLevel::Warn);
    assert!(!prefs.reduce_motion);
    assert!(!prefs.auto_accept_trusted);

    assert!(prefs_file.exists());

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_settings_atomic_update() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_update_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");

    let store = SettingsStore::new(prefs_file.clone());
    let updated = store
        .update(|p| {
            p.theme = ThemeMode::Light;
            p.locale = "en".to_string();
            p.reduce_motion = true;
        })
        .expect("update atómico debe completarse");

    assert_eq!(updated.theme, ThemeMode::Light);
    assert_eq!(updated.locale, "en");
    assert!(updated.reduce_motion);

    // Reabrir desde disco para verificar persistencia real
    let store2 = SettingsStore::new(prefs_file.clone());
    let reloaded = store2.get();
    assert_eq!(reloaded.theme, ThemeMode::Light);
    assert_eq!(reloaded.locale, "en");
    assert!(reloaded.reduce_motion);

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_settings_corrupt_file_quarantine_and_recovery() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_corrupt_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");
    let _ = fs::create_dir_all(&tmp_dir);

    // Escribir contenido corrupto
    fs::write(&prefs_file, "{ malformed json ::: corrupted").unwrap();

    // Debe aislar el archivo corrupto y recuperar defaults
    let store = SettingsStore::new(prefs_file.clone());
    let recovered = store.get();
    assert_eq!(recovered.theme, ThemeMode::Dark);

    // Verificar que existe el archivo de cuarentena
    let entries: Vec<_> = fs::read_dir(&tmp_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("corrupt"))
        .collect();
    assert_eq!(entries.len(), 1);

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_settings_future_schema_version_preserved() {
    let tmp_dir =
        std::env::temp_dir().join(format!("nb_test_future_pref_{}", uuid::Uuid::new_v4()));
    let prefs_file = tmp_dir.join("settings.json");
    let _ = fs::create_dir_all(&tmp_dir);

    // Escribir JSON con schemaVersion futuro 99
    let future_json = r#"{
        "schemaVersion": 99,
        "theme": "light",
        "locale": "es",
        "reduceMotion": false,
        "logLevel": "info",
        "autoAcceptTrusted": false,
        "customControlPort": null
    }"#;
    fs::write(&prefs_file, future_json).unwrap();

    // Al cargar, no debe sobrescribir el archivo en disco de versión futura
    let store = SettingsStore::new(prefs_file.clone());
    let prefs = store.get();
    // En memoria retorna defaults seguros
    assert_eq!(prefs.schema_version, 1);

    // En disco el archivo original con schemaVersion 99 no debe haber sido destruido
    let disk_content = fs::read_to_string(&prefs_file).unwrap();
    assert!(disk_content.contains("\"schemaVersion\": 99"));

    let _ = fs::remove_dir_all(&tmp_dir);
}
