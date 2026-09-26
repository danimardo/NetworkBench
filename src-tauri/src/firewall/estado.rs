//! Estado real de las reglas de NetworkBench en el Firewall de Windows (`Historias.md` §14.3).
//!
//! Se lee **sin elevación** con `Get-NetFirewallRule` y sus filtros, que devuelven valores
//! (`True`, `Allow`, `TCP`, `7411`…) iguales en cualquier idioma de Windows. `netsh` no sirve
//! para esto: su salida está traducida («Habilitada: Sí») y hay que adivinar cada idioma.

use super::inspect::RuleStatus;
use super::reglas::{self, ReglaEsperada, ReglaLeida};
use serde::Serialize;
#[cfg(target_os = "windows")]
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
    /// `netsh` equivalente, para copiarlo.
    pub netsh_agregar: String,
}

/// Compara cada regla esperada con la que hay en el sistema.
pub fn consultar(esperadas: &[ReglaEsperada]) -> Result<Vec<EstadoRegla>, String> {
    #[cfg(target_os = "windows")]
    let leidas = leer_del_sistema()?;
    #[cfg(not(target_os = "windows"))]
    let leidas: std::collections::HashMap<String, ReglaLeida> = esperadas
        .iter()
        .map(|e| {
            (
                e.nombre.clone(),
                ReglaLeida {
                    habilitada: true,
                    accion: "Allow".into(),
                    direccion: "Inbound".into(),
                    perfil: "Any".into(),
                    protocolo: e.protocolo.clone(),
                    puertos_locales: e.puertos.clone(),
                    programa: e.programa.clone(),
                },
            )
        })
        .collect();

    Ok(esperadas
        .iter()
        .map(|e| {
            let (estado, detalle) = reglas::evaluar(e, leidas.get(&e.nombre));
            EstadoRegla {
                estado,
                detalle,
                programa_existe: std::path::Path::new(&e.programa).is_file(),
                netsh_agregar: reglas::netsh_agregar(e),
                regla: e.clone(),
            }
        })
        .collect())
}

/// Nombres de las reglas de la aplicación que existen ahora mismo, sean o no las esperadas
/// (p. ej. la de descubrimiento tras apagar mDNS). Para «Eliminar todas».
pub fn existentes() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(leer_del_sistema()?.into_keys().collect())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

/// Interpreta la salida JSON del script: un objeto, un array o nada.
pub fn interpretar(json: &str) -> Result<std::collections::HashMap<String, ReglaLeida>, String> {
    let json = json.trim().trim_start_matches('\u{feff}');
    if json.is_empty() {
        return Ok(Default::default());
    }
    let valor: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Salida de PowerShell no válida: {e}"))?;
    let lista = match valor {
        serde_json::Value::Array(v) => v,
        otro @ serde_json::Value::Object(_) => vec![otro],
        _ => return Ok(Default::default()),
    };

    let texto = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or_default().to_string();
    Ok(lista
        .iter()
        .map(|r| {
            (
                texto(r, "Name"),
                ReglaLeida {
                    habilitada: texto(r, "Enabled").eq_ignore_ascii_case("true"),
                    accion: texto(r, "Action"),
                    direccion: texto(r, "Direction"),
                    perfil: texto(r, "Profile"),
                    protocolo: texto(r, "Protocol"),
                    puertos_locales: texto(r, "LocalPort"),
                    programa: texto(r, "Program"),
                },
            )
        })
        .collect())
}

#[cfg(target_os = "windows")]
fn leer_del_sistema() -> Result<HashMap<String, ReglaLeida>, String> {
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
           Direction=[string]$_.Direction;Profile=[string]$_.Profile;Protocol=[string]$p.Protocol;\
           LocalPort=(@($p.LocalPort) -join ',');Program=[string]$a.Program}} }}); \
         ConvertTo-Json -InputObject $r -Compress"
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

    #[test]
    fn interpreta_un_objeto_un_array_y_la_salida_vacia() {
        let una = r#"{"Name":"NetworkBench - Control","Enabled":"True","Action":"Allow","Direction":"Inbound","Profile":"Domain, Private","Protocol":"TCP","LocalPort":"7411","Program":"C:\\x\\NetworkBench.exe"}"#;
        let m = interpretar(una).unwrap();
        let r = &m["NetworkBench - Control"];
        assert!(r.habilitada);
        assert_eq!(r.puertos_locales, "7411");
        assert_eq!(r.programa, r"C:\x\NetworkBench.exe");

        let dos = format!("[{una},{}]", una.replace("Control", "NTTTCP TCP"));
        assert_eq!(interpretar(&dos).unwrap().len(), 2);

        assert!(interpretar("").unwrap().is_empty());
        assert!(interpretar("[]").unwrap().is_empty());
        assert!(interpretar("\u{feff}[]").unwrap().is_empty());
    }

    #[test]
    fn una_salida_que_no_es_json_es_un_error_y_no_un_falso_ausente() {
        assert!(interpretar("Get-NetFirewallRule : algo salió mal").is_err());
    }

    #[test]
    fn habilitada_solo_con_true() {
        let m = interpretar(r#"{"Name":"x","Enabled":"False"}"#).unwrap();
        assert!(!m["x"].habilitada);
    }

    #[test]
    fn consultar_devuelve_un_estado_por_cada_regla_esperada() {
        let e = reglas::Entorno {
            puerto_control: 7411,
            puerto_base_motor: 5001,
            mdns_activo: false,
            ejecutable: std::path::PathBuf::from(r"C:\no\existe\NetworkBench.exe"),
            motor: std::path::PathBuf::from(r"C:\no\existe\ntttcp.exe"),
        };
        let esperadas = reglas::esperadas(&e);
        let estados = consultar(&esperadas).expect("consultar");
        assert_eq!(estados.len(), esperadas.len());
        assert!(estados.iter().all(|s| !s.programa_existe));
    }
}
