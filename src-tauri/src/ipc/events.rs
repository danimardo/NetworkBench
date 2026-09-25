//! Emisión de eventos Tauri hacia el frontend (T150).
//!
//! Hasta esta tarea, el backend no emitía un solo evento: `src/lib/api/samples.ts`
//! escucha `session://sample-batch` desde que existe (T058), y nada lo disparaba nunca.
//!
//! El progreso general y los cambios de estado de sesión no tienen todavía un contrato
//! de evento definido en el frontend — solo existe el de muestras. Antes de inventar un
//! nombre y una forma de payload que nadie consume, se deja documentado como pendiente
//! (ver `VALIDACION.md`): el mecanismo real para eso hoy es la suscripción con
//! `revision` monotónica de `ipc/snapshot.rs` (T024), que ya cubre `isSessionActive` y
//! `activeSessionId`.
//!
//! El emisor va detrás de un trait, igual que el motor de medida (`engine_port.rs`):
//! así el código que agrupa y despacha muestras se prueba sin un `AppHandle` real.

use super::response::IpcResult;
use crate::control::consent::SolicitudEntranteEvento;
use crate::sampling::aggregate::SampleBatch;
use tauri::{AppHandle, Emitter};

pub const EVENTO_MUESTRAS: &str = "session://sample-batch";
pub const EVENTO_SOLICITUD_ENTRANTE: &str = "session://incoming-request";

/// Sale de la sesión de medida hacia el frontend. No decide cuándo hay un lote listo
/// —eso es `SampleBatcher`— solo lo entrega.
pub trait EmisorDeEventos: Send + Sync {
    fn emitir_muestras(&self, batch: &SampleBatch) -> Result<(), String>;

    /// Avisa de una solicitud de sesión entrante que espera decisión humana (T177,
    /// FR-016). Los dobles de prueba que no la necesitan heredan este no-op: no se
    /// convierten en un contrato roto solo por no comprobar consentimiento.
    fn emitir_solicitud_entrante(
        &self,
        _solicitud: &SolicitudEntranteEvento,
    ) -> Result<(), String> {
        Ok(())
    }
}

impl EmisorDeEventos for AppHandle {
    fn emitir_muestras(&self, batch: &SampleBatch) -> Result<(), String> {
        self.emit(EVENTO_MUESTRAS, batch)
            .map_err(|e| format!("No se pudo emitir {EVENTO_MUESTRAS}: {e}"))
    }

    fn emitir_solicitud_entrante(&self, solicitud: &SolicitudEntranteEvento) -> Result<(), String> {
        self.emit(EVENTO_SOLICITUD_ENTRANTE, solicitud)
            .map_err(|e| format!("No se pudo emitir {EVENTO_SOLICITUD_ENTRANTE}: {e}"))
    }
}

/// Envía un lote ya formado. Separado de `EmisorDeEventos::emitir_muestras` para poder
/// envolverlo en un `IpcResult` sin que cada llamador repita el mapeo de error.
pub fn despachar_lote(emisor: &dyn EmisorDeEventos, batch: &SampleBatch) -> IpcResult<()> {
    match emisor.emitir_muestras(batch) {
        Ok(()) => IpcResult::ok(()),
        Err(e) => IpcResult::err(crate::errors::AppError::new(
            crate::errors::ErrorCode::InternalError,
            crate::errors::ErrorSeverity::Warning,
            e,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampling::aggregate::SampleBatcher;
    use crate::sampling::samples::SamplePoint;
    use std::sync::Mutex;

    /// Doble de prueba: registra lo que se le entregó en vez de hablar con WebView2.
    #[derive(Default)]
    struct EmisorDePrueba {
        recibidos: Mutex<Vec<SampleBatch>>,
    }

    impl EmisorDeEventos for EmisorDePrueba {
        fn emitir_muestras(&self, batch: &SampleBatch) -> Result<(), String> {
            self.recibidos.lock().unwrap().push(batch.clone());
            Ok(())
        }
    }

    #[test]
    fn test_un_lote_agrupado_se_despacha_con_su_forma_exacta() {
        let mut batcher = SampleBatcher::new("sesion-1", "forward");
        let emisor = EmisorDePrueba::default();

        // Empuja hasta que el batcher entregue un lote (respeta MIN_BATCH_INTERVAL_MS).
        let lote = (0..).find_map(|i| {
            batcher.push(SamplePoint {
                t_ms: i * 260,
                direction: "forward".to_string(),
                bps: 900_000_000,
                cpu_percent: Some(12.0),
                gap: false,
            })
        });
        let lote = lote.expect("el batcher debe entregar un lote tras suficientes muestras");

        let resultado = despachar_lote(&emisor, &lote);
        assert!(resultado.is_ok());

        let recibidos = emisor.recibidos.lock().unwrap();
        assert_eq!(recibidos.len(), 1);
        // La forma debe coincidir campo a campo con `sampleBatchSchema` en
        // src/lib/api/samples.ts: sessionId, direction, samples, latestBps,
        // latestCpuPercent, hasGaps. Un cambio de nombre aquí rompería ese contrato.
        let json = serde_json::to_value(&recibidos[0]).unwrap();
        for campo in [
            "sessionId",
            "direction",
            "samples",
            "latestBps",
            "latestCpuPercent",
            "hasGaps",
        ] {
            assert!(
                json.get(campo).is_some(),
                "falta el campo {campo} que el frontend espera"
            );
        }
    }

    #[test]
    fn test_un_emisor_que_falla_no_hace_panico_ni_se_propaga_sin_control() {
        struct EmisorRoto;
        impl EmisorDeEventos for EmisorRoto {
            fn emitir_muestras(&self, _batch: &SampleBatch) -> Result<(), String> {
                Err("WebView2 no disponible".to_string())
            }
        }

        let mut batcher = SampleBatcher::new("sesion-2", "reverse");
        let lote = (0..)
            .find_map(|i| {
                batcher.push(SamplePoint {
                    t_ms: i * 260,
                    direction: "reverse".to_string(),
                    bps: 1,
                    cpu_percent: None,
                    gap: false,
                })
            })
            .unwrap();

        let resultado = despachar_lote(&EmisorRoto, &lote);
        assert!(
            !resultado.is_ok(),
            "un fallo de emisión debe reflejarse en el IpcResult"
        );
    }
}
