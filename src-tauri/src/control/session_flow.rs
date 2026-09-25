//! Diálogo completo de una sesión de medida entre dos peers (T144, FR-018 a FR-021).
//!
//! Hasta esta tarea existían todas las piezas por separado —el motor detrás de un
//! puerto (`engine_port.rs`), el ensamblado de resultado (`orquestador.rs`), la
//! negociación de inicio (`protocol.rs`)— y nada las unía en un camino de ejecución que
//! hable con un peer real. Este módulo es esa unión, para una dirección TCP secuencial.
//!
//! Flujo, sobre el canal TLS ya autenticado que deja `ControlServer`/`conectar_y_saludar`:
//!
//! ```text
//! Iniciador                          Receptor
//!   REQUEST(plan)          ------->
//!                          <-------  RESPONSE(accepted)
//!   PREPARE(direction,port) ------->
//!                                    (spawns el motor en rol Receiver, escucha)
//!                          <-------  READY(listening_port)
//!   START(delay,tolerance)  ------->
//!                          <-------  STARTED
//!   (ejecuta motor Sender)            (el motor Receiver ya en marcha completa)
//!   ENGINE_DONE(sender)     ------->
//!                          <-------  ENGINE_DONE(receiver)
//! ```
//!
//! **Alcance de esta tarea**: una dirección, TCP, secuencial. Bidireccional en la misma
//! sesión, UDP y simultáneo son extensiones de este mismo esqueleto, no rediseños; se
//! dejan para no inflar esta pieza más allá de lo que se puede probar con evidencia real
//! en esta pasada.

use crate::control::engine_port::MotorError;
use crate::control::orquestador::{Direccion, EntradaSesion, MedidasDireccion, Orquestador};
use crate::control::protocol::{
    medir_rtt, momento_inicio_local, negociar_inicio, responder_heartbeat,
};
use crate::control::server::ahora_rfc3339;
use crate::control::transport::{recv_envelope, send_envelope};
use crate::engine::ntttcp::parser::{NtttcpParsedResult, NtttcpRole};
use crate::model::peer::Peer;
use crate::model::plan::BenchmarkPlan;
use crate::model::protocol::{
    EngineDonePayload, PreparePayload, ProtocolEnvelope, ProtocolMessageType, ReadyPayload,
    RequestPayload, ResponsePayload, StartedPayload,
};
use crate::model::result::{DirectionResult, PeerSnapshot, SessionResult};
use crate::sampling::samples::SampleCollector;
use std::io::{Error, ErrorKind, Result as IoResult};
use tokio::io::{AsyncRead, AsyncWrite};
use uuid::Uuid;

/// Motivo de rechazo de un `REQUEST`. Nunca se envía texto libre del solicitante al
/// peer: solo estas categorías cerradas (FR-021, el receptor construye su propia
/// explicación a partir de campos permitidos, no del payload ajeno).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotivoRechazo {
    PlanInvalido,
    RechazadoPorElUsuario,
    OcupadoConOtraSesion,
}

impl MotivoRechazo {
    fn como_str(&self) -> &'static str {
        match self {
            Self::PlanInvalido => "invalidPlan",
            Self::RechazadoPorElUsuario => "rejectedByUser",
            Self::OcupadoConOtraSesion => "busy",
        }
    }
}

fn engine_done_a_parsed(role: NtttcpRole, p: &EngineDonePayload) -> Option<NtttcpParsedResult> {
    Some(NtttcpParsedResult {
        role,
        total_bytes: p.total_bytes.parse().ok()?,
        total_buffers: p.total_buffers.unwrap_or(0),
        realtime_seconds: p.realtime_seconds.unwrap_or(0.0),
        throughput_bps: p.throughput_bps.parse().ok()?,
        cpu_percent: p.cpu_percent,
        errors_count: p.errors_count.unwrap_or(0),
        packets_sent: None,
        packets_received: None,
        packets_retransmitted: None,
    })
}

