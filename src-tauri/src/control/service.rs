use super::domain::{SessionState, SessionStateMachine, StateMachineError};
use super::orquestador::{Direccion, Orquestador};
use super::server::ahora_rfc3339;
use super::session_flow::{
    EventoPata, MotivoRechazo, ResultadoReconciliado, SenalCancelacion, atender_direccion_desde,
    ejecutar_prueba_estandar_como_iniciador, ejecutar_prueba_estandar_como_respondedor,
    ensamblar_resultado, enviar_resultado_y_esperar_ack, recibir_resultado_o_conservar_local,
};
use crate::discovery::conectar_y_saludar;
use crate::history::database::Database;
use crate::history::get_peer_by_fingerprint;
use crate::history::sessions::guardar_resultado_de_sesion;
use crate::identity::InstanceIdentity;
use crate::model::peer::{Peer, TrustState};
use crate::model::plan::BenchmarkPlan;
use crate::model::protocol::{ProtocolEnvelope, RequestPayload};
use crate::sampling::SAMPLE_INTERVAL_MS;
use crate::sampling::vivo::{MuestreadorActivo, Muestreo, Sentido};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_rustls::server::TlsStream;
use uuid::Uuid;

/// Lo que una sesión real necesita del resto de la aplicación.
///
/// Se pasa como argumento y no vive dentro de `SessionService` para que el servicio
/// siga siendo, ante todo, el dueño de la única sesión activa: quien lo construye en las
/// pruebas de estados no tiene que fabricar un motor, una identidad y una base de datos.
#[derive(Clone)]
pub struct ContextoSesion {
    pub orquestador: Arc<Orquestador>,
    pub identity: Arc<InstanceIdentity>,
    pub database: Arc<Database>,
    /// Muestreo en vivo hacia la interfaz (T174). Con `None` la sesión funciona igual y
    /// solo se queda sin gráfica en directo: útil en pruebas y si la plataforma no ofrece
    /// contadores.
    pub muestreo: Option<Muestreo>,
    /// Solicitudes de sesión entrante esperando consentimiento humano (T177, FR-016).
    pub consentimiento: Arc<crate::control::consent::SolicitudesEntrantes>,
    /// Por dónde avisar a la interfaz de una solicitud entrante. Independiente de
    /// `muestreo.emisor`: puede haber consentimiento sin muestreo en vivo configurado.
    /// `None` en las pruebas que no necesitan verificar el aviso.
    pub emisor_eventos: Option<Arc<dyn crate::ipc::events::EmisorDeEventos>>,
    /// Emparejamientos entrantes esperando decisión humana (T182, FR-012).
    pub emparejamientos: Arc<crate::control::consent::EmparejamientosEntrantes>,
}

/// Convierte las fronteras de pata en transiciones de estado y en muestreo en vivo.
///
/// Las patas llegan por un canal desde un closure síncrono (ver `session_flow`). Cada
/// pata abre un muestreador propio sobre la interfaz que lleva el tráfico hacia `remoto`
/// y lo cierra al terminar. Si la sesión se cancela, el emisor del canal se suelta con el
/// futuro de la sesión, el canal se cierra y el muestreo pendiente se detiene aquí.
///
/// `emite_en_la_ida` es verdadero para quien inicia: en la ida emite y en la vuelta recibe.
fn lanzar_aplicador(
    maquina: Arc<Mutex<SessionStateMachine>>,
    session_id: Uuid,
    muestreo: Option<Muestreo>,
    remoto: IpAddr,
    emite_en_la_ida: bool,
) -> (
    tokio::sync::mpsc::UnboundedSender<EventoPata>,
    JoinHandle<()>,
) {
    let (avisos, mut recibidos) = tokio::sync::mpsc::unbounded_channel::<EventoPata>();
    let tarea = tokio::spawn(async move {
        let mut activo: Option<MuestreadorActivo> = None;
        while let Some(evento) = recibidos.recv().await {
            match evento {
                EventoPata::Inicia(sentido) => {
                    let emite = (sentido == Direccion::Forward) == emite_en_la_ida;
                    let destino = if emite {
                        SessionState::RunningSend
                    } else {
                        SessionState::RunningReceive
                    };
                    let _ = maquina.lock().await.transition_to(destino, session_id);

                    if let Some(m) = &muestreo {
                        match m.contadores.abrir(remoto) {
                            Ok(fuente) => {
                                activo = Some(MuestreadorActivo::iniciar(
                                    fuente,
                                    if emite {
                                        Sentido::Emision
                                    } else {
                                        Sentido::Recepcion
                                    },
                                    session_id.to_string(),
                                    sentido.como_str(),
                                    Arc::clone(&m.emisor),
                                    Duration::from_millis(SAMPLE_INTERVAL_MS),
                                ));
                            }
                            // Sin contadores no hay gráfica en directo, pero la medición
                            // sigue: el resultado no depende de ellos.
                            Err(e) => tracing::warn!("Sin muestreo en vivo: {e}"),
                        }
                    }
                }
                EventoPata::Termina(_) => {
                    if let Some(a) = activo.take() {
                        a.detener().await;
                    }
                }
            }
        }
        if let Some(a) = activo.take() {
            a.detener().await;
        }
    });
    (avisos, tarea)
}

