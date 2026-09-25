//! Orquestación de una sesión de medida.
//!
//! Hasta ahora `control::service` solo movía estados: el motor, el muestreo y el
//! diagnóstico existían con sus pruebas y **nadie los invocaba**. Este módulo es el
//! camino de ejecución que los une.
//!
//! Reparto de responsabilidades, según el plan:
//!
//! - El motor se alcanza por el puerto `MotorDeMedida`, nunca directamente.
//! - `officialBps` sale **siempre del receptor** (FR-027). Las cifras del emisor se
//!   conservan como evidencia, no como veredicto.
//! - Un dato ausente se queda ausente: no se convierte en cero ni en éxito (FR-030).
//! - El diagnóstico se calcula sobre hechos ya recogidos; este módulo no inventa reglas.
//!
//! Lo que **no** hace: hablar con el peer. PREPARE, READY y START son T152 y T153. Aquí
//! se ejecuta la mitad local de una dirección y se ensambla el resultado con lo que el
//! extremo remoto haya aportado.

use crate::control::engine_port::{MotorDeMedida, MotorError, PeticionMedida};
use crate::diagnostic::{
    AsymmetryStats, CapacityReference, DiagnosticEngine, RetransmissionLevel, RetransmissionStats,
    SessionVerdict, StabilityStats, VerdictInput,
};
use crate::engine::ntttcp::parser::{NtttcpParsedResult, NtttcpRole};
use crate::model::plan::{BenchmarkPlan, BenchmarkProtocol};
use crate::model::result::{
    DirectionResult, EngineResult, PeerSnapshot, ResultVersions, SessionResult,
};
use crate::sampling::samples::SampleCollector;
use std::sync::Arc;
use uuid::Uuid;

/// Sentido de una dirección de medida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direccion {
    Forward,
    Reverse,
}

impl Direccion {
    pub fn como_str(&self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::Reverse => "reverse",
        }
    }
}

/// Medidas de los dos extremos de una misma dirección.
///
/// `receptor` es obligatorio: sin él no hay velocidad oficial y la dirección no puede
/// declararse completada (FR-027, FR-034).
pub struct MedidasDireccion {
    pub direccion: Direccion,
    /// De él depende qué se calcula con los contadores de paquetes: retransmisión en
    /// TCP, pérdida en UDP (T184). No son la misma métrica ni la misma fórmula.
    pub protocolo: BenchmarkProtocol,
    pub emisor: Option<NtttcpParsedResult>,
    pub receptor: Option<NtttcpParsedResult>,
    pub muestras: SampleCollector,
}

pub struct Orquestador {
    motor: Arc<dyn MotorDeMedida>,
    diagnostico: DiagnosticEngine,
}

impl Orquestador {
    pub fn new(motor: Arc<dyn MotorDeMedida>) -> Self {
        Self {
            motor,
            diagnostico: DiagnosticEngine::new(),
        }
    }

    pub fn motor(&self) -> &dyn MotorDeMedida {
        self.motor.as_ref()
    }

    /// Ejecuta la mitad local de una dirección.
    ///
    /// El rol lo decide quién mide hacia dónde: en `forward` el equipo local es emisor
    /// y el remoto receptor; en `reverse`, al revés.
    pub async fn ejecutar_mitad_local(
        &self,
        plan: &BenchmarkPlan,
        role: NtttcpRole,
        target_host: Option<&str>,
    ) -> Result<NtttcpParsedResult, MotorError> {
        self.motor
            .medir(PeticionMedida {
                role,
                plan,
                target_host,
            })
            .await
    }

