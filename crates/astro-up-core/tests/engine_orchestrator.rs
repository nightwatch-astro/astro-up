#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unwrap_in_result,
    clippy::assigning_clones
)]
//! Integration tests for the update orchestrator pipeline.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use astro_up_core::backup::types::{
    BackupListEntry, BackupMetadata, BackupRequest, FileChangeSummary, RestoreRequest,
};
use astro_up_core::catalog::{PackageId, VersionEntry};
use astro_up_core::detect::DetectionError;
use astro_up_core::detect::scanner::{LedgerStore, PackageSource};
use astro_up_core::download::{DownloadRequest, DownloadResult};
use astro_up_core::engine::history::OperationStatus;
use astro_up_core::engine::orchestrator::{EventCallback, Orchestrator, UpdateOrchestrator};
use astro_up_core::engine::planner::{PlannedUpdate, UpdatePlan};
use astro_up_core::engine::version_cmp::VersionFormat;
use astro_up_core::error::CoreError;
use astro_up_core::events::Event;
use astro_up_core::install::types::{InstallRequest, InstallResult, UninstallRequest};
use astro_up_core::ledger::LedgerEntry;
use astro_up_core::traits::{BackupManager, Downloader, Installer};
use astro_up_core::types::{
    Category, DetectionConfig, DetectionMethod, InstallMethod, Software, SoftwareType, Version,
};
use tokio_util::sync::CancellationToken;

// ---------------------------------------------------------------------------
// Mock implementations
// ---------------------------------------------------------------------------

struct MockPackageSource;
impl PackageSource for MockPackageSource {
    fn list_all(&self) -> Result<Vec<Software>, DetectionError> {
        Ok(vec![])
    }
}

struct MockLedgerStore;
impl LedgerStore for MockLedgerStore {
    fn list_acknowledged(&self) -> Result<Vec<LedgerEntry>, DetectionError> {
        Ok(vec![])
    }
    fn upsert_acknowledged(
        &self,
        _package_id: &str,
        _version: &Version,
    ) -> Result<(), DetectionError> {
        Ok(())
    }
    fn remove_acknowledged(&self, _package_id: &str) -> Result<(), DetectionError> {
        Ok(())
    }
}

struct SuccessDownloader;
impl Downloader for SuccessDownloader {
    async fn download(
        &self,
        _request: &DownloadRequest,
        _cancel_token: CancellationToken,
    ) -> Result<DownloadResult, CoreError> {
        Ok(DownloadResult::Cached {
            path: PathBuf::from("/tmp/fake-installer.exe"),
        })
    }
}

struct SuccessInstaller;
impl Installer for SuccessInstaller {
    async fn install(&self, _request: &InstallRequest) -> Result<InstallResult, CoreError> {
        Ok(InstallResult::Success { path: None })
    }
    async fn uninstall(&self, _request: &UninstallRequest) -> Result<(), CoreError> {
        Ok(())
    }
    fn supports(&self, _method: &InstallMethod) -> bool {
        true
    }
}

/// An installer that always fails with an error.
struct FailingInstaller;
impl Installer for FailingInstaller {
    async fn install(&self, _request: &InstallRequest) -> Result<InstallResult, CoreError> {
        Err(CoreError::InstallerFailed {
            exit_code: 1,
            response: astro_up_core::types::KnownExitCode::ContactSupport,
        })
    }
    async fn uninstall(&self, _request: &UninstallRequest) -> Result<(), CoreError> {
        Ok(())
    }
    fn supports(&self, _method: &InstallMethod) -> bool {
        true
    }
}

