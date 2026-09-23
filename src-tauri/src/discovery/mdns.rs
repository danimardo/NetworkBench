use crate::control::server::{VERSION_PROTOCOLO, VERSION_PROTOCOLO_MIN, ahora_rfc3339};
use crate::control::tls::{client_config, peer_fingerprint};
use crate::control::transport::{recv_envelope, send_envelope};
use crate::identity::InstanceIdentity;
use crate::model::peer::Peer;
use crate::model::protocol::{HelloPayload, ProtocolEnvelope, ProtocolMessageType};
use crate::netinfo::resolve::{resolve_target_address, sanitize_display_name};
use rustls::pki_types::ServerName;
use std::time::Duration;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use uuid::Uuid;

pub const CONTROL_PORT_DEFAULT: u16 = 7411;
pub const MDNS_SERVICE_TYPE: &str = "_netbench._tcp.local.";

/// Nombre presentado en el SNI. No identifica a nadie ni se valida: en este protocolo
/// la identidad la da la huella del certificado, no el nombre de host (FR-011).
const SNI: &str = "netbench";

const PLAZO_CONEXION: Duration = Duration::from_secs(5);

/// Conecta manualmente con un peer por IP o DNS y completa TLS mutuo más el saludo.
///
/// La huella del `Peer` devuelto procede **del certificado presentado en el handshake**,
/// no del `instanceId` que el remoto declara. Es la diferencia entre una identidad que
/// se demuestra poseyendo una clave privada y una que basta con afirmar.
pub async fn manual_connect_peer(
    host: &str,
    port: u16,
    local_identity: &InstanceIdentity,
) -> Result<Peer, String> {
    let addrs = resolve_target_address(host, port).await?;
    let target_addr = *addrs
        .first()
        .ok_or_else(|| "No se encontró ninguna dirección IP para el host".to_string())?;

    let connector = TlsConnector::from(
        client_config(local_identity).map_err(|e| format!("Error preparando TLS: {e}"))?,
    );

    let tcp = match timeout(PLAZO_CONEXION, tokio::net::TcpStream::connect(target_addr)).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return Err(format!("No se pudo conectar a {target_addr}: {e}")),
        Err(_) => {
            return Err("Tiempo de espera agotado al conectar al equipo remoto (5 s)".to_string());
        }
    };

    let sni = ServerName::try_from(SNI).map_err(|e| format!("SNI no válido: {e}"))?;
    let mut stream = match timeout(PLAZO_CONEXION, connector.connect(sni, tcp)).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return Err(format!("El canal seguro con {target_addr} falló: {e}")),
        Err(_) => return Err("Tiempo de espera agotado en el canal seguro (5 s)".to_string()),
    };

    // Huella del certificado del servidor. Sin certificado no se sigue: un extremo
    // sin identidad criptográfica no puede convertirse en un peer conocido.
    let fingerprint = {
        let (_, conexion) = stream.get_ref();
        peer_fingerprint(conexion.peer_certificates())
            .ok_or_else(|| format!("El equipo {target_addr} no presentó certificado"))?
    };

    let hello = ProtocolEnvelope {
        msg_type: ProtocolMessageType::Hello,
        id: Uuid::new_v4(),
        session_id: None,
        ts: ahora_rfc3339(),
        in_reply_to: None,
        payload: HelloPayload {
            protocol_version: VERSION_PROTOCOLO,
            protocol_min: VERSION_PROTOCOLO_MIN,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            instance_id: local_identity.instance_id,
            display_name: local_identity.display_name.clone(),
            platform: "windows".to_string(),
            is_busy: false,
        },
    };

    send_envelope(&mut stream, &hello)
        .await
        .map_err(|e| format!("Error enviando HELLO: {e}"))?;

    let remote_hello: ProtocolEnvelope<HelloPayload> =
        match timeout(PLAZO_CONEXION, recv_envelope(&mut stream)).await {
            Ok(Ok(env)) => env,
            Ok(Err(e)) => return Err(format!("Error recibiendo HELLO: {e}")),
            Err(_) => {
                return Err(
                    "Tiempo de espera agotado esperando HELLO del equipo remoto".to_string()
                );
            }
        };

    if remote_hello.msg_type != ProtocolMessageType::Hello {
        return Err(format!(
            "Mensaje inesperado en saludo inicial: {:?}",
            remote_hello.msg_type
        ));
    }

    // Texto remoto: normalizado, acotado y no ejecutable (FR-056).
    let clean_name = sanitize_display_name(&remote_hello.payload.display_name);

    Peer::new(
        remote_hello.payload.instance_id,
        clean_name,
        fingerprint,
        vec![target_addr.to_string()],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::server::ControlServer;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_conexion_manual_toma_la_huella_del_certificado() {
        let remoto =
            Arc::new(InstanceIdentity::generate("<script>PC-Remoto</script>".into()).unwrap());
        let local = InstanceIdentity::generate("Local-PC".into()).unwrap();
        let huella_remota = remoto.fingerprint.clone();
        let id_remoto = remoto.instance_id;

        let servidor = ControlServer::bind(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            Arc::clone(&remoto),
        )
        .await
        .unwrap();
        let port = servidor.local_addr().unwrap().port();
        tokio::spawn(async move { servidor.accept_one().await });

        let peer = manual_connect_peer("127.0.0.1", port, &local)
            .await
            .expect("conexión manual");

        assert_eq!(peer.instance_id, id_remoto);
        // La huella es la del certificado del servidor, no un hash del instanceId.
        assert_eq!(peer.fingerprint, huella_remota);
        // Y el nombre remoto llega saneado (FR-056).
        assert_eq!(peer.display_name, "&lt;script&gt;PC-Remoto&lt;/script&gt;");
    }

    #[tokio::test]
    async fn test_un_extremo_sin_tls_no_produce_peer() {
        let local = InstanceIdentity::generate("Local-PC".into()).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Servidor TCP en claro: acepta y no habla TLS.
        tokio::spawn(async move {
            let _ = listener.accept().await;
            tokio::time::sleep(Duration::from_secs(2)).await;
        });

        let resultado = manual_connect_peer("127.0.0.1", port, &local).await;
        assert!(resultado.is_err(), "no debe producir un peer sin TLS");
    }
}
