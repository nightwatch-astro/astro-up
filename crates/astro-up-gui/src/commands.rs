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
fn emit_event(app: &AppHandle, event: &astro_up_core::events::Event) {
    if let Err(e) = app.emit("core-event", event) {
        tracing::warn!("Failed to emit core event: {e}");
    }
}

// --- Read commands ---

#[tauri::command]
pub async fn list_software(filter: String) -> Result<serde_json::Value, CoreError> {
    tracing::debug!(command = "list_software", filter, "Command invoked");
    // TODO: delegate to core crate once App struct is wired
    Ok(serde_json::json!([]))
}

#[tauri::command]
pub async fn search_catalog(query: String) -> Result<serde_json::Value, CoreError> {
    tracing::debug!(command = "search_catalog", query, "Command invoked");
    Ok(serde_json::json!([]))
}

#[tauri::command]
pub async fn check_for_updates() -> Result<serde_json::Value, CoreError> {
    tracing::debug!(command = "check_for_updates", "Command invoked");
    Ok(serde_json::json!([]))
}

#[tauri::command]
pub async fn get_config() -> Result<serde_json::Value, CoreError> {
    tracing::debug!(command = "get_config", "Command invoked");
    Ok(serde_json::json!({}))
}

// --- Write commands ---

#[tauri::command]
pub async fn save_config(_config: serde_json::Value) -> Result<(), CoreError> {
    tracing::debug!(command = "save_config", "Command invoked");
    Ok(())
}

// --- Long-running operation commands ---

#[tauri::command]
pub async fn scan_installed(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, CoreError> {
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "scan_installed",
        operation_id = op_id.id,
        "Command invoked"
    );
    let _ = &app; // Will use for emit_event once core is wired
    let _ = emit_event; // suppress unused warning until wired
    state.remove_operation(&op_id.id);
    Ok(serde_json::json!({"total_found": 0}))
}

#[tauri::command]
pub async fn install_software(
    _app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<OperationId, CoreError> {
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "install_software",
        package = id,
        operation_id = op_id.id,
        "Command invoked"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn update_software(
    _app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<OperationId, CoreError> {
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "update_software",
        package = id,
        operation_id = op_id.id,
        "Command invoked"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn create_backup(
    _app: AppHandle,
    state: State<'_, AppState>,
    _paths: Vec<String>,
) -> Result<OperationId, CoreError> {
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "create_backup",
        operation_id = op_id.id,
        "Command invoked"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn restore_backup(
    _app: AppHandle,
    state: State<'_, AppState>,
    archive: String,
    _filter: Option<Vec<String>>,
) -> Result<OperationId, CoreError> {
    let (op_id, _token) = state.register_operation();
    tracing::debug!(
        command = "restore_backup",
        archive,
        operation_id = op_id.id,
        "Command invoked"
    );
    Ok(op_id)
}

#[tauri::command]
pub async fn cancel_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), CoreError> {
    tracing::debug!(
        command = "cancel_operation",
        operation_id,
        "Command invoked"
    );
    if state.cancel_operation(&operation_id) {
        tracing::info!(operation_id, "Operation cancelled");
        Ok(())
    } else {
        Err(CoreError {
            message: format!("Operation {operation_id} not found"),
            code: "not_found".into(),
        })
    }
}
