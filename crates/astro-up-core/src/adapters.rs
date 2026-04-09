//! Concrete adapters for Scanner traits.
//!
//! These bridge the catalog/ledger storage to the trait interfaces used by
//! the detection scanner. Shared by both the CLI and GUI crates.

use std::path::PathBuf;

use crate::catalog::SqliteCatalogReader;
use crate::detect::scanner::{LedgerStore, PackageSource};
use crate::detect::{DetectionError, DetectionResult, PackageDetection};
use crate::ledger::{LedgerEntry, LedgerSource};
use crate::types::{Software, Version};

/// Adapter: catalog reader -> PackageSource trait for the scanner.
pub struct CatalogPackageSource {
    catalog_path: PathBuf,
}

impl CatalogPackageSource {
    pub fn new(catalog_path: PathBuf) -> Self {
        Self { catalog_path }
    }
}

impl PackageSource for CatalogPackageSource {
    fn list_all(&self) -> Result<Vec<Software>, DetectionError> {
        let reader = SqliteCatalogReader::open(&self.catalog_path)
            .map_err(|e| DetectionError::CatalogError(e.to_string()))?;
        reader
            .list_all_with_detection()
            .map_err(|e| DetectionError::CatalogError(e.to_string()))
    }

    fn latest_version(
        &self,
        id: &crate::catalog::PackageId,
    ) -> Result<Option<crate::catalog::VersionEntry>, DetectionError> {
        let reader = SqliteCatalogReader::open(&self.catalog_path)
            .map_err(|e| DetectionError::CatalogError(e.to_string()))?;
        reader
            .latest_version(id)
            .map_err(|e| DetectionError::CatalogError(e.to_string()))
    }
}

/// Adapter: SQLite -> LedgerStore trait for the scanner.
pub struct SqliteLedgerStore {
    db_path: PathBuf,
}

impl SqliteLedgerStore {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    fn open_conn(&self) -> Result<rusqlite::Connection, DetectionError> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS ledger (
                package_id TEXT NOT NULL,
                version TEXT NOT NULL,
                source TEXT NOT NULL,
                recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
                notes TEXT,
                install_path TEXT,
                PRIMARY KEY (package_id, source)
            )",
        )
        .map_err(|e| DetectionError::LedgerError(e.to_string()))?;
        Ok(conn)
    }
}

impl LedgerStore for SqliteLedgerStore {
    fn list_acknowledged(&self) -> Result<Vec<LedgerEntry>, DetectionError> {
        let conn = self.open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT package_id, version, source, recorded_at, notes, install_path
                 FROM ledger WHERE source = 'acknowledged'",
            )
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;

        let entries = stmt
            .query_map([], |row| {
                Ok(LedgerEntry {
                    package_id: row.get(0)?,
                    version: Version::parse(
                        &row.get::<_, String>(1)
                            .unwrap_or_else(|_| "0.0.0".to_string()),
                    ),
                    source: LedgerSource::Acknowledged,
                    recorded_at: chrono::Utc::now(),
                    notes: row.get(4)?,
                    install_path: row
                        .get::<_, Option<String>>(5)?
                        .map(std::path::PathBuf::from),
                })
            })
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;

