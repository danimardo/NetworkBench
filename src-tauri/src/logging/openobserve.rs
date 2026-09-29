//! Envío opt-in de diagnóstico a un servidor OpenObserve propio (constitución, enmienda
//! 0.8.0 al principio IV: un reenvío que la propia persona activa y configura, hacia un
//! servidor que ella misma opera, no es la telemetría automática hacia terceros que el
//! principio prohíbe). Apagado por defecto.
//!
//! Cubre las dos fuentes de registro que ya existen (`logging::mod`, hallazgo T180 del
//! 2026-09-26/28): el `LogEvent` propio en JSON (`Logger::log`) y las líneas de `tracing`
//! (descubrimiento, emparejamiento, protocolo, motor NTTTCP — donde vive el detalle de
//! diagnóstico avanzado que la enmienda 0.8.0 al principio XIII permite solo con
//! Debug/Trace activado explícitamente). Como las dos vías ya pasan por el mismo filtro de
//! nivel antes de llegar aquí, esta cola no necesita comprobar el nivel por su cuenta.
//!
//! Cola no bloqueante (mejor esfuerzo): si está apagado, mal configurado o el envío falla,
//! el evento se descarta en silencio. El registro local en fichero, que ya escribió antes
//! de llegar aquí, nunca depende de que esto funcione. Deliberadamente no se registra el
//! fallo de sus propios envíos (evitaría recursión: un fallo de red al mandar a OpenObserve
//! generaría un log que también se intentaría mandar a OpenObserve). El botón "Probar
//! conexión" (`probar_conexion`) es la vía para diagnosticar la configuración.

use serde::{Deserialize, Serialize};
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

const CAPACIDAD_COLA: usize = 512;
const TAMANO_LOTE: usize = 20;
const ESPERA_LOTE: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenObserveConfig {
    pub enabled: bool,
    pub url: String,
    pub org: String,
    pub stream: String,
    /// El valor completo de la cabecera `Authorization` tal como lo emite OpenObserve
    /// (`Basic <base64>`), no una contraseña que haya que combinar con nada: se usa tal
    /// cual, sin anteponerle "Basic " ni volver a codificarlo.
    pub token: String,
}

impl OpenObserveConfig {
    pub fn from_preferences(p: &crate::settings::Preferences) -> Self {
        Self {
            enabled: p.open_observe_enabled,
            url: p.open_observe_url.clone(),
            org: p.open_observe_org.clone(),
            stream: p.open_observe_stream.clone(),
            token: p.open_observe_token.clone(),
        }
    }

    /// Los cuatro campos necesarios para poder enviar algo, todos no vacíos tras recortar
    /// espacios. `enabled` se comprueba aparte: una config completa pero apagada no envía.
    pub fn esta_completa(&self) -> bool {
        !self.url.trim().is_empty()
            && !self.org.trim().is_empty()
            && !self.stream.trim().is_empty()
            && !self.token.trim().is_empty()
    }

    fn activa(&self) -> bool {
        self.enabled && self.esta_completa()
    }

    /// `{url_sin_barra_final}/api/{org}/{stream}/_json` — API nativa de ingesta JSON de
    /// OpenObserve (sin OTLP), que acepta un lote como array de objetos.
    pub fn endpoint(&self) -> String {
        let base = self.url.trim_end_matches('/');
        format!("{base}/api/{}/{}/_json", self.org, self.stream)
    }
}

static CONFIG: OnceLock<RwLock<OpenObserveConfig>> = OnceLock::new();
static COLA: OnceLock<tokio::sync::mpsc::Sender<serde_json::Value>> = OnceLock::new();

fn config_lock() -> &'static RwLock<OpenObserveConfig> {
    CONFIG.get_or_init(|| RwLock::new(OpenObserveConfig::default()))
}

/// Sustituye la configuración activa. Se llama al arrancar (con lo guardado en
/// `settings.json`) y cada vez que `settings_update` guarda un cambio — mismo patrón que
/// `Logger::set_level`/`set_tracing_level` para el nivel de registro.
pub fn actualizar_config(cfg: OpenObserveConfig) {
    if let Ok(mut actual) = config_lock().write() {
        *actual = cfg;
    }
}

pub fn config_actual() -> OpenObserveConfig {
    config_lock().read().map(|c| c.clone()).unwrap_or_default()
}

/// Falso siempre que no haya nada que hacer: evita construir el `serde_json::Value` del
/// evento en el caso común (apagado) antes de descubrir que se iba a descartar igualmente.
pub fn activo() -> bool {
    config_actual().activa()
}

