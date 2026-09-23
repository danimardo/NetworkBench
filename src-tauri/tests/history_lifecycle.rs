use networkbench_lib::errors::ErrorCode;
use networkbench_lib::history::database::Database;
use networkbench_lib::history::delete::{
    confirm_delete, preview_delete, DeleteTarget, DeleteTokenStore,
};
use networkbench_lib::history::migrations::CURRENT_SCHEMA_VERSION;
use networkbench_lib::history::sessions::{
    get_session_by_id, insert_session_idempotent, SampleRecord, SessionRecord,
};
use networkbench_lib::history::upsert_peer;
use networkbench_lib::model::peer::Peer;
use rusqlite::Connection;
use std::fs;
use uuid::Uuid;

fn create_temp_db_dir() -> (std::path::PathBuf, std::path::PathBuf) {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_lifecycle_{}", Uuid::new_v4()));
    let db_path = tmp_dir.join("history.db");
    let _ = fs::create_dir_all(&tmp_dir);
    (tmp_dir, db_path)
}

#[test]
fn test_reboot_persistence_and_pragmas() {
    let (tmp_dir, db_path) = create_temp_db_dir();
    let session_id = Uuid::new_v4();

    // 1. Primer arranque: inserción de datos
    {
        let db = Database::open(db_path.clone()).expect("Primer arranque de BD");
        let mut conn = db.connection().lock().unwrap();

        let session = SessionRecord {
            id: session_id,
            created_at: "2026-09-22T08:00:00Z".into(),
            status: "completed".into(),
            protocol: "tcp".into(),
            forward_bps: Some("950000000".into()),
            samples: vec![SampleRecord {
                t_ms: 0,
                direction: "forward".into(),
                bps: "950000000".into(),
                cpu_percent: Some(5.0),
                gap: false,
            }],
            ..Default::default()
        };
        insert_session_idempotent(&mut conn, &session).unwrap();
    } // drop db and conn

    // 2. Simular reinicio de la aplicación y reapertura de la misma BD
    {
        let db = Database::open(db_path.clone()).expect("Reinicio y reapertura de BD");
        let conn = db.connection().lock().unwrap();

        // Verificar pragmas tras reapertura
        let journal: String = conn
            .query_row("PRAGMA journal_mode;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(journal.to_lowercase(), "wal");

        let fks: i32 = conn
            .query_row("PRAGMA foreign_keys;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fks, 1);

        let ver: u32 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ver, CURRENT_SCHEMA_VERSION);

        // Verificar persistencia íntegra de la sesión y sus muestras
        let session = get_session_by_id(&conn, &session_id)
            .unwrap()
            .expect("Sesión debe existir tras reinicio");
        assert_eq!(session.id, session_id);
        assert_eq!(session.forward_bps.as_deref(), Some("950000000"));
        assert_eq!(session.samples.len(), 1);
    }

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_ancient_schema_v1_migration_with_automatic_backup() {
    let (tmp_dir, db_path) = create_temp_db_dir();

    // 1. Crear manualmente una base de datos con esquema v1
    {
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            r#"
            PRAGMA user_version = 1;
            CREATE TABLE peers (
                id TEXT PRIMARY KEY,
                fingerprint TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                alias TEXT,
                is_trusted INTEGER NOT NULL DEFAULT 0,
                auto_accept INTEGER NOT NULL DEFAULT 0,
                first_seen_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL
            );
            CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                peer_id TEXT REFERENCES peers(id) ON DELETE SET NULL,
                status TEXT NOT NULL,
                protocol TEXT NOT NULL,
                streams INTEGER NOT NULL,
                duration_seconds INTEGER NOT NULL,
                forward_bps TEXT,
                reverse_bps TEXT,
                is_partial INTEGER NOT NULL DEFAULT 0,
                diagnostic_verdict TEXT
            );
            CREATE TABLE samples (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                t_ms INTEGER NOT NULL,
                direction TEXT NOT NULL,
                bps TEXT NOT NULL,
                cpu_percent REAL,
                gap INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO sessions (id, created_at, status, protocol, streams, duration_seconds)
            VALUES ('s_old', '2026-09-20T10:00:00Z', 'completed', 'tcp', 1, 10);
            "#,
        )
        .unwrap();
    }

    // 2. Abrir con Database::open: debe crear backup .v1.bak y migrar a CURRENT_SCHEMA_VERSION (2)
    let db = Database::open(db_path.clone()).expect("Migración exitosa");
    let conn = db.connection().lock().unwrap();

    let ver: u32 = conn
        .query_row("PRAGMA user_version;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(ver, CURRENT_SCHEMA_VERSION);

    // Verificar que el backup automático .v1.bak existe
    let backup_file = db_path.with_extension("v1.bak");
    assert!(backup_file.exists(), "Debe haberse creado la copia de seguridad v1.bak antes de migrar");

    // Verificar que la columna client_interface agregada en v2 existe y es legible
    let (s_id, client_iface): (String, Option<String>) = conn
        .query_row(
            "SELECT id, client_interface FROM sessions WHERE id = 's_old';",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(s_id, "s_old");
    assert_eq!(client_iface, None);

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_future_schema_rejected() {
    let (tmp_dir, db_path) = create_temp_db_dir();

    // Crear base de datos con versión futura 999
    {
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("PRAGMA user_version = 999;", []).unwrap();
    }

    let res = Database::open(db_path);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert_eq!(err.code, ErrorCode::VersionIncompatible);
    assert_eq!(err.message_key, "errors.NB-VERSION-001");

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_transactional_delete_preview_token_and_cascade() {
    let (tmp_dir, db_path) = create_temp_db_dir();
    let db = Database::open(db_path).unwrap();
    let mut conn = db.connection().lock().unwrap();
    let token_store = DeleteTokenStore::new();

    let peer1_id = Uuid::new_v4();
    let peer2_id = Uuid::new_v4();

    let peer1 = Peer::new(
        peer1_id,
        "Peer-1".into(),
        "1111111111111111111111111111111111111111111111111111111111111111".into(),
        vec!["192.168.1.1:7411".into()],
    )
    .unwrap();
    upsert_peer(&conn, &peer1).unwrap();

    let peer2 = Peer::new(
        peer2_id,
        "Peer-2".into(),
        "2222222222222222222222222222222222222222222222222222222222222222".into(),
        vec!["192.168.1.2:7411".into()],
    )
    .unwrap();
    upsert_peer(&conn, &peer2).unwrap();

    // Sesión 1 con Peer 1 + 2 muestras
    let s1 = Uuid::new_v4();
    insert_session_idempotent(
        &mut conn,
        &SessionRecord {
            id: s1,
            created_at: "2026-09-01T12:00:00Z".into(),
            peer_id: Some(peer1_id),
            samples: vec![
                SampleRecord { t_ms: 0, direction: "forward".into(), bps: "100".into(), cpu_percent: None, gap: false },
                SampleRecord { t_ms: 500, direction: "forward".into(), bps: "100".into(), cpu_percent: None, gap: false },
            ],
            ..Default::default()
        },
    )
    .unwrap();

    // Sesión 2 con Peer 1 + 1 muestra
    let s2 = Uuid::new_v4();
    insert_session_idempotent(
        &mut conn,
        &SessionRecord {
            id: s2,
            created_at: "2026-09-10T12:00:00Z".into(),
            peer_id: Some(peer1_id),
            samples: vec![
                SampleRecord { t_ms: 0, direction: "forward".into(), bps: "100".into(), cpu_percent: None, gap: false },
            ],
            ..Default::default()
        },
    )
    .unwrap();

    // Sesión 3 con Peer 2 + 1 muestra
    let s3 = Uuid::new_v4();
    insert_session_idempotent(
        &mut conn,
        &SessionRecord {
            id: s3,
            created_at: "2026-09-20T12:00:00Z".into(),
            peer_id: Some(peer2_id),
            samples: vec![
                SampleRecord { t_ms: 0, direction: "forward".into(), bps: "100".into(), cpu_percent: None, gap: false },
            ],
            ..Default::default()
        },
    )
    .unwrap();

    // 1. Preview de borrado de Peer 1
    let preview = preview_delete(&conn, &DeleteTarget::ByPeer(peer1_id), &token_store).unwrap();
    assert_eq!(preview.affected_sessions, 2);
    assert_eq!(preview.affected_samples, 3);
    assert!(!preview.confirmation_token.is_empty());

    // 2. Intento de confirmación con token inválido -> rechazo
    let fake_res = confirm_delete(&mut conn, "fake-token-123", &token_store).unwrap();
    assert_eq!(fake_res, None);

    // 3. Confirmación con token válido -> borrado exitoso
    let ok_res = confirm_delete(&mut conn, &preview.confirmation_token, &token_store)
        .unwrap()
        .expect("Borrado confirmado");
    assert_eq!(ok_res.deleted_sessions, 2);

    // 4. Segundo intento con el mismo token -> token de un solo uso consumido
    let consumed_res = confirm_delete(&mut conn, &preview.confirmation_token, &token_store).unwrap();
    assert_eq!(consumed_res, None);

    // 5. Verificar que la sesión de Peer 2 sigue intacta con sus muestras
    let remaining_sessions: i64 = conn.query_row("SELECT COUNT(*) FROM sessions;", [], |r| r.get(0)).unwrap();
    assert_eq!(remaining_sessions, 1);

    let remaining_samples: i64 = conn.query_row("SELECT COUNT(*) FROM samples;", [], |r| r.get(0)).unwrap();
    assert_eq!(remaining_samples, 1);

    let p2_session = get_session_by_id(&conn, &s3).unwrap();
    assert!(p2_session.is_some());

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}
