use crate::detect::DetectionResult;
use crate::types::DetectionConfig;
#[cfg(windows)]
use crate::types::{DetectionMethod, Version};

/// Detect installed software via Windows uninstall registry keys.
///
/// Checks HKLM + HKCU, both 64-bit and 32-bit (WOW6432Node) views.
/// Reads the registry value specified in `config.registry_value` (default: DisplayVersion).
pub async fn detect(config: &DetectionConfig) -> DetectionResult {
    #[cfg(windows)]
    {
        detect_windows(config)
    }
    #[cfg(not(windows))]
    {
        let _ = config;
        DetectionResult::Unavailable {
            reason: "registry detection requires Windows".into(),
        }
    }
}

#[cfg(windows)]
fn detect_windows(config: &DetectionConfig) -> DetectionResult {
    use winreg::enums::*;
    use winreg::RegKey;

    let Some(ref key_path) = config.registry_key else {
        return DetectionResult::NotInstalled;
    };

    let value_name = config
        .registry_value
        .as_deref()
        .unwrap_or("DisplayVersion");

    // Search order: HKLM 64-bit, HKLM 32-bit, HKCU
    let searches = [
        (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_32KEY),
        (HKEY_CURRENT_USER, KEY_READ),
    ];

    for (hive, flags) in searches {
        let root = RegKey::predef(hive);
        let uninstall_path = format!(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{}",
            key_path
        );

        let subkey = match root.open_subkey_with_flags(&uninstall_path, flags) {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return DetectionResult::Unavailable {
                    reason: format!("permission denied reading registry key: {uninstall_path}"),
                };
            }
            Err(_) => continue, // key not found, try next search
        };

        match subkey.get_value::<String, _>(value_name) {
            Ok(version_str) if !version_str.trim().is_empty() => {
                return DetectionResult::Installed {
                    version: Version::parse(version_str.trim()),
                    method: DetectionMethod::Registry,
                };
            }
            Ok(_) => {
                // Value exists but is empty
                return DetectionResult::InstalledUnknownVersion {
                    method: DetectionMethod::Registry,
                };
            }
            Err(_) => {
                // Key exists but value missing — still installed
                return DetectionResult::InstalledUnknownVersion {
                    method: DetectionMethod::Registry,
                };
            }
        }
    }

    DetectionResult::NotInstalled
}
