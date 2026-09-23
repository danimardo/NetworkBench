use networkbench_lib::ipc::snapshot::{AppSnapshot, SnapshotManager};
use std::sync::Arc;
use std::thread;

#[test]
fn test_snapshot_initial_revision() {
    let initial = AppSnapshot {
        revision: 1,
        app_version: "0.1.0".to_string(),
        locale: "es".to_string(),
        theme: "dark".to_string(),
        instance_id: "inst-test".to_string(),
        instance_name: "HostTest".to_string(),
        is_session_active: false,
        active_session_id: None,
        peers_count: 0,
    };

    let manager = SnapshotManager::new(initial);
    assert_eq!(manager.current_revision(), 1);
    let snap = manager.get_snapshot();
    assert_eq!(snap.revision, 1);
    assert_eq!(snap.locale, "es");
}

#[test]
fn test_snapshot_monotonic_updates() {
    let manager = SnapshotManager::default();
    assert_eq!(manager.current_revision(), 1);

    let snap2 = manager.update(|s| {
        s.peers_count = 5;
    });
    assert_eq!(snap2.revision, 2);
    assert_eq!(snap2.peers_count, 5);
    assert_eq!(manager.current_revision(), 2);

    let snap3 = manager.update(|s| {
        s.is_session_active = true;
    });
    assert_eq!(snap3.revision, 3);
    assert_eq!(snap3.is_session_active, true);
    assert_eq!(manager.current_revision(), 3);
}

#[test]
fn test_snapshot_concurrent_monotonic_ordering() {
    let manager = Arc::new(SnapshotManager::default());
    let mut handles = Vec::new();

    for _ in 0..10 {
        let mgr = Arc::clone(&manager);
        handles.push(thread::spawn(move || {
            for _ in 0..10 {
                mgr.update(|s| {
                    s.peers_count += 1;
                });
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // 1 inicial + 100 updates = 101
    assert_eq!(manager.current_revision(), 101);
    let final_snap = manager.get_snapshot();
    assert_eq!(final_snap.revision, 101);
    assert_eq!(final_snap.peers_count, 100);
}
