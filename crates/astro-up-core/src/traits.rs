use std::path::Path;

use crate::error::CoreError;
use crate::release::Release;
use crate::types::{CheckverConfig, InstallConfig, InstallMethod};

/// Options for an install operation.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub asset_path: String,
    pub config: InstallConfig,
    pub quiet: bool,
}

/// Callback for download progress: (bytes_downloaded, total_bytes, speed_bytes_per_sec)
pub type ProgressCallback = Box<dyn Fn(u64, u64, f64) + Send>;

/// Options for a download operation.
pub struct DownloadOptions {
    pub on_progress: Option<ProgressCallback>,
    pub checksum: Option<String>,
    pub resume: bool,
}

/// Checks for the latest version of software from a remote source.
#[trait_variant::make(ProviderDyn: Send)]
pub trait Provider {
    fn name(&self) -> &str;
    async fn latest_release(&self, cfg: &CheckverConfig) -> Result<Release, CoreError>;
    async fn list_releases(
        &self,
        cfg: &CheckverConfig,
        limit: usize,
    ) -> Result<Vec<Release>, CoreError>;
}

/// Installs or updates software on the local system.
#[trait_variant::make(InstallerDyn: Send)]
pub trait Installer {
    async fn install(&self, opts: &InstallOptions) -> Result<(), CoreError>;
    fn supports(&self, method: &InstallMethod) -> bool;
}

/// Downloads files with progress reporting and verification.
#[trait_variant::make(DownloaderDyn: Send)]
pub trait Downloader {
    async fn download(
        &self,
        url: &str,
        dest: &Path,
        opts: &DownloadOptions,
    ) -> Result<(), CoreError>;
}

/// Backup result from a completed backup operation.
#[derive(Debug, Clone)]
pub struct BackupResult {
    pub software_id: String,
    pub timestamp: String,
    pub path: String,
    pub size: u64,
}

/// Entry in a backup list.
#[derive(Debug, Clone)]
pub struct BackupEntry {
    pub timestamp: String,
    pub path: String,
    pub size: u64,
}

/// Backs up and restores software configuration files.
#[trait_variant::make(BackupManagerDyn: Send)]
pub trait BackupManager {
    async fn backup(&self, software_id: &str, paths: &[String]) -> Result<BackupResult, CoreError>;
    async fn restore(&self, software_id: &str, timestamp: &str) -> Result<(), CoreError>;
    async fn list(&self, software_id: &str) -> Result<Vec<BackupEntry>, CoreError>;
    async fn prune(&self, software_id: &str, keep: usize) -> Result<(), CoreError>;
}

