use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

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
    /// Active operation cancellation tokens, keyed by OperationId.
    pub operations: DashMap<String, CancellationToken>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            operations: DashMap::new(),
        }
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
