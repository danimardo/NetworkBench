//! El helper elevado de firewall, ejecutado de verdad (T154, FR-057).
//!
//! Lo único que no se puede probar aquí es el diálogo de UAC de `ShellExecuteEx`: abriría
//! una ventana en el escritorio de quien ejecuta la prueba, y aceptarla no es cosa de un
//! test. Todo lo demás sí: el binario real recibe la línea de parámetros **exacta** que
//! construye el cliente, la analiza con el separador de Windows, la valida y responde con
//! su código de salida. `--validate-only` hace todo eso sin privilegios y sin tocar el
//! firewall, así que estas pruebas no pueden cambiar ninguna regla.

#![cfg(target_os = "windows")]

use networkbench_lib::firewall::FirewallHelperRequest;
use networkbench_lib::firewall::validation::{argumentos_de, directorios_permitidos, salida};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Una «instalación»: una copia del helper real con un `ntttcp.exe` junto a él, en una
/// carpeta con espacios (el caso que un entrecomillado mal hecho rompe).
struct Instalacion {
    carpeta: PathBuf,
    helper: PathBuf,
}

impl Instalacion {
    fn nueva() -> Self {
        let carpeta =
            std::env::temp_dir().join(format!("nb fw prueba {}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&carpeta).unwrap();
        let helper = carpeta.join("networkbench-firewall-helper.exe");
        std::fs::copy(env!("CARGO_BIN_EXE_networkbench-firewall-helper"), &helper).unwrap();
        std::fs::write(carpeta.join("ntttcp.exe"), b"no es un ejecutable").unwrap();
        Self { carpeta, helper }
    }

    fn permitidos(&self) -> Vec<PathBuf> {
        directorios_permitidos(&self.carpeta)
    }

    fn peticion(&self) -> FirewallHelperRequest {
        FirewallHelperRequest {
            operation: "add".into(),
            rule_name: "NetworkBench - NTTTCP TCP".into(),
            protocol: "TCP".into(),
            port_range: "5001-5064".into(),
            program: self.carpeta.join("ntttcp.exe").display().to_string(),
            profiles: vec!["Domain".into(), "Private".into()],
        }
    }

    /// Lanza el helper con `linea` tal cual, como lo haría `ShellExecuteEx`.
    fn ejecutar(&self, linea: &str) -> i32 {
        Command::new(&self.helper)
            .arg("--validate-only")
            .raw_arg(linea)
            .output()
            .expect("lanzar el helper")
            .status
            .code()
            .expect("el helper debe terminar con código")
    }
}

impl Drop for Instalacion {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.carpeta);
    }
}

#[test]
fn una_peticion_valida_llega_intacta_al_helper_real() {
    let i = Instalacion::nueva();
    let linea = argumentos_de(&[i.peticion()], &i.permitidos()).unwrap();
    assert_eq!(i.ejecutar(&linea), salida::OK, "línea: {linea}");
}

#[test]
fn borrar_una_regla_de_la_aplicacion_llega_al_helper_real() {
    let i = Instalacion::nueva();
    let mut r = i.peticion();
    r.operation = "remove".into();
    r.protocol.clear();
    r.port_range.clear();
    r.program.clear();
    r.profiles.clear();
    let linea = argumentos_de(&[r], &i.permitidos()).unwrap();
    assert_eq!(i.ejecutar(&linea), salida::OK, "línea: {linea}");
}

