use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum TrustState {
    #[default]
    Unknown,
    Known,
    Trusted,
    TrustedAutoAccept,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    pub instance_id: Uuid,
    pub display_name: String,
    pub fingerprint: String,
    pub addresses: Vec<String>,
    pub trust_state: TrustState,
    pub auto_accept: bool,
    pub last_seen: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

fn system_time_to_iso8601(time: SystemTime) -> String {
    let dur = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();

    // Cálculo de fecha/hora UTC desde segundos UNIX
    let sec = total_secs % 60;
    let min = (total_secs / 60) % 60;
    let hour = (total_secs / 3600) % 24;
    let mut days = (total_secs / 86400) as i64;

    let mut year = 1970;
    loop {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if days >= days_in_year {
            days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_months = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1;
    for &dim in &days_in_months {
        if days >= dim {
            days -= dim;
            month += 1;
        } else {
            break;
        }
    }
    let day = days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, min, sec
    )
}

impl Peer {
    pub fn new(
        instance_id: Uuid,
        display_name: String,
        fingerprint: String,
        addresses: Vec<String>,
    ) -> Result<Self, String> {
        let cleaned_name = display_name.trim();
        if cleaned_name.is_empty() {
            return Err("El nombre de equipo no puede estar vacío".to_string());
        }
        if cleaned_name.chars().count() > 48 {
            return Err("El nombre de equipo no puede exceder 48 caracteres".to_string());
        }
        if cleaned_name.chars().any(|c| {
            c.is_control()
                || ('\u{202A}'..='\u{202E}').contains(&c)
                || ('\u{2066}'..='\u{2069}').contains(&c)
        }) {
            return Err("El nombre no puede contener caracteres de control ni bidi".to_string());
        }
        let fp_norm = fingerprint.trim().to_lowercase();
        if fp_norm.len() != 64 || !fp_norm.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(
                "La huella debe ser un hash SHA-256 de 64 caracteres hexadecimales".to_string(),
            );
        }

        Ok(Self {
            instance_id,
            display_name: cleaned_name.to_string(),
            fingerprint: fp_norm,
            addresses,
            trust_state: TrustState::Unknown,
            auto_accept: false,
            last_seen: system_time_to_iso8601(SystemTime::now()),
            alias: None,
        })
    }
}