struct MockBackupManager;
impl BackupManager for MockBackupManager {
    async fn backup(&self, _request: &BackupRequest) -> Result<BackupMetadata, CoreError> {
        Ok(BackupMetadata {
            package_id: String::new(),
            version: Version::parse("0.0.0"),
            created_at: chrono::Utc::now(),
            paths: vec![],
            file_count: 0,
            total_size: 0,
            excluded_files: vec![],
            file_hashes: HashMap::new(),
        })
    }
    async fn restore(&self, _request: &RestoreRequest) -> Result<(), CoreError> {
        Ok(())
    }
    async fn restore_preview(
        &self,
        _archive_path: &std::path::Path,
    ) -> Result<FileChangeSummary, CoreError> {
        Ok(FileChangeSummary::default())
    }
    async fn list(&self, _package_id: &str) -> Result<Vec<BackupListEntry>, CoreError> {
        Ok(vec![])
    }
    async fn prune(&self, _package_id: &str, _keep: usize) -> Result<u32, CoreError> {
        Ok(0)
    }
}

/// A backup manager that returns a real backup path.
struct BackupWithPathManager;
impl BackupManager for BackupWithPathManager {
    async fn backup(&self, request: &BackupRequest) -> Result<BackupMetadata, CoreError> {
        Ok(BackupMetadata {
            package_id: request.package_id.clone(),
            version: request.version.clone(),
            created_at: chrono::Utc::now(),
            paths: vec![PathBuf::from("/tmp/backups/config-backup.zip")],
            file_count: 3,
            total_size: 1024,
            excluded_files: vec![],
            file_hashes: HashMap::new(),
        })
    }
    async fn restore(&self, _request: &RestoreRequest) -> Result<(), CoreError> {
        Ok(())
    }
    async fn restore_preview(
        &self,
        _archive_path: &std::path::Path,
    ) -> Result<FileChangeSummary, CoreError> {
        Ok(FileChangeSummary::default())
    }
    async fn list(&self, _package_id: &str) -> Result<Vec<BackupListEntry>, CoreError> {
        Ok(vec![])
    }
    async fn prune(&self, _package_id: &str, _keep: usize) -> Result<u32, CoreError> {
        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect events into a shared Vec.
fn event_collector() -> (EventCallback, Arc<Mutex<Vec<Event>>>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();
    let callback: EventCallback = Box::new(move |e| {
        events_clone.lock().unwrap().push(e);
    });
    (callback, events)
}

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
        Arc::new(std::sync::Mutex::new(db)),
        std::env::temp_dir().join("astro-up").join("downloads"),
    )
}

fn build_failing_installer_with_backup_orchestrator(
    lock_path: &std::path::Path,
) -> Result<
    UpdateOrchestrator<
        MockPackageSource,
        MockLedgerStore,
        SuccessDownloader,
        FailingInstaller,
        BackupWithPathManager,
    >,
    CoreError,
> {
    let db = rusqlite::Connection::open_in_memory().unwrap();
    UpdateOrchestrator::new(
        lock_path,
        MockPackageSource,
        MockLedgerStore,
        SuccessDownloader,
        FailingInstaller,
        BackupWithPathManager,
        Arc::new(std::sync::Mutex::new(db)),
        std::env::temp_dir().join("astro-up").join("downloads"),
    )
}

