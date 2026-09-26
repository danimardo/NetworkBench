//! Cambiar el puerto de control «se aplica al instante» (`Historias.md` §23): el canal se
//! reabre en el puerto nuevo y el anterior queda libre. Si el nuevo está ocupado, falla sin
//! haber cerrado el que funcionaba.
//!
//! Se ejecuta sobre el runtime de Tauri (`async_runtime`), igual que en la aplicación: un
//! `TcpListener` ligado al runtime de un `#[tokio::test]` no atendería desde el de Tauri.

use networkbench_lib::app::start_control_server;
use networkbench_lib::control::engine_port::MotorDeLaboratorio;
use networkbench_lib::control::orquestador::Orquestador;
use networkbench_lib::control::service::{ContextoSesion, SessionService};
use networkbench_lib::history::database::Database;
use networkbench_lib::identity::InstanceIdentity;
use std::net::{Ipv4Addr, SocketAddr, TcpListener as TcpEstandar, TcpStream};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

fn contexto() -> ContextoSesion {
    let identity = Arc::new(InstanceIdentity::generate("Reabrir".into()).unwrap());
    let ruta = std::env::temp_dir().join(format!("nb_reabrir_{}.db", Uuid::new_v4()));
    ContextoSesion {
        orquestador: Arc::new(Orquestador::new(Arc::new(
            MotorDeLaboratorio::con_respuestas(vec![]),
        ))),
        identity,
        database: Arc::new(Database::open(ruta).expect("abrir base de datos")),
        muestreo: None,
        consentimiento: Arc::new(networkbench_lib::control::consent::SolicitudesEntrantes::new()),
        emisor_eventos: None,
        emparejamientos: Arc::new(
            networkbench_lib::control::consent::EmparejamientosEntrantes::new(),
        ),
        aviso: Arc::new(tokio::sync::Notify::new()),
    }
}

/// Un puerto que ahora mismo nadie usa.
fn puerto_libre() -> u16 {
    TcpEstandar::bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn escucha(puerto: u16) -> bool {
    TcpStream::connect_timeout(
        &SocketAddr::from((Ipv4Addr::LOCALHOST, puerto)),
        Duration::from_millis(500),
    )
    .is_ok()
}

#[test]
fn reabrir_en_otro_puerto_libera_el_anterior_y_atiende_en_el_nuevo() {
    tauri::async_runtime::block_on(async {
        let (a, b) = (puerto_libre(), puerto_libre());
        let servicio = Arc::new(SessionService::new());

        let primero = start_control_server(contexto(), Arc::clone(&servicio), a)
            .await
            .expect("abrir en A");
        assert_eq!(primero.puerto, a);
        assert!(escucha(a), "A debería atender");

        // Se abre el nuevo antes de soltar el anterior: durante un instante hay dos.
        let segundo = start_control_server(contexto(), Arc::clone(&servicio), b)
            .await
            .expect("abrir en B");
        assert!(escucha(a) && escucha(b));

        // Soltar el manejador detiene el bucle y cierra los sockets: A queda libre.
        drop(primero);
        let mut libre = false;
        for _ in 0..40 {
            if !escucha(a) {
                libre = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(libre, "el puerto anterior debe quedar libre tras soltarlo");
        assert!(escucha(b), "B sigue atendiendo");
        drop(segundo);
    });
}

#[test]
fn un_puerto_ocupado_falla_sin_cerrar_el_que_ya_atendia() {
    tauri::async_runtime::block_on(async {
        let servicio = Arc::new(SessionService::new());
        let a = puerto_libre();
        let atendiendo = start_control_server(contexto(), Arc::clone(&servicio), a)
            .await
            .expect("abrir en A");

        // Otro proceso ocupa B.
        let ocupado = TcpEstandar::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let b = ocupado.local_addr().unwrap().port();

        let resultado = start_control_server(contexto(), Arc::clone(&servicio), b).await;
        assert!(resultado.is_err(), "B está ocupado: no debe abrirse");
        assert!(escucha(a), "el canal anterior sigue atendiendo");
        drop(atendiendo);
    });
}
