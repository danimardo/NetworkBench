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
        .setup(move |app| {
            // El canal de control se levanta durante el arranque, antes de que el usuario
            // abra nada: una instancia debe poder recibir un saludo aunque nadie haya
            // tocado la ventana (FR-009). Se hace aquí y no antes del `Builder` porque las
            // muestras en vivo de las sesiones entrantes necesitan el `AppHandle`.
            let ctx = app_state.contexto_de_sesion(Some(std::sync::Arc::new(
                tauri::Manager::app_handle(app).clone(),
            )));
            let servicio = std::sync::Arc::clone(&app_state.session_service);
            tauri::async_runtime::spawn(async move {
                if let Err(e) =
                    app::start_control_server(ctx, servicio, discovery::CONTROL_PORT_DEFAULT).await
                {
                    tracing::error!("El canal de control no pudo arrancar: {e}");
                }
            });
            // Con el ajuste encendido (por defecto), la instancia se anuncia y busca a las demás.
            app_state.aplicar_descubrimiento(app_state.settings.get().mdns_enabled);
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
            ipc::peers::peers_manual_connect,
            ipc::peers::peers_set_trust,
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
            ipc::firewall::firewall_inspect,
            ipc::firewall::firewall_apply,
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
            ipc::settings::app_close_evaluate,
            ipc::updater::updater_check,
            ipc::updater::updater_evaluate,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
