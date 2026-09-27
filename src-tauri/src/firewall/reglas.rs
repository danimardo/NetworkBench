//! Las reglas de firewall que NetworkBench necesita (`Historias.md` §14.1) y su evaluación.
//!
//! Aquí solo hay lógica pura y testeable: qué reglas se esperan para un puerto de control y
//! un programa dados, y cómo se compara una regla leída del sistema con la esperada. Leer el
//! firewall (`estado`) y cambiarlo (`helper_client`, con UAC) están en sus módulos.

use super::inspect::RuleStatus;
use super::validation::{FirewallHelperRequest, GRUPO_REGLAS};
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const NOMBRE_CONTROL: &str = "NetworkBench - Control";
pub const NOMBRE_MOTOR_TCP: &str = "NetworkBench - NTTTCP TCP";
pub const NOMBRE_MOTOR_UDP: &str = "NetworkBench - NTTTCP UDP";
pub const NOMBRE_DESCUBRIMIENTO: &str = "NetworkBench - Descubrimiento";

/// Primer puerto del bloque de datos de NTTTCP. Todavía no es un ajuste: la interfaz de
/// opciones avanzadas usa el mismo valor por defecto.
pub const PUERTO_BASE_MOTOR: u16 = 5001;
/// Puertos que reserva cada protocolo del motor: `base..base+63` (§14.1).
pub const PUERTOS_MOTOR: u16 = 64;
pub const PUERTO_MDNS: u16 = 5353;
/// Perfiles por defecto (§14.5): Dominio y Privado.
pub const PERFILES_POR_DEFECTO: [&str; 2] = ["Domain", "Private"];
pub const PERFIL_PUBLICO: &str = "Public";

/// Una regla tal y como debe existir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReglaEsperada {
    pub nombre: String,
    pub protocolo: String,
    /// Un puerto (`7411`) o un rango (`5001-5064`).
    pub puertos: String,
    pub programa: String,
    pub perfiles: Vec<String>,
    /// Grupo con el que se crea; el desinstalador retira las reglas por él.
    pub grupo: String,
}

/// De qué depende el conjunto de reglas: los puertos configurados y dónde está cada programa.
#[derive(Debug, Clone)]
pub struct Entorno {
    pub puerto_control: u16,
    pub puerto_base_motor: u16,
    pub mdns_activo: bool,
    /// La persona ha aceptado permitir también las redes que Windows llama públicas (§14.5).
    pub permitir_publico: bool,
    pub ejecutable: PathBuf,
    pub motor: PathBuf,
}

impl Entorno {
    /// Junto al ejecutable están el motor y el helper, tanto en desarrollo como instalado
    /// (mismo criterio que `app::init` para localizar `ntttcp.exe`).
    pub fn actual(puerto_control: u16, mdns_activo: bool, permitir_publico: bool) -> Option<Self> {
        let ejecutable = std::env::current_exe().ok()?;
        let motor = ejecutable.parent()?.join("ntttcp.exe");
        Some(Self {
            puerto_control,
            puerto_base_motor: PUERTO_BASE_MOTOR,
            mdns_activo,
            permitir_publico,
            ejecutable,
            motor,
        })
    }

    fn perfiles(&self) -> Vec<String> {
        let mut v: Vec<String> = PERFILES_POR_DEFECTO.iter().map(|p| p.to_string()).collect();
        if self.permitir_publico {
            v.push(PERFIL_PUBLICO.to_string());
        }
        v
    }
}

/// Las reglas que deben existir. La de descubrimiento solo cuenta con mDNS encendido.
pub fn esperadas(e: &Entorno) -> Vec<ReglaEsperada> {
    let bloque = format!(
        "{}-{}",
        e.puerto_base_motor,
        e.puerto_base_motor + PUERTOS_MOTOR - 1
    );
    let exe = e.ejecutable.display().to_string();
    let motor = e.motor.display().to_string();
    let regla = |nombre: &str, protocolo: &str, puertos: String, programa: &str| ReglaEsperada {
        nombre: nombre.to_string(),
        protocolo: protocolo.to_string(),
        puertos,
        programa: programa.to_string(),
        perfiles: e.perfiles(),
        grupo: GRUPO_REGLAS.to_string(),
    };

    let mut v = vec![
        regla(NOMBRE_CONTROL, "TCP", e.puerto_control.to_string(), &exe),
        regla(NOMBRE_MOTOR_TCP, "TCP", bloque.clone(), &motor),
        regla(NOMBRE_MOTOR_UDP, "UDP", bloque, &motor),
    ];
    if e.mdns_activo {
        v.push(regla(
            NOMBRE_DESCUBRIMIENTO,
            "UDP",
            PUERTO_MDNS.to_string(),
            &exe,
        ));
    }
    v
}

