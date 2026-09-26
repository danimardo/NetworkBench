//! Las reglas de firewall que NetworkBench necesita (`Historias.md` §14.1) y su evaluación.
//!
//! Aquí solo hay lógica pura y testeable: qué reglas se esperan para un puerto de control y
//! un programa dados, y cómo se compara una regla leída del sistema con la esperada. Leer el
//! firewall (`inspect`) y cambiarlo (`helper_client`, con UAC) están en sus módulos.

use super::inspect::RuleStatus;
use super::validation::FirewallHelperRequest;
use serde::Serialize;
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
/// Perfiles por defecto (§14.5): Dominio y Privado. Público solo si la persona lo acepta.
pub const PERFILES_POR_DEFECTO: [&str; 2] = ["Domain", "Private"];

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
}

/// De qué depende el conjunto de reglas: los puertos configurados y dónde está cada programa.
#[derive(Debug, Clone)]
pub struct Entorno {
    pub puerto_control: u16,
    pub puerto_base_motor: u16,
    pub mdns_activo: bool,
    pub ejecutable: PathBuf,
    pub motor: PathBuf,
}

impl Entorno {
    /// Junto al ejecutable están el motor y el helper, tanto en desarrollo como instalado
    /// (mismo criterio que `app::init` para localizar `ntttcp.exe`).
    pub fn actual(puerto_control: u16, mdns_activo: bool) -> Option<Self> {
        let ejecutable = std::env::current_exe().ok()?;
        let motor = ejecutable.parent()?.join("ntttcp.exe");
        Some(Self {
            puerto_control,
            puerto_base_motor: PUERTO_BASE_MOTOR,
            mdns_activo,
            ejecutable,
            motor,
        })
    }
}

fn perfiles() -> Vec<String> {
    PERFILES_POR_DEFECTO.iter().map(|p| p.to_string()).collect()
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
        perfiles: perfiles(),
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

/// `netsh` equivalente, para quien prefiera (o necesite, con una directiva corporativa)
/// crear la regla a mano. Solo texto para copiar: la aplicación no lo ejecuta.
pub fn netsh_agregar(r: &ReglaEsperada) -> String {
    format!(
        "netsh advfirewall firewall add rule name=\"{}\" dir=in action=allow protocol={} localport={} program=\"{}\" profile={}",
        r.nombre,
        r.protocolo.to_lowercase(),
        r.puertos,
        r.programa,
        r.perfiles.join(",").to_lowercase()
    )
}

pub fn netsh_eliminar(nombre: &str) -> String {
    format!("netsh advfirewall firewall delete rule name=\"{nombre}\"")
}

/// Una regla tal y como la devuelve el sistema (`Get-NetFirewallRule` y sus filtros).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReglaLeida {
    pub habilitada: bool,
    pub accion: String,
    pub direccion: String,
    pub perfil: String,
    pub protocolo: String,
    pub puertos_locales: String,
    pub programa: String,
}

/// Compara la regla leída con la esperada.
///
/// «Modificada» es cualquier campo que ya no casa (acción, sentido, protocolo, puertos,
/// programa o perfiles). Que la regla tenga **además** otros perfiles no la modifica: quien
/// permite también la red pública no la ha roto.
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
    if !cubre_perfiles(&l.perfil, &esperada.perfiles) {
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

/// La regla cubre los perfiles esperados si los incluye todos, o si vale para cualquiera.
fn cubre_perfiles(leido: &str, esperados: &[String]) -> bool {
    let leido = leido.to_lowercase();
    if leido.contains("any") || leido.contains("all") {
        return true;
    }
    esperados.iter().all(|p| leido.contains(&p.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entorno() -> Entorno {
        Entorno {
            puerto_control: 7411,
            puerto_base_motor: 5001,
            mdns_activo: true,
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
    fn el_netsh_para_copiar_es_el_equivalente_de_la_regla() {
        let r = &esperadas(&entorno())[0];
        let n = netsh_agregar(r);
        assert!(n.contains("name=\"NetworkBench - Control\""));
        assert!(n.contains("localport=7411") && n.contains("protocol=tcp"));
        assert!(n.contains("profile=domain,private"));
        assert_eq!(
            netsh_eliminar("NetworkBench - Control"),
            "netsh advfirewall firewall delete rule name=\"NetworkBench - Control\""
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
        l.programa = r#"\\?\c:\apps\networkbench\NETWORKBENCH.EXE"#.into();
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
    fn permitir_ademas_la_red_publica_no_la_modifica_pero_quitar_un_perfil_si() {
        let r = &esperadas(&entorno())[0];
        let mut l = correcta(r);
        l.perfil = "Domain, Private, Public".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Present);
        l.perfil = "Private".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Modified);
        l.perfil = "Any".into();
        assert_eq!(evaluar(r, Some(&l)).0, RuleStatus::Present);
    }
}
