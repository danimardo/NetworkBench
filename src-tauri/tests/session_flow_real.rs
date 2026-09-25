//! Diálogo de sesión completo entre dos peers, con el motor NTTTCP real (T144).
//!
//! A diferencia de `engine_real.rs`, que invoca el motor directamente, esta prueba
//! atraviesa el protocolo entero: TLS mutuo, HELLO, REQUEST/RESPONSE, PREPARE/READY,
//! negociación de inicio y ENGINE_DONE, entre dos `ControlServer` reales en localhost.
//!
//! Marcada `#[ignore]` porque lanza NTTTCP de verdad y abre sockets:
//!
//! ```text
//! cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real -- --ignored --test-threads=1
//! ```

use networkbench_lib::control::engine_port::MotorNtttcp;
use networkbench_lib::control::orquestador::{Direccion, Orquestador};
use networkbench_lib::control::server::ControlServer;
use networkbench_lib::control::session_flow::{
    MotivoRechazo, atender_direccion, ejecutar_prueba_estandar_como_iniciador,
    ejecutar_prueba_estandar_como_respondedor, ensamblar_resultado, iniciar_direccion,
};
use networkbench_lib::identity::InstanceIdentity;
use networkbench_lib::model::peer::Peer;
use networkbench_lib::model::plan::{BenchmarkPlan, BenchmarkProtocol};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

fn orquestador_real() -> Orquestador {
    let motor = MotorNtttcp::new(
        PathBuf::from("../engine/ntttcp.exe"),
        networkbench_lib::engine::ntttcp::engine_sha256().to_string(),
        std::env::temp_dir(),
    );
    Orquestador::new(Arc::new(motor))
}

fn peer_de(ident: &InstanceIdentity, direccion: &str) -> Peer {
    Peer::new(
        ident.instance_id,
        ident.display_name.clone(),
        ident.fingerprint.clone(),
        vec![direccion.to_string()],
    )
    .expect("peer válido")
}

#[tokio::test]
#[ignore = "lanza dos ControlServer y NTTTCP real; ejecutar con --ignored"]
async fn t144_dialogo_completo_produce_un_resultado_con_velocidad_oficial() {
    let id_receptor = Arc::new(InstanceIdentity::generate("Receptor".into()).unwrap());
    let id_iniciador = InstanceIdentity::generate("Iniciador".into()).unwrap();

    let servidor = ControlServer::bind(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        Arc::clone(&id_receptor),
    )
    .await
    .expect("bind del servidor");
    let puerto_control = servidor.local_addr().unwrap().port();

    let mut plan = BenchmarkPlan::new_standard_tcp(5300);
    plan.measure_seconds = 5;
    plan.warmup_seconds = 0;
    plan.cooldown_seconds = 0;
    plan.streams = 1;
    let plan_receptor = plan.clone();

    // Lado receptor: acepta el saludo TLS, atiende la dirección completa con el motor
    // real, y devuelve el DirectionResult que vio desde su lado.
    let tarea_receptor = tokio::spawn(async move {
        let orquestador = orquestador_real();
        let mut saludo = servidor.accept_one().await.expect("saludo TLS+HELLO");
        let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
        atender_direccion(
            &mut saludo.stream,
            &orquestador,
            "127.0.0.1",
            |plan| {
                plan.validate().map_err(|_| {
                    networkbench_lib::control::session_flow::MotivoRechazo::PlanInvalido
                })
            },
            &tx_cancel,
        )
        .await
    });

    // Lado iniciador: conecta por el canal de control del receptor y propone la
    // dirección forward hacia sí mismo (127.0.0.1), como en engine_real.rs.
    let orquestador_ini = orquestador_real();
    let (_peer_receptor, mut stream) =
        networkbench_lib::discovery::conectar_y_saludar("127.0.0.1", puerto_control, &id_iniciador)
            .await
            .expect("conectar al receptor");

    let session_id = uuid::Uuid::new_v4();
    let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
    let resultado_iniciador = iniciar_direccion(
        &mut stream,
        &orquestador_ini,
        session_id,
        &plan,
        Direccion::Forward,
        "127.0.0.1",
        &tx_cancel,
    )
    .await
    .expect("diálogo del iniciador");

    let resultado_receptor = tarea_receptor
        .await
        .unwrap()
        .expect("diálogo del receptor")
        .resultado;

    // Las dos vistas deben coincidir en lo esencial: completada, con la misma
    // velocidad oficial (viene del receptor en las dos, FR-027).
    assert_eq!(resultado_iniciador.status, "completed");
    assert_eq!(resultado_receptor.status, "completed");
    assert!(resultado_iniciador.official_bps.is_some());
    assert_eq!(
        resultado_iniciador.official_bps, resultado_receptor.official_bps,
        "ambos extremos deben reconciliar la misma velocidad oficial (FR-025)"
    );
    // T184: la retransmisión TCP sale de los contadores del emisor; el receptor los
    // recibe por `ENGINE_DONE`, así que los dos extremos deben llegar a lo mismo.
    assert!(resultado_iniciador.retransmission.is_some());
    assert_eq!(
        resultado_iniciador.retransmission, resultado_receptor.retransmission,
        "ambos extremos deben calcular la misma retransmisión"
    );

    let inicio = peer_de(&id_iniciador, "127.0.0.1:0");
    let receptor = peer_de(&id_receptor, "127.0.0.1:0");
    let sesion = ensamblar_resultado(
        &orquestador_ini,
        session_id,
        "2026-09-24T00:00:00.000Z",
        "2026-09-24T00:00:10.000Z",
        &plan_receptor,
        inicio,
        receptor,
        vec![resultado_iniciador],
    );
    assert_eq!(sesion.status, "completed");
}

