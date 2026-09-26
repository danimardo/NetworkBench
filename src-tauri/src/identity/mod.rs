pub mod dpapi;

use rcgen::{CertificateParams, KeyPair, PKCS_ED25519};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIdentity {
    pub instance_id: Uuid,
    pub display_name: String,
    pub fingerprint: String,
    pub cert_pem: String,
}

#[derive(Clone)]
pub struct InstanceIdentity {
    pub instance_id: Uuid,
    pub display_name: String,
    pub fingerprint: String,
    pub cert_pem: String,
    pub cert_der: Vec<u8>,
    // La clave privada NUNCA se expone fuera de este módulo
    private_key_pem: String,
}

/// Nombre que las primeras versiones fijaban en lugar del hostname; también es la reserva.
const NOMBRE_HISTORICO_POR_DEFECTO: &str = "NetworkBench";

/// `displayName` por defecto: el hostname de Windows (Historias.md §6.2), de 1 a 48
/// caracteres y sin caracteres de control. Sin hostname utilizable, la reserva.
pub fn nombre_del_equipo() -> String {
    std::env::var("COMPUTERNAME")
        .ok()
        .map(|n| {
            n.chars()
                .filter(|c| !c.is_control())
                .take(48)
                .collect::<String>()
                .trim()
                .to_string()
        })
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| NOMBRE_HISTORICO_POR_DEFECTO.to_string())
}

impl InstanceIdentity {
    /// Genera una nueva identidad local usando certificado Ed25519 autofirmado y UUID v4.
    pub fn generate(display_name: String) -> Result<Self> {
        let instance_id = Uuid::new_v4();
        let key_pair = KeyPair::generate_for(&PKCS_ED25519)
            .map_err(|e| Error::other(format!("Error generando par Ed25519: {}", e)))?;

        let mut params = CertificateParams::new(vec![
            format!("netbench-{}", instance_id),
            "localhost".to_string(),
        ])
        .map_err(|e| Error::other(format!("Error creando params cert: {}", e)))?;

        params.distinguished_name.push(
            rcgen::DnType::CommonName,
            format!("NetworkBench-{}", display_name),
        );

        let cert = params
            .self_signed(&key_pair)
            .map_err(|e| Error::other(format!("Error autofirmando cert: {}", e)))?;

        let cert_der = cert.der().to_vec();
        let cert_pem = cert.pem();
        let private_key_pem = key_pair.serialize_pem();

        // Calcular huella SHA-256 del certificado DER
        let mut hasher = Sha256::new();
        hasher.update(&cert_der);
        let hash = hasher.finalize();
        let fingerprint = hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        Ok(Self {
            instance_id,
            display_name,
            fingerprint,
            cert_pem,
            cert_der,
            private_key_pem,
        })
    }

    /// Escribe `identity.json`: metadatos públicos, sin la clave privada.
    fn write_metadata(&self, dir: &Path) -> Result<()> {
        let meta = serde_json::json!({
            "instanceId": self.instance_id.to_string(),
            "displayName": self.display_name,
            "fingerprint": self.fingerprint,
            "certPem": self.cert_pem,
        });
        fs::write(dir.join("identity.json"), serde_json::to_vec_pretty(&meta)?)
    }

    /// Guarda la identidad en disco: certificado en claro, clave privada cifrada con DPAPI.
    pub fn save_to_dir(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir)?;

        // 1. Guardar metadatos públicos
        self.write_metadata(dir)?;

        // 2. Guardar cert DER
        fs::write(dir.join("cert.der"), &self.cert_der)?;

        // 3. Proteger clave privada con DPAPI antes de escribirla
        let protected_key = dpapi::protect_bytes(self.private_key_pem.as_bytes())?;
        fs::write(dir.join("key.dpapi"), protected_key)?;

