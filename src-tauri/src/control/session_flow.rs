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
use crate::control::preflight::{PreflightEvaluator, PreflightStatus};
use crate::control::protocol::{
    medir_rtt, momento_inicio_local, negociar_inicio, responder_heartbeat,
};
use crate::control::server::ahora_rfc3339;
use crate::control::transport::{recv_envelope, send_envelope};
use crate::engine::ntttcp::parser::{NtttcpParsedResult, NtttcpRole};
use crate::model::peer::Peer;
use crate::model::plan::BenchmarkPlan;
use crate::model::protocol::{
    CancelAckPayload, CancelPayload, EngineDonePayload, PreparePayload, ProtocolEnvelope,
    ProtocolMessageType, ReadyPayload, RequestPayload, ResponsePayload, SessionAckPayload,
    StartedPayload,
};
use crate::model::result::{DirectionResult, PeerSnapshot, SessionResult};
use crate::sampling::samples::SampleCollector;
use std::io::{Error, ErrorKind, Result as IoResult};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use uuid::Uuid;

/// Lo que el extremo que atiende una dirección conoce al terminarla.
///
/// Devuelve el `session_id` y el plan porque, en una prueba bidireccional, este mismo
/// extremo propone después la pata inversa: necesita el identificador de sesión que fijó
/// quien inició (FR-025) y los parámetros del plan, que la pata inversa reutiliza con la
/// dirección invertida.
pub struct DireccionAtendida {
    pub session_id: Uuid,
    pub plan: BenchmarkPlan,
    pub resultado: DirectionResult,
}

/// Motivo de rechazo de un `REQUEST`. Nunca se envía texto libre del solicitante al
/// peer: solo estas categorías cerradas (FR-021, el receptor construye su propia
/// explicación a partir de campos permitidos, no del payload ajeno).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotivoRechazo {
    PlanInvalido,
    RechazadoPorElUsuario,
    OcupadoConOtraSesion,
    /// El bloque de puertos de datos que el plan pide no está libre en este equipo
    /// (FR-020). Se comprueba antes de aceptar, no al arrancar el motor: así el
    /// iniciador se entera con un `RESPONSE`, no con un fallo de ejecución a mitad
    /// de diálogo.
    PuertosOcupados,
}

impl MotivoRechazo {
    fn como_str(&self) -> &'static str {
        match self {
            Self::PlanInvalido => "invalidPlan",
            Self::RechazadoPorElUsuario => "rejectedByUser",
            Self::OcupadoConOtraSesion => "busy",
            Self::PuertosOcupados => "portsUnavailable",
        }
    }
}

/// Señal compartida de cancelación de una sesión (T175, FR-007, FR-023). `true` una vez
/// que alguno de los dos extremos —este equipo o el par— decidió terminar. `SessionService`
/// crea el par emisor/receptor por sesión y guarda el emisor para que `cancel()` lo
/// active; el diálogo de la dirección en curso lo consulta en sus puntos de espera.
pub type SenalCancelacion = tokio::sync::watch::Sender<bool>;

/// Cuánto se espera, como máximo, la confirmación `CANCEL_ACK` del otro extremo antes de
/// darse por cancelado igualmente. Deliberadamente corto: el presupuesto real de
/// FR-023/SC-004 es la cancelación completa —incluida la liberación del motor— en ≤2 s, y
/// esto es solo un tramo de ese presupuesto, no el límite de 2 s que el contrato del
/// protocolo da a `CANCEL_ACK` en aislamiento (`contracts/peer-protocol.md`).
const ESPERA_CANCEL_ACK: Duration = Duration::from_millis(300);

async fn esperar_cancelacion(senal: &SenalCancelacion) {
    let mut rx = senal.subscribe();
    if *rx.borrow() {
        return;
    }
    while rx.changed().await.is_ok() {
        if *rx.borrow() {
            return;
        }
    }
    // El emisor se soltó sin cancelar (no ocurre en uso real: `SessionService` lo retiene
    // toda la sesión): no hay nada más que esperar aquí.
    std::future::pending::<()>().await;
}

/// Lo que llegó en el punto de espera de un mensaje concreto.
enum EsperaConCancelacion<T> {
    Llego(T),
    CanceladoPorElPar,
}