        Ok(entries)
    }

    fn upsert_acknowledged(
        &self,
        package_id: &str,
        version: &Version,
    ) -> Result<(), DetectionError> {
        let conn = self.open_conn()?;
        conn.execute(
            "INSERT INTO ledger (package_id, version, source)
             VALUES (?1, ?2, 'acknowledged')
             ON CONFLICT(package_id, source) DO UPDATE SET version = ?2, recorded_at = datetime('now')",
            rusqlite::params![package_id, version.to_string()],
        )
        .map_err(|e| DetectionError::LedgerError(e.to_string()))?;
        Ok(())
    }

    fn remove_acknowledged(&self, package_id: &str) -> Result<(), DetectionError> {
        let conn = self.open_conn()?;
        conn.execute(
            "DELETE FROM ledger WHERE package_id = ?1 AND source = 'acknowledged'",
            rusqlite::params![package_id],
        )
        .map_err(|e| DetectionError::LedgerError(e.to_string()))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Detection result persistence
// ---------------------------------------------------------------------------

/// Persists detection scan results to the app database so they survive restarts.
pub struct DetectionStore {
    db_path: PathBuf,
}

impl DetectionStore {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    fn open_conn(&self) -> Result<rusqlite::Connection, DetectionError> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS detection_cache (
                package_id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                version TEXT,
                method TEXT,
                install_path TEXT,
                reason TEXT,
                scanned_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS scan_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )
        .map_err(|e| DetectionError::LedgerError(e.to_string()))?;
        Ok(conn)
    }

    /// Persist scan results, replacing any previous cache.
    pub fn save_results(&self, results: &[PackageDetection]) -> Result<(), DetectionError> {
        let conn = self.open_conn()?;

        conn.execute("DELETE FROM detection_cache", [])
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "INSERT INTO detection_cache (package_id, status, version, method, install_path, reason)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;

        for pd in results {
            let (status, version, method, install_path, reason) = match &pd.result {
                DetectionResult::Installed {
                    version,
                    method,
                    install_path,
                } => (
                    "installed",
                    Some(version.to_string()),
                    Some(method.to_string()),
                    install_path.clone(),
                    None,
                ),
                DetectionResult::InstalledUnknownVersion {
                    method,
                    install_path,
                } => (
                    "installed_unknown_version",
                    None,
                    Some(method.to_string()),
                    install_path.clone(),
                    None,
                ),
                DetectionResult::NotInstalled => ("not_installed", None, None, None, None),
                DetectionResult::Unavailable { reason } => {
                    ("unavailable", None, None, None, Some(reason.clone()))
                }
            };

            stmt.execute(rusqlite::params![
                pd.package_id,
                status,
                version,
                method,
                install_path,
                reason,
            ])
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;
        }

        // Update last scan timestamp
        conn.execute(
            "INSERT INTO scan_metadata (key, value) VALUES ('last_scan_at', datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = datetime('now')",
            [],
        )
        .map_err(|e| DetectionError::LedgerError(e.to_string()))?;

        Ok(())
    }

    /// Load cached detection results.
    pub fn load_results(&self) -> Result<Vec<PackageDetection>, DetectionError> {
        let conn = self.open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT package_id, status, version, method, install_path, reason
                 FROM detection_cache",
            )
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;

        let results = stmt
            .query_map([], |row| {
                let package_id: String = row.get(0)?;
                let status: String = row.get(1)?;
                let version: Option<String> = row.get(2)?;
                let method: Option<String> = row.get(3)?;
                let install_path: Option<String> = row.get(4)?;
                let reason: Option<String> = row.get(5)?;

                let result = match status.as_str() {
                    "installed" => DetectionResult::Installed {
                        version: Version::parse(&version.unwrap_or_else(|| "0.0.0".into())),
                        method: method
                            .as_deref()
                            .unwrap_or("registry")
                            .parse()
                            .unwrap_or(crate::types::DetectionMethod::Registry),
                        install_path,
                    },
                    "installed_unknown_version" => DetectionResult::InstalledUnknownVersion {
                        method: method
                            .as_deref()
                            .unwrap_or("registry")
                            .parse()
                            .unwrap_or(crate::types::DetectionMethod::Registry),
                        install_path,
                    },
                    "unavailable" => DetectionResult::Unavailable {
                        reason: reason.unwrap_or_default(),
                    },
                    _ => DetectionResult::NotInstalled,
                };

                Ok(PackageDetection { package_id, result })
            })
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DetectionError::LedgerError(e.to_string()))?;

        Ok(results)
    }

    /// Get the timestamp of the last scan, or `None` if never scanned.
    pub fn last_scan_at(&self) -> Result<Option<String>, DetectionError> {
        let conn = self.open_conn()?;
        let result: Option<String> = conn
            .query_row(
                "SELECT value FROM scan_metadata WHERE key = 'last_scan_at'",
                [],
                |row| row.get(0),
            )
            .ok();
        Ok(result)
    }
}
