use networkbench_lib::history::database::Database;
use networkbench_lib::ipc::response::{IpcResult, OneTimeTokenStore};
use networkbench_lib::ipc::snapshot::{AppSnapshot, SnapshotManager};
use networkbench_lib::logging::{LogEvent, LogLevel, LogOrigin, Logger, format_madrid_human};
use networkbench_lib::settings::{SettingsStore, ThemeMode};
use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;

#[test]
fn test_foundation_end_to_end_integration() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_foundation_{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&tmp_dir);

    // 1. Logger
    let log_dir = tmp_dir.join("logs");
    let logger = Logger::new(log_dir.clone(), LogLevel::Info);
    let event = LogEvent {
        schema_version: 1,
        timestamp: format_madrid_human(SystemTime::now()),
        level: LogLevel::Info,
        origin: LogOrigin::Backend,
        module: "foundation".to_string(),
        event_code: "INIT".to_string(),
        message: "Fundamentos inicializados correctamente".to_string(),
        error_code: None,
        duration_ms: None,
        diagnostic_id: None,
        safe_params: HashMap::new(),
    };
    logger.log(event);
    assert!(log_dir.join("networkbench.log").exists());

    // 2. Base de datos con WAL y migraciones
    let db_path = tmp_dir.join("history.db");
    let db = Database::open(db_path.clone()).expect("BD debe inicializarse con WAL y esquema v1");
    let lock = db.connection().lock().unwrap();
    let count: i32 = lock
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('peers', 'sessions', 'samples');",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 3);
    drop(lock);

    // 3. Settings con Preferences (Dark theme inicial obligatorio por FR-037)
    let settings_path = tmp_dir.join("settings.json");
    let store = SettingsStore::new(settings_path);
    let prefs = store.get();
    assert_eq!(prefs.theme, ThemeMode::Dark);
    assert_eq!(prefs.locale, "es");

    // 4. Snapshot Manager con revisión monotónica
    let snapshot_mgr = SnapshotManager::new(AppSnapshot {
        revision: 1,
        app_version: "0.1.0".to_string(),
        locale: prefs.locale,
        theme: "dark".to_string(),
        instance_id: uuid::Uuid::new_v4().to_string(),
        instance_name: "TestHost".to_string(),
        is_session_active: false,
        active_session_id: None,
        peers_count: 0,
    });
    assert_eq!(snapshot_mgr.current_revision(), 1);

    let updated_snap = snapshot_mgr.update(|s| {
        s.peers_count = 2;
    });
    assert_eq!(updated_snap.revision, 2);
    assert_eq!(updated_snap.peers_count, 2);

    // 5. Tokens de un solo uso
    let token_store = OneTimeTokenStore::new();
    let token = token_store.issue("session.start", Some("plan_hash_1"), 60);
    assert!(
        token_store
            .consume(&token, "session.start", Some("plan_hash_1"))
            .is_ok()
    );
    // Segundo intento debe ser estrictamente rechazado
    assert!(
        token_store
            .consume(&token, "session.start", Some("plan_hash_1"))
            .is_err()
    );

    // 6. IPC Result Envelope
    let ipc_success = IpcResult::ok(updated_snap);
    assert!(ipc_success.is_ok());

    let _ = fs::remove_dir_all(&tmp_dir);
}
