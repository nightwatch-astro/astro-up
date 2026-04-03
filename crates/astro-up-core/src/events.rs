use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::catalog::PackageId;

/// Typed notification from the engine to UI layers.
///
/// Adjacently tagged for clean TypeScript consumption:
/// `{"type": "download_progress", "data": {"id": "nina-app", ...}}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    CheckStarted {
        id: String,
    },
    CheckProgress {
        id: String,
        progress: f64,
    },
    CheckComplete {
        id: String,
    },
    DownloadStarted {
        id: String,
        url: String,
    },
    DownloadProgress {
        id: String,
        progress: f64,
        bytes_downloaded: u64,
        total_bytes: u64,
        speed: f64,
        elapsed: Duration,
        estimated_remaining: Option<Duration>,
    },
    DownloadComplete {
        id: String,
    },
    BackupStarted {
        id: String,
    },
    BackupProgress {
        id: String,
        files_processed: u32,
        total_files: u32,
    },
    BackupComplete {
        id: String,
    },
    RestoreStarted {
        id: String,
    },
    RestoreComplete {
        id: String,
    },
    InstallStarted {
        id: String,
    },
    InstallComplete {
        id: String,
    },
    InstallFailed {
        id: String,
        error: String,
    },
    InstallRebootRequired {
        id: String,
    },
    ManualDownloadRequired {
        id: String,
        url: String,
    },
    Error {
        id: String,
        error: String,
    },
    ScanStarted,
    ScanProgress {
        progress: f64,
        current_id: String,
    },
    ScanComplete {
        total_found: u32,
    },

    // -- Orchestration events --
    /// Plan has been computed and is ready for execution.
    PlanReady {
        total: usize,
        skipped: usize,
    },
    /// Starting the install pipeline for a single package.
    PackageStarted {
        package_id: PackageId,
        step_count: usize,
    },
    /// Single-package pipeline completed.
    PackageComplete {
        package_id: PackageId,
        /// Stringified operation status (placeholder until OperationStatus type exists).
        status: String,
        /// Error details when status is "failed".
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// Package skipped due to policy or dependency failure.
    PackageSkipped {
        package_id: PackageId,
        /// Stringified skip reason (placeholder until SkipReason type exists).
        reason: String,
    },
    /// Waiting for a blocking process to close before proceeding.
    ProcessBlocking {
        package_id: PackageId,
        process_name: String,
        pid: u32,
    },
    /// All packages have been processed.
    OrchestrationComplete {
        succeeded: usize,
        failed: usize,
        skipped: usize,
    },
}

// Compile-time assertion: Event must be Send + 'static (required for broadcast channels)
fn _assert_event_send() {
    fn _require_send<T: Send + 'static>() {}
    _require_send::<Event>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serialization_format() {
        let event = Event::DownloadProgress {
            id: "nina-app".into(),
            progress: 0.5,
            bytes_downloaded: 1024,
            total_bytes: 2048,
            speed: 512.0,
            elapsed: Duration::from_secs(2),
            estimated_remaining: Some(Duration::from_secs(2)),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "download_progress");
        assert!(json["data"]["id"].is_string());
        assert_eq!(json["data"]["progress"], 0.5);
    }

    #[test]
    fn all_events_snapshot() {
        let events = vec![
            Event::CheckStarted { id: "test".into() },
            Event::CheckProgress {
                id: "test".into(),
                progress: 0.5,
            },
            Event::CheckComplete { id: "test".into() },
            Event::DownloadStarted {
                id: "test".into(),
                url: "https://example.com".into(),
            },
            Event::DownloadProgress {
                id: "test".into(),
                progress: 0.75,
                bytes_downloaded: 768,
                total_bytes: 1024,
                speed: 256.0,
                elapsed: Duration::from_secs(3),
                estimated_remaining: Some(Duration::from_secs(1)),
            },
            Event::DownloadComplete { id: "test".into() },
            Event::BackupStarted { id: "test".into() },
            Event::BackupProgress {
                id: "test".into(),
                files_processed: 5,
                total_files: 10,
            },
            Event::BackupComplete { id: "test".into() },
            Event::RestoreStarted { id: "test".into() },
            Event::RestoreComplete { id: "test".into() },
            Event::InstallStarted { id: "test".into() },
            Event::InstallComplete { id: "test".into() },
            Event::InstallFailed {
                id: "test".into(),
                error: "installer exited with code 1".into(),
            },
            Event::InstallRebootRequired { id: "test".into() },
            Event::ManualDownloadRequired {
                id: "test".into(),
                url: "https://example.com".into(),
            },
            Event::Error {
                id: "test".into(),
                error: "something failed".into(),
            },
            Event::ScanStarted,
            Event::ScanProgress {
                progress: 0.5,
                current_id: "test".into(),
            },
            Event::ScanComplete { total_found: 42 },
            Event::PlanReady {
                total: 5,
                skipped: 1,
            },
            Event::PackageStarted {
                package_id: PackageId::new("nina-app").unwrap(),
                step_count: 3,
            },
            Event::PackageComplete {
                package_id: PackageId::new("nina-app").unwrap(),
                status: "succeeded".into(),
                error: None,
            },
            Event::PackageSkipped {
                package_id: PackageId::new("phd2").unwrap(),
                reason: "dependency_failed".into(),
            },
            Event::ProcessBlocking {
                package_id: PackageId::new("nina-app").unwrap(),
                process_name: "NINA.exe".into(),
                pid: 1234,
            },
            Event::OrchestrationComplete {
                succeeded: 3,
                failed: 1,
                skipped: 1,
            },
        ];

        insta::assert_json_snapshot!(events);
    }
}
