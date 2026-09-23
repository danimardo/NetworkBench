use super::*;
use uuid::Uuid;

#[test]
fn test_happy_path_state_transitions() {
    let mut sm = SessionStateMachine::new();
    assert_eq!(sm.current_state(), SessionState::Idle);

    let session_id = Uuid::new_v4();
    let peer_id = Uuid::new_v4();

    // 1. Iniciar sesión -> Connecting
    assert!(sm.start_session(session_id, peer_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Connecting);
    assert!(sm.current_state().is_active());

    // 2. Connecting -> HelloPending
    assert!(sm.transition_to(SessionState::HelloPending, session_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::HelloPending);

    // 3. HelloPending -> Pairing
    assert!(sm.transition_to(SessionState::Pairing, session_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Pairing);

    // 4. Pairing -> Requesting
    assert!(sm.transition_to(SessionState::Requesting, session_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Requesting);

    // 5. Requesting -> Preparing
    assert!(sm.transition_to(SessionState::Preparing, session_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Preparing);

    // 6. Preparing -> RunningSend
    assert!(sm.transition_to(SessionState::RunningSend, session_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::RunningSend);

    // 7. RunningSend -> RunningReceive
    assert!(sm.transition_to(SessionState::RunningReceive, session_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::RunningReceive);

    // 8. RunningReceive -> Analyzing
    assert!(sm.transition_to(SessionState::Analyzing, session_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Analyzing);

    // 9. Analyzing -> Completed
    assert!(sm.transition_to(SessionState::Completed, session_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Completed);
    assert!(sm.current_state().is_terminal());
    assert!(!sm.current_state().is_active());
}

#[test]
fn test_single_active_session_enforcement() {
    let mut sm = SessionStateMachine::new();
    let session_1 = Uuid::new_v4();
    let peer_1 = Uuid::new_v4();

    assert!(sm.start_session(session_1, peer_1).is_ok());

    // Intentar iniciar una segunda sesión mientras la primera está activa debe fallar
    let session_2 = Uuid::new_v4();
    let peer_2 = Uuid::new_v4();
    let res = sm.start_session(session_2, peer_2);

    assert_eq!(
        res,
        Err(StateMachineError::SessionAlreadyActive {
            active_session_id: session_1
        })
    );
}

#[test]
fn test_illegal_transitions_rejected() {
    let mut sm = SessionStateMachine::new();
    let s_id = Uuid::new_v4();
    let p_id = Uuid::new_v4();

    // Desde Idle no se puede saltar directamente a RunningSend o Analyzing
    assert_eq!(
        sm.transition_to(SessionState::RunningSend, s_id),
        Err(StateMachineError::NoActiveSession)
    );

    assert!(sm.start_session(s_id, p_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Connecting);

    // Desde Connecting no se puede saltar a Analyzing o Completed
    assert_eq!(
        sm.transition_to(SessionState::Analyzing, s_id),
        Err(StateMachineError::IllegalTransition {
            from: SessionState::Connecting,
            to: SessionState::Analyzing
        })
    );

    assert_eq!(
        sm.transition_to(SessionState::Completed, s_id),
        Err(StateMachineError::IllegalTransition {
            from: SessionState::Connecting,
            to: SessionState::Completed
        })
    );
}

#[test]
fn test_mismatched_session_id_rejected() {
    let mut sm = SessionStateMachine::new();
    let s_id1 = Uuid::new_v4();
    let s_id2 = Uuid::new_v4();
    let p_id = Uuid::new_v4();

    sm.start_session(s_id1, p_id).unwrap();

    let res = sm.transition_to(SessionState::HelloPending, s_id2);
    assert_eq!(
        res,
        Err(StateMachineError::SessionIdMismatch {
            expected: s_id1,
            actual: s_id2
        })
    );
}

#[test]
fn test_idempotent_cancellation() {
    let mut sm = SessionStateMachine::new();
    let s_id = Uuid::new_v4();
    let p_id = Uuid::new_v4();

    sm.start_session(s_id, p_id).unwrap();
    sm.transition_to(SessionState::HelloPending, s_id).unwrap();
    sm.transition_to(SessionState::Pairing, s_id).unwrap();

    // Cancelar en Pairing -> Cancelling
    assert!(sm.cancel(s_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Cancelling);

    // Segunda cancelación es idempotente
    assert!(sm.cancel(s_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Cancelling);

    // Completar cancelación -> Cancelled
    sm.complete_cancellation();
    assert_eq!(sm.current_state(), SessionState::Cancelled);
    assert!(sm.current_state().is_terminal());

    // Cancelar una sesión ya Cancelled no da error
    assert!(sm.cancel(s_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Cancelled);
}

#[test]
fn test_failure_transition() {
    let mut sm = SessionStateMachine::new();
    let s_id = Uuid::new_v4();
    let p_id = Uuid::new_v4();

    sm.start_session(s_id, p_id).unwrap();
    sm.fail(s_id).unwrap();

    assert_eq!(sm.current_state(), SessionState::Failed);
    assert!(sm.current_state().is_terminal());
    assert!(!sm.current_state().is_active());
}

#[test]
fn test_simultaneous_running_both_transitions() {
    let mut sm = SessionStateMachine::new();
    let s_id = Uuid::new_v4();
    let p_id = Uuid::new_v4();

    sm.start_session(s_id, p_id).unwrap();
    sm.transition_to(SessionState::HelloPending, s_id).unwrap();
    sm.transition_to(SessionState::Requesting, s_id).unwrap();
    sm.transition_to(SessionState::Preparing, s_id).unwrap();

    // Preparing -> RunningBoth (simultáneo)
    assert!(sm.transition_to(SessionState::RunningBoth, s_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::RunningBoth);
    assert!(sm.current_state().is_active());

    // RunningBoth -> Analyzing
    assert!(sm.transition_to(SessionState::Analyzing, s_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Analyzing);

    // Analyzing -> Completed
    assert!(sm.transition_to(SessionState::Completed, s_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Completed);
}

#[test]
fn test_unidirectional_transitions() {
    // Solo Forward: Preparing -> RunningSend -> Analyzing -> Completed
    let mut sm = SessionStateMachine::new();
    let s_id = Uuid::new_v4();
    let p_id = Uuid::new_v4();

    sm.start_session(s_id, p_id).unwrap();
    sm.transition_to(SessionState::HelloPending, s_id).unwrap();
    sm.transition_to(SessionState::Requesting, s_id).unwrap();
    sm.transition_to(SessionState::Preparing, s_id).unwrap();
    sm.transition_to(SessionState::RunningSend, s_id).unwrap();
    assert!(sm.transition_to(SessionState::Analyzing, s_id).is_ok());
    assert_eq!(sm.current_state(), SessionState::Analyzing);

    // Solo Reverse: Preparing -> RunningReceive -> Analyzing -> Completed
    let mut sm2 = SessionStateMachine::new();
    let s2_id = Uuid::new_v4();
    let p2_id = Uuid::new_v4();

    sm2.start_session(s2_id, p2_id).unwrap();
    sm2.transition_to(SessionState::HelloPending, s2_id).unwrap();
    sm2.transition_to(SessionState::Requesting, s2_id).unwrap();
    sm2.transition_to(SessionState::Preparing, s2_id).unwrap();
    sm2.transition_to(SessionState::RunningReceive, s2_id).unwrap();
    assert!(sm2.transition_to(SessionState::Analyzing, s2_id).is_ok());
    assert_eq!(sm2.current_state(), SessionState::Analyzing);
}

#[test]
fn test_running_both_serialization() {
    let state = SessionState::RunningBoth;
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(json, "\"RUNNING_BOTH\"");
    let de: SessionState = serde_json::from_str(&json).unwrap();
    assert_eq!(de, SessionState::RunningBoth);
}

