//! Ejecuta el cierre de la ventana: ocultar, salir, avisar a la interfaz (`Historias.md` §5.2).
//!
//! Toda vía de cierre (botón de la barra propia, Alt+F4, «Cerrar ventana» de la barra de
//! tareas) llega a `solicitar_cierre` por el mismo sitio: `WindowEvent::CloseRequested`,
//! interceptado con `prevent_close` en `lib.rs`. Nunca hay dos comportamientos según cómo
//! se cierre. Antes el cierre dependía de que la interfaz llamara a `window.destroy()` y,
//! sin ese permiso, el botón no hacía nada.

use super::lifecycle::{AccionAlCerrar, decidir_cierre, salir_o_confirmar};
use super::tray::Bandeja;
use crate::app::AppState;
use crate::settings::CloseAction;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

/// La interfaz debe preguntar: minimizar a la bandeja o cerrar.
pub const EVENTO_PREGUNTA: &str = "app://close-ask";
/// La interfaz debe confirmar que se cancelará la prueba en curso al salir.
pub const EVENTO_CONFIRMAR_SESION: &str = "app://close-confirm-session";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Pregunta {
    session_active: bool,
}

/// Mínimo para guardar la geometría: por debajo es una ventana minimizada o a medio crear.
const ANCHO_MINIMO: u32 = 800;
const ALTO_MINIMO: u32 = 600;

fn ventana(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window("main")
}

/// Muestra la ventana, la restaura si estaba minimizada y le da el foco.
pub fn mostrar_ventana(app: &AppHandle) {
    if let Some(w) = ventana(app) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Guarda la geometría (§16.9) en el mismo formato que la interfaz: posición y tamaño
/// interiores en píxeles físicos.
fn guardar_geometria(app: &AppHandle) {
    let Some(w) = ventana(app) else { return };
    let (Ok(pos), Ok(tam), Ok(maximizada)) = (w.inner_position(), w.inner_size(), w.is_maximized())
    else {
        return;
    };
    if tam.width < ANCHO_MINIMO || tam.height < ALTO_MINIMO {
        return;
    }
    let geometria = super::window::WindowGeometry {
        x: pos.x,
        y: pos.y,
        width: tam.width,
        height: tam.height,
        is_maximized: maximizada,
    };
    let estado = app.state::<AppState>();
    if let Err(e) = estado
        .settings
        .update(|p| p.window_geometry = Some(geometria))
    {
        tracing::warn!("No se pudo guardar la geometría de la ventana: {e:?}");
    }
}

/// Oculta la ventana. Sin icono de bandeja no habría cómo volver a abrirla: entonces se
/// minimiza a la barra de tareas, que sí se puede restaurar.
pub fn ocultar_ventana(app: &AppHandle) {
    guardar_geometria(app);
    let Some(w) = ventana(app) else { return };
    if app.try_state::<Bandeja>().is_some() {
        let _ = w.hide();
    } else {
        let _ = w.minimize();
    }
}

/// Sale del todo, guardando antes la geometría.
pub fn salir(app: &AppHandle) {
    guardar_geometria(app);
    app.exit(0);
}

async fn sesion_activa(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .session_service
        .current_state()
        .await
        .is_active()
}

/// Punto único de entrada del cierre de ventana.
pub async fn solicitar_cierre(app: &AppHandle) {
    let activa = sesion_activa(app).await;
    let ajuste = app.state::<AppState>().settings.get().close_action;
    ejecutar(app, decidir_cierre(activa, ajuste), activa);
}

/// «Salir» del menú de la bandeja. Con una prueba en curso, confirma como cualquier otro cierre.
pub async fn salir_desde_la_bandeja(app: &AppHandle) {
    let activa = sesion_activa(app).await;
    ejecutar(app, salir_o_confirmar(activa), activa);
}

/// Lo elegido en el diálogo de la interfaz. `recordar` fija el ajuste y no se vuelve a preguntar.
pub async fn aplicar_eleccion(
    app: &AppHandle,
    salir_del_todo: bool,
    recordar: bool,
) -> Result<(), String> {
    let estado = app.state::<AppState>();
    if recordar {
        let nuevo = if salir_del_todo {
            CloseAction::Exit
        } else {
            CloseAction::Minimize
        };
        estado
            .settings
            .update(|p| p.close_action = nuevo)
            .map_err(|e| format!("No se pudo guardar la elección: {e:?}"))?;
        estado.aviso_snapshot.notify_one();
    }
    let activa = sesion_activa(app).await;
    let accion = if salir_del_todo {
        salir_o_confirmar(activa)
    } else {
        AccionAlCerrar::OcultarEnBandeja
    };
    ejecutar(app, accion, activa);
    Ok(())
}

/// La persona ha confirmado salir aun con una prueba en curso: se cancela antes (§10.8).
pub async fn cancelar_prueba_y_salir(app: &AppHandle) {
    let servicio = &app.state::<AppState>().session_service;
    if let Some(id) = servicio.active_session_id().await
        && let Err(e) = servicio.cancel(id).await
    {
        tracing::warn!("No se pudo cancelar la prueba al salir: {e:?}");
    }
    salir(app);
}

fn ejecutar(app: &AppHandle, accion: AccionAlCerrar, sesion_activa: bool) {
    match accion {
        AccionAlCerrar::Preguntar => {
            mostrar_ventana(app);
            let _ = app.emit(
                EVENTO_PREGUNTA,
                Pregunta {
                    session_active: sesion_activa,
                },
            );
        }
        AccionAlCerrar::ConfirmarCancelarSesion => {
            mostrar_ventana(app);
            let _ = app.emit(EVENTO_CONFIRMAR_SESION, ());
        }
        AccionAlCerrar::OcultarEnBandeja => ocultar_ventana(app),
        AccionAlCerrar::Salir => salir(app),
    }
}
