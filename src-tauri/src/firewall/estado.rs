//! Estado real de las reglas de NetworkBench en el Firewall de Windows (`Historias.md` §14.3)
//! y perfil de las redes activas (§14.5).
//!
//! Se lee **sin elevación** con `Get-NetFirewallRule` y `Get-NetConnectionProfile`, que
//! devuelven valores (`True`, `Allow`, `TCP`, `7411`, `Public`…) iguales en cualquier idioma
//! de Windows. `netsh` no sirve para esto: su salida está traducida («Habilitada: Sí») y hay
//! que adivinar cada idioma.

use super::inspect::RuleStatus;
use super::reglas::{self, ReglaEsperada, ReglaLeida};
use serde::Serialize;
use std::collections::HashMap;

/// Una regla esperada junto con lo que hay de verdad en el sistema.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoRegla {
    #[serde(flatten)]
    pub regla: ReglaEsperada,
    pub estado: RuleStatus,
    pub detalle: String,
    /// El programa de la regla existe en disco. Sin él el helper rechaza la petición entera,
    /// así que una regla cuyo programa falta no se puede crear.
    pub programa_existe: bool,
    /// Comando de PowerShell equivalente, para copiarlo.
    pub comando_agregar: String,
}

/// Una red a la que está conectado el equipo y cómo la clasifica Windows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedActiva {
    pub nombre: String,
    pub interfaz: String,
    /// `Public`, `Private` o `DomainAuthenticated`. Una VPN suele salir como `Public`.
    pub categoria: String,
}

/// Todo lo que se lee del sistema de una vez (un solo PowerShell: cuesta unos segundos).
#[derive(Debug, Default)]
pub struct Lectura {
    pub reglas: HashMap<String, ReglaLeida>,
    pub redes: Vec<RedActiva>,
}

/// Compara cada regla esperada con la que hay en el sistema.
pub fn consultar(
    esperadas: &[ReglaEsperada],
) -> Result<(Vec<EstadoRegla>, Vec<RedActiva>), String> {
    #[cfg(target_os = "windows")]
    let lectura = leer_del_sistema()?;
    #[cfg(not(target_os = "windows"))]
    let lectura = Lectura {
        reglas: esperadas
            .iter()
            .map(|e| {
                (
                    e.nombre.clone(),
                    ReglaLeida {
                        habilitada: true,
                        accion: "Allow".into(),
                        direccion: "Inbound".into(),
                        perfil: e.perfiles.join(", "),
                        grupo: e.grupo.clone(),
                        protocolo: e.protocolo.clone(),
                        puertos_locales: e.puertos.clone(),
                        programa: e.programa.clone(),
                    },
                )
            })
            .collect(),
        redes: Vec::new(),
    };

    let estados = esperadas
        .iter()
        .map(|e| {
            let (estado, detalle) = reglas::evaluar(e, lectura.reglas.get(&e.nombre));
            EstadoRegla {
                estado,
                detalle,
                programa_existe: std::path::Path::new(&e.programa).is_file(),
                comando_agregar: reglas::comando_agregar(e),
                regla: e.clone(),
            }
        })
        .collect();
    Ok((estados, lectura.redes))
}

/// Nombres de las reglas de la aplicación que existen ahora mismo, sean o no las esperadas
/// (p. ej. la de descubrimiento tras apagar mDNS). Para «Eliminar todas».
pub fn existentes() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(leer_del_sistema()?.reglas.into_keys().collect())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

/// Interpreta la salida JSON del script: `{ Rules: [...], Networks: [...] }`, donde cada
/// lista puede llegar como objeto suelto, array o ausente.
pub fn interpretar(json: &str) -> Result<Lectura, String> {
    let json = json.trim().trim_start_matches('\u{feff}');
    if json.is_empty() {
        return Ok(Lectura::default());
    }
    let valor: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Salida de PowerShell no válida: {e}"))?;

    let como_lista = |v: &serde_json::Value| -> Vec<serde_json::Value> {
        match v {
            serde_json::Value::Array(a) => a.clone(),
            o @ serde_json::Value::Object(_) => vec![o.clone()],
            _ => Vec::new(),
        }
    };
    let texto = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or_default().to_string();

    let reglas = como_lista(&valor["Rules"])
        .iter()
        .map(|r| {
            (
                texto(r, "Name"),
                ReglaLeida {
                    habilitada: texto(r, "Enabled").eq_ignore_ascii_case("true"),
                    accion: texto(r, "Action"),
                    direccion: texto(r, "Direction"),
                    perfil: texto(r, "Profile"),
                    grupo: texto(r, "Group"),
                    protocolo: texto(r, "Protocol"),
                    puertos_locales: texto(r, "LocalPort"),
                    programa: texto(r, "Program"),
                },
            )
        })
        .collect();
    let redes = como_lista(&valor["Networks"])
        .iter()
        .map(|n| RedActiva {
            nombre: texto(n, "Name"),
            interfaz: texto(n, "Interface"),
            categoria: texto(n, "Category"),
        })
        .collect();
    Ok(Lectura { reglas, redes })
}

