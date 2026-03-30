pub mod client;
pub mod purge;
pub mod stream;
pub mod types;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::broadcast;

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
}
