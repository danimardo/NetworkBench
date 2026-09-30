use super::response::IpcResult;
use crate::app::AppState;
use crate::discovery::manual_connect_peer;
use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::history::{get_peer_by_fingerprint, upsert_peer};
use crate::model::peer::{Peer, TrustState};
use tauri::State;
use uuid::Uuid;

/// Equipos que se ven ahora mismo por mDNS. **Pistas, no identidades** (FR-011): nombre,
/// dirección y huella son lo que cada equipo *dice* de sí; la identidad real se demuestra
/// en el handshake TLS al conectar. No incluye ningún estado de confianza.
#[tauri::command]
pub async fn peers_discovered_list(
    state: State<'_, AppState>,
) -> Result<IpcResult<Vec<crate::discovery::EquipoDescubierto>>, String> {
    let equipos = state
        .descubrimiento
        .lock()
        .ok()
        .and_then(|d| d.as_ref().map(|d| d.equipos()))
        .unwrap_or_default();
    Ok(IpcResult::ok(equipos))
}

/// «Buscar de nuevo»: reinicia el descubrimiento mDNS para volver a preguntar a la red, por
/// si alguien abrió su aplicación después de que arrancara la nuestra.
#[tauri::command]
pub async fn peers_rescan(state: State<'_, AppState>) -> Result<IpcResult<()>, String> {
    state.reiniciar_descubrimiento();
    // «Buscar de nuevo» también vuelve a comprobar los equipos guardados, sin esperar al latido.
    state.alcance.forzar(None);
    Ok(IpcResult::ok(()))
}

/// Si cada equipo guardado está activo en la red. Pista para la interfaz, no identidad: ver
/// `discovery::alcance`. Un equipo que no figura aún se está comprobando.
#[tauri::command]
pub async fn peers_reachability_list(
    state: State<'_, AppState>,
) -> Result<IpcResult<Vec<crate::discovery::EstadoAlcance>>, String> {
    Ok(IpcResult::ok(state.alcance.estados()))
}

/// «Comprobar ahora»: un equipo concreto, o todos si no se indica ninguno.
#[tauri::command]
pub async fn peers_check_now(
    fingerprint: Option<String>,
    state: State<'_, AppState>,
) -> Result<IpcResult<()>, String> {
    let fp = fingerprint.map(|f| f.trim().to_lowercase());
    state.alcance.forzar(fp.as_deref());
    Ok(IpcResult::ok(()))
}

#[tauri::command]
pub async fn peers_list(state: State<'_, AppState>) -> Result<IpcResult<Vec<Peer>>, String> {
    let db_lock = state.database.connection().lock().unwrap();
    let mut stmt = match db_lock.prepare(
        r#"
        SELECT id, fingerprint, display_name, alias, is_trusted, auto_accept, last_seen_at, is_favorite, last_address
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
            )));
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
        let is_favorite: i64 = row.get(7)?;
        let last_address: Option<String> = row.get(8)?;

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
            // La última dirección conocida: el equipo puede no anunciarse ahora, pero se
            // sabe dónde se le vio y ahí se le comprueba.
            addresses: last_address.into_iter().collect(),
            trust_state,
            auto_accept: auto_accept == 1,
            last_seen,
            favorite: is_favorite == 1,
            alias,
        })
    }) {
        Ok(r) => r,
        Err(e) => {
            return Ok(IpcResult::err(AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                format!("Error mapeando peers: {}", e),
            )));
        }
    };

    let mut list = Vec::new();
    for peer in rows.flatten() {
        list.push(peer);
    }

    Ok(IpcResult::ok(list))
}

