use crate::errors::{AppError, ErrorCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallHelperRequest {
    pub operation: String, // "add" | "remove"
    pub rule_name: String,
    pub protocol: String,   // "TCP" | "UDP"
    pub port_range: String, // e.g. "5001-5064" o "5201"
    pub program: String,
    pub profiles: Vec<String>, // "Domain", "Private", "Public"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallHelperResponse {
    pub success: bool,
    pub error_code: Option<String>,
    pub message: String,
}

pub struct FirewallHelperClient;

impl FirewallHelperClient {
    /// Obtiene la ruta al binario del helper auxiliar
    /// Localiza el helper elevado.
    ///
    /// Hay dos disposiciones reales y las dos importan:
    ///
    /// - **En desarrollo**, cargo deja el helper junto al ejecutable principal.
    /// - **Instalado**, va declarado como recurso de Tauri y queda en `resources/`.
    ///   No es sidecar porque `tauri-build` valida los recursos durante la compilación
    ///   del crate y el helper es ese mismo crate (ver `scripts/package/release.mjs`).
    ///
    /// Buscar solo junto al ejecutable funcionaba en desarrollo y fallaba tras instalar.
    pub fn get_helper_path() -> PathBuf {
        const NOMBRE: &str = "networkbench-firewall-helper.exe";

        if let Ok(current_exe) = env::current_exe()
            && let Some(parent) = current_exe.parent()
        {
            for candidato in [parent.join(NOMBRE), parent.join("resources").join(NOMBRE)] {
                if candidato.exists() {
                    return candidato;
                }
            }
        }
        PathBuf::from(NOMBRE)
    }

    /// Valida que una solicitud cumple la lista blanca estricta (ADR-002, ADR-005, §14.2)
    pub fn validate_request(req: &FirewallHelperRequest) -> Result<(), String> {
        if !req.rule_name.starts_with("NetworkBench - ") {
            return Err("El nombre de la regla debe comenzar por 'NetworkBench - '".to_string());
        }

        if req.operation != "add" && req.operation != "remove" {
            return Err(format!("Operación no permitida: {}", req.operation));
        }

        let prog_lower = req.program.to_lowercase();
        let valid_program = prog_lower.ends_with("networkbench.exe")
            || prog_lower.ends_with("ntttcp.exe")
            || prog_lower.is_empty();

        if !valid_program {
            return Err(format!(
                "Programa no autorizado en lista blanca: {}",
                req.program
            ));
        }

        // Validar rango de puertos <= 64 puertos
        if req.port_range.contains('-') {
            let parts: Vec<&str> = req.port_range.split('-').collect();
            if parts.len() == 2
                && let (Ok(start), Ok(end)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>())
                && (end < start || (end - start + 1) > 64)
            {
                return Err(format!(
                    "Rango de puertos excede el límite de 64: {}",
                    req.port_range
                ));
            }
        }

        Ok(())
    }

    /// Aplica una lista de reglas mediante el helper con elevación UAC (ADR-002, ADR-005)
    pub fn apply_rules(requests: &[FirewallHelperRequest]) -> Result<(), AppError> {
        for req in requests {
            if let Err(e) = Self::validate_request(req) {
                return Err(AppError::from_code(ErrorCode::PermissionDenied).with_diagnostic_id(e));
            }
        }

        let temp_dir = env::temp_dir();
        let temp_req_file = temp_dir.join(format!("nb_fw_req_{}.json", std::process::id()));
        let temp_res_file = temp_dir.join(format!("nb_fw_res_{}.json", std::process::id()));

        let json_data = serde_json::to_string_pretty(requests).map_err(|e| {
            AppError::from_code(ErrorCode::InternalError).with_diagnostic_id(e.to_string())
        })?;

        if let Err(e) = fs::write(&temp_req_file, json_data) {
            return Err(
                AppError::from_code(ErrorCode::InternalError).with_diagnostic_id(e.to_string())
            );
        }

        let helper_path = Self::get_helper_path();

        #[cfg(target_os = "windows")]
        {
            // Lanzamiento con runas mediante PowerShell / ShellExecuteEx para elevación UAC
            let status = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "Start-Process -FilePath '{}' -ArgumentList '--file \"{}\" --out \"{}\"' -Verb RunAs -Wait -PassThru | Select-Object -ExpandProperty ExitCode",
                        helper_path.display(),
                        temp_req_file.display(),
                        temp_res_file.display()
                    ),
                ])
                .output();

            let _ = fs::remove_file(&temp_req_file);

            match status {
                Ok(out) => {
                    let out_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !out.status.success() || out_str != "0" {
                        let _ = fs::remove_file(&temp_res_file);
                        return Err(AppError::from_code(ErrorCode::FirewallUacRejected));
                    }
                }
                Err(_) => {
                    let _ = fs::remove_file(&temp_res_file);
                    return Err(AppError::from_code(ErrorCode::FirewallUacRejected));
                }
            }

            let _ = fs::remove_file(&temp_res_file);
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (temp_req_file, temp_res_file, helper_path);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowlist_validation_valid_rule() {
        let req = FirewallHelperRequest {
            operation: "add".to_string(),
            rule_name: "NetworkBench - NTTTCP TCP".to_string(),
            protocol: "TCP".to_string(),
            port_range: "5001-5064".to_string(),
            program: "C:\\Program Files\\NetworkBench\\engine\\ntttcp.exe".to_string(),
            profiles: vec!["Private".to_string(), "Domain".to_string()],
        };
        assert!(FirewallHelperClient::validate_request(&req).is_ok());
    }

    #[test]
    fn test_allowlist_validation_rejects_disallowed_prefix() {
        let req = FirewallHelperRequest {
            operation: "add".to_string(),
            rule_name: "Malicious - Rule".to_string(),
            protocol: "TCP".to_string(),
            port_range: "5001-5064".to_string(),
            program: "engine\\ntttcp.exe".to_string(),
            profiles: vec!["Private".to_string()],
        };
        assert!(FirewallHelperClient::validate_request(&req).is_err());
    }

    #[test]
    fn test_allowlist_validation_rejects_too_large_port_range() {
        let req = FirewallHelperRequest {
            operation: "add".to_string(),
            rule_name: "NetworkBench - NTTTCP TCP".to_string(),
            protocol: "TCP".to_string(),
            port_range: "5000-6000".to_string(), // 1001 ports (> 64)
            program: "engine\\ntttcp.exe".to_string(),
            profiles: vec!["Private".to_string()],
        };
        assert!(FirewallHelperClient::validate_request(&req).is_err());
    }
}
