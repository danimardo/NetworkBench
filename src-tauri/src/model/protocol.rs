use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_FRAME_SIZE_BYTES: usize = 1_048_576; // 1 MiB

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolMessageType {
    Hello,
    PairRequest,
    PairResult,
    Request,
    Response,
    Prepare,
    PrepareResult,
    Ready,
    Start,
    Started,
    Sample,
    EngineDone,
    EngineFailed,
    SessionResult,
    SessionAck,
    Cancel,
    CancelAck,
    Heartbeat,
    Error,
    Bye,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolEnvelope<T> {
    #[serde(rename = "type")]
    pub msg_type: ProtocolMessageType,
    pub id: Uuid,
    pub session_id: Option<Uuid>,
    pub ts: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<Uuid>,
    pub payload: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloPayload {
    pub protocol_version: u32,
    pub protocol_min: u32,
    pub app_version: String,
    pub instance_id: Uuid,
    pub display_name: String,
    pub platform: String,
    pub is_busy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairRequestPayload {
    pub pairing_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairResultPayload {
    pub accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPayload {
    pub plan: super::plan::BenchmarkPlan,
    pub estimate_seconds: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_interface: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponsePayload {
    pub accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelPayload {
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatPayload {
    pub nonce: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparePayload {
    pub direction: String,
    pub port: u16,
    pub streams: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadyPayload {
    pub direction: String,
    pub is_ready: bool,
    pub listening_port: u16,
}

/// Negociación de inicio sin sincronización de reloj absoluto (Constitución VII):
/// start_delay_ms: offset relativo en milisegundos desde la recepción del mensaje
/// tolerance_ms: ventana máxima admisible antes de declarar desincronización
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPayload {
    pub direction: String,
    pub start_delay_ms: u64,
    pub tolerance_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartedPayload {
    pub direction: String,
    pub local_t_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineDonePayload {
    pub direction: String,
    pub role: String,
    pub throughput_bps: String,
    pub total_bytes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionAckPayload {
    pub session_id: Uuid,
    pub persisted: bool,
    pub thresholds_hash: String,
}

/// Framing helpers:
/// 4 bytes big-endian length prefix + UTF-8 payload.
pub fn encode_frame(payload_json: &[u8]) -> Result<Vec<u8>, String> {
    let len = payload_json.len();
    if len > MAX_FRAME_SIZE_BYTES {
        return Err(format!(
            "El tamaño del payload excede el límite máximo de 1 MiB (tamaño: {} bytes)",
            len
        ));
    }
    let mut frame = Vec::with_capacity(4 + len);
    frame.extend_from_slice(&(len as u32).to_be_bytes());
    frame.extend_from_slice(payload_json);
    Ok(frame)
}

pub fn decode_frame_length(header: [u8; 4]) -> usize {
    u32::from_be_bytes(header) as usize
}