    /// Convierte las medidas de ambos extremos en el resultado de una dirección.
    ///
    /// Una dirección solo se marca `completed` cuando existe medición efectiva de los
    /// **dos** extremos (FR-034). Con uno solo queda `incomplete` y sin velocidad
    /// oficial: presentar la cifra del emisor como resultado sería atribuirle una
    /// autoridad que no tiene.
    pub fn resultado_de_direccion(&self, medidas: &MedidasDireccion) -> DirectionResult {
        let receptor = medidas.receptor.as_ref();
        let emisor = medidas.emisor.as_ref();

        let completada = receptor.is_some() && emisor.is_some();
        let official_bps = receptor.map(|r| r.throughput_bps.to_string());

        let bps: Vec<f64> = medidas
            .muestras
            .samples()
            .iter()
            .filter(|s| !s.gap && s.direction == medidas.direccion.como_str())
            .map(|s| s.bps as f64)
            .collect();
        let huecos = medidas
            .muestras
            .samples()
            .iter()
            .filter(|s| s.gap && s.direction == medidas.direccion.como_str())
            .count();

        let estabilidad: Option<StabilityStats> = if bps.is_empty() {
            None
        } else {
            Some(self.diagnostico.evaluate_stability(&bps, huecos))
        };

        // Solo con la dirección completa: con un extremo ausente, los contadores del otro
        // no bastan para afirmar nada (FR-030, un dato ausente se queda ausente).
        let paquetes: Option<RetransmissionStats> = if completada {
            Some(match medidas.protocolo {
                BenchmarkProtocol::Tcp => self.diagnostico.evaluate_retransmissions(
                    emisor.and_then(|e| e.packets_sent),
                    emisor.and_then(|e| e.packets_retransmitted),
                ),
                BenchmarkProtocol::Udp => self.diagnostico.evaluate_udp_loss(
                    emisor.and_then(|e| e.packets_sent),
                    receptor.and_then(|r| r.packets_received),
                ),
            })
        } else {
            None
        };

        DirectionResult {
            direction: medidas.direccion.como_str().to_string(),
            status: if completada {
                "completed".to_string()
            } else if receptor.is_some() || emisor.is_some() {
                "incomplete".to_string()
            } else {
                "notStarted".to_string()
            },
            sender: emisor.map(|r| a_engine_result(r, "sender")),
            receiver: receptor.map(|r| a_engine_result(r, "receiver")),
            official_bps: if completada { official_bps } else { None },
            utilization: None,
            stability: estabilidad,
            retransmission: paquetes,
            cpu_sender: emisor.and_then(|r| r.cpu_percent),
            cpu_receiver: receptor.and_then(|r| r.cpu_percent),
            samples_count: bps.len(),
            gaps_count: huecos,
        }
    }

    /// Ensambla el resultado de la sesión y su veredicto.
    pub fn ensamblar(&self, entrada: EntradaSesion<'_>) -> SessionResult {
        let EntradaSesion {
            session_id,
            started_at,
            finished_at,
            plan,
            initiator,
            responder,
            direcciones,
            capacidad,
            engine_version,
        } = entrada;

        let completa =
            !direcciones.is_empty() && direcciones.iter().all(|d| d.status == "completed");

        let bps_de = |sentido: &str| -> Option<u64> {
            direcciones
                .iter()
                .find(|d| d.direction == sentido)
                .and_then(|d| d.official_bps.as_ref())
                .and_then(|s| s.parse::<u64>().ok())
        };
        let forward_bps = bps_de("forward");
        let reverse_bps = bps_de("reverse");

        // La asimetría solo tiene sentido con ejecución secuencial y ambos sentidos
        // medidos (FR-031, US5/AC4).
        let asimetria: Option<AsymmetryStats> = if direcciones.len() == 2 {
            self.diagnostico
                .evaluate_asymmetry(forward_bps, reverse_bps)
        } else {
            None
        };

        let estab = |sentido: &str| {
            direcciones
                .iter()
                .find(|d| d.direction == sentido)
                .and_then(|d| d.stability.as_ref())
        };
        let cpu_max = direcciones
            .iter()
            .flat_map(|d| [d.cpu_sender, d.cpu_receiver])
            .flatten()
            .fold(None::<f64>, |acc, v| Some(acc.map_or(v, |a: f64| a.max(v))));

        // El veredicto toma la peor de las direcciones (`Historias.md` §13.5: «retransmisiones
        // (ELEVATED → warn, HIGH → problem)», del emisor de cada dirección). Sin este
        // paso, el dato se calculaba por dirección y el veredicto lo ignoraba (T184).
        let gravedad = |r: &&RetransmissionStats| match r.level {
            RetransmissionLevel::High => 3,
            RetransmissionLevel::Elevated => 2,
            RetransmissionLevel::Normal => 1,
            RetransmissionLevel::NotAvailable => 0,
        };
        let paquetes_peor = direcciones
            .iter()
            .filter_map(|d| d.retransmission.as_ref())
            .max_by_key(gravedad);

        let capacidad_ref = capacidad.unwrap_or(CapacityReference {
            ref_bps: None,
            ref_source: crate::diagnostic::CapacitySource::Unknown,
            cap_a_bps: None,
            cap_b_bps: None,
        });

        let verdict: Option<SessionVerdict> =
            Some(self.diagnostico.generate_verdict(VerdictInput {
                capacity: &capacidad_ref,
                forward_bps,
                reverse_bps,
                forward_stability: estab("forward"),
                reverse_stability: estab("reverse"),
                asymmetry: asimetria.as_ref(),
                retransmissions: paquetes_peor,
                max_cpu_percent: cpu_max,
                is_completed: completa,
            }));

        SessionResult {
            schema_version: 1,
            session_id,
            started_at: started_at.to_string(),
            finished_at: finished_at.to_string(),
            status: if completa {
                "completed".to_string()
            } else {
                "incomplete".to_string()
            },
            initiator,
            responder,
            plan: plan.clone(),
            capacity: Some(capacidad_ref),
            directions: direcciones,
            asymmetry: asimetria,
            verdict,
            result_source: "local".to_string(),
            versions: ResultVersions {
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                protocol_version: crate::control::server::VERSION_PROTOCOLO,
                engine_version,
                schema_version: 1,
                thresholds_hash: crate::diagnostic::THRESHOLDS_HASH.to_string(),
            },
        }
    }
}