#[tauri::command]
pub async fn peers_manual_connect(
    host: String,
    port: u16,
    state: State<'_, AppState>,
) -> Result<IpcResult<Peer>, String> {
    let peer = match manual_connect_peer(&host, port, &state.identity).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(IpcResult::err(AppError::new(
                ErrorCode::ConnCannotReach,
                ErrorSeverity::Error,
                format!("Error conectando manualmente: {}", e),
            )));
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
    state.aviso_snapshot.notify_one();

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

/// Marca o desmarca un equipo guardado como favorito. Es una preferencia local: no toca la
/// confianza ni se comunica al otro equipo.
#[tauri::command]
pub async fn peers_set_favorite(
    fingerprint: String,
    favorite: bool,
    state: State<'_, AppState>,
) -> Result<IpcResult<()>, String> {
    let db_lock = state.database.connection().lock().unwrap();
    let norm_fp = fingerprint.trim().to_lowercase();
    match set_favorite(&db_lock, &norm_fp, favorite) {
        Ok(()) => Ok(IpcResult::ok(())),
        Err(e) => Ok(IpcResult::err(AppError::new(
            ErrorCode::InternalError,
            ErrorSeverity::Error,
            format!("Error actualizando favorito: {}", e),
        ))),
    }
}

/// «Eliminar equipo»: olvida el equipo guardado (y con él su confianza). Solo actúa en este
/// equipo; el otro conserva lo suyo. Las sesiones del historial no se borran: su `peer_id`
/// pasa a `NULL` (`ON DELETE SET NULL`).
#[tauri::command]
pub async fn peers_forget(
    fingerprint: String,
    state: State<'_, AppState>,
) -> Result<IpcResult<()>, String> {
    let db_lock = state.database.connection().lock().unwrap();
    let norm_fp = fingerprint.trim().to_lowercase();
    match forget_peer(&db_lock, &norm_fp) {
        Ok(()) => {
            drop(db_lock);
            state.aviso_snapshot.notify_one();
            Ok(IpcResult::ok(()))
        }
        Err(e) => Ok(IpcResult::err(AppError::new(
            ErrorCode::InternalError,
            ErrorSeverity::Error,
            format!("Error eliminando el equipo: {}", e),
        ))),
    }
}

fn set_favorite(conn: &rusqlite::Connection, fp: &str, favorite: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE peers SET is_favorite = ?1 WHERE fingerprint = ?2",
        rusqlite::params![favorite as i64, fp],
    )
    .map(|_| ())
}

fn forget_peer(conn: &rusqlite::Connection, fp: &str) -> rusqlite::Result<()> {
    // `PRAGMA foreign_keys` puede estar apagado: se desvincula el historial a mano para que
    // el resultado no dependa de ello.
    conn.execute(
        "UPDATE sessions SET peer_id = NULL WHERE peer_id IN (SELECT id FROM peers WHERE fingerprint = ?1)",
        rusqlite::params![fp],
    )?;
    conn.execute(
        "DELETE FROM peers WHERE fingerprint = ?1",
        rusqlite::params![fp],
    )
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::migrations::MIGRATIONS;
    use rusqlite::Connection;

    fn bd() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in MIGRATIONS {
            c.execute_batch(m.sql).unwrap();
        }
        let fp = "a".repeat(64);
        c.execute(
            "INSERT INTO peers (id, fingerprint, display_name, is_trusted, first_seen_at, last_seen_at) VALUES ('p1', ?1, 'WIN11D', 1, 't', 't')",
            [&fp],
        )
        .unwrap();
        c.execute(
            "INSERT INTO sessions (id, created_at, peer_id, status, protocol, streams, duration_seconds) VALUES ('s1', 't', 'p1', 'ok', 'tcp', 1, 1)",
            [],
        )
        .unwrap();
        c
    }

    #[test]
    fn favorito_se_guarda_y_se_quita() {
        let c = bd();
        let fp = "a".repeat(64);
        set_favorite(&c, &fp, true).unwrap();
        let v: i64 = c
            .query_row("SELECT is_favorite FROM peers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 1);
        set_favorite(&c, &fp, false).unwrap();
        let v: i64 = c
            .query_row("SELECT is_favorite FROM peers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn olvidar_borra_el_equipo_y_conserva_el_historial() {
        let c = bd();
        forget_peer(&c, &"a".repeat(64)).unwrap();
        let n: i64 = c
            .query_row("SELECT COUNT(*) FROM peers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);
        let peer_id: Option<String> = c
            .query_row("SELECT peer_id FROM sessions WHERE id='s1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(peer_id, None);
    }
}