pub struct SessionService {
    state_machine: Arc<Mutex<SessionStateMachine>>,
    active_peer: Arc<Mutex<Option<Peer>>>,
    active_plan: Arc<Mutex<Option<BenchmarkPlan>>>,
    /// Tarea que ejecuta el diálogo de la sesión en curso. Cancelar la aborta si no
    /// termina por sí sola a tiempo: al soltarse el futuro se cierran el canal TLS y el
    /// proceso del motor (`Drop` de `NtttcpProcess` y `JobObject`), que es la limpieza
    /// que exige FR-023.
    tarea: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Señal para pedirle a la tarea en curso que cancele con cortesía —avisando al par
    /// con `CANCEL` y dándole una oportunidad de responder con `CANCEL_ACK`— antes de que
    /// `cancel()` la aborte sin más si no termina a tiempo (T175, FR-007, FR-023).
    cancelacion: Arc<Mutex<Option<SenalCancelacion>>>,
}

impl Default for SessionService {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            state_machine: Arc::new(Mutex::new(SessionStateMachine::new())),
            active_peer: Arc::new(Mutex::new(None)),
            active_plan: Arc::new(Mutex::new(None)),
            tarea: Arc::new(Mutex::new(None)),
            cancelacion: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn current_state(&self) -> SessionState {
        let sm = self.state_machine.lock().await;
        sm.current_state()
    }

    pub async fn active_session_id(&self) -> Option<Uuid> {
        let sm = self.state_machine.lock().await;
        sm.session_id()
    }

    pub async fn start_session(
        &self,
        peer: Peer,
        plan: BenchmarkPlan,
    ) -> Result<Uuid, StateMachineError> {
        let session_id = Uuid::new_v4();
        self.start_session_con_id(session_id, peer, plan).await?;
        Ok(session_id)
    }

    /// Registra una sesión con un identificador ya fijado. El extremo que responde lo
    /// necesita: el `session_id` lo decide quien inicia, y los dos deben compartirlo
    /// (FR-025). Falla si ya hay una sesión activa, que es como se rechaza la segunda
    /// solicitud (FR-017).
    pub async fn start_session_con_id(
        &self,
        session_id: Uuid,
        peer: Peer,
        plan: BenchmarkPlan,
    ) -> Result<(), StateMachineError> {
        let mut sm = self.state_machine.lock().await;
        sm.start_session(session_id, peer.instance_id)?;

        let mut p_lock = self.active_peer.lock().await;
        *p_lock = Some(peer);

        let mut plan_lock = self.active_plan.lock().await;
        *plan_lock = Some(plan);

        Ok(())
    }

    pub async fn transition_to(
        &self,
        target: SessionState,
        session_id: Uuid,
    ) -> Result<(), StateMachineError> {
        let mut sm = self.state_machine.lock().await;
        sm.transition_to(target, session_id)
    }

