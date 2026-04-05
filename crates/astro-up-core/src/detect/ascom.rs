use crate::detect::DetectionResult;
use crate::types::DetectionConfig;
#[cfg(windows)]
use crate::types::{DetectionMethod, Version};

/// Detect ASCOM drivers via ASCOM Profile registry keys.
///
/// Requires ASCOM Platform 7+. Reads driver registration under
/// HKLM\SOFTWARE\ASCOM\{DeviceType}\{DriverId}.
pub async fn detect(config: &DetectionConfig) -> DetectionResult {
    #[cfg(windows)]
    {
        detect_windows(config)
    }
    #[cfg(not(windows))]
    {
        let _ = config;
        DetectionResult::Unavailable {
            reason: "ASCOM detection requires Windows".into(),
        }
    }
}

#[cfg(windows)]
fn detect_windows(config: &DetectionConfig) -> DetectionResult {
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    // Check ASCOM Platform is installed and version >= 7
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(platform_key) = hklm.open_subkey(r"SOFTWARE\ASCOM") else {
        return DetectionResult::NotInstalled;
    };

    if let Ok(version_str) = platform_key.get_value::<String, _>("PlatformVersion") {
        if let Some(major) = version_str
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
        {
            if major < 7 {
                return DetectionResult::Unavailable {
                    reason: format!("ASCOM Platform {version_str} < 7 (unsupported)"),
                };
            }
        }
    }

    // Look for the specific driver via registry_key (e.g., "Camera Drivers/ASCOM.Simulator.Camera")
    let Some(ref key_path) = config.registry_key else {
        return DetectionResult::NotInstalled;
    };

    let driver_path = format!(r"SOFTWARE\ASCOM\{key_path}");
    match hklm.open_subkey(&driver_path) {
        Ok(subkey) => {
            // Try to read a version value
            if let Ok(ver) = subkey.get_value::<String, _>("Version") {
                if !ver.trim().is_empty() {
                    return DetectionResult::Installed {
                        version: Version::parse(ver.trim()),
                        method: DetectionMethod::AscomProfile,
                        install_path: None,
                    };
                }
            }
            DetectionResult::InstalledUnknownVersion {
                method: DetectionMethod::AscomProfile,
                install_path: None,
            }
        }
        Err(_) => DetectionResult::NotInstalled,
    }
}
