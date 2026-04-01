//! Operation history — record read/write for the operations table.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// The kind of operation performed on a package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OperationType {
    Install,
    Update,
    Uninstall,
}

/// The outcome of a completed operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OperationStatus {
    Success,
    Failed,
    Cancelled,
    RebootPending,
}

/// A single recorded operation in the operations table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    /// Auto-increment primary key.
    pub id: i64,
    /// The package this operation targeted.
    pub package_id: String,
    /// What kind of operation was performed.
    pub operation_type: OperationType,
    /// Version before the operation (relevant for updates/uninstalls).
    pub from_version: Option<String>,
    /// Version after the operation (relevant for installs/updates).
    pub to_version: Option<String>,
    /// How the operation concluded.
    pub status: OperationStatus,
    /// Wall-clock duration of the operation in milliseconds.
    pub duration_ms: u64,
    /// Error details when `status` is `Failed`.
    pub error_message: Option<String>,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn operation_type_display_round_trip() {
        assert_eq!(OperationType::Install.to_string(), "install");
        assert_eq!(OperationType::Update.to_string(), "update");
        assert_eq!(OperationType::Uninstall.to_string(), "uninstall");

        assert_eq!(
            OperationType::from_str("install").unwrap(),
            OperationType::Install
        );
    }

    #[test]
    fn operation_status_display_round_trip() {
        assert_eq!(OperationStatus::Success.to_string(), "success");
        assert_eq!(OperationStatus::Failed.to_string(), "failed");
        assert_eq!(OperationStatus::Cancelled.to_string(), "cancelled");
        assert_eq!(OperationStatus::RebootPending.to_string(), "reboot_pending");

        assert_eq!(
            OperationStatus::from_str("reboot_pending").unwrap(),
            OperationStatus::RebootPending
        );
    }

    #[test]
    fn operation_record_serde_round_trip() {
        let record = OperationRecord {
            id: 1,
            package_id: "nina".into(),
            operation_type: OperationType::Install,
            from_version: None,
            to_version: Some("3.1.2".into()),
            status: OperationStatus::Success,
            duration_ms: 45_000,
            error_message: None,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: OperationRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.package_id, "nina");
        assert_eq!(deserialized.operation_type, OperationType::Install);
        assert_eq!(deserialized.status, OperationStatus::Success);
        assert_eq!(deserialized.duration_ms, 45_000);
    }
}