    pub async fn cancel(&self, session_id: Uuid) -> Result<(), StateMachineError> {
        // Se avisa primero con cortesía: si la tarea está esperando al par (no hay motor
        // en marcha todavía), puede enviar CANCEL, dar una oportunidad corta a su
        // CANCEL_ACK y terminar por sí sola. Presupuesto acotado y corto a propósito: si
        // el motor ya está en marcha no hay nada cooperativo que esperar, y FR-023/SC-004
        // exigen liberar los recursos en ≤2 s, no una despedida educada.
        if let Some(senal) = self.cancelacion.lock().await.take() {
            let _ = senal.send(true);
        }
        if let Some(tarea) = self.tarea.lock().await.take() {
            let interrumpir = tarea.abort_handle();
            if tokio::time::timeout(Duration::from_millis(500), tarea)
                .await
                .is_err()
            {
                interrumpir.abort();
            }
        }

        let mut sm = self.state_machine.lock().await;
        sm.cancel(session_id)?;
        sm.complete_cancellation();

        let mut p_lock = self.active_peer.lock().await;
        *p_lock = None;

        let mut plan_lock = self.active_plan.lock().await;
        *plan_lock = None;

        Ok(())
    }

    /// Marca una transición ya validada por la máquina de estados. Un error aquí es un
    /// fallo de programación del orden de pasos, no del usuario ni de la red.
    async fn transicion(&self, destino: SessionState, id: Uuid) -> Result<(), String> {
        self.state_machine
            .lock()
            .await
            .transition_to(destino, id)
            .map_err(|e| format!("Transición ilegal a {destino:?}: {e:?}"))
    }

    /// Inicia una sesión real hacia `peer`: conecta, ejecuta la prueba estándar en ambos
    /// sentidos y persiste el resultado. Devuelve el identificador en cuanto la sesión
    /// queda registrada; el avance se consulta por el estado, no bloqueando al llamador
    /// durante los ~25 s que dura la medición.
    pub async fn iniciar_sesion_real(
        self: &Arc<Self>,
        ctx: ContextoSesion,
        peer: Peer,
        plan: BenchmarkPlan,
    ) -> Result<Uuid, StateMachineError> {
        let session_id = self.start_session(peer.clone(), plan.clone()).await?;
        let (tx_cancel, rx_cancel) = tokio::sync::watch::channel(false);
        *self.cancelacion.lock().await = Some(tx_cancel.clone());

        let servicio = Arc::clone(self);
        let tarea = tokio::spawn(async move {
            match servicio
                .recorrido_como_iniciador(&ctx, session_id, &peer, &plan, &tx_cancel)
                .await
            {
                Ok(()) => {}
                Err(motivo) => {
                    tracing::warn!("La sesión terminó sin completarse: {motivo}");
                    let mut sm = servicio.state_machine.lock().await;
                    // Una cancelación (propia o del par) no es un fallo: FR-023 exige el
                    // estado terminal explícito `Cancelled`, no `Failed`.
                    if *rx_cancel.borrow() {
                        let _ = sm.cancel(session_id);
                        sm.complete_cancellation();
                    } else {
                        let _ = sm.fail(session_id);
                    }
                }
            }
        });
        *self.tarea.lock().await = Some(tarea);

        Ok(session_id)
    }

