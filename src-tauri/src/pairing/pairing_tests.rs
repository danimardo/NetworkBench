use super::*;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[test]
fn test_pairing_code_symmetry_between_peers() {
    let fp_a = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let fp_b = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let session_id = Uuid::new_v4();

    // A deriva con (fp_a, fp_b)
    let code_a = derive_pairing_code(fp_a, fp_b, &session_id).expect("Derivación A");
    // B deriva con (fp_b, fp_a)
    let code_b = derive_pairing_code(fp_b, fp_a, &session_id).expect("Derivación B");

    assert_eq!(
        code_a, code_b,
        "Ambos peers deben obtener exactamente el mismo código"
    );
    assert_eq!(code_a.len(), 6);
    assert!(code_a.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_pairing_table_scenarios() {
    struct TestCase<'a> {
        name: &'static str,
        local_fp: &'static str,
        remote_fp: &'static str,
        claimed_remote_fp: &'static str,
        provided_code: &'a str,
        offset_seconds: u64,
        expected_result: Result<(), PairingError>,
    }

    let fp1 = "1111111111111111111111111111111111111111111111111111111111111111";
    let fp2 = "2222222222222222222222222222222222222222222222222222222222222222";
    let fp3_impostor = "3333333333333333333333333333333333333333333333333333333333333333";

    let session_id = Uuid::parse_str("12345678-1234-4234-8234-123456789abc").unwrap();
    let correct_code = derive_pairing_code(fp1, fp2, &session_id).unwrap();

    let tests = vec![
        TestCase {
            name: "Código coincidente dentro de tiempo (10 s) -> Éxito",
            local_fp: fp1,
            remote_fp: fp2,
            claimed_remote_fp: fp2,
            provided_code: &correct_code,
            offset_seconds: 10,
            expected_result: Ok(()),
        },
        TestCase {
            name: "Código coincidente exactamente en límite de tiempo (60 s) -> Éxito",
            local_fp: fp1,
            remote_fp: fp2,
            claimed_remote_fp: fp2,
            provided_code: &correct_code,
            offset_seconds: 60,
            expected_result: Ok(()),
        },
        TestCase {
            name: "Código caducado a los 61 s -> Expired",
            local_fp: fp1,
            remote_fp: fp2,
            claimed_remote_fp: fp2,
            provided_code: &correct_code,
            offset_seconds: 61,
            expected_result: Err(PairingError::Expired),
        },
        TestCase {
            name: "Código erróneo dentro de tiempo -> CodeMismatch",
            local_fp: fp1,
            remote_fp: fp2,
            claimed_remote_fp: fp2,
            provided_code: "999999",
            offset_seconds: 5,
            expected_result: Err(PairingError::CodeMismatch),
        },
        TestCase {
            name: "Huella cambiada / impostor -> FingerprintMismatch",
            local_fp: fp1,
            remote_fp: fp2,
            claimed_remote_fp: fp3_impostor,
            provided_code: &correct_code,
            offset_seconds: 5,
            expected_result: Err(PairingError::FingerprintMismatch {
                expected: fp2.to_string(),
                actual: fp3_impostor.to_string(),
            }),
        },
        TestCase {
            name: "Código con formato inválido (no 6 dígitos) -> InvalidFormat",
            local_fp: fp1,
            remote_fp: fp2,
            claimed_remote_fp: fp2,
            provided_code: "123",
            offset_seconds: 5,
            expected_result: Err(PairingError::InvalidFormat),
        },
    ];

    for tc in tests {
        let base_instant = Instant::now();
        let pairing = ActivePairing::with_instant(
            session_id,
            tc.local_fp,
            tc.remote_fp,
            base_instant,
            Duration::from_secs(60),
        )
        .expect("Creación de ActivePairing");

        let verify_instant = base_instant + Duration::from_secs(tc.offset_seconds);
        let result = pairing.verify(tc.provided_code, tc.claimed_remote_fp, verify_instant);

        assert_eq!(
            result, tc.expected_result,
            "Fallo en caso de prueba: {}",
            tc.name
        );
    }
}
