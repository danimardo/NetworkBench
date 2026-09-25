//! Consentimiento local para solicitudes de sesión entrantes (T177, FR-016).
//!
//! Hasta esta tarea, un equipo `Trusted` sin autoaceptación se rechazaba sin más: no
//! existía forma de preguntarle a la persona. Este módulo guarda la solicitud —con lo
//! que la interfaz necesita mostrar— mientras se espera su decisión, sin bloquear otras
//! solicitudes de otros equipos ni el resto de la aplicación mientras tanto.
//!
//! El emparejamiento entrante (`PAIR_REQUEST`, FR-012) no está aquí: sigue sin
//! atenderse, es un flujo distinto con su propio código de verificación y queda como
//! trabajo pendiente declarado (Fase 13).

use crate::model::peer::Peer;
use crate::model::plan::BenchmarkPlan;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{Mutex, oneshot};
use uuid::Uuid;

/// Cuánto se espera la decisión de la persona antes de rechazar por su cuenta
/// (`contracts/peer-protocol.md`: «Pairing or user acceptance: 60 s»).
pub const ESPERA_DECISION: Duration = Duration::from_secs(60);

/// Lo que la interfaz necesita mostrar para decidir. Sin material privado: el mismo
/// `Peer` que ya se le enseña en la pantalla de equipos.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolicitudEntranteEvento {
    pub request_id: Uuid,
    pub peer: Peer,
    pub plan: BenchmarkPlan,
}

struct Pendiente {
    solicitud: SolicitudEntranteEvento,
    decidir: oneshot::Sender<bool>,
}

/// Solicitudes de sesión esperando una decisión humana. Varias pueden coexistir —una por
/// equipo que pregunte a la vez—, aunque solo una prueba puede terminar aceptándose:
/// la reserva de la sesión única sigue ocurriendo después, en `SessionService`.
#[derive(Default)]
pub struct SolicitudesEntrantes {
    activas: Mutex<HashMap<Uuid, Pendiente>>,
}

impl SolicitudesEntrantes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra una solicitud y devuelve el evento a anunciar y el receptor de la
    /// decisión.
    pub async fn registrar(
        &self,
        peer: Peer,
        plan: BenchmarkPlan,
    ) -> (SolicitudEntranteEvento, oneshot::Receiver<bool>) {
        let evento = SolicitudEntranteEvento {
            request_id: Uuid::new_v4(),
            peer,
            plan,
        };
        let (decidir, esperar) = oneshot::channel();
        self.activas.lock().await.insert(
            evento.request_id,
            Pendiente {
                solicitud: evento.clone(),
                decidir,
            },
        );
        (evento, esperar)
    }

    /// Lo que sigue esperando decisión ahora mismo. Para una interfaz que recién se abre
    /// y necesita ponerse al día sin haber visto el evento original.
    pub async fn listar(&self) -> Vec<SolicitudEntranteEvento> {
        self.activas
            .lock()
            .await
            .values()
            .map(|p| p.solicitud.clone())
            .collect()
    }

    /// La persona decidió. `Err` si la solicitud ya no está (caducó, o ya se decidió):
    /// una segunda respuesta a la misma solicitud no tiene efecto, no es un error del
    /// usuario que deba mostrarse como tal.
    pub async fn responder(&self, id: Uuid, aceptar: bool) -> Result<(), String> {
        let pendiente = self
            .activas
            .lock()
            .await
            .remove(&id)
            .ok_or_else(|| "Esta solicitud ya no está esperando una decisión".to_string())?;
        let _ = pendiente.decidir.send(aceptar);
        Ok(())
    }

    /// Retira la solicitud sin decidir nada: para cuando el plazo se agotó y hay que
    /// dejar de ofrecerla en la interfaz.
    pub async fn retirar(&self, id: Uuid) {
        self.activas.lock().await.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::peer::TrustState;

    fn peer_de_prueba() -> Peer {
        Peer {
            instance_id: Uuid::new_v4(),
            display_name: "Equipo de prueba".to_string(),
            fingerprint: "0".repeat(64),
            addresses: vec!["127.0.0.1:0".to_string()],
            trust_state: TrustState::Trusted,
            auto_accept: false,
            last_seen: String::new(),
            alias: None,
        }
    }

    #[tokio::test]
    async fn registrar_y_responder_entrega_la_decision() {
        let store = SolicitudesEntrantes::new();
        let (evento, esperar) = store
            .registrar(peer_de_prueba(), BenchmarkPlan::new_standard_tcp(5990))
            .await;
        store.responder(evento.request_id, true).await.unwrap();
        assert!(esperar.await.unwrap());
    }

    #[tokio::test]
    async fn listar_muestra_lo_que_sigue_pendiente() {
        let store = SolicitudesEntrantes::new();
        let (evento, _esperar) = store
            .registrar(peer_de_prueba(), BenchmarkPlan::new_standard_tcp(5991))
            .await;

        let pendientes = store.listar().await;
        assert_eq!(pendientes.len(), 1);
        assert_eq!(pendientes[0].request_id, evento.request_id);

        store.responder(evento.request_id, false).await.unwrap();
        assert!(store.listar().await.is_empty());
    }

    #[tokio::test]
    async fn responder_una_solicitud_que_ya_no_esta_no_es_un_panico() {
        let store = SolicitudesEntrantes::new();
        let error = store.responder(Uuid::new_v4(), true).await.unwrap_err();
        assert!(!error.is_empty());
    }

    #[tokio::test]
    async fn retirar_hace_que_esperar_la_decision_falle_en_vez_de_colgarse() {
        let store = SolicitudesEntrantes::new();
        let (evento, esperar) = store
            .registrar(peer_de_prueba(), BenchmarkPlan::new_standard_tcp(5992))
            .await;
        store.retirar(evento.request_id).await;
        assert!(esperar.await.is_err());
    }

    #[tokio::test]
    async fn dos_solicitudes_a_la_vez_se_deciden_de_forma_independiente() {
        let store = SolicitudesEntrantes::new();
        let (e1, esperar1) = store
            .registrar(peer_de_prueba(), BenchmarkPlan::new_standard_tcp(5993))
            .await;
        let (e2, esperar2) = store
            .registrar(peer_de_prueba(), BenchmarkPlan::new_standard_tcp(5994))
            .await;
        store.responder(e2.request_id, false).await.unwrap();
        store.responder(e1.request_id, true).await.unwrap();
        assert!(esperar1.await.unwrap());
        assert!(!esperar2.await.unwrap());
    }
}