    async fn recorrido_como_iniciador(
        &self,
        ctx: &ContextoSesion,
        session_id: Uuid,
        peer: &Peer,
        plan: &BenchmarkPlan,
        cancelacion: &SenalCancelacion,
    ) -> Result<(), String> {
        let inicio = ahora_rfc3339();
        self.transicion(SessionState::HelloPending, session_id)
            .await?;

        let direccion: SocketAddr = peer
            .addresses
            .first()
            .ok_or("El equipo no tiene ninguna dirección conocida")?
            .parse()
            .map_err(|e| format!("Dirección del equipo no válida: {e}"))?;

        let (visto, mut stream) =
            conectar_y_saludar(&direccion.ip().to_string(), direccion.port(), &ctx.identity)
                .await?;

        // FR-013: si el certificado presentado no es el que se guardó al emparejar, el
        // equipo ya no es el mismo. No se mide contra una identidad que no se verificó.
        if !visto.fingerprint.eq_ignore_ascii_case(&peer.fingerprint) {
            return Err("La identidad del equipo ha cambiado desde el emparejamiento".into());
        }

        // Se salta el emparejamiento porque el peer ya es de confianza.
        self.transicion(SessionState::Requesting, session_id)
            .await?;
        self.transicion(SessionState::Preparing, session_id).await?;

        // La interfaz de escucha para la vuelta es la propia del canal de control: la
        // que efectivamente llega al peer, sin adivinar entre varios adaptadores.
        let local = stream
            .get_ref()
            .0
            .local_addr()
            .map_err(|e| format!("No se pudo determinar la interfaz local: {e}"))?
            .ip()
            .to_string();

        // El avance por patas viaja por un canal hasta una tarea que aplica las
        // transiciones y gobierna el muestreo. Se espera a que termine antes de
        // `Analyzing`, para que un aviso rezagado no intente mover el estado hacia atrás.
        let (avisos, aplicador) = lanzar_aplicador(
            self.state_machine.clone(),
            session_id,
            ctx.muestreo.clone(),
            direccion.ip(),
            true,
        );

        let prueba = ejecutar_prueba_estandar_como_iniciador(
            &mut stream,
            &ctx.orquestador,
            session_id,
            plan,
            &direccion.ip().to_string(),
            &local,
            |evento| {
                let _ = avisos.send(evento);
            },
            cancelacion,
        )
        .await
        .map_err(|e| format!("Fallo durante la prueba: {e}"))?;

        drop(avisos);
        let _ = aplicador.await;

        self.transicion(SessionState::Analyzing, session_id).await?;

        // Sin ninguna dirección medida no hay resultado que guardar: un rechazo o un
        // fallo previo a medir no se registra como sesión (US4/AC3, FR-034).
        if !prueba.direcciones.iter().any(|d| d.status == "completed") {
            return Err("El otro equipo no aceptó la prueba o no llegó a medir".into());
        }

        let yo = Peer::new(
            ctx.identity.instance_id,
            ctx.identity.display_name.clone(),
            ctx.identity.fingerprint.clone(),
            vec![format!("{local}:0")],
        )?;

        let prueba_interrupcion = prueba.interrupcion.clone();
        let mut resultado = ensamblar_resultado(
            &ctx.orquestador,
            session_id,
            &inicio,
            &ahora_rfc3339(),
            &prueba.plan,
            yo,
            peer.clone(),
            prueba.direcciones,
        );
        // Quien inicia construye el resultado canónico de la sesión (contrato
        // `session-result.md`, punto 1): su propia copia siempre lleva esta procedencia,
        // reconcilie o no el otro extremo (T176, FR-025).
        resultado.result_source = "initiator".to_string();

        {
            let mut conn = ctx
                .database
                .connection()
                .lock()
                .map_err(|_| "La base de datos está bloqueada".to_string())?;
            guardar_resultado_de_sesion(&mut conn, peer, &resultado)
                .map_err(|e| format!("No se pudo guardar el resultado: {e}"))?;
        }

        // Cortada tras completar una pata (FR-024): el resultado incompleto ya está
        // guardado. No se intenta el intercambio por un canal que se sabe perdido, y la
        // sesión termina como fallida con el motivo de la pérdida (NB-CONN-005).
        if let Some(motivo) = &prueba_interrupcion {
            return Err(format!(
                "Canal perdido a mitad de la prueba (NB-CONN-005): {motivo}"
            ));
        }

        // Un `SESSION_ACK` ausente o tardío no deshace lo que este equipo ya guardó: solo
        // queda registrado que el otro extremo puede haberse quedado con su propia vista
        // local en vez de la canónica (T176).
        enviar_resultado_y_esperar_ack(
            &mut stream,
            session_id,
            &resultado,
            crate::control::session_flow::ESPERA_RECONCILIACION,
        )
        .await;

        self.transicion(SessionState::Completed, session_id).await?;
        Ok(())
    }

