use super::args::build_ntttcp_args;
use super::job_object::JobObject;
use super::parser::{NtttcpParsedResult, NtttcpRole, parse_ntttcp_xml};
use crate::model::plan::BenchmarkPlan;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};
use uuid::Uuid;

pub struct NtttcpProcess {
    child: Option<Child>,
    xml_path: PathBuf,
    _job: JobObject,
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
        let xml_path = temp_dir.join(format!("ntttcp_{}.xml", Uuid::new_v4()));

        let args = build_ntttcp_args(role, plan, target_host, &xml_path)
            .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("{:?}", e)))?;

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

        let child = cmd.spawn()?;

        #[cfg(windows)]
        if let Some(raw_handle) = child.raw_handle() {
            let _ = job.assign_process(windows::Win32::Foundation::HANDLE(raw_handle as _));
        }

        Ok(Self {
            child: Some(child),
            xml_path,
            _job: job,
        })
    }

    pub async fn wait_and_parse(mut self) -> Result<NtttcpParsedResult> {
        if let Some(mut child) = self.child.take() {
            let status = child.wait().await?;
            if !status.success() {
                let _ = fs::remove_file(&self.xml_path);
                return Err(Error::other(format!(
                    "ntttcp terminó con código de error: {:?}",
                    status.code()
                )));
            }

            let xml_content = fs::read_to_string(&self.xml_path)?;
            let _ = fs::remove_file(&self.xml_path);

            parse_ntttcp_xml(&xml_content).map_err(|e| {
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
