use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::logging::LogLevel;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

pub const CURRENT_SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ThemeMode {
    System,
    Light,
    #[default]
    Dark,
}

/// Qué hace el botón de cerrar de la ventana (`Historias.md` §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloseAction {
    /// Pregunta cada vez: minimizar a la bandeja o cerrar.
    #[default]
    Ask,
    /// Oculta la ventana; la aplicación sigue escuchando desde la bandeja.
    Minimize,
    /// Sale del todo.
    Exit,
}

fn default_mdns_enabled() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub schema_version: u32,
    pub theme: ThemeMode,
    pub locale: String,
    pub reduce_motion: bool,
    pub log_level: LogLevel,
    pub auto_accept_trusted: bool,
    pub custom_control_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_geometry: Option<crate::platform::window::WindowGeometry>,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub close_action: CloseAction,
    #[serde(default = "default_mdns_enabled")]
    pub mdns_enabled: bool,
    /// Las reglas de firewall también valen en redes que Windows considera públicas
    /// (`Historias.md` §14.5). Apagado por defecto: la exposición mayor la decide la persona.
    #[serde(default)]
    pub firewall_allow_public: bool,
    /// Envío opt-in de diagnóstico a un servidor OpenObserve propio (constitución, enmienda
    /// 0.8.0 al principio IV). Apagado por defecto: enviar algo fuera de esta máquina es
    /// siempre una decisión explícita de la persona, nunca automática.
    #[serde(default)]
    pub open_observe_enabled: bool,
    #[serde(default)]
    pub open_observe_url: String,
    #[serde(default)]
    pub open_observe_org: String,
    #[serde(default)]
    pub open_observe_stream: String,
    /// El token de ingesta que genera OpenObserve (ya es un `Basic <base64>` completo, no
    /// una contraseña que haya que combinar con nada). No es un secreto de la aplicación
    /// —lo emite el servidor de la persona—, pero tampoco se registra nunca en ningún log.
    #[serde(default)]
    pub open_observe_token: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SETTINGS_VERSION,
            theme: ThemeMode::Dark,
            locale: "es".to_string(),
            reduce_motion: false,
            log_level: LogLevel::Warn,
            auto_accept_trusted: false,
            custom_control_port: None,
            window_geometry: None,
            autostart: false,
            close_action: CloseAction::Ask,
            mdns_enabled: true,
            firewall_allow_public: false,
            open_observe_enabled: false,
            open_observe_url: String::new(),
            open_observe_org: String::new(),
            open_observe_stream: String::new(),
            open_observe_token: String::new(),
        }
    }
}

/// Los ajustes guardados antes de `closeAction` tenían un interruptor `minimizeToTray` que
/// no llegó a hacer nada (no había bandeja). Quien lo dejó encendido pidió minimizar al
/// cerrar: se conserva esa intención. Con `closeAction` ya presente no se toca nada.
fn migrar_ajustes_antiguos(contenido: &str) -> String {
    let Ok(mut valor) = serde_json::from_str::<serde_json::Value>(contenido) else {
        return contenido.to_string();
    };
    let Some(objeto) = valor.as_object_mut() else {
        return contenido.to_string();
    };
    let quiere_bandeja = objeto.get("minimizeToTray").and_then(|v| v.as_bool()) == Some(true);
    if quiere_bandeja && !objeto.contains_key("closeAction") {
        objeto.insert("closeAction".into(), "minimize".into());
    }
    valor.to_string()
}

pub struct SettingsStore {
    file_path: PathBuf,
    current: Mutex<Preferences>,
}

impl SettingsStore {
    pub fn new(file_path: PathBuf) -> Self {
        let prefs = Self::load_or_recover(&file_path);
        Self {
            file_path,
            current: Mutex::new(prefs),
        }
    }

    pub fn get(&self) -> Preferences {
        self.current.lock().unwrap().clone()
    }

    pub fn update<F>(&self, mutate: F) -> Result<Preferences, AppError>
    where
        F: FnOnce(&mut Preferences),
    {
        self.validate_and_update(false, mutate)
    }

    pub fn validate_and_update<F>(
        &self,
        is_session_active: bool,
        mutate: F,
    ) -> Result<Preferences, AppError>
    where
        F: FnOnce(&mut Preferences),
    {
        let mut lock = self.current.lock().unwrap();
        let prev = lock.clone();
        let mut candidate = prev.clone();
        mutate(&mut candidate);

        if is_session_active {
            let network_critical_changed = candidate.custom_control_port
                != prev.custom_control_port
                || candidate.auto_accept_trusted != prev.auto_accept_trusted
                || candidate.mdns_enabled != prev.mdns_enabled;

            if network_critical_changed {
                return Err(AppError::new(
                    ErrorCode::PeerBusy,
                    ErrorSeverity::Warning,
                    "errors.NB-PEER-003",
                )
                .with_issue(crate::errors::AppIssue {
                    path: "settings".to_string(),
                    code: "SESSION_ACTIVE_REJECTED".to_string(),
                    message_key: "errors.NB-PEER-003".to_string(),
                    safe_params: Some(serde_json::json!({
                        "reason": "Cannot modify network/control settings while benchmark session is active"
                    })),
                }));
            }
        }

        *lock = candidate;
        self.atomic_save(&self.file_path, &lock)?;
        Ok(lock.clone())
    }