/// Lee el siguiente mensaje sin comprometerse de antemano a su forma: si es del tipo
/// esperado, deserializa su payload; si es un `CANCEL` fuera de turno, lo señala en vez de
/// tratarlo como un mensaje de protocolo inesperado (T175). Cualquier otro tipo, o un
/// payload que no encaja, sigue siendo un error de protocolo como antes.
async fn recibir_o_cancelacion<S, T>(
    stream: &mut S,
    tipo_esperado: ProtocolMessageType,
    nombre_esperado: &str,
) -> IoResult<EsperaConCancelacion<T>>
where
    S: AsyncRead + Unpin,
    T: serde::de::DeserializeOwned,
{
    let generico: ProtocolEnvelope<serde_json::Value> = recv_envelope(stream).await?;
    if generico.msg_type == ProtocolMessageType::Cancel {
        return Ok(EsperaConCancelacion::CanceladoPorElPar);
    }
    if generico.msg_type != tipo_esperado {
        return Err(protocolo_inesperado(nombre_esperado, generico.msg_type));
    }
    let payload: T = serde_json::from_value(generico.payload).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Payload de {nombre_esperado} con forma no válida: {e}"),
        )
    })?;
    Ok(EsperaConCancelacion::Llego(payload))
}

/// Este equipo decidió cancelar mientras esperaba al otro extremo, en un punto donde
/// todavía no hay nada en ejecución: avisa con `CANCEL` y da un margen corto a la
/// `CANCEL_ACK`, sin bloquear más de lo que permite el presupuesto conjunto de FR-023.
async fn cancelar_localmente<S>(stream: &mut S, session_id: Uuid) -> Error
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if let Err(e) = send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::Cancel,
            CancelPayload {
                reason: "cancelledByUser".to_string(),
                code: None,
            },
        ),
    )
    .await
    {
        return e;
    }
    // Mejor esfuerzo: si el `CANCEL_ACK` no llega a tiempo, se sigue igual. El resultado
    // ya es "cancelada" independientemente de si el otro extremo confirmó a tiempo.
    let _ = tokio::time::timeout(
        ESPERA_CANCEL_ACK,
        recv_envelope::<_, CancelAckPayload>(stream),
    )
    .await;
    Error::other("Sesión cancelada por este equipo")
}

