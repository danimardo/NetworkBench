use crate::control::transport::{recv_envelope, send_envelope};
use crate::identity::PublicIdentity;
use crate::model::peer::Peer;
use crate::model::protocol::{HelloPayload, ProtocolEnvelope, ProtocolMessageType};
use crate::netinfo::resolve::{resolve_target_address, sanitize_display_name};
use sha2::Digest;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

pub const CONTROL_PORT_DEFAULT: u16 = 7411;
pub const MDNS_SERVICE_TYPE: &str = "_netbench._tcp.local.";

/// Realiza una conexión manual a un peer remoto vía IP o DNS, ejecuta el handshake HELLO y obtiene su identidad.
pub async fn manual_connect_peer(
    host: &str,
    port: u16,
    local_identity: &PublicIdentity,
) -> Result<Peer, String> {
    let addrs = resolve_target_address(host, port).await?;
    let target_addr = addrs
        .first()
        .ok_or_else(|| "No se encontró ninguna dirección IP para el host".to_string())?;

    // Intentar conectar con timeout de 5 segundos
    let mut stream = match timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(target_addr),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return Err(format!("No se pudo conectar a {}: {}", target_addr, e)),
        Err(_) => {
            return Err("Tiempo de espera agotado al conectar al equipo remoto (5 s)".to_string());
        }
    };

    let (mut reader, mut writer) = stream.split();

    // 1. Enviar HELLO de la instancia local
    let hello = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Hello,
        id: Uuid::new_v4(),
        session_id: None,
        ts: "2026-09-21T20:00:00Z".to_string(),
        in_reply_to: None,
        payload: HelloPayload {
            protocol_version: 1,
            protocol_min: 1,
            app_version: "0.1.0".to_string(),
            instance_id: local_identity.instance_id,
            display_name: local_identity.display_name.clone(),
            platform: "windows".to_string(),
            is_busy: false,
        },
    };

    send_envelope(&mut writer, &hello)
        .await
        .map_err(|e| format!("Error enviando HELLO: {}", e))?;

    // 2. Recibir HELLO del peer remoto
    let remote_hello: ProtocolEnvelope<HelloPayload> =
        timeout(Duration::from_secs(5), recv_envelope(&mut reader))
            .await
            .map_err(|_| "Tiempo de espera agotado esperando HELLO del equipo remoto".to_string())?
            .map_err(|e| format!("Error recibiendo HELLO: {}", e))?;

    if remote_hello.msg_type != ProtocolMessageType::Hello {
        return Err(format!(
            "Mensaje inesperado en saludo inicial: {:?}",
            remote_hello.msg_type
        ));
    }

    // 3. Sanitizar nombre remoto (FR-056)
    let clean_name = sanitize_display_name(&remote_hello.payload.display_name);

    // 4. Construir Peer
    let hash = sha2::Sha256::digest(remote_hello.payload.instance_id.as_bytes());
    let fingerprint = hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let peer = Peer::new(
        remote_hello.payload.instance_id,
        clean_name,
        fingerprint,
        vec![target_addr.to_string()],
    )?;

    Ok(peer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_manual_connect_handshake() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let remote_id = Uuid::new_v4();

        // Tarea del peer remoto simulado
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let (mut reader, mut writer) = socket.split();

            let _req: ProtocolEnvelope<HelloPayload> = recv_envelope(&mut reader).await.unwrap();

            let resp = ProtocolEnvelope {
                msg_type: ProtocolMessageType::Hello,
                id: Uuid::new_v4(),
                session_id: None,
                ts: "2026-09-21T20:00:00Z".to_string(),
                in_reply_to: None,
                payload: HelloPayload {
                    protocol_version: 1,
                    protocol_min: 1,
                    app_version: "0.1.0".to_string(),
                    instance_id: remote_id,
                    display_name: "<script>PC-Remoto</script>".to_string(),
                    platform: "windows".to_string(),
                    is_busy: false,
                },
            };
            send_envelope(&mut writer, &resp).await.unwrap();
        });

        let local_ident = PublicIdentity {
            instance_id: Uuid::new_v4(),
            display_name: "Local-PC".to_string(),
            fingerprint: "1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
            cert_pem: "".to_string(),
        };

        let peer = manual_connect_peer("127.0.0.1", port, &local_ident)
            .await
            .expect("Conexión manual");

        assert_eq!(peer.instance_id, remote_id);
        // Debe haberse sanitizado el HTML
        assert_eq!(peer.display_name, "&lt;script&gt;PC-Remoto&lt;/script&gt;");
    }
}
