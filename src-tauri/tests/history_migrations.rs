use networkbench_lib::errors::ErrorCode;
use networkbench_lib::history::database::Database;
use rusqlite::Connection;
use std::fs;

#[test]
fn test_database_initialization_and_pragmas() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_db_{}", uuid::Uuid::new_v4()));
    let db_path = tmp_dir.join("history.db");

    let db = Database::open(db_path.clone()).expect("debe inicializar la base de datos");

    let lock = db.connection().lock().unwrap();

    // Comprobar pragmas
    let journal_mode: String = lock
        .query_row("PRAGMA journal_mode;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");

    let foreign_keys: i32 = lock
        .query_row("PRAGMA foreign_keys;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(foreign_keys, 1);

    let user_version: u32 = lock
        .query_row("PRAGMA user_version;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(user_version, networkbench_lib::history::migrations::CURRENT_SCHEMA_VERSION);

    // Comprobar que las tablas existen
    let count: i32 = lock
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('peers', 'sessions', 'samples');",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 3);

    drop(lock);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_database_foreign_key_enforcement() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_fk_{}", uuid::Uuid::new_v4()));
    let db_path = tmp_dir.join("history.db");

    let db = Database::open(db_path.clone()).expect("debe inicializar");
    let lock = db.connection().lock().unwrap();

    // Inserción con peer_id inexistente debe fallar si viola la foreign key
    let res = lock.execute(
        "INSERT INTO sessions (id, created_at, peer_id, status, protocol, streams, duration_seconds)
         VALUES ('s1', '2026-09-21T12:00:00Z', 'nonexistent_peer', 'completed', 'tcp', 1, 10);",
        [],
    );
    assert!(res.is_err(), "debe fallar por foreign key violation");

    // Inserción con peer_id NULL sí es válida (ON DELETE SET NULL)
    let res_null = lock.execute(
        "INSERT INTO sessions (id, created_at, peer_id, status, protocol, streams, duration_seconds)
         VALUES ('s2', '2026-09-21T12:00:00Z', NULL, 'completed', 'tcp', 1, 10);",
        [],
    );
    assert!(res_null.is_ok());

    drop(lock);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_database_rejects_future_schema() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_future_{}", uuid::Uuid::new_v4()));
    let db_path = tmp_dir.join("history.db");
    let _ = fs::create_dir_all(&tmp_dir);

    // Creamos una BD manual con versión futura
    {
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("PRAGMA user_version = 999;", []).unwrap();
    }

    // Database::open debe rechazarla sin modificarla
    let res = Database::open(db_path.clone());
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert_eq!(err.code, ErrorCode::VersionIncompatible);
    assert_eq!(err.message_key, "errors.NB-VERSION-001");

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_database_backup_operation() {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_backup_{}", uuid::Uuid::new_v4()));
    let db_path = tmp_dir.join("history.db");
    let backup_path = tmp_dir.join("history_backup.db");

    let db = Database::open(db_path.clone()).expect("debe inicializar");

    // Insertar un registro
    {
        let lock = db.connection().lock().unwrap();
        lock.execute(
            "INSERT INTO peers (id, fingerprint, display_name, first_seen_at, last_seen_at)
             VALUES ('p1', 'sha256:abc', 'PC1', '2026-09-21', '2026-09-21');",
            [],
        )
        .unwrap();
    }

    // Ejecutar backup
    db.backup_to(&backup_path).expect("backup debe completarse con éxito");
    assert!(backup_path.exists());

    // Verificar contenido en el backup
    let bkp_conn = Connection::open(&backup_path).unwrap();
    let name: String = bkp_conn
        .query_row("SELECT display_name FROM peers WHERE id='p1';", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(name, "PC1");

    let _ = fs::remove_dir_all(&tmp_dir);
}
