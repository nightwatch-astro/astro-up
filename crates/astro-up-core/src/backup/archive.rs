use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::{info, trace, warn};
use walkdir::WalkDir;

use crate::backup::types::{BackupMetadata, BackupRequest};
use crate::error::CoreError;
use crate::events::Event;

/// Creates a backup ZIP archive from the configured paths.
///
/// Walks each config_path, hashes files with SHA-256, writes to ZIP with relative paths.
/// Skips locked/inaccessible files with a warning. Writes `metadata.json` into the archive.
/// Emits `BackupProgress` events during archiving.
#[tracing::instrument(skip_all, fields(package))]
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
    trace!(count, paths = paths.len(), "counted files for backup");
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
            let Ok(relative) = source_path.strip_prefix(config_path) else {
                continue;
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

            let digest = Sha256::digest(&contents);
            let hash: String = crate::hex_encode(&digest);
            file_hashes.insert(archive_entry_path.clone(), hash);

            zip.start_file(&archive_entry_path, options)
                .map_err(|e| CoreError::Io(io::Error::other(e)))?;
            zip.write_all(&contents)?;

            trace!(
                file = %archive_entry_path,
                size = contents.len(),
                "archived file"
            );

            total_size += contents.len() as u64;
            file_count += 1;
            files_processed += 1;

            // Emit progress every 10 files or on last file
            if files_processed % 10 == 0 || files_processed == total_files {
                // Intentionally silent: progress event in file-processing loop
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
            .map_or_else(|| "backup".to_string(), |n| n.to_string_lossy().to_string());

        let count = used.entry(base.clone()).or_insert(0);
        *count += 1;

        if *count == 1 {
            names.push(base);
        } else {
            names.push(format!("{base}_{count}"));
        }
    }
    names
}

/// Reads metadata.json from a backup archive (sync, usable from spawn_blocking).
pub(crate) fn read_metadata_sync(archive_path: &Path) -> Result<BackupMetadata, CoreError> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| CoreError::Io(io::Error::other(e)))?;
    let mut entry = archive
        .by_name("metadata.json")
        .map_err(|e| CoreError::Io(io::Error::other(e)))?;
    let mut buf = String::new();
    entry.read_to_string(&mut buf)?;
    let metadata: BackupMetadata = serde_json::from_str(&buf)?;
    Ok(metadata)
}

/// Reads metadata.json from a backup archive without extracting.
#[tracing::instrument(skip_all)]
pub async fn read_metadata(archive_path: &Path) -> Result<BackupMetadata, CoreError> {
    let archive_path = archive_path.to_path_buf();
    tokio::task::spawn_blocking(move || read_metadata_sync(&archive_path))
        .await
        .map_err(|e| CoreError::Io(io::Error::other(e)))?
}

/// Extracts a backup archive to the original paths stored in metadata.
#[tracing::instrument(skip_all)]
pub async fn restore(archive_path: &Path, path_filter: Option<&str>) -> Result<(), CoreError> {
    let archive_path = archive_path.to_path_buf();
    let filter = path_filter.map(ToString::to_string);

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

    // Check if path_filter matches anything (component-based matching)
    if let Some(filter) = path_filter {
        let filter_path = Path::new(filter);
        let has_match = (0..archive.len()).any(|i| {
            archive
                .by_index(i)
                .ok()
                .is_some_and(|e| Path::new(e.name()).starts_with(filter_path))
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

        // Apply path filter (component-based matching, not string prefix)
        if let Some(filter) = path_filter {
            if !Path::new(&name).starts_with(Path::new(filter)) {
                continue;
            }
        }

        // Reject symlinks (Unix symlink mode or zip-reported symlink)
        let is_symlink =
            entry.is_symlink() || entry.unix_mode().is_some_and(|m| m & 0xF000 == 0xA000);
        if is_symlink {
            warn!(entry = %name, "skipping symlink archive entry");
            continue;
        }

        // Resolve target path: validate entry safety and map to original path
        let target = match resolve_restore_target(&name, &dir_to_path) {
            Ok(Some(t)) => t,
            Ok(None) => {
                warn!(entry = %name, "skipping archive entry: cannot resolve restore target");
                continue;
            }
            Err(e) => {
                warn!(entry = %name, error = %e, "skipping unsafe archive entry");
                continue;
            }
        };

        if entry.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&target)?;
            io::copy(&mut entry, &mut outfile)?;
            trace!(file = %target.display(), "restored file");
        }
    }

    info!(archive = %archive_path.display(), "restore complete");
    Ok(())
}

