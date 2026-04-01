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

// ---------------------------------------------------------------------------
// UpdateOrchestrator — concrete implementation
// ---------------------------------------------------------------------------

/// Concrete orchestrator that coordinates the full update pipeline.
///
/// Uses generics for each subsystem trait so async traits remain
/// dyn-compatible-free. The [`OrchestrationLock`] guard ensures only one
/// orchestrator runs at a time.
///
/// # Type parameters
///
/// - `C` — catalog / package source ([`PackageSource`](crate::detect::scanner::PackageSource))
/// - `L` — ledger store ([`LedgerStore`](crate::detect::scanner::LedgerStore))
/// - `D` — file downloader ([`Downloader`](crate::traits::Downloader))
/// - `I` — installer ([`Installer`](crate::traits::Installer))
/// - `B` — backup manager ([`BackupManager`](crate::traits::BackupManager))
pub struct UpdateOrchestrator<C, L, D, I, B>
where
    C: crate::detect::scanner::PackageSource + Send + Sync,
    L: crate::detect::scanner::LedgerStore + Send + Sync,
    D: crate::traits::Downloader + Send + Sync,
    I: crate::traits::Installer + Send + Sync,
    B: crate::traits::BackupManager + Send + Sync,
{
    // Fields are used by pipeline methods added in T014–T016.
    /// Read-only catalog access (package list, version entries).
    #[allow(dead_code)]
    pub(crate) catalog: C,
    /// Installed-version detection (ledger read/write).
    #[allow(dead_code)]
    pub(crate) detector: L,
    /// File downloader.
    #[allow(dead_code)]
    pub(crate) downloader: D,
    /// Installer / uninstaller.
    #[allow(dead_code)]
    pub(crate) installer: I,
    /// Backup & restore manager.
    #[allow(dead_code)]
    pub(crate) backup: B,
    /// Shared database connection for history recording.
    #[allow(dead_code)]
    pub(crate) db: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    /// Global orchestration lock — held for the lifetime of this struct.
    #[allow(dead_code)]
    pub(crate) lock: super::lock::OrchestrationLock,
}

impl<C, L, D, I, B> std::fmt::Debug for UpdateOrchestrator<C, L, D, I, B>
where
    C: crate::detect::scanner::PackageSource + Send + Sync,
    L: crate::detect::scanner::LedgerStore + Send + Sync,
    D: crate::traits::Downloader + Send + Sync,
    I: crate::traits::Installer + Send + Sync,
    B: crate::traits::BackupManager + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateOrchestrator")
            .field("lock", &self.lock)
            .finish_non_exhaustive()
    }
}

impl<C, L, D, I, B> UpdateOrchestrator<C, L, D, I, B>
where
    C: crate::detect::scanner::PackageSource + Send + Sync,
    L: crate::detect::scanner::LedgerStore + Send + Sync,
    D: crate::traits::Downloader + Send + Sync,
    I: crate::traits::Installer + Send + Sync,
    B: crate::traits::BackupManager + Send + Sync,
{
    /// Create a new orchestrator, acquiring the global lock.
    ///
    /// Returns [`CoreError::OrchestrationLocked`] if another instance is
    /// already running.
    pub fn new(
        lock_path: &std::path::Path,
        catalog: C,
        detector: L,
        downloader: D,
        installer: I,
        backup: B,
        db: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    ) -> Result<Self, CoreError> {
        let lock = super::lock::OrchestrationLock::acquire(lock_path)?;

        Ok(Self {
            catalog,
            detector,
            downloader,
            installer,
            backup,
            db,
            lock,
        })
    }
}

