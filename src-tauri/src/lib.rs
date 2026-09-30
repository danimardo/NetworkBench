pub mod app;
pub mod config;
pub mod control;
pub mod diagnostic;
pub mod discovery;
pub mod engine;
pub mod errors;
pub mod export;
pub mod firewall;
pub mod history;
pub mod identity;
pub mod ipc;
pub mod logging;
pub mod model;
pub mod netinfo;
pub mod pairing;
pub mod platform;
pub mod sampling;
pub mod settings;
pub mod updater;

pub fn run() {
    let app_state = app::init().expect("fallo al inicializar el estado central de la aplicación");

    tauri::Builder::default()
        // Toda vía de cierre de la ventana pasa por aquí (§5.2): se cancela el cierre nativo
        // y decide `cierre::solicitar_cierre`. Sin esto no habría un único comportamiento.
        .on_window_event(|ventana, evento| {
            if ventana.label() == "main"
                && let tauri::WindowEvent::CloseRequested { api, .. } = evento
            {
                api.prevent_close();
                let app = tauri::Manager::app_handle(ventana).clone();
                tauri::async_runtime::spawn(async move {
                    platform::cierre::solicitar_cierre(&app).await;
                });
            }
        })
        .setup(move |app| {
            // El canal de control se levanta durante el arranque, antes de que el usuario
            // abra nada: una instancia debe poder recibir un saludo aunque nadie haya
            // tocado la ventana (FR-009). Se hace aquí y no antes del `Builder` porque las
            // muestras en vivo de las sesiones entrantes necesitan el `AppHandle`.
            let ctx = app_state.contexto_de_sesion(Some(std::sync::Arc::new(
                tauri::Manager::app_handle(app).clone(),
            )));
            let servicio = std::sync::Arc::clone(&app_state.session_service);
            let control = std::sync::Arc::clone(&app_state.control);
            let puerto = app_state.puerto_de_control();
            tauri::async_runtime::spawn(async move {
                // El error pasa a texto antes de esperar el candado: un `Box<dyn Error>` no es
                // `Send` y no puede cruzar un `.await`.
                let arrancado = app::start_control_server(ctx, servicio, puerto)
                    .await
                    .map_err(|e| e.to_string());
                match arrancado {
                    Ok(escucha) => *control.lock().await = Some(escucha),
                    Err(e) => tracing::error!("El canal de control no pudo arrancar: {e}"),
                }
            });
            // El snapshot que lee la interfaz se mantiene al día y se emite cuando cambia.
            tauri::async_runtime::spawn(
                app_state
                    .proyector()
                    .vigilar(std::sync::Arc::new(tauri::Manager::app_handle(app).clone())),
            );
            // Con el ajuste encendido (por defecto), la instancia se anuncia y busca a las demás.
            app_state.aplicar_descubrimiento(app_state.settings.get().mdns_enabled);
            // Primera comprobación de los equipos guardados ya al arrancar, y luego periódica.
            tauri::async_runtime::spawn(discovery::alcance::vigilar(
                tauri::Manager::app_handle(app).clone(),
            ));
            // El envío opt-in a OpenObserve ya tiene su config (`app::init` la cargó); aquí
            // solo arranca la tarea de fondo que consume la cola, que necesita el runtime
            // async de Tauri en marcha.
            logging::openobserve::iniciar();
            // Icono de la bandeja (§5.2). Si no se pudiera crear, «minimizar» no oculta la
            // ventana sino que la minimiza a la barra de tareas, para no dejarla sin salida.
            match platform::tray::crear(app.handle(), &app_state.settings.get().locale) {
                Ok(bandeja) => {
                    tauri::Manager::manage(app, bandeja);
                }
                Err(e) => tracing::warn!("No se pudo crear el icono de la bandeja: {e}"),
            }
            tauri::Manager::manage(app, app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::app_get_snapshot,
            app::window_get_geometry,
            app::window_save_geometry,
            app::window_restore_and_show,
            ipc::peers::peers_list,
            ipc::peers::peers_discovered_list,
            ipc::peers::peers_rescan,
            ipc::peers::peers_manual_connect,
            ipc::peers::peers_set_trust,
            ipc::peers::peers_set_favorite,
            ipc::peers::peers_forget,
            ipc::peers::peers_reachability_list,
            ipc::peers::peers_check_now,
            ipc::pairing::peers_pairing_start,
            ipc::pairing::peers_pairing_confirm,
            ipc::session::session_start,
            ipc::session::session_cancel,
            ipc::session::session_get_state,
            ipc::consent::session_incoming_list,
            ipc::consent::session_incoming_respond,
            ipc::consent::peers_pairing_incoming_list,
            ipc::consent::peers_pairing_incoming_respond,
            ipc::diagnostics::diagnostics_get_report,
            ipc::diagnostics::diagnostics_log_frontend_event,
            ipc::firewall::firewall_inspect,
            ipc::firewall::firewall_apply,
            ipc::firewall::firewall_rules_status,
            ipc::firewall::firewall_rules_create,
            ipc::firewall::firewall_rules_remove,
            ipc::firewall::firewall_open_network_settings,
            ipc::history::history_list,
            ipc::history::history_get,
            ipc::history::history_delete_preview,
            ipc::history::history_delete_confirm,
            ipc::history::history_get_trend,
            ipc::history::history_compare_cohort,
            ipc::history::history_repeat_plan,
            ipc::export::export_preview,
            ipc::export::export_execute,
            ipc::settings::settings_get,
            ipc::settings::settings_update,
            ipc::settings::settings_autostart_get,
            ipc::settings::settings_autostart_set,
            ipc::settings::settings_data_info,
            ipc::settings::settings_data_purge,
            ipc::settings::settings_about_info,
            ipc::settings::settings_diagnostic_paths,
            ipc::settings::settings_open_log_folder,
            ipc::settings::settings_openobserve_test,
            ipc::cierre::app_close_apply,
            ipc::cierre::app_close_confirmed,
            ipc::updater::updater_check,
            ipc::updater::updater_evaluate,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
