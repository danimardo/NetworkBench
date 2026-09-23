use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub revision: u64,
    pub app_version: String,
    pub locale: String,
    pub theme: String,
    pub instance_id: String,
    pub instance_name: String,
    pub is_session_active: bool,
    pub active_session_id: Option<String>,
    pub peers_count: usize,
}

impl Default for AppSnapshot {
    fn default() -> Self {
        Self {
            revision: 1,
            app_version: "0.1.0".to_string(),
            locale: "es".to_string(),
            theme: "dark".to_string(),
            instance_id: String::new(),
            instance_name: "NetworkBench".to_string(),
            is_session_active: false,
            active_session_id: None,
            peers_count: 0,
        }
    }
}

pub struct SnapshotManager {
    revision_counter: AtomicU64,
    current_snapshot: Mutex<AppSnapshot>,
}

impl SnapshotManager {
    pub fn new(initial: AppSnapshot) -> Self {
        let rev = initial.revision;
        Self {
            revision_counter: AtomicU64::new(rev),
            current_snapshot: Mutex::new(initial),
        }
    }

    pub fn get_snapshot(&self) -> AppSnapshot {
        let lock = self.current_snapshot.lock().unwrap();
        lock.clone()
    }

    pub fn current_revision(&self) -> u64 {
        self.revision_counter.load(Ordering::SeqCst)
    }

    /// Incrementa atómicamente la revisión monotónica y actualiza el snapshot
    pub fn update<F>(&self, mutate: F) -> AppSnapshot
    where
        F: FnOnce(&mut AppSnapshot),
    {
        let mut lock = self.current_snapshot.lock().unwrap();
        let next_rev = self.revision_counter.fetch_add(1, Ordering::SeqCst) + 1;
        mutate(&mut lock);
        lock.revision = next_rev;
        lock.clone()
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new(AppSnapshot::default())
    }
}
