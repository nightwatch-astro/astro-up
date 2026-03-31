// Contract: BackupService — spec 013 backup and restore
// This is a design contract, not compilable code.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

// --- New types (backup/types.rs) ---

/// Metadata stored in each backup archive as metadata.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub package_id: String,
    pub version: Version,
    pub created_at: DateTime<Utc>,
    pub paths: Vec<PathBuf>,
    pub file_count: u32,
    pub total_size: u64,
    pub excluded_files: Vec<String>,
    pub file_hashes: HashMap<String, String>, // relative_path -> SHA-256
}

/// Per-file comparison for restore preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeSummary {
    pub overwritten: Vec<String>,
    pub unchanged: Vec<String>,
    pub new_files: Vec<String>,
    pub missing: Vec<String>,
}

/// Summary for listing available backups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupListEntry {
    pub archive_path: PathBuf,
    pub package_id: String,
    pub version: Version,
    pub created_at: DateTime<Utc>,
    pub file_count: u32,
    pub total_size: u64,
}

/// Input for a backup operation.
pub struct BackupRequest {
    pub package_id: String,
    pub version: Version,
    pub config_paths: Vec<PathBuf>,
    pub event_tx: broadcast::Sender<Event>,
}

/// Input for a restore operation.
pub struct RestoreRequest {
    pub archive_path: PathBuf,
    pub path_filter: Option<String>,
    pub event_tx: broadcast::Sender<Event>,
}

// --- Modified trait (traits.rs) ---

/// Updated BackupManager trait with richer operations.
#[trait_variant::make(BackupManagerDyn: Send)]
pub trait BackupManager: Send {
    /// Create a backup archive from configured paths.
    async fn backup(&self, request: &BackupRequest) -> Result<BackupMetadata, CoreError>;

    /// Restore files from a backup archive.
    async fn restore(&self, request: &RestoreRequest) -> Result<(), CoreError>;

    /// Preview what restore would change without modifying files.
    async fn restore_preview(&self, archive_path: &Path) -> Result<FileChangeSummary, CoreError>;

    /// List available backups for a package, sorted by date descending.
    async fn list(&self, package_id: &str) -> Result<Vec<BackupListEntry>, CoreError>;

    /// Delete old backups beyond retention count. Returns number deleted.
    async fn prune(&self, package_id: &str, keep: usize) -> Result<u32, CoreError>;
}

// --- Service (backup/mod.rs) ---

/// Facade for backup operations. Implements BackupManager trait.
pub struct BackupService {
    backup_dir: PathBuf, // {data_dir}/astro-up/backups/
}

impl BackupService {
    pub fn new(backup_dir: PathBuf) -> Self;
}