/// T178 (US5): el mismo diálogo, pero con un plan UDP, para comprobar que el protocolo
/// entre peers en sí es agnóstico al protocolo de transporte del plan —nada en
/// `session_flow.rs` ni en `orquestador.rs` distingue TCP de UDP— y que solo faltaba
/// esta evidencia de punta a punta, no código nuevo.
#[tokio::test]
#[ignore = "lanza dos ControlServer y NTTTCP real en UDP; ejecutar con --ignored"]
async fn t178_dialogo_completo_con_udp_real() {
    let id_receptor = Arc::new(InstanceIdentity::generate("ReceptorUdp".into()).unwrap());
    let id_iniciador = InstanceIdentity::generate("IniciadorUdp".into()).unwrap();

    let servidor = ControlServer::bind(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        Arc::clone(&id_receptor),
    )
    .await
    .expect("bind del servidor");
    let puerto_control = servidor.local_addr().unwrap().port();

    let mut plan = BenchmarkPlan::new_standard_tcp(5330);
    plan.protocol = BenchmarkProtocol::Udp;
    plan.measure_seconds = 5;
    plan.warmup_seconds = 0;
    plan.cooldown_seconds = 0;
    plan.streams = 1;

    let tarea_receptor = tokio::spawn(async move {
        let orquestador = orquestador_real();
        let mut saludo = servidor.accept_one().await.expect("saludo TLS+HELLO");
        let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
        atender_direccion(
            &mut saludo.stream,
            &orquestador,
            "127.0.0.1",
            |plan| {
                plan.validate().map_err(|_| {
                    networkbench_lib::control::session_flow::MotivoRechazo::PlanInvalido
                })
            },
            &tx_cancel,
        )
        .await
    });

    let orquestador_ini = orquestador_real();
    let (_peer_receptor, mut stream) =
        networkbench_lib::discovery::conectar_y_saludar("127.0.0.1", puerto_control, &id_iniciador)
            .await
            .expect("conectar al receptor");

    let session_id = uuid::Uuid::new_v4();
    let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
    let resultado_iniciador = iniciar_direccion(
        &mut stream,
        &orquestador_ini,
        session_id,
        &plan,
        Direccion::Forward,
        "127.0.0.1",
        &tx_cancel,
    )
    .await
    .expect("diálogo del iniciador en UDP");

    let resultado_receptor = tarea_receptor
        .await
        .unwrap()
        .expect("diálogo del receptor en UDP")
        .resultado;

    assert_eq!(resultado_iniciador.status, "completed");
    assert_eq!(resultado_receptor.status, "completed");
    assert!(resultado_iniciador.official_bps.is_some());
    assert_eq!(
        resultado_iniciador.official_bps, resultado_receptor.official_bps,
        "también en UDP la velocidad oficial es la del receptor, reconciliada en los dos extremos"
    );
    assert!(resultado_iniciador.sender.is_some(), "emisor UDP");
    assert!(resultado_receptor.receiver.is_some(), "receptor UDP");

    // T184: la pérdida UDP se calcula en los dos extremos con `1 − recibidos/enviados`.
    // Cada lado midió una mitad y recibió la otra por `ENGINE_DONE`, que ahora lleva los
    // contadores de paquetes; si no viajaran, uno de los dos no podría calcularla.
    let en_iniciador = resultado_iniciador
        .retransmission
        .as_ref()
        .expect("pérdida UDP calculada en el iniciador");
    let en_receptor = resultado_receptor
        .retransmission
        .as_ref()
        .expect("pérdida UDP calculada en el receptor");
    assert!(
        en_iniciador.packets_sent.is_some_and(|n| n > 0),
        "el emisor UDP debe haber informado de paquetes enviados"
    );
    assert_ne!(
        en_iniciador.level,
        networkbench_lib::diagnostic::RetransmissionLevel::NotAvailable
    );
    assert_eq!(
        en_iniciador, en_receptor,
        "los dos extremos deben llegar a la misma pérdida"
    );
}

