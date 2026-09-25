use super::response::IpcResult;
use crate::app::AppState;
use crate::control::SessionState;
use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::history::get_peer_by_fingerprint;
use crate::model::peer::{Peer, TrustState};
use crate::model::plan::BenchmarkPlan;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionPayload {
    pub peer: Peer,
    pub plan: BenchmarkPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResult {
    pub session_id: Uuid,
}

#[tauri::command]
pub async fn session_start(
    payload: StartSessionPayload,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<IpcResult<StartSessionResult>, String> {
    if let Err(e) = payload.plan.validate() {
        return Ok(IpcResult::err(AppError::new(
            ErrorCode::InternalError,
            ErrorSeverity::Error,
            format!("Plan inválido: {}", e),
        )));
    }

    // La autoridad sobre la confianza es de Rust, no de lo que diga el frontend: el peer
    // se recupera de la base de datos por su huella y debe estar emparejado. Del payload
    // solo se toma dónde conectar, y esa dirección no otorga nada: si el certificado que
    // se presente no coincide con la huella guardada, la sesión no arranca (FR-013).
    let almacenado = {
        let db = state.database.connection().lock().unwrap();
        get_peer_by_fingerprint(&db, &payload.peer.fingerprint)
    };
    let mut peer = match almacenado {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Ok(IpcResult::err(AppError::new(
                ErrorCode::PeerRejected,
                ErrorSeverity::Error,
                "Este equipo no está emparejado: verifícalo antes de medir".to_string(),
            )));
        }
        Err(e) => {
            return Ok(IpcResult::err(AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                format!("No se pudo consultar el equipo: {e}"),
            )));
        }
    };
    if !matches!(
        peer.trust_state,
        TrustState::Trusted | TrustState::TrustedAutoAccept
    ) {
        return Ok(IpcResult::err(AppError::new(
            ErrorCode::PeerRejected,
            ErrorSeverity::Error,
            "Este equipo no es de confianza: verifícalo antes de medir".to_string(),
        )));
    }
    peer.addresses = payload.peer.addresses;

    let ctx = state.contexto_de_sesion(Some(Arc::new(app)));

    match state
        .session_service
        .iniciar_sesion_real(ctx, peer, payload.plan)
        .await
    {
        Ok(session_id) => Ok(IpcResult::ok(StartSessionResult { session_id })),
        Err(e) => Ok(IpcResult::err(AppError::new(
            ErrorCode::ConnInterrupted,
            ErrorSeverity::Error,
            format!("No se pudo iniciar la sesión: {:?}", e),
        ))),
    }
}

#[tauri::command]
pub async fn session_cancel(
    session_id: Uuid,
    _reason: Option<String>,
    state: State<'_, AppState>,
) -> Result<IpcResult<()>, String> {
    match state.session_service.cancel(session_id).await {
        Ok(()) => Ok(IpcResult::ok(())),
        Err(e) => Ok(IpcResult::err(AppError::new(
            ErrorCode::InternalError,
            ErrorSeverity::Error,
            format!("Error al cancelar la sesión: {:?}", e),
        ))),
    }
}

#[tauri::command]
pub async fn session_get_state(
    state: State<'_, AppState>,
) -> Result<IpcResult<SessionState>, String> {
    let current = state.session_service.current_state().await;
    Ok(IpcResult::ok(current))
}
