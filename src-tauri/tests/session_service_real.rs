//! Sesión completa a través de `SessionService`, entre dos nodos con todo su equipamiento
//! (T144): identidad, base de datos propia, servidor de control, despachador y motor.
//!
//! A diferencia de `session_flow_real.rs`, que llama a las funciones del protocolo, esta
//! prueba entra por donde entra la aplicación: `iniciar_sesion_real` en un extremo y el
//! despachador del servidor en el otro. Comprueba el cableado, no solo el diálogo.
//!
//! Las que lanzan NTTTCP van `#[ignore]`:
//!
//! ```text
//! cargo test --manifest-path src-tauri/Cargo.toml --test session_service_real -- --ignored --test-threads=1
//! ```

use networkbench_lib::control::despachador::despachar;
use networkbench_lib::control::domain::SessionState;
use networkbench_lib::control::engine_port::{MotorDeLaboratorio, MotorNtttcp};
use networkbench_lib::control::orquestador::Orquestador;
use networkbench_lib::control::server::ControlServer;
use networkbench_lib::control::service::{ContextoSesion, SessionService};
use networkbench_lib::history::database::Database;
use networkbench_lib::history::{get_session_by_id, upsert_peer};
use networkbench_lib::identity::InstanceIdentity;
use networkbench_lib::ipc::events::EmisorDeEventos;
use networkbench_lib::model::peer::{Peer, TrustState};
use networkbench_lib::model::plan::BenchmarkPlan;
use networkbench_lib::sampling::SampleBatch;
use networkbench_lib::sampling::vivo::{ContadoresWindows, Muestreo};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Guarda los lotes de muestras y las solicitudes entrantes que la sesión emitiría hacia
/// la interfaz.
#[derive(Default)]
struct Recoge(
    Mutex<Vec<SampleBatch>>,
    Mutex<Vec<networkbench_lib::control::consent::SolicitudEntranteEvento>>,
    Mutex<Vec<networkbench_lib::control::consent::EmparejamientoEntranteEvento>>,
);

impl EmisorDeEventos for Recoge {
    fn emitir_muestras(&self, batch: &SampleBatch) -> Result<(), String> {
        self.0.lock().unwrap().push(batch.clone());
        Ok(())
    }

    fn emitir_solicitud_entrante(
        &self,
        solicitud: &networkbench_lib::control::consent::SolicitudEntranteEvento,
    ) -> Result<(), String> {
        self.1.lock().unwrap().push(solicitud.clone());
        Ok(())
    }

    fn emitir_emparejamiento_entrante(
        &self,
        emparejamiento: &networkbench_lib::control::consent::EmparejamientoEntranteEvento,
    ) -> Result<(), String> {
        self.2.lock().unwrap().push(emparejamiento.clone());
        Ok(())
    }
}

/// Un equipo completo de la prueba.
struct Nodo {
    muestras: Arc<Recoge>,
    identity: Arc<InstanceIdentity>,
    servicio: Arc<SessionService>,
    ctx: ContextoSesion,
    puerto_control: u16,
}

