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
    atender_direccion, ensamblar_resultado, iniciar_direccion,
};
use networkbench_lib::identity::InstanceIdentity;
use networkbench_lib::model::peer::Peer;
use networkbench_lib::model::plan::BenchmarkPlan;
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
        atender_direccion(&mut saludo.stream, &orquestador, "127.0.0.1", |plan| {
            plan.validate()
                .map_err(|_| networkbench_lib::control::session_flow::MotivoRechazo::PlanInvalido)
        })
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
    let resultado_iniciador = iniciar_direccion(
        &mut stream,
        &orquestador_ini,
        session_id,
        &plan,
        Direccion::Forward,
        "127.0.0.1",
    )
    .await
    .expect("diálogo del iniciador");

    let resultado_receptor = tarea_receptor.await.unwrap().expect("diálogo del receptor");

    // Las dos vistas deben coincidir en lo esencial: completada, con la misma
    // velocidad oficial (viene del receptor en las dos, FR-027).
    assert_eq!(resultado_iniciador.status, "completed");
    assert_eq!(resultado_receptor.status, "completed");
    assert!(resultado_iniciador.official_bps.is_some());
    assert_eq!(
        resultado_iniciador.official_bps, resultado_receptor.official_bps,
        "ambos extremos deben reconciliar la misma velocidad oficial (FR-025)"
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
