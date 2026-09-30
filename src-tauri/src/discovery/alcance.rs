//! Comprobación periódica de si los equipos guardados están activos en la red.
//!
//! mDNS dice quién se anuncia **ahora**, pero un equipo guardado puede estar encendido y no
//! anunciarse (otra subred, mDNS bloqueado, la aplicación abierta hace un instante). Por eso,
//! para cada equipo guardado que mDNS no ve, se intenta una conexión TCP corta al puerto de
//! control de la última dirección en la que se le vio. No se usa ICMP: Windows bloquea el
//! eco entrante por defecto y un `ping` tampoco dice si NetworkBench está abierto.
//!
//! **Es una pista para la interfaz, no una prueba de identidad.** Un TCP que conecta solo
//! demuestra que algo escucha en esa dirección. Quien sea de verdad se demuestra en el
//! handshake TLS con la huella guardada (FR-013) al medir. Solo se comprueban equipos que ya
//! están guardados: nunca se barre la red.
//!
//! Para no parpadear: un equipo que estaba alcanzable necesita **dos** fallos seguidos para
//! pasar a inalcanzable (el segundo intento llega a los 10 s). Uno que nunca ha respondido
//! se marca inalcanzable a la primera, para que el arranque ya muestre la verdad.

use super::EquipoDescubierto;
use crate::history::PeerParaAlcance;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use uuid::Uuid;

/// Cada cuánto se repite la comprobación de un equipo alcanzable.
pub const INTERVALO: Duration = Duration::from_secs(30);
/// Reintento tras un primer fallo de un equipo que estaba alcanzable.
pub const CONFIRMACION: Duration = Duration::from_secs(10);
/// Tiempo máximo de un intento de conexión.
pub const TIEMPO_MAXIMO: Duration = Duration::from_millis(1500);
/// Cada cuánto se despierta el bucle a mirar si hay algo vencido.
pub const LATIDO: Duration = Duration::from_secs(5);
/// Al arrancar, mDNS aún no ha visto a nadie: un equipo sin dirección guardada espera este
/// margen a que lo anuncie antes de darlo por inalcanzable.
const GRACIA_DE_ARRANQUE: Duration = Duration::from_secs(6);
/// No se reescribe la base de datos por cada latido de un equipo que se sigue viendo.
const REGISTRO_MINIMO: Duration = Duration::from_secs(60);
const FALLOS_PARA_CAER: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Alcance {
    #[serde(rename = "checking")]
    Comprobando,
    #[serde(rename = "reachable")]
    Alcanzable,
    #[serde(rename = "unreachable")]
    Inalcanzable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoAlcance {
    pub fingerprint: String,
    pub estado: Alcance,
}

/// Qué anotar en la base de datos tras una ronda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Avistamiento {
    pub fingerprint: String,
    pub direccion: Option<String>,
}

#[derive(Debug, Clone)]
struct Entrada {
    instance_id: Uuid,
    estado: Alcance,
    fallos: u32,
    proxima: Instant,
    ultimo_registro: Option<Instant>,
}

/// Espera hasta la siguiente comprobación de un equipo que lleva `fallos` seguidos:
/// 30 s, 30 s, 1 min, 2 min y, a partir de ahí, 5 min. Un equipo apagado no se martillea.
pub fn espera_tras_fallos(fallos: u32) -> Duration {
    match fallos {
        0..=2 => INTERVALO,
        3 => Duration::from_secs(60),
        4 => Duration::from_secs(120),
        _ => Duration::from_secs(300),
    }
}

/// Resultado de aplicar una comprobación a la entrada anterior: pura y sin red.
fn aplicar_resultado(
    anterior: Option<&Entrada>,
    instance_id: Uuid,
    ok: bool,
    ahora: Instant,
) -> Entrada {
    let (estado_previo, fallos_previos, ultimo_registro) = match anterior {
        Some(e) => (e.estado, e.fallos, e.ultimo_registro),
        None => (Alcance::Comprobando, 0, None),
    };
    if ok {
        return Entrada {
            instance_id,
            estado: Alcance::Alcanzable,
            fallos: 0,
            proxima: ahora + INTERVALO,
            ultimo_registro,
        };
    }
    let fallos = fallos_previos + 1;
    // Un equipo visto hace un momento no cae por un solo fallo: se reintenta enseguida.
    if estado_previo == Alcance::Alcanzable && fallos < FALLOS_PARA_CAER {
        return Entrada {
            instance_id,
            estado: Alcance::Alcanzable,
            fallos,
            proxima: ahora + CONFIRMACION,
            ultimo_registro,
        };
    }
    Entrada {
        instance_id,
        estado: Alcance::Inalcanzable,
        fallos,
        proxima: ahora + espera_tras_fallos(fallos),
        ultimo_registro,
    }
}

