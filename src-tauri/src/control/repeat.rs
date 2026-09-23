use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::history::sessions::get_session_by_id;
use crate::model::peer::Peer;
use crate::model::plan::{BenchmarkDirection, BenchmarkPlan, BenchmarkProtocol};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepeatPlanConfig {
    pub peer: Option<Peer>,
    pub plan: BenchmarkPlan,
    pub original_session_id: Uuid,
}

/// Reconstruye un plan validado para repetir una prueba histórica según §19.2.
/// Nunca pasa argumentos históricos libres: reconstruye y valida estrictamente
/// contra los límites de seguridad actuales del sistema.
pub fn reconstruct_repeat_plan(
    conn: &Connection,
    session_id: &Uuid,
) -> Result<RepeatPlanConfig, AppError> {
    let session = get_session_by_id(conn, session_id)
        .map_err(|e| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Fatal,
                "errors.NB-INTERNAL-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "repeat.get_session".into(),
                code: "DB_ERROR".into(),
                message_key: "errors.NB-INTERNAL-001".into(),
                safe_params: Some(serde_json::json!({ "details": e.to_string() })),
            })
        })?
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::ParamRejected,
                ErrorSeverity::Error,
                "errors.NB-PARAM-001",
            )
            .with_issue(crate::errors::AppIssue {
                path: "repeat.session_not_found".into(),
                code: "SESSION_NOT_FOUND".into(),
                message_key: "errors.NB-PARAM-001".into(),
                safe_params: Some(serde_json::json!({ "sessionId": session_id.to_string() })),
            })
        })?;

    // Reconstruir o deserializar plan
    let plan = if let Some(ref plan_json) = session.plan_json {
        serde_json::from_str::<BenchmarkPlan>(plan_json)
            .unwrap_or_else(|_| fallback_plan_from_session(&session))
    } else {
        fallback_plan_from_session(&session)
    };

    // Validar rigurosamente los límites
    plan.validate().map_err(|err_msg| {
        AppError::new(
            ErrorCode::ParamRejected,
            ErrorSeverity::Error,
            "errors.NB-PARAM-001",
        )
        .with_issue(crate::errors::AppIssue {
            path: "repeat.plan_validation".into(),
            code: "PLAN_INVALID".into(),
            message_key: "errors.NB-PARAM-001".into(),
            safe_params: Some(serde_json::json!({ "reason": err_msg })),
        })
    })?;

    // Cargar peer asociado si existe
    let peer = if let Some(ref pid) = session.peer_id {
        crate::history::peers::get_peer_by_fingerprint(conn, &pid.to_string())
            .ok()
            .flatten()
    } else {
        None
    };

    Ok(RepeatPlanConfig {
        peer,
        plan,
        original_session_id: *session_id,
    })
}

fn fallback_plan_from_session(session: &crate::history::sessions::SessionRecord) -> BenchmarkPlan {
    let protocol = if session.protocol.to_lowercase() == "udp" {
        BenchmarkProtocol::Udp
    } else {
        BenchmarkProtocol::Tcp
    };

    BenchmarkPlan {
        protocol,
        direction: BenchmarkDirection::Forward,
        streams: session.streams.clamp(1, 64),
        warmup_seconds: 1,
        measure_seconds: session.duration_seconds.clamp(5, 300),
        cooldown_seconds: 1,
        port: 7412,
        buffer_size_bytes: None,
        udp_target_rate_bps: None,
        udp_packet_size_bytes: None,
        expected_capacity_bps: None,
    }
}