fn parsed_a_engine_done(direction: &str, role: &str, r: &NtttcpParsedResult) -> EngineDonePayload {
    EngineDonePayload {
        direction: direction.to_string(),
        role: role.to_string(),
        throughput_bps: r.throughput_bps.to_string(),
        total_bytes: r.total_bytes.to_string(),
        total_buffers: Some(r.total_buffers),
        realtime_seconds: Some(r.realtime_seconds),
        cpu_percent: r.cpu_percent,
        errors_count: Some(r.errors_count),
    }
}

fn sobre<T>(session_id: Uuid, msg_type: ProtocolMessageType, payload: T) -> ProtocolEnvelope<T> {
    ProtocolEnvelope {
        msg_type,
        id: Uuid::new_v4(),
        session_id: Some(session_id),
        ts: ahora_rfc3339(),
        in_reply_to: None,
        payload,
    }
}

/// Lado que inicia una dirección: propone el plan, prepara al receptor, negocia el
/// inicio y ejecuta su mitad (emisor) del motor.
///
/// Devuelve el `DirectionResult` ensamblado con lo que este extremo sabe: su propia
/// medición y la que el receptor comunicó por `ENGINE_DONE`. `officialBps` sale de esa
/// medición del receptor, nunca de la propia (FR-027).
pub async fn iniciar_direccion<S>(
    stream: &mut S,
    orquestador: &Orquestador,
    session_id: Uuid,
    plan: &BenchmarkPlan,
    direccion: Direccion,
    target_host: &str,
) -> IoResult<DirectionResult>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // 1. REQUEST / RESPONSE
    send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::Request,
            RequestPayload {
                plan: plan.clone(),
                estimate_seconds: plan.warmup_seconds
                    + plan.measure_seconds
                    + plan.cooldown_seconds,
                suggested_interface: None,
            },
        ),
    )
    .await?;

    let respuesta: ProtocolEnvelope<ResponsePayload> = recv_envelope(stream).await?;
    if respuesta.msg_type != ProtocolMessageType::Response {
        return Err(protocolo_inesperado("RESPONSE", respuesta.msg_type));
    }
    if !respuesta.payload.accepted {
        // El receptor rechazó: la dirección no se inició, no es un fallo de esta capa.
        return Ok(DirectionResult::new_incomplete(direccion.como_str()));
    }

    // 2. PREPARE / READY
    send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::Prepare,
            PreparePayload {
                direction: direccion.como_str().to_string(),
                port: plan.port,
                streams: plan.streams,
            },
        ),
    )
    .await?;

    let listo: ProtocolEnvelope<ReadyPayload> = recv_envelope(stream).await?;
    if listo.msg_type != ProtocolMessageType::Ready {
        return Err(protocolo_inesperado("READY", listo.msg_type));
    }
    if !listo.payload.is_ready {
        return Ok(DirectionResult::new_incomplete(direccion.como_str()));
    }

    // 3. Negociación de inicio (G2, control::protocol) y START / STARTED
    let rtt = medir_rtt(stream).await?;
    let plan_inicio = negociar_inicio(rtt, direccion.como_str());
    send_envelope(
        stream,
        &sobre(session_id, ProtocolMessageType::Start, plan_inicio.clone()),
    )
    .await?;
    let recibido_en = std::time::Instant::now();

    let iniciado: ProtocolEnvelope<StartedPayload> = recv_envelope(stream).await?;
    if iniciado.msg_type != ProtocolMessageType::Started {
        return Err(protocolo_inesperado("STARTED", iniciado.msg_type));
    }

    // Arranca en el instante local negociado, no en cuanto llega STARTED: ese es
    // justamente el punto de G2 (constitución VII).
    tokio::time::sleep_until(momento_inicio_local(recibido_en, &plan_inicio).into()).await;

    // 4. Ejecuta la mitad local (emisor) y comparte el resultado.
    let local = orquestador
        .ejecutar_mitad_local(plan, NtttcpRole::Sender, Some(target_host))
        .await
        .map_err(motor_error_a_io)?;

    send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::EngineDone,
            parsed_a_engine_done(direccion.como_str(), "sender", &local),
        ),
    )
    .await?;

    let remoto: ProtocolEnvelope<EngineDonePayload> = recv_envelope(stream).await?;
    if remoto.msg_type != ProtocolMessageType::EngineDone {
        return Err(protocolo_inesperado("ENGINE_DONE", remoto.msg_type));
    }
    let receptor = engine_done_a_parsed(NtttcpRole::Receiver, &remoto.payload);

    let medidas = MedidasDireccion {
        direccion,
        emisor: Some(local),
        receptor,
        muestras: SampleCollector::new(),
    };
    Ok(orquestador.resultado_de_direccion(&medidas))
}

