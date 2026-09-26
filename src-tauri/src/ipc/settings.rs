use crate::app::AppState;
use crate::ipc::response::IpcResult;
use crate::platform::autostart;
use crate::platform::lifecycle::{CloseActionDecision, evaluate_close_request};
use crate::settings::Preferences;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataInfo {
    pub db_path: String,
    pub db_size_bytes: u64,
    pub settings_path: String,
    pub settings_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AboutInfo {
    pub app_name: String,
    pub app_version: String,
    pub engine_name: String,
    pub engine_version: String,
    pub protocol_version: String,
    pub license: String,
    pub copyright: String,
}

#[tauri::command]
pub fn settings_get(state: tauri::State<AppState>) -> IpcResult<Preferences> {
    IpcResult::ok(state.settings.get())
}

#[tauri::command]
pub async fn settings_update(
    state: tauri::State<'_, AppState>,
    preferences: Preferences,
) -> Result<IpcResult<Preferences>, String> {
    let session_state = state.session_service.current_state().await;
    // `Completed` y `Failed` son terminales: una sesión que ya acabó no bloquea los ajustes.
    let is_session_active = session_state.is_active();

    let res = state.settings.validate_and_update(is_session_active, |p| {
        *p = preferences;
    });

    match res {
        Ok(saved) => {
            // El ajuste se aplica al instante (`Historias.md` §7.1): apagarlo retira el
            // anuncio y deja de navegar; encenderlo publica y vuelve a navegar.
            state.aplicar_descubrimiento(saved.mdns_enabled);
            state.aviso_snapshot.notify_one();
            Ok(IpcResult::ok(saved))
        }
        Err(e) => Ok(IpcResult::err(e)),
    }
}

#[tauri::command]
pub fn settings_autostart_get() -> IpcResult<bool> {
    match autostart::is_autostart_enabled() {
        Ok(enabled) => IpcResult::ok(enabled),
        Err(e) => {
            crate::logging::log(
                crate::logging::LogLevel::Warn,
                "settings.autostart",
                &format!("Error consultando autoarranque: {}", e),
            );
            IpcResult::ok(false)
        }
    }
}

#[tauri::command]
pub fn settings_autostart_set(enabled: bool) -> IpcResult<()> {
    match autostart::set_autostart(enabled) {
        Ok(_) => IpcResult::ok(()),
        Err(e) => {
            let app_err = crate::errors::AppError::new(
                crate::errors::ErrorCode::InternalError,
                crate::errors::ErrorSeverity::Warning,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "settings.autostart".to_string(),
                code: "AUTOSTART_SET_FAILED".to_string(),
                message_key: "errors.NB-INTERNAL-001".to_string(),
                safe_params: Some(serde_json::json!({ "details": e })),
            });
            IpcResult::err(app_err)
        }
    }
}

#[tauri::command]
pub fn settings_data_info(state: tauri::State<AppState>) -> IpcResult<DataInfo> {
    let base_dir = crate::app::dirs_or_fallback();
    let db_path = base_dir.join("history.db");
    let settings_path = base_dir.join("settings.json");

    let db_size_bytes = fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let settings_size_bytes = fs::metadata(&settings_path).map(|m| m.len()).unwrap_or(0);

    let _ = state;

    IpcResult::ok(DataInfo {
        db_path: db_path.to_string_lossy().to_string(),
        db_size_bytes,
        settings_path: settings_path.to_string_lossy().to_string(),
        settings_size_bytes,
    })
}

#[tauri::command]
pub fn settings_data_purge(state: tauri::State<AppState>) -> IpcResult<()> {
    let base_dir = crate::app::dirs_or_fallback();
    let db_path = base_dir.join("history.db");

    let _ = state;

    // Purgar archivo de BD SQLite
    if db_path.exists() {
        let _ = fs::remove_file(&db_path);
    }

    IpcResult::ok(())
}

#[tauri::command]
pub fn settings_about_info() -> IpcResult<AboutInfo> {
    IpcResult::ok(AboutInfo {
        app_name: "NetworkBench".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        engine_name: "Microsoft NTTTCP".to_string(),
        engine_version: "5.35".to_string(),
        protocol_version: "v1".to_string(),
        license: "GPL-3.0-or-later".to_string(),
        copyright: "© 2026 Daniel Díez Mardomingo y colaboradores".to_string(),
    })
}

#[tauri::command]
pub async fn app_close_evaluate(
    state: tauri::State<'_, AppState>,
    force: bool,
) -> Result<IpcResult<CloseActionDecision>, String> {
    let is_session_active = state.session_service.current_state().await.is_active();
    let minimize_to_tray = state.settings.get().minimize_to_tray;

    let decision = evaluate_close_request(is_session_active, minimize_to_tray, force);
    Ok(IpcResult::ok(decision))
}
