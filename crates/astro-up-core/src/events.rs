use serde::{Deserialize, Serialize};

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
    },
    DownloadComplete {
        id: String,
    },
    BackupStarted {
        id: String,
    },
    BackupComplete {
        id: String,
    },
    InstallStarted {
        id: String,
    },
    InstallComplete {
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
}

// Compile-time assertion: Event must be Send + 'static (required for flume channels)
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
            },
            Event::DownloadComplete { id: "test".into() },
            Event::BackupStarted { id: "test".into() },
            Event::BackupComplete { id: "test".into() },
            Event::InstallStarted { id: "test".into() },
            Event::InstallComplete { id: "test".into() },
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
        ];

        insta::assert_json_snapshot!(events);
    }
}