/// Create a minimal PlannedUpdate for testing.
fn test_planned_update(id: &str) -> PlannedUpdate {
    PlannedUpdate {
        package_id: PackageId::new(id).unwrap(),
        software: Software {
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
        version_format: VersionFormat::Semver,
        has_backup_config: false,
        dependencies: vec![],
    }
}

/// Create a PlannedUpdate whose detection config file_path matches a running process.
fn test_planned_update_with_running_process(id: &str) -> PlannedUpdate {
    // Use the current executable (test runner) as the blocking process.
    // This ensures the test finds itself as a running process, which works reliably
    // in both cargo test and nextest, regardless of the parent process.
    let process_file = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
        .unwrap_or_else(|| "cargo".to_string());

    let mut planned = test_planned_update(id);
    planned.software.name = process_file.clone();
    planned.software.detection = Some(DetectionConfig {
        method: DetectionMethod::PeFile,
        registry_key: None,
        registry_value: None,
        file_path: Some(process_file),
        version_regex: None,
        product_code: None,
        upgrade_code: None,
        inf_provider: None,
        device_class: None,
        inf_name: None,
        fallback: None,
    });
    planned
}

/// Create a PlannedUpdate with backup config enabled.
fn test_planned_update_with_backup(id: &str) -> PlannedUpdate {
    let mut planned = test_planned_update(id);
    planned.has_backup_config = true;
    planned.software.backup = Some(astro_up_core::types::BackupConfig {
        config_paths: vec!["/fake/config/path".to_string()],
    });
    planned
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Happy path: single-package update succeeds, events emitted in order.
#[tokio::test]
async fn single_package_happy_path_events_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("orchestration.lock");
    let orch = build_success_orchestrator(&lock_path).unwrap();

    let (on_event, events) = event_collector();
    let cancel = CancellationToken::new();

    let plan = UpdatePlan {
        items: vec![test_planned_update("nina-app")],
        skipped: vec![],
        warnings: vec![],
        quiet: true,
        install_scope: astro_up_core::config::InstallScope::default(),
        portable_apps_dir: None,
    };

    let result = orch.execute(plan, on_event, None, cancel).await.unwrap();

    assert_eq!(result.succeeded.len(), 1);
    assert!(result.failed.is_empty());
    assert_eq!(result.succeeded[0].status, OperationStatus::Success,);
    assert_eq!(
        result.succeeded[0].package_id,
        PackageId::new("nina-app").unwrap(),
    );

    // Verify event order: PlanReady -> PackageStarted -> PackageComplete -> OrchestrationComplete
    let captured = events.lock().unwrap();
    let event_types: Vec<&str> = captured
        .iter()
        .map(|e| match e {
            Event::PlanReady { .. } => "PlanReady",
            Event::PackageStarted { .. } => "PackageStarted",
            Event::PackageComplete { .. } => "PackageComplete",
            Event::OrchestrationComplete { .. } => "OrchestrationComplete",
            _ => "Other",
        })
        .collect();

    assert_eq!(
        event_types,
        vec![
            "PlanReady",
            "PackageStarted",
            "PackageComplete",
            "OrchestrationComplete"
        ],
        "events should be emitted in the expected order"
    );

    // Verify PlanReady content
    assert!(
        matches!(
            &captured[0],
            Event::PlanReady {
                total: 1,
                skipped: 0
            }
        ),
        "PlanReady should report 1 total, 0 skipped"
    );

    // Verify PackageStarted content
    assert!(
        matches!(&captured[1], Event::PackageStarted { package_id, .. }
            if *package_id == PackageId::new("nina-app").unwrap()),
        "PackageStarted should reference nina-app"
    );

    // Verify PackageComplete content
    assert!(
        matches!(&captured[2], Event::PackageComplete { package_id, status, .. }
            if *package_id == PackageId::new("nina-app").unwrap() && status == "succeeded"),
        "PackageComplete should report succeeded"
    );

    // Verify OrchestrationComplete content
    assert!(
        matches!(
            &captured[3],
            Event::OrchestrationComplete {
                succeeded: 1,
                failed: 0,
                skipped: 0
            }
        ),
        "OrchestrationComplete should report 1 succeeded"
    );
}

/// Process blocking: when a blocking process is detected, the pipeline emits
/// ProcessBlocking and reports failure.
#[tokio::test]
async fn process_blocking_emits_event_and_fails() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("orchestration.lock");
    let orch = build_success_orchestrator(&lock_path).unwrap();

    let (on_event, events) = event_collector();
    let cancel = CancellationToken::new();

    let plan = UpdatePlan {
        items: vec![test_planned_update_with_running_process("blocking-pkg")],
        skipped: vec![],
        warnings: vec![],
        quiet: true,
        install_scope: astro_up_core::config::InstallScope::default(),
        portable_apps_dir: None,
    };

    let result = orch.execute(plan, on_event, None, cancel).await.unwrap();

    assert!(
        result.succeeded.is_empty(),
        "no packages should succeed when process is blocking"
    );
    assert_eq!(result.failed.len(), 1);
    assert_eq!(result.failed[0].status, OperationStatus::Failed);
    assert!(
        result.failed[0]
            .error
            .as_ref()
            .unwrap()
            .contains("is running"),
        "error should mention the running process"
    );

    // Verify ProcessBlocking event was emitted
    let captured = events.lock().unwrap();
    let has_process_blocking = captured.iter().any(|e| {
        matches!(
            e,
            Event::ProcessBlocking {
                package_id,
                pid,
                ..
            } if *package_id == PackageId::new("blocking-pkg").unwrap() && *pid > 0
        )
    });
    assert!(
        has_process_blocking,
        "should emit ProcessBlocking event with valid PID"
    );
}

