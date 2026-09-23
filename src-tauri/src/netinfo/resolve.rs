use std::net::{IpAddr, SocketAddr};
use unicode_normalization::UnicodeNormalization;

/// Normaliza y sanea un nombre de equipo remoto según FR-056:
/// - Normalización Unicode NFC
/// - Eliminación de caracteres de control
/// - Eliminación de caracteres bidireccionales (evitar spoofing bidi)
/// - Límite de 48 caracteres
/// - Escape de caracteres HTML sensibles (<, >, &, ", ')
pub fn sanitize_display_name(raw: &str) -> String {
    // 1. Normalización NFC
    let nfc: String = raw.nfc().collect();

    // 2. Filtrar caracteres de control y bidi
    let filtered: String = nfc
        .chars()
        .filter(|c| {
            !c.is_control()
                && !('\u{202A}'..='\u{202E}').contains(c)
                && !('\u{2066}'..='\u{2069}').contains(c)
        })
        .take(48)
        .collect();

    let trimmed = filtered.trim();
    if trimmed.is_empty() {
        return "Equipo-Remoto".to_string();
    }

    // 3. Escape HTML
    let mut escaped = String::with_capacity(trimmed.len());
    for c in trimmed.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(c),
        }
    }

    escaped
}

/// Resuelve una cadena "host:puerto" o "ip:puerto" a direcciones SocketAddr válidas
pub async fn resolve_target_address(host: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
    let clean_host = host.trim();
    if clean_host.is_empty() {
        return Err("La dirección de destino no puede estar vacía".to_string());
    }

    // Comprobar si es una IP directa (IPv4 o IPv6)
    if let Ok(ip) = clean_host.parse::<IpAddr>() {
        return Ok(vec![SocketAddr::new(ip, port)]);
    }

    // Resolución DNS asíncrona
    let target = format!("{}:{}", clean_host, port);
    match tokio::net::lookup_host(&target).await {
        Ok(addrs) => {
            let list: Vec<SocketAddr> = addrs.collect();
            if list.is_empty() {
                Err(format!("No se pudo resolver el host: {}", clean_host))
            } else {
                Ok(list)
            }
        }
        Err(e) => Err(format!("Fallo en la resolución de {}: {}", clean_host, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_display_name_rules() {
        // Normalización y longitud
        let long_name = "a".repeat(100);
        let sanitized = sanitize_display_name(&long_name);
        assert_eq!(sanitized.chars().count(), 48);

        // Control y bidi
        let bidi_name = "PC\u{202E}Inverso\u{0000}Test";
        let clean = sanitize_display_name(bidi_name);
        assert_eq!(clean, "PCInversoTest");

        // HTML escapado
        let html_name = "<script>alert('pwn')</script>";
        let escaped = sanitize_display_name(html_name);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(escaped.contains("&lt;script&gt;"));

        // Vacío o solo espacios
        assert_eq!(sanitize_display_name("   "), "Equipo-Remoto");
    }

    #[tokio::test]
    async fn test_resolve_ipv4_and_localhost() {
        let res_ip = resolve_target_address("127.0.0.1", 7411).await.unwrap();
        assert_eq!(res_ip[0].ip(), "127.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(res_ip[0].port(), 7411);

        let res_lh = resolve_target_address("localhost", 7412).await.unwrap();
        assert!(!res_lh.is_empty());
        assert_eq!(res_lh[0].port(), 7412);
    }
}
