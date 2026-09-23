use std::env;
use std::fs;
use std::path::Path;
use std::process::{exit, Command};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallHelperRequest {
    pub operation: String,
    pub rule_name: String,
    pub protocol: String,
    pub port_range: String,
    pub program: String,
    pub profiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallHelperResponse {
    pub success: bool,
    pub error_code: Option<String>,
    pub message: String,
}

fn validate_request(req: &FirewallHelperRequest) -> Result<(), String> {
    if !req.rule_name.starts_with("NetworkBench - ") {
        return Err("Nombre de regla inválido: debe comenzar por 'NetworkBench - '".to_string());
    }

    if req.operation != "add" && req.operation != "remove" {
        return Err(format!("Operación '{}' no autorizada", req.operation));
    }

    let prog_lower = req.program.to_lowercase();
    let valid_prog = prog_lower.ends_with("networkbench.exe") 
        || prog_lower.ends_with("ntttcp.exe") 
        || prog_lower.is_empty();

    if !valid_prog {
        return Err(format!("Programa '{}' no autorizado en lista blanca", req.program));
    }

    if req.port_range.contains('-') {
        let parts: Vec<&str> = req.port_range.split('-').collect();
        if parts.len() == 2 {
            if let (Ok(start), Ok(end)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                if end < start || (end - start + 1) > 64 {
                    return Err(format!("Rango de puertos excede el límite de 64: {}", req.port_range));
                }
            }
        }
    }

    Ok(())
}

fn apply_rule(req: &FirewallHelperRequest) -> Result<(), String> {
    validate_request(req)?;

    #[cfg(target_os = "windows")]
    {
        if req.operation == "remove" {
            let output = Command::new("netsh")
                .args(["advfirewall", "firewall", "delete", "rule", &format!("name={}", req.rule_name)])
                .output()
                .map_err(|e| format!("Fallo al ejecutar netsh delete: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("netsh delete devolvió error: {}", stderr));
            }
            return Ok(());
        }

        if req.operation == "add" {
            // Eliminar primero para evitar reglas duplicadas si ya existe
            let _ = Command::new("netsh")
                .args(["advfirewall", "firewall", "delete", "rule", &format!("name={}", req.rule_name)])
                .output();

            let profiles_str = if req.profiles.is_empty() {
                "domain,private".to_string()
            } else {
                req.profiles.join(",").to_lowercase()
            };

            let mut args = vec![
                "advfirewall".to_string(),
                "firewall".to_string(),
                "add".to_string(),
                "rule".to_string(),
                format!("name={}", req.rule_name),
                "dir=in".to_string(),
                "action=allow".to_string(),
                format!("protocol={}", req.protocol.to_lowercase()),
                format!("profile={}", profiles_str),
            ];

            if !req.port_range.is_empty() {
                args.push(format!("localport={}", req.port_range));
            }

            if !req.program.is_empty() && Path::new(&req.program).exists() {
                args.push(format!("program={}", req.program));
            }

            let output = Command::new("netsh")
                .args(&args)
                .output()
                .map_err(|e| format!("Fallo al ejecutar netsh add: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("netsh add devolvió error: {}", stderr));
            }
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut file_path: Option<String> = None;
    let mut out_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--file" && i + 1 < args.len() {
            file_path = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--out" && i + 1 < args.len() {
            out_path = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let Some(path) = file_path else {
        eprintln!("Uso: networkbench-firewall-helper --file <ruta-json> [--out <ruta-salida>]");
        exit(1);
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error leyendo archivo de solicitud: {}", e);
            exit(1);
        }
    };

    let requests: Vec<FirewallHelperRequest> = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error parseando JSON de solicitud: {}", e);
            exit(1);
        }
    };

    for req in &requests {
        if let Err(e) = apply_rule(req) {
            eprintln!("Error aplicando regla '{}': {}", req.rule_name, e);
            if let Some(out) = out_path {
                let res = FirewallHelperResponse {
                    success: false,
                    error_code: Some("NB-FW-004".to_string()),
                    message: e,
                };
                let _ = fs::write(out, serde_json::to_string(&res).unwrap_or_default());
            }
            exit(1);
        }
    }

    if let Some(out) = out_path {
        let res = FirewallHelperResponse {
            success: true,
            error_code: None,
            message: "Reglas aplicadas correctamente".to_string(),
        };
        let _ = fs::write(out, serde_json::to_string(&res).unwrap_or_default());
    }

    exit(0);
}
