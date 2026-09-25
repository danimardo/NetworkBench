//! Muestreo en vivo durante una pata de la prueba (T174, FR-026, FR-044).
//!
//! NTTTCP entrega su resultado en un XML al terminar: mientras corre no dice nada. Para
//! que la interfaz vea avanzar la medición, este módulo lee cada 500 ms los contadores de
//! octetos de la interfaz por la que viaja el tráfico y convierte el incremento en bits
//! por segundo.
//!
//! **Lo que estas muestras son y lo que no.** Son una lectura del adaptador entero, no
//! del proceso del motor: cualquier otro tráfico del equipo por esa interfaz se suma. Sirven
//! para dibujar la evolución, nunca como cifra de la prueba: la velocidad oficial sigue
//! siendo la del receptor según NTTTCP (FR-027). Tampoco traen CPU (`cpu_percent: None`):
//! no hay todavía una fuente para ella y no se inventa.
//!
//! Si una lectura falla, o el contador retrocede (el adaptador se reinició), se emite una
//! muestra marcada como hueco, no un valor supuesto.

use super::aggregate::SampleBatcher;
use super::samples::SamplePoint;
use crate::ipc::events::EmisorDeEventos;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{Instant, MissedTickBehavior};

/// Octetos acumulados por una interfaz desde que existe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contadores {
    pub bytes_recibidos: u64,
    pub bytes_enviados: u64,
}

pub trait FuenteDeContadores: Send {
    fn leer(&mut self) -> Result<Contadores, String>;
}

/// Resuelve la interfaz que usa el tráfico hacia un equipo remoto y devuelve su lector.
pub trait ProveedorDeContadores: Send + Sync {
    fn abrir(&self, remoto: IpAddr) -> Result<Box<dyn FuenteDeContadores>, String>;
}

/// Qué contador refleja esta pata en este extremo: el emisor mira lo que sale, el
/// receptor lo que entra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sentido {
    Emision,
    Recepcion,
}

/// Lo que la sesión necesita para muestrear: adónde avisar y de dónde leer.
#[derive(Clone)]
pub struct Muestreo {
    pub emisor: Arc<dyn EmisorDeEventos>,
    pub contadores: Arc<dyn ProveedorDeContadores>,
}

/// Bits por segundo entre dos lecturas, o `None` si no se puede afirmar (el contador
/// retrocedió o el intervalo es nulo).
pub fn bps_entre(anterior: u64, actual: u64, intervalo: Duration) -> Option<u64> {
    let micros = intervalo.as_micros();
    if actual < anterior || micros == 0 {
        return None;
    }
    let bits = u128::from(actual - anterior) * 8;
    Some((bits * 1_000_000 / micros) as u64)
}

fn elegir(c: Contadores, sentido: Sentido) -> u64 {
    match sentido {
        Sentido::Emision => c.bytes_enviados,
        Sentido::Recepcion => c.bytes_recibidos,
    }
}

/// Muestreo de una pata en marcha. Se detiene con `detener`, que vacía el último lote.
/// Si se suelta sin detenerlo, la tarea se aborta.
pub struct MuestreadorActivo {
    parar: Option<oneshot::Sender<()>>,
    tarea: Option<JoinHandle<()>>,
}

impl MuestreadorActivo {
    pub fn iniciar(
        mut fuente: Box<dyn FuenteDeContadores>,
        sentido: Sentido,
        session_id: String,
        direccion: &'static str,
        emisor: Arc<dyn EmisorDeEventos>,
        intervalo: Duration,
    ) -> Self {
        let (parar, mut parada) = oneshot::channel::<()>();

        let tarea = tokio::spawn(async move {
            let inicio = Instant::now();
            let mut batcher = SampleBatcher::new(session_id, direccion);
            let mut previo = fuente.leer().ok().map(|c| (inicio, elegir(c, sentido)));

            let mut reloj = tokio::time::interval_at(inicio + intervalo, intervalo);
            reloj.set_missed_tick_behavior(MissedTickBehavior::Delay);

            let mut ultimo_t = 0u64;
            loop {
                tokio::select! {
                    _ = &mut parada => break,
                    ahora = reloj.tick() => {
                        let t_ms = ahora.duration_since(inicio).as_millis() as u64;
                        ultimo_t = t_ms;

                        let lectura = fuente.leer().ok().map(|c| elegir(c, sentido));
                        let bps = match (previo, lectura) {
                            (Some((antes, a)), Some(b)) => bps_entre(a, b, ahora.duration_since(antes)),
                            _ => None,
                        };
                        previo = lectura.map(|b| (ahora, b));

                        let muestra = SamplePoint {
                            t_ms,
                            direction: direccion.to_string(),
                            bps: bps.unwrap_or(0),
                            cpu_percent: None,
                            gap: bps.is_none(),
                        };
                        if let Some(lote) = batcher.push(muestra)
                            && let Err(e) = emisor.emitir_muestras(&lote)
                        {
                            tracing::warn!("No se pudo emitir un lote de muestras: {e}");
                        }
                    }
                }
            }

            if let Some(lote) = batcher.flush(ultimo_t)
                && let Err(e) = emisor.emitir_muestras(&lote)
            {
                tracing::warn!("No se pudo emitir el último lote de muestras: {e}");
            }
        });

        Self {
            parar: Some(parar),
            tarea: Some(tarea),
        }
    }

