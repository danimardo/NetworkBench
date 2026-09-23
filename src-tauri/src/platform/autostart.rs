use std::process::Command;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const APP_VALUE_NAME: &str = "NetworkBench";

/// Comprueba si el autoarranque en el registro de Windows está habilitado
pub fn is_autostart_enabled() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("reg")
            .args(["query", RUN_KEY, "/v", APP_VALUE_NAME])
            .output()
            .map_err(|e| format!("Failed to query registry: {}", e))?;

        Ok(output.status.success())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(false)
    }
}

/// Habilita o deshabilita el autoarranque en el registro de usuario (HKCU)
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if enabled {
            let current_exe = std::env::current_exe()
                .map_err(|e| format!("Failed to get current executable path: {}", e))?;
            let exe_str = current_exe.to_string_lossy().to_string();
            let value_data = format!("\"{}\" --minimized", exe_str);

            let output = Command::new("reg")
                .args([
                    "add",
                    RUN_KEY,
                    "/v",
                    APP_VALUE_NAME,
                    "/t",
                    "REG_SZ",
                    "/d",
                    &value_data,
                    "/f",
                ])
                .output()
                .map_err(|e| format!("Failed to write autostart registry key: {}", e))?;

            if !output.status.success() {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                return Err(format!("reg add failed: {}", err_msg));
            }
        } else {
            let output = Command::new("reg")
                .args(["delete", RUN_KEY, "/v", APP_VALUE_NAME, "/f"])
                .output()
                .map_err(|e| format!("Failed to delete autostart registry key: {}", e))?;

            // Si el código es 0 o 1 (la clave ya no existía), se considera éxito
            if !output.status.success() && output.status.code() != Some(1) {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                return Err(format!("reg delete failed: {}", err_msg));
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enabled;
        Ok(())
    }
}
