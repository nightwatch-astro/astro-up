# Data Model: 013 Backup and Restore

**Date**: 2026-03-31

## New Types

### BackupMetadata

Stored as `metadata.json` inside each backup ZIP archive. Also returned by backup operations.

| Field | Type | Description |
|-------|------|-------------|
| package_id | String | Package identifier from catalog |
| version | Version | Currently installed version at backup time |
| created_at | DateTime\<Utc\> | Backup timestamp |
| paths | Vec\<PathBuf\> | Original config_paths (expanded, absolute) |
| file_count | u32 | Number of files in archive (excluding metadata.json) |
| total_size | u64 | Total uncompressed size in bytes |
| excluded_files | Vec\<String\> | Files skipped due to locks or errors |
| file_hashes | HashMap\<String, String\> | Relative path -> SHA-256 hash for restore preview |

### FileChangeSummary

Returned by restore preview. Per-file status comparison between archive and disk.

| Field | Type | Description |
|-------|------|-------------|
| overwritten | Vec\<String\> | Files that differ (hash mismatch) |
| unchanged | Vec\<String\> | Files that are identical (hash match) |
| new_files | Vec\<String\> | Files in archive but not on disk |
| missing | Vec\<String\> | Files on disk not in archive (won't be touched) |

### BackupListEntry

Summary for listing available backups per package.

| Field | Type | Description |
|-------|------|-------------|
| archive_path | PathBuf | Full path to ZIP file |
| package_id | String | Package identifier |
| version | Version | Version at backup time |
| created_at | DateTime\<Utc\> | Backup timestamp |
| file_count | u32 | Number of files |
| total_size | u64 | Uncompressed size |

## Modified Types (Cross-spec changes to spec 003)

### BackupManager trait (evolve)

```rust
// Before (spec 003):
trait BackupManager {
    async fn backup(&self, software_id: &str, paths: &[String]) -> Result<BackupResult, CoreError>;
    async fn restore(&self, software_id: &str, timestamp: &str) -> Result<(), CoreError>;
    async fn list(&self, software_id: &str) -> Result<Vec<BackupEntry>, CoreError>;
    async fn prune(&self, software_id: &str, keep: usize) -> Result<(), CoreError>;
}

// After (spec 013):
trait BackupManager {
    async fn backup(&self, request: &BackupRequest) -> Result<BackupMetadata, CoreError>;
    async fn restore(&self, request: &RestoreRequest) -> Result<(), CoreError>;
    async fn restore_preview(&self, archive_path: &Path) -> Result<FileChangeSummary, CoreError>;
    async fn list(&self, package_id: &str) -> Result<Vec<BackupListEntry>, CoreError>;
    async fn prune(&self, package_id: &str, keep: usize) -> Result<u32, CoreError>;
}
```

### BackupRequest

| Field | Type | Description |
|-------|------|-------------|
| package_id | String | Package identifier |
| version | Version | Currently installed version |
| config_paths | Vec\<PathBuf\> | Expanded absolute paths to back up |
| event_tx | broadcast::Sender\<Event\> | Event channel for progress |

### RestoreRequest

| Field | Type | Description |
|-------|------|-------------|
| archive_path | PathBuf | Path to backup ZIP |
| path_filter | Option\<String\> | Selective restore filter (e.g., "Profiles/") |
| event_tx | broadcast::Sender\<Event\> | Event channel |

### New Event Variants

| Variant | Fields | When |
|---------|--------|------|
| BackupProgress | id: String, files_processed: u32, total_files: u32 | During backup file archiving |
| RestoreStarted | id: String | Before restore extraction begins |
| RestoreComplete | id: String | After restore extraction finishes |

## Archive Structure

```
{package_id}_{version}_{YYYYMMDD_HHMMSS}.zip
├── metadata.json              # BackupMetadata serialized
├── {config_path_0}/           # Files from first config_paths entry
│   ├── relative/path/file1
│   └── relative/path/file2
├── {config_path_1}/           # Files from second config_paths entry
│   └── ...
```

Each config_path gets a top-level directory in the archive named after the path's last component (e.g., `Profiles/`, `Settings/`). Files within are stored with their relative paths from the config_path root.
