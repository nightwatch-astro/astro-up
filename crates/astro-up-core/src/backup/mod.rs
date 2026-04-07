pub mod archive;
pub mod preview;
pub mod prune;
pub mod types;

use std::path::{Path, PathBuf};

use tracing::{debug, info, instrument, warn};

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

        if let Err(e) = request.event_tx.send(Event::BackupStarted {
            id: request.package_id.clone(),
        }) {
            debug!("failed to send BackupStarted event: {e}");
        }

        let metadata = archive::create_backup(request, &self.backup_dir).await?;

        if let Err(e) = request.event_tx.send(Event::BackupComplete {
            id: request.package_id.clone(),
        }) {
            debug!("failed to send BackupComplete event: {e}");
        }

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
        // FR-009: Warn on version mismatch
        if let Some(current_version) = &request.current_version {
            if let Ok(metadata) = archive::read_metadata(&request.archive_path).await {
                if metadata.version != *current_version {
                    warn!(
                        backup_version = %metadata.version.raw,
                        current_version = %current_version.raw,
                        "restoring backup from a different version"
                    );
                }
            }
        }

        if let Err(e) = request.event_tx.send(Event::RestoreStarted {
            id: request.archive_path.display().to_string(),
        }) {
            debug!("failed to send RestoreStarted event: {e}");
        }

        archive::restore(&request.archive_path, request.path_filter.as_deref()).await?;

        if let Err(e) = request.event_tx.send(Event::RestoreComplete {
            id: request.archive_path.display().to_string(),
        }) {
            debug!("failed to send RestoreComplete event: {e}");
        }

        Ok(())
    }

    #[instrument(skip_all, fields(archive = %archive_path.display()))]
    pub async fn restore_preview(
        &self,
        archive_path: &Path,
    ) -> Result<FileChangeSummary, CoreError> {
        preview::restore_preview(archive_path).await
    }

    #[instrument(skip_all, fields(package = %package_id))]
    pub async fn list(&self, package_id: &str) -> Result<Vec<BackupListEntry>, CoreError> {
        prune::list_backups(&self.backup_dir, package_id).await
    }

    #[instrument(skip_all, fields(package = %package_id, keep))]
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::Version;

    fn make_service(backup_dir: PathBuf) -> BackupService {
        BackupService::new(backup_dir, 0)
    }

    #[tokio::test]
    async fn backup_rejects_empty_config_paths() {
        let backup_dir = tempfile::tempdir().unwrap();
        let service = make_service(backup_dir.path().to_path_buf());
        let (tx, _rx) = tokio::sync::broadcast::channel(16);

        let request = BackupRequest {
            package_id: "test-pkg".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![],
            event_tx: tx,
        };

        let err = service.backup(&request).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("no backup paths configured"),
            "expected empty config_paths error, got: {msg}"
        );
    }

    #[tokio::test]
    async fn restore_warns_on_version_mismatch() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("Config")).unwrap();
        std::fs::write(src.path().join("Config/a.txt"), "data").unwrap();

        let service = make_service(backup_dir.path().to_path_buf());
        let (tx, _rx) = tokio::sync::broadcast::channel(16);

        let request = BackupRequest {
            package_id: "test-pkg".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Config")],
            event_tx: tx,
        };
        service.backup(&request).await.unwrap();

        let archive = std::fs::read_dir(backup_dir.path().join("test-pkg"))
            .unwrap()
            .find_map(Result::ok)
            .unwrap()
            .path();

        // Restore with a different current_version — should succeed with warning (not error)
        let (tx2, _rx2) = tokio::sync::broadcast::channel(16);
        let restore_req = RestoreRequest {
            archive_path: archive,
            path_filter: None,
            current_version: Some(Version::parse("2.0.0")),
            event_tx: tx2,
        };
        // Should not error — version mismatch is a warning, not a failure
        service.restore(&restore_req).await.unwrap();
    }

    #[tokio::test]
    async fn restore_creates_missing_target_directory() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("Config")).unwrap();
        std::fs::write(src.path().join("Config/a.txt"), "data").unwrap();

        let service = make_service(backup_dir.path().to_path_buf());
        let (tx, _rx) = tokio::sync::broadcast::channel(16);

        let request = BackupRequest {
            package_id: "test-pkg".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Config")],
            event_tx: tx,
        };
        service.backup(&request).await.unwrap();

        let archive = std::fs::read_dir(backup_dir.path().join("test-pkg"))
            .unwrap()
            .find_map(Result::ok)
            .unwrap()
            .path();

        // Delete the entire Config directory
        std::fs::remove_dir_all(src.path().join("Config")).unwrap();
        assert!(!src.path().join("Config").exists());

        // Restore should recreate the directory
        let (tx2, _rx2) = tokio::sync::broadcast::channel(16);
        let restore_req = RestoreRequest {
            archive_path: archive,
            path_filter: None,
            current_version: None,
            event_tx: tx2,
        };
        service.restore(&restore_req).await.unwrap();

        assert!(src.path().join("Config/a.txt").exists());
        let content = std::fs::read_to_string(src.path().join("Config/a.txt")).unwrap();
        assert_eq!(content, "data");
    }
}