/// Maps an archive entry name back to its original filesystem path.
///
/// Validates the relative path portion against directory traversal and
/// absolute path attacks before resolving within the original directory.
fn resolve_restore_target(
    entry_name: &str,
    dir_to_path: &HashMap<&str, &Path>,
) -> Result<Option<PathBuf>, CoreError> {
    // entry_name is like "Profiles/subdir/file.txt"
    // First component is the dir name, rest is relative path
    let Some(slash_pos) = entry_name.find('/') else {
        return Ok(None);
    };
    let dir_name = &entry_name[..slash_pos];
    let relative = &entry_name[slash_pos + 1..];

    if relative.is_empty() {
        return Ok(None); // Directory entry itself
    }

    let Some(original_path) = dir_to_path.get(dir_name) else {
        return Ok(None);
    };

    // Validate the relative path portion for traversal attacks.
    // Symlinks are already rejected by the caller before we get here.
    crate::validation::validate_zip_entry(relative, 0, original_path).map(Some)
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
            .filter_map(Result::ok)
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
            .find_map(Result::ok)
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
            .find_map(Result::ok)
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
            .find_map(Result::ok)
            .unwrap()
            .path();

        let err = restore(&archive, Some("NonExistent")).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("NonExistent"),
            "error should mention the filter: {msg}"
        );
        assert!(
            msg.contains("Profiles"),
            "error should list available paths: {msg}"
        );
        assert!(
            msg.contains("Settings"),
            "error should list available paths: {msg}"
        );
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

    // --- Path traversal integration tests (T012) ---

    /// Create a malicious ZIP with a directory traversal entry.
    fn make_traversal_zip(dest: &Path, entry_name: &str) -> PathBuf {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        let zip_path = dest.join("malicious.zip");
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        // Write metadata.json (required by restore)
        let meta = BackupMetadata {
            package_id: "test-pkg".into(),
            version: Version::parse("1.0.0"),
            created_at: chrono::Utc::now(),
            total_size: 0,
            file_count: 1,
            file_hashes: std::collections::HashMap::new(),
            paths: vec![dest.join("Profiles")],
            excluded_files: vec![],
        };
        zip.start_file("metadata.json", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(serde_json::to_string(&meta).unwrap().as_bytes())
            .unwrap();

        // Write the malicious entry
        zip.start_file(entry_name, SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"malicious content").unwrap();

        zip.finish().unwrap();
        zip_path
    }

    #[tokio::test]
    async fn restore_rejects_dotdot_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("Profiles")).unwrap();

        let zip = make_traversal_zip(tmp.path(), "Profiles/../../etc/passwd");
        let result = restore(&zip, None).await;

        // Should succeed (skips malicious entry) but not create the file
        assert!(result.is_ok());
        assert!(!tmp.path().join("etc/passwd").exists());
    }

    #[tokio::test]
    async fn restore_rejects_absolute_path_entry() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("Profiles")).unwrap();

        let zip = make_traversal_zip(tmp.path(), "Profiles//etc/shadow");
        let result = restore(&zip, None).await;

        assert!(result.is_ok());
        assert!(!PathBuf::from("/etc/shadow").exists());
    }

    #[tokio::test]
    async fn restore_valid_entries_succeed() {
        let src = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        make_test_tree(src.path());

        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let request = BackupRequest {
            package_id: "safe-pkg".into(),
            version: Version::parse("1.0.0"),
            config_paths: vec![src.path().join("Profiles"), src.path().join("Settings")],
            event_tx: tx,
        };

        create_backup(&request, backup_dir.path()).await.unwrap();

        // Modify files
        fs::write(src.path().join("Profiles/default.json"), "CHANGED").unwrap();

        let archive = fs::read_dir(backup_dir.path().join("safe-pkg"))
            .unwrap()
            .find_map(Result::ok)
            .unwrap()
            .path();

        // Restore should overwrite
        restore(&archive, None).await.unwrap();
        let content = fs::read_to_string(src.path().join("Profiles/default.json")).unwrap();
        assert_eq!(content, r#"{"name":"default"}"#);
    }

    #[test]
    fn resolve_restore_target_rejects_traversal() {
        let mut dir_map = HashMap::new();
        let profiles = PathBuf::from("/safe/Profiles");
        dir_map.insert("Profiles", profiles.as_path());

        let result = resolve_restore_target("Profiles/../../etc/passwd", &dir_map);
        assert!(result.is_err() || result.unwrap().is_none());
    }

    #[test]
    fn resolve_restore_target_accepts_valid_path() {
        let mut dir_map = HashMap::new();
        let profiles = PathBuf::from("/safe/Profiles");
        dir_map.insert("Profiles", profiles.as_path());

        let result = resolve_restore_target("Profiles/subdir/file.txt", &dir_map);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(PathBuf::from("/safe/Profiles/subdir/file.txt"))
        );
    }
}
