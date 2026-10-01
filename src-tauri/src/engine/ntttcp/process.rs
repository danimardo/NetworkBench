use super::args::build_ntttcp_args;
use super::job_object::JobObject;
use super::parser::{NtttcpParsedResult, NtttcpRole, parse_ntttcp_xml};
use crate::model::plan::BenchmarkPlan;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use uuid::Uuid;

/// Recorte para lo que se escribe en el log: `ntttcp.exe` no debería producir megabytes de
/// texto, pero un recorte defiende el fichero de registro de un caso patológico.
const MAX_SALIDA_EN_LOG: usize = 4000;

fn recortar(texto: &str) -> String {
    if texto.len() <= MAX_SALIDA_EN_LOG {
        texto.to_string()
    } else {
        format!(
            "{}… (recortado, {} bytes en total)",
            &texto[..MAX_SALIDA_EN_LOG],
            texto.len()
        )
    }
}

pub struct NtttcpProcess {
    child: Option<Child>,
    xml_path: PathBuf,
    _job: JobObject,
    /// Para el log si falla: rol, ruta y argumentos exactos con los que se lanzó.
    role: NtttcpRole,
    exe_path: PathBuf,
    args: Vec<String>,
}

impl NtttcpProcess {
    pub fn verify_executable_hash(exe_path: &Path, expected_sha256: &str) -> Result<bool> {
        let bytes = fs::read(exe_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hasher.finalize();
        let actual_hex = hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Ok(actual_hex.eq_ignore_ascii_case(expected_sha256.trim()))
    }

    pub async fn spawn(
        exe_path: &Path,
        role: NtttcpRole,
        plan: &BenchmarkPlan,
        target_host: Option<&str>,
        temp_dir: &Path,
    ) -> Result<Self> {
        // ntttcp.exe no crea la carpeta del XML: si no existe aborta al arrancar con
        // «fopen XML File, GetLastError: 3» (código 9) antes de medir nada. En una
        // instalación limpia `tmp` no existe hasta que alguien la crea.
        fs::create_dir_all(temp_dir)?;
        let xml_path = temp_dir.join(format!("ntttcp_{}.xml", Uuid::new_v4()));

        let args = build_ntttcp_args(role, plan, target_host, &xml_path).map_err(|e| {
            tracing::warn!("NTTTCP ({role:?}): argumentos no válidos: {e:?}");
            Error::new(ErrorKind::InvalidInput, format!("{:?}", e))
        })?;

        let job = JobObject::create_kill_on_close()?;

        let mut cmd = Command::new(exe_path);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        {
            // Ocultar ventana de consola
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        // Hallazgo real de la auditoría de logging (2026-09-28): sin esto, el hueco más
        // grave de todos — ni el comando exacto, ni sus argumentos, ni la salida del
        // proceso quedaban en ningún sitio. Un test que fallaba a mitad no dejaba forma
        // de reconstruir qué se había lanzado ni qué dijo ntttcp.exe por su cuenta.
        //
        // La línea de comandos completa (incluye la IP remota en los argumentos) es
        // diagnóstico avanzado (principio XIII, enmienda 0.8.0): solo a debug. El hito
        // "arrancando" sin el comando va a info.
        tracing::info!("NTTTCP ({role:?}): arrancando");
        tracing::debug!(
            "NTTTCP ({role:?}): lanzando {} {}",
            exe_path.display(),
            args.join(" ")
        );

        let child = cmd.spawn().inspect_err(|e| {
            tracing::warn!("NTTTCP ({role:?}): no se pudo lanzar el proceso");
            tracing::debug!("NTTTCP ({role:?}): no se pudo lanzar el proceso: {e}");
        })?;

        #[cfg(windows)]
        if let Some(raw_handle) = child.raw_handle() {
            let _ = job.assign_process(windows::Win32::Foundation::HANDLE(raw_handle as _));
        }

        Ok(Self {
            child: Some(child),
            xml_path,
            _job: job,
            role,
            exe_path: exe_path.to_path_buf(),
            args,
        })
    }

    pub async fn wait_and_parse(mut self) -> Result<NtttcpParsedResult> {
        if let Some(mut child) = self.child.take() {
            // stdout/stderr se leen A LA VEZ que se espera, no después: con Stdio::piped()
            // y nadie leyendo, un proceso que escribe más que el búfer del pipe del
            // sistema (típicamente 64 KiB en Windows) se queda bloqueado esperando sitio
            // — un bloqueo real que este cambio evita de paso, no solo un problema de log.
            let mut salida_estandar = child.stdout.take();
            let mut salida_error = child.stderr.take();
            let leer_stdout = async {
                let mut s = String::new();
                if let Some(p) = salida_estandar.as_mut() {
                    let _ = p.read_to_string(&mut s).await;
                }
                s
            };
            let leer_stderr = async {
                let mut s = String::new();
                if let Some(p) = salida_error.as_mut() {
                    let _ = p.read_to_string(&mut s).await;
                }
                s
            };
            let (status, stdout, stderr) = tokio::join!(child.wait(), leer_stdout, leer_stderr);
            let status = status?;

            if !status.success() {
                let _ = fs::remove_file(&self.xml_path);
                tracing::warn!(
                    "NTTTCP ({:?}): terminó con código {:?}",
                    self.role,
                    status.code()
                );
                // Comando completo y stdout/stderr (pueden citar la IP remota): diagnóstico
                // avanzado (principio XIII, enmienda 0.8.0), solo a debug.
                tracing::debug!(
                    "NTTTCP ({:?}): terminó con código {:?} — comando: {} {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
                    self.role,
                    status.code(),
                    self.exe_path.display(),
                    self.args.join(" "),
                    recortar(&stdout),
                    recortar(&stderr)
                );
                return Err(Error::other(format!(
                    "ntttcp terminó con código de error: {:?}",
                    status.code()
                )));
            }
            tracing::info!("NTTTCP ({:?}): completado (código 0)", self.role);

            let xml_content = fs::read_to_string(&self.xml_path)?;
            let _ = fs::remove_file(&self.xml_path);

            parse_ntttcp_xml(&xml_content).map_err(|e| {
                tracing::warn!("NTTTCP ({:?}): el XML de resultado no se pudo interpretar: {e:?}", self.role);
                tracing::debug!(
                    "NTTTCP ({:?}): XML no interpretable — stdout/stderr:\n--- stdout ---\n{}\n--- stderr ---\n{}",
                    self.role,
                    recortar(&stdout),
                    recortar(&stderr)
                );
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Fallo al parsear XML: {:?}", e),
                )
            })
        } else {
            Err(Error::other("Proceso ya no está activo"))
        }
    }

    pub async fn kill_and_cleanup(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
        }
        if self.xml_path.exists() {
            let _ = fs::remove_file(&self.xml_path);
        }
        Ok(())
    }
}

impl Drop for NtttcpProcess {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.start_kill();
        }
        if self.xml_path.exists() {
            let _ = fs::remove_file(&self.xml_path);
        }
    }
}