/// Todos los nombres que la aplicación puede haber creado, con o sin mDNS: al eliminar se
/// consideran los cuatro, aunque el ajuste haya cambiado desde que se crearon.
pub const NOMBRES_CONOCIDOS: [&str; 4] = [
    NOMBRE_CONTROL,
    NOMBRE_MOTOR_TCP,
    NOMBRE_MOTOR_UDP,
    NOMBRE_DESCUBRIMIENTO,
];

/// La petición que se entrega al helper elevado para esta regla.
pub fn a_peticion(r: &ReglaEsperada, operacion: &str) -> FirewallHelperRequest {
    FirewallHelperRequest {
        operation: operacion.to_string(),
        rule_name: r.nombre.clone(),
        protocol: r.protocolo.clone(),
        port_range: r.puertos.clone(),
        program: r.programa.clone(),
        profiles: r.perfiles.clone(),
    }
}

/// Comando de PowerShell equivalente, para quien prefiera (o necesite, con una directiva
/// corporativa) crear la regla a mano. Solo texto para copiar: la aplicación no lo ejecuta.
///
/// Es PowerShell y no `netsh` porque `netsh advfirewall firewall add rule` no puede asignar
/// el grupo, y sin grupo el desinstalador no encuentra la regla.
pub fn comando_agregar(r: &ReglaEsperada) -> String {
    // En una cadena entre comillas simples de PowerShell solo hay que doblar la comilla simple.
    let entre_comillas = |s: &str| format!("'{}'", s.replace('\'', "''"));
    format!(
        "New-NetFirewallRule -DisplayName {} -Group {} -Direction Inbound -Action Allow -Protocol {} -LocalPort {} -Program {} -Profile {}",
        entre_comillas(&r.nombre),
        entre_comillas(&r.grupo),
        r.protocolo,
        r.puertos,
        entre_comillas(&r.programa),
        r.perfiles.join(",")
    )
}

pub fn comando_eliminar(nombre: &str) -> String {
    format!(
        "Remove-NetFirewallRule -DisplayName '{}'",
        nombre.replace('\'', "''")
    )
}

/// Una regla tal y como la devuelve el sistema (`Get-NetFirewallRule` y sus filtros).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReglaLeida {
    pub habilitada: bool,
    pub accion: String,
    pub direccion: String,
    pub perfil: String,
    pub grupo: String,
    pub protocolo: String,
    pub puertos_locales: String,
    pub programa: String,
}

/// Compara la regla leída con la esperada.
///
/// «Modificada» es cualquier campo que ya no casa: acción, sentido, protocolo, puertos,
/// programa, grupo o perfiles. Los perfiles se comparan **exactos**: que la regla valga
/// además para la red pública es tan distinto de lo esperado como que le falte uno, porque
/// abrir la red pública es una decisión explícita de la persona (§14.5).
pub fn evaluar(esperada: &ReglaEsperada, leida: Option<&ReglaLeida>) -> (RuleStatus, String) {
    let Some(l) = leida else {
        return (RuleStatus::Missing, "Regla ausente en el firewall".into());
    };
    if !l.habilitada {
        return (
            RuleStatus::Disabled,
            "Regla encontrada pero deshabilitada".into(),
        );
    }

    let mut diferencias = Vec::new();
    if !l.accion.eq_ignore_ascii_case("Allow") {
        diferencias.push(format!("acción «{}»", l.accion));
    }
    if !l.direccion.eq_ignore_ascii_case("Inbound") {
        diferencias.push(format!("sentido «{}»", l.direccion));
    }
    if !l.protocolo.eq_ignore_ascii_case(&esperada.protocolo) {
        diferencias.push(format!("protocolo {}", l.protocolo));
    }
    if l.puertos_locales.replace(' ', "") != esperada.puertos {
        diferencias.push(format!("puertos {}", l.puertos_locales));
    }
    if !misma_ruta(&l.programa, &esperada.programa) {
        diferencias.push(format!("programa {}", l.programa));
    }
    if !l.grupo.eq_ignore_ascii_case(&esperada.grupo) {
        // Las creadas antes de que el helper asignara grupo: el desinstalador no las vería.
        diferencias.push(format!(
            "sin el grupo {}",
            if l.grupo.is_empty() {
                &esperada.grupo
            } else {
                &l.grupo
            }
        ));
    }
    if perfiles_de(&l.perfil) != perfiles_esperados(&esperada.perfiles) {
        diferencias.push(format!("perfiles {}", l.perfil));
    }

    if diferencias.is_empty() {
        (RuleStatus::Present, "Regla presente y activa".into())
    } else {
        (
            RuleStatus::Modified,
            format!("Distinta de la esperada: {}", diferencias.join(", ")),
        )
    }
}