/// El helper no confía en el cliente: aunque alguien lo lance a mano con lo que quiera,
/// rechaza lo que no esté en la lista blanca. Se le pasa la línea directamente, sin pasar
/// por la validación del cliente.
#[test]
fn el_helper_rechaza_por_su_cuenta_lo_que_el_cliente_no_dejaria_pasar() {
    let i = Instalacion::nueva();
    let programa = i.carpeta.join("ntttcp.exe").display().to_string();
    // Un `ntttcp.exe` que existe de verdad, pero fuera de la carpeta del helper.
    let intruso_dir =
        std::env::temp_dir().join(format!("nb_fw_intruso_{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&intruso_dir).unwrap();
    std::fs::write(intruso_dir.join("ntttcp.exe"), b"x").unwrap();
    let intruso = intruso_dir.join("ntttcp.exe").display().to_string();
    let base = |ops: &str, nombre: &str, proto: &str, puertos: &str, prog: &str, perfiles: &str| {
        format!("--rule {ops} \"{nombre}\" {proto} {puertos} \"{prog}\" {perfiles}")
    };

    let casos = [
        // El fallo original: cualquier `ntttcp.exe` valía. Este está en otra carpeta.
        (
            "un ntttcp.exe de otra carpeta",
            base(
                "add",
                "NetworkBench - X",
                "TCP",
                "5001-5064",
                &intruso,
                "Domain",
            ),
        ),
        // El otro: una lista de rangos se saltaba el límite de 64 puertos.
        (
            "una lista de rangos",
            base(
                "add",
                "NetworkBench - X",
                "TCP",
                "1024-1030,2000-65000",
                &programa,
                "Domain",
            ),
        ),
        (
            "todos los puertos",
            base(
                "add",
                "NetworkBench - X",
                "TCP",
                "1024-65535",
                &programa,
                "Domain",
            ),
        ),
        (
            "un protocolo que no es TCP ni UDP",
            base(
                "add",
                "NetworkBench - X",
                "ANY",
                "5001-5064",
                &programa,
                "Domain",
            ),
        ),
        (
            "un perfil inventado",
            base(
                "add",
                "NetworkBench - X",
                "TCP",
                "5001-5064",
                &programa,
                "Todos",
            ),
        ),
        (
            "una regla que no es de la aplicación",
            base("remove", "Escritorio remoto", "-", "-", "-", "-"),
        ),
        (
            "una operación desconocida",
            base(
                "modify",
                "NetworkBench - X",
                "TCP",
                "5001-5064",
                &programa,
                "Domain",
            ),
        ),
        (
            "un programa que no es de la aplicación aunque esté en su carpeta",
            {
                std::fs::write(i.carpeta.join("cmd.exe"), b"x").unwrap();
                base(
                    "add",
                    "NetworkBench - X",
                    "TCP",
                    "5001-5064",
                    &i.carpeta.join("cmd.exe").display().to_string(),
                    "Domain",
                )
            },
        ),
    ];
    for (nombre, linea) in casos {
        assert_eq!(
            i.ejecutar(&linea),
            salida::NO_AUTORIZADA,
            "debía rechazar {nombre}: {linea}"
        );
    }
    let _ = std::fs::remove_dir_all(&intruso_dir);
}

#[test]
fn el_helper_no_ignora_argumentos_desconocidos() {
    let i = Instalacion::nueva();
    // El viejo protocolo pasaba un fichero; ya no existe y no se acepta.
    for linea in [
        "--file C:\\Users\\alguien\\peticion.json --out salida.json",
        "--rule add",
        "",
        "--rule add \"NetworkBench - X\" TCP 5001 \"x\" Domain --extra",
    ] {
        assert_eq!(i.ejecutar(linea), salida::ARGUMENTOS, "línea: {linea:?}");
    }
}

/// Nada de lo que rodea a la petición depende de ficheros del usuario: la línea es todo lo
/// que el proceso elevado recibe. Comprobado sobre la propia línea, que no menciona
/// ningún fichero temporal ni JSON.
#[test]
fn la_linea_de_parametros_no_lleva_ficheros_temporales_ni_json() {
    let i = Instalacion::nueva();
    let linea = argumentos_de(&[i.peticion()], &i.permitidos()).unwrap();
    assert!(!linea.contains(".json"));
    assert!(!linea.to_lowercase().contains("nb_fw_req"));
    assert!(!linea.contains("powershell"));
    assert!(Path::new(&i.helper).is_file());
}
