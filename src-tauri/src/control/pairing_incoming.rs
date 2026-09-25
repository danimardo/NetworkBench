//! Emparejamiento entrante: el lado que recibe un `PAIR_REQUEST` (T182, FR-012).
//!
//! Hasta esta tarea, `despachador` registraba el mensaje y lo descartaba sin responder:
//! quien pedía emparejarse se quedaba esperando hasta su propio plazo, y dos equipos
//! reales no podían emparejarse nunca.
//!
//! Orden deliberado, por seguridad:
//!
//! 1. Se verifica el código y la huella **antes** de enseñar nada a la persona. Un código
//!    que no cuadra recibe `verificationFailed` sin preguntar, y sin distinguir por qué.
//! 2. La persona compara el código con el que muestra el otro equipo. Que cuadre no
//!    concede nada por sí solo.
//! 3. Solo con un sí explícito se guarda la confianza, y **antes** de contestar
//!    `accepted: true`: si guardar falla, no se afirma una aceptación que no ocurrió.
//!
//! La confianza que se concede es `Trusted` con autoaceptación desactivada (FR-015):
//! confiar en un equipo no es delegarle el consentimiento de cada prueba.

use crate::control::consent::EmparejamientoEntranteEvento;
use crate::control::pairing_flow::{contestar_emparejamiento, preparar_respuesta};
use crate::control::service::{ContextoSesion, normalizar_ip};
use crate::discovery::CONTROL_PORT_DEFAULT;
use crate::history::upsert_peer;
use crate::model::peer::{Peer, TrustState};
use crate::model::protocol::{HelloPayload, PairRequestPayload, ProtocolEnvelope};
use crate::netinfo::resolve::sanitize_display_name;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};

/// Menos que los 60 s que el iniciador espera el `PAIR_RESULT`
/// (`pairing_flow::PLAZO_DECISION`): si los dos plazos fueran iguales, la respuesta de
/// «sin decisión» llegaría justo cuando el otro lado ya se rindió.
pub const ESPERA_DECISION_EMPAREJAMIENTO: Duration = Duration::from_secs(55);

/// Atiende un `PAIR_REQUEST` ya leído del canal.
///
/// `huella` es la del certificado presentado en el handshake, no lo que el par diga de
/// sí mismo; `hello` es lo que declara y solo se usa, saneado, para mostrar y guardar.
pub async fn atender_emparejamiento_entrante<S>(
    ctx: &ContextoSesion,
    hello: &HelloPayload,
    huella: &str,
    remote_addr: SocketAddr,
    stream: &mut S,
    solicitud: ProtocolEnvelope<PairRequestPayload>,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (emparejamiento, verificacion) =
        match preparar_respuesta(&solicitud, &ctx.identity.fingerprint, huella) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("PAIR_REQUEST de {remote_addr} no utilizable: {e}");
                return;
            }
        };

    // Un código o una huella que no encajan no llegan a la persona.
    if verificacion.is_err() {
        tracing::warn!("PAIR_REQUEST de {remote_addr} con verificación fallida");
        let _ = contestar_emparejamiento(stream, solicitud.id, &verificacion, false).await;
        return;
    }

    // La dirección de vuelta: la IP desde la que llegó, en el puerto de control por
    // defecto. El puerto de origen del socket es efímero y no sirve. INFERIDO: el par
    // escucha en el puerto por defecto; el HELLO no lo declara.
    let direccion = SocketAddr::new(normalizar_ip(remote_addr.ip()), CONTROL_PORT_DEFAULT);
    let peer = match Peer::new(
        hello.instance_id,
        sanitize_display_name(&hello.display_name),
        huella.to_string(),
        vec![direccion.to_string()],
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("PAIR_REQUEST de {remote_addr} con datos de equipo no válidos: {e}");
            let _ = contestar_emparejamiento(stream, solicitud.id, &verificacion, false).await;
            return;
        }
    };

    // Un solo emparejamiento pendiente por equipo (`peer-protocol.md`, límites): quien
    // insiste mientras el anterior sigue esperando no acumula preguntas en la pantalla.
    if ctx
        .emparejamientos
        .listar()
        .await
        .iter()
        .any(|e| e.peer.fingerprint == peer.fingerprint)
    {
        tracing::warn!("Emparejamiento de {remote_addr} rechazado: ya hay uno pendiente");
        let _ = contestar_emparejamiento(stream, solicitud.id, &verificacion, false).await;
        return;
    }

    let evento = EmparejamientoEntranteEvento {
        pairing_id: emparejamiento.id,
        verification_code: emparejamiento.codigo.clone(),
        peer: peer.clone(),
    };
    let esperar = ctx
        .emparejamientos
        .registrar(evento.pairing_id, evento.clone())
        .await;

    if let Some(emisor) = &ctx.emisor_eventos
        && let Err(e) = emisor.emitir_emparejamiento_entrante(&evento)
    {
        tracing::warn!(
            "No se pudo avisar del emparejamiento entrante {}: {e}",
            evento.pairing_id
        );
    }

    let decidido = tokio::time::timeout(ESPERA_DECISION_EMPAREJAMIENTO, esperar).await;
    ctx.emparejamientos.retirar(evento.pairing_id).await;
    let mut aceptado = matches!(decidido, Ok(Ok(true)));

    if aceptado {
        let mut confiable = peer;
        confiable.trust_state = TrustState::Trusted;
        confiable.auto_accept = false;
        let guardado = match ctx.database.connection().lock() {
            Ok(db) => upsert_peer(&db, &confiable).map_err(|e| e.to_string()),
            Err(_) => Err("La base de datos está bloqueada".to_string()),
        };
        if let Err(e) = guardado {
            tracing::error!("No se pudo guardar el equipo emparejado: {e}");
            aceptado = false;
        }
    }

    let _ = contestar_emparejamiento(stream, solicitud.id, &verificacion, aceptado).await;
}
