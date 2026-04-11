use tracing::trace;

use crate::detect::DetectionResult;
use crate::types::DetectionConfig;
#[cfg(windows)]
use crate::types::{DetectionMethod, Version};

/// Detect installed software via Windows registry keys.
///
/// `registry_key` should be an absolute path starting with `HKEY_LOCAL_MACHINE\`
/// or `HKEY_CURRENT_USER\`. `WOW6432Node` segments are stripped — the WOW64
/// registry flags handle 32/64-bit redirection transparently.
///
/// For backward compatibility, bare Uninstall subkey names (e.g., `"NINA 2_is1"`)
/// are auto-prefixed with the standard Uninstall path and searched in both
/// HKLM and HKCU hives.
///
/// Reads the value named in `config.registry_value` (default: `DisplayVersion`).
#[tracing::instrument(skip_all)]
pub async fn detect(config: &DetectionConfig) -> DetectionResult {
    trace!(method = "registry", registry_key = ?config.registry_key, "detect_registry entry");
    #[cfg(windows)]
    {
        detect_windows(config)
    }
    #[cfg(not(windows))]
    {
        if let Some(ref key) = config.registry_key {
            // Accept both absolute paths and bare subkey names on non-Windows
            // (returns Unavailable regardless, but validates format for diagnostics)
            if !key.starts_with(r"HKEY_LOCAL_MACHINE\")
                && !key.starts_with(r"HKEY_CURRENT_USER\")
                && !is_bare_subkey_name(key)
            {
                tracing::warn!(
                    method = "registry",
                    key = %key,
                    "registry_key is neither an absolute path nor a bare subkey name"
                );
            }
        }
        DetectionResult::Unavailable {
            reason: "registry detection requires Windows".into(),
        }
    }
}

/// Check if a registry key string looks like a bare Uninstall subkey name
/// (e.g., "NINA 2_is1") rather than an absolute registry path.
fn is_bare_subkey_name(key: &str) -> bool {
    !key.starts_with(r"HKEY_") && !key.starts_with(r"SOFTWARE\")
}

#[cfg(windows)]
fn detect_windows(config: &DetectionConfig) -> DetectionResult {
    use tracing::debug;
    use winreg::RegKey;
    use winreg::enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    };

    let Some(ref key_path) = config.registry_key else {
        return DetectionResult::NotInstalled;
    };

    let value_name = config.registry_value.as_deref().unwrap_or("DisplayVersion");

    // Parse absolute registry path into (hive, subkey).
    // Strip WOW6432Node — WOW64 flags handle redirection transparently.
    //
    // For backward compatibility, bare subkey names (e.g., "NINA 2_is1") are
    // auto-prefixed with the standard Uninstall path and searched in all hives.
    let (hive_searches, subkey_path) =
        if let Some(rest) = key_path.strip_prefix(r"HKEY_LOCAL_MACHINE\") {
            let normalized = rest.replace(r"WOW6432Node\", "");
            (
                vec![
                    (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_64KEY),
                    (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_32KEY),
                ],
                normalized,
            )
        } else if let Some(rest) = key_path.strip_prefix(r"HKEY_CURRENT_USER\") {
            let normalized = rest.replace(r"WOW6432Node\", "");
            (vec![(HKEY_CURRENT_USER, KEY_READ)], normalized)
        } else if is_bare_subkey_name(key_path) {
            // Bare subkey name — assume it's under the standard Uninstall path.
            // Search HKLM (64-bit, 32-bit) and HKCU.
            debug!(
                method = "registry",
                key = %key_path,
                "bare subkey name detected, auto-prefixing with Uninstall path"
            );
            let subkey = format!(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{key_path}"
            );
            (
                vec![
                    (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_64KEY),
                    (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_32KEY),
                    (HKEY_CURRENT_USER, KEY_READ),
                ],
                subkey,
            )
        } else {
            return DetectionResult::Unavailable {
                reason: format!(
                    "registry_key must be an absolute path starting with \
                     HKEY_LOCAL_MACHINE\\ or HKEY_CURRENT_USER\\, got: {key_path}"
                ),
            };
        };

    for (hive, flags) in &hive_searches {
        let root = RegKey::predef(*hive);

        let subkey = match root.open_subkey_with_flags(&subkey_path, *flags) {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return DetectionResult::Unavailable {
                    reason: format!("permission denied reading registry key: {subkey_path}"),
                };
            }
            Err(_) => continue, // key not found, try next search
        };

        // Extract install location from registry (used for backup path resolution)
        let install_path: Option<String> = subkey
            .get_value::<String, _>("InstallLocation")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string());

        match subkey.get_value::<String, _>(value_name) {
            Ok(version_str) if !version_str.trim().is_empty() => {
                let version = Version::parse(version_str.trim());
                debug!(method = "registry", ?version, key = %subkey_path, "registry version parsed");
                return DetectionResult::Installed {
                    version,
                    method: DetectionMethod::Registry,
                    install_path,
                };
            }
            Ok(_) => {
                return DetectionResult::InstalledUnknownVersion {
                    method: DetectionMethod::Registry,
                    install_path,
                };
            }
            Err(_) => {
                return DetectionResult::InstalledUnknownVersion {
                    method: DetectionMethod::Registry,
                    install_path,
                };
            }
        }
    }

    DetectionResult::NotInstalled
}
