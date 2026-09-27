//! Icono y menú de la bandeja del sistema (`Historias.md` §5.2).
//!
//! Menú: «Abrir NetworkBench», el estado («Disponible» / «Prueba en curso») y «Salir». Un clic
//! simple en el icono restaura la ventana. «Salir» pasa por la misma confirmación que el
//! resto de cierres si hay una prueba en curso.

use super::cierre;
use crate::ipc::snapshot::AppSnapshot;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

const ID_ABRIR: &str = "abrir";
const ID_ESTADO: &str = "estado";
const ID_SALIR: &str = "salir";

/// Los textos del menú, en el idioma de la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Textos {
    pub abrir: &'static str,
    pub disponible: &'static str,
    pub en_curso: &'static str,
    pub salir: &'static str,
}

pub fn textos(idioma: &str) -> Textos {
    match idioma {
        "en" => Textos {
            abrir: "Open NetworkBench",
            disponible: "Available",
            en_curso: "Test in progress",
            salir: "Exit",
        },
        _ => Textos {
            abrir: "Abrir NetworkBench",
            disponible: "Disponible",
            en_curso: "Prueba en curso",
            salir: "Salir",
        },
    }
}

impl Textos {
    pub fn estado(&self, sesion_activa: bool) -> &'static str {
        if sesion_activa {
            self.en_curso
        } else {
            self.disponible
        }
    }
}

/// Texto que se ve al pasar el ratón por el icono.
pub fn tooltip(idioma: &str, sesion_activa: bool) -> String {
    format!("NetworkBench: {}", textos(idioma).estado(sesion_activa))
}

/// El icono y los elementos de menú cuyo texto cambia. Se guarda como estado de Tauri para
/// poder actualizarlo cuando cambian la sesión o el idioma.
pub struct Bandeja {
    icono: TrayIcon<Wry>,
    abrir: MenuItem<Wry>,
    estado: MenuItem<Wry>,
    salir: MenuItem<Wry>,
}

pub fn crear(app: &AppHandle, idioma: &str) -> tauri::Result<Bandeja> {
    let t = textos(idioma);
    let abrir = MenuItem::with_id(app, ID_ABRIR, t.abrir, true, None::<&str>)?;
    // Deshabilitado: es una lectura de estado, no una acción.
    let estado = MenuItem::with_id(app, ID_ESTADO, t.disponible, false, None::<&str>)?;
    let salir = MenuItem::with_id(app, ID_SALIR, t.salir, true, None::<&str>)?;
    let separador = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&abrir, &estado, &separador, &salir])?;

    let mut constructor = TrayIconBuilder::with_id("principal")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip(tooltip(idioma, false))
        .on_menu_event(|app, evento| match evento.id().as_ref() {
            ID_ABRIR => cierre::mostrar_ventana(app),
            ID_SALIR => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    cierre::salir_desde_la_bandeja(&app).await;
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|icono, evento| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = evento
            {
                cierre::mostrar_ventana(icono.app_handle());
            }
        });
    if let Some(icono) = app.default_window_icon().cloned() {
        constructor = constructor.icon(icono);
    }

    Ok(Bandeja {
        icono: constructor.build(app)?,
        abrir,
        estado,
        salir,
    })
}

/// Pone al día textos y tooltip con el snapshot (idioma y sesión activa).
pub fn refrescar(app: &AppHandle, snapshot: &AppSnapshot) {
    let Some(bandeja) = app.try_state::<Bandeja>() else {
        return;
    };
    let t = textos(&snapshot.locale);
    let _ = bandeja.abrir.set_text(t.abrir);
    let _ = bandeja
        .estado
        .set_text(t.estado(snapshot.is_session_active));
    let _ = bandeja.salir.set_text(t.salir);
    let _ = bandeja
        .icono
        .set_tooltip(Some(tooltip(&snapshot.locale, snapshot.is_session_active)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_textos_siguen_al_idioma_y_ante_uno_desconocido_van_en_espanol() {
        assert_eq!(textos("en").abrir, "Open NetworkBench");
        assert_eq!(textos("es").abrir, "Abrir NetworkBench");
        assert_eq!(textos("fr"), textos("es"));
    }

    #[test]
    fn el_estado_distingue_una_prueba_en_curso() {
        let t = textos("es");
        assert_eq!(t.estado(false), "Disponible");
        assert_eq!(t.estado(true), "Prueba en curso");
        assert_eq!(tooltip("en", true), "NetworkBench: Test in progress");
    }
}
