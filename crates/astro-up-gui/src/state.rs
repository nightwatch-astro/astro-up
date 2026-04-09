use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use astro_up_core::backup::BackupService;
use astro_up_core::catalog::{CatalogManager, SqliteCatalogReader};
use astro_up_core::config::{self, AppConfig, ConfigStore, PathsConfig};

/// Shared handle for the pending asset selection channel.
///
/// When the orchestrator encounters multiple assets, it creates a `mpsc` channel,
/// stores the `Sender` here, and blocks on the `Receiver`. The frontend calls
/// `resolve_asset_selection` which finds this sender and sends the user's choice.
pub type PendingAssetTx = Arc<Mutex<Option<std::sync::mpsc::Sender<Option<usize>>>>>;

/// Unique identifier for a long-running operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationId {
    pub id: String,
}

impl OperationId {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Managed state shared across all Tauri commands.
pub struct AppState {
    /// Active operation cancellation tokens, keyed by `OperationId`.
    pub operations: DashMap<String, CancellationToken>,
    /// Application data directory (used by detect/engine when wired).
    #[allow(dead_code)]
    pub data_dir: PathBuf,
    /// Application configuration.
    pub config: Mutex<AppConfig>,
    /// `SQLite` database path for config store.
    pub db_path: PathBuf,
    /// Catalog manager for fetch/query.
    pub catalog_manager: CatalogManager,
    /// Backup service.
    pub backup_service: BackupService,
    /// Channel sender for the pending asset selection dialog.
    /// The orchestrator sets this before blocking; `resolve_asset_selection` reads it.
    pub pending_asset_tx: PendingAssetTx,
    /// Set to `true` when the user explicitly requests quit (tray menu or dialog).
    /// The run loop checks this before calling `prevent_exit()`.
    pub quit_requested: AtomicBool,
}

impl AppState {
    pub fn new(data_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::debug!(data_dir = %data_dir.display(), "initializing app state");
        let db_path = data_dir.join("astro-up.db");
        let paths = PathsConfig {
            download_dir: data_dir.join("downloads"),
            cache_dir: data_dir.join("cache"),
            data_dir: data_dir.to_path_buf(),
            ..PathsConfig::default()
        };
        let log_file = data_dir.join("logs").join("astro-up.log");

        let app_config = config::load_config(&db_path, paths, log_file, &[])?;

        let catalog_manager = CatalogManager::new(data_dir, app_config.catalog.clone());

        let backup_dir = data_dir.join("backups");
        let backup_service = BackupService::new(backup_dir, 5);

        Ok(Self {
            operations: DashMap::new(),
            data_dir: data_dir.to_path_buf(),
            config: Mutex::new(app_config),
            db_path,
            catalog_manager,
            backup_service,
            pending_asset_tx: Arc::new(Mutex::new(None)),
            quit_requested: AtomicBool::new(false),
        })
    }

    /// Open a catalog reader for queries.
    pub fn open_catalog_reader(
        &self,
    ) -> Result<SqliteCatalogReader, astro_up_core::error::CoreError> {
        self.catalog_manager.open_reader()
    }

    /// Open a config store for writes.
    pub fn open_config_store(&self) -> Result<ConfigStore, astro_up_core::error::CoreError> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| astro_up_core::error::CoreError::Database(e.to_string()))?;
        ConfigStore::new(conn).map_err(|e| astro_up_core::error::CoreError::Database(e.to_string()))
    }

    /// Register a new operation and return its ID + token.
    pub fn register_operation(&self) -> (OperationId, CancellationToken) {
        let op_id = OperationId::new();
        let token = CancellationToken::new();
        self.operations.insert(op_id.id.clone(), token.clone());
        (op_id, token)
    }

    /// Cancel an operation by ID. Returns true if found and cancelled.
    pub fn cancel_operation(&self, id: &str) -> bool {
        if let Some((_, token)) = self.operations.remove(id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    /// Remove a completed operation.
    pub fn remove_operation(&self, id: &str) {
        self.operations.remove(id);
    }

    /// Check if any operations are active.
    pub fn has_active_operations(&self) -> bool {
        !self.operations.is_empty()
    }
}