async fn nodo(nombre: &str, motor_real: bool) -> Nodo {
    let identity = Arc::new(InstanceIdentity::generate(nombre.into()).unwrap());
    let ruta = std::env::temp_dir().join(format!("nb_nodo_{}_{}.db", nombre, Uuid::new_v4()));
    let database = Arc::new(Database::open(ruta).expect("abrir base de datos"));

    let orquestador = if motor_real {
        Orquestador::new(Arc::new(MotorNtttcp::new(
            PathBuf::from("../engine/ntttcp.exe"),
            networkbench_lib::engine::ntttcp::engine_sha256().to_string(),
            std::env::temp_dir(),
        )))
    } else {
        Orquestador::new(Arc::new(MotorDeLaboratorio::con_respuestas(vec![])))
    };

    // Solo los nodos con motor real muestrean: con el de laboratorio no viaja tráfico y
    // los contadores de la interfaz medirían otra cosa.
    let muestras = Arc::new(Recoge::default());
    let muestreo = motor_real.then(|| Muestreo {
        emisor: muestras.clone(),
        contadores: Arc::new(ContadoresWindows),
    });

    let ctx = ContextoSesion {
        orquestador: Arc::new(orquestador),
        identity: Arc::clone(&identity),
        database,
        muestreo,
        consentimiento: Arc::new(networkbench_lib::control::consent::SolicitudesEntrantes::new()),
        emisor_eventos: Some(muestras.clone()),
        emparejamientos: Arc::new(
            networkbench_lib::control::consent::EmparejamientosEntrantes::new(),
        ),
    };
    let servicio = Arc::new(SessionService::new());

    let servidor = ControlServer::bind(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        Arc::clone(&identity),
    )
    .await
    .expect("bind");
    let puerto_control = servidor.local_addr().unwrap().port();

    // Bucle equivalente al de `app::start_control_server`, sin Tauri.
    let (s, c) = (Arc::clone(&servicio), ctx.clone());
    tokio::spawn(async move {
        while let Ok(saludo) = servidor.accept_one().await {
            tokio::spawn(despachar(Arc::clone(&s), c.clone(), saludo));
        }
    });

    Nodo {
        muestras,
        identity,
        servicio,
        ctx,
        puerto_control,
    }
}

/// Registra en `en` al equipo `otro` con el estado de confianza indicado.
fn conocer(en: &Nodo, otro: &Nodo, confianza: TrustState) -> Peer {
    let mut peer = Peer::new(
        otro.identity.instance_id,
        otro.identity.display_name.clone(),
        otro.identity.fingerprint.clone(),
        vec![format!("127.0.0.1:{}", otro.puerto_control)],
    )
    .unwrap();
    peer.trust_state = confianza;
    let db = en.ctx.database.connection().lock().unwrap();
    upsert_peer(&db, &peer).unwrap();
    peer
}

fn plan_corto(puerto: u16) -> BenchmarkPlan {
    let mut plan = BenchmarkPlan::new_standard_tcp(puerto);
    plan.measure_seconds = 5;
    plan.warmup_seconds = 0;
    plan.cooldown_seconds = 0;
    plan.streams = 1;
    plan
}

