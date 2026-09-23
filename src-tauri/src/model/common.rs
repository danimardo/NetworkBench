use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Identificador de instancia (UUID v4 canónico)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstanceId(pub String);

impl InstanceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identificador de sesión (UUID v4 canónico)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Serializador y deserializador para enteros de 64 bits que cruzan fronteras JSON
/// como cadenas decimales para evitar pérdida de precisión en JavaScript (IEEE 754).
pub mod u64_decimal {
    use super::*;

    pub fn serialize<S>(val: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum NumOrStr {
            Num(u64),
            Str(String),
        }

        match NumOrStr::deserialize(deserializer)? {
            NumOrStr::Num(n) => Ok(n),
            NumOrStr::Str(s) => s.parse::<u64>().map_err(serde::de::Error::custom),
        }
    }
}

pub mod opt_u64_decimal {
    use super::*;

    pub fn serialize<S>(val: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            Some(v) => serializer.serialize_some(&v.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum OptNumOrStr {
            Num(u64),
            Str(String),
            None,
        }

        match Option::<OptNumOrStr>::deserialize(deserializer)? {
            Some(OptNumOrStr::Num(n)) => Ok(Some(n)),
            Some(OptNumOrStr::Str(s)) => {
                s.parse::<u64>().map(Some).map_err(serde::de::Error::custom)
            }
            Some(OptNumOrStr::None) | None => Ok(None),
        }
    }
}

/// Estado discriminado de una métrica o magnitud que distingue ausencia,
/// imposibilidad de evaluación e invalidez de un valor medido.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum MetricValue<T> {
    Available { value: T },
    NotAvailable,
    NotEvaluable { reason: String },
    Invalid { reason: String },
}

impl<T> MetricValue<T> {
    pub fn available(value: T) -> Self {
        Self::Available { value }
    }

    pub fn not_available() -> Self {
        Self::NotAvailable
    }

    pub fn not_evaluable(reason: impl Into<String>) -> Self {
        Self::NotEvaluable {
            reason: reason.into(),
        }
    }

    pub fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid {
            reason: reason.into(),
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Available { value } => Some(value),
            _ => None,
        }
    }
}

/// Caudal en bits por segundo (bps) con serialización decimal segura
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ThroughputBps(#[serde(with = "u64_decimal")] pub u64);

impl ThroughputBps {
    pub fn from_bps(bps: u64) -> Self {
        Self(bps)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Bytes transferidos con serialización decimal segura
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ByteCount(#[serde(with = "u64_decimal")] pub u64);

impl ByteCount {
    pub fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Duración en milisegundos
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DurationMs(pub u32);

impl DurationMs {
    pub fn from_millis(ms: u32) -> Self {
        Self(ms)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}