    /// Atiende una solicitud de prueba que llega por el canal de control.
    ///
    /// Política de aceptación, deliberadamente estricta: solo se acepta de un equipo
    /// emparejado **con autoaceptación activada** (FR-015) y estando libre (FR-017).
    /// Cualquier otro caso se rechaza con un motivo cerrado. Todavía no existe diálogo de
    /// consentimiento local (FR-016), así que sin autoaceptación no hay forma de aceptar:
    /// es el valor por defecto seguro, y el punto donde enchufar ese diálogo es la
    /// decisión que se calcula aquí.
    pub async fn atender_sesion_entrante(
        self: &Arc<Self>,
        ctx: ContextoSesion,
        fingerprint: String,
        remote_addr: SocketAddr,
        mut stream: TlsStream<TcpStream>,
        primera: ProtocolEnvelope<RequestPayload>,
    ) {
        let session_id = primera.session_id.unwrap_or_else(Uuid::new_v4);
        let plan = primera.payload.plan.clone();

        let peer = {
            match ctx.database.connection().lock() {
                Ok(db) => get_peer_by_fingerprint(&db, &fingerprint).ok().flatten(),
                Err(_) => None,
            }
        };

        let decision: Result<Peer, MotivoRechazo> = match peer {
            Some(p) if p.trust_state == TrustState::TrustedAutoAccept => Ok(p),
            // FR-016: un equipo de confianza pero sin autoaceptación no se rechaza sin
            // más — se le pregunta a la persona, con un plazo (T177). Un equipo
            // desconocido o meramente «conocido» no llega a esta pregunta: el
            // consentimiento no sustituye la verificación de emparejamiento (FR-012).
            Some(p) if p.trust_state == TrustState::Trusted => {
                pedir_consentimiento(&ctx, p, plan.clone()).await
            }
            _ => Err(MotivoRechazo::RechazadoPorElUsuario),
        };

        // La reserva de la sesión única es lo último que se comprueba y lo que la
        // convierte en real: quien llega segundo recibe «ocupado», no una carrera.
        let reservado = match decision {
            Ok(p) => match self.start_session_con_id(session_id, p.clone(), plan).await {
                Ok(()) => Ok(p),
                Err(_) => Err(MotivoRechazo::OcupadoConOtraSesion),
            },
            Err(m) => Err(m),
        };

        let peer = match reservado {
            Ok(p) => p,
            Err(motivo) => {
                // Se contesta al iniciador para que no quede esperando, y se cierra. No
                // hay una sesión real detrás: la señal de cancelación es de un solo uso,
                // desechable, este rechazo no puede cancelarse porque nunca llegó a existir.
                let (tx_rechazo, _rx_rechazo) = tokio::sync::watch::channel(false);
                let _ = atender_direccion_desde(
                    &mut stream,
                    &ctx.orquestador,
                    "0.0.0.0",
                    primera,
                    |_| Err(motivo),
                    &tx_rechazo,
                )
                .await;
                return;
            }
        };

        let (tx_cancel, rx_cancel) = tokio::sync::watch::channel(false);
        *self.cancelacion.lock().await = Some(tx_cancel.clone());

        let servicio = Arc::clone(self);
        let tarea = tokio::spawn(async move {
            let resultado = servicio
                .recorrido_como_respondedor(
                    &ctx,
                    session_id,
                    &peer,
                    remote_addr,
                    stream,
                    primera,
                    &tx_cancel,
                )
                .await;
            if let Err(motivo) = resultado {
                tracing::warn!("La sesión entrante terminó sin completarse: {motivo}");
                let mut sm = servicio.state_machine.lock().await;
                if *rx_cancel.borrow() {
                    let _ = sm.cancel(session_id);
                    sm.complete_cancellation();
                } else {
                    let _ = sm.fail(session_id);
                }
            }
        });
        *self.tarea.lock().await = Some(tarea);
    }

