pub mod client;
pub mod purge;
pub mod stream;
pub mod types;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sha2::Digest;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::config::NetworkConfig;
use crate::error::CoreError;
use crate::events::Event;

pub use types::{DownloadProgress, DownloadRequest, DownloadResult, PurgeResult};

/// Download manager — owns the HTTP client and enforces sequential downloads.
pub struct DownloadManager {
    client: reqwest::Client,
    event_tx: broadcast::Sender<Event>,
    active: Arc<AtomicBool>,
    network_config: NetworkConfig,
}

/// Drop guard that releases the sequential download lock.
struct DownloadGuard {
    active: Arc<AtomicBool>,
}

impl Drop for DownloadGuard {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

impl DownloadManager {
    /// Create a new download manager from network config.
    pub fn new(
        network_config: &NetworkConfig,
        event_tx: broadcast::Sender<Event>,
    ) -> Result<Self, CoreError> {
        let client = client::build_client(network_config)?;
        Ok(Self {
            client,
            event_tx,
            active: Arc::new(AtomicBool::new(false)),
            network_config: network_config.clone(),
        })
    }

    /// Try to acquire the sequential download lock. Returns a guard that
    /// releases the lock on drop, or `DownloadInProgress` if already held.
    fn try_acquire(&self, url: &str) -> Result<DownloadGuard, CoreError> {
        if self
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(CoreError::DownloadInProgress {
                url: url.to_owned(),
            });
        }
        Ok(DownloadGuard {
            active: Arc::clone(&self.active),
        })
    }

    /// Download a file. Returns error if another download is in progress.
    pub async fn download(
        &self,
        request: &DownloadRequest,
        cancel_token: CancellationToken,
    ) -> Result<DownloadResult, CoreError> {
        let _guard = self.try_acquire(&request.url)?;

        // Auto-create destination directory (FR-019)
        tokio::fs::create_dir_all(&request.dest_dir).await?;

        let _ = self.event_tx.send(Event::DownloadStarted {
            id: request.filename.clone(),
            url: request.url.clone(),
        });

        let result = stream::stream_download(
            &self.client,
            &request.url,
            &request.part_path(),
            &self.event_tx,
            &request.filename,
            self.network_config.download_speed_limit,
            cancel_token,
        )
        .await?;

        // Verify hash if expected
        let hash_verified = if let Some(expected) = &request.expected_hash {
            let actual = format!("{:x}", result.hasher.finalize());
            if actual != *expected {
                // Clean up .part file on mismatch
                let _ = tokio::fs::remove_file(&request.part_path()).await;
                return Err(CoreError::ChecksumMismatch {
                    expected: expected.clone(),
                    actual,
                });
            }
            true
        } else {
            false
        };

        // Rename .part to final destination
        let dest = request.dest_path();
        let part = request.part_path();
        tokio::fs::rename(&part, &dest).await.map_err(|e| CoreError::RenameFailed {
            from: part.display().to_string(),
            to: dest.display().to_string(),
            cause: Box::new(e),
        })?;

        let _ = self.event_tx.send(Event::DownloadComplete {
            id: request.filename.clone(),
        });

        Ok(DownloadResult::Success {
            path: dest,
            hash_verified,
            bytes_downloaded: result.bytes_downloaded,
            resumed: result.resumed,
        })
    }
}
