use crate::types::{CheckMethod, KnownExitCode};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("software not installed")]
    NotInstalled,

    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("provider '{provider}' unavailable: {cause}")]
    ProviderUnavailable {
        provider: CheckMethod,
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
