use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Input to a download operation.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub expected_hash: Option<String>,
    pub dest_dir: PathBuf,
    pub filename: String,
    pub resume: bool,
}

impl DownloadRequest {
    /// Final destination path: `dest_dir/filename`.
    pub fn dest_path(&self) -> PathBuf {
        self.dest_dir.join(&self.filename)
    }

    /// Temporary path during download: `dest_dir/filename.part`.
    pub fn part_path(&self) -> PathBuf {
        self.dest_dir.join(format!("{}.part", self.filename))
    }
}

/// Result of a successful download.
#[derive(Debug, Clone)]
pub enum DownloadResult {
    /// File downloaded and verified.
    Success {
        path: PathBuf,
        hash_verified: bool,
        bytes_downloaded: u64,
        resumed: bool,
    },
    /// File already exists and matches — skipped download.
    Cached { path: PathBuf },
}

/// Progress snapshot emitted during download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub elapsed: Duration,
    pub estimated_remaining: Option<Duration>,
}

/// Result of a purge operation.
#[derive(Debug, Clone)]
pub struct PurgeResult {
    pub files_deleted: u32,
    pub bytes_reclaimed: u64,
}
