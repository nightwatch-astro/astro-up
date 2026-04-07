use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::broadcast;

use astro_up_core::catalog::CatalogFilter;
use astro_up_core::config;
use astro_up_core::detect::DetectionResult;
use astro_up_core::detect::scanner::Scanner;
use astro_up_core::events::Event;

use crate::state::{AppState, OperationId};
use astro_up_core::adapters::{CatalogPackageSource, SqliteLedgerStore};

/// Payload emitted to the frontend when the orchestrator needs asset selection.
#[derive(Debug, Clone, Serialize)]
pub struct AssetSelectionRequest {
    pub package_name: String,
    pub assets: Vec<AssetOption>,
}

/// A single asset option for the selection dialog.
#[derive(Debug, Clone, Serialize)]
pub struct AssetOption {
    pub index: usize,
    pub name: String,
    pub size: u64,
}

/// Frontend response to an asset selection request.
#[derive(Debug, Deserialize)]
pub struct AssetSelectionResponse {
    /// Selected asset index, or null to cancel.
    pub index: Option<usize>,
}

/// Error type returned to the frontend via Tauri invoke.
#[derive(Debug, Clone, Serialize)]
pub struct CoreError {
    pub message: String,
    pub code: String,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl From<astro_up_core::error::CoreError> for CoreError {
    fn from(e: astro_up_core::error::CoreError) -> Self {
        Self {
            message: e.to_string(),
            code: "core_error".into(),
        }
    }
}

impl From<String> for CoreError {
    fn from(s: String) -> Self {
        Self {
            message: s,
            code: "error".into(),
        }
    }
}

/// Forward a core event to the frontend on the "core-event" channel.
pub fn emit_event(app: &AppHandle, event: &Event) {
    if let Err(e) = app.emit("core-event", event) {
        tracing::warn!("Failed to emit core event: {e}");
    }
}

/// Spawn a task that forwards events from a broadcast channel to the frontend.
fn forward_events(app: AppHandle, mut rx: broadcast::Receiver<Event>) {
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = rx.recv().await {
            emit_event(&app, &event);
        }
    });
}

// --- Catalog sync ---

#[tauri::command]
pub async fn sync_catalog(
    app: AppHandle,
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<String, CoreError> {
    tracing::info!(
        command = "sync_catalog",
        force = force.unwrap_or(false),
        "Syncing catalog..."
    );
    if let Err(e) = app.emit("catalog-status", "syncing") {
        tracing::debug!("failed to emit catalog-status syncing: {e}");
    }

    let result = if force.unwrap_or(false) {
        state.catalog_manager.refresh().await
    } else {
        state.catalog_manager.ensure_catalog().await
    };

    match result {
        Ok(result) => {
            let status = format!("{result:?}");
            tracing::info!(command = "sync_catalog", result = %status, "Catalog sync complete");
            if let Err(e) = app.emit("catalog-status", "ready") {
                tracing::debug!("failed to emit catalog-status ready: {e}");
            }
            Ok(status)
        }
        Err(e) => {
            tracing::error!(command = "sync_catalog", error = %e, "Catalog sync failed");
            if let Err(emit_err) = app.emit("catalog-status", "error") {
                tracing::debug!("failed to emit catalog-status error: {emit_err}");
            }
            Err(CoreError::from(e))
        }
    }
}

// --- Read commands (wired to core) ---

#[tauri::command]
pub async fn list_software(
    state: State<'_, AppState>,
    filter: String,
) -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "list_software", filter, "Command invoked");

    let query_result = try_list_software(&state, &filter);

    match query_result {
        Ok(value) => {
            tracing::debug!(
                command = "list_software",
                duration_ms = start.elapsed().as_millis() as u64,
                "Command completed"
            );
            Ok(value)
        }
        Err(e) => {
            // Catalog may be corrupt or have old schema — delete and re-sync
            tracing::warn!(command = "list_software", error = %e, "Query failed, attempting catalog recovery");
            let catalog_path = state.catalog_manager.catalog_path().to_path_buf();
            if catalog_path.exists() {
                if let Err(e) = std::fs::remove_file(&catalog_path) {
                    tracing::warn!(path = %catalog_path.display(), error = %e, "failed to remove corrupt catalog");
                }
                // Also remove sidecar so ensure_catalog fetches fresh
                let meta_path = catalog_path.with_extension("db.meta");
                if let Err(e) = std::fs::remove_file(&meta_path) {
                    tracing::warn!(path = %meta_path.display(), error = %e, "failed to remove catalog metadata");
                }
                tracing::info!(
                    command = "list_software",
                    "Deleted corrupt catalog, will re-sync on next attempt"
                );
            }
            Err(e.into())
        }
    }
}