pub struct MonitorAlcance {
    entradas: Mutex<HashMap<String, Entrada>>,
    /// Despierta el bucle antes del latido («Comprobar ahora», «Buscar de nuevo»).
    pub despertar: Notify,
    inicio: Instant,
}

impl Default for MonitorAlcance {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitorAlcance {
    pub fn new() -> Self {
        Self {
            entradas: Mutex::new(HashMap::new()),
            despertar: Notify::new(),
            inicio: Instant::now(),
        }
    }

    /// Estado de cada equipo comprobado. Un equipo que no figura aún se muestra como
    /// «comprobando».
    pub fn estados(&self) -> Vec<EstadoAlcance> {
        let entradas = self.entradas.lock().unwrap_or_else(|e| e.into_inner());
        let mut v: Vec<_> = entradas
            .iter()
            .map(|(fp, e)| EstadoAlcance {
                fingerprint: fp.clone(),
                estado: e.estado,
            })
            .collect();
        v.sort_by(|a, b| a.fingerprint.cmp(&b.fingerprint));
        v
    }

    /// «Comprobar ahora» sobre un equipo: vuelve a «comprobando» y se sondea en la próxima
    /// vuelta. `None` reprograma a todos sin borrar su estado (no hace parpadear la lista).
    pub fn forzar(&self, fingerprint: Option<&str>) {
        let ahora = Instant::now();
        {
            let mut entradas = self.entradas.lock().unwrap_or_else(|e| e.into_inner());
            match fingerprint {
                Some(fp) => {
                    if let Some(e) = entradas.get_mut(fp) {
                        e.estado = Alcance::Comprobando;
                        e.fallos = 0;
                        e.proxima = ahora;
                    }
                }
                None => entradas.values_mut().for_each(|e| e.proxima = ahora),
            }
        }
        self.despertar.notify_one();
    }