/// `instance_id` (UUID estable) y nombre de equipo, fijados una sola vez al arrancar
/// (`app::init`, desde `InstanceIdentity` — la misma identidad que ya usan los peers para
/// reconocerse al emparejar). Sin esto, con varias máquinas escribiendo en el mismo stream
/// de OpenObserve sus eventos serían indistinguibles entre sí (hallazgo real del
/// propietario: pensaba pedirle a un agente de IA que consultara los registros de una
/// máquina en concreto, y no había ningún campo que se lo permitiera).
static IDENTIDAD: OnceLock<(String, String)> = OnceLock::new();

pub fn establecer_identidad(instance_id: String, hostname: String) {
    let _ = IDENTIDAD.set((instance_id, hostname));
}

fn identidad_actual() -> (String, String) {
    IDENTIDAD.get().cloned().unwrap_or_default()
}

/// Añade `instanceId` y `hostname` a un evento que ya es un objeto JSON. Si por lo que sea
/// no lo es (no debería pasar: todos los llamadores construyen un objeto), lo deja igual en
/// vez de entrar en pánico.
pub fn con_identidad(mut valor: serde_json::Value) -> serde_json::Value {
    if let Some(objeto) = valor.as_object_mut() {
        let (instance_id, hostname) = identidad_actual();
        objeto.insert(
            "instanceId".to_string(),
            serde_json::Value::String(instance_id),
        );
        objeto.insert("hostname".to_string(), serde_json::Value::String(hostname));
        // El gateway y el MCP de consulta muestran `service`; sin él todo sale como «unknown».
        objeto
            .entry("service")
            .or_insert_with(|| serde_json::Value::String(SERVICIO.to_string()));
    }
    valor
}

/// Nombre de servicio con el que los eventos se identifican en el gateway de consulta.
const SERVICIO: &str = "netbench";

/// Módulos del propio transporte HTTP/TLS que envía los lotes. Sus eventos DEBUG
/// («reuse idle connection», «pooling idle connection»…) se producen por cada envío;
/// reenviarlos a OpenObserve generaría otro envío y así sin fin.
fn es_ruido_de_transporte(target: &str) -> bool {
    const MODULOS: [&str; 7] = [
        "hyper",
        "hyper_util",
        "reqwest",
        "h2",
        "rustls",
        "tokio_rustls",
        "want",
    ];
    de_modulo(target, &MODULOS)
}

/// Librería mDNS: a nivel DEBUG registra cada paquete recibido por cada interfaz (una docena
/// por segundo) y ahoga el resto. Se descarta todo salvo avisos y errores.
fn es_ruido_de_mdns(target: &str, nivel: &tracing::Level) -> bool {
    de_modulo(target, &["mdns_sd"]) && *nivel > tracing::Level::WARN
}

fn de_modulo(target: &str, modulos: &[&str]) -> bool {
    modulos.iter().any(|m| {
        target == *m || (target.starts_with(m) && target[m.len()..].starts_with("::"))
    })
}

/// Encola un evento para enviarlo en el siguiente lote. Nunca bloquea: si la cola está
/// llena (el servidor no responde o la red va más lenta que el ritmo de logging) o el envío
/// no se ha arrancado (`iniciar` no se ha llamado todavía), el evento se descarta.
pub fn encolar(evento: serde_json::Value) {
    if !activo() {
        return;
    }
    if let Some(tx) = COLA.get() {
        let _ = tx.try_send(evento);
    }
}

/// Arranca la tarea de fondo que agrupa en lotes (hasta `TAMANO_LOTE` eventos o
/// `ESPERA_LOTE` de espera desde el primero) y hace un solo POST por lote. Idempotente:
/// llamarla más de una vez no crea una segunda tarea.
pub fn iniciar() {
    if COLA.get().is_some() {
        return;
    }
    let (tx, rx) = tokio::sync::mpsc::channel::<serde_json::Value>(CAPACIDAD_COLA);
    if COLA.set(tx).is_err() {
        return;
    }
    tauri::async_runtime::spawn(recibir_y_enviar(rx));
}

async fn recibir_y_enviar(mut rx: tokio::sync::mpsc::Receiver<serde_json::Value>) {
    let cliente = reqwest::Client::new();
    let mut lote: Vec<serde_json::Value> = Vec::with_capacity(TAMANO_LOTE);

    while let Some(primero) = rx.recv().await {
        lote.push(primero);

        let plazo = tokio::time::sleep(ESPERA_LOTE);
        tokio::pin!(plazo);
        while lote.len() < TAMANO_LOTE {
            tokio::select! {
                _ = &mut plazo => break,
                recibido = rx.recv() => {
                    match recibido {
                        Some(v) => lote.push(v),
                        None => break,
                    }
                }
            }
        }

        let cfg = config_actual();
        if cfg.activa() {
            let _ = enviar_lote(&cliente, &cfg, &lote).await;
        }
        lote.clear();
    }
}

