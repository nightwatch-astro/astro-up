//! Operation history — record read/write for the operations table.

use std::fmt::Write as _;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use super::orchestrator::HistoryFilter;

/// The kind of operation performed on a package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OperationType {
    Install,
    Update,
    Uninstall,
}

/// The outcome of a completed operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
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

/// Create the operations table and indexes. Idempotent (IF NOT EXISTS).
pub fn create_table(conn: &Connection) -> Result<(), crate::error::CoreError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS operations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_id TEXT NOT NULL,
            operation_type TEXT NOT NULL,
            from_version TEXT,
            to_version TEXT,
            status TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            error_message TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_operations_package_id ON operations(package_id);
        CREATE INDEX IF NOT EXISTS idx_operations_created_at ON operations(created_at);",
    )
    .map_err(|e| crate::error::CoreError::Database(e.to_string()))
}

/// Insert an operation record. Returns the auto-generated row ID.
pub fn record_operation(
    conn: &Connection,
    record: &OperationRecord,
) -> Result<i64, crate::error::CoreError> {
    conn.execute(
        "INSERT INTO operations (package_id, operation_type, from_version, to_version, status, duration_ms, error_message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record.package_id,
            record.operation_type.to_string(),
            record.from_version,
            record.to_version,
            record.status.to_string(),
            record.duration_ms as i64,
            record.error_message,
            record.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| crate::error::CoreError::Database(e.to_string()))?;
    Ok(conn.last_insert_rowid())
}

/// Query operation history with optional filters.
pub fn query_history(
    conn: &Connection,
    filter: &HistoryFilter,
) -> Result<Vec<OperationRecord>, crate::error::CoreError> {
    let mut sql = "SELECT id, package_id, operation_type, from_version, to_version, status, duration_ms, error_message, created_at FROM operations".to_string();
    let mut conditions: Vec<String> = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref pkg) = filter.package_id {
        conditions.push(format!("package_id = ?{}", param_values.len() + 1));
        param_values.push(Box::new(pkg.as_ref().to_string()));
    }
    if let Some(ref op_type) = filter.operation_type {
        conditions.push(format!("operation_type = ?{}", param_values.len() + 1));
        param_values.push(Box::new(op_type.to_string()));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY created_at DESC");

    if let Some(limit) = filter.limit {
        let _ = write!(sql, " LIMIT {limit}");
    }

    #[allow(clippy::redundant_closure_for_method_calls)]
    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| crate::error::CoreError::Database(e.to_string()))?;
    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            let op_type_str: String = row.get(2)?;
            let status_str: String = row.get(5)?;
            let duration_i64: i64 = row.get(6)?;
            let created_str: String = row.get(8)?;
            Ok(OperationRecord {
                id: row.get(0)?,
                package_id: row.get(1)?,
                operation_type: OperationType::from_str(&op_type_str)
                    .unwrap_or(OperationType::Update),
                from_version: row.get(3)?,
                to_version: row.get(4)?,
                status: OperationStatus::from_str(&status_str).unwrap_or(OperationStatus::Failed),
                duration_ms: duration_i64 as u64,
                error_message: row.get(7)?,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
            })
        })
        .map_err(|e| crate::error::CoreError::Database(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| crate::error::CoreError::Database(e.to_string()))?);
    }
    Ok(results)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::catalog::PackageId;
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

    #[test]
    fn create_table_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        create_table(&conn).unwrap();
        create_table(&conn).unwrap(); // second call should not error
    }

    #[test]
    fn record_and_query_operation() {
        let conn = Connection::open_in_memory().unwrap();
        create_table(&conn).unwrap();

        let record = OperationRecord {
            id: 0,
            package_id: "nina".into(),
            operation_type: OperationType::Update,
            from_version: Some("3.0.0".into()),
            to_version: Some("3.1.0".into()),
            status: OperationStatus::Success,
            duration_ms: 5000,
            error_message: None,
            created_at: Utc::now(),
        };

        let id = record_operation(&conn, &record).unwrap();
        assert!(id > 0);

        let filter = HistoryFilter {
            package_id: Some(PackageId::new("nina").unwrap()),
            limit: None,
            operation_type: None,
        };
        let results = query_history(&conn, &filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].package_id, "nina");
        assert_eq!(results[0].status, OperationStatus::Success);
    }

    #[test]
    fn query_history_with_limit() {
        let conn = Connection::open_in_memory().unwrap();
        create_table(&conn).unwrap();

        for i in 0..5 {
            let record = OperationRecord {
                id: 0,
                package_id: "phd2".into(),
                operation_type: OperationType::Update,
                from_version: Some(format!("1.{i}.0")),
                to_version: Some(format!("1.{}.0", i + 1)),
                status: OperationStatus::Success,
                duration_ms: 1000,
                error_message: None,
                created_at: Utc::now(),
            };
            record_operation(&conn, &record).unwrap();
        }

        let filter = HistoryFilter {
            package_id: None,
            limit: Some(3),
            operation_type: None,
        };
        let results = query_history(&conn, &filter).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn query_history_empty_returns_empty() {
        let conn = Connection::open_in_memory().unwrap();
        create_table(&conn).unwrap();

        let filter = HistoryFilter {
            package_id: None,
            limit: None,
            operation_type: None,
        };
        let results = query_history(&conn, &filter).unwrap();
        assert!(results.is_empty());
    }
}
