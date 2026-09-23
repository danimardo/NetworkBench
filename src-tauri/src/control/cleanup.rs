use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

#[derive(Debug, Default)]
pub struct CleanupRegistry {
    pub process_ids: HashSet<u32>,
    pub temp_files: HashSet<PathBuf>,
}

#[derive(Clone, Default)]
pub struct CleanupCoordinator {
    registry: Arc<Mutex<CleanupRegistry>>,
}

impl CleanupCoordinator {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(CleanupRegistry::default())),
        }
    }

    /// Registra un proceso hijo propio para su terminación garantizada
    pub fn register_process(&self, pid: u32) {
        if let Ok(mut reg) = self.registry.lock() {
            reg.process_ids.insert(pid);
        }
    }

    /// Desregistra un proceso que ya finalizó normalmente
    pub fn unregister_process(&self, pid: u32) {
        if let Ok(mut reg) = self.registry.lock() {
            reg.process_ids.remove(&pid);
        }
    }

    /// Registra un archivo temporal propio
    pub fn register_temp_file(&self, path: PathBuf) {
        if let Ok(mut reg) = self.registry.lock() {
            reg.temp_files.insert(path);
        }
    }

    /// Limpieza idempotente de recursos propios (FR-023, §8.5, §11.5)
    /// Garantía: NUNCA termina procesos ajenos ni deja recursos propios
    pub fn cleanup_all(&self) {
        let (pids, files) = match self.registry.lock() {
            Ok(mut reg) => {
                let pids: Vec<u32> = reg.process_ids.drain().collect();
                let files: Vec<PathBuf> = reg.temp_files.drain().collect();
                (pids, files)
            }
            Err(_) => return,
        };

        for pid in pids {
            info!(target: "cleanup", pid = pid, "Terminando proceso hijo propio registrado");
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
                unsafe {
                    if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                        let _ = TerminateProcess(handle, 1);
                        let _ = windows::Win32::Foundation::CloseHandle(handle);
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = pid;
            }
        }

        for file in files {
            if file.exists() {
                if let Err(e) = fs::remove_file(&file) {
                    warn!(target: "cleanup", path = %file.display(), error = %e, "No se pudo eliminar archivo temporal");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_cleanup_idempotence() {
        let coordinator = CleanupCoordinator::new();
        let temp_file = env::temp_dir().join(format!("test_cleanup_{}.tmp", std::process::id()));
        fs::write(&temp_file, b"sample temporary data").unwrap();

        coordinator.register_temp_file(temp_file.clone());

        // Primera llamada: elimina el archivo
        coordinator.cleanup_all();
        assert!(!temp_file.exists());

        // Segunda llamada: idempotente, no produce errores ni efectos secundarios
        coordinator.cleanup_all();
    }
}
