//! Helper elevado de firewall (ADR-002, ADR-005, T154, FR-057).
//!
//! Se ejecuta con privilegios y **puede lanzarlo cualquiera**, no solo la aplicación: por
//! eso no se fía de quién le llama ni de lo que le llega. Recibe la petición como
//! argumentos de línea de órdenes (`--rule ...`, ver `validation::argumentos_de`), la
//! valida entera contra la lista blanca compartida con el cliente y solo entonces toca el
//! firewall. Devuelve únicamente un código de salida (`validation::salida`).
//!
//! `--validate-only` como primer argumento hace todo menos aplicar: sirve para probar el
//! camino completo —línea de órdenes, análisis, lista blanca— sin privilegios y sin tocar
//! el firewall.

use std::env;
use std::process::exit;

// ---------------------------------------------------------------------------------------
// Registro de diagnóstico (hallazgo real del propietario, 2026-09-27): sin esto, un fallo
// de COM al crear una regla quedaba completamente invisible — `eprintln!` escribe a un
// stderr que nadie captura, porque `ShellExecuteExW` con "runas" no redirige la salida del
// proceso elevado hacia el padre. El resultado era una contradicción silenciosa: el código
// de salida decía éxito (o un error genérico sin detalle) y la regla nunca aparecía.
//
// Va a %ProgramData%, no al %LOCALAPPDATA% de la aplicación: este proceso corre elevado, a
// veces con las credenciales de OTRA cuenta (el UAC de "ejecutar como" pide usuario y
// contraseña cuando la cuenta actual no es administradora), así que %LOCALAPPDATA% podría
// apuntar al perfil de una cuenta distinta de la que abrió la aplicación. %ProgramData% es
// la única carpeta que ambas cuentas comparten sin ambigüedad.
// ---------------------------------------------------------------------------------------

fn carpeta_log() -> Option<std::path::PathBuf> {
    let base = env::var_os("ProgramData")?;
    Some(
        std::path::PathBuf::from(base)
            .join("NetworkBench")
            .join("logs"),
    )
}

/// Un fichero de texto sin rotación: el volumen es bajísimo (solo se escribe al crear o
/// eliminar reglas, una acción manual y poco frecuente). Si no se puede escribir —permisos,
/// disco lleno—, no aborta nada: es diagnóstico, no una dependencia del funcionamiento.
fn registrar(linea: &str) {
    use std::io::Write as _;
    let Some(dir) = carpeta_log() else { return };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let ahora = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("firewall-helper.log"))
    {
        // Segundos desde 1970 (UTC), no una fecha legible: este binario evita a propósito
        // cualquier dependencia externa (superficie de ataque mínima al correr elevado), y
        // formatear una fecha civil a mano es un sitio innecesario para introducir un bug
        // de calendario en código que se ejecuta con privilegios.
        let _ = writeln!(f, "[{ahora}] {linea}");
    }
}

// La lista blanca es un solo fichero compilado en este binario y en la biblioteca, para
// que no puedan divergir. No todo lo que contiene lo usa el helper.
#[allow(dead_code)]
#[path = "../src/firewall/validation.rs"]
mod validation;

use validation::{FirewallHelperRequest, GRUPO_REGLAS, salida};

/// Protocolo IANA de la regla: TCP = 6, UDP = 17.
fn numero_de_protocolo(protocolo: &str) -> Option<i32> {
    match protocolo {
        "TCP" => Some(6),
        "UDP" => Some(17),
        _ => None,
    }
}

