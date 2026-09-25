use crate::model::result::{DirectionResult, EngineResult, PeerSnapshot, SessionResult};

pub const REDACTED_IP_PLACEHOLDER: &str = "[REDACTED_IP]";
pub const REDACTED_MAC_PLACEHOLDER: &str = "[REDACTED_MAC]";
pub const REDACTED_FINGERPRINT_PLACEHOLDER: &str = "[REDACTED_FINGERPRINT]";
pub const RAW_OMITTED_PLACEHOLDER: &str = "[RAW_OUTPUT_OMITTED_FOR_PRIVACY]";

/// Comprueba si una cadena es una dirección IPv4 válida
pub fn is_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

/// Comprueba si una cadena es una dirección IPv6 (o link-local fe80::)
pub fn is_ipv6(s: &str) -> bool {
    s.parse::<std::net::Ipv6Addr>().is_ok()
        || (s.to_ascii_lowercase().starts_with("fe80::") && s.len() >= 8)
}

/// Comprueba si una cadena es una dirección MAC válida (XX:XX:XX:XX:XX:XX o XX-XX-XX-XX-XX-XX)
pub fn is_mac(s: &str) -> bool {
    if s.len() != 17 {
        return false;
    }
    let sep = s.as_bytes()[2];
    if sep != b':' && sep != b'-' {
        return false;
    }
    for (i, &b) in s.as_bytes().iter().enumerate() {
        if i == 2 || i == 5 || i == 8 || i == 11 || i == 14 {
            if b != sep {
                return false;
            }
        } else if !b.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

/// Comprueba si una cadena es una huella SHA-256 (64 dígitos hexadecimales)
pub fn is_fingerprint(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Reemplaza todas las apariciones de IPs, MACs y huellas en cualquier texto arbitrario
pub fn redact_all_identifiers(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Encontrar inicio del siguiente token alfanumérico o con caracteres de red (., :, -)
        if !chars[i].is_alphanumeric() && chars[i] != '.' && chars[i] != ':' && chars[i] != '-' {
            result.push(chars[i]);
            i += 1;
            continue;
        }

        let start = i;
        while i < chars.len()
            && (chars[i].is_alphanumeric() || chars[i] == '.' || chars[i] == ':' || chars[i] == '-')
        {
            i += 1;
        }
        let token: String = chars[start..i].iter().collect();

        // Limpiar puntuaciones al borde si no forman parte de la dirección
        let trimmed_token = token.trim_end_matches(['.', ',', ';']);
        let trailing = &token[trimmed_token.len()..];

        if is_fingerprint(trimmed_token) {
            result.push_str(REDACTED_FINGERPRINT_PLACEHOLDER);
            result.push_str(trailing);
        } else if is_mac(trimmed_token) {
            result.push_str(REDACTED_MAC_PLACEHOLDER);
            result.push_str(trailing);
        } else if is_ipv4(trimmed_token) || is_ipv6(trimmed_token) {
            result.push_str(REDACTED_IP_PLACEHOLDER);
            result.push_str(trailing);
        } else if let Some((ip_part, port_part)) = trimmed_token.split_once(':') {
            if is_ipv4(ip_part) && port_part.parse::<u16>().is_ok() {
                result.push_str(REDACTED_IP_PLACEHOLDER);
                result.push(':');
                result.push_str(port_part);
                result.push_str(trailing);
            } else {
                result.push_str(&token);
            }
        } else {
            result.push_str(&token);
        }
    }

    result
}

pub fn redact_ips(text: &str) -> String {
    redact_all_identifiers(text)
}

pub fn redact_macs(text: &str) -> String {
    redact_all_identifiers(text)
}

pub fn redact_fingerprints(text: &str) -> String {
    redact_all_identifiers(text)
}

/// Redacta un snapshot de peer
pub fn redact_peer(peer: &PeerSnapshot, redact_identifiers: bool) -> PeerSnapshot {
    if !redact_identifiers {
        return peer.clone();
    }

    let address = redact_all_identifiers(&peer.address);
    let fingerprint = REDACTED_FINGERPRINT_PLACEHOLDER.to_string();

    PeerSnapshot {
        instance_id: peer.instance_id,
        display_name: peer.display_name.clone(),
        fingerprint,
        address,
    }
}

/// Redacta un EngineResult, aplicando la política estricta de saneamiento u omisión
/// de bloques crudos no saneables según §20 y T106.
pub fn redact_engine_result(engine: &EngineResult, redact_identifiers: bool) -> EngineResult {
    if !redact_identifiers {
        return engine.clone();
    }

    let raw = engine.raw.as_ref().map(|raw_text| {
        let sanitized = redact_all_identifiers(raw_text);
        if sanitized.contains(REDACTED_IP_PLACEHOLDER)
            || sanitized.contains(REDACTED_MAC_PLACEHOLDER)
            || sanitized.contains(REDACTED_FINGERPRINT_PLACEHOLDER)
            || raw_text.contains("<ntttcp>")
            || raw_text.contains("ntttcp.exe")
        {
            RAW_OMITTED_PLACEHOLDER.to_string()
        } else {
            sanitized
        }
    });

    EngineResult {
        role: engine.role.clone(),
        total_bytes: engine.total_bytes.clone(),
        realtime_seconds: engine.realtime_seconds,
        throughput_bps: engine.throughput_bps.clone(),
        cpu_percent: engine.cpu_percent,
        buffers_count: engine.buffers_count,
        errors_count: engine.errors_count,
        packets_sent: engine.packets_sent,
        packets_received: engine.packets_received,
        packets_retransmitted: engine.packets_retransmitted,
        raw,
    }
}

/// Redacta un DirectionResult completo
pub fn redact_direction(dir: &DirectionResult, redact_identifiers: bool) -> DirectionResult {
    DirectionResult {
        direction: dir.direction.clone(),
        status: dir.status.clone(),
        sender: dir
            .sender
            .as_ref()
            .map(|s| redact_engine_result(s, redact_identifiers)),
        receiver: dir
            .receiver
            .as_ref()
            .map(|r| redact_engine_result(r, redact_identifiers)),
        official_bps: dir.official_bps.clone(),
        utilization: dir.utilization,
        stability: dir.stability.clone(),
        retransmission: dir.retransmission.clone(),
        cpu_sender: dir.cpu_sender,
        cpu_receiver: dir.cpu_receiver,
        samples_count: dir.samples_count,
        gaps_count: dir.gaps_count,
    }
}

/// Redacta un SessionResult completo aplicando las opciones solicitadas
pub fn redact_session_result(session: &SessionResult, redact_ips_and_macs: bool) -> SessionResult {
    if !redact_ips_and_macs {
        return session.clone();
    }

    let initiator = redact_peer(&session.initiator, true);
    let responder = redact_peer(&session.responder, true);
    let directions = session
        .directions
        .iter()
        .map(|d| redact_direction(d, true))
        .collect();

    SessionResult {
        schema_version: session.schema_version,
        session_id: session.session_id,
        started_at: session.started_at.clone(),
        finished_at: session.finished_at.clone(),
        status: session.status.clone(),
        initiator,
        responder,
        plan: session.plan.clone(),
        capacity: session.capacity.clone(),
        directions,
        asymmetry: session.asymmetry.clone(),
        verdict: session.verdict.clone(),
        result_source: session.result_source.clone(),
        versions: session.versions.clone(),
    }
}
