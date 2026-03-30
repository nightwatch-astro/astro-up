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
        ..registry_config("NonExistent")
    };

    let resolver = PathResolver::new();
    let result = detect::run_chain(&config, &resolver).await;

    // On non-Windows: registry returns Unavailable (not installed-like), so chain continues to PE
    // PE should find version 3.2.1
    if cfg!(not(windows)) {
        // Registry returns Unavailable which is not "installed", so chain falls through
        match result {
            DetectionResult::Installed { version, method } => {
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
    let result = detect::run_chain(&config, &resolver).await;

    match result {
        DetectionResult::Installed { version, method } => {
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
    let result = detect::run_chain(&config, &resolver).await;

    assert!(matches!(result, DetectionResult::NotInstalled));
}