/// Lado que responde: valida el plan propuesto, prepara su mitad (receptor) y ejecuta.
///
/// `interfaz_local` es la dirección a la que el motor receptor se liga —no un destino
/// remoto—, seleccionada por preflight/netinfo antes de llegar aquí; nunca «127.0.0.1»
/// salvo en pruebas de bucle local.
///
/// `aceptar` decide si el plan se acepta más allá de su validez estructural: hoy puede
/// ser "siempre que sea válido", pero el punto de extensión existe para conectar
/// consentimiento explícito del usuario sin tocar el protocolo (FR-016).
pub async fn atender_direccion<S>(
    stream: &mut S,
    orquestador: &Orquestador,
    interfaz_local: &str,
    aceptar: impl FnOnce(&BenchmarkPlan) -> Result<(), MotivoRechazo>,
) -> IoResult<DirectionResult>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let solicitud: ProtocolEnvelope<RequestPayload> = recv_envelope(stream).await?;
    if solicitud.msg_type != ProtocolMessageType::Request {
        return Err(protocolo_inesperado("REQUEST", solicitud.msg_type));
    }
    let session_id = solicitud.session_id.unwrap_or_else(Uuid::new_v4);
    let plan = solicitud.payload.plan;
    let direccion = if plan.direction == crate::model::plan::BenchmarkDirection::Reverse {
        Direccion::Reverse
    } else {
        Direccion::Forward
    };

    // El receptor revalida el plan por su cuenta: nunca confía en que el emisor lo
    // valide por él (FR-021). Un plan inválido no llega a construir ejecución.
    let decision = plan
        .validate()
        .map_err(|_| MotivoRechazo::PlanInvalido)
        .and_then(|_| aceptar(&plan));

    if let Err(motivo) = decision {
        send_envelope(
            stream,
            &sobre(
                session_id,
                ProtocolMessageType::Response,
                ResponsePayload {
                    accepted: false,
                    reason: Some(motivo.como_str().to_string()),
                    interface_name: None,
                },
            ),
        )
        .await?;
        return Ok(DirectionResult::new_incomplete(direccion.como_str()));
    }

    send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::Response,
            ResponsePayload {
                accepted: true,
                reason: None,
                interface_name: None,
            },
        ),
    )
    .await?;

    let preparar: ProtocolEnvelope<PreparePayload> = recv_envelope(stream).await?;
    if preparar.msg_type != ProtocolMessageType::Prepare {
        return Err(protocolo_inesperado("PREPARE", preparar.msg_type));
    }

    // Confirma preparado con el puerto solicitado: en esta fase no se renegocia a un
    // puerto alternativo (V-01 lo deja para el lote avanzado).
    send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::Ready,
            ReadyPayload {
                direction: direccion.como_str().to_string(),
                is_ready: true,
                listening_port: plan.port,
            },
        ),
    )
    .await?;

    // El motor receptor debe estar escuchando ANTES de que el emisor intente conectar
    // (comprobado empíricamente en tests/engine_real.rs). Por eso se ejecuta en
    // paralelo con el resto del protocolo (heartbeat, START/STARTED), no después: si se
    // esperara a que termine el intercambio de mensajes para arrancarlo, el emisor
    // podría alcanzar su propio instante de inicio antes de que el receptor escuche.
    let intercambio_de_mensajes = async {
        let ping: ProtocolEnvelope<crate::model::protocol::HeartbeatPayload> =
            recv_envelope(stream).await?;
        responder_heartbeat(stream, &ping).await?;

        let inicio: ProtocolEnvelope<crate::model::protocol::StartPayload> =
            recv_envelope(stream).await?;
        if inicio.msg_type != ProtocolMessageType::Start {
            return Err(protocolo_inesperado("START", inicio.msg_type));
        }
        let recibido_en = std::time::Instant::now();

        send_envelope(
            stream,
            &sobre(
                session_id,
                ProtocolMessageType::Started,
                StartedPayload {
                    direction: direccion.como_str().to_string(),
                    local_t_ms: 0,
                },
            ),
        )
        .await?;

        // Aquí el margen ya cumplió su función (dar tiempo al motor, que arranca en
        // paralelo). Esta espera es solo para que el instante de "arranque lógico" de
        // este extremo sea comparable al del otro, no para retrasar el motor.
        tokio::time::sleep_until(momento_inicio_local(recibido_en, &inicio.payload).into()).await;
        IoResult::Ok(())
    };

    let (resultado_motor, resultado_mensajes) = tokio::join!(
        orquestador.ejecutar_mitad_local(&plan, NtttcpRole::Receiver, Some(interfaz_local)),
        intercambio_de_mensajes,
    );
    resultado_mensajes?;
    let local = resultado_motor.map_err(motor_error_a_io)?;

    let remoto: ProtocolEnvelope<EngineDonePayload> = recv_envelope(stream).await?;
    if remoto.msg_type != ProtocolMessageType::EngineDone {
        return Err(protocolo_inesperado("ENGINE_DONE", remoto.msg_type));
    }
    let emisor = engine_done_a_parsed(NtttcpRole::Sender, &remoto.payload);

    send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::EngineDone,
            parsed_a_engine_done(direccion.como_str(), "receiver", &local),
        ),
    )
    .await?;

    let medidas = MedidasDireccion {
        direccion,
        emisor,
        receptor: Some(local),
        muestras: SampleCollector::new(),
    };
    Ok(orquestador.resultado_de_direccion(&medidas))
}

