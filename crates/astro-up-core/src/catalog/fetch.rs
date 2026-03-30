//! HTTP fetch — ETag conditional request, retry, download.

use std::path::Path;
use std::time::Duration;

use reqwest::StatusCode;

use crate::error::CoreError;

/// Result of a catalog fetch attempt.
#[derive(Debug)]
pub enum FetchOutcome {
    /// New catalog downloaded. Contains bytes and optional ETag.
    Downloaded {
        catalog_bytes: Vec<u8>,
        sig_bytes: Vec<u8>,
        etag: Option<String>,
    },
    /// Server returned 304 — local catalog is current.
    NotModified,
}

/// Fetch the catalog and signature from the given URL.
///
/// Sends an `If-None-Match` header if `etag` is provided. Retries once on
/// transient failure (timeout, 5xx) with a 2-second backoff.
pub async fn fetch_catalog(
    catalog_url: &str,
    etag: Option<&str>,
    timeout: Duration,
) -> Result<FetchOutcome, CoreError> {
    match fetch_catalog_inner(catalog_url, etag, timeout).await {
        Ok(outcome) => Ok(outcome),
        Err(e) if is_transient(&e) => {
            tracing::warn!("catalog fetch failed (transient), retrying in 2s: {e}");
            tokio::time::sleep(Duration::from_secs(2)).await;
            fetch_catalog_inner(catalog_url, etag, timeout).await
        }
        Err(e) => Err(e),
    }
}

async fn fetch_catalog_inner(
    catalog_url: &str,
    etag: Option<&str>,
    timeout: Duration,
) -> Result<FetchOutcome, CoreError> {
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(concat!("astro-up/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| CoreError::CatalogFetchFailed {
            reason: format!("failed to create HTTP client: {e}"),
        })?;

    // Fetch catalog
    let mut req = client.get(catalog_url);
    if let Some(etag_val) = etag {
        req = req.header("If-None-Match", etag_val);
    }

    let response = req.send().await.map_err(|e| CoreError::CatalogFetchFailed {
        reason: format!("network error: {e}"),
    })?;

    match response.status() {
        StatusCode::NOT_MODIFIED => Ok(FetchOutcome::NotModified),
        StatusCode::OK => {
            let new_etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(String::from);

            let catalog_bytes =
                response
                    .bytes()
                    .await
                    .map_err(|e| CoreError::CatalogFetchFailed {
                        reason: format!("failed to read catalog body: {e}"),
                    })?;

            // Fetch signature file (same URL + .minisig)
            let sig_url = format!("{catalog_url}.minisig");
            let sig_response =
                client
                    .get(&sig_url)
                    .send()
                    .await
                    .map_err(|e| CoreError::CatalogFetchFailed {
                        reason: format!("failed to fetch signature: {e}"),
                    })?;

            if !sig_response.status().is_success() {
                return Err(CoreError::CatalogSignatureMissing);
            }

            let sig_bytes =
                sig_response
                    .bytes()
                    .await
                    .map_err(|e| CoreError::CatalogFetchFailed {
                        reason: format!("failed to read signature body: {e}"),
                    })?;

            Ok(FetchOutcome::Downloaded {
                catalog_bytes: catalog_bytes.to_vec(),
                sig_bytes: sig_bytes.to_vec(),
                etag: new_etag,
            })
        }
        status => Err(CoreError::CatalogFetchFailed {
            reason: format!("unexpected HTTP status: {status}"),
        }),
    }
}

fn is_transient(err: &CoreError) -> bool {
    match err {
        CoreError::CatalogFetchFailed { reason } => {
            reason.contains("timeout")
                || reason.contains("timed out")
                || reason.contains("connection")
                || reason.contains("500")
                || reason.contains("502")
                || reason.contains("503")
                || reason.contains("504")
        }
        _ => false,
    }
}

/// Convenience: save fetched bytes to disk atomically.
pub fn save_fetched(catalog_path: &Path, catalog_bytes: &[u8], sig_bytes: &[u8]) -> Result<(), CoreError> {
    let sig_path = super::verify::sig_path_for(catalog_path);

    // Write to temp files first, then rename for atomicity
    let dir = catalog_path
        .parent()
        .ok_or_else(|| CoreError::CatalogFetchFailed {
            reason: "catalog path has no parent directory".into(),
        })?;
    std::fs::create_dir_all(dir)?;

    let tmp_catalog = dir.join(".catalog.db.tmp");
    let tmp_sig = dir.join(".catalog.db.minisig.tmp");

    std::fs::write(&tmp_catalog, catalog_bytes)?;
    std::fs::write(&tmp_sig, sig_bytes)?;

    std::fs::rename(&tmp_catalog, catalog_path)?;
    std::fs::rename(&tmp_sig, &sig_path)?;

    Ok(())
}
