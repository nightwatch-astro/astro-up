// Contract: CliState — shared state for CLI command handlers
// Mirrors the GUI's AppState pattern without Tauri dependency.

use std::path::PathBuf;

use astro_up_core::backup::BackupService;
use astro_up_core::catalog::CatalogManager;
use astro_up_core::config::AppConfig;

/// Shared CLI state, initialized once per invocation.
pub struct CliState {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub config: AppConfig,
    pub catalog_manager: CatalogManager,
    pub backup_service: BackupService,
}

impl CliState {
    /// Initialize from platform directories.
    /// Creates data_dir if it doesn't exist.
    pub fn new() -> color_eyre::eyre::Result<Self> {
        // ...
    }

    /// Open a catalog reader (after ensure_catalog).
    pub fn open_catalog_reader(&self) -> color_eyre::eyre::Result<SqliteCatalogReader> {
        // ...
    }

    /// Open a ledger store for scan result persistence.
    pub fn open_ledger(&self) -> color_eyre::eyre::Result<SqliteLedgerStore> {
        // ...
    }
}

// Command handler signatures (all take &CliState):

pub async fn handle_scan(state: &CliState, mode: &OutputMode) -> Result<()>;
pub async fn handle_install(state: &CliState, package: &str, dry_run: bool, yes: bool, mode: &OutputMode, cancel: CancellationToken) -> Result<()>;
pub async fn handle_update(state: &CliState, package: Option<&str>, all: bool, dry_run: bool, allow_major: bool, yes: bool, mode: &OutputMode, cancel: CancellationToken) -> Result<()>;
pub fn handle_show(state: &CliState, filter: Option<ShowFilter>, mode: &OutputMode) -> Result<()>;
pub async fn handle_backup(state: &CliState, package: &str, mode: &OutputMode) -> Result<()>;
pub async fn handle_self_update(dry_run: bool, mode: &OutputMode) -> Result<()>; // No state needed — standalone