    #[allow(clippy::too_many_arguments)]
    async fn recorrido_como_respondedor(
        &self,
        ctx: &ContextoSesion,
        session_id: Uuid,
        peer: &Peer,
        remote_addr: SocketAddr,
        mut stream: TlsStream<TcpStream>,
        primera: ProtocolEnvelope<RequestPayload>,
        cancelacion: &SenalCancelacion,
    ) -> Result<(), String> {
        let inicio = ahora_rfc3339();
        self.transicion(SessionState::HelloPending, session_id)
            .await?;
        self.transicion(SessionState::Requesting, session_id)
            .await?;
        self.transicion(SessionState::Preparing, session_id).await?;

        let local = normalizar_ip(
            stream
                .get_ref()
                .0
                .local_addr()
                .map_err(|e| format!("No se pudo determinar la interfaz local: {e}"))?
                .ip(),
        )
        .to_string();
        let remoto = normalizar_ip(remote_addr.ip()).to_string();

        let (avisos, aplicador) = lanzar_aplicador(
            self.state_machine.clone(),
            session_id,
            ctx.muestreo.clone(),
            normalizar_ip(remote_addr.ip()),
            false,
        );

        let prueba = ejecutar_prueba_estandar_como_respondedor(
            &mut stream,
            &ctx.orquestador,
            &local,
            &remoto,
            primera,
            |plan| plan.validate().map_err(|_| MotivoRechazo::PlanInvalido),
            |evento| {
                let _ = avisos.send(evento);
            },
            cancelacion,
        )
        .await
        .map_err(|e| format!("Fallo durante la prueba: {e}"))?;

        drop(avisos);
        let _ = aplicador.await;

        self.transicion(SessionState::Analyzing, session_id).await?;

        if !prueba.direcciones.iter().any(|d| d.status == "completed") {
            return Err("La prueba no llegó a medir".into());
        }

        let yo = Peer::new(
            ctx.identity.instance_id,
            ctx.identity.display_name.clone(),
            ctx.identity.fingerprint.clone(),
            vec![format!("{local}:0")],
        )?;

        // Este extremo ensambla su propia vista, con el mismo identificador y las mismas
        // cifras oficiales que el iniciador. Es la vista que se persiste si el
        // `SESSION_RESULT` canónico no llega o no coincide (contrato, punto 5); si llega y
        // coincide, se persiste esa en su lugar (T176).
        let prueba_interrupcion = prueba.interrupcion.clone();
        let local_result = ensamblar_resultado(
            &ctx.orquestador,
            session_id,
            &inicio,
            &ahora_rfc3339(),
            &prueba.plan,
            peer.clone(),
            yo,
            prueba.direcciones,
        );

        // Cortada tras completar una pata: no hay canal por el que esperar el resultado
        // canónico. Se conserva la vista local, marcada `local` (degradada), sin esperar.
        let resultado = if prueba_interrupcion.is_some() {
            local_result
        } else {
            match recibir_resultado_o_conservar_local(
                &mut stream,
                session_id,
                &local_result,
                crate::control::session_flow::ESPERA_RECONCILIACION,
            )
            .await
            {
                ResultadoReconciliado::Recibido(canonico) => *canonico,
                ResultadoReconciliado::Local => local_result,
            }
        };

        {
            let mut conn = ctx
                .database
                .connection()
                .lock()
                .map_err(|_| "La base de datos está bloqueada".to_string())?;
            guardar_resultado_de_sesion(&mut conn, peer, &resultado)
                .map_err(|e| format!("No se pudo guardar el resultado: {e}"))?;
        }

        if let Some(motivo) = prueba_interrupcion {
            return Err(format!(
                "Canal perdido a mitad de la prueba (NB-CONN-005): {motivo}"
            ));
        }

        self.transicion(SessionState::Completed, session_id).await?;
        Ok(())
    }
}

/// Pregunta a la persona si acepta la prueba de un equipo de confianza sin
/// autoaceptación (T177, FR-016). Registra la solicitud, avisa a la interfaz si hay por
/// dónde hacerlo, y espera hasta `ESPERA_DECISION`. Sin respuesta a tiempo, sin
/// diferencia con un rechazo explícito: el iniciador ve lo mismo en los dos casos.
async fn pedir_consentimiento(
    ctx: &ContextoSesion,
    peer: Peer,
    plan: BenchmarkPlan,
) -> Result<Peer, MotivoRechazo> {
    let (evento, esperar) = ctx.consentimiento.registrar(peer.clone(), plan).await;

    if let Some(emisor) = &ctx.emisor_eventos
        && let Err(e) = emisor.emitir_solicitud_entrante(&evento)
    {
        tracing::warn!(
            "No se pudo avisar de la solicitud entrante {}: {e}",
            evento.request_id
        );
    }

    let decidido = tokio::time::timeout(crate::control::consent::ESPERA_DECISION, esperar).await;
    // Por si la persona decide justo cuando el plazo expira: no dejar la solicitud
    // ofrecida en la interfaz después de haber seguido adelante sin ella.
    ctx.consentimiento.retirar(evento.request_id).await;

    match decidido {
        Ok(Ok(true)) => Ok(peer),
        _ => Err(MotivoRechazo::RechazadoPorElUsuario),
    }
}

