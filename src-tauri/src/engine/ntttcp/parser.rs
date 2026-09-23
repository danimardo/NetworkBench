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
    pub throughput_bps: u64,
    pub cpu_percent: Option<f64>,
    pub errors_count: u32,
    pub packets_sent: Option<u64>,
    pub packets_received: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NtttcpParseError {
    EmptyXml,
    MalformedXml(String),
    MissingField(&'static str),
    InvalidFieldValue(&'static str),
}

pub fn parse_ntttcp_xml(xml: &str) -> Result<NtttcpParsedResult, NtttcpParseError> {
    let trimmed = xml.trim();
    if trimmed.is_empty() || trimmed.starts_with("<!-- empty") {
        return Err(NtttcpParseError::EmptyXml);
    }

    let mut reader = Reader::from_str(trimmed);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut current_tag = String::new();

    let mut role: Option<NtttcpRole> = None;
    let mut total_bytes: Option<u64> = None;
    let mut total_buffers: Option<u64> = None;
    let mut realtime_seconds: Option<f64> = None;
    let mut throughput_bps: Option<u64> = None;
    let mut throughput_mbps: Option<f64> = None;
    let mut cpu_percent: Option<f64> = None;
    let mut errors_count: Option<u32> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_tag = e.name().as_ref().to_lowercase();
            }
            Ok(Event::Text(ref e)) => {
                let val = e.as_ref().trim();
                match current_tag.as_str() {
                    "role" => {
                        if val.eq_ignore_ascii_case("receiver") {
                            role = Some(NtttcpRole::Receiver);
                        } else if val.eq_ignore_ascii_case("sender") {
                            role = Some(NtttcpRole::Sender);
                        }
                    }
                    "total_bytes" => {
                        if let Ok(b) = val.parse::<u64>() {
                            total_bytes = Some(b);
                        }
                    }
                    "total_buffers" => {
                        if let Ok(b) = val.parse::<u64>() {
                            total_buffers = Some(b);
                        }
                    }
                    "realtime" => {
                        if let Ok(s) = val.parse::<f64>() {
                            realtime_seconds = Some(s);
                        }
                    }
                    "throughput_bps" => {
                        if let Ok(bps) = val.parse::<u64>() {
                            throughput_bps = Some(bps);
                        }
                    }
                    "throughput" => {
                        if let Ok(mbps) = val.parse::<f64>() {
                            throughput_mbps = Some(mbps);
                        }
                    }
                    "cpu" => {
                        if let Ok(cpu) = val.parse::<f64>() {
                            cpu_percent = Some(cpu);
                        }
                    }
                    "errors" => {
                        if let Ok(errs) = val.parse::<u32>() {
                            errors_count = Some(errs);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
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

    // Calcular bps desde throughput_bps o derivar de throughput_mbps o de total_bytes / realtime_seconds
    let throughput_bps = if let Some(bps) = throughput_bps {
        bps
    } else if let Some(mbps) = throughput_mbps {
        (mbps * 1_000_000.0) as u64
    } else if realtime_seconds > 0.0 {
        ((total_bytes as f64 * 8.0) / realtime_seconds) as u64
    } else {
        return Err(NtttcpParseError::MissingField("throughput"));
    };

    let (packets_sent, packets_received) = match role {
        NtttcpRole::Sender => (Some(total_buffers), None),
        NtttcpRole::Receiver => (None, Some(total_buffers)),
    };

    Ok(NtttcpParsedResult {
        role,
        total_bytes,
        total_buffers,
        realtime_seconds,
        throughput_bps,
        cpu_percent,
        errors_count: errors_count.unwrap_or(0),
        packets_sent,
        packets_received,
    })
}
