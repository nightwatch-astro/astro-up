use tracing::{debug, trace};

use crate::detect::{DetectionResult, PathResolver};
use crate::types::{DetectionConfig, DetectionMethod, Version};

/// Detect version from PE file headers (VS_FIXEDFILEINFO).
///
/// Cross-platform — pelite works on Linux/macOS too.
///
/// Resolution order:
/// 1. Windows Search index (instant, covers any install location)
/// 2. Path template expansion (both Program Files dirs)
/// 3. Ledger path fallback (install ledger's recorded executable path)
pub async fn detect(
    config: &DetectionConfig,
    resolver: &PathResolver,
    ledger_path: Option<&str>,
) -> DetectionResult {
    debug!(method = "pe", file_path = ?config.file_path, "detect_pe path resolution");
    // Build list of candidate paths to try
    let mut candidates: Vec<String> = Vec::new();

    if let Some(template) = &config.file_path {
        // 1. Try Windows Search index first (extract filename from template)
        #[cfg(windows)]
        if let Some(fname) = std::path::Path::new(template)
            .file_name()
            .and_then(|f| f.to_str())
        {
            match super::search::find_file(fname) {
                Ok(Some(found_path)) => {
                    tracing::debug!(
                        filename = %fname,
                        found = %found_path,
                        "file found via Windows Search"
                    );
                    candidates.push(found_path);
                }
                Ok(None) => {
                    tracing::debug!(filename = %fname, "not found in Windows Search index");
                }
                Err(e) => {
                    tracing::debug!(
                        filename = %fname,
                        error = %e,
                        "Windows Search unavailable, falling back to path expansion"
                    );
                }
            }
        }

        // 2. Template expansion (both Program Files dirs)
        candidates.extend(resolver.expand_all(template));
    }

    // 3. Ledger path fallback
    if let Some(lp) = ledger_path {
        if !candidates.iter().any(|c| c == lp) {
            candidates.push(lp.to_string());
        }
    }

    if candidates.is_empty() {
        return if config.file_path.is_some() {
            DetectionResult::Unavailable {
                reason: format!(
                    "cannot resolve path template: {}",
                    config.file_path.as_deref().unwrap_or("?")
                ),
            }
        } else {
            DetectionResult::NotInstalled
        };
    }

    // Try each candidate path
    let result = tokio::task::spawn_blocking(move || {
        for path in &candidates {
            let result = read_pe_version(path);
            match &result {
                DetectionResult::Installed { .. }
                | DetectionResult::InstalledUnknownVersion { .. } => return result,
                DetectionResult::NotInstalled | DetectionResult::Unavailable { .. } => {}
            }
        }
        DetectionResult::NotInstalled
    })
    .await;

    match result {
        Ok(r) => r,
        Err(e) => DetectionResult::Unavailable {
            reason: format!("PE detection task failed: {e}"),
        },
    }
}

fn read_pe_version(path: &str) -> DetectionResult {
    trace!(method = "pe", %path, "reading PE file");
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return DetectionResult::NotInstalled,
        Err(e) => {
            return DetectionResult::Unavailable {
                reason: format!("cannot read PE file: {e}"),
            };
        }
    };

    trace!(method = "pe", %path, bytes = data.len(), "parsing PE headers");
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