/// Try to open the catalog and run the list query.
fn try_list_software(
    state: &AppState,
    filter: &str,
) -> Result<serde_json::Value, astro_up_core::error::CoreError> {
    let reader = state.open_catalog_reader()?;
    let packages = match filter {
        "all" => reader.list_all()?,
        f if f.starts_with("category:") => {
            let category = f.strip_prefix("category:").unwrap();
            let cat_filter = CatalogFilter {
                category: category.parse().ok(),
                ..CatalogFilter::default()
            };
            reader.filter(&cat_filter)?
        }
        _ => reader.list_all()?,
    };
    tracing::debug!(
        command = "list_software",
        count = packages.len(),
        "Query OK"
    );
    serde_json::to_value(&packages)
        .map_err(|e| astro_up_core::error::CoreError::Database(format!("serialization error: {e}")))
}

#[tauri::command]
pub async fn get_versions(
    state: State<'_, AppState>,
    id: String,
) -> Result<serde_json::Value, CoreError> {
    tracing::debug!(command = "get_versions", id, "Command invoked");
    let reader = state.open_catalog_reader()?;
    let pkg_id: astro_up_core::catalog::PackageId = id
        .parse()
        .map_err(|e: astro_up_core::error::CoreError| CoreError::from(e))?;
    let versions = reader.versions(&pkg_id)?;
    tracing::debug!(command = "get_versions", count = versions.len(), "Command completed");
    serde_json::to_value(&versions).map_err(|e| CoreError::from(e.to_string()))
}

#[tauri::command]
pub async fn search_catalog(
    state: State<'_, AppState>,
    query: String,
) -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "search_catalog", query, "Command invoked");

    let reader = state.open_catalog_reader()?;
    let results = reader.search(&query)?;
    // SearchResult doesn't derive Serialize, so map to value manually
    let value: Vec<serde_json::Value> = results
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "package": serde_json::to_value(&r.package).unwrap_or_default(),
                "rank": r.rank,
            })
        })
        .collect();

    tracing::debug!(
        command = "search_catalog",
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(serde_json::Value::Array(value))
}

#[tauri::command]
pub async fn check_for_updates(state: State<'_, AppState>) -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "check_for_updates", "Command invoked");

    let catalog_path = state.catalog_manager.catalog_path().to_path_buf();
    let packages = CatalogPackageSource::new(catalog_path.clone());
    let ledger = SqliteLedgerStore::new(state.db_path.clone());
    let scanner = Scanner::new(packages, ledger);

    // Run scan to get current detection state
    let scan_result = scanner.scan().await.map_err(|e| CoreError {
        message: e.to_string(),
        code: "scan_error".into(),
    })?;

    // Compare installed versions with latest catalog versions
    let reader = state.open_catalog_reader()?;
    let mut updates = Vec::new();

    for detection in &scan_result.results {
        if let DetectionResult::Installed { ref version, .. } = detection.result {
            let pkg_id = detection
                .package_id
                .parse()
                .map_err(|e: astro_up_core::error::CoreError| CoreError::from(e.to_string()))?;
            if let Ok(Some(latest)) = reader.latest_version(&pkg_id) {
                let latest_ver = astro_up_core::types::Version::parse(&latest.version);
                if latest_ver > *version {
                    updates.push(serde_json::json!({
                        "id": detection.package_id,
                        "current_version": version.to_string(),
                        "latest_version": latest.version,
                    }));
                }
            }
        }
    }

    tracing::info!(
        command = "check_for_updates",
        update_count = updates.len(),
        duration_ms = start.elapsed().as_millis() as u64,
        "Update check complete"
    );
    Ok(serde_json::Value::Array(updates))
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "get_config", "Command invoked");

    let config = state.config.lock().unwrap().clone();
    let value = serde_json::to_value(&config).map_err(|e| CoreError::from(e.to_string()))?;

    tracing::debug!(
        command = "get_config",
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(value)
}

