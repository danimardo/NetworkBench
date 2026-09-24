//! Emparejamiento sobre el canal de control (FR-012, FR-013).
//!
//! El código de seis dígitos **no viaja como secreto**: ambos extremos lo derivan por
//! su cuenta de las dos huellas y del identificador de emparejamiento. Que coincida
//! solo demuestra que los dos calcularon sobre las mismas huellas. Lo que establece la
//! confianza es que **una persona compare los dos códigos en las dos pantallas**.
//!
//! De ahí el reparto: esta capa verifica la coherencia criptográfica y deja la decisión
//! fuera. Un `PairResult` con `accepted: true` significa «el usuario del otro extremo
//! dijo que sí», nunca «el protocolo lo aprobó».

use crate::control::server::ahora_rfc3339;
use crate::control::transport::{recv_envelope, send_envelope};
use crate::model::protocol::{
    PairRequestPayload, PairResultPayload, ProtocolEnvelope, ProtocolMessageType,
};
use crate::pairing::{ActivePairing, PairingError, derive_pairing_code};
use std::io::{Error, ErrorKind, Result};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use uuid::Uuid;

/// Plazo para que la persona compare los códigos y decida. Coincide con la caducidad
/// del código (`PAIRING_CODE_TTL`): pasado ese tiempo el emparejamiento se repite.
const PLAZO_DECISION: Duration = Duration::from_secs(60);

/// Emparejamiento en curso, con el código que hay que enseñar al usuario.
pub struct EmparejamientoEnCurso {
    pub id: Uuid,
    /// Los seis dígitos que esta pantalla debe mostrar.
    pub codigo: String,
    pub huella_remota: String,
    activo: ActivePairing,
}

impl EmparejamientoEnCurso {
    pub fn iniciar(huella_local: &str, huella_remota: &str) -> Result<Self> {
        let id = Uuid::new_v4();
        Self::con_id(id, huella_local, huella_remota)
    }

    /// El extremo que responde reutiliza el identificador del que inicia: sin eso, cada
    /// lado derivaría un código distinto y la comparación humana nunca cuadraría.
    pub fn con_id(id: Uuid, huella_local: &str, huella_remota: &str) -> Result<Self> {
        let codigo = derive_pairing_code(huella_local, huella_remota, &id).map_err(a_io)?;
        let activo = ActivePairing::new(id, huella_local, huella_remota).map_err(a_io)?;
        Ok(Self {
            id,
            codigo,
            huella_remota: huella_remota.to_lowercase(),
            activo,
        })
    }

    /// Comprueba lo que llega del otro extremo antes de enseñar nada al usuario.
    pub fn verificar(&self, codigo_remoto: &str, huella_remota: &str) -> Result<()> {
        self.activo
            .verify(codigo_remoto, huella_remota, Instant::now())
            .map_err(a_io)
    }
}

fn a_io(e: PairingError) -> Error {
    let (clase, texto) = match e {
        PairingError::CodeMismatch => (
            ErrorKind::PermissionDenied,
            "Los códigos de verificación no coinciden".to_string(),
        ),
        PairingError::Expired => (
            ErrorKind::TimedOut,
            "El código de verificación caducó; repite el emparejamiento".to_string(),
        ),
        PairingError::FingerprintMismatch { expected, actual } => (
            ErrorKind::PermissionDenied,
            format!(
                "La identidad del equipo cambió: se esperaba {}… y llegó {}…",
                &expected[..expected.len().min(8)],
                &actual[..actual.len().min(8)]
            ),
        ),
        PairingError::InvalidFormat => (
            ErrorKind::InvalidData,
            "Formato de código o huella no válido".to_string(),
        ),
    };
    Error::new(clase, texto)
}

/// Lado que inicia: envía la solicitud y espera la decisión del otro usuario.
///
/// `decision_local` es lo que la persona de este lado respondió tras comparar. Si aquí
/// se rechaza, no se envía nada que pueda interpretarse como aceptación.
pub async fn solicitar_emparejamiento<S>(
    stream: &mut S,
    emparejamiento: &EmparejamientoEnCurso,
    decision_local: bool,
) -> Result<bool>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if !decision_local {
        return Ok(false);
    }

    let solicitud = ProtocolEnvelope {
        msg_type: ProtocolMessageType::PairRequest,
        id: emparejamiento.id,
        session_id: None,
        ts: ahora_rfc3339(),
        in_reply_to: None,
        payload: PairRequestPayload {
            pairing_code: emparejamiento.codigo.clone(),
        },
    };
    send_envelope(stream, &solicitud).await?;

    let respuesta: ProtocolEnvelope<PairResultPayload> =
        timeout(PLAZO_DECISION, recv_envelope(stream))
            .await
            .map_err(|_| {
                Error::new(
                    ErrorKind::TimedOut,
                    "El otro equipo no respondió al emparejamiento en 60 s",
                )
            })??;

    if respuesta.msg_type != ProtocolMessageType::PairResult {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Se esperaba PAIR_RESULT y llegó {:?}", respuesta.msg_type),
        ));
    }

    Ok(respuesta.payload.accepted)
}