    fn load_or_recover(file_path: &Path) -> Preferences {
        if !file_path.exists() {
            let default_prefs = Preferences::default();
            let _ = Self::atomic_save_internal(file_path, &default_prefs);
            return default_prefs;
        }

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => {
                Self::quarantine_corrupt(file_path);
                let default_prefs = Preferences::default();
                let _ = Self::atomic_save_internal(file_path, &default_prefs);
                return default_prefs;
            }
        };

        // Comprobar si es un JSON válido con esquema futuro antes de deserializar Preferences
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(v) = val.get("schemaVersion").and_then(|v| v.as_u64())
            && v as u32 > CURRENT_SETTINGS_VERSION
        {
            // Esquema futuro: no sobrescribir en disco, usar defaults en memoria
            return Preferences::default();
        }

        match serde_json::from_str::<Preferences>(&migrar_ajustes_antiguos(&content)) {
            Ok(prefs) => prefs,
            Err(_) => {
                // Archivo corrupto o malformado: poner en cuarentena explícita y regenerar defaults
                Self::quarantine_corrupt(file_path);
                let default_prefs = Preferences::default();
                let _ = Self::atomic_save_internal(file_path, &default_prefs);
                default_prefs
            }
        }
    }

    fn quarantine_corrupt(file_path: &Path) {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let quarantine_path = file_path.with_extension(format!("corrupt.{}.bak", ts));
        let _ = fs::rename(file_path, quarantine_path);
    }

    pub fn atomic_save(&self, file_path: &Path, prefs: &Preferences) -> Result<(), AppError> {
        Self::atomic_save_internal(file_path, prefs)
    }

    fn atomic_save_internal(file_path: &Path, prefs: &Preferences) -> Result<(), AppError> {
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let tmp_path = file_path.with_extension("tmp");
        let json_data = serde_json::to_string_pretty(prefs).map_err(|e| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "settings.serialize".to_string(),
                code: "SERIALIZATION_FAILED".to_string(),
                message_key: "errors.NB-INTERNAL-001".to_string(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            })
        })?;

        {
            let mut tmp_file = File::create(&tmp_path).map_err(|e| {
                AppError::new(
                    ErrorCode::InternalError,
                    ErrorSeverity::Error,
                    "errors.NB-INTERNAL-001",
                )
                .with_issue(crate::errors::AppIssue {
                    path: "settings.create_tmp".to_string(),
                    code: "TMP_CREATE_FAILED".to_string(),
                    message_key: "errors.NB-INTERNAL-001".to_string(),
                    safe_params: Some(serde_json::json!({ "details": e.to_string() })),
                })
            })?;

            tmp_file.write_all(json_data.as_bytes()).map_err(|e| {
                AppError::new(
                    ErrorCode::InternalError,
                    ErrorSeverity::Error,
                    "errors.NB-INTERNAL-001",
                )
                .with_issue(crate::errors::AppIssue {
                    path: "settings.write_tmp".to_string(),
                    code: "TMP_WRITE_FAILED".to_string(),
                    message_key: "errors.NB-INTERNAL-001".to_string(),
                    safe_params: Some(serde_json::json!({ "details": e.to_string() })),
                })
            })?;

            tmp_file.sync_all().map_err(|e| {
                AppError::new(
                    ErrorCode::InternalError,
                    ErrorSeverity::Error,
                    "errors.NB-INTERNAL-001",
                )
                .with_issue(crate::errors::AppIssue {
                    path: "settings.sync".to_string(),
                    code: "SYNC_FAILED".to_string(),
                    message_key: "errors.NB-INTERNAL-001".to_string(),
                    safe_params: Some(serde_json::json!({ "details": e.to_string() })),
                })
            })?;
        }

        // Reemplazo atómico en disco (en Windows se sobrescribe si existe usando rename seguro)
        if file_path.exists() {
            let _ = fs::remove_file(file_path);
        }
        fs::rename(&tmp_path, file_path).map_err(|e| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Error,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "settings.rename".to_string(),
                code: "RENAME_FAILED".to_string(),
                message_key: "errors.NB-INTERNAL-001".to_string(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            })
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lo que manda el frontend en `settings_update` (esquema Zod → JSON camelCase): sin
    /// `windowGeometry` cuando no hay ninguna y con `customControlPort: null`.
    const DEL_FRONTEND: &str = r#"{
        "schemaVersion": 1, "theme": "dark", "locale": "en", "reduceMotion": false,
        "logLevel": "info", "autoAcceptTrusted": true, "customControlPort": null,
        "autostart": false, "closeAction": "minimize", "mdnsEnabled": false
    }"#;

    #[test]
    fn acepta_las_preferencias_tal_como_las_envia_el_frontend() {
        let p: Preferences = serde_json::from_str(DEL_FRONTEND).expect("deserializar");
        assert_eq!(p.locale, "en");
        assert_eq!(p.log_level, LogLevel::Info);
        assert!(p.auto_accept_trusted && !p.mdns_enabled);
        assert_eq!(p.close_action, CloseAction::Minimize);
        assert_eq!(p.custom_control_port, None);
        assert_eq!(p.window_geometry, None);
    }

    #[test]
    fn el_nivel_de_registro_acepta_todos_los_valores_que_ofrece_el_frontend() {
        for nivel in ["trace", "debug", "info", "warn", "error"] {
            let json = DEL_FRONTEND.replace("\"info\"", &format!("\"{nivel}\""));
            let p: Preferences =
                serde_json::from_str(&json).unwrap_or_else(|e| panic!("{nivel}: {e}"));
            assert_eq!(p.log_level.as_str(), nivel);
        }
    }

    #[test]
    fn guardar_y_releer_conserva_lo_que_se_cambio() {
        let dir = std::env::temp_dir().join(format!("nb_settings_{}", uuid::Uuid::new_v4()));
        let ruta = dir.join("settings.json");
        let almacen = SettingsStore::new(ruta.clone());

        let guardadas = almacen
            .validate_and_update(false, |p| {
                *p = serde_json::from_str(DEL_FRONTEND).unwrap();
            })
            .expect("guardar");
        assert!(guardadas.auto_accept_trusted);

        // Un almacén nuevo sobre el mismo fichero (= reabrir la app) ve lo guardado.
        let releidas = SettingsStore::new(ruta).get();
        assert!(releidas.auto_accept_trusted);
        assert_eq!(releidas.close_action, CloseAction::Minimize);
        assert_eq!(releidas.locale, "en");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn por_defecto_al_cerrar_pregunta() {
        assert_eq!(Preferences::default().close_action, CloseAction::Ask);
    }

    #[test]
    fn el_interruptor_antiguo_de_bandeja_se_convierte_en_minimizar() {
        let antiguo = r#"{"schemaVersion":1,"theme":"dark","locale":"es","reduceMotion":false,
            "logLevel":"warn","autoAcceptTrusted":false,"customControlPort":null,
            "autostart":false,"minimizeToTray":true,"mdnsEnabled":true}"#;
        let p: Preferences = serde_json::from_str(&migrar_ajustes_antiguos(antiguo)).unwrap();
        assert_eq!(p.close_action, CloseAction::Minimize);

        // Apagado (lo normal: nunca hizo nada), sigue preguntando.
        let apagado = antiguo.replace("\"minimizeToTray\":true", "\"minimizeToTray\":false");
        let p: Preferences = serde_json::from_str(&migrar_ajustes_antiguos(&apagado)).unwrap();
        assert_eq!(p.close_action, CloseAction::Ask);

        // Con closeAction ya presente manda ese: la migración no pisa una elección nueva.
        let nuevo = antiguo.replace(
            "\"minimizeToTray\":true",
            "\"minimizeToTray\":true,\"closeAction\":\"exit\"",
        );
        let p: Preferences = serde_json::from_str(&migrar_ajustes_antiguos(&nuevo)).unwrap();
        assert_eq!(p.close_action, CloseAction::Exit);
    }

    #[test]
    fn un_fichero_antiguo_con_el_interruptor_encendido_arranca_en_minimizar() {
        let dir = std::env::temp_dir().join(format!("nb_settings_mig_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let ruta = dir.join("settings.json");
        std::fs::write(
            &ruta,
            r#"{"schemaVersion":1,"theme":"dark","locale":"es","reduceMotion":false,
                "logLevel":"warn","autoAcceptTrusted":false,"customControlPort":null,
                "autostart":false,"minimizeToTray":true,"mdnsEnabled":true}"#,
        )
        .unwrap();
        assert_eq!(
            SettingsStore::new(ruta).get().close_action,
            CloseAction::Minimize
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn el_nombre_en_el_json_es_el_que_espera_el_frontend() {
        let json = serde_json::to_value(Preferences::default()).unwrap();
        assert_eq!(json["closeAction"], "ask");
        assert!(json.get("minimizeToTray").is_none());
    }
}
