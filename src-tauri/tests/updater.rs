use networkbench_lib::updater::{
    UpdateStatus, evaluate_manifest, is_newer_version, parse_version, validate_download_url,
};

#[test]
fn test_updater_version_parsing_and_comparison() {
    assert_eq!(parse_version("1.0.0"), Some((1, 0, 0)));
    assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
    assert_eq!(parse_version("2.10.4-beta1"), Some((2, 10, 4)));

    // Comparaciones SemVer
    assert!(is_newer_version("1.0.0", "1.0.1"));
    assert!(is_newer_version("1.0.0", "1.1.0"));
    assert!(is_newer_version("1.0.0", "2.0.0"));
    assert!(!is_newer_version("1.0.0", "1.0.0"));
    assert!(!is_newer_version("1.1.0", "1.0.9"));
    assert!(!is_newer_version("2.0.0", "1.9.9"));
}

#[test]
fn test_updater_manifest_valid_and_upgrade_available() {
    let manifest_json = r#"{
        "version": "1.1.0",
        "notes": "Nuevas mejoras de rendimiento y diagnóstico",
        "pub_date": "2026-09-22T10:00:00Z",
        "platforms": {
            "windows-x86_64": {
                "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXkKUldUUWd2...",
                "url": "https://github.com/danimardo/NetworkBench/releases/download/v1.1.0/NetworkBench-Setup-v1.1.0.exe"
            }
        }
    }"#;

    let res = evaluate_manifest(manifest_json, "1.0.0", "windows-x86_64", false);
    assert!(res.is_ok());
    match res.unwrap() {
        UpdateStatus::UpdateAvailable {
            version,
            notes,
            download_url,
            signature,
        } => {
            assert_eq!(version, "1.1.0");
            assert_eq!(notes, "Nuevas mejoras de rendimiento y diagnóstico");
            assert!(download_url.contains("/releases/download/v1.1.0/"));
            assert!(!signature.is_empty());
        }
        other => panic!("Esperado UpdateAvailable, obtenido {:?}", other),
    }
}

#[test]
fn test_updater_manifest_same_or_lower_version_up_to_date() {
    let manifest_json = r#"{
        "version": "1.0.0",
        "platforms": {
            "windows-x86_64": {
                "signature": "sig123",
                "url": "https://github.com/danimardo/NetworkBench/releases/download/v1.0.0/NetworkBench-Setup-v1.0.0.exe"
            }
        }
    }"#;

    // Misma versión
    let res_same = evaluate_manifest(manifest_json, "1.0.0", "windows-x86_64", false).unwrap();
    assert_eq!(res_same, UpdateStatus::UpToDate);

    // Intento de downgrade (versión candidata 0.9.0 menor que la instalada 1.0.0)
    let downgrade_json = r#"{
        "version": "0.9.0",
        "platforms": {
            "windows-x86_64": {
                "signature": "sig123",
                "url": "https://github.com/danimardo/NetworkBench/releases/download/v0.9.0/NetworkBench-Setup-v0.9.0.exe"
            }
        }
    }"#;
    let res_downgrade =
        evaluate_manifest(downgrade_json, "1.0.0", "windows-x86_64", false).unwrap();
    assert_eq!(res_downgrade, UpdateStatus::UpToDate);
}

#[test]
fn test_updater_rejects_mutable_latest_urls() {
    // Prohibición constitucional y ADR-007 de URLs flotantes/mutables como /latest/
    let mutable_url_1 =
        "https://github.com/danimardo/NetworkBench/releases/latest/download/NetworkBench.exe";
    let mutable_url_2 = "https://example.com/downloads/latest.exe";
    let valid_url = "https://github.com/danimardo/NetworkBench/releases/download/v1.2.0/NetworkBench-Setup-v1.2.0.exe";

    assert!(validate_download_url(mutable_url_1, "1.2.0").is_err());
    assert!(validate_download_url(mutable_url_2, "1.2.0").is_err());
    assert!(validate_download_url(valid_url, "1.2.0").is_ok());

    // Falla si la versión no coincide con el tag de release
    assert!(validate_download_url(valid_url, "1.3.0").is_err());
}

#[test]
fn test_updater_rejects_missing_signature_or_platform() {
    let no_sig_json = r#"{
        "version": "1.2.0",
        "platforms": {
            "windows-x86_64": {
                "signature": "   ",
                "url": "https://github.com/danimardo/NetworkBench/releases/download/v1.2.0/NetworkBench-Setup-v1.2.0.exe"
            }
        }
    }"#;
    let res_no_sig = evaluate_manifest(no_sig_json, "1.0.0", "windows-x86_64", false);
    assert!(res_no_sig.is_err(), "Debe requerir firma criptográfica");

    let no_plat_json = r#"{
        "version": "1.2.0",
        "platforms": {
            "darwin-arm64": {
                "signature": "sig",
                "url": "https://github.com/danimardo/NetworkBench/releases/download/v1.2.0/NetworkBench.zip"
            }
        }
    }"#;
    let res_no_plat = evaluate_manifest(no_plat_json, "1.0.0", "windows-x86_64", false);
    assert!(
        res_no_plat.is_err(),
        "Debe rechazar si la plataforma solicitada no está en el manifiesto"
    );
}

#[test]
fn test_updater_deferred_during_active_session() {
    // Si hay una sesión activa de medición, NUNCA se interrumpe al usuario (FR-042)
    let manifest_json = r#"{
        "version": "2.0.0",
        "platforms": {
            "windows-x86_64": {
                "signature": "sig123",
                "url": "https://github.com/danimardo/NetworkBench/releases/download/v2.0.0/NetworkBench-Setup-v2.0.0.exe"
            }
        }
    }"#;

    let res = evaluate_manifest(manifest_json, "1.0.0", "windows-x86_64", true).unwrap();
    assert_eq!(
        res,
        UpdateStatus::DeferredDueToActiveSession,
        "La comprobación debe posponerse automáticamente si hay sesión activa"
    );
}

#[test]
fn test_updater_corrupted_or_invalid_json_manifest() {
    let invalid_json = "<html><body>404 Not Found</body></html>";
    let res = evaluate_manifest(invalid_json, "1.0.0", "windows-x86_64", false);
    assert!(
        res.is_err(),
        "Debe reportar error ante contenido HTML o JSON corrupto"
    );
}