/// El otro extremo canceló mientras este esperaba su turno: confirma con `CANCEL_ACK` y
/// marca la señal compartida, para que quien orquesta la sesión sepa que el desenlace es
/// una cancelación —del par, no propia— y no un fallo (T175).
async fn responder_cancelacion_remota<S>(
    stream: &mut S,
    session_id: Uuid,
    senal: &SenalCancelacion,
) -> Error
where
    S: AsyncWrite + Unpin,
{
    let _ = senal.send(true);
    if let Err(e) = send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::CancelAck,
            CancelAckPayload {},
        ),
    )
    .await
    {
        return e;
    }
    Error::other("Sesión cancelada por el otro equipo")
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
        packets_sent: p.packets_sent,
        packets_received: p.packets_received,
        packets_retransmitted: p.packets_retransmitted,
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
        packets_sent: r.packets_sent,
        packets_received: r.packets_received,
        packets_retransmitted: r.packets_retransmitted,
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
    cancelacion: &SenalCancelacion,
) -> IoResult<DirectionResult>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // 0. Preflight: hay ruta local hacia el objetivo antes de proponer nada (FR-019).
    // Un `REQUEST` que no puede llegar a ejecutarse en este extremo no debe hacer perder
    // el turno al otro lado del diálogo.
    let objetivo = target_host.parse().map_err(|e| {
        Error::other(format!(
            "Dirección de destino '{target_host}' no válida: {e}"
        ))
    })?;
    let nic = PreflightEvaluator::check_nic(objetivo);
    if nic.status == PreflightStatus::Failed {
        return Err(Error::other(format!(
            "Sin ruta local hacia {target_host}: {}",
            nic.error
                .and_then(|e| e.diagnostic_id)
                .unwrap_or_else(|| "sin más detalle".to_string())
        )));
    }

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

    let respuesta = tokio::select! {
        _ = esperar_cancelacion(cancelacion) => return Err(cancelar_localmente(stream, session_id).await),
        r = recibir_o_cancelacion::<_, ResponsePayload>(stream, ProtocolMessageType::Response, "RESPONSE") => r?,
    };
    let respuesta = match respuesta {
        EsperaConCancelacion::Llego(p) => p,
        EsperaConCancelacion::CanceladoPorElPar => {
            return Err(responder_cancelacion_remota(stream, session_id, cancelacion).await);
        }
    };
    if !respuesta.accepted {
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

    let listo = tokio::select! {
        _ = esperar_cancelacion(cancelacion) => return Err(cancelar_localmente(stream, session_id).await),
        r = recibir_o_cancelacion::<_, ReadyPayload>(stream, ProtocolMessageType::Ready, "READY") => r?,
    };
    let listo = match listo {
        EsperaConCancelacion::Llego(p) => p,
        EsperaConCancelacion::CanceladoPorElPar => {
            return Err(responder_cancelacion_remota(stream, session_id, cancelacion).await);
        }
    };
    if !listo.is_ready {
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
        protocolo: plan.protocol,
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
///
/// Devuelve también el `session_id` que fijó quien inicia en su `REQUEST`: en una prueba
/// bidireccional este mismo extremo lo reutiliza al proponer la pata inversa, sin lo
/// cual las dos patas no compartirían identificador de sesión (FR-025).
pub async fn atender_direccion<S>(
    stream: &mut S,
    orquestador: &Orquestador,
    interfaz_local: &str,
    aceptar: impl FnOnce(&BenchmarkPlan) -> Result<(), MotivoRechazo>,
    cancelacion: &SenalCancelacion,
) -> IoResult<DireccionAtendida>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let solicitud: ProtocolEnvelope<RequestPayload> = recv_envelope(stream).await?;
    atender_direccion_desde(
        stream,
        orquestador,
        interfaz_local,
        solicitud,
        aceptar,
        cancelacion,
    )
    .await
}

/// Igual que `atender_direccion`, pero con el `REQUEST` ya leído.
///
/// Existe para el despachador del servidor: al aceptar una conexión no sabe todavía si
/// el primer mensaje es un `PAIR_REQUEST` o un `REQUEST`, así que lo lee una vez, mira su
/// tipo y entrega el sobre ya recibido al manejador que corresponda. Releer desde el
/// canal no es posible: el mensaje ya se consumió.
pub async fn atender_direccion_desde<S>(
    stream: &mut S,
    orquestador: &Orquestador,
    interfaz_local: &str,
    solicitud: ProtocolEnvelope<RequestPayload>,
    aceptar: impl FnOnce(&BenchmarkPlan) -> Result<(), MotivoRechazo>,
    cancelacion: &SenalCancelacion,
) -> IoResult<DireccionAtendida>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
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
    //
    // El puerto se comprueba aquí, no dentro de `aceptar`: es una condición técnica del
    // equipo (FR-020), no una decisión de negocio como la confianza o el consentimiento
    // (FR-016), y de este lado siempre se va a ejecutar como receptor en esta pata.
    let decision = plan
        .validate()
        .map_err(|_| MotivoRechazo::PlanInvalido)
        .and_then(|_| verificar_puerto_de_datos_libre(&plan))
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
        return Ok(DireccionAtendida {
            session_id,
            plan,
            resultado: DirectionResult::new_incomplete(direccion.como_str()),
        });
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

    let preparar = tokio::select! {
        _ = esperar_cancelacion(cancelacion) => return Err(cancelar_localmente(stream, session_id).await),
        r = recibir_o_cancelacion::<_, PreparePayload>(stream, ProtocolMessageType::Prepare, "PREPARE") => r?,
    };
    if matches!(preparar, EsperaConCancelacion::CanceladoPorElPar) {
        return Err(responder_cancelacion_remota(stream, session_id, cancelacion).await);
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
        protocolo: plan.protocol,
        emisor,
        receptor: Some(local),
        muestras: SampleCollector::new(),
    };
    Ok(DireccionAtendida {
        session_id,
        plan,
        resultado: orquestador.resultado_de_direccion(&medidas),
    })
}

/// Resultado de una prueba estándar bidireccional vista desde uno de sus extremos.
pub struct PruebaBidireccional {
    pub session_id: Uuid,
    pub plan: BenchmarkPlan,
    /// `[ida, vuelta]`, en el orden en que se ejecutaron.
    pub direcciones: Vec<DirectionResult>,
}

fn aceptar_si_es_valido(plan: &BenchmarkPlan) -> Result<(), MotivoRechazo> {
    plan.validate().map_err(|_| MotivoRechazo::PlanInvalido)
}

/// FR-020: antes de aceptar, este equipo comprueba que puede abrir el bloque de puertos
/// de datos que el plan pide para escuchar como receptor en esta pata. `control_port` va
/// en `None`: quien pregunta ya está escuchando en el suyo por definición (es el canal
/// por el que llegó este mismo `REQUEST`), así que comprobarlo no diría nada nuevo.
fn verificar_puerto_de_datos_libre(plan: &BenchmarkPlan) -> Result<(), MotivoRechazo> {
    let check = PreflightEvaluator::check_ports(None, plan.port, plan.streams);
    if check.status == PreflightStatus::Failed {
        Err(MotivoRechazo::PuertosOcupados)
    } else {
        Ok(())
    }
}

/// Frontera de una pata para quien orquesta la sesión.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventoPata {
    Inicia(Direccion),
    Termina(Direccion),
}

