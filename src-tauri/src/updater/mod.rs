use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestPlatform {
    pub signature: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub pub_date: Option<String>,
    pub platforms: HashMap<String, ManifestPlatform>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable {
        version: String,
        notes: String,
        download_url: String,
        signature: String,
    },
    DeferredDueToActiveSession,
}

/// Parsea una versión SemVer básica `major.minor.patch` (opcionalmente con prefijo 'v')
pub fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let clean = v.trim().strip_prefix('v').unwrap_or(v.trim());
    let mut parts = clean.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    let patch_part = parts.next()?.split(&['-', '+'][..]).next()?;
    let patch = patch_part.parse::<u32>().ok()?;
    Some((major, minor, patch))
}

/// Determina si `candidate` es estrictamente mayor que `current`
pub fn is_newer_version(current: &str, candidate: &str) -> bool {
    match (parse_version(current), parse_version(candidate)) {
        (Some((c_maj, c_min, c_pat)), Some((t_maj, t_min, t_pat))) => {
            (t_maj, t_min, t_pat) > (c_maj, c_min, c_pat)
        }
        _ => false,
    }
}

/// Valida las restricciones de ADR-007 sobre URLs de actualización:
/// 1. Debe ser HTTPS
/// 2. Debe ser una URL inmutable versionada (`/releases/download/vX.Y.Z/` o similar)
/// 3. Prohíbe explícitamente enlaces flotantes mutables (`/latest/`, `latest.exe`)
pub fn validate_download_url(url: &str, candidate_version: &str) -> Result<(), &'static str> {
    if !url.starts_with("https://") {
        return Err("Download URL must use HTTPS");
    }

    let url_lower = url.to_lowercase();
    if url_lower.contains("/latest/") || url_lower.ends_with("latest.exe") {
        return Err("Mutable 'latest' download URLs are prohibited by ADR-007");
    }

    let expected_tag_v = format!("releases/download/v{}/", candidate_version);
    let expected_tag = format!("releases/download/{}/", candidate_version);
    if !url.contains(&expected_tag_v) && !url.contains(&expected_tag) {
        return Err("Download URL must point to an immutable release asset tagged with candidate version");
    }

    if !url.ends_with(".exe") && !url.ends_with(".msi") && !url.ends_with(".zip") {
        return Err("Download URL must end with supported installer artifact extension (.exe, .msi, .zip)");
    }

    Ok(())
}

/// Evalúa el contenido del manifiesto según el estado actual de la aplicación
pub fn evaluate_manifest(
    manifest_json: &str,
    current_version: &str,
    target_platform: &str,
    is_session_active: bool,
) -> Result<UpdateStatus, String> {
    // Si hay una sesión activa de medición, cualquier comprobación o actualización
    // se pospone obligatoriamente para no interferir en el benchmark (FR-042 / ADR-007)
    if is_session_active {
        return Ok(UpdateStatus::DeferredDueToActiveSession);
    }

    let manifest: UpdateManifest = serde_json::from_str(manifest_json)
        .map_err(|e| format!("Invalid manifest JSON: {}", e))?;

    if !is_newer_version(current_version, &manifest.version) {
        return Ok(UpdateStatus::UpToDate);
    }

    let platform = manifest
        .platforms
        .get(target_platform)
        .ok_or_else(|| format!("Platform '{}' not found in update manifest", target_platform))?;

    if platform.signature.trim().is_empty() {
        return Err("Missing cryptographic signature for platform artifact".to_string());
    }

    validate_download_url(&platform.url, &manifest.version)
        .map_err(|e| format!("Invalid artifact URL: {}", e))?;

    Ok(UpdateStatus::UpdateAvailable {
        version: manifest.version,
        notes: manifest.notes.unwrap_or_default(),
        download_url: platform.url.clone(),
        signature: platform.signature.clone(),
    })
}
