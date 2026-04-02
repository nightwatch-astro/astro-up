use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::state::{AppState, OperationId};

/// Error type returned to the frontend via Tauri invoke.
#[derive(Debug, Clone, Serialize)]
pub struct CoreError {
    pub message: String,
    pub code: String,
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
pub fn emit_event(app: &AppHandle, event: &astro_up_core::events::Event) {
    if let Err(e) = app.emit("core-event", event) {
        tracing::warn!("Failed to emit core event: {e}");
    }
}

// --- Read commands ---

#[tauri::command]
pub async fn list_software(filter: String) -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "list_software", filter, "Command invoked");
    // TODO: delegate to core crate (catalog query with filter)
    let result = Ok(serde_json::json!([]));
    tracing::debug!(
        command = "list_software",
        duration_ms = start.elapsed().as_millis() as u64,
        success = result.is_ok(),
        "Command completed"
    );
    result
}

#[tauri::command]
pub async fn search_catalog(query: String) -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "search_catalog", query, "Command invoked");
    // TODO: delegate to core crate (FTS5 search)
    let result = Ok(serde_json::json!([]));
    tracing::debug!(
        command = "search_catalog",
        duration_ms = start.elapsed().as_millis() as u64,
        success = result.is_ok(),
        "Command completed"
    );
    result
}

#[tauri::command]
pub async fn check_for_updates() -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "check_for_updates", "Command invoked");
    // TODO: delegate to core crate (compare installed vs catalog versions)
    let result = Ok(serde_json::json!([]));
    tracing::debug!(
        command = "check_for_updates",
        duration_ms = start.elapsed().as_millis() as u64,
        success = result.is_ok(),
        "Command completed"
    );
    result
}

#[tauri::command]
pub async fn get_config() -> Result<serde_json::Value, CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "get_config", "Command invoked");
    // TODO: delegate to core crate (config::load_config)
    let result = Ok(serde_json::json!({}));
    tracing::debug!(
        command = "get_config",
        duration_ms = start.elapsed().as_millis() as u64,
        success = result.is_ok(),
        "Command completed"
    );
    result
}

// --- Write commands ---

#[tauri::command]
pub async fn save_config(_config: serde_json::Value) -> Result<(), CoreError> {
    let start = std::time::Instant::now();
    tracing::debug!(command = "save_config", "Command invoked");
    // TODO: delegate to core crate (config::save_config)
    let result: Result<(), CoreError> = Ok(());
    tracing::debug!(
        command = "save_config",
        duration_ms = start.elapsed().as_millis() as u64,
        success = result.is_ok(),
        "Command completed"
    );
    result
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
    // TODO: spawn tokio task, delegate to core detect::Scanner, emit events via emit_event()
    let _ = emit_event;
    let _ = &app;
    state.remove_operation(&op_id.id);
    let result = Ok(serde_json::json!({"total_found": 0}));
    tracing::debug!(
        command = "scan_installed",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        success = result.is_ok(),
        "Command completed"
    );
    result
}

#[tauri::command]
pub async fn install_software(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<OperationId, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "install_software",
        package = id,
        operation_id = op_id.id,
        "Command invoked"
    );
    // TODO: spawn tokio task with CancellationToken, delegate to core engine::Orchestrator,
    // forward events via emit_event(&app, &event), remove operation on completion
    let _ = &app;
    tracing::debug!(
        command = "install_software",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        success = true,
        "Command completed (stub)"
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
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "update_software",
        package = id,
        operation_id = op_id.id,
        "Command invoked"
    );
    // TODO: spawn tokio task, delegate to core engine::Orchestrator
    let _ = &app;
    tracing::debug!(
        command = "update_software",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        success = true,
        "Command completed (stub)"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn create_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    _paths: Vec<String>,
) -> Result<OperationId, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "create_backup",
        operation_id = op_id.id,
        "Command invoked"
    );
    // TODO: spawn tokio task, delegate to core backup::BackupService
    let _ = &app;
    tracing::debug!(
        command = "create_backup",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        success = true,
        "Command completed (stub)"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    archive: String,
    _filter: Option<Vec<String>>,
) -> Result<OperationId, CoreError> {
    let start = std::time::Instant::now();
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "restore_backup",
        archive,
        operation_id = op_id.id,
        "Command invoked"
    );
    // TODO: spawn tokio task, delegate to core backup::BackupService
    let _ = &app;
    tracing::debug!(
        command = "restore_backup",
        operation_id = op_id.id,
        duration_ms = start.elapsed().as_millis() as u64,
        success = true,
        "Command completed (stub)"
    );
    Ok(op_id)
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
