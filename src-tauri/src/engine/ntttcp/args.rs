use super::parser::NtttcpRole;
use crate::model::plan::{BenchmarkPlan, BenchmarkProtocol};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NtttcpArgError {
    InvalidPlan(String),
    InvalidTargetPath(String),
    IllegalFlag(String),
}

/// Allowlist de argumentos permitidos para Microsoft NTTTCP en Windows:
/// Flags permitidos: -s, -r, -m, -l, -a, -t, -cd, -wu, -xml, -u (para UDP en H5)
pub fn build_ntttcp_args(
    role: NtttcpRole,
    plan: &BenchmarkPlan,
    target_host: Option<&str>,
    xml_output_file: &Path,
) -> Result<Vec<String>, NtttcpArgError> {
    plan.validate().map_err(NtttcpArgError::InvalidPlan)?;

    let mut args = Vec::new();

    // 1. Rol (-r para receptor, -s para emisor)
    match role {
        NtttcpRole::Receiver => {
            args.push("-r".to_string());
        }
        NtttcpRole::Sender => {
            args.push("-s".to_string());
        }
    }

    // 2. Mapeo: `-m <hilos>,<procesador>,<dirección>`, sin paréntesis y SIN el puerto.
    //
    // Comprobado contra NTTTCP 5.40 el 2026-09-23: el puerto va en `-p`, no dentro del
    // mapeo, y los paréntesis que aparecen en el XML de salida no forman parte del
    // argumento. La forma anterior, `(1,*,host,puerto)`, hacía que el motor terminara
    // con código 9 —error de uso— y ninguna prueba lo detectaba porque no había motor.
    //
    // La dirección es obligatoria en los dos roles: el emisor indica a dónde conecta y
    // el receptor a qué interfaz se liga. `*` deja la afinidad de procesador al sistema.
    let host = target_host.ok_or_else(|| {
        NtttcpArgError::InvalidPlan(
            "Se requiere dirección: destino para el emisor, interfaz local para el receptor"
                .to_string(),
        )
    })?;
    if host.is_empty()
        || host
            .chars()
            .any(|c| c.is_whitespace() || c == '"' || c == ';' || c == ',')
    {
        return Err(NtttcpArgError::IllegalFlag(
            "Caracteres inválidos en la dirección".to_string(),
        ));
    }
    args.push("-m".to_string());
    args.push(format!("{},*,{}", plan.streams, host));

    // 2b. Puerto base. Cada stream usa `puerto + i` (V-01).
    args.push("-p".to_string());
    args.push(plan.port.to_string());

    // 3. Tamaño de buffer: por defecto 64KB (65536) para TCP, o datagrama UDP (1472 por defecto)
    let buffer_size = if plan.protocol == BenchmarkProtocol::Udp {
        plan.udp_packet_size_bytes
            .map(|s| s.to_string())
            .unwrap_or_else(|| "1472".to_string())
    } else {
        plan.buffer_size_bytes
            .as_deref()
            .unwrap_or("65536")
            .to_string()
    };
    args.push("-l".to_string());
    args.push(buffer_size);

    // 4. Modo asíncrono (-a 8)
    args.push("-a".to_string());
    args.push("8".to_string());

    // 5. Duración de medición (-t en segundos)
    args.push("-t".to_string());
    args.push(plan.measure_seconds.to_string());

    // 6. Calentamiento (-wu en segundos)
    if plan.warmup_seconds > 0 {
        args.push("-wu".to_string());
        args.push(plan.warmup_seconds.to_string());
    }

    // 7. Enfriamiento (-cd en segundos)
    if plan.cooldown_seconds > 0 {
        args.push("-cd".to_string());
        args.push(plan.cooldown_seconds.to_string());
    }

    // 8. Protocolo UDP si aplica (-u)
    if plan.protocol == BenchmarkProtocol::Udp {
        args.push("-u".to_string());
    }

    // 9. Archivo de salida XML (-xml)
    let path_str = xml_output_file.to_str().ok_or_else(|| {
        NtttcpArgError::InvalidTargetPath("Ruta de archivo XML no es UTF-8".to_string())
    })?;
    args.push("-xml".to_string());
    args.push(path_str.to_string());

    Ok(args)
}
