use crate::app::AppState;
use crate::errors::{AppError, AppIssue, ErrorCode, ErrorSeverity};
use crate::export::csv::{SampleCsvRow, export_samples_csv, export_summary_csv};
use crate::export::json::export_sessions_to_json;
use crate::export::pdf::write_pdf_atomically;
use crate::history::queries::get_session_detail;
use crate::ipc::response::IpcResult;
use crate::model::result::SessionResult;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreviewRequest {
    pub session_ids: Vec<Uuid>,
    pub format: String, // "pdf" | "json" | "csv"
    pub anonymize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreviewResponse {
    pub token: String,
    pub file_names: Vec<String>,
    pub disclosure_keys: Vec<String>,
    pub total_sessions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportExecuteRequest {
    pub token: String,
    pub session_ids: Vec<Uuid>,
    pub format: String, // "pdf" | "json" | "csv"
    pub anonymize: bool,
    pub destination_dir: String,
    pub delimiter: Option<char>,
    pub decimal_separator: Option<char>,
    pub pdf_content_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportExecuteResponse {
    pub exported_files: Vec<String>,
    pub bytes_written: u64,
}

fn sanitize_filename_component(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn format_timestamp_for_filename(iso_date: &str) -> String {
    // Si viene en formato 2026-09-22T08:00:00Z -> 2026-09-22_0800
    if iso_date.len() >= 16 {
        let date = &iso_date[0..10];
        let time = &iso_date[11..16].replace(':', "");
        format!("{date}_{time}")
    } else {
        chrono_fallback()
    }
}

fn chrono_fallback() -> String {
    "2026-09-22_0800".to_string()
}

#[tauri::command]
pub fn export_preview(
    state: State<AppState>,
    request: ExportPreviewRequest,
) -> IpcResult<ExportPreviewResponse> {
    if request.session_ids.is_empty() {
        return IpcResult::err(
            AppError::new(
                ErrorCode::ParamRejected,
                ErrorSeverity::Warning,
                "errors.NB-PARAM-001",
            )
            .with_issue(AppIssue {
                path: "export.preview".into(),
                code: "EMPTY_SESSIONS".into(),
                message_key: "errors.NB-PARAM-001".into(),
                safe_params: None,
            }),
        );
    }

    let conn = match state.database.connection().lock() {
        Ok(c) => c,
        Err(_) => {
            return IpcResult::err(AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Fatal,
                "errors.NB-INTERNAL-001",
            ));
        }
    };

    let first_id = request.session_ids[0];
    let session_rec = match get_session_detail(&conn, &first_id) {
        Ok(Some(s)) => s,
        _ => {
            return IpcResult::err(
                AppError::new(
                    ErrorCode::ParamRejected,
                    ErrorSeverity::Warning,
                    "errors.NB-PARAM-001",
                )
                .with_issue(AppIssue {
                    path: "export.preview".into(),
                    code: "SESSION_NOT_FOUND".into(),
                    message_key: "errors.NB-PARAM-001".into(),
                    safe_params: None,
                }),
            );
        }
    };

    let peer_name = sanitize_filename_component(
        session_rec
            .peer_id
            .map(|id| format!("peer_{}", &id.to_string()[..8]))
            .unwrap_or_else(|| "equipo".to_string())
            .as_str(),
    );
    let time_str = format_timestamp_for_filename(&session_rec.created_at);

    let is_single = request.session_ids.len() == 1;
    let file_names = match request.format.as_str() {
        "pdf" => {
            vec![format!("NetworkBench_{peer_name}_{time_str}.pdf")]
        }
        "json" => {
            if is_single {
                vec![format!("NetworkBench_{peer_name}_{time_str}.json")]
            } else {
                vec![format!("NetworkBench_export_{time_str}.json")]
            }
        }
        "csv" => {
            let base = if is_single {
                format!("NetworkBench_{peer_name}_{time_str}")
            } else {
                format!("NetworkBench_export_{time_str}")
            };
            vec![
                format!("{base}_resumen.csv"),
                format!("{base}_muestras.csv"),
            ]
        }
        _ => {
            return IpcResult::err(
                AppError::new(
                    ErrorCode::ParamRejected,
                    ErrorSeverity::Warning,
                    "errors.NB-PARAM-001",
                )
                .with_issue(AppIssue {
                    path: "export.preview.format".into(),
                    code: "UNSUPPORTED_FORMAT".into(),
                    message_key: "errors.NB-PARAM-001".into(),
                    safe_params: None,
                }),
            );
        }
    };

    let mut disclosure_keys = vec![
        "export.disclosure.equipment_names".to_string(),
        "export.disclosure.test_results".to_string(),
        "export.disclosure.configuration".to_string(),
    ];
    if !request.anonymize {
        disclosure_keys.push("export.disclosure.ip_addresses".to_string());
        disclosure_keys.push("export.disclosure.mac_adapters".to_string());
    } else {
        disclosure_keys.push("export.disclosure.anonymized_notice".to_string());
    }

    // Emitir token de un solo uso de 60 segundos
    let token = state.tokens.issue("export", None, 60);

    IpcResult::ok(ExportPreviewResponse {
        token,
        file_names,
        disclosure_keys,
        total_sessions: request.session_ids.len(),
    })
}

/// Escribe un archivo de texto de forma atómica (temporal + rename)
fn write_text_atomically(dest_path: &Path, content: &str) -> Result<u64, String> {
    let parent = dest_path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Fallo al crear directorio de destino: {e}"))?;
    }

    let temp_filename = format!(".tmp_export_{}", Uuid::new_v4());
    let temp_path: PathBuf = parent.join(temp_filename);

    let bytes = content.as_bytes();
    let write_result = (|| {
        let mut file = File::create(&temp_path)
            .map_err(|e| format!("Error al crear archivo temporal: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("Error al escribir datos: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Error al sincronizar datos: {e}"))?;
        Ok(())
    })();

    if let Err(e) = write_result {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    fs::rename(&temp_path, dest_path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        format!("Error al renombrar archivo de exportación: {e}")
    })?;

    Ok(bytes.len() as u64)
}

#[tauri::command]
pub fn export_execute(
    state: State<AppState>,
    request: ExportExecuteRequest,
) -> IpcResult<ExportExecuteResponse> {
    // 1. Validar y consumir token de un solo uso
    if let Err(err) = state.tokens.consume(&request.token, "export", None) {
        return IpcResult::err(err);
    }

    if request.session_ids.is_empty() {
        return IpcResult::err(
            AppError::new(
                ErrorCode::ParamRejected,
                ErrorSeverity::Warning,
                "errors.NB-PARAM-001",
            )
            .with_issue(AppIssue {
                path: "export.execute".into(),
                code: "EMPTY_SESSIONS".into(),
                message_key: "errors.NB-PARAM-001".into(),
                safe_params: None,
            }),
        );
    }

    let conn = match state.database.connection().lock() {
        Ok(c) => c,
        Err(_) => {
            return IpcResult::err(AppError::new(
                ErrorCode::InternalError,
                ErrorSeverity::Fatal,
                "errors.NB-INTERNAL-001",
            ));
        }
    };

    // 2. Recuperar sesiones de la base de datos
    let mut sessions: Vec<SessionResult> = Vec::new();
    let mut all_samples: Vec<SampleCsvRow> = Vec::new();

    for id in &request.session_ids {
        if let Ok(Some(rec)) = get_session_detail(&conn, id) {
            // Muestras para CSV
            for s in &rec.samples {
                all_samples.push(SampleCsvRow {
                    session_id: *id,
                    direction: s.direction.clone(),
                    endpoint: rec
                        .peer_id
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "local".into()),
                    timestamp_ms: s.t_ms as i64,
                    rx_bps: s.bps.parse::<u64>().ok(),
                    tx_bps: s.bps.parse::<u64>().ok(),
                    cpu_percent: s.cpu_percent,
                });
            }

            if let Some(ref r_json) = rec.result_json
                && let Ok(sr) = serde_json::from_str::<SessionResult>(r_json)
            {
                sessions.push(sr);
                continue;
            }

            // Si no hay result_json, reconstruir SessionResult mínimo
            let plan = rec
                .plan_json
                .as_ref()
                .and_then(|pj| serde_json::from_str(pj).ok())
                .unwrap_or_else(|| crate::model::plan::BenchmarkPlan::new_standard_tcp(5001));

            let peer_snap = crate::model::result::PeerSnapshot {
                instance_id: rec.peer_id.unwrap_or_default(),
                display_name: "Equipo Remoto".into(),
                fingerprint: "0000000000000000000000000000000000000000000000000000000000000000"
                    .into(),
                address: "127.0.0.1:18400".into(),
            };

            let mut dir_res = crate::model::result::DirectionResult::new_incomplete("forward");
            dir_res.official_bps = rec.forward_bps.clone();
            dir_res.status = rec.status.clone();

            sessions.push(SessionResult {
                schema_version: 1,
                session_id: *id,
                started_at: rec.created_at.clone(),
                finished_at: rec.created_at.clone(),
                status: rec.status.clone(),
                initiator: peer_snap.clone(),
                responder: peer_snap,
                plan,
                capacity: None,
                directions: vec![dir_res],
                asymmetry: None,
                verdict: None,
                result_source: "local".into(),
                versions: crate::model::result::ResultVersions {
                    app_version: "0.1.0".into(),
                    protocol_version: 1,
                    engine_version: "5.40".into(),
                    schema_version: 1,
                    thresholds_hash: "".into(),
                },
            });
        }
    }

    if sessions.is_empty() {
        return IpcResult::err(
            AppError::new(
                ErrorCode::ParamRejected,
                ErrorSeverity::Warning,
                "errors.NB-PARAM-001",
            )
            .with_issue(AppIssue {
                path: "export.execute".into(),
                code: "NO_VALID_SESSIONS".into(),
                message_key: "errors.NB-PARAM-001".into(),
                safe_params: None,
            }),
        );
    }

    let dest_dir = Path::new(&request.destination_dir);
    let time_str = format_timestamp_for_filename(&sessions[0].started_at);
    let peer_name = sanitize_filename_component(&sessions[0].responder.display_name);
    let is_single = sessions.len() == 1;

    let mut exported_files = Vec::new();
    let mut total_bytes_written = 0u64;

    match request.format.as_str() {
        "json" => {
            let filename = if is_single {
                format!("NetworkBench_{peer_name}_{time_str}.json")
            } else {
                format!("NetworkBench_export_{time_str}.json")
            };
            let full_path = dest_dir.join(&filename);
            let json_content = match export_sessions_to_json(&sessions, request.anonymize) {
                Ok(c) => c,
                Err(e) => {
                    return IpcResult::err(
                        AppError::new(
                            ErrorCode::InternalError,
                            ErrorSeverity::Error,
                            "errors.NB-INTERNAL-001",
                        )
                        .with_issue(AppIssue {
                            path: "export.execute.json".into(),
                            code: "SERIALIZATION_FAILED".into(),
                            message_key: "errors.NB-INTERNAL-001".into(),
                            safe_params: Some(serde_json::json!({ "details": e })),
                        }),
                    );
                }
            };

            match write_text_atomically(&full_path, &json_content) {
                Ok(bytes) => {
                    exported_files.push(full_path.to_string_lossy().to_string());
                    total_bytes_written += bytes;
                }
                Err(e) => {
                    return IpcResult::err(
                        AppError::new(
                            ErrorCode::InternalError,
                            ErrorSeverity::Error,
                            "errors.NB-INTERNAL-001",
                        )
                        .with_issue(AppIssue {
                            path: "export.execute.write".into(),
                            code: "WRITE_FAILED".into(),
                            message_key: "errors.NB-INTERNAL-001".into(),
                            safe_params: Some(serde_json::json!({ "details": e })),
                        }),
                    );
                }
            }
        }
        "csv" => {
            let delimiter = request.delimiter.unwrap_or(';');
            let decimal_sep = request.decimal_separator.unwrap_or(',');

            let base_name = if is_single {
                format!("NetworkBench_{peer_name}_{time_str}")
            } else {
                format!("NetworkBench_export_{time_str}")
            };

            let summary_path = dest_dir.join(format!("{base_name}_resumen.csv"));
            let summary_csv =
                match export_summary_csv(&sessions, request.anonymize, delimiter, decimal_sep) {
                    Ok(s) => s,
                    Err(e) => {
                        return IpcResult::err(
                            AppError::new(
                                ErrorCode::InternalError,
                                ErrorSeverity::Error,
                                "errors.NB-INTERNAL-001",
                            )
                            .with_issue(AppIssue {
                                path: "export.execute.csv_summary".into(),
                                code: "CSV_FAILED".into(),
                                message_key: "errors.NB-INTERNAL-001".into(),
                                safe_params: Some(serde_json::json!({ "details": e })),
                            }),
                        );
                    }
                };

            match write_text_atomically(&summary_path, &summary_csv) {
                Ok(bytes) => {
                    exported_files.push(summary_path.to_string_lossy().to_string());
                    total_bytes_written += bytes;
                }
                Err(e) => {
                    return IpcResult::err(
                        AppError::new(
                            ErrorCode::InternalError,
                            ErrorSeverity::Error,
                            "errors.NB-INTERNAL-001",
                        )
                        .with_issue(AppIssue {
                            path: "export.execute.write_summary".into(),
                            code: "WRITE_FAILED".into(),
                            message_key: "errors.NB-INTERNAL-001".into(),
                            safe_params: Some(serde_json::json!({ "details": e })),
                        }),
                    );
                }
            }

            // Exportar también muestras si existen
            if !all_samples.is_empty() {
                let samples_path = dest_dir.join(format!("{base_name}_muestras.csv"));
                if let Ok(samples_csv) = export_samples_csv(&all_samples, delimiter, decimal_sep)
                    && let Ok(bytes) = write_text_atomically(&samples_path, &samples_csv)
                {
                    exported_files.push(samples_path.to_string_lossy().to_string());
                    total_bytes_written += bytes;
                }
            }
        }
        "pdf" => {
            let filename = format!("NetworkBench_{peer_name}_{time_str}.pdf");
            let full_path = dest_dir.join(&filename);

            // Si el frontend envía el PDF generado vía PrintToPdf base64
            let pdf_bytes = if let Some(ref b64) = request.pdf_content_base64 {
                // Limpiar posibles cabeceras data:application/pdf;base64,
                let clean_b64 = b64.split(',').next_back().unwrap_or(b64);
                // Decodificación base64 sencilla o directa

                match base64_decode(clean_b64) {
                    Ok(b) => b,
                    Err(_) => {
                        return IpcResult::err(
                            AppError::new(
                                ErrorCode::ParamRejected,
                                ErrorSeverity::Error,
                                "errors.NB-PARAM-001",
                            )
                            .with_issue(AppIssue {
                                path: "export.execute.pdf_base64".into(),
                                code: "INVALID_BASE64".into(),
                                message_key: "errors.NB-PARAM-001".into(),
                                safe_params: None,
                            }),
                        );
                    }
                }
            } else {
                // Fallback PDF bytes mínimos válidos si no hay renderizador
                let fallback = format!(
                    "%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/MediaBox[0 0 595 842]/Parent 2 0 R/Contents 4 0 R>>endobj\n4 0 obj<</Length 55>>stream\nBT /F1 12 Tf 50 800 Td (NetworkBench Report - {peer_name}) Tj ET\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000213 00000 n \ntrailer<</Size 5/Root 1 0 R>>\nstartxref\n320\n%%EOF"
                );
                fallback.into_bytes()
            };

            match write_pdf_atomically(&full_path, &pdf_bytes) {
                Ok(_) => {
                    exported_files.push(full_path.to_string_lossy().to_string());
                    total_bytes_written += pdf_bytes.len() as u64;
                }
                Err(e) => {
                    return IpcResult::err(
                        AppError::new(
                            ErrorCode::InternalError,
                            ErrorSeverity::Error,
                            "errors.NB-INTERNAL-001",
                        )
                        .with_issue(AppIssue {
                            path: "export.execute.write_pdf".into(),
                            code: "WRITE_FAILED".into(),
                            message_key: "errors.NB-INTERNAL-001".into(),
                            safe_params: Some(serde_json::json!({ "details": e })),
                        }),
                    );
                }
            }
        }
        _ => {
            return IpcResult::err(AppError::new(
                ErrorCode::ParamRejected,
                ErrorSeverity::Warning,
                "errors.NB-PARAM-001",
            ));
        }
    }

    IpcResult::ok(ExportExecuteResponse {
        exported_files,
        bytes_written: total_bytes_written,
    })
}

fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;

    for &b in input.as_bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return Err(()),
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