// --- Write commands (wired to core) ---

#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>,
    config: serde_json::Value,
) -> Result<(), CoreError> {
    let start = std::time::Instant::now();
    tracing::info!(command = "save_config", "Saving configuration...");

    // Extract key-value pairs from the JSON and write to config store
    let store = state.open_config_store()?;
    let current = state.config.lock().unwrap().clone();

    if let Some(obj) = config.as_object() {
        for (section, values) in obj {
            if let Some(inner) = values.as_object() {
                for (key, value) in inner {
                    // Skip null values — they represent "use default"
                    if value.is_null() {
                        continue;
                    }
                    let dotpath = format!("{section}.{key}");
                    let str_value = match value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        other => other.to_string(),
                    };
                    // Skip empty strings for path-type fields
                    if str_value.is_empty()
                        && (dotpath.ends_with("_dir") || dotpath.ends_with("_file"))
                    {
                        continue;
                    }
                    if let Err(e) = config::config_set(&store, &current, &dotpath, &str_value) {
                        tracing::warn!(key = dotpath, error = %e, "Failed to save config key");
                    }
                }
            }
        }
    }

    // Reload config to pick up changes
    let paths = current.paths.clone();
    let log_file = current.logging.log_file;
    let new_config = config::load_config(&state.db_path, paths, log_file, &[])?;
    *state.config.lock().unwrap() = new_config;

    tracing::debug!(
        command = "save_config",
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(())
}

// --- Long-running operation commands ---

#[tauri::command]
pub async fn scan_installed(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, _token) = state.register_operation();
    tracing::info!(
        command = "scan_installed",
        operation_id = op_id.id,
        "Scanning installed software..."
    );

    // Emit scan_started event
    emit_event(&app, &Event::ScanStarted);

    let catalog_path = state.catalog_manager.catalog_path().to_path_buf();
    let packages = CatalogPackageSource::new(catalog_path);
    let ledger = SqliteLedgerStore::new(state.db_path.clone());
    let scanner = Scanner::new(packages, ledger);

    let scan_result = scanner.scan().await.map_err(|e| CoreError {
        message: e.to_string(),
        code: "scan_error".into(),
    })?;

    let total_found = scan_result
        .results
        .iter()
        .filter(|r| r.result.is_installed())
        .count();

    // Emit scan_complete event
    emit_event(
        &app,
        &Event::ScanComplete {
            total_found: total_found as u32,
        },
    );

    let value = serde_json::to_value(&scan_result).map_err(|e| CoreError::from(e.to_string()))?;

    state.remove_operation(&op_id.id);
    tracing::info!(
        command = "scan_installed",
        operation_id = op_id.id,
        total_found,
        duration_ms = start.elapsed().as_millis() as u64,
        "Scan complete"
    );
    Ok(value)
}

