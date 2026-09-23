use crate::export::redact::redact_session_result;
use crate::model::result::SessionResult;

/// Exporta una o varias sesiones a formato JSON versionado.
/// Si `sessions` tiene 1 elemento, se exporta como un objeto `SessionResult`.
/// Si `sessions` tiene más de 1 elemento, se exporta como un array `[SessionResult, ...]`.
/// Si `anonymize` es true, se aplican las reglas de redacción de identificadores (§20, T106).
pub fn export_sessions_to_json(
    sessions: &[SessionResult],
    anonymize: bool,
) -> Result<String, String> {
    if sessions.is_empty() {
        return Err("No hay sesiones para exportar".to_string());
    }

    let processed: Vec<SessionResult> = sessions
        .iter()
        .map(|s| redact_session_result(s, anonymize))
        .collect();

    if processed.len() == 1 {
        serde_json::to_string_pretty(&processed[0])
            .map_err(|e| format!("Error al serializar sesión a JSON: {e}"))
    } else {
        serde_json::to_string_pretty(&processed)
            .map_err(|e| format!("Error al serializar array de sesiones a JSON: {e}"))
    }
}