/// Cancellation mid-pipeline: cancel token is triggered after PackageStarted,
/// resulting in a Cancelled status.
#[tokio::test]
async fn cancellation_mid_pipeline() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("orchestration.lock");
    let orch = build_success_orchestrator(&lock_path).unwrap();

    let cancel = CancellationToken::new();
    let cancel_trigger = cancel.clone();

    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();

    // Cancel immediately after seeing PackageStarted
    let on_event: EventCallback = Box::new(move |e| {
        if matches!(&e, Event::PackageStarted { .. }) {
            cancel_trigger.cancel();
        }
        events_clone.lock().unwrap().push(e);
    });

    let plan = UpdatePlan {
        items: vec![
            test_planned_update("pkg-first"),
            test_planned_update("pkg-second"),
        ],
        skipped: vec![],
        warnings: vec![],
        quiet: true,
        install_scope: astro_up_core::config::InstallScope::default(),
        portable_apps_dir: None,
    };

    let result = orch.execute(plan, on_event, None, cancel).await.unwrap();

    // The first package should be cancelled (token fires after PackageStarted
    // but before the process check completes its next check_cancel!())
    let all_results: Vec<_> = result
        .succeeded
        .iter()
        .chain(result.failed.iter())
        .collect();

    // At least one package should have been cancelled
    let has_cancelled = all_results
        .iter()
        .any(|r| r.status == OperationStatus::Cancelled);
    assert!(has_cancelled, "at least one package should be cancelled");

    // The second package should NOT have succeeded (either cancelled or not started)
    let second_succeeded = result
        .succeeded
        .iter()
        .any(|r| r.package_id == PackageId::new("pkg-second").unwrap());
    assert!(
        !second_succeeded,
        "second package should not succeed after cancellation"
    );

    // Verify OrchestrationComplete was still emitted
    let captured = events.lock().unwrap();
    assert!(
        captured
            .iter()
            .any(|e| matches!(e, Event::OrchestrationComplete { .. })),
        "OrchestrationComplete should always be emitted"
    );
}

/// Failure after backup: when the installer fails after a backup was created,
/// the PackageResult should include the backup_path.
#[tokio::test]
async fn failure_after_backup_includes_backup_path() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("orchestration.lock");
    let orch = build_failing_installer_with_backup_orchestrator(&lock_path).unwrap();

    let (on_event, events) = event_collector();
    let cancel = CancellationToken::new();

    let plan = UpdatePlan {
        items: vec![test_planned_update_with_backup("backup-pkg")],
        skipped: vec![],
        warnings: vec![],
        quiet: true,
        install_scope: astro_up_core::config::InstallScope::default(),
        portable_apps_dir: None,
    };

    let result = orch.execute(plan, on_event, None, cancel).await.unwrap();

    assert!(result.succeeded.is_empty(), "install should have failed");
    assert_eq!(result.failed.len(), 1);

    let failed = &result.failed[0];
    assert_eq!(failed.status, OperationStatus::Failed);
    assert!(
        failed.error.as_ref().unwrap().contains("install failed"),
        "error should mention install failure"
    );
    assert_eq!(
        failed.backup_path,
        Some(PathBuf::from("/tmp/backups/config-backup.zip")),
        "backup_path should be set to the path returned by the backup manager"
    );

    // Verify PackageComplete event reports failure
    let captured = events.lock().unwrap();
    let has_failed_complete = captured.iter().any(|e| {
        matches!(
            e,
            Event::PackageComplete { status, .. } if status == "failed"
        )
    });
    assert!(
        has_failed_complete,
        "should emit PackageComplete with failed status"
    );
}
