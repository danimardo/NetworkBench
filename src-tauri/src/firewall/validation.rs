//! Petición al helper elevado de firewall y su validación (T154, FR-057, ADR-002/005).
//!
//! **Este fichero lo compilan dos crates a la vez**: la biblioteca (`firewall::validation`)
//! y el helper elevado (`helper/main.rs`, con `#[path]`). Es a propósito: la lista blanca
//! vive en un solo sitio, así que cliente y helper no pueden divergir. Por eso solo puede
//! depender de `std` y `serde`.
//!
//! El helper se ejecuta con privilegios y **lo puede lanzar cualquiera**, no solo la
//! aplicación: su validación es la frontera real, la del cliente es una comodidad. Por eso
//! aquí no se confía en nada que llegue: cada campo se comprueba contra una lista de
//! valores permitidos, no contra una lista de cosas prohibidas.
//!
//! Antes de esta tarea la lista blanca comprobaba el programa por el **sufijo** de su
//! nombre (`C:\malo\ntttcp.exe` pasaba), el rango de puertos solo si tenía exactamente un
//! guion (`1024-1030,2000-65000` se saltaba el límite), y no comprobaba protocolo ni
//! perfiles. La petición viajaba en un fichero de `%TEMP%` de nombre predecible.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallHelperRequest {
    pub operation: String, // "add" | "remove"
    pub rule_name: String,
    pub protocol: String,   // "TCP" | "UDP"
    pub port_range: String, // "5201" o "5001-5064"
    pub program: String,
    pub profiles: Vec<String>, // "Domain", "Private", "Public"
}

pub const PREFIJO_REGLA: &str = "NetworkBench - ";
pub const MAX_PUERTOS: u32 = 64;
pub const MAX_PETICIONES: usize = 8;
/// Límite de la línea de órdenes que se entrega al helper (el máximo de Windows es 32 767).
pub const MAX_LINEA: usize = 4096;
const PUERTO_MINIMO: u32 = 1024;
const PROGRAMAS: [&str; 2] = ["ntttcp.exe", "networkbench.exe"];
const PERFILES: [&str; 3] = ["Domain", "Private", "Public"];

/// Códigos de salida del helper. El cliente solo dispone de ellos como respuesta.
pub mod salida {
    pub const OK: i32 = 0;
    pub const ARGUMENTOS: i32 = 2;
    pub const NO_AUTORIZADA: i32 = 3;
    pub const FALLO_DE_REGLA: i32 = 4;
}

/// Directorios donde puede estar un programa autorizado, derivados de dónde está el
/// propio helper: en desarrollo, junto al ejecutable; instalado, en `resources\` con los
/// programas un nivel más arriba (y, según `Historias.md` §14.1, en `engine\`).
pub fn directorios_permitidos(directorio_del_helper: &Path) -> Vec<PathBuf> {
    let mut v = vec![directorio_del_helper.to_path_buf()];
    if let Some(padre) = directorio_del_helper.parent() {
        v.push(padre.to_path_buf());
        v.push(padre.join("engine"));
    }
    v.push(directorio_del_helper.join("engine"));
    v.into_iter()
        .filter_map(|d| d.canonicalize().ok())
        .collect()
}

fn caracteres_de_nombre(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
}

fn puerto(s: &str) -> Result<u32, String> {
    if s.is_empty() || s.len() > 5 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("Puerto no válido: '{s}'"));
    }
    let n: u32 = s.parse().map_err(|_| format!("Puerto no válido: '{s}'"))?;
    if !(PUERTO_MINIMO..=65535).contains(&n) {
        return Err(format!(
            "Puerto fuera de rango ({PUERTO_MINIMO}-65535): {n}"
        ));
    }
    Ok(n)
}

/// Un puerto o un rango `a-b` de como mucho 64 puertos. Nada más: ni listas, ni comas,
/// ni espacios, ni «cualquier puerto».
pub fn validar_rango(rango: &str) -> Result<(), String> {
    match rango.split_once('-') {
        None => puerto(rango).map(|_| ()),
        Some((a, b)) => {
            let (a, b) = (puerto(a)?, puerto(b)?);
            if b < a {
                return Err(format!("Rango invertido: {rango}"));
            }
            if b - a + 1 > MAX_PUERTOS {
                return Err(format!("Rango de más de {MAX_PUERTOS} puertos: {rango}"));
            }
            Ok(())
        }
    }
}

