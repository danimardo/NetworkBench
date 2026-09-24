//! Puerto de I/O hacia el motor de medida.
//!
//! El orquestador no conoce NTTTCP: conoce este trait. Así la lógica de sesión se
//! prueba sin binario externo, y el día que el motor cambie —o se valide su salida en
//! laboratorio— solo se toca la implementación, no el coordinador (plan, §Fronteras).
//!
//! `engine/ntttcp` sigue siendo el único módulo que construye argumentos o interpreta
//! XML. Este puerto no los conoce.

use crate::engine::ntttcp::parser::{NtttcpParsedResult, NtttcpRole};
use crate::engine::ntttcp::process::NtttcpProcess;
use crate::model::plan::BenchmarkPlan;
use std::fmt;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

/// Lo que el orquestador pide al motor para una dirección.
pub struct PeticionMedida<'a> {
    pub role: NtttcpRole,
    pub plan: &'a BenchmarkPlan,
    /// Host remoto al que apunta el emisor. El receptor escucha y no lo necesita.
    pub target_host: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotorError {
    /// El ejecutable no está donde se esperaba.
    NoDisponible(String),
    /// El hash no coincide con el registrado. Nunca se ejecuta en este caso (FR-063).
    IntegridadFallida { esperado: String, obtenido: String },
    /// El proceso arrancó y terminó mal, o su salida no se pudo interpretar.
    Ejecucion(String),
    /// Cancelado por el usuario o por el coordinador.
    Cancelado,
}

impl fmt::Display for MotorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDisponible(p) => write!(f, "El motor de medida no está disponible en {p}"),
            Self::IntegridadFallida { esperado, obtenido } => write!(
                f,
                "La integridad del motor no se pudo confirmar (esperado {esperado}, obtenido {obtenido})"
            ),
            Self::Ejecucion(m) => write!(f, "La medición falló: {m}"),
            Self::Cancelado => write!(f, "Medición cancelada"),
        }
    }
}

pub type FuturoMedida<'a> =
    Pin<Box<dyn Future<Output = Result<NtttcpParsedResult, MotorError>> + Send + 'a>>;

/// Motor capaz de ejecutar una dirección de medida.
pub trait MotorDeMedida: Send + Sync {
    fn medir<'a>(&'a self, peticion: PeticionMedida<'a>) -> FuturoMedida<'a>;

    /// Identificación para diagnóstico y para el registro de evidencia. No aparece en
    /// recorridos ordinarios de la interfaz (FR-008).
    fn describir(&self) -> String;
}

/// Motor real: NTTTCP empaquetado, verificado por hash antes de cada ejecución.
pub struct MotorNtttcp {
    exe_path: PathBuf,
    hash_esperado: String,
    temp_dir: PathBuf,
}

impl MotorNtttcp {
    pub fn new(exe_path: PathBuf, hash_esperado: String, temp_dir: PathBuf) -> Self {
        Self {
            exe_path,
            hash_esperado,
            temp_dir,
        }
    }
}

impl MotorDeMedida for MotorNtttcp {
    fn medir<'a>(&'a self, peticion: PeticionMedida<'a>) -> FuturoMedida<'a> {
        Box::pin(async move {
            if !self.exe_path.exists() {
                return Err(MotorError::NoDisponible(
                    self.exe_path.display().to_string(),
                ));
            }

            // FR-063: se comprueba antes de usarlo, en cada ejecución, no una vez al
            // instalar. Un binario sustituido entre dos mediciones no pasa.
            let integro =
                NtttcpProcess::verify_executable_hash(&self.exe_path, &self.hash_esperado)
                    .map_err(|e| MotorError::Ejecucion(e.to_string()))?;
            if !integro {
                return Err(MotorError::IntegridadFallida {
                    esperado: self.hash_esperado.clone(),
                    obtenido: "distinto".to_string(),
                });
            }

            let proceso = NtttcpProcess::spawn(
                &self.exe_path,
                peticion.role,
                peticion.plan,
                peticion.target_host,
                &self.temp_dir,
            )
            .await
            .map_err(|e| MotorError::Ejecucion(e.to_string()))?;

            proceso
                .wait_and_parse()
                .await
                .map_err(|e| MotorError::Ejecucion(e.to_string()))
        })
    }

    fn describir(&self) -> String {
        format!("NTTTCP en {}", self.exe_path.display())
    }
}

/// Motor de laboratorio: devuelve resultados preparados sin lanzar ningún proceso.
///
/// **No mide nada.** Existe para probar la orquestación —estados, ensamblado del
/// resultado, diagnóstico, persistencia y cancelación— sin depender del binario. Nunca
/// se registra en el arranque de la aplicación: `app::init` construye `MotorNtttcp`.
/// Cualquier evidencia obtenida con este motor es simulada y debe declararse como tal.
pub struct MotorDeLaboratorio {
    respuestas: std::sync::Mutex<Vec<Result<NtttcpParsedResult, MotorError>>>,
}

impl MotorDeLaboratorio {
    /// Las respuestas se consumen en orden de petición.
    pub fn con_respuestas(respuestas: Vec<Result<NtttcpParsedResult, MotorError>>) -> Self {
        Self {
            respuestas: std::sync::Mutex::new(respuestas),
        }
    }

    /// Resultado sintético con el caudal indicado.
    pub fn resultado(role: NtttcpRole, throughput_bps: u64, segundos: f64) -> NtttcpParsedResult {
        NtttcpParsedResult {
            role,
            total_bytes: (throughput_bps as f64 / 8.0 * segundos) as u64,
            total_buffers: 1000,
            realtime_seconds: segundos,
            throughput_bps,
            cpu_percent: Some(12.5),
            errors_count: 0,
            packets_sent: None,
            packets_received: None,
            packets_retransmitted: None,
        }
    }
}

impl MotorDeMedida for MotorDeLaboratorio {
    fn medir<'a>(&'a self, _peticion: PeticionMedida<'a>) -> FuturoMedida<'a> {
        Box::pin(async move {
            let mut cola = self
                .respuestas
                .lock()
                .expect("mutex del motor de laboratorio");
            if cola.is_empty() {
                return Err(MotorError::Ejecucion(
                    "El motor de laboratorio se quedó sin respuestas preparadas".to_string(),
                ));
            }
            cola.remove(0)
        })
    }

    fn describir(&self) -> String {
        "motor de laboratorio (no mide)".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_motor_sin_ejecutable_no_mide() {
        let motor = MotorNtttcp::new(
            PathBuf::from("no/existe/ntttcp.exe"),
            "0".repeat(64),
            std::env::temp_dir(),
        );
        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let r = motor
            .medir(PeticionMedida {
                role: NtttcpRole::Receiver,
                plan: &plan,
                target_host: None,
            })
            .await;
        assert!(matches!(r, Err(MotorError::NoDisponible(_))));
    }

    #[tokio::test]
    async fn test_laboratorio_entrega_en_orden_y_se_agota() {
        let motor = MotorDeLaboratorio::con_respuestas(vec![
            Ok(MotorDeLaboratorio::resultado(
                NtttcpRole::Receiver,
                940_000_000,
                10.0,
            )),
            Err(MotorError::Cancelado),
        ]);
        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let p = || PeticionMedida {
            role: NtttcpRole::Receiver,
            plan: &plan,
            target_host: None,
        };

        assert_eq!(motor.medir(p()).await.unwrap().throughput_bps, 940_000_000);
        assert_eq!(motor.medir(p()).await.unwrap_err(), MotorError::Cancelado);
        assert!(matches!(
            motor.medir(p()).await,
            Err(MotorError::Ejecucion(_))
        ));
    }
}