/// Datos de entrada del ensamblado. Agrupados con nombres para que ninguna cadena o
/// snapshot se pueda intercambiar por error.
pub struct EntradaSesion<'a> {
    pub session_id: Uuid,
    pub started_at: &'a str,
    pub finished_at: &'a str,
    pub plan: &'a BenchmarkPlan,
    pub initiator: PeerSnapshot,
    pub responder: PeerSnapshot,
    pub direcciones: Vec<DirectionResult>,
    pub capacidad: Option<CapacityReference>,
    pub engine_version: String,
}

fn a_engine_result(r: &NtttcpParsedResult, role: &str) -> EngineResult {
    EngineResult {
        role: role.to_string(),
        // Enteros que pueden superar el rango seguro de JavaScript: cruzan como cadena
        // decimal, según el plan §Datos.
        total_bytes: r.total_bytes.to_string(),
        realtime_seconds: r.realtime_seconds,
        throughput_bps: r.throughput_bps.to_string(),
        cpu_percent: r.cpu_percent,
        buffers_count: Some(r.total_buffers),
        errors_count: r.errors_count,
        packets_sent: r.packets_sent,
        packets_received: r.packets_received,
        packets_retransmitted: r.packets_retransmitted,
        raw: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::engine_port::MotorDeLaboratorio;

    fn snapshot(nombre: &str) -> PeerSnapshot {
        PeerSnapshot {
            instance_id: Uuid::new_v4(),
            display_name: nombre.to_string(),
            fingerprint: "a".repeat(64),
            address: "192.168.1.10:7411".to_string(),
        }
    }

    fn orquestador_con(respuestas: Vec<Result<NtttcpParsedResult, MotorError>>) -> Orquestador {
        Orquestador::new(Arc::new(MotorDeLaboratorio::con_respuestas(respuestas)))
    }

    fn medidas(
        direccion: Direccion,
        emisor: Option<NtttcpParsedResult>,
        receptor: Option<NtttcpParsedResult>,
    ) -> MedidasDireccion {
        MedidasDireccion {
            direccion,
            protocolo: BenchmarkProtocol::Tcp,
            emisor,
            receptor,
            muestras: SampleCollector::new(),
        }
    }

    #[tokio::test]
    async fn test_el_orquestador_invoca_el_motor() {
        let o = orquestador_con(vec![Ok(MotorDeLaboratorio::resultado(
            NtttcpRole::Receiver,
            940_000_000,
            10.0,
        ))]);
        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let r = o
            .ejecutar_mitad_local(&plan, NtttcpRole::Receiver, None)
            .await
            .expect("medida");
        assert_eq!(r.throughput_bps, 940_000_000);
    }

    #[test]
    fn test_la_velocidad_oficial_es_la_del_receptor() {
        let o = orquestador_con(vec![]);
        // El emisor declara más de lo que el receptor recibió: gana el receptor.
        let d = o.resultado_de_direccion(&medidas(
            Direccion::Forward,
            Some(MotorDeLaboratorio::resultado(
                NtttcpRole::Sender,
                990_000_000,
                10.0,
            )),
            Some(MotorDeLaboratorio::resultado(
                NtttcpRole::Receiver,
                940_000_000,
                10.0,
            )),
        ));
        assert_eq!(d.status, "completed");
        assert_eq!(d.official_bps.as_deref(), Some("940000000"));
    }

    #[test]
    fn test_sin_receptor_no_hay_velocidad_oficial() {
        let o = orquestador_con(vec![]);
        let d = o.resultado_de_direccion(&medidas(
            Direccion::Forward,
            Some(MotorDeLaboratorio::resultado(
                NtttcpRole::Sender,
                990_000_000,
                10.0,
            )),
            None,
        ));
        assert_eq!(d.status, "incomplete");
        // FR-030: ausente se queda ausente, no pasa a cero.
        assert_eq!(d.official_bps, None);
    }

    #[test]
    fn test_direccion_sin_datos_es_no_iniciada() {
        let o = orquestador_con(vec![]);
        let d = o.resultado_de_direccion(&medidas(Direccion::Reverse, None, None));
        assert_eq!(d.status, "notStarted");
        assert_eq!(d.official_bps, None);
        assert_eq!(d.samples_count, 0);
    }

    #[test]
    fn test_sesion_incompleta_no_se_declara_completada() {
        let o = orquestador_con(vec![]);
        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let completa = o.resultado_de_direccion(&medidas(
            Direccion::Forward,
            Some(MotorDeLaboratorio::resultado(
                NtttcpRole::Sender,
                950_000_000,
                10.0,
            )),
            Some(MotorDeLaboratorio::resultado(
                NtttcpRole::Receiver,
                940_000_000,
                10.0,
            )),
        ));
        let coja = o.resultado_de_direccion(&medidas(
            Direccion::Reverse,
            Some(MotorDeLaboratorio::resultado(
                NtttcpRole::Sender,
                900_000_000,
                10.0,
            )),
            None,
        ));

        let r = o.ensamblar(EntradaSesion {
            session_id: Uuid::new_v4(),
            started_at: "2026-09-23T10:00:00.000Z",
            finished_at: "2026-09-23T10:01:00.000Z",
            plan: &plan,
            initiator: snapshot("A"),
            responder: snapshot("B"),
            direcciones: vec![completa, coja],
            capacidad: None,
            engine_version: "5.40".into(),
        });

        assert_eq!(r.status, "incomplete");
        // Sin capacidad de referencia no puede haber veredicto de rendimiento (FR-030).
        assert_eq!(r.verdict.as_ref().unwrap().performance_level, None);
        assert_eq!(
            r.versions.thresholds_hash,
            crate::diagnostic::THRESHOLDS_HASH
        );
    }

    #[test]
    fn test_una_sola_direccion_no_produce_veredicto_de_asimetria() {
        let o = orquestador_con(vec![]);
        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let unica = o.resultado_de_direccion(&medidas(
            Direccion::Forward,
            Some(MotorDeLaboratorio::resultado(
                NtttcpRole::Sender,
                950_000_000,
                10.0,
            )),
            Some(MotorDeLaboratorio::resultado(
                NtttcpRole::Receiver,
                940_000_000,
                10.0,
            )),
        ));

        let r = o.ensamblar(EntradaSesion {
            session_id: Uuid::new_v4(),
            started_at: "2026-09-23T10:00:00.000Z",
            finished_at: "2026-09-23T10:00:30.000Z",
            plan: &plan,
            initiator: snapshot("A"),
            responder: snapshot("B"),
            direcciones: vec![unica],
            capacidad: None,
            engine_version: "5.40".into(),
        });

        assert_eq!(r.status, "completed");
        assert!(
            r.asymmetry.is_none(),
            "una dirección no permite juzgar asimetría"
        );
    }

    fn con_paquetes(
        mut r: NtttcpParsedResult,
        enviados: Option<u64>,
        recibidos: Option<u64>,
        retransmitidos: Option<u64>,
    ) -> NtttcpParsedResult {
        r.packets_sent = enviados;
        r.packets_received = recibidos;
        r.packets_retransmitted = retransmitidos;
        r
    }

    fn medidas_de(
        protocolo: BenchmarkProtocol,
        emisor: NtttcpParsedResult,
        receptor: NtttcpParsedResult,
    ) -> MedidasDireccion {
        MedidasDireccion {
            protocolo,
            ..medidas(Direccion::Forward, Some(emisor), Some(receptor))
        }
    }

    /// T184: en TCP se calcula la retransmisión con los contadores del emisor.
    #[test]
    fn test_tcp_calcula_retransmision_con_los_contadores_del_emisor() {
        let o = orquestador_con(vec![]);
        let emisor = con_paquetes(
            MotorDeLaboratorio::resultado(NtttcpRole::Sender, 940_000_000, 10.0),
            Some(10_000),
            None,
            Some(150),
        );
        let receptor = MotorDeLaboratorio::resultado(NtttcpRole::Receiver, 938_000_000, 10.0);

        let d = o.resultado_de_direccion(&medidas_de(BenchmarkProtocol::Tcp, emisor, receptor));
        let r = d.retransmission.expect("dirección completa con contadores");
        assert_eq!(r.level, RetransmissionLevel::High);
        assert_eq!(r.packets_retransmitted, Some(150));
    }

    /// T184: en UDP se calcula la pérdida, con lo enviado por el emisor y lo recibido
    /// por el receptor; los contadores de retransmisión no cuentan.
    #[test]
    fn test_udp_calcula_perdida_con_enviados_y_recibidos() {
        let o = orquestador_con(vec![]);
        let emisor = con_paquetes(
            MotorDeLaboratorio::resultado(NtttcpRole::Sender, 100_000_000, 10.0),
            Some(10_000),
            None,
            Some(9_999), // un valor absurdo a propósito: en UDP no debe usarse
        );
        let receptor = con_paquetes(
            MotorDeLaboratorio::resultado(NtttcpRole::Receiver, 98_500_000, 10.0),
            None,
            Some(9_850),
            None,
        );

        let d = o.resultado_de_direccion(&medidas_de(BenchmarkProtocol::Udp, emisor, receptor));
        let r = d.retransmission.expect("dirección completa con contadores");
        assert_eq!(r.level, RetransmissionLevel::High);
        assert_eq!(
            r.packets_retransmitted,
            Some(150),
            "perdidos = 10000 - 9850"
        );
        assert!((r.ratio.unwrap() - 0.015).abs() < 1e-9);
    }

    /// Sin contadores no se inventa nada, y con la dirección incompleta tampoco se
    /// calcula (FR-030).
    #[test]
    fn test_sin_contadores_o_sin_direccion_completa_no_hay_estadistica_inventada() {
        let o = orquestador_con(vec![]);

        let sin_contadores = o.resultado_de_direccion(&medidas_de(
            BenchmarkProtocol::Udp,
            MotorDeLaboratorio::resultado(NtttcpRole::Sender, 1, 10.0),
            MotorDeLaboratorio::resultado(NtttcpRole::Receiver, 1, 10.0),
        ));
        assert_eq!(
            sin_contadores.retransmission.unwrap().level,
            RetransmissionLevel::NotAvailable
        );

        let mut incompleta = medidas(Direccion::Forward, None, None);
        incompleta.emisor = Some(con_paquetes(
            MotorDeLaboratorio::resultado(NtttcpRole::Sender, 1, 10.0),
            Some(100),
            None,
            Some(50),
        ));
        assert!(
            o.resultado_de_direccion(&incompleta)
                .retransmission
                .is_none()
        );
    }

    /// T184: el veredicto ya no ignora la retransmisión: una dirección con retransmisión
    /// alta lo lleva a `problem`, aunque la otra vaya limpia.
    #[test]
    fn test_el_veredicto_toma_la_peor_retransmision_de_las_direcciones() {
        let o = orquestador_con(vec![]);
        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let limpia = o.resultado_de_direccion(&medidas_de(
            BenchmarkProtocol::Tcp,
            con_paquetes(
                MotorDeLaboratorio::resultado(NtttcpRole::Sender, 940_000_000, 10.0),
                Some(10_000),
                None,
                Some(0),
            ),
            MotorDeLaboratorio::resultado(NtttcpRole::Receiver, 940_000_000, 10.0),
        ));
        let mut mala = o.resultado_de_direccion(&medidas_de(
            BenchmarkProtocol::Tcp,
            con_paquetes(
                MotorDeLaboratorio::resultado(NtttcpRole::Sender, 940_000_000, 10.0),
                Some(10_000),
                None,
                Some(500),
            ),
            MotorDeLaboratorio::resultado(NtttcpRole::Receiver, 940_000_000, 10.0),
        ));
        mala.direction = "reverse".to_string();

        let r = o.ensamblar(EntradaSesion {
            session_id: Uuid::new_v4(),
            started_at: "2026-09-25T10:00:00.000Z",
            finished_at: "2026-09-25T10:00:30.000Z",
            plan: &plan,
            initiator: snapshot("A"),
            responder: snapshot("B"),
            direcciones: vec![limpia, mala],
            capacidad: None,
            engine_version: "5.40".into(),
        });
        let veredicto = r.verdict.expect("veredicto");
        assert_eq!(veredicto.retransmission_level, RetransmissionLevel::High);
    }

    #[tokio::test]
    async fn test_un_motor_alterado_no_produce_medida() {
        let o = orquestador_con(vec![Err(MotorError::IntegridadFallida {
            esperado: "a".repeat(64),
            obtenido: "distinto".into(),
        })]);
        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let r = o
            .ejecutar_mitad_local(&plan, NtttcpRole::Receiver, None)
            .await;
        assert!(matches!(r, Err(MotorError::IntegridadFallida { .. })));
    }
}
