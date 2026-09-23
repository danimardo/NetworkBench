use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub log_level: Option<String>,
    pub custom_ntttcp_path: Option<PathBuf>,
    pub custom_data_dir: Option<PathBuf>,
}

impl AppConfig {
    /// Carga la configuración validando únicamente las variables de entorno allowlisted
    pub fn load_from_env() -> Result<Self, String> {
        let log_level = env::var("NB_LOG_LEVEL")
            .or_else(|_| env::var("LOG_LEVEL"))
            .ok()
            .map(|s| s.to_lowercase());

        if let Some(ref level) = log_level {
            match level.as_str() {
                "trace" | "debug" | "info" | "warn" | "error" => {}
                other => return Err(format!("Nivel de log inválido en entorno: '{}'", other)),
            }
        }

        let custom_ntttcp_path = env::var("NB_NTTTCP_PATH").ok().map(PathBuf::from);
        let custom_data_dir = env::var("NB_DATA_DIR").ok().map(PathBuf::from);

        Ok(Self {
            log_level,
            custom_ntttcp_path,
            custom_data_dir,
        })
    }
}
