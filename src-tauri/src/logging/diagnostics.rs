use crate::model::result::SessionResult;

pub struct DiagnosticSanitizer;

impl DiagnosticSanitizer {
    /// Sanea y redacta información sensible de un informe de diagnóstico técnico.
    pub fn sanitize_text(input: &str) -> String {
        let mut out = String::new();
        let mut remaining = input;

        while let Some(pos) = remaining.find("sha256:") {
            out.push_str(&remaining[..pos]);
            let after = &remaining[pos + 7..];
            // Si le siguen al menos 64 caracteres hex
            if after.len() >= 64 && after[..64].chars().all(|c| c.is_ascii_hexdigit()) {
                out.push_str("sha256:");
                out.push_str(&after[..8]);
                out.push_str("...");
                remaining = &after[64..];
            } else {
                out.push_str("sha256:");
                remaining = after;
            }
        }
        out.push_str(remaining);
        out
    }

    /// Genera un informe textual técnico listo para copiar o exportar
    pub fn build_technical_report(result: &SessionResult) -> String {
        let mut report = String::new();
        report.push_str("=== INFORME DE DIAGNÓSTICO TÉCNICO - NETWORKBENCH ===\n");
        report.push_str(&format!("Versión App: {}\n", result.versions.app_version));
        report.push_str(&format!(
            "Versión Motor: {}\n",
            result.versions.engine_version
        ));
        report.push_str(&format!("ID de Sesión: {}\n", result.session_id));
        report.push_str(&format!(
            "Inicio: {} | Fin: {}\n",
            result.started_at, result.finished_at
        ));
        report.push_str(&format!("Estado: {}\n", result.status));
        report.push_str(&format!(
            "Iniciador: {} ({})\n",
            result.initiator.display_name, result.initiator.address
        ));
        report.push_str(&format!(
            "Receptor: {} ({})\n",
            result.responder.display_name, result.responder.address
        ));

        if let Some(cap) = &result.capacity {
            let ref_str = cap
                .ref_bps
                .map(|b| format!("{} bps", b))
                .unwrap_or_else(|| "No determinable".into());
            report.push_str(&format!(
                "Capacidad de Referencia: {} (Origen: {:?})\n",
                ref_str, cap.ref_source
            ));
        }

        report.push_str("\n--- DIRECCIONES ---\n");
        for dir in &result.directions {
            report.push_str(&format!(
                "Dirección: {} | Estado: {}\n",
                dir.direction, dir.status
            ));
            if let Some(bps) = &dir.official_bps {
                report.push_str(&format!("  Throughput Oficial (Receptor): {} bps\n", bps));
            }
            if let Some(u) = dir.utilization {
                report.push_str(&format!("  Utilización: {:.1}%\n", u * 100.0));
            }
            if let Some(st) = &dir.stability {
                report.push_str(&format!(
                    "  Estabilidad: {:?} (CV: {:.3}, Caídas: {})\n",
                    st.level, st.cv, st.drops_count
                ));
            }
            if let Some(rt) = &dir.retransmission {
                let ratio_str = rt
                    .ratio
                    .map(|r| format!("{:.4}", r))
                    .unwrap_or_else(|| "N/A".into());
                report.push_str(&format!(
                    "  Retransmisiones: {:?} (Ratio: {})\n",
                    rt.level, ratio_str
                ));
            }
            if let (Some(cs), Some(cr)) = (dir.cpu_sender, dir.cpu_receiver) {
                report.push_str(&format!(
                    "  CPU: Emisor: {:.1}% | Receptor: {:.1}%\n",
                    cs, cr
                ));
            }
        }

        if let Some(v) = &result.verdict {
            report.push_str("\n--- VEREDICTO DIAGNÓSTICO ---\n");
            report.push_str(&format!("Nivel Global: {:?}\n", v.level));
            report.push_str(&format!("Título: {}\n", v.title_key));
            if !v.facts.is_empty() {
                report.push_str("Hechos:\n");
                for f in &v.facts {
                    report.push_str(&format!("  - [{}] {}\n", f.rule_id, f.message_key));
                }
            }
            if !v.observations.is_empty() {
                report.push_str("Observaciones:\n");
                for o in &v.observations {
                    report.push_str(&format!("  - [{}] {}\n", o.rule_id, o.message_key));
                }
            }
            if !v.possible_causes.is_empty() {
                report.push_str("Posibles Causas:\n");
                for c in &v.possible_causes {
                    report.push_str(&format!("  - [{}] {}\n", c.rule_id, c.message_key));
                }
            }
            if !v.actions.is_empty() {
                report.push_str("Acciones Sugeridas:\n");
                for a in &v.actions {
                    report.push_str(&format!("  - [{}] {}\n", a.rule_id, a.message_key));
                }
            }
        }

        Self::sanitize_text(&report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizer_redacts_long_fingerprints() {
        let input = "Peer huella: sha256:11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff en sistema";
        let output = DiagnosticSanitizer::sanitize_text(input);
        assert_eq!(output, "Peer huella: sha256:11223344... en sistema");
    }
}
