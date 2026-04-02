use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::backup::types::{BackupMetadata, BackupRequest};
use crate::error::CoreError;
use crate::events::Event;

/// Creates a backup ZIP archive from the configured paths.
///
/// Walks each config_path, hashes files with SHA-256, writes to ZIP with relative paths.
/// Skips locked/inaccessible files with a warning. Writes `metadata.json` into the archive.
/// Emits `BackupProgress` events during archiving.
pub async fn create_backup(
    request: &BackupRequest,
    backup_dir: &Path,
) -> Result<BackupMetadata, CoreError> {
    let package_dir = backup_dir.join(&request.package_id);
    tokio::fs::create_dir_all(&package_dir).await?;

    // Build archive filename: {package_id}_{version}_{YYYYMMDD_HHMMSS}.zip
    let timestamp = Utc::now();
    let ts_str = timestamp.format("%Y%m%d_%H%M%S").to_string();
    let filename = format!(
        "{}_{}_{}.zip",
        request.package_id, request.version.raw, ts_str
    );
    let archive_path = package_dir.join(&filename);

    let config_paths = request.config_paths.clone();
    let package_id = request.package_id.clone();
    let event_tx = request.event_tx.clone();

    // Count total files first for progress
    let paths_for_count = config_paths.clone();
    let total_files = tokio::task::spawn_blocking(move || count_files(&paths_for_count))
        .await
        .map_err(|e| CoreError::Io(io::Error::other(e)))?;

    // Create archive in blocking thread
    let archive_path_clone = archive_path.clone();
    let version = request.version.clone();
    let paths_clone = config_paths.clone();

    let metadata = tokio::task::spawn_blocking(move || {
        create_archive_sync(
            &archive_path_clone,
            &package_id,
            &version,
            &paths_clone,
            timestamp,
            total_files,
            &event_tx,
        )
    })
    .await
    .map_err(|e| CoreError::Io(io::Error::other(e)))??;

    Ok(metadata)
}

fn count_files(paths: &[PathBuf]) -> u32 {
    let mut count = 0u32;
    for path in paths {
        if path.exists() {
            for entry in WalkDir::new(path).follow_links(true).into_iter().flatten() {
                if entry.file_type().is_file() {
                    count += 1;
                }
            }
        }
    }
    count
}

fn create_archive_sync(
    archive_path: &Path,
    package_id: &str,
    version: &crate::types::Version,
    config_paths: &[PathBuf],
    timestamp: chrono::DateTime<Utc>,
    total_files: u32,
    event_tx: &tokio::sync::broadcast::Sender<Event>,
) -> Result<BackupMetadata, CoreError> {
    let file = File::create(archive_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut file_hashes: HashMap<String, String> = HashMap::new();
    let mut excluded_files: Vec<String> = Vec::new();
    let mut file_count: u32 = 0;
    let mut total_size: u64 = 0;
    let mut files_processed: u32 = 0;

    // Resolve archive directory names, disambiguating collisions
    let dir_names = resolve_dir_names(config_paths);

    for (i, config_path) in config_paths.iter().enumerate() {
        if !config_path.exists() {
            warn!(path = %config_path.display(), "config path does not exist, skipping");
            continue;
        }

        let archive_prefix = &dir_names[i];

        for entry in WalkDir::new(config_path).follow_links(true) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!(error = %e, "walkdir error, skipping entry");
                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let source_path = entry.path();
            let relative = match source_path.strip_prefix(config_path) {
                Ok(r) => r,
                Err(_) => continue,
            };

            // Build archive path with forward slashes (ZIP spec)
            let archive_entry_path = format!(
                "{}/{}",
                archive_prefix,
                relative.to_string_lossy().replace('\\', "/")
            );

            // Try to open — skip locked files
            let mut source_file = match File::open(source_path) {
                Ok(f) => f,
                Err(e) if is_locked_error(&e) => {
                    warn!(path = %source_path.display(), "file locked, skipping");
                    excluded_files.push(source_path.display().to_string());
                    continue;
                }
                Err(e) => return Err(CoreError::Io(e)),
            };

            // Read file, compute hash, write to zip
            let mut contents = Vec::new();
            source_file.read_to_end(&mut contents)?;

            let hash = format!("{:x}", Sha256::digest(&contents));
            file_hashes.insert(archive_entry_path.clone(), hash);

            zip.start_file(&archive_entry_path, options)
                .map_err(|e| CoreError::Io(io::Error::other(e)))?;
            zip.write_all(&contents)?;

            total_size += contents.len() as u64;
            file_count += 1;
            files_processed += 1;

            // Emit progress every 10 files or on last file
            if files_processed % 10 == 0 || files_processed == total_files {
                let _ = event_tx.send(Event::BackupProgress {
                    id: package_id.to_string(),
                    files_processed,
                    total_files,
                });
            }
        }
    }

    // Write metadata.json
    let metadata = BackupMetadata {
        package_id: package_id.to_string(),
        version: version.clone(),
        created_at: timestamp,
        paths: config_paths.to_vec(),
        file_count,
        total_size,
        excluded_files,
        file_hashes,
    };

    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    zip.start_file("metadata.json", options)
        .map_err(|e| CoreError::Io(io::Error::other(e)))?;
    zip.write_all(metadata_json.as_bytes())?;

    zip.finish()
        .map_err(|e| CoreError::Io(io::Error::other(e)))?;

    info!(
        archive = %archive_path.display(),
        files = file_count,
        size = total_size,
        "backup archive created"
    );

    Ok(metadata)
}

