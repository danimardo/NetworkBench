use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UdpLossLevel {
    Low,
    Moderate,
    High,
    NotEvaluable,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UdpThresholds {
    pub low_loss_max_ratio: f64,
    pub moderate_loss_max_ratio: f64,
}

impl Default for UdpThresholds {
    fn default() -> Self {
        Self {
            low_loss_max_ratio: 0.001,  // 0.1 %
            moderate_loss_max_ratio: 0.01, // 1.0 %
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UdpDiagnosticResult {
    pub target_rate_bps: Option<u64>,
    pub emitted_rate_bps: u64,
    pub received_rate_bps: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub loss_ratio: f64,
    pub loss_percent: f64,
    pub loss_level: UdpLossLevel,
    pub title_key: String,
    pub observations: Vec<String>,
    pub is_target_exceeded: bool,
}

/// Evalúa el diagnóstico de una prueba UDP según Historias.md §18:
/// - Métrica principal: pérdida = 1 - packets_received / packets_sent.
/// - Umbrales [PROV]:
///   - pérdida < 0,1 %: «sin pérdidas apreciables» (Low / Ok)
///   - 0,1 % – 1 %: «pérdidas ligeras» (Moderate / Warn)
///   - > 1 %: «pérdidas significativas» (High / Problem)
/// - Si la tasa real emitida superó la capacidad de referencia, se añade observación.
pub fn evaluate_udp_diagnostics(
    packets_sent: u64,
    packets_received: u64,
    emitted_rate_bps: u64,
    received_rate_bps: u64,
    target_rate_bps: Option<u64>,
    capacity_ref_bps: Option<u64>,
    custom_thresholds: Option<&UdpThresholds>,
) -> UdpDiagnosticResult {
    let default_thresholds = UdpThresholds::default();
    let thresholds = custom_thresholds.unwrap_or(&default_thresholds);

    if packets_sent == 0 {
        return UdpDiagnosticResult {
            target_rate_bps,
            emitted_rate_bps,
            received_rate_bps,
            packets_sent: 0,
            packets_received: 0,
            packets_lost: 0,
            loss_ratio: 0.0,
            loss_percent: 0.0,
            loss_level: UdpLossLevel::NotEvaluable,
            title_key: "diagnostics.udp.not_evaluable".to_string(),
            observations: vec!["diagnostics.udp.no_packets_sent".to_string()],
            is_target_exceeded: false,
        };
    }

    let packets_lost = packets_sent.saturating_sub(packets_received);
    let loss_ratio = (packets_lost as f64) / (packets_sent as f64);
    let loss_percent = loss_ratio * 100.0;

    let (loss_level, title_key) = if loss_ratio < thresholds.low_loss_max_ratio {
        (UdpLossLevel::Low, "diagnostics.udp.loss_low".to_string())
    } else if loss_ratio <= thresholds.moderate_loss_max_ratio {
        (UdpLossLevel::Moderate, "diagnostics.udp.loss_moderate".to_string())
    } else {
        (UdpLossLevel::High, "diagnostics.udp.loss_high".to_string())
    };

    let mut observations = Vec::new();
    let mut is_target_exceeded = false;

    // Si la tasa real emitida superó ref_bps, añadir observación (§18)
    if let Some(ref_bps) = capacity_ref_bps {
        if emitted_rate_bps > ref_bps {
            is_target_exceeded = true;
            observations.push("diagnostics.udp.target_exceeded_capacity".to_string());
        }
    }

    UdpDiagnosticResult {
        target_rate_bps,
        emitted_rate_bps,
        received_rate_bps,
        packets_sent,
        packets_received,
        packets_lost,
        loss_ratio,
        loss_percent,
        loss_level,
        title_key,
        observations,
        is_target_exceeded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_diagnostics_not_evaluable_when_zero_packets() {
        let res = evaluate_udp_diagnostics(0, 0, 0, 0, None, None, None);
        assert_eq!(res.loss_level, UdpLossLevel::NotEvaluable);
        assert_eq!(res.title_key, "diagnostics.udp.not_evaluable");
    }

    #[test]
    fn test_udp_diagnostics_low_loss() {
        // 50 perdidos de 100_000 = 0.05 % < 0.1 %
        let res = evaluate_udp_diagnostics(100_000, 99_950, 100_000_000, 99_950_000, Some(100_000_000), Some(1_000_000_000), None);
        assert_eq!(res.loss_level, UdpLossLevel::Low);
        assert_eq!(res.title_key, "diagnostics.udp.loss_low");
        assert_eq!(res.packets_lost, 50);
        assert!((res.loss_percent - 0.05).abs() < 1e-6);
        assert!(!res.is_target_exceeded);
    }

    #[test]
    fn test_udp_diagnostics_moderate_loss() {
        // 500 perdidos de 100_000 = 0.5 % (entre 0.1 % y 1.0 %)
        let res = evaluate_udp_diagnostics(100_000, 99_500, 100_000_000, 99_500_000, Some(100_000_000), Some(1_000_000_000), None);
        assert_eq!(res.loss_level, UdpLossLevel::Moderate);
        assert_eq!(res.title_key, "diagnostics.udp.loss_moderate");
        assert_eq!(res.packets_lost, 500);
        assert!((res.loss_percent - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_udp_diagnostics_high_loss() {
        // 2500 perdidos de 100_000 = 2.5 % > 1.0 %
        let res = evaluate_udp_diagnostics(100_000, 97_500, 100_000_000, 97_500_000, Some(100_000_000), Some(1_000_000_000), None);
        assert_eq!(res.loss_level, UdpLossLevel::High);
        assert_eq!(res.title_key, "diagnostics.udp.loss_high");
        assert_eq!(res.packets_lost, 2500);
        assert!((res.loss_percent - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_udp_target_exceeded_capacity_observation() {
        // Emitted rate 1.2 Gbps > capacity ref 1.0 Gbps
        let res = evaluate_udp_diagnostics(100_000, 95_000, 1_200_000_000, 950_000_000, Some(1_200_000_000), Some(1_000_000_000), None);
        assert!(res.is_target_exceeded);
        assert!(res.observations.contains(&"diagnostics.udp.target_exceeded_capacity".to_string()));
    }
}
