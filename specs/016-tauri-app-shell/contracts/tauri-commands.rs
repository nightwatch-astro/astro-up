// Tauri Command Contracts — spec 016-tauri-app-shell
// These are the command signatures the frontend depends on.
// All commands are thin adapters delegating to astro-up-core.

use astro_up_core::{
    backup::{BackupInfo, RestoreResult},
    catalog::SearchResult,
    config::AppConfig,
    detect::ScanResult,
    engine::SoftwareEntry,
    events::Event,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, State};
use tokio_util::sync::CancellationToken;

/// Managed state shared across all commands.
pub struct AppState {
    pub core: astro_up_core::App,
    pub operations: dashmap::DashMap<String, CancellationToken>,
}

/// Unique identifier for a long-running operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationId {
    pub id: String,
}

/// Error type returned to frontend — serialized as JSON string.
#[derive(Debug, Clone, Serialize)]
pub struct CoreError {
    pub message: String,
    pub code: String,
}

/// Update availability info returned by check_for_updates.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateAvailable {
    pub id: String,
    pub current_version: String,
    pub latest_version: String,
}

// --- Read commands (useQuery on frontend) ---

#[tauri::command]
async fn list_software(
    state: State<'_, AppState>,
    filter: String,
) -> Result<Vec<SoftwareEntry>, CoreError>;

#[tauri::command]
async fn search_catalog(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, CoreError>;

#[tauri::command]
async fn check_for_updates(
    state: State<'_, AppState>,
) -> Result<Vec<UpdateAvailable>, CoreError>;

#[tauri::command]
async fn get_config(
    state: State<'_, AppState>,
) -> Result<AppConfig, CoreError>;

#[tauri::command]
async fn scan_installed(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ScanResult, CoreError>;

// --- Write/operation commands (useMutation on frontend) ---

#[tauri::command]
async fn install_software(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<OperationId, CoreError>;

#[tauri::command]
async fn update_software(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<OperationId, CoreError>;

#[tauri::command]
async fn create_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<BackupInfo, CoreError>;

#[tauri::command]
async fn restore_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    archive: String,
    filter: Option<Vec<String>>,
) -> Result<RestoreResult, CoreError>;

#[tauri::command]
async fn save_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), CoreError>;

#[tauri::command]
async fn cancel_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), CoreError>;
