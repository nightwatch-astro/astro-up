use std::path::Path;

use crate::detect::{DetectionResult, PathResolver};
use crate::types::{DetectionConfig, DetectionMethod, Version};

/// Check if a file exists at the resolved path. Returns InstalledUnknownVersion or NotInstalled.
pub async fn detect_exists(config: &DetectionConfig, resolver: &PathResolver) -> DetectionResult {
    let Some(ref template) = config.file_path else {
        return DetectionResult::NotInstalled;
    };

    let Some(path) = resolver.expand(template) else {
        return DetectionResult::Unavailable {
            reason: format!("cannot resolve path template: {template}"),
        };
    };

    if Path::new(&path).exists() {
        DetectionResult::InstalledUnknownVersion {
            method: DetectionMethod::FileExists,
            install_path: Some(path),
        }
    } else {
        DetectionResult::NotInstalled
    }
}

/// Read a config/manifest file and extract version via regex capture group 1.
pub async fn detect_config(config: &DetectionConfig, resolver: &PathResolver) -> DetectionResult {
    let Some(ref template) = config.file_path else {
        return DetectionResult::NotInstalled;
    };
    let Some(ref pattern) = config.version_regex else {
        return DetectionResult::Unavailable {
            reason: "config_file method requires version_regex".into(),
        };
    };

    let Some(path) = resolver.expand(template) else {
        return DetectionResult::Unavailable {
            reason: format!("cannot resolve path template: {template}"),
        };
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return DetectionResult::NotInstalled;
        }
        Err(e) => {
            return DetectionResult::Unavailable {
                reason: format!("cannot read config file: {e}"),
            };
        }
    };

    let re = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            return DetectionResult::Unavailable {
                reason: format!("invalid version_regex: {e}"),
            };
        }
    };

    match re.captures(&content).and_then(|caps| caps.get(1)) {
        Some(m) => DetectionResult::Installed {
            version: Version::parse(m.as_str()),
            method: DetectionMethod::ConfigFile,
            install_path: Some(path),
        },
        None => DetectionResult::InstalledUnknownVersion {
            method: DetectionMethod::ConfigFile,
            install_path: Some(path),
        },
    }
}
