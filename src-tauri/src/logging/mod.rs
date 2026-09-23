use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

pub mod diagnostics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" | "warning" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogOrigin {
    Backend,
    Frontend,
    Helper,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub schema_version: u32,
    pub timestamp: String,
    pub level: LogLevel,
    pub origin: LogOrigin,
    pub module: String,
    pub event_code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_id: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub safe_params: HashMap<String, String>,
}

pub struct LoggerConfig {
    pub log_dir: PathBuf,
    pub effective_level: LogLevel,
    pub max_files: usize,
    pub max_total_bytes: u64,
}

pub struct Logger {
    config: Mutex<LoggerConfig>,
    dropped_events: AtomicUsize,
}

static GLOBAL_LOGGER: OnceLock<Arc<Logger>> = OnceLock::new();

/// Lista permitida de campos saneados; campos sensibles como tokens, passwords, ips o payloads
/// se redactan automáticamente.
pub fn sanitize_param_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    !lower.contains("secret")
        && !lower.contains("token")
        && !lower.contains("password")
        && !lower.contains("private")
        && !lower.contains("sessionid")
        && !lower.contains("fingerprint")
}

pub fn sanitize_value(key: &str, val: &str) -> String {
    if !sanitize_param_key(key) {
        return "[REDACTED]".to_string();
    }
    // Truncar longitudes excesivas y escapar caracteres de control
    let clean: String = val
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(512)
        .collect();
    clean
}

/// Formatea una fecha y hora en formato legible con zona Europe/Madrid
/// (formato: DD/MM/YYYY HH:mm:ss.SSS +02:00 o +01:00)
pub fn format_madrid_human(time: SystemTime) -> String {
    let dur = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();
    let millis = dur.subsec_millis();

    // Regla de horario de verano de la Unión Europea para Europe/Madrid:
    // UTC+1 en invierno (CET), UTC+2 en verano (CEST: último domingo de marzo a la 01:00 UTC hasta último domingo de octubre a la 01:00 UTC)
    let is_cest = is_madrid_dst(total_secs);
    let offset_hours = if is_cest { 2 } else { 1 };
    let offset_secs = offset_hours * 3600;

    let local_secs = (total_secs as i64) + offset_secs;
    let days = local_secs / 86400;
    let day_secs = local_secs % 86400;

    let hour = day_secs / 3600;
    let minute = (day_secs % 3600) / 60;
    let second = day_secs % 60;

    // Conversión de días a fecha (año, mes, día)
    let (y, m, d) = days_to_ymd(days);

    format!(
        "{:02}/{:02}/{:04} {:02}:{:02}:{:02}.{:03} +{:02}:00",
        d, m, y, hour, minute, second, millis, offset_hours
    )
}

/// Sello UTC en RFC 3339 con milisegundos (`2026-09-23T18:04:05.123Z`).
///
/// La constitución XIII lo exige en cada evento de log, y el protocolo entre peers usa
/// el mismo formato en el campo `ts` de su sobre. Vive aquí, junto al resto del formateo
/// de fechas, para que no existan dos implementaciones que puedan divergir.
pub fn format_rfc3339_utc(time: SystemTime) -> String {
    let dur = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs() as i64;
    let millis = dur.subsec_millis();

    let days = total_secs.div_euclid(86400);
    let day_secs = total_secs.rem_euclid(86400);
    let (y, m, d) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y,
        m,
        d,
        day_secs / 3600,
        (day_secs % 3600) / 60,
        day_secs % 60,
        millis
    )
}

fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub fn is_madrid_dst(unix_secs: u64) -> bool {
    let days = (unix_secs / 86400) as i64;
    let day_secs = unix_secs % 86400;
    let (year, month, day) = days_to_ymd(days);
    if !(3..=10).contains(&month) {
        return false;
    }
    if month > 3 && month < 10 {
        return true;
    }
    // En marzo: último domingo a la 01:00 UTC (02:00 local CET -> 03:00 local CEST)
    // En octubre: último domingo a la 01:00 UTC (03:00 local CEST -> 02:00 local CET)
    let last_sunday_march = last_sunday(year, 3);
    let last_sunday_october = last_sunday(year, 10);

    if month == 3 {
        if day < last_sunday_march {
            false
        } else if day > last_sunday_march {
            true
        } else {
            day_secs >= 3600
        }
    } else if day < last_sunday_october {
        true
    } else if day > last_sunday_october {
        false
    } else {
        day_secs < 3600
    }
}

