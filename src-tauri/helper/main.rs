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
use std::path::PathBuf;
use std::process::{Command, exit};

// La lista blanca es un solo fichero compilado en este binario y en la biblioteca, para
// que no puedan divergir. No todo lo que contiene lo usa el helper.
#[allow(dead_code)]
#[path = "../src/firewall/validation.rs"]
mod validation;

use validation::{FirewallHelperRequest, salida};

/// `netsh` por ruta absoluta: un proceso elevado no debe resolver su nombre por el
/// directorio actual ni por el `PATH`.
fn netsh() -> PathBuf {
    let raiz = env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    PathBuf::from(raiz).join("System32").join("netsh.exe")
}

fn ejecutar_netsh(args: &[String]) -> Result<(), String> {
    let salida = Command::new(netsh())
        .args(args)
        .output()
        .map_err(|e| format!("No se pudo ejecutar netsh: {e}"))?;
    if salida.status.success() {
        Ok(())
    } else {
        Err(format!(
            "netsh devolvió error: {}",
            String::from_utf8_lossy(&salida.stdout)
        ))
    }
}

fn aplicar(req: &FirewallHelperRequest) -> Result<(), String> {
    let borrar = [
        "advfirewall".to_string(),
        "firewall".to_string(),
        "delete".to_string(),
        "rule".to_string(),
        format!("name={}", req.rule_name),
    ];

    if req.operation == "remove" {
        return ejecutar_netsh(&borrar);
    }

    // Quitar antes la regla del mismo nombre evita duplicados; que no exista no es un fallo.
    let _ = ejecutar_netsh(&borrar);

    ejecutar_netsh(&[
        "advfirewall".to_string(),
        "firewall".to_string(),
        "add".to_string(),
        "rule".to_string(),
        format!("name={}", req.rule_name),
        "dir=in".to_string(),
        "action=allow".to_string(),
        format!("protocol={}", req.protocol.to_lowercase()),
        format!("localport={}", req.port_range),
        format!("program={}", req.program),
        format!("profile={}", req.profiles.join(",").to_lowercase()),
    ])
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
