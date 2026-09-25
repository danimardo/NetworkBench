use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleRecord {
    pub t_ms: u64,
    pub direction: String,
    pub bps: String,
    pub cpu_percent: Option<f64>,
    pub gap: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub id: Uuid,
    pub created_at: String,
    pub peer_id: Option<Uuid>,
    pub status: String,
    pub protocol: String,
    pub streams: u32,
    pub duration_seconds: u32,
    pub forward_bps: Option<String>,
    pub reverse_bps: Option<String>,
    pub is_partial: bool,
    pub diagnostic_verdict: Option<String>,
    #[serde(default)]
    pub client_interface: Option<String>,
    #[serde(default)]
    pub server_interface: Option<String>,
    #[serde(default)]
    pub result_json: Option<String>,
    #[serde(default)]
    pub plan_json: Option<String>,
    pub samples: Vec<SampleRecord>,
}

impl Default for SessionRecord {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            created_at: String::new(),
            peer_id: None,
            status: "completed".into(),
            protocol: "tcp".into(),
            streams: 1,
            duration_seconds: 10,
            forward_bps: None,
            reverse_bps: None,
            is_partial: false,
            diagnostic_verdict: None,
            client_interface: None,
            server_interface: None,
            result_json: None,
            plan_json: None,
            samples: Vec::new(),
        }
    }
}

pub fn insert_session_idempotent(conn: &mut Connection, session: &SessionRecord) -> Result<()> {
    let tx = conn.transaction()?;

    // 1. Insert or replace session
    tx.execute(
        r#"
        INSERT INTO sessions (
            id, created_at, peer_id, status, protocol, streams,
            duration_seconds, forward_bps, reverse_bps, is_partial, diagnostic_verdict,
            client_interface, server_interface, result_json, plan_json
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ON CONFLICT(id) DO UPDATE SET
            status = excluded.status,
            forward_bps = excluded.forward_bps,
            reverse_bps = excluded.reverse_bps,
            is_partial = excluded.is_partial,
            diagnostic_verdict = excluded.diagnostic_verdict,
            client_interface = excluded.client_interface,
            server_interface = excluded.server_interface,
            result_json = excluded.result_json,
            plan_json = excluded.plan_json;
        "#,
        params![
            session.id.to_string(),
            session.created_at,
            session.peer_id.map(|id| id.to_string()),
            session.status,
            session.protocol,
            session.streams,
            session.duration_seconds,
            session.forward_bps,
            session.reverse_bps,
            session.is_partial as i64,
            session.diagnostic_verdict,
            session.client_interface,
            session.server_interface,
            session.result_json,
            session.plan_json,
        ],
    )?;

    // 2. Limpiar muestras previas si es una reescritura idempotente
    tx.execute(
        "DELETE FROM samples WHERE session_id = ?1",
        params![session.id.to_string()],
    )?;

    // 3. Insertar muestras
    let mut stmt = tx.prepare(
        r#"
        INSERT INTO samples (session_id, t_ms, direction, bps, cpu_percent, gap)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )?;

    for sample in &session.samples {
        stmt.execute(params![
            session.id.to_string(),
            sample.t_ms as i64,
            sample.direction,
            sample.bps,
            sample.cpu_percent,
            sample.gap as i64,
        ])?;
    }
    drop(stmt);

    tx.commit()?;
    Ok(())
}

pub fn get_session_by_id(conn: &Connection, session_id: &Uuid) -> Result<Option<SessionRecord>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, created_at, peer_id, status, protocol, streams,
               duration_seconds, forward_bps, reverse_bps, is_partial, diagnostic_verdict,
               client_interface, server_interface, result_json, plan_json
        FROM sessions
        WHERE id = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![session_id.to_string()])?;
    if let Some(row) = rows.next()? {
        let id_str: String = row.get(0)?;
        let created_at: String = row.get(1)?;
        let peer_id_str: Option<String> = row.get(2)?;
        let status: String = row.get(3)?;
        let protocol: String = row.get(4)?;
        let streams: u32 = row.get(5)?;
        let duration_seconds: u32 = row.get(6)?;
        let forward_bps: Option<String> = row.get(7)?;
        let reverse_bps: Option<String> = row.get(8)?;
        let is_partial: i64 = row.get(9)?;
        let diagnostic_verdict: Option<String> = row.get(10)?;
        let client_interface: Option<String> = row.get(11)?;
        let server_interface: Option<String> = row.get(12)?;
        let result_json: Option<String> = row.get(13)?;
        let plan_json: Option<String> = row.get(14)?;

        let peer_id = peer_id_str.and_then(|s| Uuid::parse_str(&s).ok());
        let id = Uuid::parse_str(&id_str).unwrap_or(*session_id);

        // Cargar muestras ordenadas por t_ms
        let mut sample_stmt = conn.prepare(
            r#"
            SELECT t_ms, direction, bps, cpu_percent, gap
            FROM samples
            WHERE session_id = ?1
            ORDER BY t_ms ASC
            "#,
        )?;

        let sample_rows = sample_stmt.query_map(params![session_id.to_string()], |s_row| {
            let t_ms: i64 = s_row.get(0)?;
            let direction: String = s_row.get(1)?;
            let bps: String = s_row.get(2)?;
            let cpu_percent: Option<f64> = s_row.get(3)?;
            let gap: i64 = s_row.get(4)?;

            Ok(SampleRecord {
                t_ms: t_ms as u64,
                direction,
                bps,
                cpu_percent,
                gap: gap == 1,
            })
        })?;

        let mut samples = Vec::new();
        for s in sample_rows {
            samples.push(s?);
        }

        Ok(Some(SessionRecord {
            id,
            created_at,
            peer_id,
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
            result_json,
            plan_json,
            samples,
        }))
    } else {
        Ok(None)
    }
}

