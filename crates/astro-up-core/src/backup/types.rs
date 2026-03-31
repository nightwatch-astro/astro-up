use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::events::Event;
use crate::types::Version;

/// Metadata stored as `metadata.json` inside each backup ZIP archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub package_id: String,
    pub version: Version,
    pub created_at: DateTime<Utc>,
    pub paths: Vec<PathBuf>,
    pub file_count: u32,
    pub total_size: u64,
    pub excluded_files: Vec<String>,
    pub file_hashes: HashMap<String, String>,
}

/// Per-file comparison result for restore preview.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug)]
pub struct BackupRequest {
    pub package_id: String,
    pub version: Version,
    pub config_paths: Vec<PathBuf>,
    pub event_tx: broadcast::Sender<Event>,
}

/// Input for a restore operation.
#[derive(Debug)]
pub struct RestoreRequest {
    pub archive_path: PathBuf,
    pub path_filter: Option<String>,
    pub event_tx: broadcast::Sender<Event>,
}
