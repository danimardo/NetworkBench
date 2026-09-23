use super::response::IpcResult;
use crate::app::AppState;
use crate::control::SessionState;
use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::model::peer::Peer;
use crate::model::plan::BenchmarkPlan;
use serde::{Deserialize, Serialize};
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
    state: State<'_, AppState>,
) -> Result<IpcResult<StartSessionResult>, String> {
    if let Err(e) = payload.plan.validate() {
        return Ok(IpcResult::err(AppError::new(
            ErrorCode::InternalError,
            ErrorSeverity::Error,
            format!("Plan inválido: {}", e),
        )));
    }

    match state
        .session_service
        .start_session(payload.peer, payload.plan)
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
pub async fn session_get_state(state: State<'_, AppState>) -> Result<IpcResult<SessionState>, String> {
    let current = state.session_service.current_state().await;
    Ok(IpcResult::ok(current))
}
