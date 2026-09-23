use networkbench_lib::errors::ErrorCode;
use networkbench_lib::ipc::response::{IpcResult, OneTimeTokenStore};

#[test]
fn test_ipc_commands_input_validation() {
    let store = OneTimeTokenStore::new();

    // Intentar consumir un token con cadena vacía o token no emitido
    let empty_res = store.consume("", "app.confirmClose", None);
    assert!(empty_res.is_err());
    let err = empty_res.unwrap_err();
    assert_eq!(err.code, ErrorCode::PeerPairingExpired);

    // Intentar consumir token con operación no permitida / no coincidente
    let valid_token = store.issue("app.confirmClose", None, 60);
    let disallowed_res = store.consume(&valid_token, "unauthorized.command", None);
    assert!(disallowed_res.is_err());
    let err_disallowed = disallowed_res.unwrap_err();
    assert_eq!(err_disallowed.code, ErrorCode::PeerPairingMismatch);
}

#[test]
fn test_ipc_result_error_sanitization_no_secrets() {
    let err = networkbench_lib::errors::AppError::new(
        ErrorCode::InternalError,
        networkbench_lib::errors::ErrorSeverity::Fatal,
        "errors.NB-INTERNAL-001",
    );

    let res: IpcResult<String> = IpcResult::err(err);
    let json = serde_json::to_string(&res).unwrap();

    // El JSON no debe contener secretos ni trazas crudas de depuración
    assert!(!json.contains("secret"));
    assert!(!json.contains("password"));
    assert!(!json.contains("stacktrace"));
}
