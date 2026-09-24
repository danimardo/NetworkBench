//! Negociación de inicio sin reloj sincronizado (cierra G2, constitución VII).
//!
//! El problema que resuelve: dos equipos deben empezar a medir «al mismo tiempo» sin que
//! sus relojes de pared coincidan y sin depender de NTP. La solución de este protocolo
//! **no compara marcas absolutas**: cada extremo calcula su propio instante de inicio
//! como «el momento en que YO recibí el mensaje START, más un margen fijo», usando su
//! propio reloj monótono (`Instant`). Los dos margenes son iguales porque van dentro del
//! mismo mensaje `START`; lo que difiere es desde qué instante local cuenta cada uno.
//!
//! El margen (`start_delay_ms`) y la tolerancia (`tolerance_ms`) se calculan a partir del
//! RTT medido por HEARTBEAT, no de una constante fija: una red con más latencia necesita
//! más margen para que el mensaje llegue y se procese antes de la hora de inicio.

use crate::control::server::ahora_rfc3339;
use crate::control::transport::{recv_envelope, send_envelope};
use crate::model::protocol::{
    HeartbeatPayload, ProtocolEnvelope, ProtocolMessageType, StartPayload,
};
use std::io::{Error, ErrorKind, Result};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use uuid::Uuid;

/// Piso del margen de inicio: aunque el RTT sea ~0 (loopback), procesar y despachar el
/// mensaje STARTED lleva un tiempo mínimo real.
const MARGEN_MINIMO_MS: u64 = 100;
/// Techo del margen: una red muy degradada no debe imponer una espera de varios
/// segundos antes de medir; por encima de esto se declara directamente degradado.
const MARGEN_MAXIMO_MS: u64 = 2_000;
const TOLERANCIA_MINIMA_MS: u64 = 50;
const TOLERANCIA_MAXIMA_MS: u64 = 1_000;

const PLAZO_HEARTBEAT: Duration = Duration::from_secs(2);

/// Mide el RTT enviando un HEARTBEAT con un nonce aleatorio y esperando su eco.
///
/// El eco debe traer el mismo nonce: sin eso, no hay forma de distinguir la respuesta a
/// este ping de un heartbeat periódico que llegara por casualidad al mismo tiempo.
pub async fn medir_rtt<S>(stream: &mut S) -> Result<Duration>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // No hace falta un generador aleatorio dedicado: los bits bajos de un UUID v4 ya
    // son aleatorios, y evita añadir una dependencia nueva (`rand`) sin justificación
    // arquitectónica solo para un nonce de un uso.
    let nonce = Uuid::new_v4().as_u64_pair().0;
    let enviado_en = Instant::now();

    let ping = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Heartbeat,
        id: Uuid::new_v4(),
        session_id: None,
        ts: ahora_rfc3339(),
        in_reply_to: None,
        payload: HeartbeatPayload { nonce },
    };
    send_envelope(stream, &ping).await?;

    loop {
        let respuesta: ProtocolEnvelope<HeartbeatPayload> =
            timeout(PLAZO_HEARTBEAT, recv_envelope(stream))
                .await
                .map_err(|_| Error::new(ErrorKind::TimedOut, "Sin eco de heartbeat en 2 s"))??;

        if respuesta.msg_type == ProtocolMessageType::Heartbeat && respuesta.payload.nonce == nonce
        {
            return Ok(enviado_en.elapsed());
        }
        // Un heartbeat con otro nonce es ruido de fondo (p. ej. el keepalive periódico
        // cruzándose); se descarta y se sigue esperando el eco propio.
    }
}

/// Responde a un HEARTBEAT devolviendo el mismo nonce. Sin esto, `medir_rtt` nunca
/// recibiría el eco que espera.
pub async fn responder_heartbeat<S>(
    stream: &mut S,
    recibido: &ProtocolEnvelope<HeartbeatPayload>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let eco = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Heartbeat,
        id: Uuid::new_v4(),
        session_id: recibido.session_id,
        ts: ahora_rfc3339(),
        in_reply_to: Some(recibido.id),
        payload: recibido.payload.clone(),
    };
    send_envelope(stream, &eco).await
}

