use super::response::IpcResult;
use crate::app::AppState;
use crate::logging::diagnostics::DiagnosticSanitizer;
use crate::logging::{LogEvent, LogLevel, LogOrigin, format_madrid_human, sanitize_value};
use crate::model::result::SessionResult;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::SystemTime;
use tauri::State;

#[tauri::command]
pub async fn diagnostics_get_report(
    result: SessionResult,
    _state: State<'_, AppState>,
) -> Result<IpcResult<String>, String> {
    let report = DiagnosticSanitizer::build_technical_report(&result);
    Ok(IpcResult::ok(report))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendLogEvent {
    pub level: LogLevel,
    pub module: String,
    pub event_code: String,
    pub message: String,
    pub error_code: Option<String>,
    pub diagnostic_id: Option<String>,
    #[serde(default)]
    pub safe_params: HashMap<String, String>,
}

/// Aparte de `diagnostics_log_frontend_event` para poder probar el mapeo de campos sin
/// depender de si `GLOBAL_LOGGER` (un `OnceLock` de todo el proceso) ya lo inicializó otro
/// test — con eso no hay forma fiable de comprobar en qué fichero acabaría escribiendo.
fn evento_a_log_event(evento: FrontendLogEvent) -> LogEvent {
    let safe_params = evento
        .safe_params
        .into_iter()
        .map(|(k, v)| {
            let saneado = sanitize_value(&k, &v);
            (k, saneado)
        })
        .collect();
    LogEvent {
        schema_version: 1,
        timestamp: format_madrid_human(SystemTime::now()),
        level: evento.level,
        origin: LogOrigin::Frontend,
        module: evento.module,
        event_code: evento.event_code,
        message: evento.message,
        error_code: evento.error_code,
        duration_ms: None,
        diagnostic_id: evento.diagnostic_id,
        safe_params,
    }
}

/// Puente para que los errores que ve la persona en pantalla (`IpcError` del lado del
/// frontend, incluidos los que nunca llegaron a construirse como `AppError` en Rust:
/// fallos de `invoke()`, de validación Zod...) dejen el mismo rastro que los del backend.
///
/// Hallazgo real de la auditoría de logging (2026-09-28): `logger.setBridge(...)` existía
/// en el frontend desde el principio, pero nadie lo llamaba nunca — `logger.error(...)`
/// solo llegaba a la consola del navegador (invisible en la app empaquetada) y a ningún
/// fichero. Este comando, más `setBridge` registrándolo al arrancar, es lo que le da un
/// destino real.
#[tauri::command]
pub fn diagnostics_log_frontend_event(evento: FrontendLogEvent) -> IpcResult<()> {
    if let Some(logger) = crate::logging::get_logger() {
        logger.log(evento_a_log_event(evento));
    }
    IpcResult::ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_evento_del_frontend_se_marca_como_origen_frontend_y_sanea_parametros() {
        let evento = FrontendLogEvent {
            level: LogLevel::Error,
            module: "transport".to_string(),
            event_code: "IPC_ERROR".to_string(),
            message: "firewall_rules_status: errors.NB-FW-003".to_string(),
            error_code: Some("NB-FW-003".to_string()),
            diagnostic_id: Some("detalle".to_string()),
            safe_params: HashMap::from([
                ("cmd".to_string(), "firewall_rules_status".to_string()),
                ("token".to_string(), "no-debe-salir".to_string()),
            ]),
        };
        let log_event = evento_a_log_event(evento);

        assert_eq!(log_event.origin, LogOrigin::Frontend);
        assert_eq!(log_event.level, LogLevel::Error);
        assert_eq!(log_event.error_code.as_deref(), Some("NB-FW-003"));
        assert_eq!(log_event.diagnostic_id.as_deref(), Some("detalle"));
        assert_eq!(
            log_event.safe_params.get("cmd").map(String::as_str),
            Some("firewall_rules_status")
        );
        // El campo se llama "token": la lista blanca de sanitize_value lo redacta, igual
        // que ya hace para los eventos del propio backend.
        assert_eq!(
            log_event.safe_params.get("token").map(String::as_str),
            Some("[REDACTED]")
        );
    }
}