async fn esperar_estado_terminal(servicio: &SessionService, limite: Duration) -> SessionState {
    let inicio = std::time::Instant::now();
    loop {
        let estado = servicio.current_state().await;
        if estado.is_terminal() || inicio.elapsed() > limite {
            return estado;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

fn sesiones_guardadas(n: &Nodo) -> i64 {
    let db = n.ctx.database.connection().lock().unwrap();
    db.query_row("SELECT COUNT(*) FROM sessions", [], |f| f.get(0))
        .unwrap()
}

/// Espera a que aparezca una solicitud pendiente en `nodo` y la decide (T177). El plazo
/// de producción es de 60 s; aquí basta con que la tarea del despachador haya tenido
/// tiempo de registrarla, que es casi instantáneo.
async fn decidir_primera_solicitud(nodo: &Nodo, aceptar: bool) -> Uuid {
    let inicio = std::time::Instant::now();
    loop {
        let pendientes = nodo.ctx.consentimiento.listar().await;
        if let Some(solicitud) = pendientes.into_iter().next() {
            nodo.ctx
                .consentimiento
                .responder(solicitud.request_id, aceptar)
                .await
                .expect("la solicitud debía seguir pendiente");
            return solicitud.request_id;
        }
        if inicio.elapsed() > Duration::from_secs(5) {
            panic!("no apareció ninguna solicitud pendiente de consentimiento");
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// T177 (FR-016): un equipo de confianza sin autoaceptación ya no se rechaza sin más —se
/// le pregunta a la persona—, y rechazar esa pregunta termina igual que el rechazo
/// automático de antes: nada se reserva ni se guarda.
#[tokio::test]
async fn t144_un_equipo_sin_autoaceptacion_rechaza_y_no_se_guarda_nada() {
    let a = nodo("A", false).await;
    let b = nodo("B", false).await;

    conocer(&b, &a, TrustState::Trusted);
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    a.servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan_corto(5510))
        .await
        .expect("iniciar");

    // La persona en B ve la solicitud y la rechaza.
    decidir_primera_solicitud(&b, false).await;

    let estado = esperar_estado_terminal(&a.servicio, Duration::from_secs(20)).await;
    assert_eq!(estado, SessionState::Failed);
    assert_eq!(
        sesiones_guardadas(&a),
        0,
        "un rechazo no se registra como sesión"
    );
    assert_eq!(sesiones_guardadas(&b), 0);
    assert_eq!(
        b.servicio.current_state().await,
        SessionState::Idle,
        "el receptor no debe haber reservado la sesión"
    );
}

/// T177: un equipo meramente «conocido» (emparejado, pero sin la confianza explícita que
/// exige medir) no llega a preguntarle nada a la persona: el consentimiento de esta tarea
/// es para «confío pero no quiero autoaceptar», no un atajo alrededor de la confianza.
#[tokio::test]
async fn t177_un_equipo_solo_conocido_se_rechaza_sin_preguntar() {
    let a = nodo("A", false).await;
    let b = nodo("B", false).await;

    conocer(&b, &a, TrustState::Known);
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    a.servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan_corto(5512))
        .await
        .expect("iniciar");

    let estado = esperar_estado_terminal(&a.servicio, Duration::from_secs(20)).await;
    assert_eq!(estado, SessionState::Failed);
    assert!(
        b.ctx.consentimiento.listar().await.is_empty(),
        "un equipo solo conocido no debe generar una solicitud de consentimiento"
    );
}

/// Un equipo desconocido para el receptor recibe el mismo rechazo.
#[tokio::test]
async fn t144_un_equipo_desconocido_es_rechazado() {
    let a = nodo("A", false).await;
    let b = nodo("B", false).await;

    // B no sabe quién es A.
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    a.servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan_corto(5511))
        .await
        .expect("iniciar");

    let estado = esperar_estado_terminal(&a.servicio, Duration::from_secs(20)).await;
    assert_eq!(estado, SessionState::Failed);
    assert_eq!(sesiones_guardadas(&b), 0);
}

/// Un receptor que ya tiene una sesión activa rechaza la segunda, sin pisar la primera (FR-017).
#[tokio::test]
async fn t144_un_receptor_ocupado_rechaza_la_segunda_solicitud() {
    let a = nodo("A", false).await;
    let b = nodo("B", false).await;
    let peer_a = conocer(&b, &a, TrustState::TrustedAutoAccept);
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    // B ya está atendiendo otra sesión.
    let sesion_previa = Uuid::new_v4();
    b.servicio
        .start_session_con_id(sesion_previa, peer_a, plan_corto(5513))
        .await
        .expect("reservar la sesión previa en B");

    a.servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan_corto(5514))
        .await
        .expect("iniciar");

    let estado = esperar_estado_terminal(&a.servicio, Duration::from_secs(20)).await;
    assert_eq!(estado, SessionState::Failed);
    assert_eq!(sesiones_guardadas(&a), 0);

    // La sesión que ya estaba en curso en B sigue siendo la suya, intacta.
    assert_eq!(b.servicio.active_session_id().await, Some(sesion_previa));
    assert!(b.servicio.current_state().await.is_active());
}

/// Un equipo cuyo certificado ya no es el guardado no se mide (FR-013).
#[tokio::test]
async fn t144_una_identidad_cambiada_no_llega_a_medir() {
    let a = nodo("A", false).await;
    let b = nodo("B", false).await;
    conocer(&b, &a, TrustState::TrustedAutoAccept);

    // A guarda una huella distinta para B, como si B hubiera cambiado de identidad.
    let mut peer_b = conocer(&a, &b, TrustState::Trusted);
    peer_b.fingerprint = "f".repeat(64);

    a.servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan_corto(5512))
        .await
        .expect("iniciar");

    let estado = esperar_estado_terminal(&a.servicio, Duration::from_secs(20)).await;
    assert_eq!(estado, SessionState::Failed);
    assert_eq!(sesiones_guardadas(&a), 0);
    assert_eq!(
        b.servicio.current_state().await,
        SessionState::Idle,
        "B no debe haber recibido ninguna solicitud"
    );
}

/// Recorrido completo con NTTTCP real: la prueba que da sentido a todo lo anterior.
#[tokio::test]
#[ignore = "lanza NTTTCP real dos veces; ejecutar con --ignored"]
async fn t144_sesion_completa_entre_dos_servicios_con_motor_real() {
    let a = nodo("A", true).await;
    let b = nodo("B", true).await;

    conocer(&b, &a, TrustState::TrustedAutoAccept);
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    let session_id = a
        .servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan_corto(5520))
        .await
        .expect("iniciar");

    let estado_a = esperar_estado_terminal(&a.servicio, Duration::from_secs(90)).await;
    assert_eq!(
        estado_a,
        SessionState::Completed,
        "el iniciador debe completar"
    );

    let estado_b = esperar_estado_terminal(&b.servicio, Duration::from_secs(30)).await;
    assert_eq!(
        estado_b,
        SessionState::Completed,
        "el receptor debe completar"
    );

    // FR-025: el mismo identificador y las mismas cifras oficiales en los dos historiales.
    let en_a = {
        let db = a.ctx.database.connection().lock().unwrap();
        get_session_by_id(&db, &session_id)
            .unwrap()
            .expect("sesión en A")
    };
    let en_b = {
        let db = b.ctx.database.connection().lock().unwrap();
        get_session_by_id(&db, &session_id)
            .unwrap()
            .expect("sesión en B")
    };
    assert_eq!(en_a.status, "completed");
    assert_eq!(en_b.status, "completed");
    assert!(en_a.forward_bps.is_some() && en_a.reverse_bps.is_some());
    assert_eq!(en_a.forward_bps, en_b.forward_bps);
    assert_eq!(en_a.reverse_bps, en_b.reverse_bps);
    assert!(!en_a.is_partial);

    // T174: en bucle local Windows no expone contadores de interfaz, así que el
    // muestreo en vivo se declara no disponible y no se emite nada (ni ceros con aspecto
    // de medición). La sesión, mientras tanto, completa igual: el resultado no depende
    // del muestreo. Que con tráfico real sí salgan muestras solo se puede comprobar entre
    // dos equipos; aquí lo cubren las pruebas de `service.rs` con un proveedor falso.
    for (nombre, nodo) in [("A", &a), ("B", &b)] {
        assert!(
            nodo.muestras.0.lock().unwrap().is_empty(),
            "{nombre}: en bucle local no debe emitirse muestreo"
        );
    }

    // T176: A construyó el resultado canónico y B lo recibió, lo validó y lo adoptó en
    // vez de conservar el suyo propio — los dos quedan con la misma procedencia.
    let fuente = |registro: &networkbench_lib::history::sessions::SessionRecord| {
        let json: serde_json::Value =
            serde_json::from_str(registro.result_json.as_ref().unwrap()).unwrap();
        json["resultSource"].as_str().unwrap().to_string()
    };
    assert_eq!(fuente(&en_a), "initiator");
    assert_eq!(
        fuente(&en_b),
        "initiator",
        "B debió recibir y validar el SESSION_RESULT canónico de A, no quedarse con el suyo"
    );
}

