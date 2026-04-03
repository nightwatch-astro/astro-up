use std::path::PathBuf;

use astro_up_core::catalog::SqliteCatalogReader;
use astro_up_core::detect::DetectionError;
use astro_up_core::detect::scanner::{LedgerStore, PackageSource};
use astro_up_core::ledger::{LedgerEntry, LedgerSource};
use astro_up_core::types::{Software, Version};

/// Adapter: catalog reader → PackageSource trait for the scanner.
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
        id: &astro_up_core::catalog::PackageId,
    ) -> Result<Option<astro_up_core::catalog::VersionEntry>, DetectionError> {
        let reader = SqliteCatalogReader::open(&self.catalog_path)
            .map_err(|e| DetectionError::CatalogError(e.to_string()))?;
        reader
            .latest_version(id)
            .map_err(|e| DetectionError::CatalogError(e.to_string()))
    }
}

/// Adapter: SQLite → LedgerStore trait for the scanner.
///
/// Stores Acknowledged detection results in the app database.
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
