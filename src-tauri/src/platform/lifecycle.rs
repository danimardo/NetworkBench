use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CloseActionDecision {
    AllowExit,
    RequireConfirmation,
    MinimizeToTray,
}

/// Determina qué acción debe tomarse al solicitar el cierre de la ventana
pub fn evaluate_close_request(
    is_session_active: bool,
    minimize_to_tray_enabled: bool,
    force: bool,
) -> CloseActionDecision {
    if force {
        return CloseActionDecision::AllowExit;
    }

    if is_session_active {
        // Por FR-023 y constitución, si hay sesión activa SIEMPRE se requiere
        // confirmación explícita del usuario para evitar interrupciones accidentales
        return CloseActionDecision::RequireConfirmation;
    }

    if minimize_to_tray_enabled {
        CloseActionDecision::MinimizeToTray
    } else {
        CloseActionDecision::AllowExit
    }
}
