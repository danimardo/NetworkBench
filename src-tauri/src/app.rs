use crate::control::engine_port::MotorNtttcp;
use crate::control::orquestador::Orquestador;
use crate::control::server::ControlServer;
use crate::control::service::{ContextoSesion, SessionService};
use crate::engine::ntttcp::engine_sha256;
use crate::history::database::Database;
use crate::identity::InstanceIdentity;
use crate::ipc::events::EmisorDeEventos;
use crate::ipc::response::{IpcResult, OneTimeTokenStore};
use crate::ipc::snapshot::{AppSnapshot, SnapshotManager};
use crate::logging::{LogLevel, init_logger};
use crate::sampling::vivo::{ContadoresWindows, Muestreo};
use crate::settings::SettingsStore;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub snapshot: Arc<SnapshotManager>,
    pub database: Arc<Database>,
    pub settings: Arc<SettingsStore>,
    pub tokens: Arc<OneTimeTokenStore>,
    pub identity: Arc<InstanceIdentity>,
    pub session_service: Arc<SessionService>,
    pub delete_tokens: Arc<crate::history::delete::DeleteTokenStore>,
    /// Orquestador de medida, con el motor real detrás de su puerto. Que exista no
    /// significa que el motor esté: `engine/ntttcp.exe` puede faltar, y entonces cada
    /// medición devuelve `MotorError::NoDisponible` en vez de un resultado inventado.
    pub orquestador: Arc<Orquestador>,
    /// Emparejamientos esperando la decisión del usuario, con su canal TLS abierto.
    pub pairings: Arc<crate::ipc::pairing::PairingStore>,
    /// Solicitudes de sesión entrante esperando consentimiento humano (T177, FR-016).
    pub solicitudes_entrantes: Arc<crate::control::consent::SolicitudesEntrantes>,
    /// Emparejamientos entrantes esperando decisión humana (T182, FR-012).
    pub emparejamientos_entrantes: Arc<crate::control::consent::EmparejamientosEntrantes>,
    /// Anuncio y descubrimiento mDNS (T151). `None` si el ajuste está apagado o si mDNS no
    /// pudo abrirse: sin descubrimiento se sigue pudiendo conectar a mano (FR-010).
    pub descubrimiento: std::sync::Mutex<Option<crate::discovery::Descubrimiento>>,
}

impl AppState {
    /// Enciende o apaga el anuncio y la navegación mDNS según el ajuste «Descubrimiento
    /// automático» (`Historias.md` §7.1: apagado, la instancia ni publica ni navega).
    /// Idempotente. Apagarlo suelta `Descubrimiento`, cuyo `Drop` retira el anuncio.
    pub fn aplicar_descubrimiento(&self, activo: bool) {
        let Ok(mut guardado) = self.descubrimiento.lock() else {
            return;
        };
        match (activo, guardado.is_some()) {
            (true, false) => {
                match crate::discovery::Descubrimiento::iniciar(
                    &self.identity,
                    crate::discovery::CONTROL_PORT_DEFAULT,
                    env!("CARGO_PKG_VERSION"),
                    crate::control::server::VERSION_PROTOCOLO,
                ) {
                    Ok(d) => *guardado = Some(d),
                    Err(e) => tracing::warn!("Descubrimiento mDNS no disponible: {e}"),
                }
            }
            (false, true) => *guardado = None,
            _ => {}
        }
    }

    /// Lo que una sesión necesita del resto de la aplicación. `emisor` es por donde salen
    /// las muestras en vivo hacia la interfaz; sin él, la sesión no las produce.
    pub fn contexto_de_sesion(&self, emisor: Option<Arc<dyn EmisorDeEventos>>) -> ContextoSesion {
        ContextoSesion {
            orquestador: Arc::clone(&self.orquestador),
            identity: Arc::clone(&self.identity),
            database: Arc::clone(&self.database),
            muestreo: emisor.clone().map(|emisor| Muestreo {
                emisor,
                contadores: Arc::new(ContadoresWindows),
            }),
            consentimiento: Arc::clone(&self.solicitudes_entrantes),
            emisor_eventos: emisor,
            emparejamientos: Arc::clone(&self.emparejamientos_entrantes),
        }
    }
}

use crate::platform::window::{MonitorBounds, WindowGeometry, normalize_or_fallback_geometry};

#[tauri::command]
pub fn app_get_snapshot(state: tauri::State<AppState>) -> IpcResult<AppSnapshot> {
    IpcResult::ok(state.snapshot.get_snapshot())
}

#[tauri::command]
pub fn window_get_geometry(state: tauri::State<AppState>) -> IpcResult<Option<WindowGeometry>> {
    IpcResult::ok(state.settings.get().window_geometry)
}

#[tauri::command]
pub fn window_save_geometry(
    state: tauri::State<AppState>,
    geometry: WindowGeometry,
) -> IpcResult<()> {
    match state.settings.update(|p| {
        p.window_geometry = Some(geometry);
    }) {
        Ok(_) => IpcResult::ok(()),
        Err(e) => IpcResult::err(e),
    }
}

#[tauri::command]
pub fn window_restore_and_show(
    window: tauri::WebviewWindow,
    state: tauri::State<AppState>,
) -> IpcResult<WindowGeometry> {
    let saved = state.settings.get().window_geometry;

    let available = window.available_monitors().unwrap_or_default();
    let monitors: Vec<MonitorBounds> = available
        .into_iter()
        .map(|m| {
            let pos = m.position();
            let size = m.size();
            MonitorBounds {
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
            }
        })
        .collect();

    let primary = window.primary_monitor().ok().flatten().map(|m| {
        let pos = m.position();
        let size = m.size();
        MonitorBounds {
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
        }
    });

    let target_geom = normalize_or_fallback_geometry(saved, primary, &monitors);

    let _ = window.set_size(tauri::LogicalSize::new(
        target_geom.width,
        target_geom.height,
    ));
    let _ = window.set_position(tauri::LogicalPosition::new(target_geom.x, target_geom.y));
    if target_geom.is_maximized {
        let _ = window.maximize();
    }
    let _ = window.show();
    let _ = window.set_focus();

    IpcResult::ok(target_geom)
}