/// Prueba estándar en ambos sentidos vista desde quien **inicia** la sesión (FR-018).
///
/// Las dos patas van una tras otra sobre el mismo canal y con turnos alternos: en la
/// ida este extremo propone y emite; en la vuelta propone el otro y este recibe. El
/// reparto de roles no se negocia: es la convención de la prueba estándar, y por eso los
/// dos extremos deben ejecutar sus conductores en el mismo orden.
///
/// `en_pata` se invoca justo antes de cada pata (`EventoPata::Inicia`) y justo después
/// (`EventoPata::Termina`), con su sentido. Sirve para que quien orquesta refleje el
/// avance en la máquina de estados (`RunningSend`, luego `RunningReceive`) y arranque o
/// pare el muestreo en vivo, sin que este módulo conozca `SessionService`. Es síncrono a
/// propósito: un closure asíncrono impide demostrar que el futuro es `Send`, y sin eso
/// no se puede lanzar la sesión con `tokio::spawn`. Quien lo use envía por un canal.
#[allow(clippy::too_many_arguments)]
pub async fn ejecutar_prueba_estandar_como_iniciador<S>(
    stream: &mut S,
    orquestador: &Orquestador,
    session_id: Uuid,
    plan: &BenchmarkPlan,
    host_del_receptor: &str,
    interfaz_local: &str,
    mut en_pata: impl FnMut(EventoPata),
    cancelacion: &SenalCancelacion,
) -> IoResult<PruebaBidireccional>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut plan_ida = plan.clone();
    plan_ida.direction = crate::model::plan::BenchmarkDirection::Forward;

    en_pata(EventoPata::Inicia(Direccion::Forward));
    let ida = iniciar_direccion(
        stream,
        orquestador,
        session_id,
        &plan_ida,
        Direccion::Forward,
        host_del_receptor,
        cancelacion,
    )
    .await;
    en_pata(EventoPata::Termina(Direccion::Forward));
    let ida = ida?;

    // Vuelta: ahora es el otro extremo quien propone; este atiende y recibe.
    en_pata(EventoPata::Inicia(Direccion::Reverse));
    let vuelta = atender_direccion(
        stream,
        orquestador,
        interfaz_local,
        aceptar_si_es_valido,
        cancelacion,
    )
    .await;
    en_pata(EventoPata::Termina(Direccion::Reverse));
    let vuelta = vuelta?.resultado;

    Ok(PruebaBidireccional {
        session_id,
        plan: plan_ida,
        direcciones: vec![ida, vuelta],
    })
}

/// Prueba estándar en ambos sentidos vista desde quien **responde**.
///
/// `primera_solicitud` es el `REQUEST` de la ida, ya leído por el despachador que decidió
/// que este canal trae una prueba y no un emparejamiento. La vuelta la propone este
/// extremo reutilizando el plan recibido con la dirección invertida y el mismo
/// `session_id` que fijó el iniciador (FR-025).
#[allow(clippy::too_many_arguments)]
pub async fn ejecutar_prueba_estandar_como_respondedor<S>(
    stream: &mut S,
    orquestador: &Orquestador,
    interfaz_local: &str,
    host_del_iniciador: &str,
    primera_solicitud: ProtocolEnvelope<RequestPayload>,
    aceptar: impl FnOnce(&BenchmarkPlan) -> Result<(), MotivoRechazo>,
    mut en_pata: impl FnMut(EventoPata),
    cancelacion: &SenalCancelacion,
) -> IoResult<PruebaBidireccional>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    en_pata(EventoPata::Inicia(Direccion::Forward));
    let atendida = atender_direccion_desde(
        stream,
        orquestador,
        interfaz_local,
        primera_solicitud,
        aceptar,
        cancelacion,
    )
    .await;
    en_pata(EventoPata::Termina(Direccion::Forward));
    let atendida = atendida?;

    // Si rechazó la ida, no hay prueba: no se propone una vuelta que nadie aceptó.
    if atendida.resultado.status == "incomplete" && atendida.resultado.receiver.is_none() {
        return Ok(PruebaBidireccional {
            session_id: atendida.session_id,
            plan: atendida.plan,
            direcciones: vec![atendida.resultado],
        });
    }

    let mut plan_vuelta = atendida.plan.clone();
    plan_vuelta.direction = crate::model::plan::BenchmarkDirection::Reverse;

    en_pata(EventoPata::Inicia(Direccion::Reverse));
    let vuelta = iniciar_direccion(
        stream,
        orquestador,
        atendida.session_id,
        &plan_vuelta,
        Direccion::Reverse,
        host_del_iniciador,
        cancelacion,
    )
    .await;
    en_pata(EventoPata::Termina(Direccion::Reverse));
    let vuelta = vuelta?;

    Ok(PruebaBidireccional {
        session_id: atendida.session_id,
        plan: atendida.plan,
        direcciones: vec![atendida.resultado, vuelta],
    })
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

