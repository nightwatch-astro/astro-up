pub mod archive;
pub mod preview;
pub mod prune;
pub mod types;

use std::path::{Path, PathBuf};

use tracing::{info, instrument};

use crate::error::CoreError;
use crate::events::Event;
use crate::traits::BackupManager;

use self::types::{
    BackupListEntry, BackupMetadata, BackupRequest, FileChangeSummary, RestoreRequest,
};

/// Facade for backup and restore operations. Implements the BackupManager trait.
pub struct BackupService {
    backup_dir: PathBuf,
    retention: usize,
}

impl BackupService {
    pub fn new(backup_dir: PathBuf, retention: usize) -> Self {
        Self {
            backup_dir,
            retention,
        }
    }

    #[instrument(skip_all, fields(package = %request.package_id))]
    pub async fn backup(&self, request: &BackupRequest) -> Result<BackupMetadata, CoreError> {
        if request.config_paths.is_empty() {
            return Err(CoreError::Io(std::io::Error::other(
                "no backup paths configured for this package",
            )));
        }

        let _ = request.event_tx.send(Event::BackupStarted {
            id: request.package_id.clone(),
        });

        let metadata = archive::create_backup(request, &self.backup_dir).await?;

        let _ = request.event_tx.send(Event::BackupComplete {
            id: request.package_id.clone(),
        });

        // Auto-prune after backup
        if self.retention > 0 {
            let pruned =
                prune::prune_backups(&self.backup_dir, &request.package_id, self.retention).await?;
            if pruned > 0 {
                info!(package = %request.package_id, pruned, "pruned old backups");
            }
        }

        Ok(metadata)
    }

    #[instrument(skip_all, fields(archive = %request.archive_path.display()))]
    pub async fn restore(&self, request: &RestoreRequest) -> Result<(), CoreError> {
        let _ = request.event_tx.send(Event::RestoreStarted {
            id: request.archive_path.display().to_string(),
        });

        archive::restore(&request.archive_path, request.path_filter.as_deref()).await?;

        let _ = request.event_tx.send(Event::RestoreComplete {
            id: request.archive_path.display().to_string(),
        });

        Ok(())
    }

    pub async fn restore_preview(
        &self,
        archive_path: &Path,
    ) -> Result<FileChangeSummary, CoreError> {
        preview::restore_preview(archive_path).await
    }

    pub async fn list(&self, package_id: &str) -> Result<Vec<BackupListEntry>, CoreError> {
        prune::list_backups(&self.backup_dir, package_id).await
    }

    pub async fn prune(&self, package_id: &str, keep: usize) -> Result<u32, CoreError> {
        prune::prune_backups(&self.backup_dir, package_id, keep).await
    }
}

impl BackupManager for BackupService {
    async fn backup(&self, request: &BackupRequest) -> Result<BackupMetadata, CoreError> {
        self.backup(request).await
    }

    async fn restore(&self, request: &RestoreRequest) -> Result<(), CoreError> {
        self.restore(request).await
    }

    async fn restore_preview(&self, archive_path: &Path) -> Result<FileChangeSummary, CoreError> {
        self.restore_preview(archive_path).await
    }

    async fn list(&self, package_id: &str) -> Result<Vec<BackupListEntry>, CoreError> {
        self.list(package_id).await
    }

    async fn prune(&self, package_id: &str, keep: usize) -> Result<u32, CoreError> {
        self.prune(package_id, keep).await
    }
}