/// Ensambla el `SessionResult` final a partir de las direcciones ya ejecutadas. Función
/// fina sobre `Orquestador::ensamblar`; existe para que quien orquesta la sesión
/// completa no tenga que conocer los campos internos de `EntradaSesion`.
#[allow(clippy::too_many_arguments)]
pub fn ensamblar_resultado(
    orquestador: &Orquestador,
    session_id: Uuid,
    started_at: &str,
    finished_at: &str,
    plan: &BenchmarkPlan,
    iniciador: Peer,
    receptor: Peer,
    direcciones: Vec<DirectionResult>,
) -> SessionResult {
    let a_snapshot = |p: Peer| PeerSnapshot {
        instance_id: p.instance_id,
        display_name: p.display_name,
        fingerprint: p.fingerprint,
        address: p.addresses.first().cloned().unwrap_or_default(),
    };

    orquestador.ensamblar(EntradaSesion {
        session_id,
        started_at,
        finished_at,
        plan,
        initiator: a_snapshot(iniciador),
        responder: a_snapshot(receptor),
        direcciones,
        capacidad: None,
        engine_version: crate::engine::ntttcp::engine_version().to_string(),
    })
}

fn protocolo_inesperado(esperado: &str, llegado: ProtocolMessageType) -> Error {
    Error::new(
        ErrorKind::InvalidData,
        format!("Se esperaba {esperado} y llegó {llegado:?}"),
    )
}

fn motor_error_a_io(e: MotorError) -> Error {
    Error::other(e.to_string())
}
