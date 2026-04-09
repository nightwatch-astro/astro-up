#![allow(clippy::unwrap_used, clippy::expect_used)]

use astro_up_core::detect::{self, DetectionResult, PathResolver};
use astro_up_core::types::{DetectionConfig, DetectionMethod};

fn registry_config(key: &str) -> DetectionConfig {
    DetectionConfig {
        method: DetectionMethod::Registry,
        registry_key: Some(key.into()),
        registry_value: None,
        file_path: None,
        version_regex: None,
        product_code: None,
        upgrade_code: None,
        inf_provider: None,
        device_class: None,
        inf_name: None,
        fallback: None,
    }
}

fn pe_config(path: &str) -> DetectionConfig {
    DetectionConfig {
        method: DetectionMethod::PeFile,
        file_path: Some(path.into()),
        registry_key: None,
        registry_value: None,
        version_regex: None,
        product_code: None,
        upgrade_code: None,
        inf_provider: None,
        device_class: None,
        inf_name: None,
        fallback: None,
    }
}

#[tokio::test]
async fn chain_pe_fallback_on_non_windows() {
    // On non-Windows, registry returns Unavailable, chain should fall through to PE
    let config = DetectionConfig {
        fallback: Some(Box::new(pe_config("tests/fixtures/test.exe"))),
        ..registry_config(
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\NonExistent",
        )
    };

    let resolver = PathResolver::new();
    let result = detect::run_chain(&config, &resolver, None, None).await;

    // On non-Windows: registry returns Unavailable (not on Windows), so chain continues to PE
    // PE should find version 3.2.1
    if cfg!(not(windows)) {
        // Registry returns Unavailable which is not "installed", so chain falls through
        match result {
            DetectionResult::Installed {
                version, method, ..
            } => {
                assert_eq!(version.raw, "3.2.1");
                assert_eq!(method, DetectionMethod::PeFile);
            }
            other => panic!("expected PE fallback to succeed, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn chain_stops_at_pe_success() {
    // PE succeeds on first try — no fallback needed
    let config = pe_config("tests/fixtures/test.exe");
    let resolver = PathResolver::new();
    let result = detect::run_chain(&config, &resolver, None, None).await;

    match result {
        DetectionResult::Installed {
            version, method, ..
        } => {
            assert_eq!(version.raw, "3.2.1");
            assert_eq!(method, DetectionMethod::PeFile);
        }
        other => panic!("expected Installed, got {other:?}"),
    }
}

#[tokio::test]
async fn chain_exhausted_returns_not_installed() {
    // PE with nonexistent file, no fallback
    let config = pe_config("tests/fixtures/nonexistent.exe");
    let resolver = PathResolver::new();
    let result = detect::run_chain(&config, &resolver, None, None).await;

    assert!(matches!(result, DetectionResult::NotInstalled));
}

#[tokio::test]
async fn registry_rejects_relative_key() {
    // registry_key without HKEY_ prefix should return Unavailable, not silently fail
    let config = registry_config("PHD 2_is1");
    let resolver = PathResolver::new();
    let result = detect::run_chain(&config, &resolver, None, None).await;

    match result {
        DetectionResult::Unavailable { reason } => {
            assert!(
                reason.contains("absolute path"),
                "expected absolute path error, got: {reason}"
            );
        }
        other => panic!("expected Unavailable for relative key, got {other:?}"),
    }
}

#[tokio::test]
async fn registry_rejects_relative_key_falls_through_to_pe() {
    // Chain should continue to PE fallback when registry key is invalid
    let config = DetectionConfig {
        fallback: Some(Box::new(pe_config("tests/fixtures/test.exe"))),
        ..registry_config("SOFTWARE\\Some\\Key")
    };

    let resolver = PathResolver::new();
    let result = detect::run_chain(&config, &resolver, None, None).await;

    // On non-Windows: registry returns Unavailable (bad key), chain falls to PE
    if cfg!(not(windows)) {
        match result {
            DetectionResult::Installed {
                version, method, ..
            } => {
                assert_eq!(version.raw, "3.2.1");
                assert_eq!(method, DetectionMethod::PeFile);
            }
            other => panic!("expected PE fallback after invalid registry key, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn chain_pe_uses_ledger_path_fallback() {
    // PE config with an unresolvable template, but ledger provides a valid path
    let config = pe_config("{nonexistent_token}/test.exe");
    let resolver = PathResolver::new();

    // Without ledger path: should return Unavailable (can't resolve template)
    let result = detect::run_chain(&config, &resolver, None, None).await;
    assert!(
        matches!(result, DetectionResult::Unavailable { .. }),
        "expected Unavailable without ledger path, got {result:?}"
    );

    // With ledger path pointing to real fixture: should find version
    let result = detect::run_chain(&config, &resolver, Some("tests/fixtures/test.exe"), None).await;
    match result {
        DetectionResult::Installed {
            version, method, ..
        } => {
            assert_eq!(version.raw, "3.2.1");
            assert_eq!(method, DetectionMethod::PeFile);
        }
        other => panic!("expected Installed from ledger path fallback, got {other:?}"),
    }
}
