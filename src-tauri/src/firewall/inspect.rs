use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuleStatus {
    Present,
    Missing,
    Modified,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallInspection {
    pub rule_name: String,
    pub status: RuleStatus,
    pub is_policy_managed: bool,
    pub details: Option<String>,
}

pub struct FirewallInspector;

impl FirewallInspector {
    /// Inspecciona el estado de una regla en el Firewall de Windows sin elevación (ADR-002, ADR-005, §14.3)
    pub fn inspect_rule(
        rule_name: &str,
        expected_port_range: &str,
        expected_proto: &str,
    ) -> FirewallInspection {
        #[cfg(target_os = "windows")]
        {
            Self::inspect_windows(rule_name, expected_port_range, expected_proto)
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (expected_port_range, expected_proto);
            FirewallInspection {
                rule_name: rule_name.to_string(),
                status: RuleStatus::Present,
                is_policy_managed: false,
                details: Some("Plataforma no Windows (inspección simulada)".to_string()),
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn inspect_windows(
        rule_name: &str,
        expected_port_range: &str,
        expected_proto: &str,
    ) -> FirewallInspection {
        let output = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "show",
                "rule",
                &format!("name={}", rule_name),
            ])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                if !out.status.success()
                    || stdout.contains("No rules match")
                    || stdout.contains("No coincide ninguna regla")
                {
                    return FirewallInspection {
                        rule_name: rule_name.to_string(),
                        status: RuleStatus::Missing,
                        is_policy_managed: false,
                        details: Some("Regla ausente en el firewall".to_string()),
                    };
                }

                let enabled = stdout.contains("Enabled:\t\t\t\tYes")
                    || stdout.contains("Habilitado:\t\t\t\tSí")
                    || stdout.contains("Habilitado:\t\t\t\tSi")
                    || stdout.contains("Enabled: Yes")
                    || stdout.contains("Habilitado: S");

                if !enabled {
                    return FirewallInspection {
                        rule_name: rule_name.to_string(),
                        status: RuleStatus::Disabled,
                        is_policy_managed: false,
                        details: Some("Regla encontrada pero deshabilitada".to_string()),
                    };
                }

                let proto_match = stdout
                    .to_lowercase()
                    .contains(&expected_proto.to_lowercase());
                let port_match =
                    expected_port_range.is_empty() || stdout.contains(expected_port_range);

                if !proto_match || !port_match {
                    return FirewallInspection {
                        rule_name: rule_name.to_string(),
                        status: RuleStatus::Modified,
                        is_policy_managed: false,
                        details: Some("Regla con parámetros distintos a los esperados".to_string()),
                    };
                }

                FirewallInspection {
                    rule_name: rule_name.to_string(),
                    status: RuleStatus::Present,
                    is_policy_managed: false,
                    details: Some("Regla presente y activa".to_string()),
                }
            }
            Err(e) => FirewallInspection {
                rule_name: rule_name.to_string(),
                status: RuleStatus::Missing,
                is_policy_managed: false,
                details: Some(format!("No se pudo consultar netsh: {}", e)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_nonexistent_rule() {
        let inspection =
            FirewallInspector::inspect_rule("ReglaInexistente_NetworkBench_XYZ_123", "5201", "TCP");
        assert_eq!(inspection.status, RuleStatus::Missing);
        assert!(!inspection.is_policy_managed);
    }
}
