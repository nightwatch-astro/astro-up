// Engine Service Contract — public API surface for install orchestration
//
// This contract defines the interface that CLI and GUI consume.
// All types referenced here live in astro-up-core.

/// Main entry point for install orchestration.
///
/// Coordinates the full update pipeline for one or more packages.
/// Acquires a global lock, builds an update plan, executes sequentially.
pub trait Orchestrator: Send {
    /// Build an update plan by comparing installed vs catalog versions.
    ///
    /// Does NOT execute anything — returns the plan for review or dry-run display.
    /// Resolves dependencies, applies update policies, checks version formats.
    async fn plan(
        &self,
        request: UpdateRequest,
    ) -> Result<UpdatePlan, CoreError>;

    /// Execute an update plan. Runs each package through the pipeline:
    /// compare → download → backup → install → verify.
    ///
    /// Acquires the global lock. Checks for running processes before each package.
    /// Emits events via the provided callback. Logs operations to history.
    /// Respects cancellation token. Continues on independent package failure.
    async fn execute(
        &self,
        plan: UpdatePlan,
        on_event: EventCallback,
        cancel: CancellationToken,
    ) -> Result<OrchestrationResult, CoreError>;

    /// Query operation history for a package or all packages.
    async fn history(
        &self,
        filter: HistoryFilter,
    ) -> Result<Vec<OperationRecord>, CoreError>;
}

/// What the user wants to update.
pub struct UpdateRequest {
    /// Specific package IDs, or empty for "all".
    pub packages: Vec<PackageId>,
    /// Override: allow major version upgrades for this run.
    pub allow_major: bool,
    /// Override: allow downgrades for this run.
    pub allow_downgrade: bool,
    /// Dry-run mode: build plan only, don't execute.
    pub dry_run: bool,
    /// Skip confirmation prompt (for CLI --yes flag).
    pub confirmed: bool,
}

/// Result of executing the full orchestration.
pub struct OrchestrationResult {
    pub succeeded: Vec<PackageResult>,
    pub failed: Vec<PackageResult>,
    pub skipped: Vec<SkippedPackage>,
    pub duration: Duration,
}

/// Result of a single package's pipeline execution.
pub struct PackageResult {
    pub package_id: PackageId,
    pub from_version: Version,
    pub to_version: Version,
    pub status: OperationStatus,
    pub duration: Duration,
    pub error: Option<String>,
    pub backup_path: Option<PathBuf>,
}

/// Filter for querying operation history.
pub struct HistoryFilter {
    /// Filter to specific package, or None for all.
    pub package_id: Option<PackageId>,
    /// Maximum number of records to return.
    pub limit: Option<usize>,
    /// Filter by operation type.
    pub operation_type: Option<OperationType>,
}

// Type aliases for clarity
type EventCallback = Box<dyn Fn(EngineEvent) + Send>;
