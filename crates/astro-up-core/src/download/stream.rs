use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use crate::error::CoreError;
use crate::events::Event;

/// Outcome of the streaming download, before rename.
pub(crate) struct StreamResult {
    pub bytes_downloaded: u64,
    pub hasher: Sha256,
    pub resumed: bool,
}

/// Stream a URL to a `.part` file, emitting progress events.
///
/// Returns the total bytes written and the running SHA256 hasher.
/// The caller is responsible for finalizing the hash and renaming the file.
pub(crate) async fn stream_download(
    client: &reqwest::Client,
    url: &str,
    part_path: &Path,
    event_tx: &tokio::sync::broadcast::Sender<Event>,
    id: &str,
    throttle_bytes_per_sec: u64,
    cancel_token: CancellationToken,
) -> Result<StreamResult, CoreError> {
    let response = client.get(url).send().await.map_err(|e| {
        CoreError::DownloadFailed {
            url: url.to_owned(),
            status: e.status().map_or(0, |s| s.as_u16()),
            reason: e.to_string(),
        }
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(CoreError::DownloadFailed {
            url: url.to_owned(),
            status: status.as_u16(),
            reason: status.canonical_reason().unwrap_or("unknown").to_owned(),
        });
    }

    let total_bytes = response.content_length().unwrap_or(0);
    let mut response = response;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(part_path)
        .await?;

    let mut hasher = Sha256::new();
    let mut bytes_downloaded: u64 = 0;
    let start = Instant::now();
    let mut last_progress = Instant::now();
    let mut speed_window: VecDeque<(Instant, u64)> = VecDeque::new();
    speed_window.push_back((start, 0));

    while let Some(chunk) = response.chunk().await.map_err(|e| CoreError::DownloadFailed {
        url: url.to_owned(),
        status: 0,
        reason: e.to_string(),
    })? {
        // Cancellation check
        if cancel_token.is_cancelled() {
            file.flush().await?;
            return Err(CoreError::Cancelled);
        }

        file.write_all(&chunk).await?;
        hasher.update(&chunk);
        bytes_downloaded += chunk.len() as u64;

        // Update speed window
        let now = Instant::now();
        speed_window.push_back((now, bytes_downloaded));
        // Trim entries older than 5 seconds
        while speed_window.len() > 1 {
            if now.duration_since(speed_window[0].0) > Duration::from_secs(5) {
                speed_window.pop_front();
            } else {
                break;
            }
        }

        // Compute speed from rolling window
        let speed = if speed_window.len() >= 2 {
            let first = &speed_window[0];
            let elapsed_window = now.duration_since(first.0).as_secs_f64();
            if elapsed_window > 0.0 {
                (bytes_downloaded - first.1) as f64 / elapsed_window
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Emit progress every 100ms or 64KB
        let since_last = now.duration_since(last_progress);
        if since_last >= Duration::from_millis(100) || chunk.len() >= 65536 {
            let progress = if total_bytes > 0 {
                bytes_downloaded as f64 / total_bytes as f64
            } else {
                0.0
            };

            let _ = event_tx.send(Event::DownloadProgress {
                id: id.to_owned(),
                progress,
                bytes_downloaded,
                total_bytes,
                speed,
            });
            last_progress = now;
        }

        // Bandwidth throttling
        if throttle_bytes_per_sec > 0 {
            let expected_time = bytes_downloaded as f64 / throttle_bytes_per_sec as f64;
            let actual_time = start.elapsed().as_secs_f64();
            let ahead_by = expected_time - actual_time;
            if ahead_by > 0.001 {
                tokio::time::sleep(Duration::from_secs_f64(ahead_by)).await;
            }
        }
    }

    file.flush().await?;

    Ok(StreamResult {
        bytes_downloaded,
        hasher,
        resumed: false,
    })
}
