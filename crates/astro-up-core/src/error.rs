use crate::types::KnownExitCode;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("software not installed")]
    NotInstalled,

    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("provider '{provider}' unavailable: {cause}")]
    ProviderUnavailable {
        provider: String,
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("manifest '{id}' invalid: {reason}")]
    ManifestInvalid { id: String, reason: String },

    #[error("installer failed with exit code {exit_code}: {response}")]
    InstallerFailed {
        exit_code: i32,
        response: KnownExitCode,
    },

    #[error("elevation required")]
    ElevationRequired,

    #[error("reboot required")]
    RebootRequired,

    #[error("installer timed out after {timeout_secs}s")]
    InstallerTimeout { timeout_secs: u64 },

    #[error("installer busy")]
    InstallerBusy,

    #[error("package in use by {process_name}")]
    PackageInUse { process_name: String },

    #[error("already installed: {id} {version}")]
    AlreadyInstalled { id: String, version: String },

    #[error("missing dependency: {dep_id}")]
    MissingDependency { dep_id: String },

    #[error("version parse failed for '{raw}': {cause}")]
    VersionParseFailed {
        raw: String,
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("unsupported platform")]
    UnsupportedPlatform,

    #[error("not found: {input}")]
    NotFound { input: String },

    #[error("manual download required for {id}: {url} ({cause})")]
    ManualDownloadRequired {
        id: String,
        url: String,
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_are_readable() {
        let errors = vec![
            CoreError::NotInstalled,
            CoreError::ChecksumMismatch {
                expected: "abc123".into(),
                actual: "def456".into(),
            },
            CoreError::ManifestInvalid {
                id: "nina-app".into(),
                reason: "missing detection".into(),
            },
            CoreError::InstallerFailed {
                exit_code: 1,
                response: KnownExitCode::PackageInUse,
            },
            CoreError::ElevationRequired,
            CoreError::RebootRequired,
            CoreError::InstallerTimeout { timeout_secs: 600 },
            CoreError::InstallerBusy,
            CoreError::PackageInUse {
                process_name: "NINA.exe".into(),
            },
            CoreError::AlreadyInstalled {
                id: "nina-app".into(),
                version: "3.1.2".into(),
            },
            CoreError::MissingDependency {
                dep_id: "ascom-platform".into(),
            },
            CoreError::UnsupportedPlatform,
            CoreError::NotFound {
                input: "nonexistent".into(),
            },
        ];

        for err in &errors {
            let msg = err.to_string();
            assert!(!msg.is_empty(), "error message should not be empty");
            assert!(
                !msg.contains("CoreError"),
                "error message should not contain type name: {msg}"
            );
        }

        insta::assert_snapshot!(errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n"));
    }
}
