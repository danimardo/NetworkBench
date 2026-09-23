use crate::export::redact::redact_session_result;
use crate::model::result::SessionResult;
use uuid::Uuid;

pub const UTF8_BOM: &str = "\u{FEFF}";

#[derive(Debug, Clone)]
pub struct SampleCsvRow {
    pub session_id: Uuid,
    pub direction: String,
    pub endpoint: String,
    pub timestamp_ms: i64,
    pub rx_bps: Option<u64>,
    pub tx_bps: Option<u64>,
    pub cpu_percent: Option<f64>,
}

/// Neutraliza cualquier posible inyección de fórmulas en aplicaciones de hojas de cálculo
/// (Excel, LibreOffice Calc). Si la cadena empieza por '=', '+', '-', o '@', se antepone un apóstrofo "'".
pub fn sanitize_csv_cell(value: &str) -> String {
    let trimmed = value.trim_start();
    if trimmed.starts_with('=')
        || trimmed.starts_with('+')
        || trimmed.starts_with('-')
        || trimmed.starts_with('@')
        || trimmed.starts_with('\t')
        || trimmed.starts_with('\r')
    {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

/// Escapa un campo CSV envolviéndolo en comillas dobles si contiene el separador, comillas o saltos de línea.
pub fn escape_csv_field(value: &str, delimiter: char) -> String {
    let sanitized = sanitize_csv_cell(value);
    if sanitized.contains(delimiter)
        || sanitized.contains('"')
        || sanitized.contains('\n')
        || sanitized.contains('\r')
    {
        let escaped = sanitized.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        sanitized
    }
}

/// Formatea un float según el separador decimal indicado (',' para español, '.' para inglés)
pub fn format_float_locale(val: f64, precision: usize, decimal_sep: char) -> String {
    let s = format!("{val:.precision$}");
    if decimal_sep != '.' {
        s.replace('.', &decimal_sep.to_string())
    } else {
        s
    }
}

/// Exporta el resumen de una o varias sesiones en formato CSV con UTF-8 BOM.
/// Encabezados:
/// `sessionId, fecha, equipoLocal, equipoRemoto, direccion, protocolo, velocidadMbit, capacidadRefMbit, aprovechamientoPct, estabilidad, cv, retransmisiones, paquetesEnviados, ratioRetrans, cpuEmisorPct, cpuReceptorPct, streams, bufferBytes, duracionS, veredicto, estado`
pub fn export_summary_csv(
    sessions: &[SessionResult],
    anonymize: bool,
    delimiter: char,
    decimal_sep: char,
) -> Result<String, String> {
    if sessions.is_empty() {
        return Err("No hay sesiones para exportar".to_string());
    }

    let mut output = String::from(UTF8_BOM);

    let headers = [
        "sessionId",
        "fecha",
        "equipoLocal",
        "equipoRemoto",
        "direccion",
        "protocolo",
        "velocidadMbit",
        "capacidadRefMbit",
        "aprovechamientoPct",
        "estabilidad",
        "cv",
        "retransmisiones",
        "paquetesEnviados",
        "ratioRetrans",
        "cpuEmisorPct",
        "cpuReceptorPct",
        "streams",
        "bufferBytes",
        "duracionS",
        "veredicto",
        "estado",
    ];

    output.push_str(&headers.join(&delimiter.to_string()));
    output.push('\n');

    for raw_session in sessions {
        let session = redact_session_result(raw_session, anonymize);
        let cap_ref_mbit = session.capacity.as_ref().and_then(|c| {
            c.ref_bps.map(|bps| format_float_locale(bps as f64 / 1_000_000.0, 2, decimal_sep))
        }).unwrap_or_default();

        let verdict_str = session.verdict.as_ref().map(|v| format!("{:?}", v.level)).unwrap_or_default();

        for dir in &session.directions {
            let speed_mbit = dir.official_bps.as_deref().and_then(|b| {
                b.parse::<f64>().ok().map(|bps| format_float_locale(bps / 1_000_000.0, 2, decimal_sep))
            }).unwrap_or_default();

            let util_pct = dir.utilization.map(|u| format_float_locale(u * 100.0, 1, decimal_sep)).unwrap_or_default();
            let stability_level = dir.stability.as_ref().map(|s| format!("{:?}", s.level)).unwrap_or_default();
            let cv_str = dir.stability.as_ref().map(|s| format_float_locale(s.cv, 4, decimal_sep)).unwrap_or_default();

            let retrans_count = dir.retransmission.as_ref().and_then(|r| r.packets_retransmitted).map(|p| p.to_string()).unwrap_or_default();
            let pkts_sent = dir.retransmission.as_ref().and_then(|r| r.packets_sent).map(|p| p.to_string()).unwrap_or_default();
            let retrans_ratio = dir.retransmission.as_ref().and_then(|r| r.ratio).map(|r| format_float_locale(r, 5, decimal_sep)).unwrap_or_default();

            let cpu_emisor = dir.cpu_sender.map(|c| format_float_locale(c, 1, decimal_sep)).unwrap_or_default();
            let cpu_receptor = dir.cpu_receiver.map(|c| format_float_locale(c, 1, decimal_sep)).unwrap_or_default();

            let proto_str = match session.plan.protocol {
                crate::model::plan::BenchmarkProtocol::Tcp => "tcp",
                crate::model::plan::BenchmarkProtocol::Udp => "udp",
            };
            let buf_str = session.plan.buffer_size_bytes.as_deref().unwrap_or_default();

            let row = [
                escape_csv_field(&session.session_id.to_string(), delimiter),
                escape_csv_field(&session.started_at, delimiter),
                escape_csv_field(&session.initiator.display_name, delimiter),
                escape_csv_field(&session.responder.display_name, delimiter),
                escape_csv_field(&dir.direction, delimiter),
                escape_csv_field(proto_str, delimiter),
                escape_csv_field(&speed_mbit, delimiter),
                escape_csv_field(&cap_ref_mbit, delimiter),
                escape_csv_field(&util_pct, delimiter),
                escape_csv_field(&stability_level, delimiter),
                escape_csv_field(&cv_str, delimiter),
                escape_csv_field(&retrans_count, delimiter),
                escape_csv_field(&pkts_sent, delimiter),
                escape_csv_field(&retrans_ratio, delimiter),
                escape_csv_field(&cpu_emisor, delimiter),
                escape_csv_field(&cpu_receptor, delimiter),
                escape_csv_field(&session.plan.streams.to_string(), delimiter),
                escape_csv_field(buf_str, delimiter),
                escape_csv_field(&session.plan.measure_seconds.to_string(), delimiter),
                escape_csv_field(&verdict_str, delimiter),
                escape_csv_field(&dir.status, delimiter),
            ];

            output.push_str(&row.join(&delimiter.to_string()));
            output.push('\n');
        }
    }

    Ok(output)
}

/// Exporta las muestras de throughput/cpu en formato CSV con UTF-8 BOM.
/// Encabezados:
/// `sessionId, direccion, equipo, tMs, rxMbit, txMbit, cpuPct`
pub fn export_samples_csv(
    samples: &[SampleCsvRow],
    delimiter: char,
    decimal_sep: char,
) -> Result<String, String> {
    let mut output = String::from(UTF8_BOM);

    let headers = [
        "sessionId",
        "direccion",
        "equipo",
        "tMs",
        "rxMbit",
        "txMbit",
        "cpuPct",
    ];

    output.push_str(&headers.join(&delimiter.to_string()));
    output.push('\n');

    for s in samples {
        let rx_mbit = s.rx_bps.map(|bps| format_float_locale(bps as f64 / 1_000_000.0, 2, decimal_sep)).unwrap_or_default();
        let tx_mbit = s.tx_bps.map(|bps| format_float_locale(bps as f64 / 1_000_000.0, 2, decimal_sep)).unwrap_or_default();
        let cpu_str = s.cpu_percent.map(|c| format_float_locale(c, 1, decimal_sep)).unwrap_or_default();

        let row = [
            escape_csv_field(&s.session_id.to_string(), delimiter),
            escape_csv_field(&s.direction, delimiter),
            escape_csv_field(&s.endpoint, delimiter),
            escape_csv_field(&s.timestamp_ms.to_string(), delimiter),
            escape_csv_field(&rx_mbit, delimiter),
            escape_csv_field(&tx_mbit, delimiter),
            escape_csv_field(&cpu_str, delimiter),
        ];

        output.push_str(&row.join(&delimiter.to_string()));
        output.push('\n');
    }

    Ok(output)
}