/// Resolves top-level directory names for each config_path in the archive.
/// Disambiguates collisions with numeric suffixes.
pub(crate) fn resolve_dir_names(paths: &[PathBuf]) -> Vec<String> {
    let mut names: Vec<String> = Vec::with_capacity(paths.len());
    let mut used: HashMap<String, u32> = HashMap::new();

    for path in paths {
        let base = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "backup".to_string());

        let count = used.entry(base.clone()).or_insert(0);
        *count += 1;

        if *count == 1 {
            names.push(base);
        } else {
            names.push(format!("{}_{}", base, count));
        }
    }
    names
}

/// Reads metadata.json from a backup archive without extracting.
pub async fn read_metadata(archive_path: &Path) -> Result<BackupMetadata, CoreError> {
    let archive_path = archive_path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let file = File::open(&archive_path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| CoreError::Io(io::Error::other(e)))?;
        let mut entry = archive
            .by_name("metadata.json")
            .map_err(|e| CoreError::Io(io::Error::other(e)))?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        let metadata: BackupMetadata = serde_json::from_str(&buf)?;
        Ok(metadata)
    })
    .await
    .map_err(|e| CoreError::Io(io::Error::other(e)))?
}

/// Extracts a backup archive to the original paths stored in metadata.
pub async fn restore(archive_path: &Path, path_filter: Option<&str>) -> Result<(), CoreError> {
    let archive_path = archive_path.to_path_buf();
    let filter = path_filter.map(|s| s.to_string());

    tokio::task::spawn_blocking(move || restore_sync(&archive_path, filter.as_deref()))
        .await
        .map_err(|e| CoreError::Io(io::Error::other(e)))?
}

fn restore_sync(archive_path: &Path, path_filter: Option<&str>) -> Result<(), CoreError> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| CoreError::Io(io::Error::other(e)))?;

    // Read metadata.json first
    let metadata: BackupMetadata = {
        let mut entry = archive
            .by_name("metadata.json")
            .map_err(|e| CoreError::Io(io::Error::other(e)))?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        serde_json::from_str(&buf)?
    };

    // Build a map from archive dir name -> original path
    let dir_names = resolve_dir_names(&metadata.paths);
    let dir_to_path: HashMap<&str, &Path> = dir_names
        .iter()
        .zip(metadata.paths.iter())
        .map(|(name, path)| (name.as_str(), path.as_path()))
        .collect();

    // Check if path_filter matches anything
    if let Some(filter) = path_filter {
        let has_match = (0..archive.len()).any(|i| {
            archive
                .by_index(i)
                .ok()
                .map(|e| e.name().starts_with(filter))
                .unwrap_or(false)
        });
        if !has_match {
            let available: Vec<String> = dir_names.clone();
            return Err(CoreError::NotFound {
                input: format!(
                    "path filter {:?} not found in archive. Available: {}",
                    filter,
                    available.join(", ")
                ),
            });
        }
    }

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| CoreError::Io(io::Error::other(e)))?;

        let name = entry.name().to_string();
        if name == "metadata.json" {
            continue;
        }

        // Apply path filter
        if let Some(filter) = path_filter {
            if !name.starts_with(filter) {
                continue;
            }
        }

        // Resolve target path: find which dir prefix matches, map to original path
        let target = resolve_restore_target(&name, &dir_to_path);
        let Some(target) = target else { continue };

        if entry.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&target)?;
            io::copy(&mut entry, &mut outfile)?;
        }
    }

    info!(archive = %archive_path.display(), "restore complete");
    Ok(())
}

/// Maps an archive entry name back to its original filesystem path.
fn resolve_restore_target(entry_name: &str, dir_to_path: &HashMap<&str, &Path>) -> Option<PathBuf> {
    // entry_name is like "Profiles/subdir/file.txt"
    // First component is the dir name, rest is relative path
    let slash_pos = entry_name.find('/')?;
    let dir_name = &entry_name[..slash_pos];
    let relative = &entry_name[slash_pos + 1..];

    if relative.is_empty() {
        return None; // Directory entry itself
    }

    let original_path = dir_to_path.get(dir_name)?;
    Some(original_path.join(relative))
}

