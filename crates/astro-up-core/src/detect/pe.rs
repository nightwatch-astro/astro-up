use crate::detect::{DetectionResult, PathResolver};
use crate::types::{DetectionConfig, DetectionMethod, Version};

/// Detect version from PE file headers (VS_FIXEDFILEINFO).
///
/// Cross-platform — pelite works on Linux/macOS too.
/// Resolves file path via PathResolver, then falls back to `ledger_path`
/// (the install ledger's recorded executable path) if template resolution fails.
pub async fn detect(
    config: &DetectionConfig,
    resolver: &PathResolver,
    ledger_path: Option<&str>,
) -> DetectionResult {
    let path = match &config.file_path {
        Some(template) => match resolver.expand(template) {
            Some(resolved) => resolved,
            None => match ledger_path {
                Some(lp) => lp.to_string(),
                None => {
                    return DetectionResult::Unavailable {
                        reason: format!("cannot resolve path template: {template}"),
                    };
                }
            },
        },
        None => match ledger_path {
            Some(lp) => lp.to_string(),
            None => return DetectionResult::NotInstalled,
        },
    };

    // PE parsing is blocking I/O — run on blocking thread
    match tokio::task::spawn_blocking(move || read_pe_version(&path)).await {
        Ok(result) => result,
        Err(e) => DetectionResult::Unavailable {
            reason: format!("PE detection task failed: {e}"),
        },
    }
}

fn read_pe_version(path: &str) -> DetectionResult {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return DetectionResult::NotInstalled,
        Err(e) => {
            return DetectionResult::Unavailable {
                reason: format!("cannot read PE file: {e}"),
            };
        }
    };

    let pe = match pelite::PeFile::from_bytes(&data) {
        Ok(pe) => pe,
        Err(e) => {
            return DetectionResult::Unavailable {
                reason: format!("invalid PE file: {e}"),
            };
        }
    };

    let Ok(resources) = pe.resources() else {
        return DetectionResult::InstalledUnknownVersion {
            method: DetectionMethod::PeFile,
            install_path: Some(path.to_string()),
        };
    };

    let Ok(version_info) = resources.version_info() else {
        return DetectionResult::InstalledUnknownVersion {
            method: DetectionMethod::PeFile,
            install_path: Some(path.to_string()),
        };
    };

    // Prefer VS_FIXEDFILEINFO.dwFileVersion (binary, reliable)
    if let Some(fixed) = version_info.fixed() {
        let v = fixed.dwFileVersion;
        let version_str = format!("{}.{}.{}", v.Major, v.Minor, v.Patch);
        return DetectionResult::Installed {
            version: Version::parse(&version_str),
            method: DetectionMethod::PeFile,
            install_path: Some(path.to_string()),
        };
    }

    // Fall back to string "FileVersion" if fixed info is absent
    let translations = version_info.translation();
    if let Some(&lang) = translations.first() {
        if let Some(file_version) = version_info.value(lang, "FileVersion") {
            let trimmed = file_version.trim_end_matches('\0').trim();
            if !trimmed.is_empty() {
                return DetectionResult::Installed {
                    version: Version::parse(trimmed),
                    method: DetectionMethod::PeFile,
                    install_path: Some(path.to_string()),
                };
            }
        }
    }

    DetectionResult::InstalledUnknownVersion {
        method: DetectionMethod::PeFile,
        install_path: Some(path.to_string()),
    }
}

/// Synchronous version — useful for testing without a tokio runtime.
pub fn read_pe_version_sync(path: &str) -> DetectionResult {
    read_pe_version(path)
}
