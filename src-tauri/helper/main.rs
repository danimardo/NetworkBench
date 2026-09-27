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
        configurar().map_err(|e| format!("No se pudo añadir la regla: {e}"))
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
        exit(salida::NO_AUTORIZADA);
    }

    if solo_validar {
        exit(salida::OK);
    }

    for req in &peticiones {
        if let Err(e) = aplicar(req) {
            eprintln!("Error aplicando la regla '{}': {e}", req.rule_name);
            exit(salida::FALLO_DE_REGLA);
        }
    }
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
