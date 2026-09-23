use super::response::IpcResult;
use crate::app::AppState;
use crate::logging::diagnostics::DiagnosticSanitizer;
use crate::model::result::SessionResult;
use tauri::State;

#[tauri::command]
pub async fn diagnostics_get_report(
    result: SessionResult,
    _state: State<'_, AppState>,
) -> Result<IpcResult<String>, String> {
    let report = DiagnosticSanitizer::build_technical_report(&result);
    Ok(IpcResult::ok(report))
}
