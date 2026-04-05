use std::path::{Path, PathBuf};

use color_eyre::eyre::{Result, eyre};

use astro_up_core::backup::BackupService;
use astro_up_core::catalog::{CatalogManager, SqliteCatalogReader};
use astro_up_core::config::{self, AppConfig, PathsConfig};

/// Shared CLI state, initialized once per invocation.
/// Mirrors the GUI's `AppState` pattern without Tauri dependency.
pub struct CliState {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub config: AppConfig,
    pub catalog_manager: CatalogManager,
    pub backup_service: BackupService,
}

impl CliState {
    /// Initialize from platform directories.
    pub fn new() -> Result<Self> {
        let data_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up")
            .map(|dirs| dirs.data_dir().to_owned())
            .ok_or_else(|| eyre!("could not determine data directory"))?;

        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("astro-up.db");
        let paths = PathsConfig {
            download_dir: data_dir.join("downloads"),
            cache_dir: data_dir.join("cache"),
            data_dir: data_dir.clone(),
            ..PathsConfig::default()
        };
        let log_file = data_dir.join("logs").join("astro-up.log");

        let app_config = config::load_config(&db_path, paths, log_file, &[])
            .map_err(|e| eyre!("failed to load config: {e}"))?;

        let catalog_manager = CatalogManager::new(&data_dir, app_config.catalog.clone());

        let backup_dir = data_dir.join("backups");
        let backup_service = BackupService::new(backup_dir, 5);

        Ok(Self {
            data_dir,
            db_path,
            config: app_config,
            catalog_manager,
            backup_service,
        })
    }

    /// Open a catalog reader for queries.
    pub fn open_catalog_reader(&self) -> Result<SqliteCatalogReader> {
        self.catalog_manager
            .open_reader()
            .map_err(|e| eyre!("failed to open catalog: {e}"))
    }

    /// Ensure catalog is available (download if needed), then open a reader.
    pub async fn open_catalog_reader_ensure(&self) -> Result<SqliteCatalogReader> {
        let result = self
            .catalog_manager
            .ensure_catalog()
            .await
            .map_err(|e| eyre!("catalog sync failed: {e}"))?;
        tracing::info!(?result, "catalog status");
        self.open_catalog_reader()
    }

    /// Get the catalog database path.
    pub fn catalog_path(&self) -> &Path {
        self.catalog_manager.catalog_path()
    }

    /// Open a database connection wrapped in Arc<Mutex> for the orchestrator.
    pub fn open_db(
        &self,
    ) -> Result<std::sync::Arc<std::sync::Mutex<astro_up_core::rusqlite::Connection>>> {
        let conn = astro_up_core::rusqlite::Connection::open(&self.db_path)
            .map_err(|e| eyre!("failed to open database: {e}"))?;
        Ok(std::sync::Arc::new(std::sync::Mutex::new(conn)))
    }
}
