use networkbench_lib::history::database::Database;
use networkbench_lib::history::{
    get_session_detail, insert_session_idempotent, query_history, upsert_peer, HistoryFilter,
    Pagination, SampleRecord, SessionRecord,
};
use networkbench_lib::model::peer::Peer;
use std::fs;
use uuid::Uuid;

fn setup_test_db() -> (Database, std::path::PathBuf) {
    let tmp_dir = std::env::temp_dir().join(format!("nb_test_queries_{}", Uuid::new_v4()));
    let db_path = tmp_dir.join("history.db");
    let db = Database::open(db_path).expect("Abrir base de datos de test");
    (db, tmp_dir)
}

#[test]
fn test_pagination_and_total_count() {
    let (db, tmp_dir) = setup_test_db();
    let mut conn = db.connection().lock().unwrap();

    // Insertar 25 sesiones
    for i in 0..25 {
        let session = SessionRecord {
            id: Uuid::new_v4(),
            created_at: format!("2026-09-21T10:{:02}:00Z", i),
            peer_id: None,
            status: "completed".into(),
            protocol: "tcp".into(),
            streams: 1,
            duration_seconds: 10,
            forward_bps: Some("940000000".into()),
            reverse_bps: Some("930000000".into()),
            is_partial: false,
            diagnostic_verdict: Some(r#"{"level":"ok"}"#.into()),
            ..Default::default()
        };
        insert_session_idempotent(&mut conn, &session).unwrap();
    }

    // Página 1: limit 10, offset 0
    let page1 = query_history(
        &conn,
        &HistoryFilter::default(),
        &Pagination { limit: 10, offset: 0 },
    )
    .unwrap();
    assert_eq!(page1.total_count, 25);
    assert_eq!(page1.items.len(), 10);
    assert!(page1.has_more);

    // Página 2: limit 10, offset 10
    let page2 = query_history(
        &conn,
        &HistoryFilter::default(),
        &Pagination { limit: 10, offset: 10 },
    )
    .unwrap();
    assert_eq!(page2.total_count, 25);
    assert_eq!(page2.items.len(), 10);
    assert!(page2.has_more);

    // Página 3: limit 10, offset 20
    let page3 = query_history(
        &conn,
        &HistoryFilter::default(),
        &Pagination { limit: 10, offset: 20 },
    )
    .unwrap();
    assert_eq!(page3.total_count, 25);
    assert_eq!(page3.items.len(), 5);
    assert!(!page3.has_more);

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_filters_by_peer_verdict_protocol_and_date() {
    let (db, tmp_dir) = setup_test_db();
    let mut conn = db.connection().lock().unwrap();

    let peer1_id = Uuid::new_v4();
    let peer1 = Peer::new(
        peer1_id,
        "Servidor-Alpha".into(),
        "1111111111111111111111111111111111111111111111111111111111111111".into(),
        vec!["192.168.1.10:7411".into()],
    )
    .unwrap();
    upsert_peer(&conn, &peer1).unwrap();

    let peer2_id = Uuid::new_v4();
    let peer2 = Peer::new(
        peer2_id,
        "Servidor-Beta".into(),
        "2222222222222222222222222222222222222222222222222222222222222222".into(),
        vec!["192.168.1.20:7411".into()],
    )
    .unwrap();
    upsert_peer(&conn, &peer2).unwrap();

    // Sesión 1: Peer 1, TCP, ok, 2026-09-01
    insert_session_idempotent(
        &mut conn,
        &SessionRecord {
            id: Uuid::new_v4(),
            created_at: "2026-09-01T12:00:00Z".into(),
            peer_id: Some(peer1_id),
            status: "completed".into(),
            protocol: "tcp".into(),
            diagnostic_verdict: Some(r#"{"level":"ok"}"#.into()),
            ..Default::default()
        },
    )
    .unwrap();

    // Sesión 2: Peer 1, UDP, warning, 2026-09-15
    insert_session_idempotent(
        &mut conn,
        &SessionRecord {
            id: Uuid::new_v4(),
            created_at: "2026-09-15T12:00:00Z".into(),
            peer_id: Some(peer1_id),
            status: "completed".into(),
            protocol: "udp".into(),
            diagnostic_verdict: Some(r#"{"level":"warning"}"#.into()),
            ..Default::default()
        },
    )
    .unwrap();

    // Sesión 3: Peer 2, TCP, problem, 2026-09-20
    insert_session_idempotent(
        &mut conn,
        &SessionRecord {
            id: Uuid::new_v4(),
            created_at: "2026-09-20T12:00:00Z".into(),
            peer_id: Some(peer2_id),
            status: "completed".into(),
            protocol: "tcp".into(),
            diagnostic_verdict: Some(r#"{"level":"problem"}"#.into()),
            ..Default::default()
        },
    )
    .unwrap();

    // 1. Filtro por peer_id
    let by_peer1 = query_history(
        &conn,
        &HistoryFilter {
            peer_id: Some(peer1_id),
            ..Default::default()
        },
        &Pagination::default(),
    )
    .unwrap();
    assert_eq!(by_peer1.total_count, 2);

    // 2. Filtro por protocolo
    let by_udp = query_history(
        &conn,
        &HistoryFilter {
            protocol: Some("udp".into()),
            ..Default::default()
        },
        &Pagination::default(),
    )
    .unwrap();
    assert_eq!(by_udp.total_count, 1);
    assert_eq!(by_udp.items[0].protocol, "udp");

    // 3. Filtro por veredicto
    let by_verdict = query_history(
        &conn,
        &HistoryFilter {
            verdict: Some("problem".into()),
            ..Default::default()
        },
        &Pagination::default(),
    )
    .unwrap();
    assert_eq!(by_verdict.total_count, 1);
    assert_eq!(by_verdict.items[0].peer_id, Some(peer2_id));

    // 4. Filtro por fecha
    let by_date = query_history(
        &conn,
        &HistoryFilter {
            from_date: Some("2026-09-10T00:00:00Z".into()),
            to_date: Some("2026-09-18T00:00:00Z".into()),
            ..Default::default()
        },
        &Pagination::default(),
    )
    .unwrap();
    assert_eq!(by_date.total_count, 1);
    assert_eq!(by_date.items[0].created_at, "2026-09-15T12:00:00Z");

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_search_by_peer_name_and_alias() {
    let (db, tmp_dir) = setup_test_db();
    let mut conn = db.connection().lock().unwrap();

    let p_id = Uuid::new_v4();
    let mut peer = Peer::new(
        p_id,
        "DESKTOP-8492KFX".into(),
        "3333333333333333333333333333333333333333333333333333333333333333".into(),
        vec!["192.168.1.30:7411".into()],
    )
    .unwrap();
    peer.alias = Some("Oficina Central".into());
    upsert_peer(&conn, &peer).unwrap();

    insert_session_idempotent(
        &mut conn,
        &SessionRecord {
            id: Uuid::new_v4(),
            created_at: "2026-09-21T15:00:00Z".into(),
            peer_id: Some(p_id),
            ..Default::default()
        },
    )
    .unwrap();

    // Búsqueda por alias
    let search_alias = query_history(
        &conn,
        &HistoryFilter {
            search_query: Some("Oficina".into()),
            ..Default::default()
        },
        &Pagination::default(),
    )
    .unwrap();
    assert_eq!(search_alias.total_count, 1);
    assert_eq!(search_alias.items[0].peer_name, "Oficina Central");

    // Búsqueda por display_name original
    let search_disp = query_history(
        &conn,
        &HistoryFilter {
            search_query: Some("DESKTOP".into()),
            ..Default::default()
        },
        &Pagination::default(),
    )
    .unwrap();
    assert_eq!(search_disp.total_count, 1);

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_reopen_detail_and_retention_without_expiration() {
    let (db, tmp_dir) = setup_test_db();
    let mut conn = db.connection().lock().unwrap();

    let s_id = Uuid::new_v4();
    let old_date = "2021-03-15T08:30:00Z"; // Sesión de hace más de 5 años

    let session = SessionRecord {
        id: s_id,
        created_at: old_date.into(),
        peer_id: None,
        status: "completed".into(),
        protocol: "tcp".into(),
        streams: 4,
        duration_seconds: 30,
        forward_bps: Some("9800000000".into()),
        reverse_bps: Some("9750000000".into()),
        is_partial: false,
        diagnostic_verdict: Some(r#"{"level":"ok","utilization":98}"#.into()),
        client_interface: Some("Ethernet 10GbE".into()),
        server_interface: Some("Mellanox ConnectX".into()),
        result_json: Some(r#"{"fullResult":true}"#.into()),
        plan_json: Some(r#"{"protocol":"tcp","streams":4}"#.into()),
        samples: vec![
            SampleRecord {
                t_ms: 0,
                direction: "forward".into(),
                bps: "9800000000".into(),
                cpu_percent: Some(12.0),
                gap: false,
            },
            SampleRecord {
                t_ms: 500,
                direction: "forward".into(),
                bps: "9810000000".into(),
                cpu_percent: Some(13.5),
                gap: false,
            },
        ],
    };

    insert_session_idempotent(&mut conn, &session).unwrap();

    // 1. Reapertura íntegra mediante get_session_detail
    let detail = get_session_detail(&conn, &s_id).unwrap().expect("Sesión debe existir");
    assert_eq!(detail.id, s_id);
    assert_eq!(detail.created_at, old_date);
    assert_eq!(detail.client_interface.as_deref(), Some("Ethernet 10GbE"));
    assert_eq!(detail.server_interface.as_deref(), Some("Mellanox ConnectX"));
    assert_eq!(detail.result_json.as_deref(), Some(r#"{"fullResult":true}"#));
    assert_eq!(detail.samples.len(), 2);
    assert_eq!(detail.samples[1].t_ms, 500);

    // 2. Retención sin caducidad: la sesión antigua sigue presente en query_history
    let page = query_history(
        &conn,
        &HistoryFilter::default(),
        &Pagination::default(),
    )
    .unwrap();
    assert_eq!(page.total_count, 1);
    assert_eq!(page.items[0].id, s_id);

    drop(conn);
    let _ = fs::remove_dir_all(&tmp_dir);
}
