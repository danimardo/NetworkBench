//! Despacho de las conexiones entrantes del canal de control.
//!
//! `ControlServer::accept_one` deja la conexión en el punto en que ya hay TLS mutuo y
//! HELLO, y nada más: no sabe si el par viene a emparejarse o a proponer una prueba. Este
//! módulo lee el primer mensaje, mira su tipo y entrega la conexión a quien corresponde.
//!
//! Hasta esta tarea, el bucle del servidor registraba el saludo y soltaba la conexión: un
//! equipo podía conectar y saludar, pero ninguna solicitud tenía a nadie que la atendiera.

use crate::control::pairing_incoming::atender_emparejamiento_entrante;
use crate::control::server::SaludoEntrante;
use crate::control::service::{ContextoSesion, SessionService};
use crate::control::transport::recv_envelope;
use crate::model::protocol::{
    PairRequestPayload, ProtocolEnvelope, ProtocolMessageType, RequestPayload,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Un par que conecta y calla no debe retener un socket indefinidamente (FR-060).
const PLAZO_PRIMER_MENSAJE: Duration = Duration::from_secs(15);

pub async fn despachar(servicio: Arc<SessionService>, ctx: ContextoSesion, saludo: SaludoEntrante) {
    let SaludoEntrante {
        fingerprint,
        hello,
        remote_addr,
        mut stream,
    } = saludo;

    let primero: ProtocolEnvelope<serde_json::Value> =
        match timeout(PLAZO_PRIMER_MENSAJE, recv_envelope(&mut stream)).await {
            Ok(Ok(sobre)) => sobre,
            Ok(Err(e)) => {
                tracing::warn!("Conexión de {remote_addr} cerrada antes de su primer mensaje: {e}");
                return;
            }
            Err(_) => {
                tracing::warn!("Conexión de {remote_addr} sin ningún mensaje en 15 s; se cierra");
                return;
            }
        };

    match primero.msg_type {
        ProtocolMessageType::Request => {
            // El payload llegó como JSON genérico para poder mirar el tipo antes de
            // comprometerse con una forma; ahora se valida contra la de `REQUEST`.
            let payload: RequestPayload = match serde_json::from_value(primero.payload) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("REQUEST de {remote_addr} con forma no válida: {e}");
                    return;
                }
            };
            let tipado = ProtocolEnvelope {
                msg_type: primero.msg_type,
                id: primero.id,
                session_id: primero.session_id,
                ts: primero.ts,
                in_reply_to: primero.in_reply_to,
                payload,
            };
            servicio
                .atender_sesion_entrante(ctx, fingerprint, remote_addr, stream, tipado)
                .await;
        }
        ProtocolMessageType::PairRequest => {
            // Aceptar sin que una persona compare los códigos vaciaría de sentido la
            // verificación (FR-012): el flujo entrante espera su decisión (T182).
            let payload: PairRequestPayload = match serde_json::from_value(primero.payload) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("PAIR_REQUEST de {remote_addr} con forma no válida: {e}");
                    return;
                }
            };
            let tipado = ProtocolEnvelope {
                msg_type: primero.msg_type,
                id: primero.id,
                session_id: primero.session_id,
                ts: primero.ts,
                in_reply_to: primero.in_reply_to,
                payload,
            };
            atender_emparejamiento_entrante(
                &ctx,
                &hello,
                &fingerprint,
                remote_addr,
                &mut stream,
                tipado,
            )
            .await;
        }
        otro => {
            tracing::warn!("Primer mensaje inesperado de {remote_addr}: {otro:?}");
        }
    }
}
