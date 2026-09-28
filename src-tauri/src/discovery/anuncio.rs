//! Anuncio y descubrimiento en LAN por mDNS/DNS-SD (T151, FR-010, `Historias.md` §7.1).
//!
//! Hasta esta tarea, `mdns-sd` era una dependencia sin uso y `discovery/mdns.rs` solo
//! conectaba a mano por IP: dos equipos en la misma red no se veían.
//!
//! **Lo que llega por mDNS no es confiable** (FR-011, plan §Plan de seguridad: «mDNS» está
//! entre los actores no confiables). Cualquiera en la red puede anunciar cualquier nombre,
//! identificador o huella. Por eso esto solo produce **pistas para mostrar**: no concede
//! confianza, no guarda nada y no se usa para autenticar. La identidad real se demuestra
//! en el handshake TLS con la huella del certificado; la `fp` anunciada es una pista que
//! esa conexión puede desmentir.
//!
//! Todo campo de texto anunciado se valida y se acota antes de existir como dato del
//! programa (`equipo_desde_anuncio`, pura y probada con entradas hostiles).

use crate::identity::InstanceIdentity;
use crate::netinfo::resolve::sanitize_display_name;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// `_networkbench._tcp.local.` (`Historias.md` §7.1). Antes constaba `_netbench`, que no
/// coincidía con la especificación y habría hecho invisible a la aplicación para cualquier
/// otra implementación.
pub const MDNS_SERVICE_TYPE: &str = "_networkbench._tcp.local.";

const MAX_DIRECCIONES: usize = 4;
const MAX_TXT: usize = 64;

/// Un equipo visto en la red. **Ningún campo es una prueba de identidad.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipoDescubierto {
    pub instance_id: Uuid,
    /// Lo que el equipo dice que es su huella. Una pista: la que vale es la del
    /// certificado que presente al conectar.
    pub fingerprint_declarada: String,
    pub display_name: String,
    /// `ip:puerto`, IPv4 primero (§7.1: «IPv4 preferida»).
    pub addresses: Vec<String>,
    pub protocol_version: u32,
    pub app_version: String,
    /// Velocidad de enlace anunciada en Mbit/s; `0` si no la declara.
    pub link_mbps: u32,
    pub busy: bool,
}

fn es_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn version_acotada(s: &str) -> String {
    if !s.is_empty()
        && s.len() <= 32
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || ".+-".contains(c))
    {
        s.to_string()
    } else {
        "?".to_string()
    }
}

/// Una dirección que merece ofrecerse: descarta las que no sirven para conectar desde otro
/// equipo (sin especificar, multidifusión, bucle local) y las IPv6 de enlace local, que
/// sin identificador de ámbito no son alcanzables.
fn direccion_util(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            !(v4.is_unspecified() || v4.is_multicast() || v4.is_loopback() || v4.is_broadcast())
        }
        IpAddr::V6(v6) => {
            !(v6.is_unspecified()
                || v6.is_multicast()
                || v6.is_loopback()
                || (v6.segments()[0] & 0xffc0) == 0xfe80)
        }
    }
}

/// Compara los tres primeros octetos: mismo segmento /24, la señal barata de "esto se
/// alcanza directamente, sin pasar por una puerta de enlace" que usa `equipo_desde_anuncio`
/// para ordenar direcciones candidatas.
fn mismo_segmento_24(a: &std::net::Ipv4Addr, b: &std::net::Ipv4Addr) -> bool {
    a.octets()[..3] == b.octets()[..3]
}

/// La implementación real de `alcanzable_directamente`: le pregunta a la tabla de rutas del
/// sistema qué interfaz local usaría para llegar a `ip` (la misma técnica ya probada en
/// `netinfo::find_best_interface_for_target`, un socket UDP sin conectar de verdad) y
/// comprueba si esa interfaz está en el mismo segmento que `ip` — si lo está, es una ruta
/// directa; si no, hace falta una puerta de enlace (o no se puede llegar en absoluto).
fn alcanzable_directamente(ip: &IpAddr) -> bool {
    let IpAddr::V4(candidata) = ip else {
        // El criterio es solo para desempatar entre IPv4: las IPv6 ya van después de
        // cualquier IPv4 por el primer criterio de orden.
        return false;
    };
    crate::netinfo::find_best_interface_for_target(*ip)
        .ok()
        .and_then(|iface| iface.ip_address.parse::<std::net::Ipv4Addr>().ok())
        .is_some_and(|local| mismo_segmento_24(&local, candidata))
}

