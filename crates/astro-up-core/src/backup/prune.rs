use std::fs;
use std::io;
use std::path::Path;

use tracing::{info, warn};

use crate::backup::archive::read_metadata_sync;
use crate::backup::types::BackupListEntry;
use crate::error::CoreError;

/// Lists available backups for a package, sorted by date descending (newest first).
pub async fn list_backups(
    backup_dir: &Path,
    package_id: &str,
) -> Result<Vec<BackupListEntry>, CoreError> {
    let package_dir = backup_dir.join(package_id);
    if !package_dir.exists() {
        return Ok(Vec::new());
    }

    let package_dir_clone = package_dir.clone();
    tokio::task::spawn_blocking(move || list_backups_sync(&package_dir_clone))
        .await
        .map_err(|e| CoreError::Io(io::Error::other(e)))?
}

fn list_backups_sync(package_dir: &Path) -> Result<Vec<BackupListEntry>, CoreError> {
    let mut entries = Vec::new();

    for dir_entry in fs::read_dir(package_dir)? {
        let dir_entry = dir_entry?;
        let path = dir_entry.path();

        if path.extension().is_some_and(|ext| ext == "zip") {
            match read_metadata_sync(&path) {
                Ok(metadata) => {
                    entries.push(BackupListEntry {
                        archive_path: path,
                        package_id: metadata.package_id,
                        version: metadata.version,
                        created_at: metadata.created_at,
                        file_count: metadata.file_count,
                        total_size: metadata.total_size,
                    });
                }
                Err(e) => {
                    warn!(archive = %path.display(), error = %e, "skipping corrupt archive");
                }
            }
        }
    }

    // Sort by date descending (newest first)
    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(entries)
}

/// Deletes old backups beyond the retention count.
/// Returns the number of archives deleted.
/// A `keep` of 0 means unlimited — no pruning.
pub async fn prune_backups(
    backup_dir: &Path,
    package_id: &str,
    keep: usize,
) -> Result<u32, CoreError> {
    if keep == 0 {
        return Ok(0); // Unlimited retention
    }

    let entries = list_backups(backup_dir, package_id).await?;

    if entries.len() <= keep {
        return Ok(0);
    }

    let to_delete = &entries[keep..];
    let mut deleted = 0u32;

    for entry in to_delete {
        match tokio::fs::remove_file(&entry.archive_path).await {
            Ok(()) => {
                info!(archive = %entry.archive_path.display(), "pruned old backup");
                deleted += 1;
            }
            Err(e) => {
                warn!(archive = %entry.archive_path.display(), error = %e, "failed to prune backup");
            }
        }
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::archive::create_backup;
    use crate::backup::types::BackupRequest;
    use crate::types::Version;

    async fn create_test_backup(src: &Path, backup_dir: &Path, package_id: &str, version: &str) {
        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: package_id.into(),
            version: Version::parse(version),
            config_paths: vec![src.to_path_buf()],
            event_tx: tx,
        };
        create_backup(&request, backup_dir).await.unwrap();
        // Small delay to ensure unique timestamps
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn list_returns_sorted_by_date() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("config.toml"), "data").unwrap();

        create_test_backup(src.path(), backup_dir.path(), "test-pkg", "1.0.0").await;
        create_test_backup(src.path(), backup_dir.path(), "test-pkg", "1.1.0").await;
        create_test_backup(src.path(), backup_dir.path(), "test-pkg", "1.2.0").await;

        let entries = list_backups(backup_dir.path(), "test-pkg").await.unwrap();
        assert_eq!(entries.len(), 3);
        // Newest first
        assert!(entries[0].created_at >= entries[1].created_at);
        assert!(entries[1].created_at >= entries[2].created_at);
    }

    #[tokio::test]
    async fn prune_keeps_n_newest() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("config.toml"), "data").unwrap();

        for i in 0..5 {
            create_test_backup(
                src.path(),
                backup_dir.path(),
                "test-pkg",
                &format!("1.{i}.0"),
            )
            .await;
        }

        let deleted = prune_backups(backup_dir.path(), "test-pkg", 3)
            .await
            .unwrap();
        assert_eq!(deleted, 2);

        let remaining = list_backups(backup_dir.path(), "test-pkg").await.unwrap();
        assert_eq!(remaining.len(), 3);
    }

    #[tokio::test]
    async fn prune_zero_means_unlimited() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("config.toml"), "data").unwrap();

        for i in 0..3 {
            create_test_backup(
                src.path(),
                backup_dir.path(),
                "test-pkg",
                &format!("1.{i}.0"),
            )
            .await;
        }

        let deleted = prune_backups(backup_dir.path(), "test-pkg", 0)
            .await
            .unwrap();
        assert_eq!(deleted, 0);

        let remaining = list_backups(backup_dir.path(), "test-pkg").await.unwrap();
        assert_eq!(remaining.len(), 3);
    }

    #[tokio::test]
    async fn prune_fewer_than_retention() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("config.toml"), "data").unwrap();

        create_test_backup(src.path(), backup_dir.path(), "test-pkg", "1.0.0").await;

        let deleted = prune_backups(backup_dir.path(), "test-pkg", 5)
            .await
            .unwrap();
        assert_eq!(deleted, 0);
    }

    #[tokio::test]
    async fn list_empty_package() {
        let backup_dir = tempfile::tempdir().unwrap();
        let entries = list_backups(backup_dir.path(), "nonexistent")
            .await
            .unwrap();
        assert!(entries.is_empty());
    }
}
