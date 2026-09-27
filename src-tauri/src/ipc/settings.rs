use crate::app::AppState;
use crate::ipc::response::IpcResult;
use crate::platform::autostart;
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
    /// Hash corto del commit con el que se compiló este ejecutable concreto (`build.rs`).
    /// `app_version` (de `Cargo.toml`) no cambia entre sesiones de desarrollo, así que por
    /// sí sola no basta para saber si dos ordenadores tienen el mismo build (hallazgo real
    /// del propietario, comparando dos máquinas).
    pub build_hash: String,
    /// Fecha del commit compilado, en ISO 8601 (no la fecha de compilación: esa cambiaría en
    /// cada `cargo build` aunque el código fuera idéntico, y no ayudaría a comparar builds).
    pub build_date: String,
    /// Había cambios sin comitear en el árbol de trabajo en el momento de compilar.
    pub build_dirty: bool,
}

#[tauri::command]
pub fn settings_get(state: tauri::State<AppState>) -> IpcResult<Preferences> {
    IpcResult::ok(state.settings.get())
}

#[tauri::command]
pub async fn settings_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    preferences: Preferences,
) -> Result<IpcResult<Preferences>, String> {
    let puerto_anterior = state.puerto_de_control();
    let ajuste_anterior = state.settings.get().custom_control_port;
    let session_state = state.session_service.current_state().await;
    // `Completed` y `Failed` son terminales: una sesión que ya acabó no bloquea los ajustes.
    let is_session_active = session_state.is_active();

    let res = state.settings.validate_and_update(is_session_active, |p| {
        *p = preferences;
    });

    match res {
        Ok(saved) => {
            // Cambiar el puerto de control se aplica al instante (§23): se reabre el canal en
            // el nuevo. Si no se puede (puerto ocupado), se deshace el cambio: nunca queda
            // guardado un puerto en el que no se está escuchando.
            let puerto_nuevo = saved
                .custom_control_port
                .unwrap_or(crate::discovery::CONTROL_PORT_DEFAULT);
            if puerto_nuevo != puerto_anterior {
                let emisor: std::sync::Arc<dyn crate::ipc::events::EmisorDeEventos> =
                    std::sync::Arc::new(app);
                if let Err(e) = state.reabrir_control(Some(emisor), puerto_nuevo).await {
                    tracing::warn!("No se pudo reabrir el canal de control en {puerto_nuevo}: {e}");
                    let _ = state
                        .settings
                        .update(|p| p.custom_control_port = ajuste_anterior);
                    return Ok(IpcResult::err(crate::errors::AppError::from_code(
                        crate::errors::ErrorCode::PortControlInUse,
                    )));
                }
                // Se vuelve a anunciar por mDNS con el puerto nuevo.
                state.reiniciar_descubrimiento();
            }
            // El ajuste se aplica al instante (`Historias.md` §7.1): apagarlo retira el
            // anuncio y deja de navegar; encenderlo publica y vuelve a navegar.
            state.aplicar_descubrimiento(saved.mdns_enabled);
            // El nivel de registro también se aplica al instante, sin reiniciar.
            if let Some(logger) = crate::logging::get_logger() {
                logger.set_level(saved.log_level);
            }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticPaths {
    /// Carpeta de registro de la aplicación (`networkbench.log`, `tracing.log`): el perfil
    /// del usuario que la abrió, no un directorio del proyecto (hallazgo real del
    /// propietario: buscaba los registros dentro del repositorio, donde la app nunca
    /// escribe).
    pub app_log_dir: String,
    /// El ayudante elevado de firewall escribe su propio registro aparte, en %ProgramData%
    /// en vez de en el perfil del usuario: corre elevado, a veces con las credenciales de
    /// OTRA cuenta (el UAC de "ejecutar como" cuando la cuenta actual no es administradora),
    /// así que %LOCALAPPDATA% podría no ser el de quien abrió la aplicación.
    pub firewall_helper_log_path: String,
}

#[tauri::command]
pub fn settings_diagnostic_paths() -> IpcResult<DiagnosticPaths> {
    let app_log_dir = crate::app::dirs_or_fallback().join("logs");
    let firewall_helper_log_dir = std::env::var_os("ProgramData")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\ProgramData"))
        .join("NetworkBench")
        .join("logs");
    IpcResult::ok(DiagnosticPaths {
        app_log_dir: app_log_dir.to_string_lossy().to_string(),
        firewall_helper_log_path: firewall_helper_log_dir
            .join("firewall-helper.log")
            .to_string_lossy()
            .to_string(),
    })
}

/// Abre la carpeta de registro de la aplicación en el Explorador. La del ayudante
/// (%ProgramData%) se abre igual desde ahí mismo: son carpetas hermanas.
#[tauri::command]
pub fn settings_open_log_folder() -> IpcResult<()> {
    let dir = crate::app::dirs_or_fallback().join("logs");
    let _ = fs::create_dir_all(&dir);
    #[cfg(target_os = "windows")]
    {
        let raiz = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
        let explorador = std::path::PathBuf::from(raiz).join("explorer.exe");
        if let Err(e) = std::process::Command::new(explorador).arg(&dir).spawn() {
            return IpcResult::err(
                crate::errors::AppError::from_code(crate::errors::ErrorCode::InternalError)
                    .with_diagnostic_id(format!("No se pudo abrir la carpeta de registro: {e}")),
            );
        }
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
        build_hash: env!("NB_GIT_HASH").to_string(),
        build_date: env!("NB_GIT_DATE").to_string(),
        build_dirty: env!("NB_GIT_DIRTY") == "true",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_hash_de_compilacion_no_esta_vacio_ni_se_queda_fijo_como_la_version_de_cargo() {
        let IpcResult::Success(info) = settings_about_info() else {
            panic!("settings_about_info debe tener éxito siempre");
        };
        // No es una comprobación de contenido exacto (el hash cambia en cada commit): lo
        // que importa es que build.rs de verdad rellenó algo y no dejó la variable vacía.
        assert!(!info.build_hash.is_empty());
        assert_ne!(
            info.build_hash, "desconocido",
            "debe correr dentro de un checkout git"
        );
        assert!(!info.build_date.is_empty());
        assert_ne!(info.app_version, info.build_hash);
    }

    #[test]
    fn las_rutas_de_diagnostico_apuntan_a_carpetas_distintas_para_app_y_ayudante() {
        let IpcResult::Success(rutas) = settings_diagnostic_paths() else {
            panic!("settings_diagnostic_paths debe tener éxito siempre");
        };
        assert!(rutas.app_log_dir.ends_with("logs"));
        assert!(rutas.app_log_dir.contains("NetworkBench"));
        assert!(rutas.firewall_helper_log_path.contains("ProgramData"));
        assert!(
            rutas
                .firewall_helper_log_path
                .contains("firewall-helper.log")
        );
        // No es la misma carpeta: una es del perfil de usuario, la otra de todo el equipo
        // (hallazgo real del propietario: el ayudante puede correr con otra cuenta por UAC).
        assert_ne!(rutas.app_log_dir, rutas.firewall_helper_log_path);
    }
}
