//! Update orchestrator — main pipeline coordinator.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::catalog::PackageId;
use crate::error::CoreError;
use crate::events::Event;
use crate::types::Version;

use super::history::{OperationRecord, OperationStatus, OperationType};
use super::planner::{SkippedPackage, UpdatePlan};

// ---------------------------------------------------------------------------
// UpdateRequest
// ---------------------------------------------------------------------------

/// Parameters for an orchestrated update run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRequest {
    /// Which packages to consider for update (empty = all managed).
    pub packages: Vec<PackageId>,
    /// Allow updates that cross a major version boundary.
    pub allow_major: bool,
    /// Allow downgrades when the catalog version is older than the installed one.
    pub allow_downgrade: bool,
    /// Plan only — do not execute installers.
    pub dry_run: bool,
    /// The user has reviewed and confirmed the plan.
    pub confirmed: bool,
}

// ---------------------------------------------------------------------------
// PackageResult
// ---------------------------------------------------------------------------

/// Outcome of a single package operation within an orchestration run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageResult {
    /// Identifier of the processed package.
    pub package_id: PackageId,
    /// Version before the operation.
    pub from_version: Version,
    /// Version after the operation.
    pub to_version: Version,
    /// How the operation concluded.
    pub status: OperationStatus,
    /// Wall-clock time spent on this package.
    pub duration: Duration,
    /// Error details when `status` is `Failed`.
    pub error: Option<String>,
    /// Path to the backup created before the operation, if any.
    pub backup_path: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// OrchestrationResult
// ---------------------------------------------------------------------------

/// Aggregate outcome of an orchestrated update run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    /// Packages that completed successfully.
    pub succeeded: Vec<PackageResult>,
    /// Packages that failed during execution.
    pub failed: Vec<PackageResult>,
    /// Packages excluded from execution (policy, dependency, etc.).
    pub skipped: Vec<SkippedPackage>,
    /// Total wall-clock duration of the run.
    pub duration: Duration,
}

// ---------------------------------------------------------------------------
// HistoryFilter
// ---------------------------------------------------------------------------

/// Criteria for querying operation history.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryFilter {
    /// Restrict results to a single package.
    pub package_id: Option<PackageId>,
    /// Maximum number of records to return.
    pub limit: Option<usize>,
    /// Restrict results to a specific operation type.
    pub operation_type: Option<OperationType>,
}

// ---------------------------------------------------------------------------
// EventCallback
// ---------------------------------------------------------------------------

/// Callback for streaming engine events to the UI layer.
pub type EventCallback = Box<dyn Fn(Event) + Send>;

// ---------------------------------------------------------------------------
// Orchestrator trait
// ---------------------------------------------------------------------------

/// Main engine trait — plans, executes, and records update operations.
#[trait_variant::make(OrchestratorDyn: Send)]
pub trait Orchestrator: Send {
    /// Build an update plan for the given request.
    async fn plan(&self, request: UpdateRequest) -> Result<UpdatePlan, CoreError>;

    /// Execute a previously built plan, streaming events via the callback.
    async fn execute(
        &self,
        plan: UpdatePlan,
        on_event: EventCallback,
        cancel: CancellationToken,
    ) -> Result<OrchestrationResult, CoreError>;

    /// Query operation history from the operations table.
    async fn history(&self, filter: HistoryFilter) -> Result<Vec<OperationRecord>, CoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_request_serde_round_trip() {
        let req = UpdateRequest {
            packages: vec![PackageId::new("nina-app").unwrap()],
            allow_major: false,
            allow_downgrade: false,
            dry_run: true,
            confirmed: false,
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: UpdateRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.packages.len(), 1);
        assert!(deserialized.dry_run);
        assert!(!deserialized.confirmed);
    }

    #[test]
    fn history_filter_default() {
        let filter = HistoryFilter::default();
        assert!(filter.package_id.is_none());
        assert!(filter.limit.is_none());
        assert!(filter.operation_type.is_none());
    }

    #[test]
    fn orchestration_result_serde_round_trip() {
        let result = OrchestrationResult {
            succeeded: vec![],
            failed: vec![],
            skipped: vec![],
            duration: Duration::from_secs(42),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: OrchestrationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.duration, Duration::from_secs(42));
        assert!(deserialized.succeeded.is_empty());
    }
}