/// Shared helper: create an orchestrator, plan, and execute.
/// Pass an empty slice for `ids` to plan all available updates.
///
/// Runs inside `spawn_blocking` + `Handle::block_on` because the
/// `InstallerService` uses Windows APIs with `!Send` types (PCWSTR, HANDLE).
async fn run_orchestrated_operation(
    app: &AppHandle,
    state: &AppState,
    ids: &[&str],
    _op_id: &OperationId,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<serde_json::Value, CoreError> {
    use astro_up_core::backup::BackupService;
    use astro_up_core::download::DownloadManager;
    use astro_up_core::engine::orchestrator::{Orchestrator, UpdateRequest};
    use astro_up_core::install::InstallerService;

    let catalog_path = state.catalog_manager.catalog_path().to_path_buf();
    let db_path = state.db_path.clone();
    let config = state.config.lock().unwrap().clone();
    let backup_dir = state
        .db_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("backups");
    let lock_path = state
        .db_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("orchestration.lock");
    let pkg_ids: Vec<astro_up_core::catalog::PackageId> = ids
        .iter()
        .map(|id| {
            id.parse()
                .map_err(|e: astro_up_core::error::CoreError| CoreError::from(e))
        })
        .collect::<Result<_, _>>()?;

    let app_for_events = app.clone();
    let (event_tx, rx) = broadcast::channel::<Event>(64);
    forward_events(app.clone(), rx);

    // Clone the pending asset tx Arc so it can move into spawn_blocking
    let pending_asset_tx = state.pending_asset_tx.clone();

    let handle = tokio::runtime::Handle::current();
    tokio::task::spawn_blocking(move || {
        handle.block_on(async move {
            let packages = CatalogPackageSource::new(catalog_path);
            let ledger = SqliteLedgerStore::new(db_path.clone());
            let downloader = DownloadManager::new(&config.network, event_tx)?;
            let installer = InstallerService::new(
                std::time::Duration::from_secs(600),
                std::env::temp_dir().join("astro-up").join("installs"),
            );
            let backup = BackupService::new(backup_dir, 5);
            let db_conn =
                rusqlite::Connection::open(&db_path).map_err(|e| CoreError::from(e.to_string()))?;
            let db = std::sync::Arc::new(std::sync::Mutex::new(db_conn));

            let download_dir = if config.paths.download_dir.as_os_str().is_empty() {
                std::env::temp_dir().join("astro-up").join("downloads")
            } else {
                config.paths.download_dir.clone()
            };
            let orchestrator = astro_up_core::engine::orchestrator::UpdateOrchestrator::new(
                &lock_path,
                packages,
                ledger,
                downloader,
                installer,
                backup,
                db,
                download_dir,
            )?;

            let plan = orchestrator
                .plan(UpdateRequest {
                    packages: pkg_ids,
                    allow_major: false,
                    allow_downgrade: false,
                    dry_run: false,
                    confirmed: true,
                })
                .await?;

            // Clone before move into on_event closure
            let app_for_assets = app_for_events.clone();

            let on_event: astro_up_core::engine::orchestrator::EventCallback =
                Box::new(move |event| {
                    emit_event(&app_for_events, &event);
                });

            // Asset selector: emit event to frontend, block until user responds
            let pending_tx = pending_asset_tx;
            let asset_selector: astro_up_core::engine::orchestrator::AssetSelector =
                Box::new(move |package_name, assets| {
                    if assets.len() <= 1 {
                        return Some(0);
                    }

                    // Create a one-shot channel for this selection
                    let (tx, rx) = std::sync::mpsc::channel();

                    // Store the sender so resolve_asset_selection can find it
                    *pending_tx.lock().unwrap() = Some(tx);

                    // Emit event to frontend with asset options
                    let request = AssetSelectionRequest {
                        package_name: package_name.to_string(),
                        assets: assets
                            .iter()
                            .enumerate()
                            .map(|(i, a)| AssetOption {
                                index: i,
                                name: a.name.clone(),
                                size: a.size,
                            })
                            .collect(),
                    };
                    if let Err(e) = app_for_assets.emit("asset-selection-required", &request) {
                        tracing::error!("Failed to emit asset selection event: {e}");
                        return Some(0);
                    }

                    tracing::info!(
                        package = package_name,
                        count = assets.len(),
                        "waiting for user to select asset"
                    );

                    // Block until the frontend responds (30s timeout)
                    let result = match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                        Ok(choice) => choice,
                        Err(_) => {
                            tracing::warn!(
                                package = package_name,
                                "asset selection timed out, auto-picking first"
                            );
                            Some(0)
                        }
                    };

                    // Clear the pending sender
                    *pending_tx.lock().unwrap() = None;
                    result
                });

            let result = orchestrator
                .execute(plan, on_event, Some(asset_selector), cancel_token)
                .await?;
            serde_json::to_value(&result).map_err(|e| CoreError::from(e.to_string()))
        })
    })
    .await
    .map_err(|e| CoreError::from(e.to_string()))?
}