/// Checks if an I/O error indicates a locked/sharing violation file.
fn is_locked_error(e: &io::Error) -> bool {
    // Windows: ERROR_SHARING_VIOLATION = 32
    if e.raw_os_error() == Some(32) {
        return true;
    }
    // Also catch PermissionDenied as a fallback
    e.kind() == io::ErrorKind::PermissionDenied
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Version;

    fn make_test_tree(root: &Path) {
        let profiles = root.join("Profiles");
        let settings = root.join("Settings");
        fs::create_dir_all(&profiles).unwrap();
        fs::create_dir_all(&settings).unwrap();
        fs::write(profiles.join("default.json"), r#"{"name":"default"}"#).unwrap();
        fs::write(profiles.join("astro.json"), r#"{"name":"astro"}"#).unwrap();
        fs::write(settings.join("config.toml"), "theme = \"dark\"").unwrap();
    }

    #[tokio::test]
    async fn backup_creates_archive_with_metadata() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        make_test_tree(src.path());

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "nina-app".into(),
            version: Version::parse("3.1.2"),
            config_paths: vec![src.path().join("Profiles"), src.path().join("Settings")],
            event_tx: tx,
        };

        let metadata = create_backup(&request, backup_dir.path()).await.unwrap();
        assert_eq!(metadata.package_id, "nina-app");
        assert_eq!(metadata.file_count, 3);
        assert_eq!(metadata.excluded_files.len(), 0);
        assert_eq!(metadata.file_hashes.len(), 3);

        // Verify archive exists
        let archives: Vec<_> = fs::read_dir(backup_dir.path().join("nina-app"))
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(archives.len(), 1);
        assert!(archives[0].file_name().to_string_lossy().ends_with(".zip"));
    }

    #[tokio::test]
    async fn backup_skips_missing_path() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(src.path().join("Profiles")).unwrap();
        fs::write(src.path().join("Profiles/a.txt"), "hello").unwrap();

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "test-pkg".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Profiles"), src.path().join("DoesNotExist")],
            event_tx: tx,
        };

        let metadata = create_backup(&request, backup_dir.path()).await.unwrap();
        assert_eq!(metadata.file_count, 1);
    }

    #[tokio::test]
    async fn backup_and_restore_round_trip() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        make_test_tree(src.path());

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "test-pkg".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Profiles"), src.path().join("Settings")],
            event_tx: tx,
        };

        let metadata = create_backup(&request, backup_dir.path()).await.unwrap();

        // Modify a file
        fs::write(src.path().join("Profiles/default.json"), "MODIFIED").unwrap();

        // Find archive
        let archive = fs::read_dir(backup_dir.path().join("test-pkg"))
            .unwrap()
            .filter_map(|e| e.ok())
            .next()
            .unwrap()
            .path();

        // Restore
        restore(&archive, None).await.unwrap();

        // Verify original content restored
        let restored = fs::read_to_string(src.path().join("Profiles/default.json")).unwrap();
        assert_eq!(restored, r#"{"name":"default"}"#);
        assert_eq!(metadata.file_count, 3);
    }

    #[tokio::test]
    async fn selective_restore_filters_by_path() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        make_test_tree(src.path());

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "test-pkg".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Profiles"), src.path().join("Settings")],
            event_tx: tx,
        };

        create_backup(&request, backup_dir.path()).await.unwrap();

        // Modify both
        fs::write(src.path().join("Profiles/default.json"), "MODIFIED").unwrap();
        fs::write(src.path().join("Settings/config.toml"), "MODIFIED").unwrap();

        let archive = fs::read_dir(backup_dir.path().join("test-pkg"))
            .unwrap()
            .filter_map(|e| e.ok())
            .next()
            .unwrap()
            .path();

        // Restore only Profiles
        restore(&archive, Some("Profiles")).await.unwrap();

        // Profiles restored, Settings still modified
        let profiles = fs::read_to_string(src.path().join("Profiles/default.json")).unwrap();
        let settings = fs::read_to_string(src.path().join("Settings/config.toml")).unwrap();
        assert_eq!(profiles, r#"{"name":"default"}"#);
        assert_eq!(settings, "MODIFIED");
    }

    #[tokio::test]
    async fn selective_restore_invalid_filter_lists_available_paths() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        make_test_tree(src.path());

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "test-pkg".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Profiles"), src.path().join("Settings")],
            event_tx: tx,
        };

        create_backup(&request, backup_dir.path()).await.unwrap();

        let archive = fs::read_dir(backup_dir.path().join("test-pkg"))
            .unwrap()
            .filter_map(|e| e.ok())
            .next()
            .unwrap()
            .path();

        let err = restore(&archive, Some("NonExistent")).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("NonExistent"), "error should mention the filter: {msg}");
        assert!(msg.contains("Profiles"), "error should list available paths: {msg}");
        assert!(msg.contains("Settings"), "error should list available paths: {msg}");
    }

    #[test]
    fn dir_name_collision_disambiguated() {
        let paths = vec![
            PathBuf::from("/a/Settings"),
            PathBuf::from("/b/Settings"),
            PathBuf::from("/c/Profiles"),
        ];
        let names = resolve_dir_names(&paths);
        assert_eq!(names, vec!["Settings", "Settings_2", "Profiles"]);
    }
}