/// El programa debe ser **exactamente** uno de los dos de la instalación: nombre exacto y
/// directorio exacto (no «cualquier `ntttcp.exe`»), ya canonicalizado para que ni `..`
/// ni enlaces lo desvíen.
fn validar_programa(programa: &str, permitidos: &[PathBuf]) -> Result<(), String> {
    if programa.is_empty() {
        return Err(
            "Falta el programa: la regla debe acotarse a programa y puerto a la vez".into(),
        );
    }
    if programa.chars().any(|c| c.is_control() || c == '"') {
        return Err("Ruta de programa con caracteres no permitidos".into());
    }
    let ruta = Path::new(programa);
    if !ruta.is_absolute() {
        return Err("La ruta del programa debe ser absoluta".into());
    }
    let canonica = ruta
        .canonicalize()
        .map_err(|_| format!("El programa no existe: {programa}"))?;
    let nombre = canonica
        .file_name()
        .and_then(|n| n.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();
    if !PROGRAMAS.contains(&nombre.as_str()) {
        return Err(format!("Programa no autorizado: {programa}"));
    }
    let carpeta = canonica.parent().map(Path::to_path_buf).unwrap_or_default();
    if !permitidos.contains(&carpeta) {
        return Err(format!(
            "El programa no está en la carpeta de la instalación: {programa}"
        ));
    }
    Ok(())
}

pub fn validar_peticion(req: &FirewallHelperRequest, permitidos: &[PathBuf]) -> Result<(), String> {
    if req.operation != "add" && req.operation != "remove" {
        return Err(format!("Operación no permitida: {}", req.operation));
    }

    let nombre = &req.rule_name;
    if !nombre.starts_with(PREFIJO_REGLA) || nombre.len() <= PREFIJO_REGLA.len() {
        return Err(format!(
            "El nombre de la regla debe comenzar por '{PREFIJO_REGLA}'"
        ));
    }
    if nombre.len() > 80 || !caracteres_de_nombre(nombre) {
        return Err("Nombre de regla con caracteres no permitidos o demasiado largo".into());
    }

    // Borrar solo necesita el nombre; el resto no se usa y no se interpreta.
    if req.operation == "remove" {
        return Ok(());
    }

    if req.protocol != "TCP" && req.protocol != "UDP" {
        return Err(format!("Protocolo no permitido: {}", req.protocol));
    }
    validar_rango(&req.port_range)?;
    validar_programa(&req.program, permitidos)?;

    if req.profiles.is_empty() || req.profiles.len() > PERFILES.len() {
        return Err("Perfiles de red ausentes o repetidos".into());
    }
    for (i, p) in req.profiles.iter().enumerate() {
        if !PERFILES.contains(&p.as_str()) || req.profiles[..i].contains(p) {
            return Err(format!("Perfil no permitido o repetido: {p}"));
        }
    }
    Ok(())
}

pub fn validar_lote(reqs: &[FirewallHelperRequest], permitidos: &[PathBuf]) -> Result<(), String> {
    if reqs.is_empty() || reqs.len() > MAX_PETICIONES {
        return Err(format!("Debe haber entre 1 y {MAX_PETICIONES} operaciones"));
    }
    reqs.iter()
        .try_for_each(|r| validar_peticion(r, permitidos))
}

/// Cadena de parámetros para `ShellExecuteEx`: una entrada `--rule` por operación con seis
/// valores fijos, cada uno entre comillas. Sin JSON ni ficheros: los valores ya pasaron
/// por `validar_lote` y no pueden contener comillas, así que el entrecomillado no es
/// ambiguo. El programa ausente (solo en `remove`) viaja como `-`.
pub fn argumentos_de(
    reqs: &[FirewallHelperRequest],
    permitidos: &[PathBuf],
) -> Result<String, String> {
    validar_lote(reqs, permitidos)?;
    let mut linea = String::new();
    for r in reqs {
        let programa = if r.program.is_empty() {
            "-"
        } else {
            &r.program
        };
        let puertos = if r.port_range.is_empty() {
            "-"
        } else {
            &r.port_range
        };
        let protocolo = if r.protocol.is_empty() {
            "-"
        } else {
            &r.protocol
        };
        let perfiles = if r.profiles.is_empty() {
            "-".to_string()
        } else {
            r.profiles.join(",")
        };
        for campo in [&r.rule_name, programa, puertos, protocolo, &perfiles] {
            if campo.contains('"') {
                return Err("Un valor contiene comillas".into());
            }
        }
        if !linea.is_empty() {
            linea.push(' ');
        }
        linea.push_str(&format!(
            "--rule {} \"{}\" {} {} \"{}\" {}",
            r.operation, r.rule_name, protocolo, puertos, programa, perfiles
        ));
    }
    if linea.len() > MAX_LINEA {
        return Err("La petición es demasiado grande".into());
    }
    Ok(linea)
}

/// Inversa de `argumentos_de`, en el helper. Estricta: cualquier argumento que no sea el
/// esperado es un error, no se ignora.
pub fn peticiones_de_argumentos(args: &[String]) -> Result<Vec<FirewallHelperRequest>, String> {
    let mut v = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] != "--rule" || i + 7 > args.len() {
            return Err(format!("Argumento inesperado: '{}'", args[i]));
        }
        let quitar = |s: &str| {
            if s == "-" {
                String::new()
            } else {
                s.to_string()
            }
        };
        v.push(FirewallHelperRequest {
            operation: args[i + 1].clone(),
            rule_name: args[i + 2].clone(),
            protocol: quitar(&args[i + 3]),
            port_range: quitar(&args[i + 4]),
            program: quitar(&args[i + 5]),
            profiles: if args[i + 6] == "-" {
                vec![]
            } else {
                args[i + 6].split(',').map(str::to_string).collect()
            },
        });
        i += 7;
    }
    if v.is_empty() {
        return Err("No hay ninguna operación".into());
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Instalación de mentira: `<tmp>/resources/helper` y programas en `<tmp>/`.
    fn instalacion() -> (PathBuf, PathBuf, Vec<PathBuf>) {
        let raiz = std::env::temp_dir().join(format!("nb_fw_val_{}", uuid_like()));
        let resources = raiz.join("resources");
        fs::create_dir_all(&resources).unwrap();
        let ntttcp = raiz.join("ntttcp.exe");
        fs::write(&ntttcp, b"x").unwrap();
        fs::write(raiz.join("NetworkBench.exe"), b"x").unwrap();
        let permitidos = directorios_permitidos(&resources);
        (raiz, ntttcp, permitidos)
    }

    fn uuid_like() -> String {
        format!(
            "{}_{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }

    fn valida(programa: &Path) -> FirewallHelperRequest {
        FirewallHelperRequest {
            operation: "add".into(),
            rule_name: "NetworkBench - NTTTCP TCP".into(),
            protocol: "TCP".into(),
            port_range: "5001-5064".into(),
            program: programa.display().to_string(),
            profiles: vec!["Domain".into(), "Private".into()],
        }
    }

    #[test]
    fn una_peticion_de_la_instalacion_es_valida() {
        let (_r, ntttcp, permitidos) = instalacion();
        assert!(validar_peticion(&valida(&ntttcp), &permitidos).is_ok());
    }

    /// El fallo original: bastaba que el nombre terminase en `ntttcp.exe`.
    #[test]
    fn un_ntttcp_de_otra_carpeta_ya_no_pasa() {
        let (_r, _n, permitidos) = instalacion();
        let intruso_dir = std::env::temp_dir().join(format!("nb_fw_intruso_{}", uuid_like()));
        fs::create_dir_all(&intruso_dir).unwrap();
        let intruso = intruso_dir.join("ntttcp.exe");
        fs::write(&intruso, b"x").unwrap();
        let e = validar_peticion(&valida(&intruso), &permitidos).unwrap_err();
        assert!(e.contains("carpeta de la instalación"), "{e}");
    }

    #[test]
    fn un_programa_con_otro_nombre_no_pasa_aunque_este_en_la_carpeta() {
        let (raiz, _n, permitidos) = instalacion();
        let otro = raiz.join("cmd.exe");
        fs::write(&otro, b"x").unwrap();
        assert!(validar_peticion(&valida(&otro), &permitidos).is_err());
    }

    #[test]
    fn el_programa_no_puede_escaparse_con_puntos_puntos() {
        let (raiz, _n, permitidos) = instalacion();
        let hostil = raiz
            .join("resources")
            .join("..")
            .join("..")
            .join("nb_no_existe")
            .join("ntttcp.exe");
        assert!(validar_peticion(&valida(&hostil), &permitidos).is_err());
    }

    /// El otro fallo original: solo se miraba el rango si tenía exactamente un guion.
    #[test]
    fn las_listas_de_puertos_y_rangos_raros_se_rechazan() {
        for malo in [
            "1024-1030,2000-65000", // se saltaba el límite
            "1024-1030,2000",
            "5001,5002",
            "5001 - 5064",
            "0-65535",
            "80", // por debajo de 1024
            "5064-5001",
            "5000-6000", // más de 64
            "70000",
            "-5",
            "",
            "any",
            "5001-5002-5003",
        ] {
            assert!(validar_rango(malo).is_err(), "debía rechazar {malo:?}");
        }
        for bueno in ["5201", "5001-5064", "7411"] {
            assert!(validar_rango(bueno).is_ok(), "debía aceptar {bueno:?}");
        }
    }

    #[test]
    fn protocolo_perfiles_y_nombre_se_comprueban() {
        let (_r, ntttcp, permitidos) = instalacion();
        let base = valida(&ntttcp);

        for proto in ["tcp", "ANY", "ICMP", ""] {
            let mut r = base.clone();
            r.protocol = proto.into();
            assert!(validar_peticion(&r, &permitidos).is_err(), "{proto:?}");
        }
        for perfiles in [
            vec![],
            vec!["Cualquiera".to_string()],
            vec!["Public".into(), "Public".into()],
        ] {
            let mut r = base.clone();
            r.profiles = perfiles;
            assert!(validar_peticion(&r, &permitidos).is_err());
        }
        for nombre in [
            "Otra - regla",
            "NetworkBench - ",
            "NetworkBench - a\"b",
            "NetworkBench - a,b",
            "NetworkBench - a=b",
            "NetworkBench - a&b",
        ] {
            let mut r = base.clone();
            r.rule_name = nombre.into();
            assert!(validar_peticion(&r, &permitidos).is_err(), "{nombre:?}");
        }
        let mut sin_programa = base.clone();
        sin_programa.program = String::new();
        assert!(validar_peticion(&sin_programa, &permitidos).is_err());
    }

    #[test]
    fn borrar_solo_exige_un_nombre_de_la_aplicacion() {
        let (_r, _n, permitidos) = instalacion();
        let r = FirewallHelperRequest {
            operation: "remove".into(),
            rule_name: "NetworkBench - Control".into(),
            protocol: String::new(),
            port_range: String::new(),
            program: String::new(),
            profiles: vec![],
        };
        assert!(validar_peticion(&r, &permitidos).is_ok());
        let mut ajena = r.clone();
        ajena.rule_name = "Regla del sistema".into();
        assert!(validar_peticion(&ajena, &permitidos).is_err());
    }

    #[test]
    fn el_lote_tiene_limite() {
        let (_r, ntttcp, permitidos) = instalacion();
        assert!(validar_lote(&[], &permitidos).is_err());
        let muchas = vec![valida(&ntttcp); MAX_PETICIONES + 1];
        assert!(validar_lote(&muchas, &permitidos).is_err());
    }

    /// Lo que se entrega al helper se reconstruye exactamente al otro lado.
    #[test]
    fn los_argumentos_hacen_el_viaje_de_ida_y_vuelta() {
        let (_r, ntttcp, permitidos) = instalacion();
        let mut borrar = valida(&ntttcp);
        borrar.operation = "remove".into();
        borrar.protocol = String::new();
        borrar.port_range = String::new();
        borrar.program = String::new();
        borrar.profiles = vec![];
        let reqs = vec![valida(&ntttcp), borrar];

        let linea = argumentos_de(&reqs, &permitidos).unwrap();
        // Como los separaría Windows: comillas agrupan, espacios separan.
        let args = separar_como_windows(&linea);
        let vuelta = peticiones_de_argumentos(&args).unwrap();

        assert_eq!(vuelta.len(), 2);
        assert_eq!(vuelta[0].rule_name, "NetworkBench - NTTTCP TCP");
        assert_eq!(vuelta[0].program, reqs[0].program);
        assert_eq!(vuelta[0].profiles, vec!["Domain", "Private"]);
        assert_eq!(vuelta[1].operation, "remove");
        assert!(vuelta[1].program.is_empty() && vuelta[1].profiles.is_empty());
    }

    #[test]
    fn el_helper_no_ignora_argumentos_que_no_conoce() {
        for malos in [
            vec!["--file".to_string(), "x.json".to_string()],
            vec!["--rule".to_string(), "add".to_string()],
            vec![],
        ] {
            assert!(peticiones_de_argumentos(&malos).is_err(), "{malos:?}");
        }
    }

    /// Separación de una línea de órdenes suficiente para lo que genera `argumentos_de`.
    fn separar_como_windows(linea: &str) -> Vec<String> {
        let (mut v, mut actual, mut dentro, mut hay) = (vec![], String::new(), false, false);
        for c in linea.chars() {
            match c {
                '"' => {
                    dentro = !dentro;
                    hay = true;
                }
                ' ' if !dentro => {
                    if hay {
                        v.push(std::mem::take(&mut actual));
                        hay = false;
                    }
                }
                _ => {
                    actual.push(c);
                    hay = true;
                }
            }
        }
        if hay {
            v.push(actual);
        }
        v
    }
}
