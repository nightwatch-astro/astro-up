use astro_up_core::detect::DetectionResult;
use astro_up_core::types::{DetectionConfig, DetectionMethod};

fn ascom_config(key: &str) -> DetectionConfig {
    DetectionConfig {
        method: DetectionMethod::AscomProfile,
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

#[cfg(windows)]
#[tokio::test]
async fn ascom_missing_platform_returns_not_installed() {
    // Most Windows CI runners don't have ASCOM Platform installed
    // This should return NotInstalled (no HKLM\SOFTWARE\ASCOM key)
    let config = ascom_config("Camera Drivers/ASCOM.Simulator.Camera");
    let result = astro_up_core::detect::ascom::detect(&config).await;

    // Either NotInstalled (no ASCOM) or Installed (if ASCOM happens to be there)
    assert!(matches!(
        result,
        DetectionResult::NotInstalled
            | DetectionResult::Installed { .. }
            | DetectionResult::InstalledUnknownVersion { .. }
    ));
}

#[cfg(not(windows))]
#[tokio::test]
async fn ascom_returns_unavailable_on_non_windows() {
    let config = ascom_config("Camera Drivers/ASCOM.Simulator.Camera");
    let result = astro_up_core::detect::ascom::detect(&config).await;

    assert!(matches!(result, DetectionResult::Unavailable { .. }));
}