    pub async fn detener(mut self) {
        if let Some(parar) = self.parar.take() {
            let _ = parar.send(());
        }
        if let Some(tarea) = self.tarea.take() {
            let _ = tarea.await;
        }
    }
}

impl Drop for MuestreadorActivo {
    fn drop(&mut self) {
        if let Some(tarea) = self.tarea.take() {
            tarea.abort();
        }
    }
}

/// Lectura real de los contadores de una interfaz de Windows (`GetIfEntry2`).
pub struct ContadoresWindows;

/// `IF_TYPE_SOFTWARE_LOOPBACK` de `ipifcons.h`.
const TIPO_BUCLE_LOCAL: u32 = 24;

struct LectorDeInterfaz {
    indice: u32,
}

impl LectorDeInterfaz {
    fn fila(&self) -> Result<windows::Win32::NetworkManagement::IpHelper::MIB_IF_ROW2, String> {
        use windows::Win32::NetworkManagement::IpHelper::{GetIfEntry2, MIB_IF_ROW2};

        let mut fila = MIB_IF_ROW2 {
            InterfaceIndex: self.indice,
            ..Default::default()
        };
        // SAFETY: `fila` es un MIB_IF_ROW2 válido y vive durante la llamada; la API solo
        // escribe en él y solo lee `InterfaceIndex`.
        let estado = unsafe { GetIfEntry2(&mut fila) };
        if estado.0 != 0 {
            return Err(format!(
                "GetIfEntry2 falló para la interfaz {}: {}",
                self.indice, estado.0
            ));
        }
        Ok(fila)
    }

    fn tipo(&self) -> Result<u32, String> {
        Ok(self.fila()?.Type)
    }
}

impl FuenteDeContadores for LectorDeInterfaz {
    fn leer(&mut self) -> Result<Contadores, String> {
        let fila = self.fila()?;
        Ok(Contadores {
            bytes_recibidos: fila.InOctets,
            bytes_enviados: fila.OutOctets,
        })
    }
}

