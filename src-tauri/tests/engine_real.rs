//! Medición real con el motor empaquetado (puerta G1).
//!
//! Estas pruebas **lanzan NTTTCP de verdad** contra la interfaz de bucle local. Van
//! marcadas `#[ignore]` porque abren sockets y tardan segundos: no deben correr en cada
//! edición. Se ejecutan a propósito:
//!
//! ```text
//! cargo test --manifest-path src-tauri/Cargo.toml --test engine_real -- --ignored --test-threads=1
//! ```
//!
//! Miden sobre `127.0.0.1`, así que **no validan una red real**: validan que el
//! ejecutable arranca, que los argumentos que construimos le sirven y que su XML se
//! interpreta. La medición entre dos equipos sigue siendo trabajo de laboratorio.

use networkbench_lib::control::engine_port::{
    MotorDeMedida, MotorError, MotorNtttcp, PeticionMedida,
};
use networkbench_lib::engine::ntttcp::engine_sha256;
use networkbench_lib::engine::ntttcp::parser::NtttcpRole;
use networkbench_lib::model::plan::BenchmarkPlan;
use std::path::PathBuf;
use std::sync::Arc;

fn ruta_motor() -> PathBuf {
    // Las pruebas corren con el directorio del crate (`src-tauri`) como raíz.
    PathBuf::from("../engine/ntttcp.exe")
}

fn motor() -> MotorNtttcp {
    MotorNtttcp::new(
        ruta_motor(),
        engine_sha256().to_string(),
        std::env::temp_dir(),
    )
}

fn plan_corto(puerto: u16) -> BenchmarkPlan {
    let mut plan = BenchmarkPlan::new_standard_tcp(puerto);
    plan.measure_seconds = 5;
    plan.warmup_seconds = 0;
    plan.cooldown_seconds = 0;
    plan.streams = 1;
    plan
}

#[tokio::test]
#[ignore = "lanza NTTTCP y abre sockets; ejecutar con --ignored"]
async fn g1_medicion_real_en_bucle_local() {
    let motor = Arc::new(motor());
    let plan = plan_corto(5100);

    let receptor = {
        let motor = Arc::clone(&motor);
        let plan = plan.clone();
        tokio::spawn(async move {
            motor
                .medir(PeticionMedida {
                    role: NtttcpRole::Receiver,
                    plan: &plan,
                    target_host: Some("127.0.0.1"),
                })
                .await
        })
    };

    // El receptor necesita estar escuchando antes de que el emisor conecte.
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    let emisor = motor
        .medir(PeticionMedida {
            role: NtttcpRole::Sender,
            plan: &plan,
            target_host: Some("127.0.0.1"),
        })
        .await
        .expect("el emisor debe completar");

    let receptor = receptor.await.unwrap().expect("el receptor debe completar");

    assert_eq!(receptor.role, NtttcpRole::Receiver);
    assert_eq!(emisor.role, NtttcpRole::Sender);

    // Sobre loopback el caudal es alto, pero lo que se comprueba es que hay cifras
    // coherentes, no un valor concreto: eso dependería de la máquina.
    assert!(receptor.throughput_bps > 0, "el receptor no midió caudal");
    assert!(receptor.total_bytes > 0, "el receptor no recibió bytes");
    assert!(
        receptor.realtime_seconds >= 4.0,
        "duración inverosímil: {}",
        receptor.realtime_seconds
    );
    assert!(receptor.cpu_percent.is_some(), "falta el dato de CPU");
}

#[tokio::test]
#[ignore = "lanza NTTTCP; ejecutar con --ignored"]
async fn g1_un_motor_con_hash_distinto_no_se_ejecuta() {
    // FR-063: el hash se comprueba antes de cada ejecución. Con un valor esperado que
    // no corresponde, el binario auténtico tampoco debe lanzarse.
    let motor = MotorNtttcp::new(ruta_motor(), "0".repeat(64), std::env::temp_dir());
    let plan = plan_corto(5200);

    let r = motor
        .medir(PeticionMedida {
            role: NtttcpRole::Receiver,
            plan: &plan,
            target_host: Some("127.0.0.1"),
        })
        .await;

    assert!(
        matches!(r, Err(MotorError::IntegridadFallida { .. })),
        "un motor cuyo hash no coincide no debe ejecutarse, y devolvió {r:?}"
    );
}
