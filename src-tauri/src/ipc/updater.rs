use crate::app::AppState;
use crate::control::domain::SessionState;
use crate::ipc::response::IpcResult;
use crate::updater::{evaluate_manifest, UpdateStatus};

#[tauri::command]
pub async fn updater_check(state: tauri::State<'_, AppState>) -> Result<IpcResult<UpdateStatus>, ()> {
    let session_state = state.session_service.current_state().await;
    let is_session_active = session_state != SessionState::Idle && session_state != SessionState::Cancelled;

    if is_session_active {
        return Ok(IpcResult::ok(UpdateStatus::DeferredDueToActiveSession));
    }

    // Retorna por defecto UpToDate en entorno local / offline
    Ok(IpcResult::ok(UpdateStatus::UpToDate))
}

#[tauri::command]
pub async fn updater_evaluate(
    state: tauri::State<'_, AppState>,
    manifest_json: String,
) -> Result<IpcResult<UpdateStatus>, ()> {
    let session_state = state.session_service.current_state().await;
    let is_session_active = session_state != SessionState::Idle && session_state != SessionState::Cancelled;

    let current_version = env!("CARGO_PKG_VERSION");
    let target_platform = if cfg!(target_arch = "x86_64") {
        "windows-x86_64"
    } else {
        "windows-aarch64"
    };

    match evaluate_manifest(&manifest_json, current_version, target_platform, is_session_active) {
        Ok(status) => Ok(IpcResult::ok(status)),
        Err(e) => {
            let app_err = crate::errors::AppError::new(
                crate::errors::ErrorCode::InternalError,
                crate::errors::ErrorSeverity::Warning,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "updater".to_string(),
                code: "UPDATE_EVALUATION_FAILED".to_string(),
                message_key: "errors.NB-INTERNAL-001".to_string(),
                safe_params: Some(serde_json::json!({ "details": e })),
            });
            Ok(IpcResult::err(app_err))
        }
    }
}
