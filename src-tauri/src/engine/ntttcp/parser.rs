use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtttcpRole {
    Sender,
    Receiver,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NtttcpParsedResult {
    pub role: NtttcpRole,
    pub total_bytes: u64,
    pub total_buffers: u64,
    pub realtime_seconds: f64,
    /// Caudal en **bits** por segundo, derivado del elemento `Bps` (bytes) del motor.
    pub throughput_bps: u64,
    pub cpu_percent: Option<f64>,
    pub errors_count: u32,
    pub packets_sent: Option<u64>,
    pub packets_received: Option<u64>,
    pub packets_retransmitted: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NtttcpParseError {
    EmptyXml,
    MalformedXml(String),
    MissingField(&'static str),
    InvalidFieldValue(&'static str),
}

/// Interpreta la salida XML de NTTTCP 5.40.
///
/// Tres particularidades del formato real que condicionan esta implementación, todas
/// comprobadas contra una ejecución auténtica (fixtures `real_5.40_*.xml`):
///
/// 1. **El rol está en el elemento raíz**, `<ntttcpr>` o `<ntttcps>`. No existe ningún
///    elemento `<role>`.
/// 2. **Varios nombres se repiten a distinta profundidad.** `<realtime>` aparece dentro
///    de cada `<thread>` valiendo `0.000` y otra vez como total; `<throughput>` aparece
///    con cinco métricas distintas (`KB/s`, `MB/s`, `mbps`, `Bps`, `buffers/s`). Por eso
///    solo se leen los hijos directos de la raíz y `<throughput>` se elige por su
///    atributo `metric`, no por posición.
/// 3. **Las cifras llegan con decimales y unidad.** `total_bytes` viene en MB y
///    `total_buffers` como `83445.000`.
pub fn parse_ntttcp_xml(xml: &str) -> Result<NtttcpParsedResult, NtttcpParseError> {
    let trimmed = xml.trim();
    if trimmed.is_empty() || trimmed.starts_with("<!-- empty") {
        return Err(NtttcpParseError::EmptyXml);
    }

    let mut reader = Reader::from_str(trimmed);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut profundidad: i32 = 0;
    let mut tag = String::new();
    let mut metric = String::new();

    let mut role: Option<NtttcpRole> = None;
    let mut total_bytes: Option<u64> = None;
    let mut total_buffers: Option<u64> = None;
    let mut realtime_seconds: Option<f64> = None;
    let mut throughput_bytes_s: Option<f64> = None;
    let mut throughput_mbps: Option<f64> = None;
    let mut cpu_percent: Option<f64> = None;
    let mut errors_count: Option<u32> = None;
    let mut packets_sent: Option<u64> = None;
    let mut packets_received: Option<u64> = None;
    let mut packets_retransmitted: Option<u64> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                profundidad += 1;
                let nombre = e.name().as_ref().to_lowercase();

                if profundidad == 1 {
                    role = match nombre.as_str() {
                        "ntttcpr" => Some(NtttcpRole::Receiver),
                        "ntttcps" => Some(NtttcpRole::Sender),
                        _ => None,
                    };
                }

                metric = e
                    .try_get_attribute("metric")
                    .ok()
                    .flatten()
                    .map(|a| a.value.as_ref().to_string())
                    .unwrap_or_default();
                tag = nombre;
            }
            Ok(Event::Text(ref e)) => {
                // Solo hijos directos de la raíz: descarta `<parameters>` y `<thread>`.
                if profundidad != 2 {
                    continue;
                }
                let val = e.as_ref().trim().to_string();
                let num = || val.parse::<f64>().ok();

                match tag.as_str() {
                    "total_bytes" => {
                        total_bytes = num().map(|v| match metric.as_str() {
                            "MB" => (v * 1_048_576.0) as u64,
                            "KB" => (v * 1024.0) as u64,
                            _ => v as u64,
                        });
                    }
                    "total_buffers" => total_buffers = num().map(|v| v as u64),
                    "realtime" => realtime_seconds = num(),
                    "throughput" => match metric.as_str() {
                        "Bps" => throughput_bytes_s = num(),
                        "mbps" => throughput_mbps = num(),
                        _ => {}
                    },
                    "cpu" => cpu_percent = num(),
                    "errors" => errors_count = num().map(|v| v as u32),
                    "packets_sent" => packets_sent = num().map(|v| v as u64),
                    "packets_received" => packets_received = num().map(|v| v as u64),
                    "packets_retransmitted" => packets_retransmitted = num().map(|v| v as u64),
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                profundidad -= 1;
                tag.clear();
                metric.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(NtttcpParseError::MalformedXml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    let role = role.ok_or(NtttcpParseError::MissingField("role"))?;
    let total_bytes = total_bytes.ok_or(NtttcpParseError::MissingField("total_bytes"))?;
    let total_buffers = total_buffers.unwrap_or(0);
    let realtime_seconds = realtime_seconds.ok_or(NtttcpParseError::MissingField("realtime"))?;

    // `Bps` son bytes por segundo; el resultado se expresa en bits.
    let throughput_bps = if let Some(bytes_s) = throughput_bytes_s {
        (bytes_s * 8.0) as u64
    } else if let Some(mbps) = throughput_mbps {
        (mbps * 1_000_000.0) as u64
    } else if realtime_seconds > 0.0 {
        ((total_bytes as f64 * 8.0) / realtime_seconds) as u64
    } else {
        return Err(NtttcpParseError::MissingField("throughput"));
    };

    Ok(NtttcpParsedResult {
        role,
        total_bytes,
        total_buffers,
        realtime_seconds,
        throughput_bps,
        cpu_percent,
        errors_count: errors_count.unwrap_or(0),
        // Contadores reales del motor. Antes se rellenaban con `total_buffers`, que no
        // es el número de paquetes: eso falseaba el cálculo de pérdida UDP (V-02).
        packets_sent,
        packets_received,
        packets_retransmitted,
    })
}
