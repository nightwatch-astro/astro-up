pub mod client;
pub(crate) mod purge;
pub mod stream;
pub mod types;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::Digest;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::config::NetworkConfig;
use crate::error::CoreError;
use crate::events::Event;

pub use types::{DownloadRequest, DownloadResult, PurgeResult};

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
    #[tracing::instrument(skip_all, fields(url = %request.url, expected_hash))]
    pub async fn download(
        &self,
        request: &DownloadRequest,
        cancel_token: CancellationToken,
    ) -> Result<DownloadResult, CoreError> {
        let download_start = std::time::Instant::now();
        tracing::info!(
            url = %request.url,
            expected_hash = request.expected_hash.as_deref().unwrap_or("none"),
            "download started"
        );
        let _guard = self.try_acquire(&request.url)?;

        // Auto-create destination directory (FR-019)
        tokio::fs::create_dir_all(&request.dest_dir).await?;

        // If final file already exists, check with server via conditional request (FR-008)
        let dest = request.dest_path();
        let etag_path = request.dest_dir.join(format!("{}.etag", request.filename));
        if dest.exists() {
            let mut req_builder = self.client.head(&request.url);

            // Send If-None-Match if we have a stored ETag
            if let Ok(etag) = tokio::fs::read_to_string(&etag_path).await {
                req_builder = req_builder.header("If-None-Match", etag.trim());
            }

            // Use file modification time for If-Modified-Since
            if let Ok(meta) = tokio::fs::metadata(&dest).await {
                if let Ok(modified) = meta.modified() {
                    let datetime: chrono::DateTime<chrono::Utc> = modified.into();
                    req_builder = req_builder.header("If-Modified-Since", datetime.to_rfc2822());
                }
            }

            if let Ok(resp) = req_builder.send().await {
                if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
                    let elapsed = download_start.elapsed();
                    tracing::info!(
                        duration_ms = elapsed.as_millis() as u64,
                        cached = true,
                        "download complete (cached)"
                    );
                    return Ok(DownloadResult::Cached { path: dest });
                }
            }
        }

        if let Err(e) = self.event_tx.send(Event::DownloadStarted {
            id: request.filename.clone(),
            url: request.url.clone(),
        }) {
            tracing::debug!("failed to send DownloadStarted event: {e}");
        }

        let result = stream::stream_download(
            &self.client,
            &request.url,
            &request.part_path(),
            &self.event_tx,
            &request.filename,
            self.network_config.download_speed_limit,
            cancel_token.clone(),
            request.resume,
        )
        .await?;

        // Verify hash if expected
        let hash_verified = if let Some(expected) = &request.expected_hash {
            let digest = result.hasher.finalize();
            let actual: String = crate::hex_encode(&digest);
            if actual != *expected {
                // Clean up .part file on mismatch
                if let Err(e) = tokio::fs::remove_file(&request.part_path()).await {
                    tracing::debug!(path = %request.part_path().display(), error = %e, "failed to remove .part file after hash mismatch");
                }

                // If this was a resumed download, retry once from scratch (CHK012)
                if result.resumed {
                    tracing::warn!(
                        retry_count = 1,
                        "hash mismatch after resumed download, retrying from scratch"
                    );
                    let retry = stream::stream_download(
                        &self.client,
                        &request.url,
                        &request.part_path(),
                        &self.event_tx,
                        &request.filename,
                        self.network_config.download_speed_limit,
                        cancel_token,
                        false, // no resume on retry
                    )
                    .await?;

                    let digest = retry.hasher.finalize();
                    let retry_actual: String = crate::hex_encode(&digest);
                    if retry_actual != *expected {
                        if let Err(e) = tokio::fs::remove_file(&request.part_path()).await {
                            tracing::debug!(path = %request.part_path().display(), error = %e, "failed to remove .part file after retry hash mismatch");
                        }
                        return Err(CoreError::ChecksumMismatch {
                            expected: expected.clone(),
                            actual: retry_actual,
                        });
                    }
                    // Retry succeeded
                } else {
                    return Err(CoreError::ChecksumMismatch {
                        expected: expected.clone(),
                        actual,
                    });
                }
            }
            true
        } else {
            false
        };

        // Rename .part to final destination, retry up to 3 times with backoff
        // (Windows: antivirus or stale handles can lock the .part file)
        let dest = request.dest_path();
        let part = request.part_path();
        let mut last_err = None;
        for attempt in 0..3 {
            // On retry, remove a stale target file that may block the rename
            if attempt > 0 && dest.exists() {
                if let Err(e) = tokio::fs::remove_file(&dest).await {
                    tracing::warn!(
                        attempt,
                        path = %dest.display(),
                        error = %e,
                        "failed to remove stale target file before rename retry"
                    );
                }
            }

            match tokio::fs::rename(&part, &dest).await {
                Ok(()) => {
                    last_err = None;
                    break;
                }
                Err(e) => {
                    if attempt < 2 {
                        tracing::warn!(
                            attempt = attempt + 1,
                            max_attempts = 3,
                            from = %part.display(),
                            to = %dest.display(),
                            error = %e,
                            "rename failed, retrying after backoff"
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                    last_err = Some(e);
                }
            }
        }
        if let Some(e) = last_err {
            return Err(CoreError::RenameFailed {
                from: part.display().to_string(),
                to: dest.display().to_string(),
                cause: Box::new(e),
            });
        }

        // Store ETag for future conditional requests (FR-008)
        if let Some(etag) = &result.etag {
            if let Err(e) = tokio::fs::write(&etag_path, etag).await {
                tracing::debug!(path = %etag_path.display(), error = %e, "failed to write ETag cache");
            }
        }

        if let Err(e) = self.event_tx.send(Event::DownloadComplete {
            id: request.filename.clone(),
        }) {
            tracing::debug!("failed to send DownloadComplete event: {e}");
        }

        let elapsed = download_start.elapsed();
        tracing::info!(
            bytes_downloaded = result.bytes_downloaded,
            duration_ms = elapsed.as_millis() as u64,
            cached = false,
            resumed = result.resumed,
            "download complete"
        );

        Ok(DownloadResult::Success {
            path: dest,
            hash_verified,
            bytes_downloaded: result.bytes_downloaded,
            resumed: result.resumed,
        })
    }

    /// Purge installers older than `max_age_days` from `download_dir`.
    ///
    /// Called by the background service (spec 016), not on a timer here.
    /// Returns the number of files deleted and total bytes reclaimed.
    pub async fn purge(
        &self,
        download_dir: &std::path::Path,
        max_age_days: u32,
    ) -> Result<PurgeResult, CoreError> {
        purge::purge(download_dir, max_age_days).await
    }
}

impl crate::traits::Downloader for DownloadManager {
    async fn download(
        &self,
        request: &DownloadRequest,
        cancel_token: CancellationToken,
    ) -> Result<DownloadResult, CoreError> {
        self.download(request, cancel_token).await
    }
}