/// Tauri command: resolve a pending asset selection from the frontend dialog.
#[tauri::command]
pub async fn resolve_asset_selection(
    state: State<'_, AppState>,
    response: AssetSelectionResponse,
) -> Result<(), CoreError> {
    let tx = state.pending_asset_tx.lock().unwrap().take();
    if let Some(tx) = tx {
        let _ = tx.send(response.index);
        tracing::info!(index = ?response.index, "asset selection resolved");
        Ok(())
    } else {
        Err(CoreError::from("no pending asset selection".to_string()))
    }
}

#[tauri::command]
pub async fn install_software(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<OperationId, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, token) = state.register_operation();
    tracing::info!(
        command = "install_software",
        package = id,
        operation_id = op_id.id,
        "Command invoked"
    );

    match run_orchestrated_operation(&app, &state, &[&id], &op_id, token).await {
        Ok(_) => {
            tracing::info!(command = "install_software", package = id, "Completed");
        }
        Err(e) => {
            tracing::error!(command = "install_software", package = id, error = %e, "Failed");
            emit_event(
                &app,
                &Event::InstallFailed {
                    id: id.clone(),
                    error: e.message,
                },
            );
        }
    }
    state.remove_operation(&op_id.id);

    tracing::debug!(
        command = "install_software",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn update_software(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<OperationId, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, token) = state.register_operation();
    tracing::info!(
        command = "update_software",
        package = id,
        operation_id = op_id.id,
        "Command invoked"
    );

    match run_orchestrated_operation(&app, &state, &[&id], &op_id, token).await {
        Ok(_) => {
            tracing::info!(command = "update_software", package = id, "Completed");
        }
        Err(e) => {
            tracing::error!(command = "update_software", package = id, error = %e, "Failed");
            emit_event(
                &app,
                &Event::InstallFailed {
                    id: id.clone(),
                    error: e.message,
                },
            );
        }
    }
    state.remove_operation(&op_id.id);

    tracing::debug!(
        command = "update_software",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn update_all(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<OperationId, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, token) = state.register_operation();
    tracing::info!(
        command = "update_all",
        operation_id = op_id.id,
        "Updating all packages..."
    );

    match run_orchestrated_operation(&app, &state, &[], &op_id, token).await {
        Ok(_) => {
            tracing::info!(command = "update_all", "Completed");
        }
        Err(e) => {
            tracing::error!(command = "update_all", error = %e, "Failed");
            emit_event(
                &app,
                &Event::Error {
                    id: "update-all".into(),
                    error: e.message,
                },
            );
        }
    }
    state.remove_operation(&op_id.id);

    tracing::debug!(
        command = "update_all",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn create_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, _token) = state.register_operation();
    tracing::info!(
        command = "create_backup",
        operation_id = op_id.id,
        "Creating backup..."
    );

    let (tx, rx) = broadcast::channel::<Event>(64);
    forward_events(app, rx);

    let request = astro_up_core::backup::types::BackupRequest {
        package_id: "manual".into(),
        version: astro_up_core::types::Version::parse("0.0.0"),
        config_paths: paths.into_iter().map(std::path::PathBuf::from).collect(),
        event_tx: tx,
    };

    let metadata = state.backup_service.backup(&request).await?;
    state.remove_operation(&op_id.id);

    let value = serde_json::to_value(&metadata).map_err(|e| CoreError::from(e.to_string()))?;

    tracing::debug!(
        command = "create_backup",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(value)
}

#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    archive: String,
    filter: Option<Vec<String>>,
) -> Result<(), CoreError> {
    let start = std::time::Instant::now();
    let (op_id, _token) = state.register_operation();
    tracing::info!(
        command = "restore_backup",
        archive,
        operation_id = op_id.id,
        "Command invoked"
    );

    let (tx, rx) = broadcast::channel::<Event>(64);
    forward_events(app, rx);

    let request = astro_up_core::backup::types::RestoreRequest {
        archive_path: std::path::PathBuf::from(&archive),
        path_filter: filter.map(|f| f.join(",")),
        current_version: None,
        event_tx: tx,
    };

    state.backup_service.restore(&request).await?;
    state.remove_operation(&op_id.id);

    tracing::debug!(
        command = "restore_backup",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(())
}

#[tauri::command]
pub async fn cancel_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(
        command = "cancel_operation",
        operation_id,
        "Command invoked"
    );
    let result = if state.cancel_operation(&operation_id) {
        tracing::info!(operation_id, "Operation cancelled");
        Ok(())
    } else {
        Err(CoreError {
            message: format!("Operation {operation_id} not found"),
            code: "not_found".into(),
        })
    };
    tracing::debug!(
        command = "cancel_operation",
        duration_ms = start.elapsed().as_millis() as u64,
        success = result.is_ok(),
        "Command completed"
    );
    result
}

// --- Backup CRUD commands (#508) ---

#[tauri::command]
pub async fn list_backups(
    state: State<'_, AppState>,
    package_id: String,
) -> Result<serde_json::Value, CoreError> {
    tracing::debug!(command = "list_backups", package_id, "Command invoked");
    let entries = state.backup_service.list(&package_id).await?;
    tracing::debug!(command = "list_backups", count = entries.len(), "Command completed");
    let value = serde_json::to_value(&entries).map_err(|e| CoreError::from(e.to_string()))?;
    Ok(value)
}

#[tauri::command]
pub async fn backup_preview(
    state: State<'_, AppState>,
    archive: String,
) -> Result<serde_json::Value, CoreError> {
    tracing::debug!(command = "backup_preview", archive, "Command invoked");
    let preview = state
        .backup_service
        .restore_preview(std::path::Path::new(&archive))
        .await?;
    let value = serde_json::to_value(&preview).map_err(|e| CoreError::from(e.to_string()))?;
    tracing::debug!(command = "backup_preview", "Command completed");
    Ok(value)
}

#[tauri::command]
pub async fn delete_backup(archive: String) -> Result<(), CoreError> {
    tracing::info!(command = "delete_backup", archive, "Deleting backup...");
    tokio::fs::remove_file(&archive)
        .await
        .map_err(|e| CoreError {
            message: format!("Failed to delete backup: {e}"),
            code: "io_error".into(),
        })?;
    tracing::info!(command = "delete_backup", archive, "Backup deleted");
    Ok(())
}

#[tauri::command]
pub async fn clear_directory(state: State<'_, AppState>, dir: String) -> Result<(), CoreError> {
    let _state = &state; // keep State in scope for future config-based path resolution
    let path = if dir.is_empty() {
        return Err(CoreError {
            message: "No directory specified".into(),
            code: "invalid_input".into(),
        });
    } else {
        std::path::PathBuf::from(&dir)
    };

    tracing::info!(command = "clear_directory", path = %path.display(), "Clearing directory...");

    if !path.exists() {
        return Ok(());
    }

    let mut count = 0u32;
    let mut entries = tokio::fs::read_dir(&path).await.map_err(|e| CoreError {
        message: format!("Failed to read directory: {e}"),
        code: "io_error".into(),
    })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| CoreError {
        message: format!("Failed to read entry: {e}"),
        code: "io_error".into(),
    })? {
        let entry_path = entry.path();
        if entry_path.is_file() {
            tokio::fs::remove_file(&entry_path).await.ok();
            count += 1;
        }
    }

    tracing::info!(command = "clear_directory", count, "Cleared files");
    Ok(())
}