/// Cuánto se espera un `SESSION_RESULT`/`SESSION_ACK` antes de darse por vencido y usar
/// la vista local (`contracts/peer-protocol.md`: sin timeout propio, aplica el genérico
/// de 10 s de «Generic request/response»).
pub const ESPERA_RECONCILIACION: Duration = Duration::from_secs(10);

/// Quien inicia la sesión construye el resultado canónico (contrato `session-result.md`,
/// punto 1): lo envía y da una oportunidad a la confirmación del otro extremo. Este
/// equipo ya completó y va a persistir su propia copia sea cual sea el desenlace del
/// envío: un `SESSION_ACK` ausente o tardío no deshace nada local, solo dice que el otro
/// extremo puede haberse quedado con su propia vista en vez de la canónica (T176).
pub async fn enviar_resultado_y_esperar_ack<S>(
    stream: &mut S,
    session_id: Uuid,
    resultado: &SessionResult,
    plazo: Duration,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if let Err(e) = send_envelope(
        stream,
        &sobre(
            session_id,
            ProtocolMessageType::SessionResult,
            resultado.clone(),
        ),
    )
    .await
    {
        tracing::warn!("No se pudo enviar SESSION_RESULT ({session_id}): {e}");
        return;
    }

    match tokio::time::timeout(plazo, recv_envelope::<_, SessionAckPayload>(stream)).await {
        Ok(Ok(ack)) if ack.msg_type == ProtocolMessageType::SessionAck => {
            if !ack.payload.persisted {
                tracing::warn!(
                    "El otro equipo no reconcilió el resultado de la sesión {session_id}: se quedó con su propia vista local"
                );
            }
        }
        Ok(Ok(otro)) => {
            tracing::warn!(
                "Se esperaba SESSION_ACK para {session_id} y llegó {:?}",
                otro.msg_type
            );
        }
        Ok(Err(e)) => tracing::warn!("Fallo al leer SESSION_ACK para {session_id}: {e}"),
        Err(_) => tracing::warn!(
            "Sin SESSION_ACK para {session_id} en {plazo:?}: el otro equipo puede quedar con su propia vista local"
        ),
    }
}

/// Lo que el extremo que responde terminó usando: la copia canónica que le llegó y
/// validó, o la suya propia porque no llegó ninguna o no encajaba.
pub enum ResultadoReconciliado {
    Recibido(Box<SessionResult>),
    Local,
}

/// Quien atiende la sesión valida el `SESSION_RESULT` antes del ACK (contrato, punto 2):
/// vínculo de sesión y coherencia con lo que este equipo mismo vio. Si no llega a tiempo
/// o no coincide, se queda con `local` —su propia vista, ya ensamblada— marcada como
/// degradada (`resultSource: "local"`), sin sobrescribir evidencia distinta en silencio
/// (contrato, punto 5).
pub async fn recibir_resultado_o_conservar_local<S>(
    stream: &mut S,
    session_id: Uuid,
    local: &SessionResult,
    plazo: Duration,
) -> ResultadoReconciliado
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let recibido = tokio::time::timeout(plazo, recv_envelope::<_, SessionResult>(stream)).await;

    let validado = match recibido {
        Ok(Ok(sobre))
            if sobre.msg_type == ProtocolMessageType::SessionResult
                && coherente_con(local, &sobre.payload) =>
        {
            Some(sobre.payload)
        }
        _ => None,
    };

    match validado {
        Some(mut canonico) => {
            // Este equipo no es la fuente canónica: por definición, lo que llega y valida
            // aquí viene de quien inició (FR-025).
            canonico.result_source = "initiator".to_string();
            let _ = send_envelope(
                stream,
                &sobre(
                    session_id,
                    ProtocolMessageType::SessionAck,
                    SessionAckPayload {
                        session_id,
                        persisted: true,
                        thresholds_hash: canonico.versions.thresholds_hash.clone(),
                    },
                ),
            )
            .await;
            ResultadoReconciliado::Recibido(Box::new(canonico))
        }
        None => {
            let _ = send_envelope(
                stream,
                &sobre(
                    session_id,
                    ProtocolMessageType::SessionAck,
                    SessionAckPayload {
                        session_id,
                        persisted: false,
                        thresholds_hash: local.versions.thresholds_hash.clone(),
                    },
                ),
            )
            .await;
            ResultadoReconciliado::Local
        }
    }
}

