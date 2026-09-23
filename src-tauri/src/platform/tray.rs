use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrayAction {
    ToggleWindow,
    ShowStatus,
    ExitApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayState {
    pub is_session_active: bool,
    pub tooltip: String,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            is_session_active: false,
            tooltip: "NetworkBench - Medidor de Rendimiento de Red".to_string(),
        }
    }
}

/// Genera el texto del tooltip para la bandeja del sistema según el estado actual
pub fn get_tray_tooltip(is_session_active: bool) -> String {
    if is_session_active {
        "NetworkBench: Prueba de red en curso...".to_string()
    } else {
        "NetworkBench: Listo".to_string()
    }
}
