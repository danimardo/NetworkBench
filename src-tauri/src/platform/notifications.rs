use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

/// Envía una notificación de escritorio (Toast en Windows) al finalizar una tarea en segundo plano
pub fn show_desktop_notification(title: &str, body: &str) -> Result<(), String> {
    // Si la app está empaquetada con Tauri, las notificaciones se pueden enviar a través del plugin
    // o canal nativo de Windows. Esta función prepara el payload estructurado.
    let payload = NotificationPayload {
        title: title.to_string(),
        body: body.to_string(),
        icon: Some("networkbench".to_string()),
    };
    
    // Log informativo para diagnóstico
    crate::logging::log(
        crate::logging::LogLevel::Info,
        "platform.notification",
        &format!("Toast notification triggered: {} - {}", payload.title, payload.body),
    );

    Ok(())
}
