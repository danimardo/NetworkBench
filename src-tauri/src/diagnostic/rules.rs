use super::capacity::CapacityReference;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VerdictLevel {
    Ok,
    Warn,
    Problem,
    NotEvaluable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StabilityLevel {
    VeryStable,
    Stable,
    Variable,
    VeryVariable,
    NotEvaluable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RetransmissionLevel {
    Normal,
    Elevated,
    High,
    NotAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CpuLevel {
    Low,
    Moderate,
    High,
    NotAvailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StabilityStats {
    pub level: StabilityLevel,
    pub mean_bps: f64,
    pub std_dev_bps: f64,
    pub cv: f64,
    pub min_bps: f64,
    pub max_bps: f64,
    pub p5_bps: f64,
    pub p50_bps: f64,
    pub p95_bps: f64,
    pub drops_count: usize,
    pub samples_count: usize,
    pub gaps_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetransmissionStats {
    pub level: RetransmissionLevel,
    pub ratio: Option<f64>,
    pub packets_sent: Option<u64>,
    pub packets_retransmitted: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsymmetryStats {
    pub ratio: f64,
    pub is_asymmetric: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticItem {
    pub rule_id: String,
    pub message_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_params: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionVerdict {
    pub rules_version: String,
    pub level: VerdictLevel,
    pub title_key: String,
    pub performance_level: Option<VerdictLevel>,
    pub stability_level: StabilityLevel,
    pub asymmetry_level: Option<VerdictLevel>,
    pub retransmission_level: RetransmissionLevel,
    pub cpu_level: CpuLevel,
    pub facts: Vec<DiagnosticItem>,
    pub observations: Vec<DiagnosticItem>,
    pub possible_causes: Vec<DiagnosticItem>,
    pub actions: Vec<DiagnosticItem>,
}

pub struct DiagnosticEngine {
    pub rules_version: String,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        Self {
            rules_version: "1.0.0".to_string(),
        }
    }

    /// Evalúa la estabilidad de una serie de muestras (en bps).
    pub fn evaluate_stability(&self, samples: &[f64], gaps_count: usize) -> StabilityStats {
        if samples.len() < 10 {
            return StabilityStats {
                level: StabilityLevel::NotEvaluable,
                mean_bps: 0.0,
                std_dev_bps: 0.0,
                cv: 0.0,
                min_bps: 0.0,
                max_bps: 0.0,
                p5_bps: 0.0,
                p50_bps: 0.0,
                p95_bps: 0.0,
                drops_count: 0,
                samples_count: samples.len(),
                gaps_count,
            };
        }

        let count = samples.len() as f64;
        let sum: f64 = samples.iter().sum();
        let mean = sum / count;

        let variance: f64 = samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (count - 1.0);
        let std_dev = variance.sqrt();
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

        let drop_threshold = 0.50 * mean;
        let drops = samples.iter().filter(|&&x| x < drop_threshold).count();

        // Percentiles
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let min = *sorted.first().unwrap_or(&0.0);
        let max = *sorted.last().unwrap_or(&0.0);
        let p5 = sorted[((count * 0.05).floor() as usize).min(sorted.len() - 1)];
        let p50 = sorted[((count * 0.50).floor() as usize).min(sorted.len() - 1)];
        let p95 = sorted[((count * 0.95).floor() as usize).min(sorted.len() - 1)];

        let mut base_level = if cv < 0.05 {
            StabilityLevel::VeryStable
        } else if cv < 0.10 {
            StabilityLevel::Stable
        } else if cv < 0.20 {
            StabilityLevel::Variable
        } else {
            StabilityLevel::VeryVariable
        };

        if drops >= 2 {
            base_level = match base_level {
                StabilityLevel::VeryStable => StabilityLevel::Stable,
                StabilityLevel::Stable => StabilityLevel::Variable,
                StabilityLevel::Variable | StabilityLevel::VeryVariable => StabilityLevel::VeryVariable,
                StabilityLevel::NotEvaluable => StabilityLevel::NotEvaluable,
            };
        }

        StabilityStats {
            level: base_level,
            mean_bps: mean,
            std_dev_bps: std_dev,
            cv,
            min_bps: min,
            max_bps: max,
            p5_bps: p5,
            p50_bps: p50,
            p95_bps: p95,
            drops_count: drops,
            samples_count: samples.len(),
            gaps_count,
        }
    }

    /// Evalúa la asimetría entre dos velocidades secuenciales.
    pub fn evaluate_asymmetry(&self, bps_a_to_b: Option<u64>, bps_b_to_a: Option<u64>) -> Option<AsymmetryStats> {
        match (bps_a_to_b, bps_b_to_a) {
            (Some(v1), Some(v2)) if v1 > 0 && v2 > 0 => {
                let max_v = v1.max(v2) as f64;
                let min_v = v1.min(v2) as f64;
                let ratio = (max_v - min_v) / max_v;
                let is_asymmetric = ratio > 0.20;
                Some(AsymmetryStats {
                    ratio,
                    is_asymmetric,
                })
            }
            _ => None,
        }
    }

    /// Evalúa la tasa de retransmisiones TCP.
    pub fn evaluate_retransmissions(&self, sent: Option<u64>, retrans: Option<u64>) -> RetransmissionStats {
        match (sent, retrans) {
            (Some(s), Some(r)) if s > 0 => {
                let ratio = r as f64 / s as f64;
                let level = if ratio < 0.001 {
                    RetransmissionLevel::Normal
                } else if ratio <= 0.01 {
                    RetransmissionLevel::Elevated
                } else {
                    RetransmissionLevel::High
                };
                RetransmissionStats {
                    level,
                    ratio: Some(ratio),
                    packets_sent: Some(s),
                    packets_retransmitted: Some(r),
                }
            }
            _ => RetransmissionStats {
                level: RetransmissionLevel::NotAvailable,
                ratio: None,
                packets_sent: sent,
                packets_retransmitted: retrans,
            },
        }
    }

    /// Evalúa el uso de CPU.
    pub fn evaluate_cpu(&self, cpu_avg: Option<f64>) -> CpuLevel {
        match cpu_avg {
            Some(cpu) if cpu < 25.0 => CpuLevel::Low,
            Some(cpu) if cpu <= 60.0 => CpuLevel::Moderate,
            Some(_) => CpuLevel::High,
            None => CpuLevel::NotAvailable,
        }
    }

    /// Genera el veredicto global de la sesión.
    pub fn generate_verdict(
        &self,
        capacity: &CapacityReference,
        forward_bps: Option<u64>,
        reverse_bps: Option<u64>,
        forward_stability: Option<&StabilityStats>,
        reverse_stability: Option<&StabilityStats>,
        asymmetry: Option<&AsymmetryStats>,
        retransmissions: Option<&RetransmissionStats>,
        max_cpu_percent: Option<f64>,
        is_completed: bool,
    ) -> SessionVerdict {
        if !is_completed {
            return SessionVerdict {
                rules_version: self.rules_version.clone(),
                level: VerdictLevel::NotEvaluable,
                title_key: "verdict.incomplete".to_string(),
                performance_level: None,
                stability_level: StabilityLevel::NotEvaluable,
                asymmetry_level: None,
                retransmission_level: RetransmissionLevel::NotAvailable,
                cpu_level: CpuLevel::NotAvailable,
                facts: vec![],
                observations: vec![],
                possible_causes: vec![],
                actions: vec![DiagnosticItem {
                    rule_id: "SESSION_REPEAT".into(),
                    message_key: "actions.repeat_test".into(),
                    safe_params: None,
                    evidence_refs: vec![],
                }],
            };
        }

        let mut facts = Vec::new();
        let mut observations = Vec::new();
        let mut causes = Vec::new();
        let mut actions = Vec::new();

        // 1. Rendimiento
        let mut performance_level = None;
        let min_speed = match (forward_bps, reverse_bps) {
            (Some(f), Some(r)) => Some(f.min(r)),
            (Some(f), None) => Some(f),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        };

        if let Some(spd) = min_speed {
            facts.push(DiagnosticItem {
                rule_id: "FACT_SPEED".into(),
                message_key: "facts.speed_measured".into(),
                safe_params: Some(serde_json::json!({ "speedBps": spd.to_string() })),
                evidence_refs: vec!["officialBps".into()],
            });
        }

        if let (Some(ref_bps), Some(spd)) = (capacity.ref_bps, min_speed) {
            let util = spd as f64 / ref_bps as f64;
            facts.push(DiagnosticItem {
                rule_id: "FACT_CAPACITY".into(),
                message_key: "facts.link_capacity".into(),
                safe_params: Some(serde_json::json!({
                    "capacityBps": ref_bps.to_string(),
                    "utilizationPercent": (util * 100.0).round() as u64
                })),
                evidence_refs: vec!["refBps".into()],
            });

            if util >= 0.85 {
                performance_level = Some(VerdictLevel::Ok);
            } else if util >= 0.60 {
                performance_level = Some(VerdictLevel::Warn);
                observations.push(DiagnosticItem {
                    rule_id: "OBS_PERF_WARN".into(),
                    message_key: "observations.perf_warn".into(),
                    safe_params: Some(serde_json::json!({ "utilization": (util * 100.0).round() as u64 })),
                    evidence_refs: vec![],
                });
                causes.push(DiagnosticItem {
                    rule_id: "CAUSE_CABLE_SWITCH".into(),
                    message_key: "causes.cable_or_switch".into(),
                    safe_params: None,
                    evidence_refs: vec![],
                });
                actions.push(DiagnosticItem {
                    rule_id: "ACT_CHECK_CABLE".into(),
                    message_key: "actions.check_cable".into(),
                    safe_params: None,
                    evidence_refs: vec![],
                });
            } else {
                performance_level = Some(VerdictLevel::Problem);
                observations.push(DiagnosticItem {
                    rule_id: "OBS_PERF_PROBLEM".into(),
                    message_key: "observations.perf_problem".into(),
                    safe_params: Some(serde_json::json!({ "utilization": (util * 100.0).round() as u64 })),
                    evidence_refs: vec![],
                });
                causes.push(DiagnosticItem {
                    rule_id: "CAUSE_CABLE_SWITCH".into(),
                    message_key: "causes.cable_or_switch".into(),
                    safe_params: None,
                    evidence_refs: vec![],
                });
                actions.push(DiagnosticItem {
                    rule_id: "ACT_CHECK_HARDWARE".into(),
                    message_key: "actions.check_hardware".into(),
                    safe_params: None,
                    evidence_refs: vec![],
                });
            }
        }

        // 2. Estabilidad
        let stability_level = match (forward_stability, reverse_stability) {
            (Some(s1), Some(s2)) => {
                let worst = match (s1.level, s2.level) {
                    (StabilityLevel::VeryVariable, _) | (_, StabilityLevel::VeryVariable) => StabilityLevel::VeryVariable,
                    (StabilityLevel::Variable, _) | (_, StabilityLevel::Variable) => StabilityLevel::Variable,
                    (StabilityLevel::Stable, _) | (_, StabilityLevel::Stable) => StabilityLevel::Stable,
                    (StabilityLevel::VeryStable, StabilityLevel::VeryStable) => StabilityLevel::VeryStable,
                    (StabilityLevel::NotEvaluable, other) | (other, StabilityLevel::NotEvaluable) => other,
                };
                worst
            }
            (Some(s), None) | (None, Some(s)) => s.level,
            (None, None) => StabilityLevel::NotEvaluable,
        };

        if stability_level == StabilityLevel::Variable {
            observations.push(DiagnosticItem {
                rule_id: "OBS_STABILITY_VAR".into(),
                message_key: "observations.stability_variable".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
            causes.push(DiagnosticItem {
                rule_id: "CAUSE_BACKGROUND_TRAFFIC".into(),
                message_key: "causes.background_traffic".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
        } else if stability_level == StabilityLevel::VeryVariable {
            observations.push(DiagnosticItem {
                rule_id: "OBS_STABILITY_VERY_VAR".into(),
                message_key: "observations.stability_very_variable".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
            causes.push(DiagnosticItem {
                rule_id: "CAUSE_INTERFERENCE_OR_BUFFER".into(),
                message_key: "causes.interference_or_buffer".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
            actions.push(DiagnosticItem {
                rule_id: "ACT_REPEAT_ISOLATED".into(),
                message_key: "actions.repeat_isolated".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
        }

        // 3. Asimetría
        let mut asymmetry_level = None;
        if let Some(asym) = asymmetry {
            if asym.is_asymmetric {
                asymmetry_level = Some(VerdictLevel::Warn);
                observations.push(DiagnosticItem {
                    rule_id: "OBS_ASYMMETRY".into(),
                    message_key: "observations.asymmetry".into(),
                    safe_params: Some(serde_json::json!({
                        "asymmetryPercent": (asym.ratio * 100.0).round() as u64
                    })),
                    evidence_refs: vec![],
                });
                causes.push(DiagnosticItem {
                    rule_id: "CAUSE_ASYMMETRIC_ROUTE".into(),
                    message_key: "causes.asymmetric_route_or_firewall".into(),
                    safe_params: None,
                    evidence_refs: vec![],
                });
            } else {
                asymmetry_level = Some(VerdictLevel::Ok);
            }
        }

        // 4. Retransmisiones
        let retransmission_level = retransmissions.map(|r| r.level).unwrap_or(RetransmissionLevel::NotAvailable);
        if retransmission_level == RetransmissionLevel::Elevated {
            observations.push(DiagnosticItem {
                rule_id: "OBS_RETRANS_ELEVATED".into(),
                message_key: "observations.retransmissions_elevated".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
            causes.push(DiagnosticItem {
                rule_id: "CAUSE_PACKET_LOSS".into(),
                message_key: "causes.packet_loss".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
        } else if retransmission_level == RetransmissionLevel::High {
            observations.push(DiagnosticItem {
                rule_id: "OBS_RETRANS_HIGH".into(),
                message_key: "observations.retransmissions_high".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
            causes.push(DiagnosticItem {
                rule_id: "CAUSE_HIGH_PACKET_LOSS".into(),
                message_key: "causes.high_packet_loss".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
            actions.push(DiagnosticItem {
                rule_id: "ACT_CHECK_DUPLEX".into(),
                message_key: "actions.check_duplex_and_mtu".into(),
                safe_params: None,
                evidence_refs: vec![],
            });
        }

        // 5. CPU
        let cpu_level = self.evaluate_cpu(max_cpu_percent);
        if cpu_level == CpuLevel::High {
            let util_under_85 = performance_level.map(|p| p != VerdictLevel::Ok).unwrap_or(true);
            if util_under_85 {
                causes.push(DiagnosticItem {
                    rule_id: "CAUSE_CPU_HIGH".into(),
                    message_key: "causes.cpu_high".into(),
                    safe_params: None,
                    evidence_refs: vec![],
                });
            }
        }

        // Nivel global: peor nivel
        let mut overall = VerdictLevel::Ok;

        if performance_level == Some(VerdictLevel::Problem)
            || stability_level == StabilityLevel::VeryVariable
            || retransmission_level == RetransmissionLevel::High
        {
            overall = VerdictLevel::Problem;
        } else if performance_level == Some(VerdictLevel::Warn)
            || stability_level == StabilityLevel::Variable
            || retransmission_level == RetransmissionLevel::Elevated
            || asymmetry_level == Some(VerdictLevel::Warn)
        {
            overall = VerdictLevel::Warn;
        }

        let title_key = match overall {
            VerdictLevel::Ok => "verdict.ok_title".to_string(),
            VerdictLevel::Warn => "verdict.warn_title".to_string(),
            VerdictLevel::Problem => "verdict.problem_title".to_string(),
            VerdictLevel::NotEvaluable => "verdict.not_evaluable_title".to_string(),
        };

        SessionVerdict {
            rules_version: self.rules_version.clone(),
            level: overall,
            title_key,
            performance_level,
            stability_level,
            asymmetry_level,
            retransmission_level,
            cpu_level,
            facts,
            observations,
            possible_causes: causes,
            actions,
        }
    }
}
