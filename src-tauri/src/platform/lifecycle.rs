//! Qué hacer cuando alguien pide cerrar la ventana (`Historias.md` §5.2).
//!
//! Aquí solo está la decisión, que es pura y se prueba sin ventana. Ejecutarla (ocultar,
//! salir, avisar a la interfaz) está en `cierre`.

use crate::settings::CloseAction;

/// Lo que debe ocurrir ante una petición de cierre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccionAlCerrar {
    /// Preguntar: minimizar a la bandeja o cerrar (diálogo de la interfaz).
    Preguntar,
    /// Hay una prueba en curso y se ha pedido salir: confirmar que se cancelará.
    ConfirmarCancelarSesion,
    /// Ocultar la ventana; la aplicación sigue escuchando desde la bandeja.
    OcultarEnBandeja,
    /// Salir del todo.
    Salir,
}

/// Decide según el ajuste «Al cerrar la ventana».
///
/// Minimizar nunca interrumpe una prueba; salir sí, así que con una prueba en curso siempre
/// se confirma antes (FR-023): por ajuste, por diálogo o desde la bandeja.
pub fn decidir_cierre(sesion_activa: bool, ajuste: CloseAction) -> AccionAlCerrar {
    match ajuste {
        CloseAction::Ask => AccionAlCerrar::Preguntar,
        CloseAction::Minimize => AccionAlCerrar::OcultarEnBandeja,
        CloseAction::Exit => salir_o_confirmar(sesion_activa),
    }
}

/// «Salir» venga de donde venga: con una prueba en curso, primero se confirma.
pub fn salir_o_confirmar(sesion_activa: bool) -> AccionAlCerrar {
    if sesion_activa {
        AccionAlCerrar::ConfirmarCancelarSesion
    } else {
        AccionAlCerrar::Salir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preguntar_pregunta_haya_o_no_una_prueba() {
        for activa in [false, true] {
            assert_eq!(
                decidir_cierre(activa, CloseAction::Ask),
                AccionAlCerrar::Preguntar
            );
        }
    }

    #[test]
    fn minimizar_oculta_y_nunca_interrumpe_una_prueba() {
        for activa in [false, true] {
            assert_eq!(
                decidir_cierre(activa, CloseAction::Minimize),
                AccionAlCerrar::OcultarEnBandeja
            );
        }
    }

    #[test]
    fn salir_sale_sin_prueba_y_confirma_con_prueba() {
        assert_eq!(
            decidir_cierre(false, CloseAction::Exit),
            AccionAlCerrar::Salir
        );
        assert_eq!(
            decidir_cierre(true, CloseAction::Exit),
            AccionAlCerrar::ConfirmarCancelarSesion
        );
    }

    #[test]
    fn salir_desde_la_bandeja_o_el_dialogo_confirma_solo_con_prueba() {
        assert_eq!(salir_o_confirmar(false), AccionAlCerrar::Salir);
        assert_eq!(
            salir_o_confirmar(true),
            AccionAlCerrar::ConfirmarCancelarSesion
        );
    }
}
