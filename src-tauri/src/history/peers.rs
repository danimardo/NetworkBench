use crate::model::peer::{Peer, TrustState};
use rusqlite::{Connection, Result, params};
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
        INSERT INTO peers (id, fingerprint, display_name, alias, is_trusted, auto_accept, first_seen_at, last_seen_at, last_address)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)
        ON CONFLICT(fingerprint) DO UPDATE SET
            display_name = excluded.display_name,
            alias = COALESCE(excluded.alias, peers.alias),
            last_seen_at = excluded.last_seen_at,
            last_address = COALESCE(excluded.last_address, peers.last_address);
        "#,
        params![
            peer.instance_id.to_string(),
            peer.fingerprint,
            peer.display_name,
            peer.alias,
            is_trusted,
            auto_accept,
            peer.last_seen,
            peer.addresses.first(),
        ],
    )?;

    Ok(())
}

/// Un equipo guardado, con lo mínimo que necesita la comprobación de alcance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerParaAlcance {
    pub instance_id: Uuid,
    pub fingerprint: String,
    /// `ip:puerto` de control de la última vez que se vio; `None` si nunca se supo.
    pub last_address: Option<String>,
}

pub fn peers_para_alcance(conn: &Connection) -> Result<Vec<PeerParaAlcance>> {
    let mut stmt = conn.prepare("SELECT id, fingerprint, last_address FROM peers")?;
    let filas = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        Ok(PeerParaAlcance {
            instance_id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::nil()),
            fingerprint: row.get(1)?,
            last_address: row.get(2)?,
        })
    })?;
    filas.collect()
}

/// Anota que el equipo se ha visto ahora (por mDNS o por una conexión de comprobación) y,
/// si se conoce, la dirección con la que se le vio. Es un dato para mostrar y para
/// reintentar: la identidad la sigue demostrando el certificado al conectar de verdad.
pub fn registrar_avistamiento(
    conn: &Connection,
    fingerprint: &str,
    direccion: Option<&str>,
    ahora: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE peers SET last_seen_at = ?1, last_address = COALESCE(?2, last_address) WHERE fingerprint = ?3",
        params![ahora, direccion, fingerprint],
    )?;
    Ok(())
}

/// Cuántos equipos conocidos hay guardados (el `peersCount` del snapshot de la interfaz).
pub fn contar_peers(conn: &Connection) -> Result<usize> {
    conn.query_row("SELECT COUNT(*) FROM peers", [], |fila| {
        fila.get::<_, i64>(0).map(|n| n as usize)
    })
}

pub fn get_peer_by_fingerprint(conn: &Connection, fingerprint: &str) -> Result<Option<Peer>> {
    let norm_fp = fingerprint.trim().to_lowercase();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, fingerprint, display_name, alias, is_trusted, auto_accept, last_seen_at, is_favorite, last_address
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
        let is_favorite: i64 = row.get(7)?;
        let last_address: Option<String> = row.get(8)?;

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
            addresses: last_address.into_iter().collect(),
            trust_state,
            auto_accept: auto_accept == 1,
            last_seen,
            favorite: is_favorite == 1,
            alias,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::migrations::MIGRATIONS;

    fn bd() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in MIGRATIONS {
            c.execute_batch(m.sql).unwrap();
        }
        c
    }

    fn equipo(direccion: Option<&str>) -> Peer {
        let mut p = Peer::new(
            Uuid::from_u128(1),
            "WIN11D".into(),
            "a".repeat(64),
            direccion.map(str::to_string).into_iter().collect(),
        )
        .unwrap();
        p.trust_state = TrustState::Trusted;
        p
    }

    #[test]
    fn la_direccion_guardada_vuelve_al_leer_el_equipo() {
        let c = bd();
        upsert_peer(&c, &equipo(Some("192.168.1.50:7411"))).unwrap();

        let leido = get_peer_by_fingerprint(&c, &"a".repeat(64))
            .unwrap()
            .unwrap();

        assert_eq!(leido.addresses, vec!["192.168.1.50:7411".to_string()]);
    }

    #[test]
    fn volver_a_guardar_sin_direccion_no_borra_la_conocida() {
        let c = bd();
        upsert_peer(&c, &equipo(Some("192.168.1.50:7411"))).unwrap();
        upsert_peer(&c, &equipo(None)).unwrap();

        let leido = get_peer_by_fingerprint(&c, &"a".repeat(64))
            .unwrap()
            .unwrap();

        assert_eq!(leido.addresses, vec!["192.168.1.50:7411".to_string()]);
    }

    #[test]
    fn un_avistamiento_actualiza_direccion_y_fecha() {
        let c = bd();
        upsert_peer(&c, &equipo(Some("192.168.1.50:7411"))).unwrap();

        registrar_avistamiento(
            &c,
            &"a".repeat(64),
            Some("192.168.1.77:7411"),
            "2026-09-30T12:00:00.000Z",
        )
        .unwrap();

        let g = peers_para_alcance(&c).unwrap();
        assert_eq!(g[0].last_address.as_deref(), Some("192.168.1.77:7411"));
        let visto: String = c
            .query_row("SELECT last_seen_at FROM peers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(visto, "2026-09-30T12:00:00.000Z");
    }

    #[test]
    fn un_avistamiento_sin_direccion_conserva_la_anterior() {
        let c = bd();
        upsert_peer(&c, &equipo(Some("192.168.1.50:7411"))).unwrap();

        registrar_avistamiento(&c, &"a".repeat(64), None, "2026-09-30T12:00:00.000Z").unwrap();

        assert_eq!(
            peers_para_alcance(&c).unwrap()[0].last_address.as_deref(),
            Some("192.168.1.50:7411")
        );
    }
}