impl ProveedorDeContadores for ContadoresWindows {
    fn abrir(&self, remoto: IpAddr) -> Result<Box<dyn FuenteDeContadores>, String> {
        use windows::Win32::NetworkManagement::IpHelper::GetBestInterface;

        // IPv6 queda fuera hasta que el motor lo admita (T179): sin `-6` no habría tráfico
        // IPv6 que muestrear, y mirar la interfaz equivocada sería peor que no mirar.
        let IpAddr::V4(v4) = remoto else {
            return Err("El muestreo en vivo solo admite IPv4 por ahora".to_string());
        };

        let mut indice = 0u32;
        // La API espera la dirección con los octetos en orden de red tal como están en
        // memoria, que es lo que da `from_ne_bytes` sobre `octets()`.
        let destino = u32::from_ne_bytes(v4.octets());
        // SAFETY: `indice` es un u32 válido que la API rellena.
        let estado = unsafe { GetBestInterface(destino, &mut indice) };
        if estado != 0 {
            return Err(format!(
                "No se pudo determinar la interfaz hacia {v4}: error {estado}"
            ));
        }
        let lector = LectorDeInterfaz { indice };
        // La interfaz de bucle local de Windows no incrementa sus contadores (comprobado
        // moviendo ~200 MB por ella: los dos quedan a 0). Muestrearla emitiría ceros con
        // aspecto de medición, así que se declara que no hay muestreo y la sesión sigue.
        if lector.tipo()? == TIPO_BUCLE_LOCAL {
            return Err(format!(
                "La interfaz hacia {v4} es de bucle local y no expone contadores"
            ));
        }
        Ok(Box::new(lector))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampling::aggregate::SampleBatch;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct Guion(VecDeque<Result<Contadores, String>>);
    impl FuenteDeContadores for Guion {
        fn leer(&mut self) -> Result<Contadores, String> {
            self.0.pop_front().unwrap_or(Err("sin más datos".into()))
        }
    }

    #[derive(Default)]
    struct Recoge(Mutex<Vec<SampleBatch>>);
    impl EmisorDeEventos for Recoge {
        fn emitir_muestras(&self, batch: &SampleBatch) -> Result<(), String> {
            self.0.lock().unwrap().push(batch.clone());
            Ok(())
        }
    }

    fn c(enviados: u64, recibidos: u64) -> Result<Contadores, String> {
        Ok(Contadores {
            bytes_enviados: enviados,
            bytes_recibidos: recibidos,
        })
    }

    #[test]
    fn bps_entre_convierte_octetos_en_bits_por_segundo() {
        // 62,5 MB en 500 ms son 1 Gbps.
        assert_eq!(
            bps_entre(0, 62_500_000, Duration::from_millis(500)),
            Some(1_000_000_000)
        );
    }

    #[test]
    fn bps_entre_no_inventa_un_valor_si_el_contador_retrocede() {
        assert_eq!(bps_entre(10_000, 5, Duration::from_millis(500)), None);
        assert_eq!(bps_entre(0, 1, Duration::ZERO), None);
    }

    const CORTO: Duration = Duration::from_millis(20);

    fn todas(emisor: &Recoge) -> Vec<SamplePoint> {
        emisor
            .0
            .lock()
            .unwrap()
            .iter()
            .flat_map(|l| l.samples.clone())
            .collect()
    }

    #[tokio::test]
    async fn el_emisor_mide_lo_que_sale_y_el_receptor_lo_que_entra() {
        for (sentido, debe_ser_positivo) in [(Sentido::Emision, true), (Sentido::Recepcion, false)]
        {
            let emisor = Arc::new(Recoge::default());
            // Salen 50 000 octetos por lectura; no entra nada.
            let fuente = Guion(VecDeque::from([c(0, 0), c(50_000, 0), c(100_000, 0)]));
            let activo = MuestreadorActivo::iniciar(
                Box::new(fuente),
                sentido,
                "s-1".into(),
                "forward",
                emisor.clone(),
                CORTO,
            );
            tokio::time::sleep(Duration::from_millis(400)).await;
            activo.detener().await;

            let muestras = todas(&emisor);
            assert!(muestras.len() >= 2, "{sentido:?}: {}", muestras.len());
            for m in &muestras[..2] {
                assert!(!m.gap, "{sentido:?}");
                assert_eq!(m.bps > 0, debe_ser_positivo, "{sentido:?}");
                assert!(m.cpu_percent.is_none());
            }
            let lotes = emisor.0.lock().unwrap();
            assert_eq!(lotes[0].session_id, "s-1");
            assert_eq!(lotes[0].direction, "forward");
        }
    }

    #[tokio::test]
    async fn una_lectura_fallida_es_un_hueco_y_la_siguiente_no_arrastra_el_hueco() {
        let emisor = Arc::new(Recoge::default());
        let fuente = Guion(VecDeque::from([
            c(0, 0),
            Err("adaptador desaparecido".into()),
            c(200_000, 0),
            c(250_000, 0),
        ]));
        let activo = MuestreadorActivo::iniciar(
            Box::new(fuente),
            Sentido::Emision,
            "s-2".into(),
            "reverse",
            emisor.clone(),
            CORTO,
        );
        tokio::time::sleep(Duration::from_millis(400)).await;
        activo.detener().await;

        let muestras = todas(&emisor);
        assert!(muestras.len() >= 3);
        // Lecturas: base, fallo, 200 000, 250 000. La 1.ª muestra usa (base, fallo).
        assert!(muestras[0].gap && muestras[0].bps == 0);
        // Tras el fallo no hay lectura previa: la siguiente tampoco se puede afirmar.
        assert!(muestras[1].gap);
        assert!(!muestras[2].gap && muestras[2].bps > 0);
        assert!(emisor.0.lock().unwrap().iter().any(|l| l.has_gaps));
    }

    #[tokio::test]
    async fn detener_vacia_lo_pendiente_y_no_deja_la_tarea_viva() {
        let emisor = Arc::new(Recoge::default());
        let fuente = Guion(VecDeque::from([c(0, 0), c(1_000, 0)]));
        let activo = MuestreadorActivo::iniciar(
            Box::new(fuente),
            Sentido::Emision,
            "s-3".into(),
            "forward",
            emisor.clone(),
            Duration::from_secs(3_600),
        );
        // Con un intervalo de una hora no llega a haber ninguna muestra: `detener` debe
        // volver enseguida y sin emitir nada inventado.
        tokio::time::timeout(Duration::from_secs(2), activo.detener())
            .await
            .expect("detener no debe esperar al siguiente tick");
        assert!(todas(&emisor).is_empty());
    }

    #[test]
    fn el_bucle_local_se_rechaza_en_vez_de_muestrear_ceros() {
        let e = ContadoresWindows
            .abrir("127.0.0.1".parse().unwrap())
            .err()
            .expect("el bucle local no debe ofrecer muestreo");
        assert!(e.contains("bucle local"), "{e}");
    }

    #[test]
    fn ipv6_no_se_finge_soportado() {
        assert!(ContadoresWindows.abrir("::1".parse().unwrap()).is_err());
    }
}
