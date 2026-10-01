//! Comandos IPC de emparejamiento (FR-012, FR-013, FR-014).
//!
//! El emparejamiento son dos pasos con una persona en medio: primero se enseña el
//! código, después se recoge su decisión. Entre los dos, la conexión TLS **se mantiene
//! abierta**: el código se derivó de la huella de esa sesión concreta, así que
//! reconectar convertiría la comparación humana en un gesto sin valor.
//!
//! `start` no concede nada. La confianza se guarda solo cuando `confirm` recibe un sí
//! explícito y el otro extremo responde lo mismo.

use super::response::IpcResult;
use crate::app::AppState;
use crate::control::pairing_flow::{
    EmparejamientoEnCurso, completar_emparejamiento, enviar_solicitud,
};
use crate::discovery::conectar_y_saludar;
use crate::errors::{AppError, ErrorCode, ErrorSeverity};
use crate::history::guardar_emparejado;
use crate::model::peer::{Peer, TrustState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_rustls::client::TlsStream;
use uuid::Uuid;

/// Un emparejamiento abandonado no debe retener un socket indefinidamente.
const CADUCIDAD: Duration = Duration::from_secs(90);

struct EnCurso {
    stream: TlsStream<TcpStream>,
    peer: Peer,
    creado: Instant,
}

/// Emparejamientos esperando la decisión del usuario.
#[derive(Default)]
pub struct PairingStore {
    activos: Mutex<HashMap<Uuid, EnCurso>>,
}

impl PairingStore {
    pub fn new() -> Self {
        Self::default()
    }

    async fn purgar(&self, mapa: &mut HashMap<Uuid, EnCurso>) {
        let ahora = Instant::now();
        mapa.retain(|_, v| ahora.duration_since(v.creado) < CADUCIDAD);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingStarted {
    pub pairing_id: Uuid,
    /// Los seis dígitos que esta pantalla debe mostrar para que el usuario los compare
    /// con los del otro equipo. No es un secreto ni una contraseña.
    pub verification_code: String,
    pub peer: Peer,
}

#[tauri::command]
pub async fn peers_pairing_start(
    host: String,
    port: u16,
    state: State<'_, AppState>,
) -> Result<IpcResult<PairingStarted>, String> {
    let (peer, mut stream) = match conectar_y_saludar(&host, port, &state.identity).await {
        Ok(v) => v,
        Err(e) => {
            // El detalle (host:puerto, dentro de `e`) es diagnóstico avanzado (principio
            // XIII, enmienda 0.8.0): completo en el AppError que ve la persona y en el
            // log a debug; el hito de fallo, a warn, sin repetirlo.
            tracing::warn!("Emparejamiento: no se pudo conectar con el equipo");
            tracing::debug!("Emparejamiento: no se pudo conectar con {host}:{port}: {e}");
            return Ok(IpcResult::err(AppError::new(
                ErrorCode::ConnCannotReach,
                ErrorSeverity::Error,
                e,
            )));
        }
    };

    let emparejamiento =
        match EmparejamientoEnCurso::iniciar(&state.identity.fingerprint, &peer.fingerprint) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    "Emparejamiento: no se pudo iniciar con {}: {e}",
                    peer.instance_id
                );
                return Ok(IpcResult::err(AppError::new(
                    ErrorCode::InternalError,
                    ErrorSeverity::Error,
                    e.to_string(),
                )));
            }
        };

    // pairing_id es el identificador que reutiliza el otro extremo (hallazgo de la
    // auditoría de 2026-09-28): correlaciona esta línea con las de PAIR_REQUEST/
    // PAIR_RESULT en el log de la otra máquina.
    tracing::info!(
        pairing_id = %emparejamiento.id,
        "Emparejamiento: iniciado con {}",
        peer.instance_id
    );
    tracing::debug!(pairing_id = %emparejamiento.id, "Emparejamiento: iniciado con {host}:{port}");

    // El otro equipo enseña su código en cuanto recibe esto: los dos lo ven a la vez.
    if let Err(e) = enviar_solicitud(&mut stream, &emparejamiento).await {
        tracing::warn!(
            pairing_id = %emparejamiento.id,
            "Emparejamiento: no se pudo enviar la solicitud al otro equipo"
        );
        tracing::debug!(pairing_id = %emparejamiento.id, "PAIR_REQUEST a {host}:{port}: {e}");
        return Ok(IpcResult::err(AppError::new(
            ErrorCode::ConnCannotReach,
            ErrorSeverity::Error,
            e.to_string(),
        )));
    }

    let respuesta = PairingStarted {
        pairing_id: emparejamiento.id,
        verification_code: emparejamiento.codigo.clone(),
        peer: peer.clone(),
    };

    let mut activos = state.pairings.activos.lock().await;
    state.pairings.purgar(&mut activos).await;
    activos.insert(
        emparejamiento.id,
        EnCurso {
            stream,
            peer,
            creado: Instant::now(),
        },
    );

    Ok(IpcResult::ok(respuesta))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingOutcome {
    pub accepted: bool,
    pub peer: Option<Peer>,
}

#[tauri::command]
pub async fn peers_pairing_confirm(
    pairing_id: Uuid,
    accepted: bool,
    state: State<'_, AppState>,
) -> Result<IpcResult<PairingOutcome>, String> {
    let mut en_curso = {
        let mut activos = state.pairings.activos.lock().await;
        match activos.remove(&pairing_id) {
            Some(v) => v,
            None => {
                return Ok(IpcResult::err(AppError::new(
                    ErrorCode::InternalError,
                    ErrorSeverity::Error,
                    "El emparejamiento caducó o no existe; vuelve a empezar".to_string(),
                )));
            }
        }
    };

    let resultado = completar_emparejamiento(&mut en_curso.stream, accepted).await;

    match resultado {
        Ok(true) => {
            // Solo aquí se guarda confianza: los dos usuarios dijeron que sí sobre la
            // misma conexión y las mismas huellas.
            let mut confiable = en_curso.peer.clone();
            confiable.trust_state = TrustState::Trusted;
            // La autoaceptación arranca desactivada aunque el peer sea de confianza
            // (FR-015): confiar en un equipo no es delegarle el consentimiento.
            confiable.auto_accept = false;

            let db = state.database.connection().lock().unwrap();
            if let Err(e) = guardar_emparejado(&db, &confiable) {
                tracing::error!(
                    pairing_id = %pairing_id,
                    "Emparejamiento: aceptado por las dos partes pero no se pudo guardar: {e}"
                );
                return Ok(IpcResult::err(AppError::new(
                    ErrorCode::InternalError,
                    ErrorSeverity::Error,
                    format!("No se pudo guardar el equipo emparejado: {e}"),
                )));
            }
            state.aviso_snapshot.notify_one();
            tracing::info!(
                pairing_id = %pairing_id,
                "Emparejamiento: aceptado y guardado ({})",
                confiable.instance_id
            );

            Ok(IpcResult::ok(PairingOutcome {
                accepted: true,
                peer: Some(confiable),
            }))
        }
        Ok(false) => {
            tracing::info!(
                pairing_id = %pairing_id,
                "Emparejamiento: rechazado (local={accepted})"
            );
            Ok(IpcResult::ok(PairingOutcome {
                accepted: false,
                peer: None,
            }))
        }
        Err(e) => {
            tracing::warn!(pairing_id = %pairing_id, "Emparejamiento: fallo protocolar: {e}");
            Ok(IpcResult::err(AppError::new(
                ErrorCode::PeerPairingMismatch,
                ErrorSeverity::Error,
                e.to_string(),
            )))
        }
    }
}
