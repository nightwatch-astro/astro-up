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
        request: &crate::download::DownloadRequest,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<crate::download::DownloadResult, CoreError>;
}

/// Backs up and restores software configuration files.
/// (Updated by spec 013 — richer signatures with BackupMetadata, FileChangeSummary, etc.)
#[trait_variant::make(BackupManagerDyn: Send)]
pub trait BackupManager {
    async fn backup(
        &self,
        request: &crate::backup::types::BackupRequest,
    ) -> Result<crate::backup::types::BackupMetadata, CoreError>;
    async fn restore(
        &self,
        request: &crate::backup::types::RestoreRequest,
    ) -> Result<(), CoreError>;
    async fn restore_preview(
        &self,
        archive_path: &std::path::Path,
    ) -> Result<crate::backup::types::FileChangeSummary, CoreError>;
    async fn list(
        &self,
        package_id: &str,
    ) -> Result<Vec<crate::backup::types::BackupListEntry>, CoreError>;
    async fn prune(&self, package_id: &str, keep: usize) -> Result<u32, CoreError>;
}
