use super::domain::{SessionState, SessionStateMachine, StateMachineError};
use crate::model::peer::Peer;
use crate::model::plan::BenchmarkPlan;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct SessionService {
    state_machine: Arc<Mutex<SessionStateMachine>>,
    active_peer: Arc<Mutex<Option<Peer>>>,
    active_plan: Arc<Mutex<Option<BenchmarkPlan>>>,
}

impl Default for SessionService {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            state_machine: Arc::new(Mutex::new(SessionStateMachine::new())),
            active_peer: Arc::new(Mutex::new(None)),
            active_plan: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn current_state(&self) -> SessionState {
        let sm = self.state_machine.lock().await;
        sm.current_state()
    }

    pub async fn active_session_id(&self) -> Option<Uuid> {
        let sm = self.state_machine.lock().await;
        sm.session_id()
    }

    pub async fn start_session(
        &self,
        peer: Peer,
        plan: BenchmarkPlan,
    ) -> Result<Uuid, StateMachineError> {
        let session_id = Uuid::new_v4();
        let mut sm = self.state_machine.lock().await;
        sm.start_session(session_id, peer.instance_id)?;

        let mut p_lock = self.active_peer.lock().await;
        *p_lock = Some(peer);

        let mut plan_lock = self.active_plan.lock().await;
        *plan_lock = Some(plan);

        Ok(session_id)
    }

    pub async fn transition_to(
        &self,
        target: SessionState,
        session_id: Uuid,
    ) -> Result<(), StateMachineError> {
        let mut sm = self.state_machine.lock().await;
        sm.transition_to(target, session_id)
    }

    pub async fn cancel(&self, session_id: Uuid) -> Result<(), StateMachineError> {
        let mut sm = self.state_machine.lock().await;
        sm.cancel(session_id)?;
        sm.complete_cancellation();

        let mut p_lock = self.active_peer.lock().await;
        *p_lock = None;

        let mut plan_lock = self.active_plan.lock().await;
        *plan_lock = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_service_lifecycle() {
        let service = SessionService::new();
        assert_eq!(service.current_state().await, SessionState::Idle);

        let peer = Peer::new(
            Uuid::new_v4(),
            "Servidor-Pruebas".into(),
            "1111111111111111111111111111111111111111111111111111111111111111".into(),
            vec!["127.0.0.1:7411".into()],
        )
        .unwrap();

        let plan = BenchmarkPlan::new_standard_tcp(7412);
        let s_id = service.start_session(peer, plan).await.expect("Iniciar");
        assert_eq!(service.current_state().await, SessionState::Connecting);

        service.transition_to(SessionState::HelloPending, s_id).await.unwrap();
        assert_eq!(service.current_state().await, SessionState::HelloPending);

        service.cancel(s_id).await.expect("Cancelar sesión");
        assert_eq!(service.current_state().await, SessionState::Cancelled);
    }
}
