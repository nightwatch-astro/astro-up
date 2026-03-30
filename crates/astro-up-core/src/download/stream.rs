use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

use crate::error::CoreError;
use crate::events::Event;

/// Outcome of the streaming download, before rename.
pub(crate) struct StreamResult {
    pub bytes_downloaded: u64,
    pub hasher: Sha256,
    pub resumed: bool,
    pub etag: Option<String>,
}

/// Stream a URL to a `.part` file, emitting progress events.
///
/// Supports resume via Range headers when a `.part` file exists and
/// `attempt_resume` is true. Validates freshness against Last-Modified
/// and .part file size before resuming.
pub(crate) async fn stream_download(
    client: &reqwest::Client,
    url: &str,
    part_path: &Path,
    event_tx: &tokio::sync::broadcast::Sender<Event>,
    id: &str,
    throttle_bytes_per_sec: u64,
    cancel_token: CancellationToken,
    attempt_resume: bool,
) -> Result<StreamResult, CoreError> {
    let mut hasher = Sha256::new();

    // Check for existing .part file and attempt resume
    if attempt_resume && part_path.exists() {
        let part_meta = tokio::fs::metadata(part_path).await?;
        let part_size = part_meta.len();

        if part_size > 0 {
            // Probe server with Range header
            let probe = client
                .get(url)
                .header("Range", format!("bytes={part_size}-"))
                .send()
                .await
                .map_err(|e| CoreError::DownloadFailed {
                    url: url.to_owned(),
                    status: e.status().map_or(0, |s| s.as_u16()),
                    reason: e.to_string(),
                })?;

            if probe.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                // Check freshness: if server's Last-Modified is newer than .part mtime, restart
                let should_restart = if let Some(last_modified) = probe.headers().get("Last-Modified") {
                    if let Ok(server_date) = httpdate_parse(last_modified.to_str().unwrap_or("")) {
                        if let Ok(part_modified) = part_meta.modified() {
                            let part_time: chrono::DateTime<chrono::Utc> = part_modified.into();
                            server_date > part_time
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Validate .part size against server Content-Range
                let server_reports_valid = if let Some(content_range) = probe.headers().get("Content-Range") {
                    // Format: bytes START-END/TOTAL
                    if let Some(total_str) = content_range.to_str().ok().and_then(|s| s.split('/').next_back()) {
                        if let Ok(total) = total_str.parse::<u64>() {
                            part_size <= total
                        } else {
                            true // Can't parse, allow
                        }
                    } else {
                        true
                    }
                } else {
                    true
                };

                if !should_restart && server_reports_valid {
                    // Resume: hash existing bytes first, then append
                    let mut existing_file = tokio::fs::File::open(part_path).await?;
                    let mut buf = vec![0u8; 65536];
                    loop {
                        let n = existing_file.read(&mut buf).await?;
                        if n == 0 {
                            break;
                        }
                        hasher.update(&buf[..n]);
                    }
                    drop(existing_file);

                    // Stream remaining bytes from the probe response
                    // Resume doesn't capture a new ETag (use existing one)
                    return stream_response(
                        probe, part_path, true, &mut hasher, part_size,
                        event_tx, id, throttle_bytes_per_sec, cancel_token, true, None,
                    )
                    .await;
                }
            }
            // If we get here: server returned 200 (no Range), or freshness check failed,
            // or .part is corrupt — delete and restart
            let _ = tokio::fs::remove_file(part_path).await;
        }
    }

    // Fresh download
    let response = client.get(url).send().await.map_err(|e| CoreError::DownloadFailed {
        url: url.to_owned(),
        status: e.status().map_or(0, |s| s.as_u16()),
        reason: e.to_string(),
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(CoreError::DownloadFailed {
            url: url.to_owned(),
            status: status.as_u16(),
            reason: status.canonical_reason().unwrap_or("unknown").to_owned(),
        });
    }

    // Capture ETag for conditional requests (FR-008)
    let etag = response
        .headers()
        .get("ETag")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    // Disk space check (FR-017): require 2x file size, skip if unknown
    if let Some(content_length) = response.content_length() {
        let required = content_length * 2;
        if let Some(available) = available_disk_space(part_path) {
            if available < required {
                return Err(CoreError::DiskSpaceInsufficient { required, available });
            }
        }
    }

    stream_response(
        response, part_path, false, &mut hasher, 0,
        event_tx, id, throttle_bytes_per_sec, cancel_token, false, etag,
    )
    .await
}

/// Parse an HTTP date string (RFC 2822 or RFC 7231).
fn httpdate_parse(s: &str) -> Result<chrono::DateTime<chrono::Utc>, ()> {
    chrono::DateTime::parse_from_rfc2822(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            // Try HTTP-date format: "Sun, 06 Nov 1994 08:49:37 GMT"
            chrono::NaiveDateTime::parse_from_str(s, "%a, %d %b %Y %H:%M:%S GMT")
                .map(|ndt| ndt.and_utc())
        })
        .map_err(|_| ())
}

/// Stream an HTTP response body to a file, updating hash and emitting progress.
async fn stream_response(
    mut response: reqwest::Response,
    part_path: &Path,
    append: bool,
    hasher: &mut Sha256,
    initial_bytes: u64,
    event_tx: &tokio::sync::broadcast::Sender<Event>,
    id: &str,
    throttle_bytes_per_sec: u64,
    cancel_token: CancellationToken,
    resumed: bool,
    etag: Option<String>,
) -> Result<StreamResult, CoreError> {
    let total_bytes = if let Some(content_length) = response.content_length() {
        initial_bytes + content_length
    } else {
        0
    };

    let mut file = if append {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(part_path)
            .await?
    } else {
        tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(part_path)
            .await?
    };

    let mut bytes_downloaded = initial_bytes;
    let start = Instant::now();
    let mut last_progress = Instant::now();
    let mut speed_window: VecDeque<(Instant, u64)> = VecDeque::new();
    speed_window.push_back((start, bytes_downloaded));

    let url = response.url().to_string();

    while let Some(chunk) = response.chunk().await.map_err(|e| CoreError::DownloadFailed {
        url: url.clone(),
        status: 0,
        reason: e.to_string(),
    })? {
        if cancel_token.is_cancelled() {
            file.flush().await?;
            return Err(CoreError::Cancelled);
        }

        file.write_all(&chunk).await?;
        hasher.update(&chunk);
        bytes_downloaded += chunk.len() as u64;

        let now = Instant::now();
        speed_window.push_back((now, bytes_downloaded));
        while speed_window.len() > 1 {
            if now.duration_since(speed_window[0].0) > Duration::from_secs(5) {
                speed_window.pop_front();
            } else {
                break;
            }
        }

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

        let since_last = now.duration_since(last_progress);
        if since_last >= Duration::from_millis(100) || chunk.len() >= 65536 {
            let progress = if total_bytes > 0 {
                bytes_downloaded as f64 / total_bytes as f64
            } else {
                0.0
            };

            let elapsed = start.elapsed();
            let estimated_remaining = if total_bytes > 0 && speed > 0.0 {
                let remaining_bytes = total_bytes.saturating_sub(bytes_downloaded);
                Some(Duration::from_secs_f64(remaining_bytes as f64 / speed))
            } else {
                None
            };

            let _ = event_tx.send(Event::DownloadProgress {
                id: id.to_owned(),
                progress,
                bytes_downloaded,
                total_bytes,
                speed,
                elapsed,
                estimated_remaining,
            });
            last_progress = now;
        }

        if throttle_bytes_per_sec > 0 {
            let new_bytes = bytes_downloaded - initial_bytes;
            let expected_time = new_bytes as f64 / throttle_bytes_per_sec as f64;
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
        hasher: hasher.clone(),
        resumed,
        etag,
    })
}

/// Get available disk space for the partition containing `path`.
fn available_disk_space(path: &Path) -> Option<u64> {
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    let mut best_match: Option<(usize, u64)> = None;
    for disk in disks.list() {
        let mount = disk.mount_point();
        if path.starts_with(mount) {
            let mount_len = mount.as_os_str().len();
            if best_match.is_none_or(|(len, _)| mount_len > len) {
                best_match = Some((mount_len, disk.available_space()));
            }
        }
    }
    best_match.map(|(_, space)| space)
}
