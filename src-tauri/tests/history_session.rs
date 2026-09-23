use networkbench_lib::history::database::Database;
use networkbench_lib::history::{
    get_peer_by_fingerprint, get_session_by_id, insert_session_idempotent, upsert_peer,
    SampleRecord, SessionRecord,
};
use networkbench_lib::model::peer::{Peer, TrustState};
use std::fs;
use uuid::Uuid;

fn create_temp_db() -> (Database, std::path::PathBuf) {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_session_{}", Uuid::new_v4()));
    let db_path = tmp_dir.join("history.db");
    let db = Database::open(db_path).expect("Inicializar base de datos de test");
    (db, tmp_dir)
}

#[test]
fn test_peer_persistence_and_upsert() {
    let (db, tmp_dir) = create_temp_db();
    let conn = db.connection().lock().unwrap();

    let p_id = Uuid::new_v4();
    let fp = "1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff";
    let mut peer = Peer::new(
        p_id,
        "Servidor-Pruebas".into(),
        fp.into(),
        vec!["192.168.1.100:7411".into()],
    )
    .unwrap();
    peer.alias = Some("Mi Servidor".into());
    peer.trust_state = TrustState::Trusted;

    // 1. Insertar peer
    upsert_peer(&conn, &peer).expect("Insertar peer");

    let loaded = get_peer_by_fingerprint(&conn, fp).expect("Cargar peer").expect("Peer debe existir");
    assert_eq!(loaded.instance_id, p_id);
    assert_eq!(loaded.display_name, "Servidor-Pruebas");
    assert_eq!(loaded.alias.as_deref(), Some("Mi Servidor"));
    assert_eq!(loaded.trust_state, TrustState::Trusted);

    // 2. Upsert con nuevo display_name y last_seen
    peer.display_name = "Servidor-Renombrado".into();
    peer.last_seen = "2026-09-21T21:00:00Z".into();
    upsert_peer(&conn, &peer).expect("Actualizar peer");

    let updated = get_peer_by_fingerprint(&conn, fp).expect("Cargar peer actualizado").unwrap();
    assert_eq!(updated.display_name, "Servidor-Renombrado");
    assert_eq!(updated.alias.as_deref(), Some("Mi Servidor")); // Mantiene alias
    assert_eq!(updated.trust_state, TrustState::Trusted); // Mantiene confianza

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_session_persistence_idempotence_and_samples() {
    let (db, tmp_dir) = create_temp_db();
    let mut conn = db.connection().lock().unwrap();

    let session_id = Uuid::new_v4();
    let peer_id = Uuid::new_v4();

    let peer = Peer::new(
        peer_id,
        "Peer-Asociado".into(),
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        vec!["192.168.1.20:7411".into()],
    )
    .unwrap();
    upsert_peer(&conn, &peer).expect("Insertar peer para sesión");

    let samples = vec![
        SampleRecord {
            t_ms: 0,
            direction: "forward".into(),
            bps: "940000000".into(),
            cpu_percent: Some(3.5),
            gap: false,
        },
        SampleRecord {
            t_ms: 500,
            direction: "forward".into(),
            bps: "950000000".into(),
            cpu_percent: Some(4.0),
            gap: false,
        },
        SampleRecord {
            t_ms: 1000,
            direction: "forward".into(),
            bps: "0".into(),
            cpu_percent: None,
            gap: true, // hueco explícito
        },
    ];

    let session = SessionRecord {
        id: session_id,
        created_at: "2026-09-21T20:10:00Z".into(),
        peer_id: Some(peer_id),
        status: "completed".into(),
        protocol: "tcp".into(),
        streams: 1,
        duration_seconds: 10,
        forward_bps: Some("945000000".into()),
        reverse_bps: Some("942000000".into()),
        is_partial: false,
        diagnostic_verdict: Some("{\"level\":\"ok\"}".into()),
        samples,
        ..Default::default()
    };

    // 1. Inserción inicial
    insert_session_idempotent(&mut conn, &session).expect("Inserción de sesión");

    let retrieved = get_session_by_id(&conn, &session_id)
        .expect("Consulta por id")
        .expect("Sesión debe existir");

    assert_eq!(retrieved.id, session_id);
    assert_eq!(retrieved.status, "completed");
    assert_eq!(retrieved.forward_bps.as_deref(), Some("945000000"));
    assert_eq!(retrieved.reverse_bps.as_deref(), Some("942000000"));
    assert_eq!(retrieved.samples.len(), 3);
    assert_eq!(retrieved.samples[2].gap, true);

    // 2. Re-inserción idempotente con actualización
    let mut updated_session = session.clone();
    updated_session.forward_bps = Some("948000000".into());
    insert_session_idempotent(&mut conn, &updated_session).expect("Re-inserción idempotente");

    let re_retrieved = get_session_by_id(&conn, &session_id).expect("Consulta").unwrap();
    assert_eq!(re_retrieved.forward_bps.as_deref(), Some("948000000"));
    // Las muestras se deben haber reescrito limpiamente, no duplicado a 6
    assert_eq!(re_retrieved.samples.len(), 3);

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_cascade_deletion_of_samples() {
    let (db, tmp_dir) = create_temp_db();
    let mut conn = db.connection().lock().unwrap();

    let session_id = Uuid::new_v4();
    let session = SessionRecord {
        id: session_id,
        created_at: "2026-09-21T20:10:00Z".into(),
        peer_id: None,
        status: "completed".into(),
        protocol: "tcp".into(),
        streams: 1,
        duration_seconds: 10,
        forward_bps: Some("900000000".into()),
        reverse_bps: None,
        is_partial: true,
        diagnostic_verdict: None,
        samples: vec![SampleRecord {
            t_ms: 0,
            direction: "forward".into(),
            bps: "900000000".into(),
            cpu_percent: None,
            gap: false,
        }],
        ..Default::default()
    };

    insert_session_idempotent(&mut conn, &session).expect("Insertar sesión");

    // Borrar la sesión
    conn.execute("DELETE FROM sessions WHERE id = ?1", rusqlite::params![session_id.to_string()])
        .expect("Borrar sesión");

    // Verificar que las muestras se eliminaron por cascade
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM samples WHERE session_id = ?1",
            rusqlite::params![session_id.to_string()],
            |r| r.get(0),
        )
        .expect("Contar muestras");

    assert_eq!(count, 0, "Las muestras deben eliminarse en cascada con la sesión");

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}