/// Lado que responde: recibe la solicitud, la verifica y contesta con la decisión local.
///
/// `decidir` recibe el código ya verificado para que la interfaz lo muestre y devuelve
/// lo que la persona eligió. Que el código cuadre **no** acepta nada por sí solo.
pub async fn atender_emparejamiento<S, F>(
    stream: &mut S,
    huella_local: &str,
    huella_remota: &str,
    decidir: F,
) -> Result<(bool, EmparejamientoEnCurso)>
where
    S: AsyncRead + AsyncWrite + Unpin,
    F: FnOnce(&str) -> bool,
{
    let solicitud: ProtocolEnvelope<PairRequestPayload> =
        timeout(PLAZO_DECISION, recv_envelope(stream))
            .await
            .map_err(|_| {
                Error::new(
                    ErrorKind::TimedOut,
                    "No llegó ninguna solicitud de emparejamiento en 60 s",
                )
            })??;

    if solicitud.msg_type != ProtocolMessageType::PairRequest {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Se esperaba PAIR_REQUEST y llegó {:?}", solicitud.msg_type),
        ));
    }

    // El identificador lo fija quien inicia; sin reutilizarlo, los códigos no cuadran.
    let emparejamiento = EmparejamientoEnCurso::con_id(solicitud.id, huella_local, huella_remota)?;

    let verificacion = emparejamiento.verificar(&solicitud.payload.pairing_code, huella_remota);
    let aceptado = match &verificacion {
        Ok(()) => decidir(&emparejamiento.codigo),
        Err(_) => false,
    };

    let respuesta = ProtocolEnvelope {
        msg_type: ProtocolMessageType::PairResult,
        id: Uuid::new_v4(),
        session_id: None,
        ts: ahora_rfc3339(),
        in_reply_to: Some(solicitud.id),
        payload: PairResultPayload {
            accepted: aceptado,
            reason: match &verificacion {
                Ok(()) if aceptado => None,
                Ok(()) => Some("rejectedByUser".to_string()),
                // Motivo genérico a propósito: distinguir «código incorrecto» de
                // «huella cambiada» ante el remoto le diría a un atacante en cuál de
                // los dos se equivocó.
                Err(_) => Some("verificationFailed".to_string()),
            },
        },
    };
    send_envelope(stream, &respuesta).await?;

    verificacion?;
    Ok((aceptado, emparejamiento))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::InstanceIdentity;
    use tokio::io::duplex;

    fn dos_identidades() -> (InstanceIdentity, InstanceIdentity) {
        (
            InstanceIdentity::generate("A".into()).unwrap(),
            InstanceIdentity::generate("B".into()).unwrap(),
        )
    }

    #[test]
    fn test_ambos_extremos_derivan_el_mismo_codigo() {
        let (a, b) = dos_identidades();
        let id = Uuid::new_v4();

        let en_a = EmparejamientoEnCurso::con_id(id, &a.fingerprint, &b.fingerprint).unwrap();
        let en_b = EmparejamientoEnCurso::con_id(id, &b.fingerprint, &a.fingerprint).unwrap();

        assert_eq!(
            en_a.codigo, en_b.codigo,
            "la comparación humana exige el mismo código"
        );
        assert_eq!(en_a.codigo.len(), 6);
        assert!(en_a.codigo.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_un_tercero_no_obtiene_el_mismo_codigo() {
        let (a, b) = dos_identidades();
        let intruso = InstanceIdentity::generate("Intruso".into()).unwrap();
        let id = Uuid::new_v4();

        let legitimo = EmparejamientoEnCurso::con_id(id, &a.fingerprint, &b.fingerprint).unwrap();
        let suplantado =
            EmparejamientoEnCurso::con_id(id, &a.fingerprint, &intruso.fingerprint).unwrap();

        assert_ne!(legitimo.codigo, suplantado.codigo);
    }

    #[tokio::test]
    async fn test_emparejamiento_aceptado_por_ambas_personas() {
        let (a, b) = dos_identidades();
        let (mut canal_a, mut canal_b) = duplex(8192);

        let fp_b = b.fingerprint.clone();
        let fp_a = a.fingerprint.clone();
        let responde = tokio::spawn(async move {
            atender_emparejamiento(&mut canal_b, &fp_b, &fp_a, |_codigo| true).await
        });

        let inicia = EmparejamientoEnCurso::iniciar(&a.fingerprint, &b.fingerprint).unwrap();
        let aceptado = solicitar_emparejamiento(&mut canal_a, &inicia, true)
            .await
            .expect("solicitud");

        let (aceptado_remoto, remoto) = responde.await.unwrap().expect("respuesta");
        assert!(aceptado);
        assert!(aceptado_remoto);
        assert_eq!(inicia.codigo, remoto.codigo);
    }

    #[tokio::test]
    async fn test_si_la_persona_del_otro_lado_rechaza_no_hay_confianza() {
        let (a, b) = dos_identidades();
        let (mut canal_a, mut canal_b) = duplex(8192);

        let fp_b = b.fingerprint.clone();
        let fp_a = a.fingerprint.clone();
        let responde = tokio::spawn(async move {
            atender_emparejamiento(&mut canal_b, &fp_b, &fp_a, |_codigo| false).await
        });

        let inicia = EmparejamientoEnCurso::iniciar(&a.fingerprint, &b.fingerprint).unwrap();
        let aceptado = solicitar_emparejamiento(&mut canal_a, &inicia, true)
            .await
            .expect("solicitud");

        let (aceptado_remoto, _) = responde.await.unwrap().expect("respuesta");
        assert!(!aceptado, "un código correcto no acepta por sí solo");
        assert!(!aceptado_remoto);
    }

    #[tokio::test]
    async fn test_una_huella_distinta_invalida_el_emparejamiento() {
        // FR-013: si la identidad del otro extremo no es la esperada, el emparejamiento
        // falla aunque quien responde diría que sí.
        let (a, b) = dos_identidades();
        let intruso = InstanceIdentity::generate("Intruso".into()).unwrap();
        let (mut canal_a, mut canal_b) = duplex(8192);

        let fp_b = b.fingerprint.clone();
        let fp_intruso = intruso.fingerprint.clone();
        let responde = tokio::spawn(async move {
            // B cree hablar con el intruso; A le manda el código de A.
            atender_emparejamiento(&mut canal_b, &fp_b, &fp_intruso, |_c| true).await
        });

        let inicia = EmparejamientoEnCurso::iniciar(&a.fingerprint, &b.fingerprint).unwrap();
        let aceptado = solicitar_emparejamiento(&mut canal_a, &inicia, true)
            .await
            .expect("solicitud");

        assert!(
            !aceptado,
            "no debe aceptarse con una huella que no corresponde"
        );
        assert!(responde.await.unwrap().is_err());
    }

    #[tokio::test]
    async fn test_emparejamiento_completo_sobre_el_canal_tls_real() {
        // Los otros casos usan un dúplex en memoria. Este atraviesa el servidor real:
        // TLS mutuo, HELLO y después el emparejamiento sobre la misma conexión.
        use crate::control::server::ControlServer;
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        use std::sync::Arc;

        let servidor_id = Arc::new(InstanceIdentity::generate("Servidor".into()).unwrap());
        let cliente_id = InstanceIdentity::generate("Cliente".into()).unwrap();
        let fp_servidor = servidor_id.fingerprint.clone();

        let servidor = ControlServer::bind(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            Arc::clone(&servidor_id),
        )
        .await
        .unwrap();
        let port = servidor.local_addr().unwrap().port();

        let tarea = tokio::spawn(async move {
            let mut saludo = servidor.accept_one().await.expect("saludo");
            let huella_cliente = saludo.fingerprint.clone();
            atender_emparejamiento(
                &mut saludo.stream,
                &servidor_id.fingerprint,
                &huella_cliente,
                |_codigo| true,
            )
            .await
        });

        let (peer, mut stream) =
            crate::discovery::conectar_y_saludar("127.0.0.1", port, &cliente_id)
                .await
                .expect("conectar");
        assert_eq!(peer.fingerprint, fp_servidor);

        let inicia =
            EmparejamientoEnCurso::iniciar(&cliente_id.fingerprint, &peer.fingerprint).unwrap();
        let aceptado = solicitar_emparejamiento(&mut stream, &inicia, true)
            .await
            .expect("emparejar");

        let (aceptado_remoto, remoto) = tarea.await.unwrap().expect("respuesta");
        assert!(aceptado && aceptado_remoto);
        assert_eq!(
            inicia.codigo, remoto.codigo,
            "las dos pantallas deben mostrar el mismo código"
        );
    }

    #[tokio::test]
    async fn test_si_el_usuario_local_rechaza_no_se_envia_solicitud() {
        let (a, b) = dos_identidades();
        let (mut canal_a, _canal_b) = duplex(8192);

        let inicia = EmparejamientoEnCurso::iniciar(&a.fingerprint, &b.fingerprint).unwrap();
        let aceptado = solicitar_emparejamiento(&mut canal_a, &inicia, false)
            .await
            .expect("decisión local");

        assert!(!aceptado);
    }
}