fn last_sunday(year: i64, month: u32) -> u32 {
    // Solo se invoca para marzo y octubre, los meses del cambio de hora en la UE.
    // Ambos tienen 31 días.
    let days_in_month = 31;
    for d in (days_in_month - 6..=days_in_month).rev() {
        let days = ymd_to_days(year, month, d);
        let day_of_week = (days + 4).rem_euclid(7); // 0 = Domingo
        if day_of_week == 0 {
            return d;
        }
    }
    days_in_month
}

fn ymd_to_days(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + (doe as i64) - 719468
}

impl Logger {
    pub fn new(log_dir: PathBuf, level: LogLevel) -> Self {
        let _ = fs::create_dir_all(&log_dir);
        Self {
            config: Mutex::new(LoggerConfig {
                log_dir,
                effective_level: level,
                max_files: 10,
                max_total_bytes: 50 * 1024 * 1024,
            }),
            dropped_events: AtomicUsize::new(0),
        }
    }

    pub fn set_level(&self, level: LogLevel) {
        if let Ok(mut cfg) = self.config.lock() {
            cfg.effective_level = level;
        }
    }

    pub fn current_level(&self) -> LogLevel {
        self.config
            .lock()
            .map(|c| c.effective_level)
            .unwrap_or(LogLevel::Info)
    }

    pub fn log(&self, event: LogEvent) {
        let current_lvl = self.current_level();
        if event.level < current_lvl {
            return;
        }

        let cfg = match self.config.lock() {
            Ok(c) => c,
            Err(_) => {
                self.dropped_events.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        let file_path = cfg.log_dir.join("networkbench.log");
        let json_line = match serde_json::to_string(&event) {
            Ok(j) => format!("{}\n", j),
            Err(_) => {
                self.dropped_events.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        // Escritura en archivo local con manejo seguro que nunca bloquea ni entra en pánico
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
        {
            let _ = file.write_all(json_line.as_bytes());
        } else {
            self.dropped_events.fetch_add(1, Ordering::Relaxed);
        }

        let _ = self.rotate_if_needed(&cfg.log_dir, cfg.max_files, cfg.max_total_bytes);
    }

    pub fn rotate_if_needed(
        &self,
        log_dir: &Path,
        max_files: usize,
        max_total_bytes: u64,
    ) -> Result<(), std::io::Error> {
        let active_file = log_dir.join("networkbench.log");
        if let Ok(metadata) = fs::metadata(&active_file) {
            let file_size = metadata.len();
            let file_max = max_total_bytes / (max_files as u64);
            if file_size >= file_max {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let rotated = log_dir.join(format!("networkbench-{}.log", timestamp));
                let _ = fs::rename(&active_file, rotated);
            }
        }

        // Eliminar archivos excedentes si superan max_files
        if let Ok(entries) = fs::read_dir(log_dir) {
            let mut logs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with("networkbench-") && s.ends_with(".log"))
                        .unwrap_or(false)
                })
                .collect();

            if logs.len() >= max_files {
                logs.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
                for old in logs.iter().take(logs.len() - max_files + 1) {
                    let _ = fs::remove_file(old.path());
                }
            }
        }
        Ok(())
    }

    pub fn dropped_count(&self) -> usize {
        self.dropped_events.load(Ordering::Relaxed)
    }
}

pub fn init_logger(log_dir: PathBuf, default_level: LogLevel) -> Arc<Logger> {
    let logger = Arc::new(Logger::new(log_dir, default_level));
    let _ = GLOBAL_LOGGER.set(logger.clone());
    logger
}

pub fn get_logger() -> Option<Arc<Logger>> {
    GLOBAL_LOGGER.get().cloned()
}

pub fn log(level: LogLevel, module: &str, message: &str) {
    if let Some(logger) = get_logger() {
        let event = LogEvent {
            schema_version: 1,
            timestamp: format_madrid_human(SystemTime::now()),
            level,
            origin: LogOrigin::Backend,
            module: module.to_string(),
            event_code: "APP_EVENT".to_string(),
            message: message.to_string(),
            error_code: None,
            duration_ms: None,
            diagnostic_id: None,
            safe_params: HashMap::new(),
        };
        logger.log(event);
    }
}
