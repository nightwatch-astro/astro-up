use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::backup::archive::resolve_dir_names;
use crate::backup::types::{BackupMetadata, FileChangeSummary};
use crate::error::CoreError;

/// Generates a preview of what a restore would change, without modifying files.
///
/// Reads file_hashes from the archive's metadata.json, hashes the on-disk files,
/// and compares to determine which files would be overwritten, unchanged, or new.
pub async fn restore_preview(archive_path: &Path) -> Result<FileChangeSummary, CoreError> {
    let archive_path = archive_path.to_path_buf();

    tokio::task::spawn_blocking(move || restore_preview_sync(&archive_path))
        .await
        .map_err(|e| CoreError::Io(io::Error::other(e)))?
}

fn restore_preview_sync(archive_path: &Path) -> Result<FileChangeSummary, CoreError> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| CoreError::Io(io::Error::other(e)))?;

    // Read metadata.json
    let metadata: BackupMetadata = {
        let mut entry = archive
            .by_name("metadata.json")
            .map_err(|e| CoreError::Io(io::Error::other(e)))?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        serde_json::from_str(&buf)?
    };

    // Build dir_name -> original_path mapping
    let dir_names = resolve_dir_names(&metadata.paths);
    let dir_to_path: HashMap<&str, &Path> = dir_names
        .iter()
        .zip(metadata.paths.iter())
        .map(|(name, path)| (name.as_str(), path.as_path()))
        .collect();

    let mut summary = FileChangeSummary::default();

    for (archive_entry_path, expected_hash) in &metadata.file_hashes {
        // Resolve archive path to on-disk path
        let on_disk = resolve_on_disk_path(archive_entry_path, &dir_to_path);
        let Some(on_disk) = on_disk else {
            summary.new_files.push(archive_entry_path.clone());
            continue;
        };

        if !on_disk.exists() {
            summary.new_files.push(archive_entry_path.clone());
            continue;
        }

        // Hash the on-disk file
        let actual_hash = hash_file(&on_disk)?;
        if actual_hash == *expected_hash {
            summary.unchanged.push(archive_entry_path.clone());
        } else {
            summary.overwritten.push(archive_entry_path.clone());
        }
    }

    // Files on disk in the config paths but NOT in the archive = missing
    // (We don't scan the full disk tree here — missing is only meaningful
    // if the caller wants to know what won't be touched by the restore)

    Ok(summary)
}

fn resolve_on_disk_path(
    archive_entry_path: &str,
    dir_to_path: &HashMap<&str, &Path>,
) -> Option<std::path::PathBuf> {
    let slash_pos = archive_entry_path.find('/')?;
    let dir_name = &archive_entry_path[..slash_pos];
    let relative = &archive_entry_path[slash_pos + 1..];
    if relative.is_empty() {
        return None;
    }
    let original_path = dir_to_path.get(dir_name)?;
    Some(original_path.join(relative))
}

fn hash_file(path: &Path) -> Result<String, CoreError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::archive::create_backup;
    use crate::backup::types::BackupRequest;
    use crate::types::Version;

    #[tokio::test]
    async fn preview_detects_changed_file() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("Config")).unwrap();
        std::fs::write(src.path().join("Config/settings.toml"), "original").unwrap();

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "test".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Config")],
            event_tx: tx,
        };

        create_backup(&request, backup_dir.path()).await.unwrap();

        // Modify the file
        std::fs::write(src.path().join("Config/settings.toml"), "modified").unwrap();

        let archive = std::fs::read_dir(backup_dir.path().join("test"))
            .unwrap()
            .filter_map(|e| e.ok())
            .next()
            .unwrap()
            .path();

        let summary = restore_preview(&archive).await.unwrap();
        assert_eq!(summary.overwritten.len(), 1);
        assert_eq!(summary.unchanged.len(), 0);
        assert_eq!(summary.new_files.len(), 0);
    }

    #[tokio::test]
    async fn preview_detects_unchanged_file() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("Config")).unwrap();
        std::fs::write(src.path().join("Config/settings.toml"), "same").unwrap();

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "test".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Config")],
            event_tx: tx,
        };

        create_backup(&request, backup_dir.path()).await.unwrap();
        // Don't modify — file stays the same

        let archive = std::fs::read_dir(backup_dir.path().join("test"))
            .unwrap()
            .filter_map(|e| e.ok())
            .next()
            .unwrap()
            .path();

        let summary = restore_preview(&archive).await.unwrap();
        assert_eq!(summary.unchanged.len(), 1);
        assert_eq!(summary.overwritten.len(), 0);
    }

    #[tokio::test]
    async fn preview_detects_new_file() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("Config")).unwrap();
        std::fs::write(src.path().join("Config/settings.toml"), "data").unwrap();

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "test".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Config")],
            event_tx: tx,
        };

        create_backup(&request, backup_dir.path()).await.unwrap();

        // Delete the file — restore would create it as "new"
        std::fs::remove_file(src.path().join("Config/settings.toml")).unwrap();

        let archive = std::fs::read_dir(backup_dir.path().join("test"))
            .unwrap()
            .filter_map(|e| e.ok())
            .next()
            .unwrap()
            .path();

        let summary = restore_preview(&archive).await.unwrap();
        assert_eq!(summary.new_files.len(), 1);
    }
}