/// T177 (FR-016): un equipo `Trusted` sin autoaceptación puede completar la prueba real
/// si la persona en el receptor acepta la solicitud que se le enseña. La aceptación llega
/// por la misma vía que usaría la interfaz (`SolicitudesEntrantes::responder`), no por un
/// atajo interno que se salte el mecanismo de consentimiento.
#[tokio::test]
#[ignore = "lanza NTTTCP real dos veces; ejecutar con --ignored"]
async fn t177_un_equipo_de_confianza_sin_autoaceptacion_completa_si_se_acepta() {
    let a = nodo("A", true).await;
    let b = nodo("B", true).await;

    // Ninguno de los dos tiene autoaceptación: B debe preguntar antes de medir.
    conocer(&b, &a, TrustState::Trusted);
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    let session_id = a
        .servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan_corto(5560))
        .await
        .expect("iniciar");

    // La solicitud llegó tanto al almacén de consentimiento como al evento hacia la
    // interfaz, con el equipo y el plan correctos.
    let id_solicitud = decidir_primera_solicitud(&b, true).await;
    {
        let avisos = b.muestras.1.lock().unwrap();
        assert_eq!(avisos.len(), 1);
        assert_eq!(avisos[0].request_id, id_solicitud);
        assert_eq!(avisos[0].peer.instance_id, a.identity.instance_id);
    }

    let estado_a = esperar_estado_terminal(&a.servicio, Duration::from_secs(90)).await;
    assert_eq!(
        estado_a,
        SessionState::Completed,
        "aceptar la solicitud debe permitir completar la prueba, no solo reservarla"
    );
    let estado_b = esperar_estado_terminal(&b.servicio, Duration::from_secs(30)).await;
    assert_eq!(estado_b, SessionState::Completed);

    let en_a = {
        let db = a.ctx.database.connection().lock().unwrap();
        get_session_by_id(&db, &session_id)
            .unwrap()
            .expect("sesión en A")
    };
    assert_eq!(en_a.status, "completed");
    assert!(en_a.forward_bps.is_some() && en_a.reverse_bps.is_some());
}

