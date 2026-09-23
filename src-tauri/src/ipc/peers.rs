use super::response::IpcResult;
use crate::app::AppState;
use crate::discovery::manual_connect_peer;
use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::history::{get_peer_by_fingerprint, upsert_peer};
use crate::model::peer::{Peer, TrustState};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn peers_list(state: State<'_, AppState>) -> Result<IpcResult<Vec<Peer>>, String> {
    let db_lock = state.database.connection().lock().unwrap();
    let mut stmt = match db_lock.prepare(
        r#"
        SELECT id, fingerprint, display_name, alias, is_trusted, auto_accept, last_seen_at
        FROM peers
        ORDER BY last_seen_at DESC
        "#,
    ) {
        Ok(s) => s,
        Err(e) => {
            return Ok(IpcResult::err(AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                format!("Error consultando peers: {}", e),
            )))
        }
    };

    let rows = match stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        let fp: String = row.get(1)?;
        let display_name: String = row.get(2)?;
        let alias: Option<String> = row.get(3)?;
        let is_trusted: i64 = row.get(4)?;
        let auto_accept: i64 = row.get(5)?;
        let last_seen: String = row.get(6)?;

        let trust_state = if is_trusted == 1 && auto_accept == 1 {
            TrustState::TrustedAutoAccept
        } else if is_trusted == 1 {
            TrustState::Trusted
        } else {
            TrustState::Known
        };

        let instance_id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::nil());

        Ok(Peer {
            instance_id,
            display_name,
            fingerprint: fp,
            addresses: vec![],
            trust_state,
            auto_accept: auto_accept == 1,
            last_seen,
            alias,
        })
    }) {
        Ok(r) => r,
        Err(e) => {
            return Ok(IpcResult::err(AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                format!("Error mapeando peers: {}", e),
            )))
        }
    };

    let mut list = Vec::new();
    for p in rows {
        if let Ok(peer) = p {
            list.push(peer);
        }
    }

    Ok(IpcResult::ok(list))
}

#[tauri::command]
pub async fn peers_manual_connect(
    host: String,
    port: u16,
    state: State<'_, AppState>,
) -> Result<IpcResult<Peer>, String> {
    let local_pub = state.identity.to_public();
    let peer = match manual_connect_peer(&host, port, &local_pub).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(IpcResult::err(AppError::new(
                ErrorCode::ConnCannotReach,
                ErrorSeverity::Error,
                format!("Error conectando manualmente: {}", e),
            )))
        }
    };

    // Si el peer ya existía en BD, conservar su estado de confianza
    let db_lock = state.database.connection().lock().unwrap();
    let mut final_peer = peer.clone();
    if let Ok(Some(existing)) = get_peer_by_fingerprint(&db_lock, &peer.fingerprint) {
        final_peer.trust_state = existing.trust_state;
        final_peer.auto_accept = existing.auto_accept;
        final_peer.alias = existing.alias;
    }
    let _ = upsert_peer(&db_lock, &final_peer);

    Ok(IpcResult::ok(final_peer))
}

#[tauri::command]
pub async fn peers_set_trust(
    fingerprint: String,
    is_trusted: bool,
    auto_accept: bool,
    state: State<'_, AppState>,
) -> Result<IpcResult<()>, String> {
    let db_lock = state.database.connection().lock().unwrap();
    let norm_fp = fingerprint.trim().to_lowercase();

    let res = db_lock.execute(
        r#"
        UPDATE peers
        SET is_trusted = ?1, auto_accept = ?2
        WHERE fingerprint = ?3
        "#,
        rusqlite::params![is_trusted as i64, auto_accept as i64, norm_fp],
    );

    match res {
        Ok(_) => Ok(IpcResult::ok(())),
        Err(e) => Ok(IpcResult::err(AppError::new(
            ErrorCode::InternalError,
            ErrorSeverity::Error,
            format!("Error actualizando confianza: {}", e),
        ))),
    }
}
