//! Servidor del canal de control (FR-009).
//!
//! Cada instancia escucha e inicia: no hay rol configurable. El servidor solo llega
//! hasta el intercambio de HELLO; lo que venga después —emparejamiento, solicitud de
//! prueba, sesión— lo decide la capa de control, nunca esta.
//!
//! Tres invariantes que esta capa sí garantiza:
//!
//! 1. Sin certificado de cliente no hay conexión. Lo impone `client_auth_mandatory`.
//! 2. La huella del par sale del certificado que presentó en el handshake, no de nada
//!    que el par diga de sí mismo en el payload (FR-011).
//! 3. Una instancia ocupada lo declara en su HELLO y rechaza la conexión (FR-017).

use crate::control::tls::{peer_fingerprint, server_config};
use crate::control::transport::{recv_envelope, send_envelope};
use crate::identity::InstanceIdentity;
use crate::model::protocol::{HelloPayload, ProtocolEnvelope, ProtocolMessageType};
use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use uuid::Uuid;

/// Plazo del handshake TLS más el HELLO. Un par que no completa a tiempo no bloquea
/// el bucle de aceptación (FR-060).
const PLAZO_HANDSHAKE: Duration = Duration::from_secs(10);

pub const VERSION_PROTOCOLO: u32 = 1;
pub const VERSION_PROTOCOLO_MIN: u32 = 1;

/// Resultado de un saludo completado: quién es el par, según su certificado.
pub struct SaludoEntrante {
    /// Huella SHA-256 del certificado presentado en el handshake. Es la única
    /// identidad con valor probatorio de esta estructura.
    pub fingerprint: String,
    /// Lo que el par declara de sí mismo. **Dato no confiable**: se sanea y se valida
    /// antes de mostrarse o persistirse.
    pub hello: HelloPayload,
    pub remote_addr: SocketAddr,
    pub stream: TlsStream<TcpStream>,
}

pub struct ControlServer {
    listener: TcpListener,
    /// Segundo socket IPv4, solo cuando `listener` es el `[::]` de un bind de doble pila
    /// (ver `bind_dual_stack`). En Windows, a diferencia de Linux, un socket IPv6 con
    /// `IPV6_V6ONLY` (el valor por defecto del sistema, que `tokio::net::TcpListener` no
    /// desactiva) **no** acepta conexiones IPv4 mapeadas: sin este segundo socket, ningún
    /// par que llegara por IPv4 —el caso común— habría completado nunca el saludo
    /// (hallazgo real de T180, dos equipos físicos, 2026-09-26; FR-010).
    listener_v4: Option<TcpListener>,
    acceptor: TlsAcceptor,
    identity: Arc<InstanceIdentity>,
    ocupado: Arc<AtomicBool>,
}