async fn enviar_lote(
    cliente: &reqwest::Client,
    cfg: &OpenObserveConfig,
    lote: &[serde_json::Value],
) -> Result<(), reqwest::Error> {
    cliente
        .post(cfg.endpoint())
        .header(reqwest::header::AUTHORIZATION, &cfg.token)
        .json(lote)
        .send()
        .await?;
    Ok(())
}

/// Un solo evento de prueba, síncrono desde el punto de vista de quien lo llama (espera la
/// respuesta): para el botón "Probar conexión" de Ajustes, que necesita saber exactamente
/// qué falló, a diferencia del envío normal que es mejor esfuerzo y no informa de errores.
pub async fn probar_conexion(cfg: &OpenObserveConfig) -> Result<(), String> {
    if !cfg.esta_completa() {
        return Err("Faltan campos: url, organización, stream o token".to_string());
    }

    let cliente = reqwest::Client::new();
    let evento = con_identidad(serde_json::json!({
        "_timestamp": crate::logging::format_rfc3339_utc(std::time::SystemTime::now()),
        "level": "info",
        "module": "settings.openobserve",
        "message": "Prueba de conexión desde NetworkBench",
    }));

    let respuesta = cliente
        .post(cfg.endpoint())
        .header(reqwest::header::AUTHORIZATION, &cfg.token)
        .json(&[evento])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let estado = respuesta.status();
    if estado.is_success() {
        return Ok(());
    }
    let cuerpo = respuesta.text().await.unwrap_or_default();
    let recortado: String = cuerpo.chars().take(300).collect();
    Err(format!("HTTP {estado}: {recortado}"))
}

/// Capa de `tracing_subscriber` que reenvía cada evento a la misma cola que `Logger::log`.
/// Se añade a la composición `Registry::default().with(filtro).with(capa_fichero)` ya
/// existente en `logging::init_tracing`: el filtro reconfigurable de ahí decide qué llega
/// aquí, así que esta capa no repite esa comprobación de nivel — solo si hay algo que enviar.
pub struct CapaOpenObserve;

impl<S> tracing_subscriber::Layer<S> for CapaOpenObserve
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if !activo() {
            return;
        }
        let metadata = event.metadata();
        let mut visitor = VisitadorDeCampos::default();
        event.record(&mut visitor);

        // Los eventos de librerías que usan el crate `log` (p. ej. `mdns-sd`) llegan puenteados
        // con `target = "log"`; el módulo real viaja en el campo `log.target`.
        let target = visitor
            .otros
            .get("log.target")
            .map(|t| t.trim_matches('"').to_string())
            .unwrap_or_else(|| metadata.target().to_string());
        if es_ruido_de_transporte(&target) || es_ruido_de_mdns(&target, metadata.level()) {
            return;
        }

        let valor = serde_json::json!({
            "_timestamp": crate::logging::format_rfc3339_utc(std::time::SystemTime::now()),
            "level": metadata.level().as_str().to_lowercase(),
            "target": target,
            "message": visitor.mensaje,
            "campos": visitor.otros,
        });
        encolar(con_identidad(valor));
    }
}

#[derive(Default)]
struct VisitadorDeCampos {
    mensaje: String,
    otros: std::collections::HashMap<String, String>,
}

