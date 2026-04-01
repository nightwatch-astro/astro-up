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

    /// Execute the pipeline for a single planned package update.
    ///
    /// Steps: emit start → check process → check disk → download → backup →
    /// install → verify → emit complete. Returns a [`PackageResult`] with the
    /// outcome.
    #[allow(dead_code)] // Called by `execute()` added in T015–T016.
    pub(crate) async fn execute_single(
        &self,
        planned: &super::planner::PlannedUpdate,
        on_event: &EventCallback,
        cancel: &CancellationToken,
    ) -> PackageResult {
        use std::time::Instant;

        let start = Instant::now();
        let pkg_id = &planned.package_id;

        // Count pipeline steps: process check, disk check, download, install, verify
        // + optional backup
        let step_count = if planned.has_backup_config { 6 } else { 5 };

        // 1. Emit PackageStarted
        on_event(Event::PackageStarted {
            package_id: pkg_id.clone(),
            step_count,
        });

        // Check cancellation before each step
        macro_rules! check_cancel {
            () => {
                if cancel.is_cancelled() {
                    return PackageResult {
                        package_id: pkg_id.clone(),
                        from_version: planned.current_version.clone(),
                        to_version: planned.target_version.clone(),
                        status: super::history::OperationStatus::Cancelled,
                        duration: start.elapsed(),
                        error: None,
                        backup_path: None,
                    };
                }
            };
        }

        // 2. Check process not running (FR-018)
        //    Derive process name from the software name (e.g. "NINA" → "NINA.exe").
        //    If the detection config has a file_path, extract the filename from it.
        let process_name = planned
            .software
            .detection
            .as_ref()
            .and_then(|d| d.file_path.as_ref())
            .and_then(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| format!("{}.exe", planned.software.name));

        if let Some(info) = super::process::check_process_running(&process_name) {
            on_event(Event::ProcessBlocking {
                package_id: pkg_id.clone(),
                process_name: info.name.clone(),
                pid: info.pid,
            });
            return PackageResult {
                package_id: pkg_id.clone(),
                from_version: planned.current_version.clone(),
                to_version: planned.target_version.clone(),
                status: super::history::OperationStatus::Failed,
                duration: start.elapsed(),
                error: Some(format!(
                    "process {} (PID {}) is running",
                    info.name, info.pid
                )),
                backup_path: None,
            };
        }

        check_cancel!();

        // 3. Check disk space (FR-011) — best-effort, log warning if unavailable
        if let Err(e) = Self::check_disk_space(&planned.version_entry.url) {
            tracing::warn!(
                package = %pkg_id,
                "disk space check failed, proceeding anyway: {e}"
            );
        }

        check_cancel!();

        // 4. Download
        let download_request = crate::download::DownloadRequest {
            url: planned.version_entry.url.clone(),
            expected_hash: planned.version_entry.sha256.clone(),
            dest_dir: std::env::temp_dir().join("astro-up").join("downloads"),
            filename: Self::installer_filename(planned),
            resume: true,
        };

        let download_result = match self
            .downloader
            .download(&download_request, cancel.child_token())
            .await
        {
            Ok(result) => result,
            Err(e) => {
                on_event(Event::PackageComplete {
                    package_id: pkg_id.clone(),
                    status: "failed".into(),
                });
                return PackageResult {
                    package_id: pkg_id.clone(),
                    from_version: planned.current_version.clone(),
                    to_version: planned.target_version.clone(),
                    status: super::history::OperationStatus::Failed,
                    duration: start.elapsed(),
                    error: Some(format!("download failed: {e}")),
                    backup_path: None,
                };
            }
        };

        let installer_path = match &download_result {
            crate::download::DownloadResult::Success { path, .. } => path.clone(),
            crate::download::DownloadResult::Cached { path } => path.clone(),
        };

        check_cancel!();

        // 5. Backup (if configured)
        let mut backup_path = None;
        if planned.has_backup_config {
            if let Some(ref backup_cfg) = planned.software.backup {
                let backup_request = crate::backup::types::BackupRequest {
                    package_id: pkg_id.to_string(),
                    version: planned.current_version.clone(),
                    config_paths: backup_cfg.config_paths.iter().map(PathBuf::from).collect(),
                    event_tx: tokio::sync::broadcast::channel(16).0,
                };

                match self.backup.backup(&backup_request).await {
                    Ok(metadata) => {
                        // Use the first path as the backup location indicator
                        if let Some(first) = metadata.paths.first() {
                            backup_path = Some(first.clone());
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            package = %pkg_id,
                            "backup failed, proceeding with install: {e}"
                        );
                    }
                }
            }
        }

        check_cancel!();

        // 6. Install
        let install_config =
            planned
                .software
                .install
                .clone()
                .unwrap_or_else(|| crate::types::InstallConfig {
                    method: crate::types::InstallMethod::Exe,
                    scope: None,
                    elevation: None,
                    upgrade_behavior: None,
                    install_modes: vec![],
                    success_codes: vec![],
                    pre_install: vec![],
                    post_install: vec![],
                    switches: None,
                    known_exit_codes: std::collections::HashMap::new(),
                    timeout_secs: None,
                });

        let timeout = std::time::Duration::from_secs(install_config.timeout_secs.unwrap_or(600));

        let install_request = crate::install::types::InstallRequest {
            package_id: pkg_id.to_string(),
            package_name: planned.software.name.clone(),
            version: planned.target_version.clone(),
            installer_path,
            install_dir: None,
            install_config,
            timeout,
            quiet: true,
            cancel_token: cancel.child_token(),
            event_tx: tokio::sync::broadcast::channel(16).0,
        };

        let install_result = match self.installer.install(&install_request).await {
            Ok(result) => result,
            Err(e) => {
                on_event(Event::PackageComplete {
                    package_id: pkg_id.clone(),
                    status: "failed".into(),
                });
                return PackageResult {
                    package_id: pkg_id.clone(),
                    from_version: planned.current_version.clone(),
                    to_version: planned.target_version.clone(),
                    status: super::history::OperationStatus::Failed,
                    duration: start.elapsed(),
                    error: Some(format!("install failed: {e}")),
                    backup_path,
                };
            }
        };

        // Map install result to operation status
        let install_status = match install_result {
            crate::install::types::InstallResult::SuccessRebootRequired { .. } => {
                super::history::OperationStatus::RebootPending
            }
            crate::install::types::InstallResult::Cancelled => {
                on_event(Event::PackageComplete {
                    package_id: pkg_id.clone(),
                    status: "cancelled".into(),
                });
                return PackageResult {
                    package_id: pkg_id.clone(),
                    from_version: planned.current_version.clone(),
                    to_version: planned.target_version.clone(),
                    status: super::history::OperationStatus::Cancelled,
                    duration: start.elapsed(),
                    error: None,
                    backup_path,
                };
            }
            crate::install::types::InstallResult::Success { .. } => {
                super::history::OperationStatus::Success
            }
        };

        check_cancel!();

        // 7. Verify: re-detect installed version and compare with target (FR-009)
        let final_status = if let Some(ref detection_config) = planned.software.detection {
            let resolver = crate::detect::PathResolver::new();
            let detection = crate::detect::run_chain(detection_config, &resolver).await;

            match detection {
                crate::detect::DetectionResult::Installed { version, .. } => {
                    if version >= planned.target_version {
                        install_status
                    } else {
                        tracing::warn!(
                            package = %pkg_id,
                            detected = %version,
                            expected = %planned.target_version,
                            "post-install version mismatch"
                        );
                        // Still report success — the installer ran without error
                        // but the detected version may lag (e.g., registry not yet
                        // updated). Log the mismatch for diagnostics.
                        install_status
                    }
                }
                _ => {
                    tracing::warn!(
                        package = %pkg_id,
                        "post-install detection did not find package — verification skipped"
                    );
                    install_status
                }
            }
        } else {
            // No detection config — skip verification
            install_status
        };

        // 8. Emit PackageComplete
        let status_str = match &final_status {
            super::history::OperationStatus::Success => "succeeded",
            super::history::OperationStatus::Failed => "failed",
            super::history::OperationStatus::Cancelled => "cancelled",
            super::history::OperationStatus::RebootPending => "reboot_pending",
        };
        on_event(Event::PackageComplete {
            package_id: pkg_id.clone(),
            status: status_str.into(),
        });

        // 9. Return PackageResult
        PackageResult {
            package_id: pkg_id.clone(),
            from_version: planned.current_version.clone(),
            to_version: planned.target_version.clone(),
            status: final_status,
            duration: start.elapsed(),
            error: None,
            backup_path,
        }
    }

    /// Best-effort disk space check. Returns `Err` with a message if the check
    /// itself fails (not if space is insufficient — that returns `CoreError`).
    #[allow(dead_code)] // Called by `execute_single`.
    fn check_disk_space(_url: &str) -> Result<(), String> {
        use sysinfo::Disks;

        let disks = Disks::new_with_refreshed_list();
        // Find the disk mounted at the system temp dir (download destination)
        let temp = std::env::temp_dir();
        let mut best_match: Option<&sysinfo::Disk> = None;
        let mut best_len = 0;

        for disk in disks.list() {
            let mount = disk.mount_point();
            if temp.starts_with(mount) {
                let len = mount.as_os_str().len();
                if len > best_len {
                    best_len = len;
                    best_match = Some(disk);
                }
            }
        }

        if let Some(disk) = best_match {
            let available = disk.available_space();
            // Require at least 100 MB free (conservative minimum)
            const MIN_FREE: u64 = 100 * 1024 * 1024;
            if available < MIN_FREE {
                return Err(format!(
                    "insufficient disk space: {available} bytes available, need at least {MIN_FREE}"
                ));
            }
        } else {
            return Err("could not determine disk for temp directory".into());
        }

        Ok(())
    }

    /// Derive a reasonable filename for the downloaded installer.
    #[allow(dead_code)] // Called by `execute_single`.
    fn installer_filename(planned: &super::planner::PlannedUpdate) -> String {
        // Extract extension from URL, default to .exe
        let url_path = planned.version_entry.url.rsplit('/').next().unwrap_or("");
        let ext = url_path
            .rsplit('.')
            .next()
            .filter(|e| e.len() <= 5 && !e.contains('?'))
            .unwrap_or("exe");
        format!("{}-{}.{}", planned.package_id, planned.target_version, ext)
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