/// Inicialización y orquestación del ciclo de vida de la aplicación sin autoridad duplicada
pub fn init() -> Result<AppState, Box<dyn std::error::Error>> {
    let app_dir = dirs_or_fallback();
    let log_dir = app_dir.join("logs");
    let db_path = app_dir.join("history.db");
    let settings_path = app_dir.join("settings.json");
    let identity_dir = app_dir.join("identity");

    let _logger = init_logger(log_dir, LogLevel::Warn);
    let settings = Arc::new(SettingsStore::new(settings_path));
    let database = Arc::new(Database::open(db_path)?);
    let tokens = Arc::new(OneTimeTokenStore::new());

    let identity = Arc::new(InstanceIdentity::get_or_create(
        &identity_dir,
        "NetworkBench",
    )?);
    let session_service = Arc::new(SessionService::new());
    let delete_tokens = Arc::new(crate::history::delete::DeleteTokenStore::new());

    // Ruta del motor junto al ejecutable, como quedará tras la instalación NSIS.
    let engine_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("ntttcp.exe")))
        .unwrap_or_else(|| PathBuf::from("engine/ntttcp.exe"));
    let orquestador = Arc::new(Orquestador::new(Arc::new(MotorNtttcp::new(
        engine_path,
        engine_sha256().to_string(),
        app_dir.join("tmp"),
    ))));

    let initial_prefs = settings.get();
    let snapshot = Arc::new(SnapshotManager::new(AppSnapshot {
        revision: 1,
        app_version: "0.1.0".to_string(),
        locale: initial_prefs.locale,
        theme: match initial_prefs.theme {
            crate::settings::ThemeMode::System => "system".to_string(),
            crate::settings::ThemeMode::Light => "light".to_string(),
            crate::settings::ThemeMode::Dark => "dark".to_string(),
        },
        instance_id: identity.instance_id.to_string(),
        instance_name: identity.display_name.clone(),
        is_session_active: false,
        active_session_id: None,
        peers_count: 0,
    }));

    Ok(AppState {
        snapshot,
        database,
        settings,
        tokens,
        identity,
        session_service,
        delete_tokens,
        orquestador,
        pairings: Arc::new(crate::ipc::pairing::PairingStore::new()),
        solicitudes_entrantes: Arc::new(crate::control::consent::SolicitudesEntrantes::new()),
        emparejamientos_entrantes: Arc::new(
            crate::control::consent::EmparejamientosEntrantes::new(),
        ),
        descubrimiento: std::sync::Mutex::new(None),
    })
}

/// Arranca el servidor del canal de control y devuelve el puerto real.
///
/// Cada conexión completa el TLS mutuo y el saludo, y se entrega a
/// `control::despachador`, que atiende solicitudes de prueba y emparejamientos entrantes,
/// ambos a la espera de una decisión humana.
///
/// Un fallo de una conexión no detiene el bucle: se registra y se sigue aceptando.
pub async fn start_control_server(
    ctx: crate::control::service::ContextoSesion,
    servicio: Arc<SessionService>,
    puerto: u16,
) -> Result<u16, Box<dyn std::error::Error>> {
    let identity = Arc::clone(&ctx.identity);
    use std::net::{IpAddr, Ipv6Addr, SocketAddr};

    // `::` acepta también IPv4 mapeada, de modo que un solo socket cubre ambas
    // familias sin abrir dos puertos (FR-010).
    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), puerto);
    let servidor = match ControlServer::bind(addr, Arc::clone(&identity)).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                "No se pudo ligar el canal de control a [::]:{puerto} ({e}); se reintenta en IPv4"
            );
            ControlServer::bind(
                SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), puerto),
                identity,
            )
            .await?
        }
    };

    let real = servidor.local_addr()?.port();
    tracing::info!("Canal de control escuchando en el puerto {real}");

    // Conexiones entrantes simultáneas acotadas a 8 (contrato del protocolo, §Límites).
    let cupo = Arc::new(tokio::sync::Semaphore::new(8));

    tauri::async_runtime::spawn(async move {
        loop {
            match servidor.accept_one().await {
                Ok(saludo) => {
                    // La huella es lo único con valor probatorio; el nombre que el par
                    // declara no se registra aquí sin sanear ni se convierte en confianza.
                    tracing::info!(
                        "Saludo completado con {} (huella {}…)",
                        saludo.remote_addr,
                        &saludo.fingerprint[..8]
                    );

                    let Ok(permiso) = Arc::clone(&cupo).try_acquire_owned() else {
                        tracing::warn!("Demasiadas conexiones simultáneas; se descarta una");
                        continue;
                    };
                    let servicio = Arc::clone(&servicio);
                    let ctx = ctx.clone();
                    tauri::async_runtime::spawn(async move {
                        crate::control::despachador::despachar(servicio, ctx, saludo).await;
                        drop(permiso);
                    });
                }
                Err(e) => {
                    tracing::warn!("Conexión entrante descartada: {e}");
                }
            }
        }
    });

    Ok(real)
}

pub fn dirs_or_fallback() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("NetworkBench")
    } else {
        std::env::temp_dir().join("NetworkBench")
    }
}
