//! TLS mutuo del canal de control (FR-054).
//!
//! Ambos extremos presentan su certificado Ed25519 autofirmado. No hay CA: la
//! confianza no la da la cadena, la da la **huella** del certificado, comparada por
//! una persona durante el emparejamiento (FR-011, FR-012).
//!
//! Por eso los verificadores de esta capa aceptan cualquier certificado bien formado
//! y **no** deciden confianza: solo garantizan que el par posee la clave privada del
//! certificado que presenta. Quién es ese par lo resuelve la capa superior comparando
//! `peer_fingerprint()` con la huella almacenada. Un certificado válido no autoriza
//! nada por sí mismo.

use crate::identity::InstanceIdentity;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{
    ClientConfig, DigitallySignedStruct, DistinguishedName, Error as TlsError, ServerConfig,
    SignatureScheme,
};
use sha2::{Digest, Sha256};
use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

/// Huella SHA-256 en hexadecimal minúscula del certificado DER.
///
/// Es la misma función que usa `InstanceIdentity` al generarse: si divergieran, un peer
/// legítimo nunca coincidiría consigo mismo.
pub fn fingerprint_of_der(der: &[u8]) -> String {
    let hash = Sha256::digest(der);
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::aws_lc_rs::default_provider())
}

fn key_der_from_pem(pem: &str) -> Result<PrivateKeyDer<'static>> {
    let key_pair = rcgen::KeyPair::from_pem(pem)
        .map_err(|e| Error::other(format!("Clave privada no reconocida: {}", e)))?;
    let der = key_pair.serialize_der();
    PrivateKeyDer::try_from(der).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Clave privada no válida: {e}"),
        )
    })
}

/// Verificador que comprueba la posesión de la clave privada y nada más.
///
/// No valida cadena, emisor, nombre ni caducidad: en este protocolo no existe autoridad
/// certificadora y el nombre de host no prueba identidad (FR-011).
#[derive(Debug)]
struct PosesionDeClave {
    provider: Arc<CryptoProvider>,
}

impl PosesionDeClave {
    fn new() -> Self {
        Self {
            provider: provider(),
        }
    }

    fn tls12(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn tls13(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn esquemas(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

impl ServerCertVerifier for PosesionDeClave {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        self.tls12(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        self.tls13(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.esquemas()
    }
}

impl ClientCertVerifier for PosesionDeClave {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> std::result::Result<ClientCertVerified, TlsError> {
        Ok(ClientCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        self.tls12(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        self.tls13(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.esquemas()
    }

    /// Autenticación de cliente obligatoria: sin certificado no hay canal. Un extremo
    /// anónimo no puede llegar siquiera a enviar HELLO.
    fn client_auth_mandatory(&self) -> bool {
        true
    }

    fn offer_client_auth(&self) -> bool {
        true
    }
}

/// Configuración de servidor: exige certificado de cliente.
pub fn server_config(identity: &InstanceIdentity) -> Result<Arc<ServerConfig>> {
    let cert = CertificateDer::from(identity.cert_der.clone());
    let key = key_der_from_pem(identity.private_key_pem())?;

    let config = ServerConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .map_err(|e| Error::other(format!("Versiones TLS no soportadas: {e}")))?
        .with_client_cert_verifier(Arc::new(PosesionDeClave::new()))
        .with_single_cert(vec![cert], key)
        .map_err(|e| Error::other(format!("Certificado de servidor no válido: {e}")))?;

    Ok(Arc::new(config))
}

/// Configuración de cliente: presenta siempre su certificado.
pub fn client_config(identity: &InstanceIdentity) -> Result<Arc<ClientConfig>> {
    let cert = CertificateDer::from(identity.cert_der.clone());
    let key = key_der_from_pem(identity.private_key_pem())?;

    let config = ClientConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .map_err(|e| Error::other(format!("Versiones TLS no soportadas: {e}")))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(PosesionDeClave::new()))
        .with_client_auth_cert(vec![cert], key)
        .map_err(|e| Error::other(format!("Certificado de cliente no válido: {e}")))?;

    Ok(Arc::new(config))
}

/// Huella del certificado que el par presentó durante el handshake.
///
/// `None` significa que el par no presentó certificado. Nunca se debe interpretar como
/// «confiable por defecto»: quien reciba `None` tiene que rechazar la conexión.
pub fn peer_fingerprint(certs: Option<&[CertificateDer<'_>]>) -> Option<String> {
    certs?.first().map(|c| fingerprint_of_der(c.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huella_coincide_con_la_de_la_identidad() {
        let ident = InstanceIdentity::generate("Test-TLS".into()).expect("identidad");
        assert_eq!(fingerprint_of_der(&ident.cert_der), ident.fingerprint);
        assert_eq!(ident.fingerprint.len(), 64);
    }

    #[test]
    fn test_configuraciones_se_construyen() {
        let ident = InstanceIdentity::generate("Test-TLS".into()).expect("identidad");
        assert!(server_config(&ident).is_ok());
        assert!(client_config(&ident).is_ok());
    }

    #[test]
    fn test_sin_certificado_no_hay_huella() {
        assert_eq!(peer_fingerprint(None), None);
        assert_eq!(peer_fingerprint(Some(&[])), None);
    }

    #[test]
    fn test_dos_identidades_tienen_huellas_distintas() {
        let a = InstanceIdentity::generate("A".into()).expect("identidad A");
        let b = InstanceIdentity::generate("B".into()).expect("identidad B");
        assert_ne!(a.fingerprint, b.fingerprint);
    }
}
