//! Comandos del diálogo de cierre de la ventana (`Historias.md` §5.2).
//!
//! La interfaz solo **muestra** las preguntas y devuelve lo que la persona eligió; quien
//! decide y ejecuta (ocultar, cancelar la prueba, salir) es el backend.

use super::response::IpcResult;
use crate::errors::{AppError, ErrorCode};
use crate::platform::cierre;
use serde::Deserialize;
use tauri::AppHandle;

/// Lo que se elige en el diálogo «¿Qué quieres hacer?».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EleccionAlCerrar {
    /// «Minimizar a la bandeja».
    Minimize,
    /// «Cerrar NetworkBench».
    Exit,
}

/// Aplica la elección del diálogo. Con `remember`, queda como ajuste y no se vuelve a preguntar.
#[tauri::command]
pub async fn app_close_apply(
    app: AppHandle,
    choice: EleccionAlCerrar,
    remember: bool,
) -> Result<IpcResult<()>, String> {
    Ok(
        match cierre::aplicar_eleccion(&app, choice == EleccionAlCerrar::Exit, remember).await {
            Ok(()) => IpcResult::ok(()),
            Err(detalle) => IpcResult::err(
                AppError::from_code(ErrorCode::InternalError).with_diagnostic_id(detalle),
            ),
        },
    )
}

/// La persona ha confirmado salir con una prueba en curso: se cancela y se sale.
#[tauri::command]
pub async fn app_close_confirmed(app: AppHandle) -> Result<IpcResult<()>, String> {
    cierre::cancelar_prueba_y_salir(&app).await;
    Ok(IpcResult::ok(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_eleccion_llega_de_la_interfaz_como_minimize_o_exit() {
        let m: EleccionAlCerrar = serde_json::from_str("\"minimize\"").unwrap();
        let s: EleccionAlCerrar = serde_json::from_str("\"exit\"").unwrap();
        assert_eq!((m, s), (EleccionAlCerrar::Minimize, EleccionAlCerrar::Exit));
        assert!(serde_json::from_str::<EleccionAlCerrar>("\"cerrar\"").is_err());
    }
}
