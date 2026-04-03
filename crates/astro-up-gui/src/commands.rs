use std::fmt;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::broadcast;

use astro_up_core::catalog::CatalogFilter;
use astro_up_core::config;
use astro_up_core::detect::DetectionResult;
use astro_up_core::detect::scanner::Scanner;
use astro_up_core::events::Event;

use crate::adapters::{CatalogPackageSource, SqliteLedgerStore};
use crate::state::{AppState, OperationId};

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
pub async fn sync_catalog(app: AppHandle, state: State<'_, AppState>) -> Result<String, CoreError> {
    tracing::info!(command = "sync_catalog", "Syncing catalog...");
    let _ = app.emit("catalog-status", "syncing");

    match state.catalog_manager.ensure_catalog().await {
        Ok(result) => {
            let status = format!("{result:?}");
            tracing::info!(command = "sync_catalog", result = %status, "Catalog sync complete");
            let _ = app.emit("catalog-status", "ready");
            Ok(status)
        }
        Err(e) => {
            tracing::error!(command = "sync_catalog", error = %e, "Catalog sync failed");
            let _ = app.emit("catalog-status", "error");
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

    let reader = state.open_catalog_reader()?;
    let result = match filter.as_str() {
        "all" => serde_json::to_value(reader.list_all()?),
        f if f.starts_with("category:") => {
            let category = f.strip_prefix("category:").unwrap();
            let cat_filter = CatalogFilter {
                category: category.parse().ok(),
                ..CatalogFilter::default()
            };
            serde_json::to_value(reader.filter(&cat_filter)?)
        }
        // "installed" and "outdated" need detect module — stub for now
        _ => serde_json::to_value(reader.list_all()?),
    }
    .map_err(|e| CoreError::from(e.to_string()))?;

    tracing::debug!(
        command = "list_software",
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(result)
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

    tracing::debug!(
        command = "check_for_updates",
        update_count = updates.len(),
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
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
    tracing::debug!(command = "save_config", "Command invoked");

    // Extract key-value pairs from the JSON and write to config store
    let store = state.open_config_store()?;
    let current = state.config.lock().unwrap().clone();

    if let Some(obj) = config.as_object() {
        for (section, values) in obj {
            if let Some(inner) = values.as_object() {
                for (key, value) in inner {
                    let dotpath = format!("{section}.{key}");
                    let str_value = match value {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    config::config_set(&store, &current, &dotpath, &str_value)?;
                }
            }
        }
    }

    // Reload config to pick up changes
    let paths = current.paths.clone();
    let log_file = current.logging.log_file.clone();
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
    tracing::debug!(
        command = "scan_installed",
        operation_id = op_id.id,
        "Command invoked"
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
    tracing::debug!(
        command = "scan_installed",
        operation_id = op_id.id,
        total_found,
        duration_ms = start.elapsed().as_millis() as u64,
        "Command completed"
    );
    Ok(value)
}

/// Shared helper: create an orchestrator, plan for a single package, and execute.
async fn run_orchestrated_operation(
    app: &AppHandle,
    state: &AppState,
    id: &str,
    _op_id: &OperationId,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<serde_json::Value, CoreError> {
    use astro_up_core::backup::BackupService;
    use astro_up_core::download::DownloadManager;
    use astro_up_core::engine::orchestrator::{Orchestrator, UpdateRequest};
    use astro_up_core::install::InstallerService;

    let catalog_path = state.catalog_manager.catalog_path().to_path_buf();
    let packages = CatalogPackageSource::new(catalog_path);
    let ledger = SqliteLedgerStore::new(state.db_path.clone());

    let config = state.config.lock().unwrap().clone();
    let (event_tx, rx) = broadcast::channel::<Event>(64);
    forward_events(app.clone(), rx);

    let downloader = DownloadManager::new(&config.network, event_tx)?;
    let installer = InstallerService::new(
        std::time::Duration::from_secs(600),
        std::env::temp_dir().join("astro-up").join("installs"),
    );
    let backup_dir = state
        .db_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("backups");
    let backup = BackupService::new(backup_dir, 5);

    let db_conn =
        rusqlite::Connection::open(&state.db_path).map_err(|e| CoreError::from(e.to_string()))?;
    let db = std::sync::Arc::new(std::sync::Mutex::new(db_conn));
    let lock_path = state
        .db_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("orchestration.lock");

    let orchestrator = astro_up_core::engine::orchestrator::UpdateOrchestrator::new(
        &lock_path, packages, ledger, downloader, installer, backup, db,
    )?;

    let pkg_id: astro_up_core::catalog::PackageId = id
        .parse()
        .map_err(|e: astro_up_core::error::CoreError| CoreError::from(e))?;
    let plan = orchestrator
        .plan(UpdateRequest {
            packages: vec![pkg_id],
            allow_major: false,
            allow_downgrade: false,
            dry_run: false,
            confirmed: true,
        })
        .await?;

    let app_clone = app.clone();
    let on_event: astro_up_core::engine::orchestrator::EventCallback = Box::new(move |event| {
        emit_event(&app_clone, &event);
    });

    let result = orchestrator.execute(plan, on_event, cancel_token).await?;
    serde_json::to_value(&result).map_err(|e| CoreError::from(e.to_string()))
}

#[tauri::command]
pub async fn install_software(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<OperationId, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, token) = state.register_operation();
    tracing::debug!(
        command = "install_software",
        package = id,
        operation_id = op_id.id,
        "Command invoked"
    );

    let app_clone = app.clone();
    let op_id_clone = op_id.clone();

    tauri::async_runtime::spawn(async move {
        let state_ref = app_clone.state::<AppState>();
        match run_orchestrated_operation(&app_clone, &state_ref, &id, &op_id_clone, token).await {
            Ok(_) => {
                tracing::info!(command = "install_software", package = id, "Completed");
            }
            Err(e) => {
                tracing::error!(command = "install_software", package = id, error = %e, "Failed");
                emit_event(
                    &app_clone,
                    &Event::InstallFailed {
                        id: id.clone(),
                        error: e.message,
                    },
                );
            }
        }
        state_ref.remove_operation(&op_id_clone.id);
    });

    tracing::debug!(
        command = "install_software",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        "Command dispatched"
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
    tracing::debug!(
        command = "update_software",
        package = id,
        operation_id = op_id.id,
        "Command invoked"
    );

    let app_clone = app.clone();
    let op_id_clone = op_id.clone();

    tauri::async_runtime::spawn(async move {
        let state_ref = app_clone.state::<AppState>();
        match run_orchestrated_operation(&app_clone, &state_ref, &id, &op_id_clone, token).await {
            Ok(_) => {
                tracing::info!(command = "update_software", package = id, "Completed");
            }
            Err(e) => {
                tracing::error!(command = "update_software", package = id, error = %e, "Failed");
                emit_event(
                    &app_clone,
                    &Event::InstallFailed {
                        id: id.clone(),
                        error: e.message,
                    },
                );
            }
        }
        state_ref.remove_operation(&op_id_clone.id);
    });

    tracing::debug!(
        command = "update_software",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        "Command dispatched"
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
    tracing::debug!(
        command = "create_backup",
        operation_id = op_id.id,
        "Command invoked"
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
    tracing::debug!(
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
    Ok(value)
}

#[tauri::command]
pub async fn delete_backup(archive: String) -> Result<(), CoreError> {
    tracing::debug!(command = "delete_backup", archive, "Command invoked");
    tokio::fs::remove_file(&archive)
        .await
        .map_err(|e| CoreError {
            message: format!("Failed to delete backup: {e}"),
            code: "io_error".into(),
        })?;
    Ok(())
}
