use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::history::migrations::{CURRENT_SCHEMA_VERSION, MIGRATIONS};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Database {
    db_path: PathBuf,
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(db_path: PathBuf) -> Result<Self, AppError> {
        if let Some(parent) = db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut conn = Connection::open(&db_path).map_err(|e| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Fatal,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "database.open".to_string(),
                code: "DB_OPEN_FAILED".to_string(),
                message_key: "errors.NB-INTERNAL-001".to_string(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            })
        })?;

        // 1. Pragmas obligatorios
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            "#,
        )
        .map_err(|e| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Fatal,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "database.pragmas".to_string(),
                code: "DB_PRAGMAS_FAILED".to_string(),
                message_key: "errors.NB-INTERNAL-001".to_string(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            })
        })?;

        // 2. Comprobación de versión de esquema
        let user_version: u32 = conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap_or(0);

        // Rechazo estricto de esquema futuro (ej. app antigua abriendo BD de versión nueva)
        if user_version > CURRENT_SCHEMA_VERSION {
            return Err(AppError::new(
                ErrorCode::VersionIncompatible,
                ErrorSeverity::Fatal,
                "errors.NB-VERSION-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "database.user_version".to_string(),
                code: "FUTURE_SCHEMA_DETECTED".to_string(),
                message_key: "errors.NB-VERSION-001".to_string(),
                safe_params: Some(serde_json::json!({
                    "dbVersion": user_version,
                    "supportedVersion": CURRENT_SCHEMA_VERSION
                })),
            }));
        }

        // 3. Ejecutar migraciones pendientes
        if user_version < CURRENT_SCHEMA_VERSION {
            // Backup automático de seguridad si ya existía una versión previa > 0
            if user_version > 0 && db_path.exists() {
                let backup_path = db_path.with_extension(format!("v{}.bak", user_version));
                let _ = fs::copy(&db_path, &backup_path);
            }

            for migration in MIGRATIONS {
                if migration.version > user_version && migration.version <= CURRENT_SCHEMA_VERSION {
                    let tx = conn.transaction().map_err(|e| {
                        AppError::new(
                            ErrorCode::InternalError,
                            ErrorSeverity::Fatal,
                            "errors.NB-INTERNAL-001",
                        )
                        .with_issue(crate::errors::AppIssue {
                            path: "database.migration_tx".to_string(),
                            code: "TX_START_FAILED".to_string(),
                            message_key: "errors.NB-INTERNAL-001".to_string(),
                            safe_params: Some(serde_json::json!({ "details": e.to_string() })),
                        })
                    })?;

                    tx.execute_batch(migration.sql).map_err(|e| {
                        AppError::new(
                            ErrorCode::InternalError,
                            ErrorSeverity::Fatal,
                            "errors.NB-INTERNAL-001",
                        )
                        .with_issue(crate::errors::AppIssue {
                            path: format!("database.migration_{}", migration.version),
                            code: "MIGRATION_FAILED".to_string(),
                            message_key: "errors.NB-INTERNAL-001".to_string(),
                            safe_params: Some(serde_json::json!({ "details": e.to_string() })),
                        })
                    })?;

                    tx.execute(
                        &format!("PRAGMA user_version = {};", migration.version),
                        [],
                    )
                    .map_err(|e| {
                        AppError::new(
                            ErrorCode::InternalError,
                            ErrorSeverity::Fatal,
                            "errors.NB-INTERNAL-001",
                        )
                        .with_issue(crate::errors::AppIssue {
                            path: "database.set_version".to_string(),
                            code: "SET_VERSION_FAILED".to_string(),
                            message_key: "errors.NB-INTERNAL-001".to_string(),
                            safe_params: Some(serde_json::json!({ "details": e.to_string() })),
                        })
                    })?;

                    tx.commit().map_err(|e| {
                        AppError::new(
                            ErrorCode::InternalError,
                            ErrorSeverity::Fatal,
                            "errors.NB-INTERNAL-001",
                        )
                        .with_issue(crate::errors::AppIssue {
                            path: "database.commit".to_string(),
                            code: "COMMIT_FAILED".to_string(),
                            message_key: "errors.NB-INTERNAL-001".to_string(),
                            safe_params: Some(serde_json::json!({ "details": e.to_string() })),
                        })
                    })?;
                }
            }
        }

        Ok(Self {
            db_path,
            conn: Mutex::new(conn),
        })
    }

    pub fn backup_to(&self, destination: &Path) -> Result<(), AppError> {
        let lock = self.conn.lock().map_err(|_| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Fatal,
                "errors.NB-INTERNAL-001",
            )
        })?;

        if destination.exists() {
            let _ = fs::remove_file(destination);
        }

        let dst_str = destination.to_str().ok_or_else(|| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
        })?;

        // VACUUM INTO genera una copia de backup atómica, consistente y libre de bloqueos WAL
        lock.execute("VACUUM INTO ?1;", [dst_str]).map_err(|e| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "database.backup".to_string(),
                code: "VACUUM_INTO_FAILED".to_string(),
                message_key: "errors.NB-INTERNAL-001".to_string(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            })
        })?;

        Ok(())
    }

    pub fn connection(&self) -> &Mutex<Connection> {
        &self.conn
    }

    pub fn path(&self) -> &Path {
        &self.db_path
    }
}