/// T179: el mismo diálogo, pero sobre IPv6 real (`::1`), para comprobar que
/// `build_ntttcp_args` añade `-6` y que NTTTCP 5.40 lo acepta de verdad —no solo que el
/// constructor de argumentos produzca la lista correcta (eso ya lo prueba
/// `engine::ntttcp::ntttcp_tests`), sino que el motor real completa una medición.
#[tokio::test]
#[ignore = "lanza dos ControlServer y NTTTCP real sobre IPv6; ejecutar con --ignored"]
async fn t179_dialogo_completo_sobre_ipv6_real() {
    let id_receptor = Arc::new(InstanceIdentity::generate("ReceptorV6".into()).unwrap());
    let id_iniciador = InstanceIdentity::generate("IniciadorV6".into()).unwrap();

    let servidor = ControlServer::bind(
        SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), 0),
        Arc::clone(&id_receptor),
    )
    .await
    .expect("bind del servidor en ::1");
    let puerto_control = servidor.local_addr().unwrap().port();

    let mut plan = BenchmarkPlan::new_standard_tcp(5320);
    plan.measure_seconds = 5;
    plan.warmup_seconds = 0;
    plan.cooldown_seconds = 0;
    plan.streams = 1;

    let tarea_receptor = tokio::spawn(async move {
        let orquestador = orquestador_real();
        let mut saludo = servidor.accept_one().await.expect("saludo TLS+HELLO");
        let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
        atender_direccion(
            &mut saludo.stream,
            &orquestador,
            "::1",
            |plan| {
                plan.validate().map_err(|_| {
                    networkbench_lib::control::session_flow::MotivoRechazo::PlanInvalido
                })
            },
            &tx_cancel,
        )
        .await
    });

    let orquestador_ini = orquestador_real();
    let (_peer_receptor, mut stream) =
        networkbench_lib::discovery::conectar_y_saludar("::1", puerto_control, &id_iniciador)
            .await
            .expect("conectar al receptor por IPv6");

    let session_id = uuid::Uuid::new_v4();
    let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
    let resultado_iniciador = iniciar_direccion(
        &mut stream,
        &orquestador_ini,
        session_id,
        &plan,
        Direccion::Forward,
        "::1",
        &tx_cancel,
    )
    .await
    .expect("diálogo del iniciador sobre IPv6");

    let resultado_receptor = tarea_receptor
        .await
        .unwrap()
        .expect("diálogo del receptor sobre IPv6")
        .resultado;

    assert_eq!(resultado_iniciador.status, "completed");
    assert_eq!(resultado_receptor.status, "completed");
    assert!(resultado_iniciador.official_bps.is_some());
    assert_eq!(
        resultado_iniciador.official_bps, resultado_receptor.official_bps,
        "ambos extremos deben reconciliar la misma velocidad oficial también sobre IPv6"
    );
}

