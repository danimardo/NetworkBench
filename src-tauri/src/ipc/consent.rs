//! Comandos IPC de consentimiento para solicitudes de sesión entrantes (T177, FR-016).
//!
//! El backend avisa a la interfaz con `session://incoming-request` en cuanto llega una
//! solicitud de un equipo de confianza sin autoaceptación (`ipc::events`). Estos dos
//! comandos son la otra mitad: recuperar lo pendiente si la interfaz se acaba de abrir, y
//! enviar la decisión de vuelta a la sesión que la está esperando.

use super::response::IpcResult;
use crate::app::AppState;
use crate::control::consent::SolicitudEntranteEvento;
use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn session_incoming_list(
    state: State<'_, AppState>,
) -> Result<IpcResult<Vec<SolicitudEntranteEvento>>, String> {
    Ok(IpcResult::ok(state.solicitudes_entrantes.listar().await))
}

#[tauri::command]
pub async fn session_incoming_respond(
    request_id: Uuid,
    accepted: bool,
    state: State<'_, AppState>,
) -> Result<IpcResult<()>, String> {
    match state
        .solicitudes_entrantes
        .responder(request_id, accepted)
        .await
    {
        Ok(()) => Ok(IpcResult::ok(())),
        Err(e) => Ok(IpcResult::err(AppError::new(
            ErrorCode::InternalError,
            ErrorSeverity::Warning,
            e,
        ))),
    }
}
