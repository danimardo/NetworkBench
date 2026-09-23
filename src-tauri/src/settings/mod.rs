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
    pub minimize_to_tray: bool,
    #[serde(default = "default_mdns_enabled")]
    pub mdns_enabled: bool,
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
            minimize_to_tray: false,
            mdns_enabled: true,
        }
    }
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

        match serde_json::from_str::<Preferences>(&content) {
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
