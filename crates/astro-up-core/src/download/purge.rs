use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::download::types::PurgeResult;
use crate::error::CoreError;

/// Purge downloaded installers older than `max_age_days` from `download_dir`.
///
/// Skips `.part` files (in-progress downloads). Returns the number of files
/// deleted and bytes reclaimed. When `max_age_days` is 0, purging is disabled
/// and no files are deleted.
pub(crate) async fn purge(
    download_dir: &Path,
    max_age_days: u32,
) -> Result<PurgeResult, CoreError> {
    if max_age_days == 0 {
        return Ok(PurgeResult {
            files_deleted: 0,
            bytes_reclaimed: 0,
        });
    }

    let max_age = Duration::from_secs(u64::from(max_age_days) * 86400);
    let now = SystemTime::now();
    let mut files_deleted: u32 = 0;
    let mut bytes_reclaimed: u64 = 0;

    let mut entries = tokio::fs::read_dir(download_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip directories and .part files
        if path.is_dir() {
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "part") {
            continue;
        }

        let meta = entry.metadata().await?;
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = now.duration_since(modified) {
                if age > max_age {
                    let size = meta.len();
                    tokio::fs::remove_file(&path).await?;

                    // Also remove companion .etag file so the downloader
                    // won't send a stale If-None-Match on the next run.
                    if let Some(name) = path.file_name() {
                        let etag_path =
                            path.with_file_name(format!("{}.etag", name.to_string_lossy()));
                        let _ = tokio::fs::remove_file(&etag_path).await;
                    }

                    files_deleted += 1;
                    bytes_reclaimed += size;
                }
            }
        }
    }

    tracing::debug!(
        files_deleted,
        bytes_reclaimed,
        max_age_days,
        "purge complete"
    );

    Ok(PurgeResult {
        files_deleted,
        bytes_reclaimed,
    })
}