/// T175: cancelar antes de que arranque el motor —en la ventana en la que solo se
/// intercambian mensajes— avisa al otro extremo con `CANCEL`/`CANCEL_ACK` en vez de
/// dejar que note un corte de socket, y los dos extremos terminan en `Cancelled`, no en
/// `Failed`. El motor real está disponible (autoaceptación en B) para que, si la
/// cancelación llegara tarde, se note: no debería quedar ningún `ntttcp.exe`.
#[tokio::test]
#[ignore = "usa el motor real por si la cancelación llega tarde; ejecutar con --ignored"]
async fn t175_cancelar_antes_de_que_arranque_el_motor_es_cortes_en_los_dos_extremos() {
    let a = nodo("A", true).await;
    let b = nodo("B", true).await;
    conocer(&b, &a, TrustState::TrustedAutoAccept);
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    let plan = plan_corto(5540);

    let session_id = a
        .servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan)
        .await
        .expect("iniciar");

    // Sin ninguna espera deliberada: se cancela lo antes posible, para pillar el diálogo
    // todavía intercambiando REQUEST/RESPONSE/PREPARE/READY, antes de lanzar NTTTCP.
    let antes = std::time::Instant::now();
    a.servicio.cancel(session_id).await.expect("cancelar");
    assert!(
        antes.elapsed() < Duration::from_secs(2),
        "SC-004: cancelar no debe tardar más de 2 s"
    );

    let estado_a = a.servicio.current_state().await;
    assert_eq!(
        estado_a,
        SessionState::Cancelled,
        "el iniciador debe quedar Cancelled, no Failed"
    );

    let estado_b = esperar_estado_terminal(&b.servicio, Duration::from_secs(5)).await;
    assert_eq!(
        estado_b,
        SessionState::Cancelled,
        "el respondedor debe enterarse por CANCEL/CANCEL_ACK y quedar Cancelled, no Failed"
    );

    assert_eq!(sesiones_guardadas(&a), 0, "una cancelación no se persiste");
    assert_eq!(sesiones_guardadas(&b), 0, "una cancelación no se persiste");

    let salida = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq ntttcp.exe", "/NH"])
        .output()
        .unwrap();
    assert!(
        !String::from_utf8_lossy(&salida.stdout).contains("ntttcp.exe"),
        "la cancelación fue antes de arrancar el motor: no debería haber ninguno corriendo"
    );
}