impl tracing::field::Visit for VisitadorDeCampos {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let texto = format!("{value:?}");
        if field.name() == "message" {
            self.mensaje = texto;
        } else {
            self.otros.insert(field.name().to_string(), texto);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn config_de_prueba(url: String) -> OpenObserveConfig {
        OpenObserveConfig {
            enabled: true,
            url,
            org: "miorg".to_string(),
            stream: "netbench".to_string(),
            token: "Basic dXN1YXJpbzpjbGF2ZQ==".to_string(),
        }
    }

    #[test]
    fn el_endpoint_sigue_el_formato_de_ingesta_json_nativa() {
        let cfg = config_de_prueba("https://mi-servidor.example".to_string());
        assert_eq!(
            cfg.endpoint(),
            "https://mi-servidor.example/api/miorg/netbench/_json"
        );
    }

    #[test]
    fn una_barra_final_en_la_url_no_se_duplica() {
        let cfg = config_de_prueba("https://mi-servidor.example/".to_string());
        assert_eq!(
            cfg.endpoint(),
            "https://mi-servidor.example/api/miorg/netbench/_json"
        );
    }

    #[test]
    fn cada_campo_vacio_por_separado_deja_la_config_incompleta() {
        let base = config_de_prueba("https://x.example".to_string());

        let mut sin_url = base.clone();
        sin_url.url = String::new();
        assert!(!sin_url.esta_completa());

        let mut sin_org = base.clone();
        sin_org.org = "   ".to_string();
        assert!(!sin_org.esta_completa());

        let mut sin_stream = base.clone();
        sin_stream.stream = String::new();
        assert!(!sin_stream.esta_completa());

        let mut sin_token = base.clone();
        sin_token.token = String::new();
        assert!(!sin_token.esta_completa());

        assert!(base.esta_completa());
    }

    #[tokio::test]
    async fn probar_conexion_rechaza_config_incompleta_sin_tocar_la_red() {
        let mut cfg = config_de_prueba("https://no-deberia-usarse.invalid".to_string());
        cfg.stream = String::new();
        let resultado = probar_conexion(&cfg).await;
        assert!(resultado.is_err());
        assert!(resultado.unwrap_err().contains("Faltan campos"));
    }

    /// Confirma método, ruta y cabecera `Authorization` de la petición HTTP real contra un
    /// `TcpListener` local — mismo patrón ya usado en otros tests del repo para servidores
    /// locales, sin crate nuevo de mocking.
    #[tokio::test]
    async fn probar_conexion_envia_post_con_la_cabecera_authorization_exacta() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local");
        let puerto = listener.local_addr().expect("puerto local").port();

        let hilo = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("aceptar conexión");
            let mut buf = [0u8; 4096];
            let leido = socket.read(&mut buf).unwrap_or(0);
            let peticion = String::from_utf8_lossy(&buf[..leido]).to_string();
            let _ = socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
            peticion
        });

        let cfg = config_de_prueba(format!("http://127.0.0.1:{puerto}"));
        let resultado = probar_conexion(&cfg).await;
        assert!(resultado.is_ok(), "esperaba éxito: {resultado:?}");

        let peticion = hilo
            .join()
            .expect("el hilo del servidor no debe entrar en pánico");
        assert!(peticion.starts_with("POST /api/miorg/netbench/_json"));
        assert!(peticion.contains("authorization: Basic dXN1YXJpbzpjbGF2ZQ==\r\n"));
    }

    #[tokio::test]
    async fn probar_conexion_devuelve_el_estado_y_el_cuerpo_cuando_el_servidor_rechaza() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local");
        let puerto = listener.local_addr().expect("puerto local").port();

        let hilo = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("aceptar conexión");
            let mut buf = [0u8; 4096];
            let _ = socket.read(&mut buf);
            let cuerpo = b"token invalido";
            let respuesta = format!(
                "HTTP/1.1 401 Unauthorized\r\nContent-Length: {}\r\n\r\n{}",
                cuerpo.len(),
                String::from_utf8_lossy(cuerpo)
            );
            let _ = socket.write_all(respuesta.as_bytes());
        });

        let cfg = config_de_prueba(format!("http://127.0.0.1:{puerto}"));
        let resultado = probar_conexion(&cfg).await;
        hilo.join()
            .expect("el hilo del servidor no debe entrar en pánico");

        let err = resultado.expect_err("un 401 debe llegar como error");
        assert!(err.contains("401"));
        assert!(err.contains("token invalido"));
    }

    #[test]
    fn sin_configurar_nunca_esta_activa() {
        assert!(!OpenObserveConfig::default().activa());
    }

    #[test]
    fn con_identidad_anade_instance_id_y_hostname_sin_tocar_el_resto() {
        establecer_identidad(
            "11111111-1111-1111-1111-111111111111".to_string(),
            "MAQUINA-A".to_string(),
        );
        let valor = con_identidad(serde_json::json!({"message": "hola"}));
        assert_eq!(valor["instanceId"], "11111111-1111-1111-1111-111111111111");
        assert_eq!(valor["hostname"], "MAQUINA-A");
        assert_eq!(valor["message"], "hola");
        assert_eq!(valor["service"], "netbench");
    }

    #[test]
    fn el_ruido_del_transporte_http_no_se_reenvia() {
        assert!(es_ruido_de_transporte("hyper_util::client::legacy::pool"));
        assert!(es_ruido_de_transporte("reqwest::connect"));
        assert!(es_ruido_de_transporte("h2"));
        assert!(!es_ruido_de_transporte("networkbench_lib::control::server"));
        assert!(!es_ruido_de_transporte("hyperion::algo"));
    }

    #[test]
    fn de_mdns_sd_solo_se_reenvian_avisos_y_errores() {
        use tracing::Level;
        assert!(es_ruido_de_mdns("mdns_sd::service_daemon", &Level::DEBUG));
        assert!(es_ruido_de_mdns("mdns_sd::service_daemon", &Level::INFO));
        assert!(!es_ruido_de_mdns("mdns_sd::service_daemon", &Level::WARN));
        assert!(!es_ruido_de_mdns("mdns_sd::service_daemon", &Level::ERROR));
        assert!(!es_ruido_de_mdns("networkbench_lib::discovery::mdns", &Level::DEBUG));
    }
}
