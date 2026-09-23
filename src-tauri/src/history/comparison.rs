use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const COHORT_SAMPLE_LIMIT: usize = 5;
pub const COHORT_DIFFERENCE_THRESHOLD_PERCENT: f64 = 20.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CohortComparisonResult {
    pub sample_count: usize,
    pub average_forward_bps: u64,
    pub current_forward_bps: u64,
    pub difference_percent: f64,
    pub is_significantly_lower: bool,
    pub is_significantly_higher: bool,
    pub observation_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerTrendPoint {
    pub session_id: Uuid,
    pub created_at: String,
    pub forward_bps: Option<String>,
    pub reverse_bps: Option<String>,
    pub verdict: Option<String>,
    pub protocol: String,
    pub client_interface: Option<String>,
    pub server_interface: Option<String>,
}

/// Formatea un valor en bps a una cadena legible en Gbit/s o Mbit/s con coma decimal según §19.4.
pub fn format_bps_human(bps: u64) -> String {
    if bps >= 1_000_000_000 {
        let gbit = bps as f64 / 1_000_000_000.0;
        format!("{:.1} Gbit/s", gbit).replace('.', ",")
    } else {
        let mbit = bps as f64 / 1_000_000.0;
        format!("{:.1} Mbit/s", mbit).replace('.', ",")
    }
}

/// Compara una sesión TCP con la cohorte histórica compatible (mismo peer, mismas interfaces, TCP, completadas).
/// Según Historias.md §19.4:
/// - Se toman hasta 5 sesiones completadas previas en la misma cohorte.
/// - Si la actual difiere en más de un 20 % de la media reciente, genera una observación de texto descriptiva.
/// - Siempre observación, nunca causa raíz.
pub fn evaluate_cohort_comparison(
    current_bps: u64,
    past_bps_values: &[u64],
) -> Option<CohortComparisonResult> {
    if past_bps_values.is_empty() {
        return None;
    }

    let sample_count = past_bps_values.len().min(COHORT_SAMPLE_LIMIT);
    let slice = &past_bps_values[..sample_count];
    let sum: u64 = slice.iter().sum();
    let average_bps = sum / sample_count as u64;

    if average_bps == 0 {
        return None;
    }

    let diff = (current_bps as f64 - average_bps as f64).abs();
    let diff_percent = (diff / average_bps as f64) * 100.0;

    let is_significantly_lower = current_bps < average_bps && diff_percent > COHORT_DIFFERENCE_THRESHOLD_PERCENT;
    let is_significantly_higher = current_bps > average_bps && diff_percent > COHORT_DIFFERENCE_THRESHOLD_PERCENT;

    let observation_text = if is_significantly_lower {
        let avg_str = format_bps_human(average_bps);
        Some(format!(
            "El rendimiento actual es un {:.0} % inferior a la media reciente entre estos equipos ({})",
            diff_percent, avg_str
        ))
    } else if is_significantly_higher {
        let avg_str = format_bps_human(average_bps);
        Some(format!(
            "El rendimiento actual es un {:.0} % superior a la media reciente entre estos equipos ({})",
            diff_percent, avg_str
        ))
    } else {
        None
    };

    Some(CohortComparisonResult {
        sample_count,
        average_forward_bps: average_bps,
        current_forward_bps: current_bps,
        difference_percent: diff_percent,
        is_significantly_lower,
        is_significantly_higher,
        observation_text,
    })
}

/// Busca las sesiones previas compatibles en la base de datos y evalúa la comparación de cohorte.
pub fn compare_session_with_cohort(
    conn: &Connection,
    session_id: &Uuid,
) -> Result<Option<CohortComparisonResult>> {
    // 1. Obtener datos de la sesión actual
    let mut stmt = conn.prepare(
        r#"
        SELECT peer_id, protocol, status, is_partial, forward_bps, client_interface, server_interface
        FROM sessions
        WHERE id = ?1
        "#,
    )?;

    let current = stmt.query_row(params![session_id.to_string()], |r| {
        let peer_id_str: Option<String> = r.get(0)?;
        let protocol: String = r.get(1)?;
        let status: String = r.get(2)?;
        let is_partial: i64 = r.get(3)?;
        let forward_bps_str: Option<String> = r.get(4)?;
        let client_interface: Option<String> = r.get(5)?;
        let server_interface: Option<String> = r.get(6)?;

        Ok((
            peer_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
            protocol,
            status,
            is_partial == 1,
            forward_bps_str.and_then(|s| s.parse::<u64>().ok()),
            client_interface,
            server_interface,
        ))
    })?;

    let (peer_id, protocol, status, is_partial, forward_bps, client_interface, server_interface) = current;

    // Solo se evalúa cohorte para TCP completadas con peer conocido y velocidad válida
    if protocol.to_lowercase() != "tcp" || status != "completed" || is_partial {
        return Ok(None);
    }

    let current_bps = match forward_bps {
        Some(bps) => bps,
        None => return Ok(None),
    };

    let peer_id = match peer_id {
        Some(pid) => pid,
        None => return Ok(None),
    };

    // 2. Buscar hasta 5 sesiones completadas anteriores en la misma cohorte
    let mut cohort_stmt = conn.prepare(
        r#"
        SELECT forward_bps
        FROM sessions
        WHERE peer_id = ?1
          AND id != ?2
          AND protocol = 'tcp'
          AND status = 'completed'
          AND is_partial = 0
          AND forward_bps IS NOT NULL
          AND ((client_interface IS NULL AND ?3 IS NULL) OR client_interface = ?3)
          AND ((server_interface IS NULL AND ?4 IS NULL) OR server_interface = ?4)
        ORDER BY created_at DESC
        LIMIT ?5
        "#,
    )?;

    let rows = cohort_stmt.query_map(
        params![
            peer_id.to_string(),
            session_id.to_string(),
            client_interface,
            server_interface,
            COHORT_SAMPLE_LIMIT as i64
        ],
        |r| {
            let bps_str: String = r.get(0)?;
            Ok(bps_str.parse::<u64>().unwrap_or(0))
        },
    )?;

    let mut past_bps = Vec::new();
    for row in rows {
        let val = row?;
        if val > 0 {
            past_bps.push(val);
        }
    }

    Ok(evaluate_cohort_comparison(current_bps, &past_bps))
}

/// Obtiene los puntos de evolución temporal para un peer específico (para gráfica de tendencia).
pub fn get_peer_trend(
    conn: &Connection,
    peer_id: &Uuid,
    protocol_filter: Option<&str>,
) -> Result<Vec<PeerTrendPoint>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, created_at, forward_bps, reverse_bps, diagnostic_verdict,
               protocol, client_interface, server_interface
        FROM sessions
        WHERE peer_id = ?1
          AND status = 'completed'
          AND (?2 IS NULL OR protocol = ?2)
        ORDER BY created_at ASC
        "#,
    )?;

    let rows = stmt.query_map(
        params![peer_id.to_string(), protocol_filter.map(|p| p.to_lowercase())],
        |r| {
            let id_str: String = r.get(0)?;
            let created_at: String = r.get(1)?;
            let forward_bps: Option<String> = r.get(2)?;
            let reverse_bps: Option<String> = r.get(3)?;
            let verdict: Option<String> = r.get(4)?;
            let protocol: String = r.get(5)?;
            let client_interface: Option<String> = r.get(6)?;
            let server_interface: Option<String> = r.get(7)?;

            let session_id = Uuid::parse_str(&id_str).unwrap_or_default();

            Ok(PeerTrendPoint {
                session_id,
                created_at,
                forward_bps,
                reverse_bps,
                verdict,
                protocol,
                client_interface,
                server_interface,
            })
        },
    )?;

    let mut points = Vec::new();
    for row in rows {
        points.push(row?);
    }
    Ok(points)
}