/// Cancelar a mitad de la medición corta el canal y no deja procesos del motor.
#[tokio::test]
#[ignore = "lanza NTTTCP real y comprueba procesos; ejecutar con --ignored"]
async fn t144_cancelar_a_mitad_no_deja_procesos_huerfanos() {
    let a = nodo("A", true).await;
    let b = nodo("B", true).await;
    conocer(&b, &a, TrustState::TrustedAutoAccept);
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    let mut plan = plan_corto(5530);
    plan.measure_seconds = 30; // suficiente para cancelar claramente a mitad

    let session_id = a
        .servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan)
        .await
        .expect("iniciar");

    // Deja que arranque la medición.
    tokio::time::sleep(Duration::from_secs(4)).await;
    assert!(a.servicio.current_state().await.is_active());

    let antes = std::time::Instant::now();
    a.servicio.cancel(session_id).await.expect("cancelar");
    assert!(
        antes.elapsed() < Duration::from_secs(2),
        "SC-004: cancelar no debe tardar más de 2 s"
    );
    assert_eq!(a.servicio.current_state().await, SessionState::Cancelled);
    assert_eq!(
        sesiones_guardadas(&a),
        0,
        "una sesión cancelada sin medir no se guarda"
    );

    // Los procesos del motor de los dos extremos deben terminar por sí solos.
    let mut quedan = true;
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let salida = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq ntttcp.exe", "/NH"])
            .output()
            .unwrap();
        if !String::from_utf8_lossy(&salida.stdout).contains("ntttcp.exe") {
            quedan = false;
            break;
        }
    }
    assert!(!quedan, "tras cancelar no debe quedar ningún ntttcp.exe");
}

// --- T182: emparejamiento entrante (FR-012), sobre TLS real y sin motor ---