        Ok(())
    }

    /// Carga una identidad previamente persistida descifrando la clave privada con DPAPI.
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let meta_raw = fs::read(dir.join("identity.json"))?;
        let meta: serde_json::Value =
            serde_json::from_slice(&meta_raw).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        let instance_id = Uuid::parse_str(
            meta["instanceId"]
                .as_str()
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "instanceId faltante"))?,
        )
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        let display_name = meta["displayName"]
            .as_str()
            .unwrap_or("NetworkBench-Local")
            .to_string();
        let fingerprint = meta["fingerprint"].as_str().unwrap_or("").to_string();
        let cert_pem = meta["certPem"].as_str().unwrap_or("").to_string();
        let cert_der = fs::read(dir.join("cert.der"))?;

        let protected_key = fs::read(dir.join("key.dpapi"))?;
        let key_bytes = dpapi::unprotect_bytes(&protected_key)?;
        let private_key_pem =
            String::from_utf8(key_bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        Ok(Self {
            instance_id,
            display_name,
            fingerprint,
            cert_pem,
            cert_der,
            private_key_pem,
        })
    }

    /// Carga o genera la identidad en el directorio dado de forma idempotente.
    pub fn get_or_create(dir: &Path, default_name: &str) -> Result<Self> {
        if dir.join("identity.json").exists() && dir.join("key.dpapi").exists() {
            match Self::load_from_dir(dir) {
                Ok(mut ident) => {
                    // Las primeras versiones sembraban «NetworkBench» en vez del hostname
                    // (Historias.md §6.2) y quedó persistido. Aún no hay ajuste para
                    // renombrar, así que ese valor nunca lo eligió el usuario: se corrige
                    // sin regenerar la identidad, que conserva huella y emparejamientos.
                    if ident.display_name == NOMBRE_HISTORICO_POR_DEFECTO
                        && default_name != NOMBRE_HISTORICO_POR_DEFECTO
                    {
                        ident.display_name = default_name.to_string();
                        if let Err(e) = ident.write_metadata(dir) {
                            tracing::warn!("No se pudo persistir el nombre corregido: {}", e);
                        }
                    }
                    return Ok(ident);
                }
                Err(e) => {
                    tracing::warn!("Error cargando identidad previa, regenerando: {}", e);
                }
            }
        }

        let new_ident = Self::generate(default_name.to_string())?;
        new_ident.save_to_dir(dir)?;
        Ok(new_ident)
    }

    pub fn to_public(&self) -> PublicIdentity {
        PublicIdentity {
            instance_id: self.instance_id,
            display_name: self.display_name.clone(),
            fingerprint: self.fingerprint.clone(),
            cert_pem: self.cert_pem.clone(),
        }
    }

    /// Acceso interno controlado para handshake TLS
    pub fn private_key_pem(&self) -> &str {
        &self.private_key_pem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation_and_dpapi_persistence() {
        let tmp = std::env::temp_dir().join(format!("nb_test_ident_{}", Uuid::new_v4()));
        let ident = InstanceIdentity::generate("Test-PC".into()).expect("Generar identidad");

        assert_eq!(ident.display_name, "Test-PC");
        assert_eq!(ident.fingerprint.len(), 64);
        assert!(!ident.cert_pem.is_empty());
        assert!(!ident.private_key_pem.is_empty());

        // Guardar y recargar mediante DPAPI
        ident.save_to_dir(&tmp).expect("Guardar identidad");

        let loaded = InstanceIdentity::load_from_dir(&tmp).expect("Cargar identidad");
        assert_eq!(loaded.instance_id, ident.instance_id);
        assert_eq!(loaded.fingerprint, ident.fingerprint);
        assert_eq!(loaded.private_key_pem, ident.private_key_pem);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn get_or_create_corrige_el_nombre_historico_sin_cambiar_la_identidad() {
        let tmp = std::env::temp_dir().join(format!("nb_test_ident_{}", Uuid::new_v4()));
        let vieja = InstanceIdentity::generate(NOMBRE_HISTORICO_POR_DEFECTO.into()).unwrap();
        vieja.save_to_dir(&tmp).unwrap();

        let corregida = InstanceIdentity::get_or_create(&tmp, "PC-DANI").unwrap();
        assert_eq!(corregida.display_name, "PC-DANI");
        // Misma identidad: huella e instanceId no cambian, o se perderían los emparejamientos.
        assert_eq!(corregida.fingerprint, vieja.fingerprint);
        assert_eq!(corregida.instance_id, vieja.instance_id);

        // La corrección queda persistida, no solo en memoria.
        let recargada = InstanceIdentity::load_from_dir(&tmp).unwrap();
        assert_eq!(recargada.display_name, "PC-DANI");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn get_or_create_respeta_un_nombre_que_no_es_el_historico() {
        let tmp = std::env::temp_dir().join(format!("nb_test_ident_{}", Uuid::new_v4()));
        InstanceIdentity::generate("Sala-3".into())
            .unwrap()
            .save_to_dir(&tmp)
            .unwrap();

        let cargada = InstanceIdentity::get_or_create(&tmp, "PC-DANI").unwrap();
        assert_eq!(cargada.display_name, "Sala-3");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn nombre_del_equipo_nunca_esta_vacio_ni_pasa_de_48() {
        let n = nombre_del_equipo();
        assert!(!n.is_empty());
        assert!(n.chars().count() <= 48);
        assert!(!n.chars().any(|c| c.is_control()));
    }
}