impl<C, L, D, I, B> Orchestrator for UpdateOrchestrator<C, L, D, I, B>
where
    C: crate::detect::scanner::PackageSource + Send + Sync,
    L: crate::detect::scanner::LedgerStore + Send + Sync,
    D: crate::traits::Downloader + Send + Sync,
    I: crate::traits::Installer + Send + Sync,
    B: crate::traits::BackupManager + Send + Sync,
{
    async fn plan(&self, _request: UpdateRequest) -> Result<UpdatePlan, CoreError> {
        todo!("T014–T020: planner integration")
    }

    async fn execute(
        &self,
        _plan: UpdatePlan,
        _on_event: EventCallback,
        _cancel: CancellationToken,
    ) -> Result<OrchestrationResult, CoreError> {
        todo!("T014–T016: pipeline execution")
    }

    async fn history(&self, _filter: HistoryFilter) -> Result<Vec<OperationRecord>, CoreError> {
        todo!("T036–T039: history queries")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Minimal mock implementations for UpdateOrchestrator constructor tests
    // -----------------------------------------------------------------------

    struct MockPackageSource;
    impl crate::detect::scanner::PackageSource for MockPackageSource {
        fn list_all(&self) -> Result<Vec<crate::types::Software>, crate::detect::DetectionError> {
            Ok(vec![])
        }
    }

    struct MockLedgerStore;
    impl crate::detect::scanner::LedgerStore for MockLedgerStore {
        fn list_acknowledged(
            &self,
        ) -> Result<Vec<crate::ledger::LedgerEntry>, crate::detect::DetectionError> {
            Ok(vec![])
        }
        fn upsert_acknowledged(
            &self,
            _package_id: &str,
            _version: &crate::types::Version,
        ) -> Result<(), crate::detect::DetectionError> {
            Ok(())
        }
        fn remove_acknowledged(
            &self,
            _package_id: &str,
        ) -> Result<(), crate::detect::DetectionError> {
            Ok(())
        }
    }

    struct MockDownloader;
    impl crate::traits::Downloader for MockDownloader {
        async fn download(
            &self,
            _request: &crate::download::DownloadRequest,
            _cancel_token: tokio_util::sync::CancellationToken,
        ) -> Result<crate::download::DownloadResult, CoreError> {
            todo!()
        }
    }

    struct MockInstaller;
    impl crate::traits::Installer for MockInstaller {
        async fn install(
            &self,
            _request: &crate::install::types::InstallRequest,
        ) -> Result<crate::install::types::InstallResult, CoreError> {
            todo!()
        }
        async fn uninstall(
            &self,
            _request: &crate::install::types::UninstallRequest,
        ) -> Result<(), CoreError> {
            todo!()
        }
        fn supports(&self, _method: &crate::types::InstallMethod) -> bool {
            true
        }
    }

    struct MockBackupManager;
    impl crate::traits::BackupManager for MockBackupManager {
        async fn backup(
            &self,
            _request: &crate::backup::types::BackupRequest,
        ) -> Result<crate::backup::types::BackupMetadata, CoreError> {
            todo!()
        }
        async fn restore(
            &self,
            _request: &crate::backup::types::RestoreRequest,
        ) -> Result<(), CoreError> {
            todo!()
        }
        async fn restore_preview(
            &self,
            _archive_path: &std::path::Path,
        ) -> Result<crate::backup::types::FileChangeSummary, CoreError> {
            todo!()
        }
        async fn list(
            &self,
            _package_id: &str,
        ) -> Result<Vec<crate::backup::types::BackupListEntry>, CoreError> {
            todo!()
        }
        async fn prune(&self, _package_id: &str, _keep: usize) -> Result<u32, CoreError> {
            todo!()
        }
    }

    /// Helper: build an `UpdateOrchestrator` with mock subsystems.
    fn build_orchestrator(
        lock_path: &std::path::Path,
    ) -> Result<
        UpdateOrchestrator<
            MockPackageSource,
            MockLedgerStore,
            MockDownloader,
            MockInstaller,
            MockBackupManager,
        >,
        CoreError,
    > {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        UpdateOrchestrator::new(
            lock_path,
            MockPackageSource,
            MockLedgerStore,
            MockDownloader,
            MockInstaller,
            MockBackupManager,
            std::sync::Arc::new(std::sync::Mutex::new(db)),
        )
    }

    #[test]
    fn orchestrator_new_acquires_lock() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");

        let orch = build_orchestrator(&lock_path).unwrap();
        assert!(lock_path.exists());
        assert_eq!(orch.lock.path(), lock_path);
    }

    #[test]
    fn orchestrator_second_instance_blocked() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");

        let _orch = build_orchestrator(&lock_path).unwrap();
        let result = build_orchestrator(&lock_path);
        assert!(
            result.is_err(),
            "second orchestrator should fail to acquire lock"
        );
    }

    #[test]
    fn orchestrator_debug_impl() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");

        let orch = build_orchestrator(&lock_path).unwrap();
        let debug = format!("{orch:?}");
        assert!(debug.contains("UpdateOrchestrator"));
    }

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
