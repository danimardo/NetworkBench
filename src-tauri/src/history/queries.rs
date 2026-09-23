use crate::history::sessions::SessionRecord;
use rusqlite::{params_from_iter, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryFilter {
    pub peer_id: Option<Uuid>,
    pub search_query: Option<String>,
    pub verdict: Option<String>,
    pub protocol: Option<String>,
    pub status: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItemSummary {
    pub id: Uuid,
    pub created_at: String,
    pub peer_id: Option<Uuid>,
    pub peer_name: String,
    pub peer_fingerprint: Option<String>,
    pub status: String,
    pub protocol: String,
    pub streams: u32,
    pub duration_seconds: u32,
    pub forward_bps: Option<String>,
    pub reverse_bps: Option<String>,
    pub is_partial: bool,
    pub diagnostic_verdict: Option<String>,
    pub client_interface: Option<String>,
    pub server_interface: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPage {
    pub items: Vec<HistoryItemSummary>,
    pub total_count: u32,
    pub has_more: bool,
}

/// Consulta de historial con paginación y filtros seguros (sin exponer SQL directo).
pub fn query_history(
    conn: &Connection,
    filter: &HistoryFilter,
    pagination: &Pagination,
) -> Result<HistoryPage> {
    let mut where_clauses: Vec<String> = Vec::new();
    let mut count_params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(peer_id) = filter.peer_id {
        where_clauses.push("s.peer_id = ?".into());
        count_params.push(peer_id.to_string().into());
    }

    if let Some(ref search) = filter.search_query {
        let pattern = format!("%{}%", search.trim());
        where_clauses.push("(p.display_name LIKE ? OR p.alias LIKE ?)".into());
        count_params.push(pattern.clone().into());
        count_params.push(pattern.into());
    }

    if let Some(ref verdict) = filter.verdict {
        let pattern = format!("%{}%", verdict.trim());
        where_clauses.push("s.diagnostic_verdict LIKE ?".into());
        count_params.push(pattern.into());
    }

    if let Some(ref protocol) = filter.protocol {
        where_clauses.push("s.protocol = ?".into());
        count_params.push(protocol.trim().to_lowercase().into());
    }

    if let Some(ref status) = filter.status {
        where_clauses.push("s.status = ?".into());
        count_params.push(status.trim().to_lowercase().into());
    }

    if let Some(ref from_date) = filter.from_date {
        where_clauses.push("s.created_at >= ?".into());
        count_params.push(from_date.trim().to_string().into());
    }

    if let Some(ref to_date) = filter.to_date {
        where_clauses.push("s.created_at <= ?".into());
        count_params.push(to_date.trim().to_string().into());
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // 1. Contar total de registros coincidentes
    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM sessions s
        LEFT JOIN peers p ON s.peer_id = p.id
        {}
        "#,
        where_sql
    );

    let mut count_stmt = conn.prepare(&count_sql)?;
    let total_count: u32 = count_stmt.query_row(params_from_iter(count_params.clone()), |r| r.get(0))?;

    // 2. Consulta de página ordenada por fecha descendente
    let limit = if pagination.limit == 0 { 20 } else { pagination.limit.min(100) };
    let offset = pagination.offset;

    let query_sql = format!(
        r#"
        SELECT s.id, s.created_at, s.peer_id,
               COALESCE(NULLIF(p.alias, ''), p.display_name, 'Equipo desconocido') AS peer_name,
               p.fingerprint,
               s.status, s.protocol, s.streams, s.duration_seconds,
               s.forward_bps, s.reverse_bps, s.is_partial, s.diagnostic_verdict,
               s.client_interface, s.server_interface
        FROM sessions s
        LEFT JOIN peers p ON s.peer_id = p.id
        {}
        ORDER BY s.created_at DESC
        LIMIT ? OFFSET ?
        "#,
        where_sql
    );

    let mut query_params = count_params;
    query_params.push((limit as i64).into());
    query_params.push((offset as i64).into());

    let mut query_stmt = conn.prepare(&query_sql)?;
    let rows = query_stmt.query_map(params_from_iter(query_params), |r| {
        let id_str: String = r.get(0)?;
        let created_at: String = r.get(1)?;
        let peer_id_str: Option<String> = r.get(2)?;
        let peer_name: String = r.get(3)?;
        let peer_fingerprint: Option<String> = r.get(4)?;
        let status: String = r.get(5)?;
        let protocol: String = r.get(6)?;
        let streams: u32 = r.get(7)?;
        let duration_seconds: u32 = r.get(8)?;
        let forward_bps: Option<String> = r.get(9)?;
        let reverse_bps: Option<String> = r.get(10)?;
        let is_partial: i64 = r.get(11)?;
        let diagnostic_verdict: Option<String> = r.get(12)?;
        let client_interface: Option<String> = r.get(13)?;
        let server_interface: Option<String> = r.get(14)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let peer_id = peer_id_str.and_then(|s| Uuid::parse_str(&s).ok());

        Ok(HistoryItemSummary {
            id,
            created_at,
            peer_id,
            peer_name,
            peer_fingerprint,
            status,
            protocol,
            streams,
            duration_seconds,
            forward_bps,
            reverse_bps,
            is_partial: is_partial == 1,
            diagnostic_verdict,
            client_interface,
            server_interface,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    let has_more = (offset + limit) < total_count;

    Ok(HistoryPage {
        items,
        total_count,
        has_more,
    })
}

/// Recupera el detalle completo de una sesión guardada junto con sus muestras para reapertura.
pub fn get_session_detail(conn: &Connection, session_id: &Uuid) -> Result<Option<SessionRecord>> {
    crate::history::sessions::get_session_by_id(conn, session_id)
}