/// Espera a que aparezca un emparejamiento pendiente en `nodo` y lo decide.
async fn decidir_primer_emparejamiento(nodo: &Nodo, aceptar: bool) {
    let inicio = std::time::Instant::now();
    loop {
        if let Some(e) = nodo.ctx.emparejamientos.listar().await.into_iter().next() {
            nodo.ctx
                .emparejamientos
                .responder(e.pairing_id, aceptar)
                .await
                .expect("el emparejamiento debía seguir pendiente");
            return;
        }
        if inicio.elapsed() > Duration::from_secs(5) {
            panic!("no apareció ningún emparejamiento pendiente");
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

fn peer_guardado(en: &Nodo, otro: &Nodo) -> Option<Peer> {
    let db = en.ctx.database.connection().lock().unwrap();
    networkbench_lib::history::get_peer_by_fingerprint(&db, &otro.identity.fingerprint).unwrap()
}

/// Lo que haría `peers_pairing_start` en A hacia B, hasta tener el código.
async fn emparejar_desde(
    a: &Nodo,
    b: &Nodo,
) -> (
    networkbench_lib::control::pairing_flow::EmparejamientoEnCurso,
    tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
) {
    let (peer_b, stream) =
        networkbench_lib::discovery::conectar_y_saludar("127.0.0.1", b.puerto_control, &a.identity)
            .await
            .expect("conectar");
    let emp = networkbench_lib::control::pairing_flow::EmparejamientoEnCurso::iniciar(
        &a.identity.fingerprint,
        &peer_b.fingerprint,
    )
    .unwrap();
    (emp, stream)
}

/// Decide en segundo plano la primera solicitud que aparezca en `b`.
fn decidir_en_segundo_plano(
    b: &Nodo,
    aceptar: bool,
) -> tokio::task::JoinHandle<networkbench_lib::control::consent::EmparejamientoEntranteEvento> {
    let almacen = b.ctx.emparejamientos.clone();
    tokio::spawn(async move {
        let inicio = std::time::Instant::now();
        loop {
            if let Some(e) = almacen.listar().await.into_iter().next() {
                almacen.responder(e.pairing_id, aceptar).await.unwrap();
                return e;
            }
            assert!(inicio.elapsed() < Duration::from_secs(5));
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
}

#[tokio::test]
async fn t182_un_emparejamiento_entrante_aceptado_guarda_confianza_sin_autoaceptacion() {
    let a = nodo("A", false).await;
    let b = nodo("B", false).await;
    assert!(peer_guardado(&b, &a).is_none(), "B no conoce a A todavía");

    let (emp, mut stream) = emparejar_desde(&a, &b).await;
    let decision = decidir_en_segundo_plano(&b, true);

    let aceptado =
        networkbench_lib::control::pairing_flow::solicitar_emparejamiento(&mut stream, &emp, true)
            .await
            .expect("solicitar");
    assert!(aceptado, "B dijo que sí");

    // Lo que B enseñó a su persona es el mismo código que ve la de A: es lo único que
    // hace que la comparación humana valga algo.
    let visto_en_b = decision.await.unwrap();
    assert_eq!(visto_en_b.verification_code, emp.codigo);
    assert_eq!(visto_en_b.pairing_id, emp.id);
    assert_eq!(
        b.muestras.2.lock().unwrap().len(),
        1,
        "se avisó a la interfaz"
    );

    let guardado = peer_guardado(&b, &a).expect("B debe haber guardado a A");
    assert_eq!(guardado.trust_state, TrustState::Trusted);
    assert!(
        !guardado.auto_accept,
        "la autoaceptación no se concede al emparejar"
    );
    assert_eq!(guardado.instance_id, a.identity.instance_id);
    // Las direcciones no se persisten (el historial las carga vacías); la que se calculó
    // para mostrar viajó en el evento hacia la interfaz.
    assert!(visto_en_b.peer.addresses[0].ends_with(":7411"));
}

#[tokio::test]
async fn t182_un_emparejamiento_entrante_rechazado_no_guarda_nada() {
    let a = nodo("A", false).await;
    let b = nodo("B", false).await;

    let (emp, mut stream) = emparejar_desde(&a, &b).await;
    let decision = decidir_en_segundo_plano(&b, false);

    let aceptado =
        networkbench_lib::control::pairing_flow::solicitar_emparejamiento(&mut stream, &emp, true)
            .await
            .expect("solicitar");
    decision.await.unwrap();
    assert!(!aceptado);
    assert!(peer_guardado(&b, &a).is_none(), "un no no deja confianza");
}

/// Un código que no cuadra no llega a la persona: se contesta `verificationFailed` sin
/// preguntar y sin registrar nada pendiente.
#[tokio::test]
async fn t182_un_codigo_incorrecto_se_rechaza_sin_preguntar() {
    use networkbench_lib::control::transport::{recv_envelope, send_envelope};
    use networkbench_lib::model::protocol::{
        PairRequestPayload, PairResultPayload, ProtocolEnvelope, ProtocolMessageType,
    };

    let a = nodo("A", false).await;
    let b = nodo("B", false).await;
    let (emp, mut stream) = emparejar_desde(&a, &b).await;

    let codigo_malo = if emp.codigo == "000000" {
        "111111"
    } else {
        "000000"
    };
    send_envelope(
        &mut stream,
        &ProtocolEnvelope {
            msg_type: ProtocolMessageType::PairRequest,
            id: emp.id,
            session_id: None,
            ts: "2026-09-25T00:00:00.000Z".to_string(),
            in_reply_to: None,
            payload: PairRequestPayload {
                pairing_code: codigo_malo.to_string(),
            },
        },
    )
    .await
    .unwrap();

    let respuesta: ProtocolEnvelope<PairResultPayload> =
        tokio::time::timeout(Duration::from_secs(5), recv_envelope(&mut stream))
            .await
            .expect("B debe contestar sin esperar a nadie")
            .unwrap();
    assert!(!respuesta.payload.accepted);
    assert_eq!(
        respuesta.payload.reason.as_deref(),
        Some("verificationFailed")
    );
    assert!(b.ctx.emparejamientos.listar().await.is_empty());
    assert!(
        b.muestras.2.lock().unwrap().is_empty(),
        "no se avisó a la interfaz"
    );
    assert!(peer_guardado(&b, &a).is_none());
}

/// Un segundo emparejamiento del mismo equipo mientras el primero espera no acumula
/// preguntas: se rechaza al instante.
#[tokio::test]
async fn t182_un_segundo_emparejamiento_del_mismo_equipo_se_rechaza_mientras_hay_uno_pendiente() {
    let a = nodo("A", false).await;
    let b = nodo("B", false).await;

    let (emp1, mut s1) = emparejar_desde(&a, &b).await;
    let primera = tokio::spawn(async move {
        networkbench_lib::control::pairing_flow::solicitar_emparejamiento(&mut s1, &emp1, true)
            .await
    });
    let inicio = std::time::Instant::now();
    while b.ctx.emparejamientos.listar().await.is_empty() {
        assert!(inicio.elapsed() < Duration::from_secs(5));
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    let (emp2, mut s2) = emparejar_desde(&a, &b).await;
    let segunda = tokio::time::timeout(
        Duration::from_secs(5),
        networkbench_lib::control::pairing_flow::solicitar_emparejamiento(&mut s2, &emp2, true),
    )
    .await
    .expect("el segundo debe rechazarse sin esperar")
    .expect("solicitar");
    assert!(!segunda);
    assert_eq!(
        b.ctx.emparejamientos.listar().await.len(),
        1,
        "sigue solo la primera"
    );

    decidir_primer_emparejamiento(&b, true).await;
    assert!(primera.await.unwrap().unwrap());
}

/// T171 (FR-024, US3/AC3, `Historias.md` §8.5/§19.3): el canal se pierde a mitad de la
/// segunda pata, con NTTTCP real. La primera, ya completa, se conserva; la sesión se guarda
/// como **incompleta**, la segunda no aparece como continua, y no queda ningún proceso.
///
/// La pérdida se provoca cancelando en B mientras B emite la vuelta: con el motor en
/// marcha no hay punto cooperativo, así que B aborta su tarea y A solo ve caer el canal,
/// sin ningún `CANCEL` (A no ha cancelado nada: para A es una pérdida, no una cancelación).
#[tokio::test]
#[ignore = "lanza NTTTCP real y corta el canal a mitad; ejecutar con --ignored"]
async fn t171_una_perdida_de_canal_en_la_segunda_pata_guarda_la_sesion_incompleta() {
    let a = nodo("A", true).await;
    let b = nodo("B", true).await;
    conocer(&b, &a, TrustState::TrustedAutoAccept);
    let peer_b = conocer(&a, &b, TrustState::Trusted);

    let session_id = a
        .servicio
        .iniciar_sesion_real(a.ctx.clone(), peer_b, plan_corto(5570))
        .await
        .expect("iniciar");

    // B es receptor en la ida y emisor en la vuelta: `RunningSend` en B es la segunda pata.
    let inicio = std::time::Instant::now();
    while b.servicio.current_state().await != SessionState::RunningSend {
        assert!(
            inicio.elapsed() < Duration::from_secs(60),
            "B no llegó a la segunda pata"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    let b_id = b.servicio.active_session_id().await.expect("sesión en B");
    b.servicio.cancel(b_id).await.expect("cancelar en B");

    let estado_a = esperar_estado_terminal(&a.servicio, Duration::from_secs(90)).await;
    assert_eq!(
        estado_a,
        SessionState::Failed,
        "para A es una pérdida de canal (NB-CONN-005), no una cancelación ni un éxito"
    );

    let en_a = {
        let db = a.ctx.database.connection().lock().unwrap();
        get_session_by_id(&db, &session_id)
            .unwrap()
            .expect("A debe haber guardado lo completado")
    };
    assert_eq!(en_a.status, "incomplete");
    assert!(en_a.is_partial);
    assert!(en_a.forward_bps.is_some(), "la primera pata se conserva");
    assert!(
        en_a.reverse_bps.is_none(),
        "la pata cortada no aparece como continua ni con velocidad"
    );

    assert_eq!(b.servicio.current_state().await, SessionState::Cancelled);
    assert_eq!(sesiones_guardadas(&b), 0, "B canceló: no guarda nada");

    let mut quedan = true;
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let salida = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq ntttcp.exe", "/NH"])
            .output()
            .unwrap();
        if !String::from_utf8_lossy(&salida.stdout).contains("ntttcp.exe") {
            quedan = false;
            break;
        }
    }
    assert!(
        !quedan,
        "tras perder el canal no debe quedar ningún ntttcp.exe"
    );
}
