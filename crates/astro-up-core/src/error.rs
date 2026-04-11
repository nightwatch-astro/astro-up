use crate::types::{CheckMethod, KnownExitCode};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
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

    #[error("upgrade denied for {package_id}")]
    UpgradeDenied { package_id: String },

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

    #[error("not found: {input}")]
    NotFound { input: String },

    #[error("manual download required for {id}: {url} ({cause})")]
    ManualDownloadRequired {
        id: String,
        url: String,
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("config validation failed:\n{0}")]
    ConfigValidation(#[from] garde::Report),

    #[error("unknown config key {key:?}, valid keys: {}", valid_keys.join(", "))]
    ConfigUnknownKey {
        key: String,
        valid_keys: Vec<String>,
    },

    #[error("config parse error for {key:?}: expected {expected}, got {got:?}")]
    ConfigParse {
        key: String,
        expected: String,
        got: String,
    },

    #[error("config store error: {0}")]
    ConfigStore(#[from] rusqlite::Error),

    #[error("catalog fetch failed: {reason}")]
    CatalogFetchFailed { reason: String },

    #[error(
        "catalog signature invalid — the downloaded catalog may be tampered with, try again later or report this issue"
    )]
    CatalogSignatureInvalid,

    #[error("catalog signature file missing — expected .minisig alongside catalog.db")]
    CatalogSignatureMissing,

    #[error(
        "catalog schema version {version} is not supported (expected {expected}) — please update astro-up"
    )]
    CatalogSchemaUnsupported { version: String, expected: String },

    #[error("no catalog available — check your network connection and try again")]
    CatalogNotAvailable,

    #[error("catalog file corrupted — will attempt to re-fetch")]
    CatalogCorrupted,

    #[error("another instance of astro-up is running (PID {pid})")]
    CatalogLocked { pid: u32 },

    #[error("orchestration engine already running (PID {pid})")]
    OrchestrationLocked { pid: u32 },

    #[error("invalid package ID {input:?}: {reason}")]
    InvalidPackageId { input: String, reason: String },

    #[error("download failed for {url}: HTTP {status} — {reason}")]
    DownloadFailed {
        url: String,
        status: u16,
        reason: String,
    },

    #[error("insufficient disk space: need {required} bytes, have {available} bytes")]
    DiskSpaceInsufficient { required: u64, available: u64 },

    #[error("download already in progress for {url}")]
    DownloadInProgress { url: String },

    #[error("failed to rename {from} to {to}: {cause}")]
    RenameFailed {
        from: String,
        to: String,
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("dependency cycle detected: {}", path.join(" -> "))]
    DependencyCycle { path: Vec<String> },

    #[error("database error: {0}")]
    Database(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("download cancelled")]
    Cancelled,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