#[tokio::test]
#[ignore = "lanza dos ControlServer y NTTTCP real, dos veces; ejecutar con --ignored"]
async fn t144_prueba_estandar_en_ambos_sentidos() {
    use networkbench_lib::control::transport::recv_envelope;
    use networkbench_lib::model::protocol::{ProtocolEnvelope, RequestPayload};

    let id_receptor = Arc::new(InstanceIdentity::generate("Receptor".into()).unwrap());
    let id_iniciador = InstanceIdentity::generate("Iniciador".into()).unwrap();

    let servidor = ControlServer::bind(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        Arc::clone(&id_receptor),
    )
    .await
    .expect("bind del servidor");
    let puerto_control = servidor.local_addr().unwrap().port();

    let mut plan = BenchmarkPlan::new_standard_tcp(5400);
    plan.measure_seconds = 5;
    plan.warmup_seconds = 0;
    plan.cooldown_seconds = 0;
    plan.streams = 1;

    // Respondedor: como el despachador real, lee el primer mensaje y lo entrega ya leído.
    let tarea_respondedor = tokio::spawn(async move {
        let orquestador = orquestador_real();
        let mut saludo = servidor.accept_one().await.expect("saludo TLS+HELLO");
        let primera: ProtocolEnvelope<RequestPayload> = recv_envelope(&mut saludo.stream)
            .await
            .expect("primer REQUEST");
        let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
        ejecutar_prueba_estandar_como_respondedor(
            &mut saludo.stream,
            &orquestador,
            "127.0.0.1",
            "127.0.0.1",
            primera,
            |p| p.validate().map_err(|_| MotivoRechazo::PlanInvalido),
            |_| {},
            &tx_cancel,
        )
        .await
    });

    let orquestador_ini = orquestador_real();
    let (_peer, mut stream) =
        networkbench_lib::discovery::conectar_y_saludar("127.0.0.1", puerto_control, &id_iniciador)
            .await
            .expect("conectar");

    let session_id = uuid::Uuid::new_v4();
    let (tx_cancel, _rx_cancel) = tokio::sync::watch::channel(false);
    let iniciador = ejecutar_prueba_estandar_como_iniciador(
        &mut stream,
        &orquestador_ini,
        session_id,
        &plan,
        "127.0.0.1",
        "127.0.0.1",
        |_| {},
        &tx_cancel,
    )
    .await
    .expect("prueba del iniciador");

    let respondedor = tarea_respondedor
        .await
        .unwrap()
        .expect("prueba del respondedor");

    // Dos patas, ambas completadas, y las dos vistas coinciden.
    assert_eq!(iniciador.direcciones.len(), 2);
    assert_eq!(respondedor.direcciones.len(), 2);
    for (i, d) in iniciador.direcciones.iter().enumerate() {
        assert_eq!(d.status, "completed", "pata {i} del iniciador");
        assert!(d.official_bps.is_some(), "pata {i} sin velocidad oficial");
    }
    assert_eq!(iniciador.direcciones[0].direction, "forward");
    assert_eq!(iniciador.direcciones[1].direction, "reverse");

    // FR-025: mismo identificador de sesión y mismas cifras oficiales en los dos extremos.
    assert_eq!(respondedor.session_id, session_id);
    for i in 0..2 {
        assert_eq!(
            iniciador.direcciones[i].official_bps, respondedor.direcciones[i].official_bps,
            "la pata {i} debe reconciliar la misma velocidad oficial en ambos extremos"
        );
    }

    let sesion = ensamblar_resultado(
        &orquestador_ini,
        session_id,
        "2026-09-25T00:00:00.000Z",
        "2026-09-25T00:00:20.000Z",
        &iniciador.plan,
        peer_de(&id_iniciador, "127.0.0.1:0"),
        peer_de(&id_receptor, "127.0.0.1:0"),
        iniciador.direcciones,
    );
    assert_eq!(sesion.status, "completed");
    assert_eq!(sesion.directions.len(), 2);
    assert!(
        sesion.asymmetry.is_some(),
        "con dos sentidos debe evaluarse la asimetría"
    );
}