/// Persiste el resultado de una sesión ya terminada, junto al peer con el que se midió.
///
/// Vive aquí porque `history` es el único propietario del esquema y del SQL (plan,
/// §Fronteras): quien orquesta la sesión entrega un `SessionResult` y no construye filas.
///
/// El peer se registra antes que la sesión porque `sessions.peer_id` tiene clave foránea
/// hacia `peers(id)`. `upsert_peer` no rebaja la confianza de un peer ya conocido.
///
/// Es idempotente por identificador de sesión (`insert_session_idempotent`): guardar dos
/// veces el mismo resultado no duplica nada, que es lo que exige reintentar tras un fallo
/// parcial sin corromper el historial (FR-045).
pub fn guardar_resultado_de_sesion(
    conn: &mut Connection,
    peer: &crate::model::peer::Peer,
    resultado: &crate::model::result::SessionResult,
) -> Result<()> {
    crate::history::peers::upsert_peer(conn, peer)?;

    let oficial = |sentido: &str| {
        resultado
            .directions
            .iter()
            .find(|d| d.direction == sentido)
            .and_then(|d| d.official_bps.clone())
    };

    let veredicto = resultado
        .verdict
        .as_ref()
        .and_then(|v| serde_json::to_value(v.level).ok())
        .and_then(|v| v.as_str().map(str::to_string));

    let protocolo = match resultado.plan.protocol {
        crate::model::plan::BenchmarkProtocol::Tcp => "tcp",
        crate::model::plan::BenchmarkProtocol::Udp => "udp",
    };

    let registro = SessionRecord {
        id: resultado.session_id,
        created_at: resultado.started_at.clone(),
        peer_id: Some(peer.instance_id),
        status: resultado.status.clone(),
        protocol: protocolo.to_string(),
        streams: resultado.plan.streams,
        duration_seconds: resultado.plan.measure_seconds,
        forward_bps: oficial("forward"),
        reverse_bps: oficial("reverse"),
        // Una sesión que no llegó a completarse se guarda, pero marcada como parcial:
        // nunca se presenta un parcial como éxito (FR-034).
        is_partial: resultado.status != "completed",
        diagnostic_verdict: veredicto,
        client_interface: None,
        server_interface: None,
        result_json: serde_json::to_string(resultado).ok(),
        plan_json: serde_json::to_string(&resultado.plan).ok(),
        samples: Vec::new(),
    };

    insert_session_idempotent(conn, &registro)
}

#[cfg(test)]
mod guardado_tests {
    use super::*;
    use crate::control::engine_port::MotorDeLaboratorio;
    use crate::control::orquestador::Orquestador;
    use crate::control::session_flow::ensamblar_resultado;
    use crate::model::peer::Peer;
    use crate::model::plan::BenchmarkPlan;
    use std::sync::Arc;

    fn peer() -> Peer {
        Peer::new(
            Uuid::new_v4(),
            "Equipo".into(),
            "a".repeat(64),
            vec!["127.0.0.1:7411".into()],
        )
        .unwrap()
    }

    fn base_de_datos() -> Connection {
        let ruta = std::env::temp_dir().join(format!("nb_guardado_{}.db", Uuid::new_v4()));
        let db = crate::history::database::Database::open(ruta).expect("abrir");
        // `Database` posee la conexión tras un Mutex; para la prueba se abre otra
        // sobre el mismo fichero migrado.
        Connection::open(db.path()).expect("abrir conexión")
    }

    fn resultado_de(peer: &Peer) -> crate::model::result::SessionResult {
        let orq = Orquestador::new(Arc::new(MotorDeLaboratorio::con_respuestas(vec![])));
        let plan = BenchmarkPlan::new_standard_tcp(7412);
        ensamblar_resultado(
            &orq,
            Uuid::new_v4(),
            "2026-09-25T10:00:00.000Z",
            "2026-09-25T10:00:30.000Z",
            &plan,
            peer.clone(),
            peer.clone(),
            vec![],
        )
    }

    #[test]
    fn test_guarda_el_resultado_y_se_puede_recuperar() {
        let mut conn = base_de_datos();
        let p = peer();
        let r = resultado_de(&p);

        guardar_resultado_de_sesion(&mut conn, &p, &r).expect("guardar");

        let leido = get_session_by_id(&conn, &r.session_id)
            .expect("consulta")
            .expect("debe existir");
        assert_eq!(leido.id, r.session_id);
        assert_eq!(leido.peer_id, Some(p.instance_id));
        // Sin direcciones completadas, la sesión es parcial y así queda registrada.
        assert!(leido.is_partial);
        assert!(leido.result_json.is_some());
    }

    #[test]
    fn test_guardar_dos_veces_no_duplica() {
        let mut conn = base_de_datos();
        let p = peer();
        let r = resultado_de(&p);

        guardar_resultado_de_sesion(&mut conn, &p, &r).unwrap();
        guardar_resultado_de_sesion(&mut conn, &p, &r).unwrap();

        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |f| f.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }
}
