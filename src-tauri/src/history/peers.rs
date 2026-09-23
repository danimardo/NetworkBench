use crate::model::peer::{Peer, TrustState};
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

pub fn upsert_peer(conn: &Connection, peer: &Peer) -> Result<()> {
    let (is_trusted, auto_accept) = match peer.trust_state {
        TrustState::Unknown => (0, 0),
        TrustState::Known => (0, 0),
        TrustState::Trusted => (1, 0),
        TrustState::TrustedAutoAccept => (1, 1),
    };

    conn.execute(
        r#"
        INSERT INTO peers (id, fingerprint, display_name, alias, is_trusted, auto_accept, first_seen_at, last_seen_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
        ON CONFLICT(fingerprint) DO UPDATE SET
            display_name = excluded.display_name,
            alias = COALESCE(excluded.alias, peers.alias),
            last_seen_at = excluded.last_seen_at;
        "#,
        params![
            peer.instance_id.to_string(),
            peer.fingerprint,
            peer.display_name,
            peer.alias,
            is_trusted,
            auto_accept,
            peer.last_seen,
        ],
    )?;

    Ok(())
}

pub fn get_peer_by_fingerprint(conn: &Connection, fingerprint: &str) -> Result<Option<Peer>> {
    let norm_fp = fingerprint.trim().to_lowercase();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, fingerprint, display_name, alias, is_trusted, auto_accept, last_seen_at
        FROM peers
        WHERE fingerprint = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![norm_fp])?;
    if let Some(row) = rows.next()? {
        let id_str: String = row.get(0)?;
        let fp: String = row.get(1)?;
        let display_name: String = row.get(2)?;
        let alias: Option<String> = row.get(3)?;
        let is_trusted: i64 = row.get(4)?;
        let auto_accept: i64 = row.get(5)?;
        let last_seen: String = row.get(6)?;

        let trust_state = if is_trusted == 1 && auto_accept == 1 {
            TrustState::TrustedAutoAccept
        } else if is_trusted == 1 {
            TrustState::Trusted
        } else {
            TrustState::Known
        };

        let instance_id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::nil());

        Ok(Some(Peer {
            instance_id,
            display_name,
            fingerprint: fp,
            addresses: vec![],
            trust_state,
            auto_accept: auto_accept == 1,
            last_seen,
            alias,
        }))
    } else {
        Ok(None)
    }
}