/// Una dirección IPv4 que llega por un socket doble pila aparece como `::ffff:a.b.c.d`.
/// NTTTCP no la entiende sin `-6`, así que se devuelve a su forma IPv4.
pub(crate) fn normalizar_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(v6)),
        otra => otra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::events::EmisorDeEventos;
    use crate::sampling::SampleBatch;
    use crate::sampling::vivo::{Contadores, FuenteDeContadores, ProveedorDeContadores};

    /// Cada lectura suma 50 000 octetos enviados y ninguno recibido.
    struct Creciente(u64);
    impl FuenteDeContadores for Creciente {
        fn leer(&mut self) -> Result<Contadores, String> {
            self.0 += 50_000;
            Ok(Contadores {
                bytes_enviados: self.0,
                bytes_recibidos: 0,
            })
        }
    }
    struct ProveedorFalso;
    impl ProveedorDeContadores for ProveedorFalso {
        fn abrir(&self, _: IpAddr) -> Result<Box<dyn FuenteDeContadores>, String> {
            Ok(Box::new(Creciente(0)))
        }
    }
    #[derive(Default)]
    struct Recoge(std::sync::Mutex<Vec<SampleBatch>>);
    impl EmisorDeEventos for Recoge {
        fn emitir_muestras(&self, batch: &SampleBatch) -> Result<(), String> {
            self.0.lock().unwrap().push(batch.clone());
            Ok(())
        }
    }

    async fn servicio_en_preparacion() -> (SessionService, Uuid) {
        let servicio = SessionService::new();
        let peer = Peer::new(
            Uuid::new_v4(),
            "Par".into(),
            "2222222222222222222222222222222222222222222222222222222222222222".into(),
            vec!["127.0.0.1:7411".into()],
        )
        .unwrap();
        let id = servicio
            .start_session(peer, BenchmarkPlan::new_standard_tcp(7412))
            .await
            .unwrap();
        for estado in [
            SessionState::HelloPending,
            SessionState::Requesting,
            SessionState::Preparing,
        ] {
            servicio.transicion(estado, id).await.unwrap();
        }
        (servicio, id)
    }

    fn muestreo_falso(emisor: Arc<Recoge>) -> Muestreo {
        Muestreo {
            emisor,
            contadores: Arc::new(ProveedorFalso),
        }
    }

    fn muestras_de(emisor: &Recoge, sentido: &str) -> Vec<crate::sampling::SamplePoint> {
        emisor
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|l| l.direction == sentido)
            .flat_map(|l| l.samples.clone())
            .collect()
    }

    #[tokio::test]
    async fn t174_cada_pata_muestrea_su_contador_y_se_detiene_al_terminar() {
        let (servicio, id) = servicio_en_preparacion().await;
        let emisor = Arc::new(Recoge::default());
        let (avisos, aplicador) = lanzar_aplicador(
            servicio.state_machine.clone(),
            id,
            Some(muestreo_falso(emisor.clone())),
            "127.0.0.1".parse().unwrap(),
            true,
        );

        // Quien inicia emite en la ida: mira los octetos enviados, que crecen.
        avisos.send(EventoPata::Inicia(Direccion::Forward)).unwrap();
        tokio::time::sleep(Duration::from_millis(1_300)).await;
        assert_eq!(servicio.current_state().await, SessionState::RunningSend);
        avisos
            .send(EventoPata::Termina(Direccion::Forward))
            .unwrap();

        // En la vuelta recibe: mira los octetos recibidos, que no se mueven.
        avisos.send(EventoPata::Inicia(Direccion::Reverse)).unwrap();
        tokio::time::sleep(Duration::from_millis(1_300)).await;
        assert_eq!(servicio.current_state().await, SessionState::RunningReceive);
        drop(avisos);
        aplicador.await.unwrap();

        let ida = muestras_de(&emisor, "forward");
        let vuelta = muestras_de(&emisor, "reverse");
        assert!(ida.len() >= 2, "ida: {}", ida.len());
        assert!(ida.iter().all(|m| !m.gap && m.bps > 0));
        assert!(vuelta.len() >= 2, "vuelta: {}", vuelta.len());
        assert!(vuelta.iter().all(|m| !m.gap && m.bps == 0));

        // Terminada la sesión no sigue saliendo nada.
        let emitidos = emisor.0.lock().unwrap().len();
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(emisor.0.lock().unwrap().len(), emitidos);
    }

    #[tokio::test]
    async fn t174_soltar_el_canal_a_mitad_de_pata_detiene_el_muestreo() {
        let (servicio, id) = servicio_en_preparacion().await;
        let emisor = Arc::new(Recoge::default());
        let (avisos, aplicador) = lanzar_aplicador(
            servicio.state_machine.clone(),
            id,
            Some(muestreo_falso(emisor.clone())),
            "127.0.0.1".parse().unwrap(),
            false,
        );
        avisos.send(EventoPata::Inicia(Direccion::Forward)).unwrap();
        tokio::time::sleep(Duration::from_millis(1_200)).await;
        assert!(!emisor.0.lock().unwrap().is_empty());

        // Es lo que ocurre al cancelar: se aborta el futuro de la sesión, que suelta el
        // emisor del canal sin haber enviado `Termina`.
        drop(avisos);
        tokio::time::timeout(Duration::from_secs(2), aplicador)
            .await
            .expect("el aplicador debe terminar al cerrarse el canal")
            .unwrap();

        let emitidos = emisor.0.lock().unwrap().len();
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(emisor.0.lock().unwrap().len(), emitidos);
    }

    #[tokio::test]
    async fn t174_sin_contadores_la_sesion_sigue_y_no_hay_muestreo() {
        struct SinContadores;
        impl ProveedorDeContadores for SinContadores {
            fn abrir(&self, _: IpAddr) -> Result<Box<dyn FuenteDeContadores>, String> {
                Err("sin contadores".into())
            }
        }
        let (servicio, id) = servicio_en_preparacion().await;
        let emisor = Arc::new(Recoge::default());
        let (avisos, aplicador) = lanzar_aplicador(
            servicio.state_machine.clone(),
            id,
            Some(Muestreo {
                emisor: emisor.clone(),
                contadores: Arc::new(SinContadores),
            }),
            "127.0.0.1".parse().unwrap(),
            true,
        );
        avisos.send(EventoPata::Inicia(Direccion::Forward)).unwrap();
        avisos
            .send(EventoPata::Termina(Direccion::Forward))
            .unwrap();
        drop(avisos);
        aplicador.await.unwrap();

        // El estado sí avanzó: la falta de contadores no frena la medición.
        assert_eq!(servicio.current_state().await, SessionState::RunningSend);
        assert!(emisor.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_session_service_lifecycle() {
        let service = SessionService::new();
        assert_eq!(service.current_state().await, SessionState::Idle);

        let peer = Peer::new(
            Uuid::new_v4(),
            "Servidor-Pruebas".into(),
            "1111111111111111111111111111111111111111111111111111111111111111".into(),
            vec!["127.0.0.1:7411".into()],
        )
        .unwrap();

        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let s_id = service.start_session(peer, plan).await.expect("Iniciar");
        assert_eq!(service.current_state().await, SessionState::Connecting);

        service
            .transition_to(SessionState::HelloPending, s_id)
            .await
            .unwrap();
        assert_eq!(service.current_state().await, SessionState::HelloPending);

        service.cancel(s_id).await.expect("Cancelar sesión");
        assert_eq!(service.current_state().await, SessionState::Cancelled);
    }
}
