use networkbench_lib::errors::{AppError, ErrorAction, ErrorCode, ErrorSeverity};
use networkbench_lib::ipc::response::{IpcResult, OneTimeTokenStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SampleData {
    name: String,
    count: u32,
}

#[test]
fn test_ipc_result_serialization_round_trip() {
    let success = IpcResult::ok(SampleData {
        name: "test".to_string(),
        count: 42,
    });

    let json_success = serde_json::to_string(&success).unwrap();
    assert!(json_success.contains("\"ok\":true"));
    assert!(json_success.contains("\"value\":{"));

    let deserialized_success: IpcResult<SampleData> = serde_json::from_str(&json_success).unwrap();
    assert_eq!(success, deserialized_success);

    let err = AppError::new(
        ErrorCode::ConnCannotReach,
        ErrorSeverity::Error,
        "errors.NB-CONN-001",
    )
    .with_action(ErrorAction::Retry);

    let failure: IpcResult<SampleData> = IpcResult::err(err);
    let json_failure = serde_json::to_string(&failure).unwrap();
    assert!(json_failure.contains("\"ok\":false"));
    assert!(json_failure.contains("\"error\":{"));

    let deserialized_failure: IpcResult<SampleData> = serde_json::from_str(&json_failure).unwrap();
    assert_eq!(failure, deserialized_failure);
}

#[test]
fn test_ipc_result_invalid_deserialization() {
    // ok: true pero falta value
    let invalid_1 = r#"{"ok":true}"#;
    let res_1: Result<IpcResult<SampleData>, _> = serde_json::from_str(invalid_1);
    assert!(res_1.is_err());

    // ok: false pero falta error
    let invalid_2 = r#"{"ok":false}"#;
    let res_2: Result<IpcResult<SampleData>, _> = serde_json::from_str(invalid_2);
    assert!(res_2.is_err());
}

#[test]
fn test_one_time_token_store_lifecycle() {
    let store = OneTimeTokenStore::new();

    // Emisión de token para confirmClose
    let token = store.issue("confirmClose", Some("hash_123"), 60);
    assert!(uuid::Uuid::parse_str(&token).is_ok());

    // Operación incorrecta debe fallar
    let err_op = store.consume(&token, "wrongOperation", Some("hash_123"));
    assert!(err_op.is_err());

    // Payload hash incorrecto debe fallar
    let err_hash = store.consume(&token, "confirmClose", Some("wrong_hash"));
    assert!(err_hash.is_err());

    // Consumo legítimo debe tener éxito
    let ok = store.consume(&token, "confirmClose", Some("hash_123"));
    assert!(ok.is_ok());

    // Segundo consumo del mismo token DEBE fallar estrictamente (un solo uso)
    let second_consume = store.consume(&token, "confirmClose", Some("hash_123"));
    assert!(second_consume.is_err());
    let err = second_consume.unwrap_err();
    assert_eq!(err.code, ErrorCode::PeerPairingExpired);
}