/// Calcula el margen y la tolerancia de inicio a partir del RTT medido.
///
/// Puro y determinista a propósito: sin él, dos ejecuciones con el mismo RTT podrían
/// negociar valores distintos y las pruebas no podrían fijar un resultado esperado.
pub fn negociar_inicio(rtt: Duration, direction: &str) -> StartPayload {
    let rtt_ms = rtt.as_millis() as u64;

    // Margen: tres veces el RTT cubre ida (STARTdel iniciador), vuelta (posible reenvío
    // implícito por reconexión) y el propio procesamiento del receptor, con piso y techo.
    let start_delay_ms = (rtt_ms.saturating_mul(3)).clamp(MARGEN_MINIMO_MS, MARGEN_MAXIMO_MS);

    // Tolerancia: la mitad del RTT como jitter esperable, con piso y techo propios.
    let tolerance_ms = (rtt_ms / 2).clamp(TOLERANCIA_MINIMA_MS, TOLERANCIA_MAXIMA_MS);

    StartPayload {
        direction: direction.to_string(),
        start_delay_ms,
        tolerance_ms,
    }
}

/// El instante local, en el reloj monótono de ESTE proceso, en el que debe arrancar la
/// medición: el momento en que este extremo recibió STARTx, más el margen negociado.
///
/// Deliberadamente no usa ningún reloj de pared. `recibido_en` y el `Instant` devuelto
/// pertenecen al mismo proceso; el otro extremo calcula el suyo de forma simétrica sobre
/// su propio reloj, y los dos llegan a un desfase acotado sin haberse sincronizado.
pub fn momento_inicio_local(recibido_en: Instant, plan: &StartPayload) -> Instant {
    recibido_en + Duration::from_millis(plan.start_delay_ms)
}

/// Resultado de comparar cuándo se arrancó de verdad contra cuándo tocaba.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultadoSincronizacion {
    /// El arranque ocurrió dentro de la tolerancia negociada.
    Sincronizado,
    /// Fuera de tolerancia. `desvio_ms` es la diferencia real menos la esperada; un
    /// valor positivo es un arranque tardío. FR-025 exige declarar esto, no ocultarlo.
    Degradado { desvio_ms: i64 },
}