/// Coherencia mínima y real, no un sello de goma: el mismo identificador de sesión, el
/// mismo número de direcciones, y ninguna dirección que este equipo vio completa con
/// velocidad oficial aparece como algo menos en la copia canónica.
fn coherente_con(local: &SessionResult, recibido: &SessionResult) -> bool {
    if recibido.session_id != local.session_id {
        return false;
    }
    if recibido.directions.len() != local.directions.len() {
        return false;
    }
    local
        .directions
        .iter()
        .filter(|d| d.status == "completed")
        .all(|propia| {
            recibido.directions.iter().any(|r| {
                r.direction == propia.direction
                    && r.status == "completed"
                    && r.official_bps.is_some()
            })
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

#[cfg(test)]
mod preflight_en_el_dialogo {
    use super::*;
    use crate::control::engine_port::MotorDeLaboratorio;
    use tokio::io::{AsyncReadExt, duplex};

    fn orquestador_de_laboratorio() -> Orquestador {
        Orquestador::new(std::sync::Arc::new(MotorDeLaboratorio::con_respuestas(
            vec![],
        )))
    }

    /// T173 (FR-019): un destino que no se puede alcanzar aborta antes de proponer nada.
    /// No hay forma sencilla y portable de fabricar una IP real sin ruta, así que esta
    /// prueba cubre la puerta de guarda con una dirección de destino mal formada, que
    /// atraviesa la misma comprobación antes de `send_envelope`; que `check_nic` en sí
    /// detecte una IP real sin ruta lo prueba `preflight_tests.rs`.
    #[tokio::test]
    async fn un_destino_no_valido_no_llega_a_enviar_request() {
        let (mut yo, mut otro) = duplex(8192);
        let orquestador = orquestador_de_laboratorio();
        let plan = BenchmarkPlan::new_standard_tcp(5900);

        let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
        let error = iniciar_direccion(
            &mut yo,
            &orquestador,
            Uuid::new_v4(),
            &plan,
            Direccion::Forward,
            "esto-no-es-una-ip",
            &tx_cancel,
        )
        .await
        .expect_err("un destino no válido debe fallar antes de dialogar");
        assert!(error.to_string().contains("no válida"));

        // Nada llegó al otro lado: se cerró sin escribir ni un byte.
        drop(yo);
        let mut buf = [0u8; 1];
        let leido = otro.read(&mut buf).await.expect("leer del otro extremo");
        assert_eq!(leido, 0, "no debió enviarse ningún REQUEST");
    }

    /// T173 (FR-020): el bloque de puertos de datos que el plan pide ya está en uso en
    /// este equipo, así que se rechaza con un motivo explícito antes de tocar el motor.
    #[tokio::test]
    async fn un_puerto_de_datos_ocupado_se_rechaza_con_ports_unavailable() {
        // `check_ports` sondea siempre `0.0.0.0` (`ports.rs::is_port_available_for_protocol`):
        // hay que ocuparlo igual, o el sondeo vería el puerto libre y la prueba se
        // quedaría esperando un PREPARE que nunca llegaría.
        let ocupado = std::net::TcpListener::bind("0.0.0.0:0").unwrap();
        let puerto = ocupado.local_addr().unwrap().port();

        let (mut lado_iniciador, mut lado_receptor) = duplex(8192);
        let session_id = Uuid::new_v4();
        let plan = BenchmarkPlan::new_standard_tcp(puerto);

        send_envelope(
            &mut lado_iniciador,
            &sobre(
                session_id,
                ProtocolMessageType::Request,
                RequestPayload {
                    plan: plan.clone(),
                    estimate_seconds: 1,
                    suggested_interface: None,
                },
            ),
        )
        .await
        .unwrap();

        let orquestador = orquestador_de_laboratorio();
        let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
        let atendida = atender_direccion(
            &mut lado_receptor,
            &orquestador,
            "127.0.0.1",
            |_| Ok(()),
            &tx_cancel,
        )
        .await
        .expect("responde con el rechazo, no con un error de protocolo");

        assert_eq!(atendida.resultado.status, "incomplete");
        assert!(atendida.resultado.receiver.is_none());

        let respuesta: ProtocolEnvelope<ResponsePayload> = recv_envelope(&mut lado_iniciador)
            .await
            .expect("el iniciador debe recibir un RESPONSE, no quedarse sin nada");
        assert!(!respuesta.payload.accepted);
        assert_eq!(
            respuesta.payload.reason.as_deref(),
            Some("portsUnavailable")
        );

        drop(ocupado);
    }

    fn puerto_libre_ahora_mismo() -> u16 {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    }

    /// T175 (FR-007, FR-023): cancelar mientras se espera el `RESPONSE` —nada del motor
    /// en marcha todavía— avisa al par con `CANCEL` en vez de simplemente colgar la
    /// llamada, y termina sin quedarse esperando indefinidamente el `CANCEL_ACK`.
    #[tokio::test]
    async fn cancelar_mientras_se_espera_la_respuesta_envia_cancel_y_no_se_cuelga() {
        let (yo, mut otro) = duplex(8192);
        let plan = BenchmarkPlan::new_standard_tcp(puerto_libre_ahora_mismo());
        let (tx_cancel, _rx) = tokio::sync::watch::channel(false);
        let session_id = Uuid::new_v4();
        let tx_de_la_tarea = tx_cancel.clone();

        let tarea = tokio::spawn(async move {
            let mut yo = yo;
            let orquestador = orquestador_de_laboratorio();
            iniciar_direccion(
                &mut yo,
                &orquestador,
                session_id,
                &plan,
                Direccion::Forward,
                "127.0.0.1",
                &tx_de_la_tarea,
            )
            .await
        });

        let _peticion: ProtocolEnvelope<RequestPayload> = recv_envelope(&mut otro).await.unwrap();
        tx_cancel.send(true).unwrap();

        let cancelacion: ProtocolEnvelope<CancelPayload> = recv_envelope(&mut otro)
            .await
            .expect("debe llegar un CANCEL en vez de quedarse esperando el RESPONSE");
        assert_eq!(cancelacion.msg_type, ProtocolMessageType::Cancel);

        send_envelope(
            &mut otro,
            &sobre(
                session_id,
                ProtocolMessageType::CancelAck,
                CancelAckPayload {},
            ),
        )
        .await
        .unwrap();

        let resultado = tokio::time::timeout(Duration::from_secs(2), tarea)
            .await
            .expect("no debe colgarse esperando el CANCEL_ACK")
            .unwrap();
        assert!(
            resultado.is_err(),
            "una cancelación local no es un resultado de la pata, es su ausencia"
        );
    }

    /// T175 (FR-007, FR-023): un `CANCEL` del par mientras este equipo espera el
    /// `PREPARE` —ya aceptó la pata, el motor aún no se lanzó— se confirma con
    /// `CANCEL_ACK` y marca la señal compartida, para que quien orquesta la sesión sepa
    /// que el desenlace es una cancelación del otro extremo, no un fallo propio.
    #[tokio::test]
    async fn un_cancel_del_par_mientras_se_espera_prepare_se_confirma_y_marca_la_senal() {
        let (mut lado_iniciador, lado_receptor) = duplex(8192);
        let session_id = Uuid::new_v4();
        let plan = BenchmarkPlan::new_standard_tcp(puerto_libre_ahora_mismo());

        send_envelope(
            &mut lado_iniciador,
            &sobre(
                session_id,
                ProtocolMessageType::Request,
                RequestPayload {
                    plan: plan.clone(),
                    estimate_seconds: 1,
                    suggested_interface: None,
                },
            ),
        )
        .await
        .unwrap();

        let (tx_cancel, _rx) = tokio::sync::watch::channel(false);
        let tx_para_comprobar = tx_cancel.clone();

        let tarea = tokio::spawn(async move {
            let mut lado_receptor = lado_receptor;
            let orquestador = orquestador_de_laboratorio();
            atender_direccion(
                &mut lado_receptor,
                &orquestador,
                "127.0.0.1",
                |_| Ok(()),
                &tx_cancel,
            )
            .await
        });

        // Recibe el RESPONSE(accepted) y, en vez de un PREPARE, cancela.
        let _respuesta: ProtocolEnvelope<ResponsePayload> =
            recv_envelope(&mut lado_iniciador).await.unwrap();
        send_envelope(
            &mut lado_iniciador,
            &sobre(
                session_id,
                ProtocolMessageType::Cancel,
                CancelPayload {
                    reason: "cancelledByUser".to_string(),
                    code: None,
                },
            ),
        )
        .await
        .unwrap();

        let ack: ProtocolEnvelope<CancelAckPayload> = recv_envelope(&mut lado_iniciador)
            .await
            .expect("debe confirmar con CANCEL_ACK");
        assert_eq!(ack.msg_type, ProtocolMessageType::CancelAck);

        let resultado = tokio::time::timeout(Duration::from_secs(2), tarea)
            .await
            .expect("no debe colgarse")
            .unwrap();
        assert!(resultado.is_err());
        assert!(
            *tx_para_comprobar.borrow(),
            "la señal debe quedar marcada: fue una cancelación del par, no un fallo"
        );
    }
}

#[cfg(test)]
mod reconciliacion_de_resultado {
    use super::*;
    use crate::model::peer::{Peer, TrustState};
    use crate::model::result::EngineResult;
    use tokio::io::duplex;

    fn peer_de_prueba(nombre: &str) -> Peer {
        Peer {
            instance_id: Uuid::new_v4(),
            display_name: nombre.to_string(),
            fingerprint: "0".repeat(64),
            addresses: vec!["127.0.0.1:0".to_string()],
            trust_state: TrustState::Trusted,
            auto_accept: false,
            last_seen: ahora_rfc3339(),
            alias: None,
        }
    }

    fn direccion_completa(sentido: &str, bps: &str) -> DirectionResult {
        let motor = |bps: &str| EngineResult {
            role: "sender".to_string(),
            total_bytes: "1000".to_string(),
            realtime_seconds: 1.0,
            throughput_bps: bps.to_string(),
            cpu_percent: None,
            buffers_count: None,
            errors_count: 0,
            packets_sent: None,
            packets_received: None,
            packets_retransmitted: None,
            raw: None,
        };
        DirectionResult::new_completed(sentido, motor(bps), motor(bps), 4, 0)
    }

    fn resultado_de_prueba(session_id: Uuid, direcciones: Vec<DirectionResult>) -> SessionResult {
        ensamblar_resultado(
            &Orquestador::new(std::sync::Arc::new(
                crate::control::engine_port::MotorDeLaboratorio::con_respuestas(vec![]),
            )),
            session_id,
            "2026-09-25T00:00:00.000Z",
            "2026-09-25T00:00:10.000Z",
            &BenchmarkPlan::new_standard_tcp(5950),
            peer_de_prueba("Iniciador"),
            peer_de_prueba("Respondedor"),
            direcciones,
        )
    }

    /// T176 (FR-025): el iniciador envía su resultado canónico y el respondedor, al
    /// validarlo, lo adopta como propio —marcado `initiator`— en vez de conservar el suyo.
    #[tokio::test]
    async fn el_respondedor_adopta_el_resultado_canonico_cuando_coincide() {
        let session_id = Uuid::new_v4();
        let canonico =
            resultado_de_prueba(session_id, vec![direccion_completa("forward", "900000000")]);
        let local =
            resultado_de_prueba(session_id, vec![direccion_completa("forward", "900000000")]);

        let (mut lado_iniciador, mut lado_respondedor) = duplex(8192);

        let tarea_iniciador = tokio::spawn(async move {
            enviar_resultado_y_esperar_ack(
                &mut lado_iniciador,
                session_id,
                &canonico,
                Duration::from_secs(2),
            )
            .await;
        });

        let reconciliado = recibir_resultado_o_conservar_local(
            &mut lado_respondedor,
            session_id,
            &local,
            Duration::from_secs(2),
        )
        .await;

        tarea_iniciador.await.unwrap();

        match reconciliado {
            ResultadoReconciliado::Recibido(r) => assert_eq!(r.result_source, "initiator"),
            ResultadoReconciliado::Local => panic!("debía adoptar la copia canónica"),
        }
    }

    /// T176 (contrato, punto 5): sin ningún `SESSION_RESULT` que leer, el respondedor
    /// conserva su propia vista, sin bloquear más allá del margen de reconciliación.
    #[tokio::test]
    async fn sin_session_result_el_respondedor_conserva_su_vista_local() {
        let session_id = Uuid::new_v4();
        let local =
            resultado_de_prueba(session_id, vec![direccion_completa("forward", "900000000")]);
        let (_lado_iniciador, mut lado_respondedor) = duplex(8192);

        // El iniciador nunca escribe nada: el respondedor debe darse por vencido solo, sin
        // que nadie tenga que cerrarle el canal a mano. Plazo corto —no el de producción,
        // 10 s— para que la prueba no tarde de verdad ese margen.
        let reconciliado = recibir_resultado_o_conservar_local(
            &mut lado_respondedor,
            session_id,
            &local,
            Duration::from_millis(80),
        )
        .await;

        assert!(matches!(reconciliado, ResultadoReconciliado::Local));
    }

    /// T176 (contrato, punto 2): un `SESSION_RESULT` que dice menos de lo que este equipo
    /// vio con sus propios ojos —una dirección que aquí se completó, y ahí no—se rechaza
    /// como incoherente y se conserva la vista local en vez de creerlo a ciegas.
    #[tokio::test]
    async fn un_resultado_incoherente_con_lo_visto_localmente_se_descarta() {
        let session_id = Uuid::new_v4();
        let local =
            resultado_de_prueba(session_id, vec![direccion_completa("forward", "900000000")]);
        let incoherente =
            resultado_de_prueba(session_id, vec![DirectionResult::new_incomplete("forward")]);

        let (mut lado_iniciador, mut lado_respondedor) = duplex(8192);

        let tarea_iniciador = tokio::spawn(async move {
            enviar_resultado_y_esperar_ack(
                &mut lado_iniciador,
                session_id,
                &incoherente,
                Duration::from_secs(2),
            )
            .await;
        });

        let reconciliado = recibir_resultado_o_conservar_local(
            &mut lado_respondedor,
            session_id,
            &local,
            Duration::from_secs(2),
        )
        .await;

        tarea_iniciador.await.unwrap();
        assert!(matches!(reconciliado, ResultadoReconciliado::Local));
    }
}