/// Convierte un anuncio en un `EquipoDescubierto`, o lo rechaza entero. Pura y estricta: un
/// anuncio a medias o hostil no produce un equipo «casi válido».
///
/// `alcanzable_directamente` decide, para cada dirección candidata, si se puede llegar a
/// ella sin pasar por una puerta de enlace (hallazgo real del propietario, 2026-09-27: un
/// equipo con una VPN u otro adaptador virtual anunciaba también esa IP —p. ej.
/// 10.10.10.122—, y el orden numérico simple la ponía por delante de la LAN real
/// —192.168.1.226—, aunque solo la segunda fuera alcanzable). Se inyecta en vez de
/// resolverse aquí dentro para que la función siga siendo pura y comprobable con entradas
/// sintéticas, sin abrir sockets de verdad en los tests; quien la llama de verdad usa
/// `netinfo::find_best_interface_for_target`.
pub fn equipo_desde_anuncio(
    txt: impl Fn(&str) -> Option<String>,
    direcciones: impl IntoIterator<Item = IpAddr>,
    puerto: u16,
    alcanzable_directamente: impl Fn(&IpAddr) -> bool,
) -> Result<EquipoDescubierto, String> {
    let leer = |clave: &str| -> Result<String, String> {
        let v = txt(clave).ok_or_else(|| format!("Falta el registro TXT '{clave}'"))?;
        if v.len() > MAX_TXT || v.chars().any(char::is_control) {
            return Err(format!("Registro TXT '{clave}' no válido"));
        }
        Ok(v)
    };

    let instance_id =
        Uuid::parse_str(&leer("id")?).map_err(|_| "Identificador no válido".to_string())?;
    let fp = leer("fp")?.to_lowercase();
    if !es_hex64(&fp) {
        return Err("Huella anunciada no válida".into());
    }
    // Nunca queda vacío: sin nombre utilizable, `sanitize_display_name` da uno de reserva,
    // igual que en la conexión manual.
    let nombre = sanitize_display_name(&leer("name")?);
    let protocol_version: u32 = leer("v")?
        .parse()
        .map_err(|_| "Versión de protocolo no válida".to_string())?;

    if puerto < 1024 {
        return Err(format!("Puerto anunciado fuera de rango: {puerto}"));
    }

    let mut ips: Vec<IpAddr> = direcciones.into_iter().filter(direccion_util).collect();
    // IPv4 antes que IPv6 (§7.1); dentro de cada familia, la directamente alcanzable antes
    // que una que solo se anunció porque el otro equipo tiene esa interfaz también (VPN,
    // adaptador virtual...) — y solo como último criterio, el orden numérico, que por sí
    // solo no dice nada sobre cuál es alcanzable.
    ips.sort_by_key(|ip| (ip.is_ipv6(), !alcanzable_directamente(ip), *ip));
    ips.dedup();
    ips.truncate(MAX_DIRECCIONES);
    if ips.is_empty() {
        return Err("El anuncio no trae ninguna dirección alcanzable".into());
    }

    Ok(EquipoDescubierto {
        instance_id,
        fingerprint_declarada: fp,
        display_name: nombre,
        addresses: ips
            .into_iter()
            .map(|ip| std::net::SocketAddr::new(ip, puerto).to_string())
            .collect(),
        protocol_version,
        app_version: version_acotada(&txt("app").unwrap_or_default()),
        // Tope: un valor absurdo no debe poder desbordar ni desfigurar la interfaz.
        link_mbps: txt("link")
            .and_then(|l| l.parse().ok())
            .unwrap_or(0)
            .min(1_000_000),
        busy: txt("busy").as_deref() == Some("1"),
    })
}

/// Nombre de instancia: `<displayName> [<instanceId corto 8 hex>]` (§7.1).
pub fn nombre_de_instancia(display_name: &str, instance_id: &Uuid) -> String {
    let corto: String = instance_id.simple().to_string().chars().take(8).collect();
    format!("{} [{}]", sanitize_display_name(display_name), corto)
}

type Mapa = Arc<Mutex<HashMap<String, EquipoDescubierto>>>;

/// Publica este equipo y navega por los demás mientras exista.
pub struct Descubrimiento {
    daemon: ServiceDaemon,
    equipos: Mapa,
    fullname: String,
}