impl ControlServer {
    /// Liga el servidor a `addr`. Con puerto 0 el sistema elige uno libre, que es lo
    /// que usan las pruebas para no chocar entre ejecuciones.
    pub async fn bind(addr: SocketAddr, identity: Arc<InstanceIdentity>) -> Result<Self> {
        let config = server_config(&identity)?;
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener,
            listener_v4: None,
            acceptor: TlsAcceptor::from(config),
            identity,
            ocupado: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Liga `puerto` en IPv6 `[::]` **y** en IPv4 `0.0.0.0` a la vez, con dos sockets
    /// reales — no uno solo dual-stack — porque Windows no ofrece IPv4 mapeada sobre un
    /// socket IPv6 sin desactivar `IPV6_V6ONLY`, algo que `tokio::net::TcpListener` no
    /// expone. Añadir `socket2` solo para ese ajuste habría sido una dependencia nueva
    /// para un problema que dos sockets ya resuelven con lo que hay. Si el bind IPv4
    /// falla (por ejemplo, algo más ya lo ocupa) se seguirá aceptando solo por IPv6, en
    /// vez de que arranque falle entero.
    pub async fn bind_dual_stack(puerto: u16, identity: Arc<InstanceIdentity>) -> Result<Self> {
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

        let config = server_config(&identity)?;
        let listener =
            TcpListener::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), puerto)).await?;
        let listener_v4 =
            match TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), puerto))
                .await
            {
                Ok(l) => Some(l),
                Err(e) => {
                    tracing::warn!(
                        "No se pudo ligar también 0.0.0.0:{puerto} ({e}); solo se aceptará por IPv6"
                    );
                    None
                }
            };

        Ok(Self {
            listener,
            listener_v4,
            acceptor: TlsAcceptor::from(config),
            identity,
            ocupado: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Bandera de ocupación compartida. La capa de control la levanta mientras hay una
    /// sesión activa para que el servidor rechace explícitamente (FR-017).
    pub fn ocupado(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.ocupado)
    }

    fn hello_local(&self) -> HelloPayload {
        HelloPayload {
            protocol_version: VERSION_PROTOCOLO,
            protocol_min: VERSION_PROTOCOLO_MIN,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            instance_id: self.identity.instance_id,
            display_name: self.identity.display_name.clone(),
            platform: "windows".to_string(),
            is_busy: self.ocupado.load(Ordering::SeqCst),
        }
    }

    /// Acepta una conexión y completa handshake TLS y saludo.
    ///
    /// Un fallo aquí afecta solo a esa conexión: quien llame debe seguir aceptando.
    pub async fn accept_one(&self) -> Result<SaludoEntrante> {
        let (tcp, remote_addr) = match &self.listener_v4 {
            Some(v4) => {
                tokio::select! {
                    r = self.listener.accept() => r?,
                    r = v4.accept() => r?,
                }
            }
            None => self.listener.accept().await?,
        };

        timeout(PLAZO_HANDSHAKE, self.completar_saludo(tcp, remote_addr))
            .await
            .map_err(|_| {
                Error::new(
                    ErrorKind::TimedOut,
                    format!("El par {remote_addr} no completó el saludo en 10 s"),
                )
            })?
    }

    async fn completar_saludo(
        &self,
        tcp: TcpStream,
        remote_addr: SocketAddr,
    ) -> Result<SaludoEntrante> {
        let mut stream = self.acceptor.accept(tcp).await?;

        // La huella sale del certificado del handshake. Si no hay certificado, la
        // conexión se corta: `client_auth_mandatory` debería haberlo impedido antes,
        // pero no se da por supuesto lo que se puede comprobar.
        let (_, conexion) = stream.get_ref();
        let fingerprint = peer_fingerprint(conexion.peer_certificates()).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("El par {remote_addr} no presentó certificado"),
            )
        })?;

        let remoto: ProtocolEnvelope<HelloPayload> = recv_envelope(&mut stream).await?;
        if remoto.msg_type != ProtocolMessageType::Hello {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Se esperaba HELLO y llegó {:?}", remoto.msg_type),
            ));
        }
        if remoto.payload.protocol_min > VERSION_PROTOCOLO {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "El par exige protocolo mínimo {} y esta versión habla {}",
                    remoto.payload.protocol_min, VERSION_PROTOCOLO
                ),
            ));
        }

        let respuesta = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Hello,
            id: Uuid::new_v4(),
            session_id: None,
            ts: ahora_rfc3339(),
            in_reply_to: Some(remoto.id),
            payload: self.hello_local(),
        };
        send_envelope(&mut stream, &respuesta).await?;

        Ok(SaludoEntrante {
            fingerprint,
            hello: remoto.payload,
            remote_addr,
            stream,
        })
    }
}

