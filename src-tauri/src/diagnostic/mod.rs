pub mod capacity;
pub mod rules;
#[cfg(test)]
pub mod rules_tests;
pub mod udp;

pub const THRESHOLDS_JSON: &str = include_str!("thresholds.json");
pub const THRESHOLDS_HASH: &str =
    "b65f73615249f16c22fa3d63685f792b7f94661eed4f7e0eea454d1dba8923c0";

pub use capacity::{CapacityReference, CapacitySource};
pub use rules::{
    AsymmetryStats, CpuLevel, DiagnosticEngine, DiagnosticItem, RetransmissionLevel,
    RetransmissionStats, SessionVerdict, StabilityLevel, StabilityStats, VerdictInput,
    VerdictLevel,
};
pub use udp::{UdpDiagnosticResult, UdpLossLevel, UdpThresholds, evaluate_udp_diagnostics};
