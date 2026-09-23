use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const PAIRING_CODE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingError {
    CodeMismatch,
    Expired,
    FingerprintMismatch { expected: String, actual: String },
    InvalidFormat,
}

#[derive(Debug, Clone)]
pub struct ActivePairing {
    pub session_id: Uuid,
    pub expected_code: String,
    pub remote_fingerprint: String,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl ActivePairing {
    pub fn new(
        session_id: Uuid,
        local_fingerprint: &str,
        remote_fingerprint: &str,
    ) -> Result<Self, PairingError> {
        let code = derive_pairing_code(local_fingerprint, remote_fingerprint, &session_id)?;
        Ok(Self {
            session_id,
            expected_code: code,
            remote_fingerprint: remote_fingerprint.to_lowercase(),
            created_at: Instant::now(),
            ttl: PAIRING_CODE_TTL,
        })
    }

    pub fn with_instant(
        session_id: Uuid,
        local_fingerprint: &str,
        remote_fingerprint: &str,
        created_at: Instant,
        ttl: Duration,
    ) -> Result<Self, PairingError> {
        let code = derive_pairing_code(local_fingerprint, remote_fingerprint, &session_id)?;
        Ok(Self {
            session_id,
            expected_code: code,
            remote_fingerprint: remote_fingerprint.to_lowercase(),
            created_at,
            ttl,
        })
    }

    pub fn verify(
        &self,
        provided_code: &str,
        claimed_remote_fingerprint: &str,
        now: Instant,
    ) -> Result<(), PairingError> {
        // 1. Verificar huella
        let norm_fp = claimed_remote_fingerprint.to_lowercase();
        if norm_fp != self.remote_fingerprint {
            return Err(PairingError::FingerprintMismatch {
                expected: self.remote_fingerprint.clone(),
                actual: norm_fp,
            });
        }

        // 2. Verificar caducidad
        if now.duration_since(self.created_at) > self.ttl {
            return Err(PairingError::Expired);
        }

        // 3. Verificar código de 6 dígitos
        let norm_code = provided_code.trim();
        if norm_code.len() != 6 || !norm_code.chars().all(|c| c.is_ascii_digit()) {
            return Err(PairingError::InvalidFormat);
        }

        if norm_code != self.expected_code {
            return Err(PairingError::CodeMismatch);
        }

        Ok(())
    }
}

/// Deriva un código de 6 dígitos simétrico y determinista a partir de dos huellas y el session_id.
pub fn derive_pairing_code(
    fp_a: &str,
    fp_b: &str,
    session_id: &Uuid,
) -> Result<String, PairingError> {
    let norm_a = fp_a.trim().to_lowercase();
    let norm_b = fp_b.trim().to_lowercase();

    if norm_a.len() != 64 || norm_b.len() != 64 {
        return Err(PairingError::InvalidFormat);
    }

    let mut hasher = Sha256::new();
    if norm_a <= norm_b {
        hasher.update(norm_a.as_bytes());
        hasher.update(norm_b.as_bytes());
    } else {
        hasher.update(norm_b.as_bytes());
        hasher.update(norm_a.as_bytes());
    }
    hasher.update(session_id.as_bytes());

    let hash = hasher.finalize();
    let num = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]) % 1_000_000;
    Ok(format!("{:06}", num))
}

#[cfg(test)]
mod pairing_tests;
