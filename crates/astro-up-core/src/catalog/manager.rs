//! Catalog manager — orchestrates fetch, verify, and refresh.

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;

use crate::config::CatalogConfig;
use crate::error::CoreError;

use super::fetch::{self, FetchOutcome};
use super::lock::PidLock;
use super::reader::SqliteCatalogReader;
use super::sidecar::CatalogSidecar;
use super::types::FetchResult;
use super::verify;

/// Manages the catalog lifecycle: fetch, verify, store, refresh.
pub struct CatalogManager {
    catalog_path: PathBuf,
    lock_path: PathBuf,
    config: CatalogConfig,
}

impl CatalogManager {
    /// Create a new CatalogManager for the given data directory.
    pub fn new(data_dir: &Path, config: CatalogConfig) -> Self {
        let catalog_path = data_dir.join("catalog.db");
        let lock_path = data_dir.join("astro-up.lock");
        Self {
            catalog_path,
            lock_path,
            config,
        }
    }

    /// Ensure a valid catalog is available. Fetches if needed.
    #[tracing::instrument(skip(self))]
    pub async fn ensure_catalog(&self) -> Result<FetchResult, CoreError> {
        let sidecar_path = CatalogSidecar::path_for(&self.catalog_path);
        let sidecar = CatalogSidecar::load(&sidecar_path)?;
        let has_local = self.catalog_path.exists();

        // Check TTL
        if has_local {
            if let Some(ref sc) = sidecar {
                let age = Utc::now().signed_duration_since(sc.fetched_at);
                if age < chrono::Duration::from_std(self.config.cache_ttl).unwrap_or_default() {
                    tracing::debug!("catalog within TTL, using local");
                    return Ok(FetchResult::Unchanged);
                }
            }
        }

        // Need to fetch — acquire lock
        let _lock = PidLock::acquire(&self.lock_path)?;

        let etag = sidecar.as_ref().and_then(|s| s.etag.as_deref());
        let timeout = Duration::from_secs(30);

        match fetch::fetch_catalog(&self.config.url, etag, timeout).await {
            Ok(FetchOutcome::NotModified) => {
                // Update fetched_at to reset TTL
                let new_sidecar = CatalogSidecar {
                    etag: sidecar.and_then(|s| s.etag),
                    fetched_at: Utc::now(),
                };
                new_sidecar.save(&sidecar_path)?;
                Ok(FetchResult::Unchanged)
            }
            Ok(FetchOutcome::Downloaded {
                catalog_bytes,
                sig_bytes,
                etag: new_etag,
            }) => {
                // Verify signature in memory BEFORE writing to disk,
                // so the previous valid catalog is preserved on failure.
                match verify::verify_bytes(&catalog_bytes, &sig_bytes) {
                    Ok(()) => {
                        tracing::info!("signature verified, saving catalog");
                        fetch::save_fetched(&self.catalog_path, &catalog_bytes, &sig_bytes)?;
                        let new_sidecar = CatalogSidecar {
                            etag: new_etag,
                            fetched_at: Utc::now(),
                        };
                        new_sidecar.save(&sidecar_path)?;
                        Ok(FetchResult::Updated)
                    }
                    Err(e) => {
                        tracing::error!("signature verification failed, keeping previous catalog: {e}");
                        if has_local {
                            tracing::warn!("falling back to existing local catalog");
                            Ok(FetchResult::FallbackToLocal {
                                reason: "signature verification failed on downloaded catalog".into(),
                            })
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            Err(e) => {
                if has_local {
                    tracing::warn!("catalog fetch failed, using local: {e}");
                    Ok(FetchResult::FallbackToLocal {
                        reason: e.to_string(),
                    })
                } else {
                    Err(CoreError::CatalogNotAvailable)
                }
            }
        }
    }

    /// Force a refresh regardless of TTL.
    pub async fn refresh(&self) -> Result<FetchResult, CoreError> {
        let _lock = PidLock::acquire(&self.lock_path)?;
        let sidecar_path = CatalogSidecar::path_for(&self.catalog_path);
        let timeout = Duration::from_secs(30);

        // Fetch without ETag to force download
        match fetch::fetch_catalog(&self.config.url, None, timeout).await {
            Ok(FetchOutcome::Downloaded {
                catalog_bytes,
                sig_bytes,
                etag,
            }) => {
                fetch::save_fetched(&self.catalog_path, &catalog_bytes, &sig_bytes)?;

                let sig_path = verify::sig_path_for(&self.catalog_path);
                verify::verify_catalog(&self.catalog_path, &sig_path)?;

                let new_sidecar = CatalogSidecar {
                    etag,
                    fetched_at: Utc::now(),
                };
                new_sidecar.save(&sidecar_path)?;
                Ok(FetchResult::Updated)
            }
            Ok(FetchOutcome::NotModified) => {
                // Shouldn't happen without ETag, but handle it
                Ok(FetchResult::Unchanged)
            }
            Err(e) => Err(e),
        }
    }

    /// Open the catalog reader (read-only).
    pub fn open_reader(&self) -> Result<SqliteCatalogReader, CoreError> {
        SqliteCatalogReader::open(&self.catalog_path)
    }

    /// Get the catalog file path.
    pub fn catalog_path(&self) -> &Path {
        &self.catalog_path
    }
}
