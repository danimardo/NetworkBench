use crate::errors::{AppError, ErrorAction, ErrorCode, ErrorSeverity};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcResult<T> {
    Success(T),
    Failure(AppError),
}

impl<T> IpcResult<T> {
    pub fn ok(value: T) -> Self {
        Self::Success(value)
    }

    pub fn err(error: AppError) -> Self {
        Self::Failure(error)
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, Self::Failure(_))
    }
}

impl<T: Serialize> Serialize for IpcResult<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            Self::Success(val) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("ok", &true)?;
                map.serialize_entry("value", val)?;
                map.end()
            }
            Self::Failure(err) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("ok", &false)?;
                map.serialize_entry("error", err)?;
                map.end()
            }
        }
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for IpcResult<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawIpc {
            ok: bool,
            value: Option<serde_json::Value>,
            error: Option<AppError>,
        }

        let raw = RawIpc::deserialize(deserializer)?;
        if raw.ok {
            if let Some(val) = raw.value {
                let parsed = serde_json::from_value(val).map_err(serde::de::Error::custom)?;
                Ok(IpcResult::Success(parsed))
            } else {
                Err(serde::de::Error::custom("Missing 'value' for ok: true"))
            }
        } else if let Some(err) = raw.error {
            Ok(IpcResult::Failure(err))
        } else {
            Err(serde::de::Error::custom("Missing 'error' for ok: false"))
        }
    }
}

#[derive(Debug, Clone)]
struct TokenEntry {
    operation: String,
    payload_hash: Option<String>,
    created_at: Instant,
    ttl: Duration,
}

pub struct OneTimeTokenStore {
    tokens: Mutex<HashMap<String, TokenEntry>>,
}

impl OneTimeTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    /// Emite un token de un solo uso vinculado a una operación y opcionalmente al hash del payload
    pub fn issue(
        &self,
        operation: impl Into<String>,
        payload_hash: Option<&str>,
        ttl_secs: u64,
    ) -> String {
        let token = Uuid::new_v4().to_string();
        let entry = TokenEntry {
            operation: operation.into(),
            payload_hash: payload_hash.map(|h| h.to_string()),
            created_at: Instant::now(),
            ttl: Duration::from_secs(ttl_secs),
        };
        if let Ok(mut lock) = self.tokens.lock() {
            // Limpieza oportunista de tokens expirados
            lock.retain(|_, v| v.created_at.elapsed() <= v.ttl);
            lock.insert(token.clone(), entry);
        }
        token
    }

    /// Consume atómicamente el token. Si no existe, caducó, o no coincide la operación o payload, falla.
    /// Al consumirse exitosamente, queda invalidado permanentemente.
    pub fn consume(
        &self,
        token: &str,
        expected_operation: &str,
        expected_payload_hash: Option<&str>,
    ) -> Result<(), AppError> {
        let mut lock = self.tokens.lock().map_err(|_| {
            AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Fatal,
                "errors.NB-INTERNAL-001",
            )
        })?;

        let entry = lock.get(token).ok_or_else(|| {
            AppError::new(
                ErrorCode::PeerPairingExpired,
                ErrorSeverity::Error,
                "errors.NB-PEER-002",
            )
            .with_action(ErrorAction::RetryPairing)
        })?;

        if entry.created_at.elapsed() > entry.ttl {
            lock.remove(token);
            return Err(AppError::new(
                ErrorCode::PeerPairingExpired,
                ErrorSeverity::Error,
                "errors.NB-PEER-002",
            )
            .with_action(ErrorAction::RetryPairing));
        }

        if entry.operation != expected_operation {
            return Err(AppError::new(
                ErrorCode::PeerPairingMismatch,
                ErrorSeverity::Error,
                "errors.NB-PEER-001",
            )
            .with_action(ErrorAction::RetryPairing));
        }

        if let Some(expected_hash) = expected_payload_hash
            && entry.payload_hash.as_deref() != Some(expected_hash)
        {
            return Err(AppError::new(
                ErrorCode::PeerPairingMismatch,
                ErrorSeverity::Error,
                "errors.NB-PEER-001",
            )
            .with_action(ErrorAction::RetryPairing));
        }

        // Consumo exitoso de un solo uso: invalidación permanente
        lock.remove(token);
        Ok(())
    }
}

impl Default for OneTimeTokenStore {
    fn default() -> Self {
        Self::new()
    }
}