    /// Una vuelta: reconoce a los que mDNS ve, sondea a los vencidos y devuelve qué anotar.
    ///
    /// `sondear` recibe la dirección y dice si algo contestó; se inyecta para probar la
    /// lógica sin abrir sockets.
    pub async fn ronda<S, F>(
        &self,
        ahora: Instant,
        guardados: &[PeerParaAlcance],
        vistos: &[EquipoDescubierto],
        sondear: S,
    ) -> Vec<Avistamiento>
    where
        S: Fn(SocketAddr) -> F + Clone + Send + 'static,
        F: Future<Output = bool> + Send + 'static,
    {
        let mut avistamientos = Vec::new();
        let mut a_sondear: Vec<(String, Uuid, SocketAddr, String)> = Vec::new();

        {
            let mut entradas = self.entradas.lock().unwrap_or_else(|e| e.into_inner());
            let vivos: HashSet<&str> = guardados.iter().map(|g| g.fingerprint.as_str()).collect();
            entradas.retain(|fp, _| vivos.contains(fp.as_str()));

            for g in guardados {
                let visto = vistos.iter().find(|v| {
                    v.instance_id == g.instance_id
                        && v.fingerprint_declarada.eq_ignore_ascii_case(&g.fingerprint)
                });
                let previa = entradas.get(&g.fingerprint).cloned();

                if let Some(v) = visto {
                    let nueva = aplicar_resultado(previa.as_ref(), g.instance_id, true, ahora);
                    let direccion = v.addresses.first().cloned();
                    let toca_registrar = previa
                        .as_ref()
                        .and_then(|p| p.ultimo_registro)
                        .is_none_or(|t| ahora.duration_since(t) >= REGISTRO_MINIMO)
                        || direccion != g.last_address;
                    let mut nueva = nueva;
                    if toca_registrar {
                        nueva.ultimo_registro = Some(ahora);
                        avistamientos.push(Avistamiento {
                            fingerprint: g.fingerprint.clone(),
                            direccion,
                        });
                    }
                    registrar_cambio(previa.as_ref(), &nueva);
                    entradas.insert(g.fingerprint.clone(), nueva);
                    continue;
                }

                if previa.as_ref().is_some_and(|p| ahora < p.proxima) {
                    continue;
                }

                match g.last_address.as_deref().map(str::parse::<SocketAddr>) {
                    Some(Ok(addr)) => {
                        a_sondear.push((
                            g.fingerprint.clone(),
                            g.instance_id,
                            addr,
                            g.last_address.clone().unwrap_or_default(),
                        ));
                    }
                    _ => {
                        // Sin dirección no hay a quién llamar: espera a que mDNS lo anuncie y,
                        // pasada la gracia del arranque, lo da por inalcanzable.
                        if ahora.duration_since(self.inicio) < GRACIA_DE_ARRANQUE {
                            let mut e = previa.clone().unwrap_or(Entrada {
                                instance_id: g.instance_id,
                                estado: Alcance::Comprobando,
                                fallos: 0,
                                proxima: ahora,
                                ultimo_registro: None,
                            });
                            e.proxima = self.inicio + GRACIA_DE_ARRANQUE;
                            entradas.insert(g.fingerprint.clone(), e);
                        } else {
                            let nueva =
                                aplicar_resultado(previa.as_ref(), g.instance_id, false, ahora);
                            registrar_cambio(previa.as_ref(), &nueva);
                            entradas.insert(g.fingerprint.clone(), nueva);
                        }
                    }
                }
            }
        }

        if a_sondear.is_empty() {
            return avistamientos;
        }

        let mut tareas = tokio::task::JoinSet::new();
        for (fp, id, addr, texto) in a_sondear {
            let sondear = sondear.clone();
            tareas.spawn(async move { (fp, id, texto, sondear(addr).await) });
        }
        let mut resultados = Vec::new();
        while let Some(r) = tareas.join_next().await {
            if let Ok(r) = r {
                resultados.push(r);
            }
        }

        let mut entradas = self.entradas.lock().unwrap_or_else(|e| e.into_inner());
        for (fp, id, texto, ok) in resultados {
            // Puede haberse olvidado el equipo mientras se sondeaba.
            let previa = entradas.get(&fp).cloned();
            if previa.is_none() && !guardados.iter().any(|g| g.fingerprint == fp) {
                continue;
            }
            let mut nueva = aplicar_resultado(previa.as_ref(), id, ok, ahora);
            if ok {
                nueva.ultimo_registro = Some(ahora);
                avistamientos.push(Avistamiento {
                    fingerprint: fp.clone(),
                    direccion: Some(texto),
                });
            }
            registrar_cambio(previa.as_ref(), &nueva);
            entradas.insert(fp, nueva);
        }
        avistamientos
    }
}

/// Deja rastro solo cuando cambia el estado visible: es lo que hace falta para reconstruir
/// una prueba entre dos máquinas («¿lo daba por caído cuando lo probé?»). La dirección no
/// se registra aquí: es detalle de diagnóstico avanzado (constitución 0.8.0, principio XIII).
fn registrar_cambio(antes: Option<&Entrada>, ahora: &Entrada) {
    let antes = antes.map(|e| e.estado);
    if antes != Some(ahora.estado) {
        tracing::info!(
            instance_id = %ahora.instance_id,
            "Alcance: equipo guardado {:?} -> {:?} (fallos seguidos: {})",
            antes.unwrap_or(Alcance::Comprobando),
            ahora.estado,
            ahora.fallos
        );
    }
}

/// Espera a que `obtener` devuelva algo, mirando cada `cada`. Existe por un fallo real
/// (2026-09-30): `vigilar` se lanza durante `.setup()`, antes de que `AppState` esté
/// registrado, y `app.state()` hace `panic` si aún no existe. Una tarea de Tauri que hace
/// `panic` muere sin dejar rastro, y las tarjetas se quedaban en «Comprobando…» para siempre.
async fn esperar<T>(cada: Duration, mut obtener: impl FnMut() -> Option<T>) -> T {
    loop {
        if let Some(v) = obtener() {
            return v;
        }
        tokio::time::sleep(cada).await;
    }
}

/// El bucle de fondo: comprueba al arrancar y después cada `LATIDO`, o antes si algo lo
/// despierta. Se pausa mientras hay una medición en curso (no debe competir con ella).
pub async fn vigilar(app: tauri::AppHandle) {
    use tauri::Manager;
    // `try_state`, nunca `state`: ver `esperar`.
    let estado = esperar(Duration::from_millis(100), || {
        app.try_state::<crate::app::AppState>()
    })
    .await;
    tracing::info!("Alcance: comprobación de equipos guardados en marcha");
    loop {
        if !estado.snapshot.get_snapshot().is_session_active {
            let guardados = {
                let db = estado.database.connection().lock().unwrap();
                crate::history::peers_para_alcance(&db).unwrap_or_default()
            };
            let vistos = estado
                .descubrimiento
                .lock()
                .ok()
                .and_then(|d| d.as_ref().map(|d| d.equipos()))
                .unwrap_or_default();
            let avistamientos = estado
                .alcance
                .ronda(Instant::now(), &guardados, &vistos, sondear_tcp)
                .await;
            if !avistamientos.is_empty() {
                let ahora = crate::control::server::ahora_rfc3339();
                let db = estado.database.connection().lock().unwrap();
                for a in &avistamientos {
                    let _ = crate::history::registrar_avistamiento(
                        &db,
                        &a.fingerprint,
                        a.direccion.as_deref(),
                        &ahora,
                    );
                }
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(LATIDO) => {}
            _ = estado.alcance.despertar.notified() => {}
        }
    }
}

/// La comprobación real: ¿acepta una conexión TCP esa dirección en `TIEMPO_MAXIMO`?
pub async fn sondear_tcp(addr: SocketAddr) -> bool {
    matches!(
        tokio::time::timeout(TIEMPO_MAXIMO, tokio::net::TcpStream::connect(addr)).await,
        Ok(Ok(_))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn fp(c: char) -> String {
        c.to_string().repeat(64)
    }

    fn guardado(n: u128, c: char, dir: Option<&str>) -> PeerParaAlcance {
        PeerParaAlcance {
            instance_id: id(n),
            fingerprint: fp(c),
            last_address: dir.map(str::to_string),
        }
    }

    fn visto(n: u128, c: char, dir: &str) -> EquipoDescubierto {
        EquipoDescubierto {
            instance_id: id(n),
            fingerprint_declarada: fp(c),
            display_name: "X".into(),
            addresses: vec![dir.into()],
            protocol_version: 1,
            app_version: "0".into(),
            link_mbps: 0,
            busy: false,
        }
    }

    fn estado_de(m: &MonitorAlcance, c: char) -> Option<Alcance> {
        m.estados()
            .into_iter()
            .find(|e| e.fingerprint == fp(c))
            .map(|e| e.estado)
    }

    fn siempre(ok: bool) -> impl Fn(SocketAddr) -> std::future::Ready<bool> + Clone {
        move |_| std::future::ready(ok)
    }

    #[test]
    fn el_contrato_con_la_interfaz_usa_identificadores_en_ingles() {
        let json = serde_json::to_string(&EstadoAlcance {
            fingerprint: "ab".into(),
            estado: Alcance::Inalcanzable,
        })
        .unwrap();
        assert_eq!(json, r#"{"fingerprint":"ab","estado":"unreachable"}"#);
        assert_eq!(
            serde_json::to_string(&Alcance::Comprobando).unwrap(),
            r#""checking""#
        );
        assert_eq!(
            serde_json::to_string(&Alcance::Alcanzable).unwrap(),
            r#""reachable""#
        );
    }

    #[test]
    fn la_espera_crece_con_los_fallos_y_se_acota() {
        assert_eq!(espera_tras_fallos(1), Duration::from_secs(30));
        assert_eq!(espera_tras_fallos(3), Duration::from_secs(60));
        assert_eq!(espera_tras_fallos(4), Duration::from_secs(120));
        assert_eq!(espera_tras_fallos(50), Duration::from_secs(300));
    }

    #[tokio::test]
    async fn al_arrancar_un_equipo_que_no_contesta_sale_inalcanzable_a_la_primera() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', Some("192.168.1.50:7411"))];

        m.ronda(Instant::now(), &g, &[], siempre(false)).await;

        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Inalcanzable));
    }

    #[tokio::test]
    async fn al_arrancar_un_equipo_que_contesta_sale_alcanzable_y_se_anota() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', Some("192.168.1.50:7411"))];

        let av = m.ronda(Instant::now(), &g, &[], siempre(true)).await;

        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Alcanzable));
        assert_eq!(
            av,
            vec![Avistamiento {
                fingerprint: fp('a'),
                direccion: Some("192.168.1.50:7411".into())
            }]
        );
    }

    #[tokio::test]
    async fn un_equipo_alcanzable_necesita_dos_fallos_para_caer() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', Some("192.168.1.50:7411"))];
        let t0 = Instant::now();
        m.ronda(t0, &g, &[], siempre(true)).await;

        let t1 = t0 + INTERVALO;
        m.ronda(t1, &g, &[], siempre(false)).await;
        assert_eq!(
            estado_de(&m, 'a'),
            Some(Alcance::Alcanzable),
            "un fallo no basta"
        );

        let t2 = t1 + CONFIRMACION;
        m.ronda(t2, &g, &[], siempre(false)).await;
        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Inalcanzable));
    }

    #[tokio::test]
    async fn no_se_sondea_antes_de_que_venza_la_espera() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', Some("192.168.1.50:7411"))];
        let t0 = Instant::now();
        m.ronda(t0, &g, &[], siempre(true)).await;

        // Habría cambiado el estado si hubiera sondeado (a fallo, y con dos vueltas).
        m.ronda(t0 + Duration::from_secs(5), &g, &[], siempre(false))
            .await;
        m.ronda(t0 + Duration::from_secs(6), &g, &[], siempre(false))
            .await;

        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Alcanzable));
    }

    #[tokio::test]
    async fn un_equipo_que_mdns_ve_es_alcanzable_sin_sondear_y_actualiza_su_direccion() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', Some("192.168.1.50:7411"))];
        let v = [visto(1, 'a', "192.168.1.77:7411")];

        // Si sondeara, devolvería fallo.
        let av = m.ronda(Instant::now(), &g, &v, siempre(false)).await;

        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Alcanzable));
        assert_eq!(av[0].direccion.as_deref(), Some("192.168.1.77:7411"));
    }

    #[tokio::test]
    async fn un_anuncio_con_otra_huella_no_da_por_visto_al_equipo_guardado() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', Some("192.168.1.50:7411"))];
        let v = [visto(1, 'b', "192.168.1.77:7411")];

        m.ronda(Instant::now(), &g, &v, siempre(false)).await;

        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Inalcanzable));
    }

    #[tokio::test]
    async fn sin_direccion_guardada_espera_la_gracia_y_despues_es_inalcanzable() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', None)];

        m.ronda(m.inicio + Duration::from_secs(1), &g, &[], siempre(true))
            .await;
        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Comprobando));

        m.ronda(m.inicio + Duration::from_secs(7), &g, &[], siempre(true))
            .await;
        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Inalcanzable));
    }

    #[tokio::test]
    async fn un_equipo_olvidado_desaparece_del_monitor() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', Some("192.168.1.50:7411"))];
        m.ronda(Instant::now(), &g, &[], siempre(true)).await;

        m.ronda(Instant::now(), &[], &[], siempre(true)).await;

        assert!(m.estados().is_empty());
    }

    #[tokio::test]
    async fn comprobar_ahora_lo_vuelve_a_comprobando_y_lo_deja_vencido() {
        let m = MonitorAlcance::new();
        let g = [guardado(1, 'a', Some("192.168.1.50:7411"))];
        let t0 = Instant::now();
        m.ronda(t0, &g, &[], siempre(true)).await;

        m.forzar(Some(&fp('a')));
        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Comprobando));

        // Vencido pese a estar dentro del intervalo: se sondea y ahora contesta que no.
        m.ronda(t0 + Duration::from_secs(1), &g, &[], siempre(false))
            .await;
        assert_eq!(estado_de(&m, 'a'), Some(Alcance::Inalcanzable));
    }

    #[tokio::test]
    async fn esperar_reintenta_hasta_que_el_valor_existe_sin_hacer_panic() {
        let mut intentos = 0;
        let v = esperar(Duration::from_millis(1), || {
            intentos += 1;
            (intentos >= 4).then_some(intentos)
        })
        .await;
        assert_eq!(v, 4);
    }

    #[tokio::test]
    async fn sondear_tcp_distingue_una_escucha_real_de_un_puerto_cerrado() {
        let escucha = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let abierta = escucha.local_addr().unwrap();
        assert!(sondear_tcp(abierta).await);

        drop(escucha);
        assert!(!sondear_tcp(abierta).await);
    }
}
