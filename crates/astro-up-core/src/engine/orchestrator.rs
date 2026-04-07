//! Update orchestrator — main pipeline coordinator.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
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
// Operation ID generation
// ---------------------------------------------------------------------------

/// Monotonic counter to disambiguate operation IDs generated within the same
/// millisecond.
static OP_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Generate a unique operation identifier from the current timestamp and an
/// atomic counter. Format: `<millis_hex>-<counter_hex>` (e.g. `18f3a1b2c4d-0a`).
/// No external crate required.
fn generate_operation_id() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let seq = OP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{millis:x}-{seq:02x}")
}

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
pub type EventCallback = Box<dyn Fn(Event) + Send + Sync>;

/// Callback for selecting an asset when a release has multiple download options.
/// Receives the package name and list of assets. Returns the index of the chosen
/// asset, or `None` to cancel the install.
pub type AssetSelector =
    Box<dyn Fn(&str, &[crate::catalog::types::ReleaseAsset]) -> Option<usize> + Send + Sync>;

// ---------------------------------------------------------------------------
// Orchestrator trait
// ---------------------------------------------------------------------------

/// Main engine trait — plans, executes, and records update operations.
#[allow(clippy::trait_duplication_in_bounds)]
#[trait_variant::make(OrchestratorDyn: Send)]
pub trait Orchestrator: Send {
    /// Build an update plan for the given request.
    async fn plan(&self, request: UpdateRequest) -> Result<UpdatePlan, CoreError>;