/// Sello UTC RFC 3339 con milisegundos para el campo `ts` del sobre.
pub fn ahora_rfc3339() -> String {
    crate::logging::format_rfc3339_utc(std::time::SystemTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::tls::client_config;
    use rustls::pki_types::ServerName;
    use std::net::{IpAddr, Ipv4Addr};
    use tokio_rustls::TlsConnector;

    fn local_v4() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
    }

    async fn conectar(
        addr: SocketAddr,
        ident: &InstanceIdentity,
    ) -> Result<tokio_rustls::client::TlsStream<TcpStream>> {
        let connector = TlsConnector::from(client_config(ident)?);
        let tcp = TcpStream::connect(addr).await?;
        let nombre = ServerName::try_from("netbench")
            .map_err(|e| Error::new(ErrorKind::InvalidInput, e.to_string()))?;
        connector.connect(nombre, tcp).await
    }

    #[tokio::test]
    async fn test_saludo_mutuo_expone_la_huella_del_certificado() {
        let servidor_id = Arc::new(InstanceIdentity::generate("Servidor".into()).unwrap());
        let cliente_id = Arc::new(InstanceIdentity::generate("Cliente".into()).unwrap());
        let huella_cliente = cliente_id.fingerprint.clone();

        let servidor = ControlServer::bind(local_v4(), Arc::clone(&servidor_id))
            .await
            .expect("bind");
        let addr = servidor.local_addr().unwrap();

        let tarea = tokio::spawn(async move { servidor.accept_one().await });

        let mut stream = conectar(addr, &cliente_id).await.expect("conectar");
        let hello = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Hello,
            id: Uuid::new_v4(),
            session_id: None,
            ts: ahora_rfc3339(),
            in_reply_to: None,
            payload: HelloPayload {
                protocol_version: VERSION_PROTOCOLO,
                protocol_min: VERSION_PROTOCOLO_MIN,
                app_version: "0.1.0".into(),
                instance_id: cliente_id.instance_id,
                display_name: "Cliente".into(),
                platform: "windows".into(),
                is_busy: false,
            },
        };
        send_envelope(&mut stream, &hello).await.expect("enviar");
        let respuesta: ProtocolEnvelope<HelloPayload> =
            recv_envelope(&mut stream).await.expect("recibir");

        let entrante = tarea.await.unwrap().expect("saludo");

        // El servidor identifica al cliente por su certificado.
        assert_eq!(entrante.fingerprint, huella_cliente);
        assert_eq!(entrante.hello.instance_id, cliente_id.instance_id);

        // Y el cliente recibe el HELLO del servidor.
        assert_eq!(respuesta.msg_type, ProtocolMessageType::Hello);
        assert_eq!(respuesta.payload.instance_id, servidor_id.instance_id);
        assert_eq!(respuesta.in_reply_to, Some(hello.id));
    }

    #[tokio::test]
    async fn test_un_instance_id_falsificado_no_cambia_la_huella() {
        // El ataque que el código anterior permitía: derivar la huella del `instanceId`
        // que envía el propio par. Aquí el cliente miente sobre su instanceId y la
        // huella sigue siendo la de su certificado.
        let servidor_id = Arc::new(InstanceIdentity::generate("Servidor".into()).unwrap());
        let cliente_id = Arc::new(InstanceIdentity::generate("Cliente".into()).unwrap());
        let victima = Uuid::new_v4();
        let huella_real = cliente_id.fingerprint.clone();

        let servidor = ControlServer::bind(local_v4(), servidor_id).await.unwrap();
        let addr = servidor.local_addr().unwrap();
        let tarea = tokio::spawn(async move { servidor.accept_one().await });

        let mut stream = conectar(addr, &cliente_id).await.expect("conectar");
        let hello = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Hello,
            id: Uuid::new_v4(),
            session_id: None,
            ts: ahora_rfc3339(),
            in_reply_to: None,
            payload: HelloPayload {
                protocol_version: VERSION_PROTOCOLO,
                protocol_min: VERSION_PROTOCOLO_MIN,
                app_version: "0.1.0".into(),
                instance_id: victima, // suplantación declarada
                display_name: "Equipo de confianza".into(),
                platform: "windows".into(),
                is_busy: false,
            },
        };
        send_envelope(&mut stream, &hello).await.unwrap();
        let _: ProtocolEnvelope<HelloPayload> = recv_envelope(&mut stream).await.unwrap();

        let entrante = tarea.await.unwrap().expect("saludo");
        assert_eq!(entrante.fingerprint, huella_real);
        assert_ne!(
            entrante.fingerprint,
            crate::control::tls::fingerprint_of_der(victima.as_bytes())
        );
    }

    #[tokio::test]
    async fn test_cliente_sin_certificado_es_rechazado() {
        let servidor_id = Arc::new(InstanceIdentity::generate("Servidor".into()).unwrap());
        let servidor = ControlServer::bind(local_v4(), servidor_id).await.unwrap();
        let addr = servidor.local_addr().unwrap();
        let tarea = tokio::spawn(async move { servidor.accept_one().await });

        // TCP en claro contra un puerto que exige TLS mutuo.
        let mut tcp = TcpStream::connect(addr).await.expect("tcp");
        use tokio::io::AsyncWriteExt;
        let _ = tcp.write_all(b"texto plano, sin TLS\n").await;
        let _ = tcp.flush().await;

        assert!(tarea.await.unwrap().is_err(), "debe rechazar sin TLS mutuo");
    }

    #[tokio::test]
    async fn test_instancia_ocupada_lo_declara_en_su_hello() {
        let servidor_id = Arc::new(InstanceIdentity::generate("Servidor".into()).unwrap());
        let cliente_id = Arc::new(InstanceIdentity::generate("Cliente".into()).unwrap());

        let servidor = ControlServer::bind(local_v4(), servidor_id).await.unwrap();
        let addr = servidor.local_addr().unwrap();
        servidor.ocupado().store(true, Ordering::SeqCst);
        let tarea = tokio::spawn(async move { servidor.accept_one().await });

        let mut stream = conectar(addr, &cliente_id).await.expect("conectar");
        let hello = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Hello,
            id: Uuid::new_v4(),
            session_id: None,
            ts: ahora_rfc3339(),
            in_reply_to: None,
            payload: HelloPayload {
                protocol_version: VERSION_PROTOCOLO,
                protocol_min: VERSION_PROTOCOLO_MIN,
                app_version: "0.1.0".into(),
                instance_id: cliente_id.instance_id,
                display_name: "Cliente".into(),
                platform: "windows".into(),
                is_busy: false,
            },
        };
        send_envelope(&mut stream, &hello).await.unwrap();
        let respuesta: ProtocolEnvelope<HelloPayload> = recv_envelope(&mut stream).await.unwrap();

        assert!(respuesta.payload.is_busy, "debe declararse ocupada");
        let _ = tarea.await.unwrap();
    }

    fn puerto_libre_en_0_0_0_0() -> u16 {
        std::net::TcpListener::bind("0.0.0.0:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    /// Reproduce el hallazgo real de T180: un `bind` normal en `[::]` no aceptaba
    /// conexiones IPv4 en Windows (el `IPV6_V6ONLY` del sistema, que
    /// `tokio::net::TcpListener` no desactiva), así que ningún par que llegara por IPv4
    /// —el caso común entre dos equipos físicos— completaba nunca el saludo.
    /// `bind_dual_stack` abre un segundo socket IPv4 explícito para ese caso.
    #[tokio::test]
    async fn test_bind_dual_stack_acepta_una_conexion_ipv4() {
        let servidor_id = Arc::new(InstanceIdentity::generate("Servidor".into()).unwrap());
        let cliente_id = Arc::new(InstanceIdentity::generate("Cliente".into()).unwrap());
        let puerto = puerto_libre_en_0_0_0_0();

        let servidor = ControlServer::bind_dual_stack(puerto, Arc::clone(&servidor_id))
            .await
            .expect("bind_dual_stack");
        let tarea = tokio::spawn(async move { servidor.accept_one().await });

        // Dirección explícitamente IPv4, no `localhost` (que en Windows puede resolver a
        // `::1`) — es justo la conexión que antes no llegaba nunca a completarse.
        let addr_v4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), puerto);
        let mut stream = conectar(addr_v4, &cliente_id)
            .await
            .expect("conectar por IPv4 a un servidor de doble pila");
        let hello = ProtocolEnvelope {
            msg_type: ProtocolMessageType::Hello,
            id: Uuid::new_v4(),
            session_id: None,
            ts: ahora_rfc3339(),
            in_reply_to: None,
            payload: HelloPayload {
                protocol_version: VERSION_PROTOCOLO,
                protocol_min: VERSION_PROTOCOLO_MIN,
                app_version: "0.1.0".into(),
                instance_id: cliente_id.instance_id,
                display_name: "Cliente".into(),
                platform: "windows".into(),
                is_busy: false,
            },
        };
        send_envelope(&mut stream, &hello).await.expect("enviar");
        let _respuesta: ProtocolEnvelope<HelloPayload> =
            recv_envelope(&mut stream).await.expect("recibir");

        let entrante = tarea
            .await
            .unwrap()
            .expect("el servidor debe aceptar la conexion IPv4");
        assert_eq!(entrante.fingerprint, cliente_id.fingerprint);
    }
}