/// La API del firewall solo acepta rutas con `\` y sin el prefijo `\\?\` que deja
/// `canonicalize`: con `/` responde «el parámetro no es correcto».
fn ruta_para_firewall(ruta: &str) -> String {
    ruta.trim_start_matches(r"\\?\").replace('/', r"\")
}

/// Máscara de `NET_FW_PROFILE_TYPE2`: Dominio = 1, Privado = 2, Público = 4.
fn mascara_de_perfiles(perfiles: &[String]) -> Option<i32> {
    perfiles.iter().try_fold(0, |mascara, p| {
        let bit = match p.as_str() {
            "Domain" => 1,
            "Private" => 2,
            "Public" => 4,
            _ => return None,
        };
        Some(mascara | bit)
    })
}

/// Las reglas se gestionan con la API COM del Firewall de Windows (`INetFwPolicy2`, §14.2),
/// no con `netsh`: `netsh advfirewall firewall add rule` no puede asignar el grupo, y sin
/// grupo el desinstalador (que borra `group="NetworkBench"`) no encuentra nada y las
/// reglas quedan huérfanas.
#[cfg(target_os = "windows")]
mod firewall {
    use super::{
        FirewallHelperRequest, GRUPO_REGLAS, mascara_de_perfiles, numero_de_protocolo,
        ruta_para_firewall,
    };
    use windows::Win32::Foundation::VARIANT_TRUE;
    use windows::Win32::NetworkManagement::WindowsFirewall::{
        INetFwPolicy2, INetFwRule, INetFwRules, NET_FW_ACTION_ALLOW, NET_FW_RULE_DIR_IN,
        NetFwPolicy2, NetFwRule,
    };
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        CoUninitialize,
    };
    use windows::core::BSTR;

    /// Un duplicado del mismo nombre no debería existir, pero `Remove` quita uno cada vez.
    const MAX_DUPLICADOS: usize = 8;

    /// COM inicializado en este hilo mientras viva.
    pub struct Com;

    impl Com {
        pub fn iniciar() -> Result<Self, String> {
            // SAFETY: primera llamada de COM en este hilo; se cierra en `Drop`.
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
                .ok()
                .map_err(|e| format!("No se pudo inicializar COM: {e}"))?;
            Ok(Self)
        }
    }

    impl Drop for Com {
        fn drop(&mut self) {
            // SAFETY: empareja el `CoInitializeEx` de `iniciar`.
            unsafe { CoUninitialize() };
        }
    }

    fn reglas() -> Result<INetFwRules, String> {
        // SAFETY: COM está inicializado (`Com`) y `NetFwPolicy2` es un servidor en proceso.
        let politica: INetFwPolicy2 =
            unsafe { CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER) }
                .map_err(|e| format!("No se pudo abrir la política del firewall: {e}"))?;
        // SAFETY: `politica` es válida.
        unsafe { politica.Rules() }.map_err(|e| format!("No se pudo leer las reglas: {e}"))
    }

    /// Quita todas las reglas con ese nombre. Que no exista no es un fallo.
    pub fn quitar(nombre: &str) -> Result<(), String> {
        let reglas = reglas()?;
        let nombre = BSTR::from(nombre);
        for _ in 0..MAX_DUPLICADOS {
            // SAFETY: `reglas` y `nombre` son válidos; `Item` falla si no hay ninguna.
            if unsafe { reglas.Item(&nombre) }.is_err() {
                return Ok(());
            }
            // SAFETY: ídem.
            unsafe { reglas.Remove(&nombre) }.map_err(|e| format!("No se pudo quitar: {e}"))?;
        }
        Ok(())
    }

    /// Crea la regla, sustituyendo otra del mismo nombre (evita duplicados).
    pub fn agregar(req: &FirewallHelperRequest) -> Result<(), String> {
        let protocolo = numero_de_protocolo(&req.protocol).ok_or("Protocolo no válido")?;
        let perfiles = mascara_de_perfiles(&req.profiles).ok_or("Perfil no válido")?;
        quitar(&req.rule_name)?;

        let reglas = reglas()?;
        // SAFETY: COM está inicializado y `NetFwRule` es un servidor en proceso.
        let regla: INetFwRule = unsafe { CoCreateInstance(&NetFwRule, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| format!("No se pudo crear la regla: {e}"))?;

        let descripcion = format!("NetworkBench {}", env!("CARGO_PKG_VERSION"));
        let configurar = || -> windows::core::Result<()> {
            // SAFETY: `regla` es válida y cada valor es un BSTR o un entero propios.
            unsafe {
                regla.SetName(&BSTR::from(req.rule_name.as_str()))?;
                regla.SetDescription(&BSTR::from(descripcion.as_str()))?;
                regla.SetGrouping(&BSTR::from(GRUPO_REGLAS))?;
                regla.SetDirection(NET_FW_RULE_DIR_IN)?;
                regla.SetAction(NET_FW_ACTION_ALLOW)?;
                regla.SetProtocol(protocolo)?;
                regla.SetLocalPorts(&BSTR::from(req.port_range.as_str()))?;
                regla.SetApplicationName(&BSTR::from(ruta_para_firewall(&req.program)))?;
                regla.SetProfiles(perfiles)?;
                regla.SetEnabled(VARIANT_TRUE)?;
                reglas.Add(&regla)
            }
        };
        configurar().map_err(|e| format!("No se pudo añadir la regla: {e}"))?;

        // Releer inmediatamente lo que se acaba de escribir. `Add` puede devolver éxito y
        // que la regla no persista de verdad —por ejemplo, si el firewall de esta máquina
        // está bajo Directiva de grupo con "Aplicar reglas locales" desactivado, la API
        // acepta la escritura sin avisar de que no tendrá efecto—. Sin esto, ese caso se
        // veía exactamente como el bug real que reportó el propietario: UAC concedido,
        // "éxito" declarado, regla ausente al comprobar después.
        let releida = unsafe { reglas.Item(&BSTR::from(req.rule_name.as_str())) }.map_err(|e| {
            format!(
                "La regla se añadió sin error pero no se puede releer justo después \
                 (posible directiva de grupo bloqueando reglas locales del firewall): {e}"
            )
        })?;

        // Comprobación aparte, y que NO hace fallar la creación: el propietario encontró en
        // una máquina real (2026-09-27) que `SetGrouping` no siempre deja el grupo
        // consultable por `Get-NetFirewallRule -Group`/`Remove-NetFirewallRule -Group` ni
        // visible en el panel de Windows, aunque la regla en sí funciona (el tráfico pasa).
        // Solo afecta a la limpieza del desinstalador, no a que la regla proteja o no la
        // conexión: por eso se registra como aviso, no como fallo de la operación.
        match unsafe { releida.Grouping() } {
            Ok(g) if g == GRUPO_REGLAS => {}
            Ok(g) => super::registrar(&format!(
                "AVISO: '{}' se creó pero su grupo es '{g}', no '{GRUPO_REGLAS}' \
                 (el desinstalador podría no encontrarla)",
                req.rule_name
            )),
            Err(e) => super::registrar(&format!(
                "AVISO: no se pudo releer el grupo de '{}' tras crearla: {e}",
                req.rule_name
            )),
        }
        Ok(())
    }
}

