//! Cliente del helper elevado de firewall (ADR-002, ADR-005, T154, FR-057).
//!
//! La aplicación **nunca se eleva**. Para cambiar reglas lanza el helper con `runas`
//! mediante `ShellExecuteExW`, directamente: sin `powershell -Command`, sin cadenas
//! interpoladas dentro de otra cadena y sin ningún fichero intermedio.
//!
//! Antes, la petición viajaba en `%TEMP%\nb_fw_req_<pid>.json`, un nombre predecible en un
//! directorio que escribe cualquier proceso del usuario, y el proceso elevado la leía
//! después: quien sustituyera el fichero entre la escritura y la lectura elegía qué
//! reglas se aplicaban con privilegios. Ahora la petición son los argumentos con los que
//! nace el proceso elevado: no hay nada que sustituir.
//!
//! La respuesta es solo el código de salida (`validation::salida`). El JSON de resultado
//! que el helper escribía en otro fichero temporal no lo leía nadie.

use super::validation::{self, FirewallHelperRequest, salida};
use crate::errors::{AppError, ErrorCode};
use std::env;
use std::path::PathBuf;

pub struct FirewallHelperClient;

impl FirewallHelperClient {
    /// Localiza el helper elevado, o `None`.
    ///
    /// Hay dos disposiciones reales: **en desarrollo**, junto al ejecutable principal;
    /// **instalado**, como recurso de Tauri en `resources/`. Ya no hay respaldo por nombre
    /// simple: pasarle a `runas` un nombre sin ruta lo resolvería por el `PATH` o el
    /// directorio actual, es decir, elevaría lo primero que alguien dejara con ese nombre.
    pub fn get_helper_path() -> Option<PathBuf> {
        const NOMBRE: &str = "networkbench-firewall-helper.exe";
        let exe = env::current_exe().ok()?;
        let padre = exe.parent()?;
        [padre.join(NOMBRE), padre.join("resources").join(NOMBRE)]
            .into_iter()
            .find(|c| c.is_file())
    }

    /// Validación previa en el cliente. La que cuenta es la del helper; esta evita pedir a
    /// la persona una elevación que se sabe que será rechazada.
    pub fn validate_request(req: &FirewallHelperRequest) -> Result<(), String> {
        let helper = Self::get_helper_path().ok_or("No se encuentra el helper de firewall")?;
        let dir = helper.parent().ok_or("El helper no tiene carpeta")?;
        validation::validar_peticion(req, &validation::directorios_permitidos(dir))
    }

    /// Aplica una lista de reglas mediante el helper con elevación UAC.
    pub fn apply_rules(requests: &[FirewallHelperRequest]) -> Result<(), AppError> {
        let Some(helper) = Self::get_helper_path() else {
            return Err(AppError::from_code(ErrorCode::InternalError)
                .with_diagnostic_id("No se encuentra el helper de firewall".to_string()));
        };
        let dir = helper.parent().map(|d| d.to_path_buf()).unwrap_or_default();

        let parametros =
            validation::argumentos_de(requests, &validation::directorios_permitidos(&dir))
                .map_err(|e| {
                    AppError::from_code(ErrorCode::PermissionDenied).with_diagnostic_id(e)
                })?;

        match lanzar_elevado(&helper, &parametros)? {
            salida::OK => Ok(()),
            salida::NO_AUTORIZADA | salida::ARGUMENTOS => {
                Err(AppError::from_code(ErrorCode::PermissionDenied)
                    .with_diagnostic_id("El helper rechazó la petición".to_string()))
            }
            otro => Err(AppError::from_code(ErrorCode::InternalError)
                .with_diagnostic_id(format!("El helper terminó con el código {otro}"))),
        }
    }
}

/// Lanza `helper` con `runas` y espera su código de salida.
///
/// **No verificable aquí**: la elevación abre un diálogo de UAC en el escritorio del
/// usuario, y una prueba automática no puede ni debe aceptarlo. Lo que sí se prueba es todo
/// lo que rodea a esta llamada: la línea de parámetros, su reconstrucción en el helper con
/// el separador de Windows, y el rechazo de peticiones hostiles por el propio helper.
#[cfg(target_os = "windows")]
fn lanzar_elevado(helper: &std::path::Path, parametros: &str) -> Result<i32, AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};
    use windows::Win32::UI::Shell::{
        SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
    };
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
    use windows::core::{HRESULT, PCWSTR};

    /// Cuánto se espera a que la persona atienda el diálogo de UAC y el helper termine.
    const ESPERA_MS: u32 = 120_000;

    let ancho = |s: &std::ffi::OsStr| -> Vec<u16> { s.encode_wide().chain(Some(0)).collect() };
    let verbo = ancho("runas".as_ref());
    let fichero = ancho(helper.as_os_str());
    let params = ancho(parametros.as_ref());

    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: PCWSTR(verbo.as_ptr()),
        lpFile: PCWSTR(fichero.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };

    // SAFETY: `info` está inicializado, y los tres búferes anchos viven hasta el final de
    // la función y terminan en cero.
    unsafe {
        if let Err(e) = ShellExecuteExW(&mut info) {
            return Err(if e.code() == HRESULT::from_win32(ERROR_CANCELLED.0) {
                AppError::from_code(ErrorCode::FirewallUacRejected)
            } else {
                AppError::from_code(ErrorCode::InternalError).with_diagnostic_id(e.to_string())
            });
        }

        let proceso = info.hProcess;
        if proceso.is_invalid() {
            return Err(AppError::from_code(ErrorCode::InternalError)
                .with_diagnostic_id("La elevación no devolvió un proceso".to_string()));
        }
        let espera = WaitForSingleObject(proceso, ESPERA_MS);
        let mut codigo = 0u32;
        let leido = GetExitCodeProcess(proceso, &mut codigo);
        let _ = CloseHandle(proceso);

        if espera != WAIT_OBJECT_0 {
            return Err(AppError::from_code(ErrorCode::InternalError)
                .with_diagnostic_id("El helper no terminó a tiempo".to_string()));
        }
        leido.map_err(|e| {
            AppError::from_code(ErrorCode::InternalError).with_diagnostic_id(e.to_string())
        })?;
        Ok(codigo as i32)
    }
}

#[cfg(not(target_os = "windows"))]
fn lanzar_elevado(_helper: &std::path::Path, _parametros: &str) -> Result<i32, AppError> {
    Ok(salida::OK)
}
