use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapacitySource {
    Manual,
    Negotiated,
    Wifi,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapacityReference {
    pub ref_bps: Option<u64>,
    pub ref_source: CapacitySource,
    pub cap_a_bps: Option<u64>,
    pub cap_b_bps: Option<u64>,
}

impl CapacityReference {
    pub fn new(
        expected_capacity_mbps: Option<u32>,
        cap_a_bps: Option<u64>,
        is_a_wireless_or_virtual: bool,
        cap_b_bps: Option<u64>,
        is_b_wireless_or_virtual: bool,
    ) -> Self {
        // 1. Capacidad manual explícita
        if let Some(mbps) = expected_capacity_mbps
            && mbps > 0
        {
            return Self {
                ref_bps: Some(mbps as u64 * 1_000_000),
                ref_source: CapacitySource::Manual,
                cap_a_bps,
                cap_b_bps,
            };
        }

        // 2. Si alguno de los adaptadores es wifi/virtual y no hay manual -> no determinable de forma fija
        if is_a_wireless_or_virtual || is_b_wireless_or_virtual {
            let source = if is_a_wireless_or_virtual || is_b_wireless_or_virtual {
                CapacitySource::Wifi
            } else {
                CapacitySource::Unknown
            };
            return Self {
                ref_bps: None,
                ref_source: source,
                cap_a_bps,
                cap_b_bps,
            };
        }

        // 3. Si ambos tienen enlace negociado (Ethernet), la capacidad es el mínimo de ambos
        match (cap_a_bps, cap_b_bps) {
            (Some(a), Some(b)) if a > 0 && b > 0 => Self {
                ref_bps: Some(a.min(b)),
                ref_source: CapacitySource::Negotiated,
                cap_a_bps: Some(a),
                cap_b_bps: Some(b),
            },
            (Some(a), None) if a > 0 => Self {
                ref_bps: Some(a),
                ref_source: CapacitySource::Negotiated,
                cap_a_bps: Some(a),
                cap_b_bps: None,
            },
            (None, Some(b)) if b > 0 => Self {
                ref_bps: Some(b),
                ref_source: CapacitySource::Negotiated,
                cap_a_bps: None,
                cap_b_bps: Some(b),
            },
            _ => Self {
                ref_bps: None,
                ref_source: CapacitySource::Unknown,
                cap_a_bps,
                cap_b_bps,
            },
        }
    }

    pub fn has_performance_verdict(&self) -> bool {
        self.ref_bps.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_capacity_overrides_everything() {
        let cap = CapacityReference::new(
            Some(1000),
            Some(10_000_000_000),
            true,
            Some(1_000_000_000),
            false,
        );
        assert_eq!(cap.ref_bps, Some(1_000_000_000));
        assert_eq!(cap.ref_source, CapacitySource::Manual);
        assert!(cap.has_performance_verdict());
    }

    #[test]
    fn test_negotiated_minimum_link_speed() {
        let cap = CapacityReference::new(
            None,
            Some(10_000_000_000),
            false,
            Some(1_000_000_000),
            false,
        );
        assert_eq!(cap.ref_bps, Some(1_000_000_000));
        assert_eq!(cap.ref_source, CapacitySource::Negotiated);
        assert!(cap.has_performance_verdict());
    }

    #[test]
    fn test_wireless_without_manual_has_no_ref_bps() {
        let cap = CapacityReference::new(None, Some(866_000_000), true, Some(1_000_000_000), false);
        assert_eq!(cap.ref_bps, None);
        assert_eq!(cap.ref_source, CapacitySource::Wifi);
        assert!(!cap.has_performance_verdict());
    }

    #[test]
    fn test_unknown_when_missing_link_speeds() {
        let cap = CapacityReference::new(None, None, false, None, false);
        assert_eq!(cap.ref_bps, None);
        assert_eq!(cap.ref_source, CapacitySource::Unknown);
        assert!(!cap.has_performance_verdict());
    }
}
