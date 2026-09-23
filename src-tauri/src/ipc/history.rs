use crate::app::AppState;
use crate::control::repeat::{reconstruct_repeat_plan, RepeatPlanConfig};
use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::history::comparison::{
    compare_session_with_cohort, get_peer_trend, CohortComparisonResult, PeerTrendPoint,
};
use crate::history::delete::{
    confirm_delete, preview_delete, DeletePreview, DeleteResult, DeleteTarget,
};
use crate::history::queries::{
    get_session_detail, query_history, HistoryFilter, HistoryPage, Pagination,
};
use crate::history::sessions::SessionRecord;
use crate::ipc::response::IpcResult;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn history_list(
    state: State<AppState>,
    filter: Option<HistoryFilter>,
    pagination: Option<Pagination>,
) -> IpcResult<HistoryPage> {
    let conn = state.database.connection().lock().unwrap();
    let f = filter.unwrap_or_default();
    let p = pagination.unwrap_or_default();

    match query_history(&conn, &f, &p) {
        Ok(page) => IpcResult::ok(page),
        Err(e) => IpcResult::err(
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "history.list".into(),
                code: "QUERY_FAILED".into(),
                message_key: "errors.NB-INTERNAL-001".into(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            }),
        ),
    }
}

#[tauri::command]
pub fn history_get(state: State<AppState>, session_id: Uuid) -> IpcResult<Option<SessionRecord>> {
    let conn = state.database.connection().lock().unwrap();
    match get_session_detail(&conn, &session_id) {
        Ok(res) => IpcResult::ok(res),
        Err(e) => IpcResult::err(
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "history.get".into(),
                code: "QUERY_FAILED".into(),
                message_key: "errors.NB-INTERNAL-001".into(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            }),
        ),
    }
}

#[tauri::command]
pub fn history_delete_preview(
    state: State<AppState>,
    target: DeleteTarget,
) -> IpcResult<DeletePreview> {
    let conn = state.database.connection().lock().unwrap();
    match preview_delete(&conn, &target, &state.delete_tokens) {
        Ok(preview) => IpcResult::ok(preview),
        Err(e) => IpcResult::err(
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "history.delete_preview".into(),
                code: "PREVIEW_FAILED".into(),
                message_key: "errors.NB-INTERNAL-001".into(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            }),
        ),
    }
}

#[tauri::command]
pub fn history_delete_confirm(
    state: State<AppState>,
    token: String,
) -> IpcResult<Option<DeleteResult>> {
    let mut conn = state.database.connection().lock().unwrap();
    match confirm_delete(&mut conn, &token, &state.delete_tokens) {
        Ok(Some(res)) => IpcResult::ok(Some(res)),
        Ok(None) => IpcResult::err(
            AppError::new(
                ErrorCode::ParamRejected,
                ErrorSeverity::Error,
                "errors.NB-PARAM-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "history.delete_confirm".into(),
                code: "INVALID_OR_EXPIRED_TOKEN".into(),
                message_key: "errors.NB-PARAM-001".into(),
                safe_params: None,
            }),
        ),
        Err(e) => IpcResult::err(
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Fatal,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "history.delete_confirm".into(),
                code: "DELETE_FAILED".into(),
                message_key: "errors.NB-INTERNAL-001".into(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            }),
        ),
    }
}

#[tauri::command]
pub fn history_get_trend(
    state: State<AppState>,
    peer_id: Uuid,
    protocol: Option<String>,
) -> IpcResult<Vec<PeerTrendPoint>> {
    let conn = state.database.connection().lock().unwrap();
    match get_peer_trend(&conn, &peer_id, protocol.as_deref()) {
        Ok(points) => IpcResult::ok(points),
        Err(e) => IpcResult::err(
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "history.get_trend".into(),
                code: "QUERY_FAILED".into(),
                message_key: "errors.NB-INTERNAL-001".into(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            }),
        ),
    }
}

#[tauri::command]
pub fn history_compare_cohort(
    state: State<AppState>,
    session_id: Uuid,
) -> IpcResult<Option<CohortComparisonResult>> {
    let conn = state.database.connection().lock().unwrap();
    match compare_session_with_cohort(&conn, &session_id) {
        Ok(res) => IpcResult::ok(res),
        Err(e) => IpcResult::err(
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "history.compare_cohort".into(),
                code: "QUERY_FAILED".into(),
                message_key: "errors.NB-INTERNAL-001".into(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            }),
        ),
    }
}

#[tauri::command]
pub fn history_repeat_plan(
    state: State<AppState>,
    session_id: Uuid,
) -> IpcResult<RepeatPlanConfig> {
    let conn = state.database.connection().lock().unwrap();
    match reconstruct_repeat_plan(&conn, &session_id) {
        Ok(config) => IpcResult::ok(config),
        Err(err) => IpcResult::err(err),
    }
}