/// Rutas de Windows: sin distinguir mayúsculas y sin el prefijo `\\?\` que deja canonicalize.
fn misma_ruta(a: &str, b: &str) -> bool {
    let limpia = |s: &str| {
        s.trim()
            .trim_start_matches(r"\\?\")
            .replace('/', "\\")
            .to_lowercase()
    };
    !a.trim().is_empty() && Path::new(&limpia(a)) == Path::new(&limpia(b))
}

const TODOS: [&str; 3] = ["domain", "private", "public"];

/// Perfiles de una regla tal y como los da PowerShell: `Domain, Private`, `Any`…
fn perfiles_de(leido: &str) -> BTreeSet<String> {
    let minusculas = leido.to_lowercase();
    if minusculas.contains("any") || minusculas.contains("all") {
        return TODOS.iter().map(|p| p.to_string()).collect();
    }
    TODOS
        .iter()
        .filter(|p| minusculas.contains(**p))
        .map(|p| p.to_string())
        .collect()
}

fn perfiles_esperados(esperados: &[String]) -> BTreeSet<String> {
    esperados.iter().map(|p| p.to_lowercase()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entorno() -> Entorno {
        Entorno {
            puerto_control: 7411,
            puerto_base_motor: 5001,
            mdns_activo: true,
            permitir_publico: false,
            ejecutable: PathBuf::from(r"C:\Apps\NetworkBench\NetworkBench.exe"),
            motor: PathBuf::from(r"C:\Apps\NetworkBench\ntttcp.exe"),
        }
    }

    fn correcta(r: &ReglaEsperada) -> ReglaLeida {
        ReglaLeida {
            habilitada: true,
            accion: "Allow".into(),
            direccion: "Inbound".into(),
            perfil: "Domain, Private".into(),
            grupo: "NetworkBench".into(),
            protocolo: r.protocolo.clone(),
            puertos_locales: r.puertos.clone(),
            programa: r.programa.clone(),
        }
    }

    #[test]
    fn son_las_cuatro_reglas_de_14_1_con_su_puerto_y_programa() {
        let r = esperadas(&entorno());
        let resumen: Vec<_> = r
            .iter()
            .map(|x| (x.nombre.as_str(), x.protocolo.as_str(), x.puertos.as_str()))
            .collect();
        assert_eq!(
            resumen,
            vec![
                ("NetworkBench - Control", "TCP", "7411"),
                ("NetworkBench - NTTTCP TCP", "TCP", "5001-5064"),
                ("NetworkBench - NTTTCP UDP", "UDP", "5001-5064"),
                ("NetworkBench - Descubrimiento", "UDP", "5353"),
            ]
        );
        assert!(r[0].programa.ends_with("NetworkBench.exe"));
        assert!(r[1].programa.ends_with("ntttcp.exe"));
        assert!(r.iter().all(|x| x.grupo == "NetworkBench"));
    }

    #[test]
    fn sin_mdns_no_se_espera_la_regla_de_descubrimiento() {
        let mut e = entorno();
        e.mdns_activo = false;
        assert_eq!(esperadas(&e).len(), 3);
    }

    #[test]
    fn el_puerto_de_control_personalizado_cambia_la_regla_de_control() {
        let mut e = entorno();
        e.puerto_control = 9000;
        assert_eq!(esperadas(&e)[0].puertos, "9000");
    }

    #[test]
    fn por_defecto_dominio_y_privado_y_con_el_permiso_tambien_publico() {
        let mut e = entorno();
        assert_eq!(esperadas(&e)[0].perfiles, vec!["Domain", "Private"]);
        e.permitir_publico = true;
        assert!(
            esperadas(&e)
                .iter()
                .all(|r| r.perfiles == vec!["Domain", "Private", "Public"])
        );
    }

    #[test]
    fn todos_los_nombres_llevan_el_prefijo_que_exige_el_helper() {
        for n in NOMBRES_CONOCIDOS {
            assert!(
                n.starts_with(super::super::validation::PREFIJO_REGLA),
                "{n}"
            );
        }
    }

    #[test]
    fn la_peticion_al_helper_lleva_lo_esperado() {
        let r = &esperadas(&entorno())[1];
        let p = a_peticion(r, "add");
        assert_eq!(p.operation, "add");
        assert_eq!(p.rule_name, "NetworkBench - NTTTCP TCP");
        assert_eq!(p.port_range, "5001-5064");
        assert_eq!(p.profiles, vec!["Domain", "Private"]);
    }

    #[test]
    fn el_comando_manual_es_powershell_con_grupo_y_escapa_las_comillas() {
        let mut r = esperadas(&entorno())[0].clone();
        let c = comando_agregar(&r);
        assert!(c.starts_with("New-NetFirewallRule -DisplayName 'NetworkBench - Control'"));
        assert!(c.contains("-Group 'NetworkBench'"));
        assert!(c.contains("-LocalPort 7411") && c.contains("-Protocol TCP"));
        assert!(c.contains("-Profile Domain,Private"));

        r.programa = r"C:\Users\O'Brien\NetworkBench.exe".into();
        assert!(comando_agregar(&r).contains(r"-Program 'C:\Users\O''Brien\NetworkBench.exe'"));
        assert_eq!(
            comando_eliminar("NetworkBench - Control"),
            "Remove-NetFirewallRule -DisplayName 'NetworkBench - Control'"
        );
    }

    #[test]
    fn ausente_si_el_sistema_no_la_tiene() {
        let r = &esperadas(&entorno())[0];
        assert_eq!(evaluar(r, None).0, RuleStatus::Missing);
    }

    #[test]
    fn presente_si_coincide_todo_y_sin_importar_mayusculas_ni_prefijo_de_ruta() {
        let r = &esperadas(&entorno())[0];
        let mut l = correcta(r);
        l.programa = r"\\?\c:\apps\networkbench\NETWORKBENCH.EXE".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Present);
    }

    #[test]
    fn deshabilitada_si_esta_apagada() {
        let r = &esperadas(&entorno())[0];
        let mut l = correcta(r);
        l.habilitada = false;
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Disabled);
    }

    #[test]
    fn modificada_cuando_cambia_el_puerto_que_es_lo_que_pasa_al_cambiar_el_ajuste() {
        let mut e = entorno();
        let vieja = esperadas(&e)[0].clone();
        e.puerto_control = 9000;
        let nueva = &esperadas(&e)[0];
        let (estado, detalle) = evaluar(nueva, Some(&correcta(&vieja)));
        assert_eq!(estado, RuleStatus::Modified);
        assert!(detalle.contains("puertos 7411"), "{detalle}");
    }

    #[test]
    fn modificada_si_cambia_programa_protocolo_accion_o_sentido() {
        let r = &esperadas(&entorno())[1];
        for cambiar in [
            (|l: &mut ReglaLeida| l.programa = r"C:\otro\ntttcp.exe".into()) as fn(&mut ReglaLeida),
            |l| l.protocolo = "UDP".into(),
            |l| l.accion = "Block".into(),
            |l| l.direccion = "Outbound".into(),
            |l| l.programa = String::new(),
        ] {
            let mut l = correcta(r);
            cambiar(&mut l);
            assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Modified, "{l:?}");
        }
    }

    #[test]
    fn una_regla_sin_grupo_esta_desactualizada_porque_el_desinstalador_no_la_veria() {
        let r = &esperadas(&entorno())[0];
        let mut l = correcta(r);
        l.grupo = String::new();
        let (estado, detalle) = evaluar(r, Some(&l));
        assert_eq!(estado, RuleStatus::Modified);
        assert!(detalle.contains("sin el grupo NetworkBench"), "{detalle}");
    }

    #[test]
    fn los_perfiles_se_comparan_exactos_en_los_dos_sentidos() {
        let mut e = entorno();
        let r = &esperadas(&e)[0];
        let mut l = correcta(r);

        // De menos: falta uno.
        l.perfil = "Private".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Modified);
        // De más: abre la red pública sin que la persona lo haya aceptado.
        l.perfil = "Domain, Private, Public".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Modified);
        // «Any» incluye la pública.
        l.perfil = "Any".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Modified);

        // Con el permiso, lo esperado es justo lo contrario.
        e.permitir_publico = true;
        let r = &esperadas(&e)[0];
        l.perfil = "Domain, Private, Public".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Present);
        l.perfil = "Any".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Present);
        l.perfil = "Domain, Private".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Modified);
    }
}