fn aplicar(req: &FirewallHelperRequest) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let _com = firewall::Com::iniciar()?;
        if req.operation == "remove" {
            firewall::quitar(&req.rule_name)
        } else {
            firewall::agregar(req)
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = req;
        Ok(())
    }
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let solo_validar = args.first().map(String::as_str) == Some("--validate-only");
    if solo_validar {
        args.remove(0);
    }

    let peticiones = match validation::peticiones_de_argumentos(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Argumentos no válidos: {e}");
            registrar(&format!("RECHAZADO (argumentos no válidos): {e}"));
            exit(salida::ARGUMENTOS);
        }
    };

    let directorio = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();
    if let Err(e) = validation::validar_lote(
        &peticiones,
        &validation::directorios_permitidos(&directorio),
    ) {
        eprintln!("Petición no autorizada: {e}");
        registrar(&format!("RECHAZADO (lista blanca): {e}"));
        exit(salida::NO_AUTORIZADA);
    }

    if solo_validar {
        exit(salida::OK);
    }

    let resumen = peticiones
        .iter()
        .map(|p| format!("{} {}", p.operation, p.rule_name))
        .collect::<Vec<_>>()
        .join(", ");
    registrar(&format!("INICIO: {resumen}"));

    for req in &peticiones {
        match aplicar(req) {
            Ok(()) => registrar(&format!("OK: {} {}", req.operation, req.rule_name)),
            Err(e) => {
                eprintln!("Error aplicando la regla '{}': {e}", req.rule_name);
                registrar(&format!("FALLO: {} {} → {e}", req.operation, req.rule_name));
                exit(salida::FALLO_DE_REGLA);
            }
        }
    }
    registrar("FIN: todas las reglas del lote aplicadas correctamente");
    exit(salida::OK);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_protocolos_y_perfiles_se_traducen_a_los_valores_de_la_api() {
        assert_eq!(numero_de_protocolo("TCP"), Some(6));
        assert_eq!(numero_de_protocolo("UDP"), Some(17));
        assert_eq!(numero_de_protocolo("ICMP"), None);

        let v = |p: &[&str]| p.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert_eq!(mascara_de_perfiles(&v(&["Domain", "Private"])), Some(3));
        assert_eq!(
            mascara_de_perfiles(&v(&["Domain", "Private", "Public"])),
            Some(7)
        );
        assert_eq!(mascara_de_perfiles(&v(&["Public"])), Some(4));
        assert_eq!(mascara_de_perfiles(&v(&["Todos"])), None);
    }

    #[test]
    fn la_ruta_va_con_barras_de_windows_y_sin_prefijo_de_canonicalize() {
        assert_eq!(
            ruta_para_firewall(r"\\?\C:\Apps\NetworkBench\NetworkBench.exe"),
            r"C:\Apps\NetworkBench\NetworkBench.exe"
        );
        assert_eq!(ruta_para_firewall("C:/Apps/x.exe"), r"C:\Apps\x.exe");
    }

    #[test]
    fn el_grupo_es_el_que_el_desinstalador_borra() {
        assert_eq!(GRUPO_REGLAS, "NetworkBench");
    }
}
