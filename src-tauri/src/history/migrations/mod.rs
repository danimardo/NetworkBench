pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const CURRENT_SCHEMA_VERSION: u32 = 2;

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "001_initial_schema",
        sql: r#"
        CREATE TABLE IF NOT EXISTS peers (
            id TEXT PRIMARY KEY,
            fingerprint TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            alias TEXT,
            is_trusted INTEGER NOT NULL DEFAULT 0,
            auto_accept INTEGER NOT NULL DEFAULT 0,
            first_seen_at TEXT NOT NULL,
            last_seen_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
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

        CREATE TABLE IF NOT EXISTS samples (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            t_ms INTEGER NOT NULL,
            direction TEXT NOT NULL,
            bps TEXT NOT NULL,
            cpu_percent REAL,
            gap INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at);
        CREATE INDEX IF NOT EXISTS idx_sessions_peer_id ON sessions(peer_id);
        CREATE INDEX IF NOT EXISTS idx_samples_session_time ON samples(session_id, t_ms);
        "#,
    },
    Migration {
        version: 2,
        name: "002_history_and_cohorts",
        sql: r#"
        ALTER TABLE sessions ADD COLUMN client_interface TEXT;
        ALTER TABLE sessions ADD COLUMN server_interface TEXT;
        ALTER TABLE sessions ADD COLUMN result_json TEXT;
        ALTER TABLE sessions ADD COLUMN plan_json TEXT;

        CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
        CREATE INDEX IF NOT EXISTS idx_sessions_protocol ON sessions(protocol);
        "#,
    },
];