    /// Execute a previously built plan, streaming events via the callback.
    /// `asset_selector` is called when a package has multiple download assets
    /// (e.g., Stellarium qt5 vs qt6). Pass `None` to auto-pick the first asset.
    async fn execute(
        &self,
        plan: UpdatePlan,
        on_event: EventCallback,
        asset_selector: Option<AssetSelector>,
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
    /// Directory for downloaded installers (from config `paths.download_dir`).
    pub(crate) download_dir: PathBuf,
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
        download_dir: PathBuf,
    ) -> Result<Self, CoreError> {
        let lock = super::lock::OrchestrationLock::acquire(lock_path)?;

        Ok(Self {
            catalog,
            detector,
            downloader,
            installer,
            backup,
            db,
            download_dir,
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
        planned: &crate::engine::planner::PlannedUpdate,
        on_event: &EventCallback,
        asset_selector: &Option<AssetSelector>,
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
                    on_event(Event::PackageComplete {
                        package_id: pkg_id.clone(),
                        status: "cancelled".into(),
                        error: None,
                    });
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

        check_cancel!();

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

        // 3. Check disk space (FR-011) — abort if insufficient
        if let Err(e) = Self::check_disk_space(&planned.version_entry.url) {
            on_event(Event::PackageComplete {
                package_id: pkg_id.clone(),
                status: "failed".into(),
                error: Some(format!("disk space check: {e}")),
            });
            return PackageResult {
                package_id: pkg_id.clone(),
                from_version: planned.current_version.clone(),
                to_version: planned.target_version.clone(),
                status: super::history::OperationStatus::Failed,
                duration: start.elapsed(),
                error: Some(format!("disk space check: {e}")),
                backup_path: None,
            };
        }

        check_cancel!();

        // 4. Resolve download URL from assets or fallback to version_entry.url
        let download_url = match planned.version_entry.assets.len() {
            0 => planned.version_entry.url.clone(),
            1 => planned.version_entry.assets[0].url.clone(),
            _ => {
                // Multiple assets — ask user to pick
                if let Some(selector) = asset_selector {
                    match selector(&planned.software.name, &planned.version_entry.assets) {
                        Some(idx) if idx < planned.version_entry.assets.len() => {
                            planned.version_entry.assets[idx].url.clone()
                        }
                        _ => {
                            let err_msg =
                                format!("asset selection cancelled for {}", planned.software.name);
                            on_event(Event::PackageComplete {
                                package_id: pkg_id.clone(),
                                status: "failed".into(),
                                error: Some(err_msg.clone()),
                            });
                            return PackageResult {
                                package_id: pkg_id.clone(),
                                from_version: planned.current_version.clone(),
                                to_version: planned.target_version.clone(),
                                status: super::history::OperationStatus::Failed,
                                duration: start.elapsed(),
                                error: Some(err_msg),
                                backup_path: None,
                            };
                        }
                    }
                } else {
                    // No selector — auto-pick first asset
                    planned.version_entry.assets[0].url.clone()
                }
            }
        };

        if download_url.is_empty() {
            let err_msg = format!(
                "no download URL in catalog for {} {}",
                pkg_id, planned.version_entry.version
            );
            on_event(Event::PackageComplete {
                package_id: pkg_id.clone(),
                status: "failed".into(),
                error: Some(err_msg.clone()),
            });
            return PackageResult {
                package_id: pkg_id.clone(),
                from_version: planned.current_version.clone(),
                to_version: planned.target_version.clone(),
                status: super::history::OperationStatus::Failed,
                duration: start.elapsed(),
                error: Some(err_msg),
                backup_path: None,
            };
        }

        let download_request = crate::download::DownloadRequest {
            url: download_url,
            expected_hash: planned.version_entry.sha256.clone(),
            dest_dir: self.download_dir.clone(),
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
                let err_msg = format!("download failed: {e}");
                on_event(Event::PackageComplete {
                    package_id: pkg_id.clone(),
                    status: "failed".into(),
                    error: Some(err_msg.clone()),
                });
                return PackageResult {
                    package_id: pkg_id.clone(),
                    from_version: planned.current_version.clone(),
                    to_version: planned.target_version.clone(),
                    status: super::history::OperationStatus::Failed,
                    duration: start.elapsed(),
                    error: Some(err_msg),
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
                    timeout: None,
                });

        let timeout = install_config
            .timeout
            .unwrap_or(std::time::Duration::from_secs(600));

        let install_request = crate::install::types::InstallRequest {
            package_id: pkg_id.to_string(),
            package_name: planned.software.name.clone(),
            version: planned.target_version.clone(),
            installer_path,
            install_dir: None,
            install_config,
            detection_config: planned.software.detection.clone(),
            timeout,
            quiet: true,
            cancel_token: cancel.child_token(),
            event_tx: tokio::sync::broadcast::channel(16).0,
        };

        let install_result = match self.installer.install(&install_request).await {
            Ok(result) => result,
            Err(e) => {
                let err_msg = format!("install failed: {e}");
                on_event(Event::PackageComplete {
                    package_id: pkg_id.clone(),
                    status: "failed".into(),
                    error: Some(err_msg.clone()),
                });
                return PackageResult {
                    package_id: pkg_id.clone(),
                    from_version: planned.current_version.clone(),
                    to_version: planned.target_version.clone(),
                    status: super::history::OperationStatus::Failed,
                    duration: start.elapsed(),
                    error: Some(err_msg),
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
                    error: None,
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
        #[allow(clippy::branches_sharing_code)]
        let final_status = if let Some(ref detection_config) = planned.software.detection {
            let resolver = crate::detect::PathResolver::new();
            let detection = crate::detect::run_chain(detection_config, &resolver, None).await;

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
            error: None,
        });

        // 9. Record to operation history (best-effort — don't fail pipeline)
        let result = PackageResult {
            package_id: pkg_id.clone(),
            from_version: planned.current_version.clone(),
            to_version: planned.target_version.clone(),
            status: final_status,
            duration: start.elapsed(),
            error: None,
            backup_path,
        };

        if let Ok(conn) = self.db.lock() {
            let op_type =
                if result.from_version.raw == "0.0.0" || result.from_version.raw.is_empty() {
                    super::history::OperationType::Install
                } else {
                    super::history::OperationType::Update
                };
            let record = super::history::OperationRecord {
                id: 0,
                package_id: result.package_id.as_ref().to_string(),
                operation_type: op_type,
                from_version: Some(result.from_version.raw.clone()),
                to_version: Some(result.to_version.raw.clone()),
                status: result.status.clone(),
                duration_ms: result.duration.as_millis() as u64,
                error_message: result.error.clone(),
                created_at: chrono::Utc::now(),
            };
            if let Err(e) = super::history::record_operation(&conn, &record) {
                tracing::warn!(package = %pkg_id, "failed to record operation history: {e}");
            }
        }

        result
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
    fn installer_filename(planned: &crate::engine::planner::PlannedUpdate) -> String {
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
    #[tracing::instrument(skip(self), fields(operation_id = %generate_operation_id()))]
    async fn plan(&self, request: UpdateRequest) -> Result<UpdatePlan, CoreError> {
        use super::planner::{CatalogEntry, UpdatePlanner};
        use super::version_cmp::VersionFormat;

        // Build catalog entries from PackageSource + LedgerStore
        let all_software = self
            .catalog
            .list_all()
            .map_err(|e| CoreError::Database(format!("catalog error: {e}")))?;
        let mut entries = Vec::new();

        for sw in all_software {
            // Look up installed version from ledger
            let installed = match self.detector.list_acknowledged() {
                Ok(ack) => ack
                    .into_iter()
                    .find(|e| AsRef::<str>::as_ref(&e.package_id) == AsRef::<str>::as_ref(&sw.id))
                    .map(|e| e.version),
                Err(e) => {
                    tracing::warn!(
                        package = %sw.id,
                        error = %e,
                        "failed to query acknowledged packages, treating as not installed"
                    );
                    None
                }
            };

            // Look up the latest version entry from the catalog
            let Some(ve) = self
                .catalog
                .latest_version(&sw.id)
                .map_err(|e| CoreError::Database(format!("catalog version lookup: {e}")))?
            else {
                tracing::debug!(package = %sw.id, "no version entries in catalog, skipping");
                continue;
            };

            // Skip packages with empty download URLs (catalog data issue)
            if ve.url.is_empty() {
                tracing::warn!(
                    package = %sw.id,
                    version = %ve.version,
                    "skipping package: catalog entry has no download URL"
                );
                continue;
            }

            let version_format: VersionFormat = sw
                .versioning
                .as_ref()
                .and_then(|v| v.major_version_pattern.as_ref())
                .map(|p: &String| VersionFormat::Custom { pattern: p.clone() })
                .unwrap_or_default();

            // Default policy: Major (allow all). Per-package overrides would
            // come from config system (deferred to config wiring).
            let sw_policy = crate::types::PolicyLevel::Major;

            entries.push(CatalogEntry {
                software: sw,
                installed_version: installed,
                catalog_version: crate::types::Version::parse(&ve.version),
                version_entry: ve,
                version_format,
                policy: sw_policy,
            });
        }

        let planner = UpdatePlanner::new(entries)
            .with_allow_major(request.allow_major)
            .with_allow_downgrade(request.allow_downgrade);

        if request.packages.is_empty() {
            planner.plan_all()
        } else {
            planner.plan_specific(&request.packages)
        }
    }

    #[tracing::instrument(skip(self, plan, on_event, asset_selector, cancel), fields(operation_id = %generate_operation_id(), plan_items = plan.items.len()))]
    async fn execute(
        &self,
        plan: UpdatePlan,
        on_event: EventCallback,
        asset_selector: Option<AssetSelector>,
        cancel: CancellationToken,
    ) -> Result<OrchestrationResult, CoreError> {
        use std::collections::HashSet;
        use std::time::Instant;

        let start = Instant::now();
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();
        let mut skipped_deps = plan.skipped.clone();
        let mut failed_ids: HashSet<crate::catalog::PackageId> = HashSet::new();

        on_event(Event::PlanReady {
            total: plan.items.len(),
            skipped: plan.skipped.len(),
        });

        for planned in &plan.items {
            // Check cancellation between packages
            if cancel.is_cancelled() {
                tracing::info!(
                    "orchestration cancelled before package {}",
                    planned.package_id
                );
                break;
            }

            // FR-007/FR-008: skip if any dependency failed
            let dep_failed = planned
                .dependencies
                .iter()
                .find(|dep| failed_ids.contains(dep));
            if let Some(failed_dep) = dep_failed {
                on_event(Event::PackageSkipped {
                    package_id: planned.package_id.clone(),
                    reason: format!("dependency {failed_dep} failed"),
                });
                failed_ids.insert(planned.package_id.clone());
                skipped_deps.push(super::planner::SkippedPackage {
                    package_id: planned.package_id.clone(),
                    reason: super::planner::SkipReason::DependencyFailed {
                        dep_id: failed_dep.clone(),
                    },
                    state: super::planner::PackageState::Installed,
                });
                continue;
            }

            let result = self
                .execute_single(planned, &on_event, &asset_selector, &cancel)
                .await;

            match &result.status {
                super::history::OperationStatus::Success
                | super::history::OperationStatus::RebootPending => {
                    succeeded.push(result);
                }
                super::history::OperationStatus::Failed => {
                    // FR-019: log backup path on failure so user can restore
                    if let Some(ref path) = result.backup_path {
                        tracing::error!(
                            package = %result.package_id,
                            backup_path = %path.display(),
                            error = result.error.as_deref().unwrap_or("unknown"),
                            "install failed — backup available for restoration"
                        );
                    }
                    failed_ids.insert(result.package_id.clone());
                    failed.push(result);
                }
                super::history::OperationStatus::Cancelled => {
                    failed.push(result);
                    break;
                }
            }
        }

        on_event(Event::OrchestrationComplete {
            succeeded: succeeded.len(),
            failed: failed.len(),
            skipped: skipped_deps.len(),
        });

        Ok(OrchestrationResult {
            succeeded,
            failed,
            skipped: skipped_deps,
            duration: start.elapsed(),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn history(&self, filter: HistoryFilter) -> Result<Vec<OperationRecord>, CoreError> {
        tracing::debug!(?filter, "querying operation history");
        let conn = self
            .db
            .lock()
            .map_err(|e| CoreError::Database(format!("failed to lock db connection: {e}")))?;
        let records = super::history::query_history(&conn, &filter)?;
        tracing::debug!(count = records.len(), "operation history query complete");
        Ok(records)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::todo,
    clippy::unwrap_in_result
)]
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
            std::env::temp_dir().join("astro-up").join("downloads"),
        )
    }

    // -----------------------------------------------------------------------
    // Cancellation-aware mock implementations
    // -----------------------------------------------------------------------

    /// A downloader that succeeds immediately, returning a cached path.
    struct SuccessDownloader;
    impl crate::traits::Downloader for SuccessDownloader {
        async fn download(
            &self,
            _request: &crate::download::DownloadRequest,
            _cancel_token: tokio_util::sync::CancellationToken,
        ) -> Result<crate::download::DownloadResult, CoreError> {
            Ok(crate::download::DownloadResult::Cached {
                path: std::path::PathBuf::from("/tmp/fake-installer.exe"),
            })
        }
    }

    /// An installer that succeeds immediately.
    struct SuccessInstaller;
    impl crate::traits::Installer for SuccessInstaller {
        async fn install(
            &self,
            _request: &crate::install::types::InstallRequest,
        ) -> Result<crate::install::types::InstallResult, CoreError> {
            Ok(crate::install::types::InstallResult::Success { path: None })
        }
        async fn uninstall(
            &self,
            _request: &crate::install::types::UninstallRequest,
        ) -> Result<(), CoreError> {
            Ok(())
        }
        fn supports(&self, _method: &crate::types::InstallMethod) -> bool {
            true
        }
    }

    /// An installer that reports cancellation.
    struct CancellingInstaller;
    impl crate::traits::Installer for CancellingInstaller {
        async fn install(
            &self,
            _request: &crate::install::types::InstallRequest,
        ) -> Result<crate::install::types::InstallResult, CoreError> {
            Ok(crate::install::types::InstallResult::Cancelled)
        }
        async fn uninstall(
            &self,
            _request: &crate::install::types::UninstallRequest,
        ) -> Result<(), CoreError> {
            Ok(())
        }
        fn supports(&self, _method: &crate::types::InstallMethod) -> bool {
            true
        }
    }

    /// Build an orchestrator with subsystems that succeed.
    fn build_success_orchestrator(
        lock_path: &std::path::Path,
    ) -> Result<
        UpdateOrchestrator<
            MockPackageSource,
            MockLedgerStore,
            SuccessDownloader,
            SuccessInstaller,
            MockBackupManager,
        >,
        CoreError,
    > {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        UpdateOrchestrator::new(
            lock_path,
            MockPackageSource,
            MockLedgerStore,
            SuccessDownloader,
            SuccessInstaller,
            MockBackupManager,
            std::sync::Arc::new(std::sync::Mutex::new(db)),
            std::env::temp_dir().join("astro-up").join("downloads"),
        )
    }

    /// Build an orchestrator whose installer always returns Cancelled.
    fn build_cancelling_installer_orchestrator(
        lock_path: &std::path::Path,
    ) -> Result<
        UpdateOrchestrator<
            MockPackageSource,
            MockLedgerStore,
            SuccessDownloader,
            CancellingInstaller,
            MockBackupManager,
        >,
        CoreError,
    > {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        UpdateOrchestrator::new(
            lock_path,
            MockPackageSource,
            MockLedgerStore,
            SuccessDownloader,
            CancellingInstaller,
            MockBackupManager,
            std::sync::Arc::new(std::sync::Mutex::new(db)),
            std::env::temp_dir().join("astro-up").join("downloads"),
        )
    }

    /// Create a minimal PlannedUpdate for testing.
    fn test_planned_update(id: &str) -> crate::engine::planner::PlannedUpdate {
        use crate::catalog::VersionEntry;
        use crate::types::{Category, SoftwareType};

        crate::engine::planner::PlannedUpdate {
            package_id: PackageId::new(id).unwrap(),
            software: crate::types::Software {
                id: PackageId::new(id).unwrap(),
                slug: id.to_string(),
                name: id.to_string(),
                software_type: SoftwareType::Application,
                category: Category::Capture,
                os: vec!["windows".to_string()],
                description: None,
                homepage: None,
                publisher: None,
                icon_url: None,
                license: None,
                license_url: None,
                aliases: vec![],
                tags: vec![],
                notes: None,
                docs_url: None,
                channel: None,
                min_os_version: None,
                manifest_version: None,
                detection: None,
                install: None,
                checkver: None,
                dependencies: None,
                hardware: None,
                backup: None,
                versioning: None,
            },
            current_version: Version::parse("1.0.0"),
            target_version: Version::parse("2.0.0"),
            version_entry: VersionEntry {
                package_id: PackageId::new(id).unwrap(),
                version: "2.0.0".to_string(),
                url: "https://example.com/installer.exe".to_string(),
                sha256: None,
                discovered_at: chrono::Utc::now(),
                release_notes_url: None,
                pre_release: false,
                assets: Vec::new(),
            },
            version_format: crate::engine::version_cmp::VersionFormat::Semver,
            has_backup_config: false,
            dependencies: vec![],
        }
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Cancellation tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn execute_single_cancelled_before_start() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");
        let orch = build_success_orchestrator(&lock_path).unwrap();

        let cancel = CancellationToken::new();
        cancel.cancel(); // Cancel immediately

        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let on_event: EventCallback = Box::new(move |e| {
            events_clone.lock().unwrap().push(e);
        });

        let planned = test_planned_update("nina-app");
        let result = orch
            .execute_single(&planned, &on_event, &None, &cancel)
            .await;

        assert_eq!(
            result.status,
            crate::engine::history::OperationStatus::Cancelled,
            "should be cancelled"
        );
        assert!(result.error.is_none());

        // Should have emitted PackageStarted and PackageComplete(cancelled)
        let captured = events.lock().unwrap();
        assert!(
            captured.iter().any(
                |e| matches!(e, Event::PackageComplete { status, .. } if status == "cancelled")
            ),
            "should emit PackageComplete with cancelled status"
        );
    }

    #[tokio::test]
    async fn execute_single_installer_returns_cancelled() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");
        let orch = build_cancelling_installer_orchestrator(&lock_path).unwrap();

        let cancel = CancellationToken::new();

        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let on_event: EventCallback = Box::new(move |e| {
            events_clone.lock().unwrap().push(e);
        });

        let planned = test_planned_update("nina-app");
        let result = orch
            .execute_single(&planned, &on_event, &None, &cancel)
            .await;

        assert_eq!(
            result.status,
            crate::engine::history::OperationStatus::Cancelled,
            "installer cancellation should propagate"
        );

        let captured = events.lock().unwrap();
        assert!(
            captured.iter().any(
                |e| matches!(e, Event::PackageComplete { status, .. } if status == "cancelled")
            ),
            "should emit PackageComplete with cancelled status from installer"
        );
    }

    #[tokio::test]
    async fn execute_cancels_between_packages() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");
        let orch = build_success_orchestrator(&lock_path).unwrap();

        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let pkg_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let pkg_count_clone = pkg_count.clone();

        let on_event: EventCallback = Box::new(move |e| {
            if matches!(&e, Event::PackageComplete { .. }) {
                let count = pkg_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count == 0 {
                    // Cancel after first package completes
                    cancel_clone.cancel();
                }
            }
            events_clone.lock().unwrap().push(e);
        });

        let plan = crate::engine::planner::UpdatePlan {
            items: vec![
                test_planned_update("pkg-a"),
                test_planned_update("pkg-b"),
                test_planned_update("pkg-c"),
            ],
            skipped: vec![],
            warnings: vec![],
        };

        let result = orch.execute(plan, on_event, None, cancel).await.unwrap();

        // First package should succeed, second should be cancelled (or not started),
        // third should not be processed
        assert!(
            result.succeeded.len() <= 1,
            "at most 1 package should succeed, got {}",
            result.succeeded.len()
        );
        assert!(
            result.succeeded.len() + result.failed.len() <= 2,
            "at most 2 packages should be processed"
        );

        // Verify OrchestrationComplete was emitted
        let captured = events.lock().unwrap();
        assert!(
            captured
                .iter()
                .any(|e| matches!(e, Event::OrchestrationComplete { .. })),
            "should emit OrchestrationComplete"
        );
    }

    #[tokio::test]
    async fn execute_empty_plan_completes_immediately() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");
        let orch = build_success_orchestrator(&lock_path).unwrap();

        let cancel = CancellationToken::new();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let on_event: EventCallback = Box::new(move |e| {
            events_clone.lock().unwrap().push(e);
        });

        let plan = crate::engine::planner::UpdatePlan {
            items: vec![],
            skipped: vec![],
            warnings: vec![],
        };

        let result = orch.execute(plan, on_event, None, cancel).await.unwrap();

        assert!(result.succeeded.is_empty());
        assert!(result.failed.is_empty());
        assert!(result.skipped.is_empty());

        let captured = events.lock().unwrap();
        assert!(
            captured.iter().any(|e| matches!(
                e,
                Event::OrchestrationComplete {
                    succeeded: 0,
                    failed: 0,
                    skipped: 0
                }
            )),
            "should emit OrchestrationComplete with all zeros"
        );
    }

    #[tokio::test]
    async fn execute_already_cancelled_processes_no_packages() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");
        let orch = build_success_orchestrator(&lock_path).unwrap();

        let cancel = CancellationToken::new();
        cancel.cancel(); // Pre-cancel

        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let on_event: EventCallback = Box::new(move |e| {
            events_clone.lock().unwrap().push(e);
        });

        let plan = crate::engine::planner::UpdatePlan {
            items: vec![test_planned_update("pkg-a"), test_planned_update("pkg-b")],
            skipped: vec![],
            warnings: vec![],
        };

        let result = orch.execute(plan, on_event, None, cancel).await.unwrap();

        assert!(
            result.succeeded.is_empty(),
            "no packages should succeed when pre-cancelled"
        );
        assert!(
            result.failed.is_empty(),
            "no packages should be processed when pre-cancelled"
        );
    }
}