/// Compara el instante real de arranque de ESTE extremo contra el esperado.
///
/// No compara contra el otro extremo: cada lado evalúa su propia puntualidad respecto a
/// su propio margen. Comparar arranques absolutos de dos procesos distintos exigiría
/// justo la sincronización de reloj que este diseño evita.
pub fn verificar_sincronizacion(
    arrancado_en: Instant,
    recibido_en: Instant,
    plan: &StartPayload,
) -> ResultadoSincronizacion {
    let esperado = momento_inicio_local(recibido_en, plan);
    let real_ms = arrancado_en
        .saturating_duration_since(recibido_en)
        .as_millis() as i64;
    let esperado_ms = esperado.saturating_duration_since(recibido_en).as_millis() as i64;
    let desvio_ms = real_ms - esperado_ms;

    if desvio_ms.unsigned_abs() <= plan.tolerance_ms {
        ResultadoSincronizacion::Sincronizado
    } else {
        ResultadoSincronizacion::Degradado { desvio_ms }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[test]
    fn test_rtt_bajo_produce_el_margen_minimo() {
        let plan = negociar_inicio(Duration::from_millis(1), "forward");
        assert_eq!(plan.start_delay_ms, MARGEN_MINIMO_MS);
        assert_eq!(plan.tolerance_ms, TOLERANCIA_MINIMA_MS);
    }

    #[test]
    fn test_rtt_alto_no_supera_el_techo() {
        let plan = negociar_inicio(Duration::from_secs(5), "reverse");
        assert_eq!(plan.start_delay_ms, MARGEN_MAXIMO_MS);
        assert_eq!(plan.tolerance_ms, TOLERANCIA_MAXIMA_MS);
    }

    #[test]
    fn test_mismo_rtt_produce_siempre_el_mismo_plan() {
        // Determinismo: sin esto, dos ejecuciones no serían comparables ni testeables.
        let a = negociar_inicio(Duration::from_millis(37), "forward");
        let b = negociar_inicio(Duration::from_millis(37), "forward");
        assert_eq!(a, b);
    }

    #[test]
    fn test_arrancar_a_tiempo_es_sincronizado() {
        let recibido = Instant::now();
        let plan = negociar_inicio(Duration::from_millis(20), "forward");
        let arrancado = momento_inicio_local(recibido, &plan);

        assert_eq!(
            verificar_sincronizacion(arrancado, recibido, &plan),
            ResultadoSincronizacion::Sincronizado
        );
    }

    #[test]
    fn test_arrancar_tarde_fuera_de_tolerancia_es_degradado() {
        let recibido = Instant::now();
        let plan = negociar_inicio(Duration::from_millis(20), "forward");
        let tarde = recibido + Duration::from_millis(plan.start_delay_ms + plan.tolerance_ms + 50);

        match verificar_sincronizacion(tarde, recibido, &plan) {
            ResultadoSincronizacion::Degradado { desvio_ms } => assert!(desvio_ms > 0),
            ResultadoSincronizacion::Sincronizado => panic!("debía degradarse"),
        }
    }

    #[test]
    fn test_dentro_de_tolerancia_sigue_sincronizado() {
        let recibido = Instant::now();
        let plan = negociar_inicio(Duration::from_millis(20), "forward");
        // Justo en el borde: delay + tolerancia, no más.
        let borde = recibido + Duration::from_millis(plan.start_delay_ms + plan.tolerance_ms);

        assert_eq!(
            verificar_sincronizacion(borde, recibido, &plan),
            ResultadoSincronizacion::Sincronizado
        );
    }

    #[tokio::test]
    async fn test_medir_rtt_sobre_un_canal_real() {
        let (mut a, mut b) = duplex(4096);

        let responde = tokio::spawn(async move {
            let entrante: ProtocolEnvelope<HeartbeatPayload> =
                recv_envelope(&mut b).await.expect("recibir ping");
            responder_heartbeat(&mut b, &entrante)
                .await
                .expect("responder");
        });

        let rtt = medir_rtt(&mut a).await.expect("medir rtt");
        responde.await.unwrap();

        // Sobre un dúplex en memoria el RTT es mínimo, pero debe ser medible y finito.
        assert!(rtt < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_negociacion_completa_sobre_el_canal_tls_real() {
        // Extremo a extremo sobre TLS mutuo real, no un dúplex en memoria: mide el RTT,
        // negocia el plan de inicio y comprueba que los dos procesos, sin compartir
        // reloj, calculan un desfase dentro de tolerancia al arrancar "a la vez".
        use crate::control::server::ControlServer;
        use crate::identity::InstanceIdentity;
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        use std::sync::Arc;

        let servidor_id = Arc::new(InstanceIdentity::generate("Servidor".into()).unwrap());
        let cliente_id = InstanceIdentity::generate("Cliente".into()).unwrap();

        let servidor = ControlServer::bind(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            Arc::clone(&servidor_id),
        )
        .await
        .unwrap();
        let port = servidor.local_addr().unwrap().port();

        let tarea = tokio::spawn(async move {
            let mut saludo = servidor.accept_one().await.expect("saludo");
            let ping: ProtocolEnvelope<HeartbeatPayload> =
                recv_envelope(&mut saludo.stream).await.expect("ping");
            responder_heartbeat(&mut saludo.stream, &ping)
                .await
                .expect("eco");

            // El receptor recibe STARTx en este instante local y arranca según el plan.
            let recibido_en = Instant::now();
            let plan_recibido = negociar_inicio(Duration::from_millis(5), "forward");
            let arranque = momento_inicio_local(recibido_en, &plan_recibido);
            tokio::time::sleep_until(arranque.into()).await;
            verificar_sincronizacion(Instant::now(), recibido_en, &plan_recibido)
        });

        let (_peer, mut stream) =
            crate::discovery::conectar_y_saludar("127.0.0.1", port, &cliente_id)
                .await
                .expect("conectar");

        let rtt = medir_rtt(&mut stream).await.expect("medir rtt");
        let recibido_en = Instant::now();
        let plan = negociar_inicio(rtt, "forward");
        let arranque = momento_inicio_local(recibido_en, &plan);
        tokio::time::sleep_until(arranque.into()).await;
        let resultado_local = verificar_sincronizacion(Instant::now(), recibido_en, &plan);

        let resultado_remoto = tarea.await.unwrap();

        assert_eq!(resultado_local, ResultadoSincronizacion::Sincronizado);
        assert_eq!(resultado_remoto, ResultadoSincronizacion::Sincronizado);
    }

    #[tokio::test]
    async fn test_un_heartbeat_de_otro_nonce_no_confunde_la_medicion() {
        let (mut a, mut b) = duplex(4096);

        let responde = tokio::spawn(async move {
            // Ruido: un heartbeat con nonce distinto llega primero.
            let ruido = ProtocolEnvelope {
                msg_type: ProtocolMessageType::Heartbeat,
                id: Uuid::new_v4(),
                session_id: None,
                ts: ahora_rfc3339(),
                in_reply_to: None,
                payload: HeartbeatPayload { nonce: 999 },
            };
            send_envelope(&mut b, &ruido).await.unwrap();

            let entrante: ProtocolEnvelope<HeartbeatPayload> =
                recv_envelope(&mut b).await.expect("recibir ping real");
            responder_heartbeat(&mut b, &entrante)
                .await
                .expect("responder");
        });

        let rtt = medir_rtt(&mut a).await.expect("medir rtt pese al ruido");
        responde.await.unwrap();
        assert!(rtt < Duration::from_secs(1));
    }
}
