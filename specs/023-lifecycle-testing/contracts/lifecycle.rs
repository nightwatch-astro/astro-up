// Contract: Lifecycle Test Runner
// Location: crates/astro-up-core/src/lifecycle.rs (new module)

use crate::detect::discovery::DiscoveryResult;
use crate::types::{DetectionConfig, Software, Version};
use std::path::PathBuf;
use std::time::Duration;

/// Status of an individual lifecycle phase.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Pass,
    Fail,
    Skipped,
}

/// Result of a single lifecycle phase.
#[derive(Debug, Clone, Serialize)]
pub struct PhaseResult {
    pub phase: String,
    pub status: PhaseStatus,
    pub duration: Duration,
    pub exit_code: Option<i32>,
    pub logs: Vec<String>,
    pub warnings: Vec<String>,
}

/// Overall lifecycle test status.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    Pass,
    PartialPass, // Install/detect succeeded but uninstall/verify had warnings
    Fail,
}

/// Full lifecycle test report for one package.
#[derive(Debug, Clone, Serialize)]
pub struct LifecycleReport {
    pub package_id: String,
    pub version: String,
    pub phases: Vec<PhaseResult>,
    pub discovered_config: Option<DetectionConfig>,
    pub overall_status: LifecycleStatus,
}

/// Options for running a lifecycle test.
pub struct LifecycleOptions {
    pub manifest_path: PathBuf, // Root of manifests repo checkout
    pub package_id: String,
    pub version: Option<String>,    // None = resolve latest from versions/ dir
    pub install_dir: Option<PathBuf>, // Required for download_only
    pub dry_run: bool,
    pub timeout: Duration, // Default: 10 minutes
}

/// The lifecycle test runner.
pub struct LifecycleRunner {
    // Uses DownloadManager, InstallerService, DiscoveryScanner internally
}

impl LifecycleRunner {
    pub fn new() -> Self;

    /// Run the full lifecycle test for a package.
    /// Returns the report and optionally the discovered detection config as TOML.
    pub async fn run(&self, options: LifecycleOptions) -> Result<LifecycleReport, CoreError>;

    /// Resolve the latest version from versions/{package_id}/ directory.
    pub fn resolve_latest_version(manifest_path: &Path, package_id: &str) -> Result<Version, CoreError>;

    /// Resolve download URL from autoupdate template + version.
    pub fn resolve_download_url(software: &Software, version: &str) -> Result<String, CoreError>;

    /// Serialize a DetectionConfig to TOML string for PR creation.
    pub fn config_to_toml(config: &DetectionConfig) -> String;
}
