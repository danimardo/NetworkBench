use crate::diagnostic::{AsymmetryStats, CapacityReference, RetransmissionStats, SessionVerdict, StabilityStats};
use super::plan::BenchmarkPlan;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CURRENT_RESULT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerSnapshot {
    pub instance_id: Uuid,
    pub display_name: String,
    pub fingerprint: String,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineResult {
    pub role: String, // "sender" | "receiver"
    pub total_bytes: String,
    pub realtime_seconds: f64,
    pub throughput_bps: String,
    pub cpu_percent: Option<f64>,
    pub buffers_count: Option<u64>,
    pub errors_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectionResult {
    pub direction: String, // "forward" | "reverse"
    pub status: String,    // "completed" | "incomplete" | "notStarted"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<EngineResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<EngineResult>,
    /// El throughput oficial de la prueba procede EXCLUSIVAMENTE del receptor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub official_bps: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utilization: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<StabilityStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retransmission: Option<RetransmissionStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_sender: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_receiver: Option<f64>,
    pub samples_count: usize,
    pub gaps_count: usize,
}

impl DirectionResult {
    pub fn new_completed(
        direction: &str,
        sender: EngineResult,
        receiver: EngineResult,
        samples_count: usize,
        gaps_count: usize,
    ) -> Self {
        let official_bps = Some(receiver.throughput_bps.clone());
        let cpu_sender = sender.cpu_percent;
        let cpu_receiver = receiver.cpu_percent;
        Self {
            direction: direction.to_string(),
            status: "completed".to_string(),
            sender: Some(sender),
            receiver: Some(receiver),
            official_bps,
            utilization: None,
            stability: None,
            retransmission: None,
            cpu_sender,
            cpu_receiver,
            samples_count,
            gaps_count,
        }
    }

    pub fn new_incomplete(direction: &str) -> Self {
        Self {
            direction: direction.to_string(),
            status: "incomplete".to_string(),
            sender: None,
            receiver: None,
            official_bps: None,
            utilization: None,
            stability: None,
            retransmission: None,
            cpu_sender: None,
            cpu_receiver: None,
            samples_count: 0,
            gaps_count: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultVersions {
    pub app_version: String,
    pub protocol_version: u32,
    pub engine_version: String,
    pub schema_version: u32,
    pub thresholds_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResult {
    pub schema_version: u32,
    pub session_id: Uuid,
    pub started_at: String,
    pub finished_at: String,
    pub status: String, // "completed" | "incomplete"
    pub initiator: PeerSnapshot,
    pub responder: PeerSnapshot,
    pub plan: BenchmarkPlan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<CapacityReference>,
    pub directions: Vec<DirectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asymmetry: Option<AsymmetryStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<SessionVerdict>,
    pub result_source: String, // "initiator" | "local"
    pub versions: ResultVersions,
}

impl SessionResult {
    pub fn is_fully_completed(&self) -> bool {
        self.status == "completed"
            && self.directions.iter().all(|d| d.status == "completed" && d.official_bps.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_result_official_bps_only_from_receiver() {
        let sender = EngineResult {
            role: "sender".into(),
            total_bytes: "1190000000".into(),
            realtime_seconds: 10.0,
            throughput_bps: "952000000".into(), // emisor reporta 952 Mbps
            cpu_percent: Some(5.0),
            buffers_count: Some(18000),
            errors_count: 0,
            raw: None,
        };
        let receiver = EngineResult {
            role: "receiver".into(),
            total_bytes: "1185000000".into(),
            realtime_seconds: 10.0,
            throughput_bps: "948000000".into(), // receptor reporta 948 Mbps
            cpu_percent: Some(4.0),
            buffers_count: Some(18000),
            errors_count: 0,
            raw: None,
        };

        let dir_res = DirectionResult::new_completed("forward", sender, receiver, 20, 0);
        // Debe ser exactamente el del receptor (948 Mbps, no 952 Mbps)
        assert_eq!(dir_res.official_bps.as_deref(), Some("948000000"));
        assert_eq!(dir_res.status, "completed");
    }

    #[test]
    fn test_incomplete_direction_has_no_official_bps() {
        let dir_res = DirectionResult::new_incomplete("forward");
        assert_eq!(dir_res.official_bps, None);
        assert_eq!(dir_res.status, "incomplete");
    }
}
