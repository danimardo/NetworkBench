use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const DELETE_TOKEN_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum DeleteTarget {
    All,
    ByPeer(Uuid),
    OlderThan(String),
    Single(Uuid),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePreview {
    pub target: DeleteTarget,
    pub affected_sessions: u32,
    pub affected_samples: u32,
    pub confirmation_token: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResult {
    pub deleted_sessions: u32,
    pub deleted_samples: u32,
}

struct TokenEntry {
    target: DeleteTarget,
    created_at: Instant,
}

pub struct DeleteTokenStore {
    tokens: Mutex<HashMap<String, TokenEntry>>,
}

impl DeleteTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn issue_token(&self, target: DeleteTarget) -> String {
        let token = Uuid::new_v4().to_string();
        let mut map = self.tokens.lock().unwrap();
        // Limpiar tokens expirados
        map.retain(|_, entry| entry.created_at.elapsed() < DELETE_TOKEN_TTL);
        map.insert(
            token.clone(),
            TokenEntry {
                target,
                created_at: Instant::now(),
            },
        );
        token
    }

    pub fn validate_and_consume(&self, token: &str) -> Option<DeleteTarget> {
        let mut map = self.tokens.lock().unwrap();
        if let Some(entry) = map.remove(token)
            && entry.created_at.elapsed() < DELETE_TOKEN_TTL
        {
            return Some(entry.target);
        }
        None
    }
}

impl Default for DeleteTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Genera la previsualización del impacto de borrado sin ejecutar ninguna modificación en la base de datos.
pub fn preview_delete(
    conn: &Connection,
    target: &DeleteTarget,
    token_store: &DeleteTokenStore,
) -> Result<DeletePreview> {
    let (sessions_sql, samples_sql, query_param): (String, String, Option<String>) = match target {
        DeleteTarget::All => (
            "SELECT COUNT(*) FROM sessions;".into(),
            "SELECT COUNT(*) FROM samples;".into(),
            None,
        ),
        DeleteTarget::ByPeer(peer_id) => (
            "SELECT COUNT(*) FROM sessions WHERE peer_id = ?1;".into(),
            "SELECT COUNT(*) FROM samples WHERE session_id IN (SELECT id FROM sessions WHERE peer_id = ?1);".into(),
            Some(peer_id.to_string()),
        ),
        DeleteTarget::OlderThan(date_str) => (
            "SELECT COUNT(*) FROM sessions WHERE created_at < ?1;".into(),
            "SELECT COUNT(*) FROM samples WHERE session_id IN (SELECT id FROM sessions WHERE created_at < ?1);".into(),
            Some(date_str.clone()),
        ),
        DeleteTarget::Single(session_id) => (
            "SELECT COUNT(*) FROM sessions WHERE id = ?1;".into(),
            "SELECT COUNT(*) FROM samples WHERE session_id = ?1;".into(),
            Some(session_id.to_string()),
        ),
    };

    let affected_sessions: u32 = if let Some(ref p) = query_param {
        conn.query_row(&sessions_sql, params![p], |r| r.get(0))?
    } else {
        conn.query_row(&sessions_sql, [], |r| r.get(0))?
    };

    let affected_samples: u32 = if let Some(ref p) = query_param {
        conn.query_row(&samples_sql, params![p], |r| r.get(0))?
    } else {
        conn.query_row(&samples_sql, [], |r| r.get(0))?
    };

    let confirmation_token = token_store.issue_token(target.clone());

    Ok(DeletePreview {
        target: target.clone(),
        affected_sessions,
        affected_samples,
        confirmation_token,
        expires_in_seconds: DELETE_TOKEN_TTL.as_secs(),
    })
}

/// Ejecuta el borrado transaccional de sesiones y muestras asociadas tras validar el token de confirmación de un solo uso.
pub fn confirm_delete(
    conn: &mut Connection,
    token: &str,
    token_store: &DeleteTokenStore,
) -> Result<Option<DeleteResult>> {
    let target = match token_store.validate_and_consume(token) {
        Some(t) => t,
        None => return Ok(None),
    };

    let tx = conn.transaction()?;

    let (del_sql, param): (String, Option<String>) = match target {
        DeleteTarget::All => ("DELETE FROM sessions;".into(), None),
        DeleteTarget::ByPeer(peer_id) => (
            "DELETE FROM sessions WHERE peer_id = ?1;".into(),
            Some(peer_id.to_string()),
        ),
        DeleteTarget::OlderThan(date_str) => (
            "DELETE FROM sessions WHERE created_at < ?1;".into(),
            Some(date_str),
        ),
        DeleteTarget::Single(session_id) => (
            "DELETE FROM sessions WHERE id = ?1;".into(),
            Some(session_id.to_string()),
        ),
    };

    let deleted_sessions = if let Some(ref p) = param {
        tx.execute(&del_sql, params![p])? as u32
    } else {
        tx.execute(&del_sql, [])? as u32
    };

    // Al haber activado PRAGMA foreign_keys = ON y ON DELETE CASCADE en samples,
    // SQLite limpia automáticamente las muestras asociadas en la misma transacción.
    tx.commit()?;

    Ok(Some(DeleteResult {
        deleted_sessions,
        deleted_samples: 0, // limpiadas en cascada por el motor relacional
    }))
}
