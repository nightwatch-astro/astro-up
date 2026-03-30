//! Download Manager public API contract.
//!
//! This file defines the public interface — NOT compilable code.
//! Implementation goes in `crates/astro-up-core/src/download/`.

use std::path::{Path, PathBuf};
use std::time::Duration;

// -- Types --

pub struct DownloadRequest {
    pub url: String,
    pub expected_hash: Option<String>,
    pub dest_dir: PathBuf,
    pub filename: String,
    pub resume: bool,
}

pub enum DownloadResult {
    Success {
        path: PathBuf,
        hash_verified: bool,
        bytes_downloaded: u64,
        resumed: bool,
    },
    Cached {
        path: PathBuf,
    },
}

pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub elapsed: Duration,
    pub estimated_remaining: Option<Duration>,
}

// -- Manager --

/// The download manager. Owns the reqwest client and enforces sequential downloads.
pub struct DownloadManager {
    // client: reqwest::Client         — configured from AppConfig
    // event_tx: broadcast::Sender<Event>  — for progress events
    // active: AtomicBool              — sequential download lock
}

impl DownloadManager {
    /// Create a new download manager from application config.
    ///
    /// Configures reqwest client with: proxy, connect_timeout, read_timeout,
    /// redirect policy (10 hops), user-agent header, rustls TLS.
    pub fn new(config: &AppConfig, event_tx: broadcast::Sender<Event>) -> Self { .. }

    /// Download a file. Returns error if another download is in progress.
    ///
    /// Flow: disk space check → conditional request (ETag/Last-Modified) →
    ///       resume probe (if .part exists) → stream chunks → hash verify → rename
    ///
    /// Emits: DownloadStarted, DownloadProgress (every 100ms or 64KB), DownloadComplete
    /// Throttle: respects `network.download_speed_limit` from config
    /// Cancel: checks `cancel_token.is_cancelled()` after each chunk
    pub async fn download(
        &self,
        request: &DownloadRequest,
        cancel_token: CancellationToken,
    ) -> Result<DownloadResult, CoreError> { .. }

    /// Purge installers older than `max_age_days` from `download_dir`.
    ///
    /// Called by the background service (spec 016), not on a timer here.
    /// Returns the number of files deleted and total bytes reclaimed.
    pub async fn purge(
        &self,
        download_dir: &Path,
        max_age_days: u32,
    ) -> Result<PurgeResult, CoreError> { .. }
}

pub struct PurgeResult {
    pub files_deleted: u32,
    pub bytes_reclaimed: u64,
}
