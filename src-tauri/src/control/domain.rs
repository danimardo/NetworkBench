use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionState {
    Idle,
    Connecting,
    HelloPending,
    Pairing,
    Requesting,
    Preparing,
    RunningSend,
    RunningReceive,
    RunningBoth,
    Analyzing,
    Completed,
    Cancelling,
    Cancelled,
    Failed,
}

impl SessionState {
    pub fn is_active(&self) -> bool {
        !matches!(
            self,
            SessionState::Idle
                | SessionState::Completed
                | SessionState::Cancelled
                | SessionState::Failed
        )
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SessionState::Completed | SessionState::Cancelled | SessionState::Failed
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateMachineError {
    SessionAlreadyActive {
        active_session_id: Uuid,
    },
    NoActiveSession,
    IllegalTransition {
        from: SessionState,
        to: SessionState,
    },
    SessionIdMismatch {
        expected: Uuid,
        actual: Uuid,
    },
}

#[derive(Debug, Clone)]
pub struct SessionStateMachine {
    current_state: SessionState,
    session_id: Option<Uuid>,
    peer_instance_id: Option<Uuid>,
}

impl Default for SessionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStateMachine {
    pub fn new() -> Self {
        Self {
            current_state: SessionState::Idle,
            session_id: None,
            peer_instance_id: None,
        }
    }

    pub fn current_state(&self) -> SessionState {
        self.current_state
    }

    pub fn session_id(&self) -> Option<Uuid> {
        self.session_id
    }

    pub fn peer_instance_id(&self) -> Option<Uuid> {
        self.peer_instance_id
    }

    /// Inicia una nueva sesión. Falla si ya hay una sesión activa en curso.
    pub fn start_session(
        &mut self,
        session_id: Uuid,
        peer_instance_id: Uuid,
    ) -> Result<(), StateMachineError> {
        if self.current_state.is_active() {
            return Err(StateMachineError::SessionAlreadyActive {
                active_session_id: self.session_id.unwrap_or(session_id),
            });
        }

        self.session_id = Some(session_id);
        self.peer_instance_id = Some(peer_instance_id);
        self.current_state = SessionState::Connecting;
        Ok(())
    }

    /// Ejecuta una transición de estado validando las reglas de la máquina de estados.
    pub fn transition_to(
        &mut self,
        target: SessionState,
        session_id: Uuid,
    ) -> Result<(), StateMachineError> {
        if let Some(active_id) = self.session_id {
            if active_id != session_id {
                return Err(StateMachineError::SessionIdMismatch {
                    expected: active_id,
                    actual: session_id,
                });
            }
        } else if target != SessionState::Idle {
            return Err(StateMachineError::NoActiveSession);
        }

        if self.can_transition_to(target) {
            self.current_state = target;
            Ok(())
        } else {
            Err(StateMachineError::IllegalTransition {
                from: self.current_state,
                to: target,
            })
        }
    }

    /// Regla pura de comprobación de si la transición de origen a destino es legal.
    pub fn can_transition_to(&self, target: SessionState) -> bool {
        use SessionState::*;

        // Idempotencia: el mismo estado
        if self.current_state == target {
            return true;
        }

        // Cancelación y fallo permitidos desde cualquier estado activo
        if matches!(target, Cancelling | Failed) && self.current_state.is_active() {
            return true;
        }

        // Transición de Cancelling a Cancelled
        if self.current_state == Cancelling && target == Cancelled {
            return true;
        }

        match (self.current_state, target) {
            (Idle, Connecting) => true,
            (Connecting, HelloPending) => true,
            (HelloPending, Pairing) => true,
            (HelloPending, Requesting) => true, // Si el peer ya era confiable (salta pairing)
            (Pairing, Requesting) => true,
            (Requesting, Preparing) => true,
            (Preparing, RunningSend) => true,
            (Preparing, RunningReceive) => true,
            (Preparing, RunningBoth) => true,
            (RunningSend, RunningReceive) => true,
            (RunningReceive, RunningSend) => true,
            (RunningSend, Analyzing) => true,
            (RunningReceive, Analyzing) => true,
            (RunningBoth, Analyzing) => true,
            (Analyzing, Completed) => true,
            // Reset al estado Idle solo tras estados terminales
            (Completed | Cancelled | Failed, Idle) => true,
            _ => false,
        }
    }

    /// Cancela la sesión activa de manera idempotente.
    pub fn cancel(&mut self, session_id: Uuid) -> Result<(), StateMachineError> {
        if let Some(active_id) = self.session_id
            && active_id != session_id
        {
            return Err(StateMachineError::SessionIdMismatch {
                expected: active_id,
                actual: session_id,
            });
        }

        if !self.current_state.is_active() {
            // Ya es terminal o Idle, cancelación idempotente
            return Ok(());
        }

        self.current_state = SessionState::Cancelling;
        Ok(())
    }

    /// Confirma la cancelación completada.
    pub fn complete_cancellation(&mut self) {
        if self.current_state == SessionState::Cancelling {
            self.current_state = SessionState::Cancelled;
        }
    }

    /// Falla la sesión con un error terminal.
    pub fn fail(&mut self, session_id: Uuid) -> Result<(), StateMachineError> {
        if let Some(active_id) = self.session_id
            && active_id != session_id
        {
            return Err(StateMachineError::SessionIdMismatch {
                expected: active_id,
                actual: session_id,
            });
        }

        self.current_state = SessionState::Failed;
        Ok(())
    }
}
