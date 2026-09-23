use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfExportOptions {
    pub page_size: String,   // "A4"
    pub orientation: String, // "portrait" | "landscape"
    pub include_charts: bool,
    pub anonymize: bool,
    pub locale: String, // "es" | "en"
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            page_size: "A4".to_string(),
            orientation: "portrait".to_string(),
            include_charts: true,
            anonymize: false,
            locale: "es".to_string(),
        }
    }
}

/// Guarda un archivo PDF de forma atómica escribiendo primero en un fichero temporal
/// y realizando después un renombrado atómico (`fs::rename`) para evitar escrituras corruptas.
pub fn write_pdf_atomically(dest_path: &Path, pdf_bytes: &[u8]) -> Result<(), String> {
    if pdf_bytes.is_empty() {
        return Err("El contenido PDF está vacío".to_string());
    }

    let parent = dest_path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Fallo al crear directorio de destino: {e}"))?;
    }

    // Nombre temporal en el mismo directorio para garantizar renombrado atómico
    let temp_filename = format!(".tmp_export_{}", uuid::Uuid::new_v4());
    let temp_path: PathBuf = parent.join(temp_filename);

    let write_result = (|| {
        let mut file = File::create(&temp_path)
            .map_err(|e| format!("Error al crear archivo temporal: {e}"))?;
        file.write_all(pdf_bytes)
            .map_err(|e| format!("Error al escribir bytes de PDF: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Error al sincronizar disco: {e}"))?;
        Ok(())
    })();

    if let Err(e) = write_result {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    fs::rename(&temp_path, dest_path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        format!("Error al renombrar archivo temporal a destino definitivo: {e}")
    })?;

    Ok(())
}