impl Descubrimiento {
    /// Publica el servicio y empieza a navegar. Falla si no hay forma de abrir mDNS; quien
    /// llama decide si eso es un aviso (sin descubrimiento se puede conectar a mano, FR-010).
    pub fn iniciar(
        identidad: &InstanceIdentity,
        puerto_control: u16,
        app_version: &str,
        protocol_version: u32,
    ) -> Result<Self, String> {
        let daemon = ServiceDaemon::new().map_err(|e| format!("No se pudo abrir mDNS: {e}"))?;

        let corto: String = identidad
            .instance_id
            .simple()
            .to_string()
            .chars()
            .take(8)
            .collect();
        let instancia = nombre_de_instancia(&identidad.display_name, &identidad.instance_id);
        let propiedades: HashMap<String, String> = [
            ("v", protocol_version.to_string()),
            ("app", app_version.to_string()),
            ("id", identidad.instance_id.to_string()),
            ("fp", identidad.fingerprint.to_lowercase()),
            ("name", sanitize_display_name(&identidad.display_name)),
            ("link", "0".to_string()),
            ("busy", "0".to_string()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

        // Direcciones automáticas: el daemon anuncia las de cada interfaz y las actualiza si
        // cambian, en vez de fijar una IP que puede no ser la que ve la otra red.
        let info = ServiceInfo::new(
            MDNS_SERVICE_TYPE,
            &instancia,
            &format!("nb-{corto}.local."),
            "",
            puerto_control,
            propiedades,
        )
        .map_err(|e| format!("Anuncio mDNS no válido: {e}"))?
        .enable_addr_auto();
        let fullname = info.get_fullname().to_string();
        daemon
            .register(info)
            .map_err(|e| format!("No se pudo publicar en mDNS: {e}"))?;
        // Hallazgo real (2026-09-28): sin esto, ni el propio anuncio ni ver/perder un
        // equipo dejaban ningún rastro — para reconstruir una prueba entre dos máquinas a
        // partir de sus logs hace falta saber, como mínimo, que cada una se anunció.
        tracing::info!(
            instance_id = %corto,
            puerto = puerto_control,
            "mDNS: anunciando este equipo ({fullname})"
        );

        let receptor = daemon
            .browse(MDNS_SERVICE_TYPE)
            .map_err(|e| format!("No se pudo navegar por mDNS: {e}"))?;

        let equipos: Mapa = Arc::default();
        let mi_id = identidad.instance_id;
        let destino = Arc::clone(&equipos);
        std::thread::Builder::new()
            .name("nb-mdns".into())
            .spawn(move || {
                // Termina solo cuando el daemon se apaga y cierra el canal.
                while let Ok(evento) = receptor.recv() {
                    match evento {
                        ServiceEvent::ServiceResolved(r) => {
                            let ips = r.addresses.iter().map(|a| a.to_ip_addr());
                            match equipo_desde_anuncio(
                                |k| r.get_property_val_str(k).map(str::to_string),
                                ips,
                                r.port,
                                alcanzable_directamente,
                            ) {
                                // Uno mismo aparece en su propia navegación: no es un «otro equipo».
                                Ok(e) if e.instance_id == mi_id => {}
                                Ok(e) => {
                                    // Nivel info, no debug: es el evento que responde a "¿llegó a
                                    // verlo?" en una prueba entre dos máquinas, no un detalle interno.
                                    tracing::info!(
                                        instance_id = %e.instance_id,
                                        direcciones = %e.addresses.join(","),
                                        "mDNS: equipo visto ({}, {})",
                                        e.display_name,
                                        r.fullname
                                    );
                                    if let Ok(mut m) = destino.lock() {
                                        m.insert(r.fullname.clone(), e);
                                    }
                                }
                                Err(motivo) => {
                                    tracing::warn!(
                                        "mDNS: anuncio descartado ({}): {motivo}",
                                        r.fullname
                                    );
                                }
                            }
                        }
                        ServiceEvent::ServiceRemoved(_, nombre) => {
                            if let Ok(mut m) = destino.lock() {
                                let existia = m.remove(&nombre);
                                if let Some(e) = existia {
                                    tracing::info!(
                                        instance_id = %e.instance_id,
                                        "mDNS: equipo perdido ({}, {nombre})",
                                        e.display_name
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
            .map_err(|e| format!("No se pudo lanzar la escucha mDNS: {e}"))?;

        Ok(Self {
            daemon,
            equipos,
            fullname,
        })
    }

    /// Los demás equipos que se ven ahora mismo, ordenados por nombre.
    pub fn equipos(&self) -> Vec<EquipoDescubierto> {
        let mut v: Vec<_> = self
            .equipos
            .lock()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        v.sort_by(|a, b| {
            a.display_name
                .cmp(&b.display_name)
                .then(a.instance_id.cmp(&b.instance_id))
        });
        v
    }
}

impl Drop for Descubrimiento {
    /// Retira el anuncio y apaga el daemon: un equipo que se cierra no debe seguir figurando
    /// en las listas de los demás hasta que caduque su registro.
    fn drop(&mut self) {
        let _ = self.daemon.unregister(&self.fullname);
        let _ = self.daemon.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn txt_valido() -> HashMap<&'static str, String> {
        HashMap::from([
            ("id", Uuid::new_v4().to_string()),
            ("fp", "a".repeat(64)),
            ("name", "Portátil de Ana".to_string()),
            ("v", "1".to_string()),
            ("app", "0.1.0".to_string()),
            ("link", "1000".to_string()),
            ("busy", "0".to_string()),
        ])
    }

    fn parsear(
        txt: &HashMap<&'static str, String>,
        ips: &[&str],
        puerto: u16,
    ) -> Result<EquipoDescubierto, String> {
        // Sin criterio de alcanzabilidad: estos tests comprueban el resto de la función, no
        // el orden por alcanzabilidad directa (que tiene los suyos propios, más abajo).
        parsear_con(txt, ips, puerto, |_| false)
    }

    fn parsear_con(
        txt: &HashMap<&'static str, String>,
        ips: &[&str],
        puerto: u16,
        alcanzable: impl Fn(&IpAddr) -> bool,
    ) -> Result<EquipoDescubierto, String> {
        equipo_desde_anuncio(
            |k| txt.get(k).cloned(),
            ips.iter().map(|i| i.parse::<IpAddr>().unwrap()),
            puerto,
            alcanzable,
        )
    }

    #[test]
    fn un_anuncio_valido_se_convierte_en_un_equipo_con_ipv4_primero() {
        let e = parsear(&txt_valido(), &["2001:db8::5", "192.168.1.20"], 7411).unwrap();
        assert_eq!(e.display_name, "Portátil de Ana");
        assert_eq!(e.addresses, vec!["192.168.1.20:7411", "[2001:db8::5]:7411"]);
        assert_eq!(e.link_mbps, 1000);
        assert!(!e.busy);
        assert_eq!(e.fingerprint_declarada, "a".repeat(64));
    }

    #[test]
    fn mismo_segmento_24_compara_solo_los_tres_primeros_octetos() {
        use std::net::Ipv4Addr;
        assert!(mismo_segmento_24(
            &Ipv4Addr::new(192, 168, 1, 226),
            &Ipv4Addr::new(192, 168, 1, 20)
        ));
        assert!(!mismo_segmento_24(
            &Ipv4Addr::new(192, 168, 1, 226),
            &Ipv4Addr::new(10, 10, 10, 122)
        ));
    }

    #[test]
    fn la_direccion_alcanzable_directamente_ordena_antes_que_una_de_una_vpn_o_adaptador_virtual() {
        // Hallazgo real del propietario, 2026-09-27: un equipo con VPN anunciaba tanto su IP
        // de VPN (10.10.10.122) como la de su LAN real (192.168.1.226); el simple orden
        // numérico ponía la primera delante, aunque solo la segunda fuera alcanzable.
        let e = parsear_con(
            &txt_valido(),
            &["10.10.10.122", "192.168.1.226"],
            7411,
            |ip| *ip == "192.168.1.226".parse::<IpAddr>().unwrap(),
        )
        .unwrap();
        assert_eq!(e.addresses, vec!["192.168.1.226:7411", "10.10.10.122:7411"]);
    }

    #[test]
    fn sin_ninguna_alcanzable_directamente_se_mantiene_el_orden_numerico_de_siempre() {
        let e = parsear_con(
            &txt_valido(),
            &["10.10.10.122", "192.168.1.226"],
            7411,
            |_| false,
        )
        .unwrap();
        assert_eq!(e.addresses, vec!["10.10.10.122:7411", "192.168.1.226:7411"]);
    }

    #[test]
    fn el_criterio_de_alcanzabilidad_no_desplaza_el_orden_ipv4_antes_que_ipv6() {
        // Aunque la única IPv6 fuera "alcanzable" y la IPv4 no, IPv4 sigue yendo primero
        // (§7.1): el criterio nuevo solo desempata dentro de la misma familia.
        let e = parsear_con(
            &txt_valido(),
            &["2001:db8::5", "192.168.1.20"],
            7411,
            |ip| ip.is_ipv6(),
        )
        .unwrap();
        assert_eq!(e.addresses, vec!["192.168.1.20:7411", "[2001:db8::5]:7411"]);
    }

    #[test]
    fn la_implementacion_real_encuentra_localhost_alcanzable_directamente() {
        // Igual que test_find_best_interface_for_localhost en netinfo: 127.0.0.1 debe
        // resolver a sí mismo como "mismo segmento", sin necesitar puerta de enlace.
        let localhost = "127.0.0.1".parse::<IpAddr>().unwrap();
        assert!(alcanzable_directamente(&localhost));
    }

    /// Lo anunciado no es confiable: cada campo que no encaja rechaza el anuncio entero.
    #[test]
    fn los_anuncios_hostiles_o_incompletos_se_rechazan() {
        let mutar = |clave: &'static str, valor: &str| {
            let mut t = txt_valido();
            t.insert(clave, valor.to_string());
            parsear(&t, &["192.168.1.20"], 7411)
        };
        assert!(mutar("id", "no-es-un-uuid").is_err());
        assert!(
            mutar("fp", &"z".repeat(64)).is_err(),
            "huella no hexadecimal"
        );
        assert!(mutar("fp", "abc").is_err(), "huella corta");
        assert!(mutar("v", "uno").is_err());
        assert_eq!(
            mutar("name", "   ").unwrap().display_name,
            "Equipo-Remoto",
            "un nombre en blanco usa el de reserva, no queda vacío"
        );
        assert!(
            mutar("name", &"x".repeat(200)).is_err(),
            "registro demasiado largo"
        );
        assert!(mutar("name", "a\u{0000}b").is_err(), "carácter de control");

        for falta in ["id", "fp", "name", "v"] {
            let mut t = txt_valido();
            t.remove(falta);
            assert!(parsear(&t, &["192.168.1.20"], 7411).is_err(), "sin {falta}");
        }
        assert!(
            parsear(&txt_valido(), &["192.168.1.20"], 80).is_err(),
            "puerto < 1024"
        );
        assert!(
            parsear(&txt_valido(), &[], 7411).is_err(),
            "sin direcciones"
        );
    }

    #[test]
    fn el_nombre_anunciado_se_sanea_antes_de_mostrarse() {
        let mut t = txt_valido();
        // Marca de dirección bidireccional: podría invertir el texto que ve el usuario.
        t.insert("name", "Equipo\u{202E}odnuM".to_string());
        let e = parsear(&t, &["192.168.1.20"], 7411).unwrap();
        assert!(!e.display_name.contains('\u{202E}'), "{:?}", e.display_name);
    }

    #[test]
    fn las_direcciones_que_no_sirven_para_conectar_se_descartan() {
        let e = parsear(
            &txt_valido(),
            &[
                "0.0.0.0",
                "127.0.0.1",
                "224.0.0.251",
                "fe80::1",
                "::1",
                "10.0.0.7",
            ],
            7411,
        )
        .unwrap();
        assert_eq!(e.addresses, vec!["10.0.0.7:7411"]);
        assert!(parsear(&txt_valido(), &["127.0.0.1", "fe80::1"], 7411).is_err());
    }

    #[test]
    fn los_campos_opcionales_no_pueden_desfigurar_nada() {
        let mut t = txt_valido();
        t.insert("link", "99999999999999999999".to_string());
        t.insert("app", "<script>alert(1)</script>".to_string());
        t.insert("busy", "quizá".to_string());
        let e = parsear(&t, &["192.168.1.20"], 7411).unwrap();
        assert_eq!(e.link_mbps, 0, "un número que no cabe se descarta");
        assert_eq!(
            e.app_version, "?",
            "una versión con caracteres raros no se muestra"
        );
        assert!(!e.busy);

        t.insert("link", "4294967295".to_string());
        assert_eq!(
            parsear(&t, &["192.168.1.20"], 7411).unwrap().link_mbps,
            1_000_000
        );
    }

    #[test]
    fn el_nombre_de_instancia_sigue_la_especificacion() {
        let id = Uuid::parse_str("0123abcd-0000-4000-8000-000000000000").unwrap();
        assert_eq!(
            nombre_de_instancia("Mi equipo", &id),
            "Mi equipo [0123abcd]"
        );
    }

    #[test]
    fn el_tipo_de_servicio_es_el_de_la_especificacion() {
        assert_eq!(MDNS_SERVICE_TYPE, "_networkbench._tcp.local.");
    }
}