#[cfg(target_os = "windows")]
fn leer_del_sistema() -> Result<Lectura, String> {
    use std::os::windows::process::CommandExt;

    // Los nombres son constantes de este crate, no entrada de nadie: van dentro del script
    // sin más. Salida en UTF-8 para que una ruta con tildes no llegue rota.
    let nombres = reglas::NOMBRES_CONOCIDOS
        .iter()
        .map(|n| format!("'{n}'"))
        .collect::<Vec<_>>()
        .join(",");
    let script = format!(
        "[Console]::OutputEncoding=[Text.Encoding]::UTF8; $ErrorActionPreference='SilentlyContinue'; \
         $r=@(Get-NetFirewallRule -DisplayName @({nombres}) | ForEach-Object {{ \
           $p=$_|Get-NetFirewallPortFilter; $a=$_|Get-NetFirewallApplicationFilter; \
           [pscustomobject]@{{Name=$_.DisplayName;Enabled=[string]$_.Enabled;Action=[string]$_.Action;\
           Direction=[string]$_.Direction;Profile=[string]$_.Profile;Group=[string]$_.Group;\
           Protocol=[string]$p.Protocol;LocalPort=(@($p.LocalPort) -join ',');Program=[string]$a.Program}} }}); \
         $n=@(Get-NetConnectionProfile | ForEach-Object {{ \
           [pscustomobject]@{{Name=[string]$_.Name;Interface=[string]$_.InterfaceAlias;Category=[string]$_.NetworkCategory}} }}); \
         ConvertTo-Json -InputObject ([pscustomobject]@{{Rules=$r;Networks=$n}}) -Compress -Depth 4"
    );

    let raiz = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let powershell = std::path::PathBuf::from(raiz)
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");

    let salida = std::process::Command::new(powershell)
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        // CREATE_NO_WINDOW: sin parpadeo de consola.
        .creation_flags(0x0800_0000)
        .output()
        .map_err(|e| format!("No se pudo ejecutar PowerShell: {e}"))?;
    if !salida.status.success() {
        return Err(format!(
            "PowerShell terminó con error: {}",
            String::from_utf8_lossy(&salida.stderr).trim()
        ));
    }
    interpretar(&String::from_utf8_lossy(&salida.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGLA: &str = r#"{"Name":"NetworkBench - Control","Enabled":"True","Action":"Allow","Direction":"Inbound","Profile":"Domain, Private","Group":"NetworkBench","Protocol":"TCP","LocalPort":"7411","Program":"C:\\x\\NetworkBench.exe"}"#;
    const RED: &str = r#"{"Name":"Red","Interface":"Ethernet","Category":"Public"}"#;

    #[test]
    fn interpreta_reglas_y_redes_como_objeto_array_o_ausentes() {
        let l = interpretar(&format!(r#"{{"Rules":{REGLA},"Networks":{RED}}}"#)).unwrap();
        let r = &l.reglas["NetworkBench - Control"];
        assert!(r.habilitada);
        assert_eq!(r.puertos_locales, "7411");
        assert_eq!(r.grupo, "NetworkBench");
        assert_eq!(r.programa, r"C:\x\NetworkBench.exe");
        assert_eq!(l.redes.len(), 1);
        assert_eq!(l.redes[0].categoria, "Public");
        assert_eq!(l.redes[0].interfaz, "Ethernet");

        let dos = format!(
            r#"{{"Rules":[{REGLA},{}],"Networks":[{RED},{RED}]}}"#,
            REGLA.replace("Control", "NTTTCP TCP")
        );
        let l = interpretar(&dos).unwrap();
        assert_eq!((l.reglas.len(), l.redes.len()), (2, 2));

        let vacio = interpretar(r#"{"Rules":[],"Networks":[]}"#).unwrap();
        assert!(vacio.reglas.is_empty() && vacio.redes.is_empty());
        assert!(interpretar("").unwrap().reglas.is_empty());
        assert!(interpretar("\u{feff}{}").unwrap().reglas.is_empty());
    }

    #[test]
    fn una_salida_que_no_es_json_es_un_error_y_no_un_falso_ausente() {
        assert!(interpretar("Get-NetFirewallRule : algo salió mal").is_err());
    }

    #[test]
    fn habilitada_solo_con_true() {
        let l = interpretar(r#"{"Rules":{"Name":"x","Enabled":"False"}}"#).unwrap();
        assert!(!l.reglas["x"].habilitada);
    }

    #[test]
    fn consultar_devuelve_un_estado_por_cada_regla_esperada() {
        let e = reglas::Entorno {
            puerto_control: 7411,
            puerto_base_motor: 5001,
            mdns_activo: false,
            permitir_publico: false,
            ejecutable: std::path::PathBuf::from(r"C:\no\existe\NetworkBench.exe"),
            motor: std::path::PathBuf::from(r"C:\no\existe\ntttcp.exe"),
        };
        let esperadas = reglas::esperadas(&e);
        let (estados, _redes) = consultar(&esperadas).expect("consultar");
        assert_eq!(estados.len(), esperadas.len());
        assert!(estados.iter().all(|s| !s.programa_existe));
    }
}
