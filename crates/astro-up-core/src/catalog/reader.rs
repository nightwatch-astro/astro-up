//! SQLite catalog reader — resolve, search, filter, list, versions.

use std::path::{Path, PathBuf};

use chrono::DateTime;
use rusqlite::{Connection, OpenFlags, params};

use crate::error::CoreError;

use super::types::{
    CatalogFilter, CatalogMeta, PackageId, PackageSummary, SearchResult, VersionEntry,
};

/// The schema version this client supports.
const SUPPORTED_SCHEMA: &str = "1";

/// Read-only SQLite catalog reader.
pub struct SqliteCatalogReader {
    conn: Connection,
    #[allow(dead_code)]
    path: PathBuf,
}

impl SqliteCatalogReader {
    /// Open a catalog database in read-only mode and verify schema version.
    #[tracing::instrument(skip_all, fields(catalog = %path.display()))]
    pub fn open(path: &Path) -> Result<Self, CoreError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        // Check integrity
        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|_| CoreError::CatalogCorrupted)?;
        if integrity != "ok" {
            return Err(CoreError::CatalogCorrupted);
        }

        // Check schema version
        let meta = Self::read_meta(&conn)?;
        if meta.schema_version != SUPPORTED_SCHEMA {
            return Err(CoreError::CatalogSchemaUnsupported {
                version: meta.schema_version,
                expected: SUPPORTED_SCHEMA.into(),
            });
        }

        Ok(Self {
            conn,
            path: path.to_owned(),
        })
    }

    /// Read catalog metadata from the `meta` table.
    fn read_meta(conn: &Connection) -> Result<CatalogMeta, CoreError> {
        let schema_version: String = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .map_err(|_| CoreError::CatalogCorrupted)?;

        let compiled_at_str: String = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'compiled_at'",
                [],
                |row| row.get(0),
            )
            .map_err(|_| CoreError::CatalogCorrupted)?;

        let compiled_at = DateTime::parse_from_rfc3339(&compiled_at_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|_| CoreError::CatalogCorrupted)?;

        Ok(CatalogMeta {
            schema_version,
            compiled_at,
        })
    }

    /// Resolve a single package by exact ID.
    pub fn resolve(&self, id: &PackageId) -> Result<PackageSummary, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, manifest_version, name, description, publisher, homepage,
                    category, type, slug, license, tags, aliases, dependencies
             FROM packages WHERE id = ?1",
        )?;

        stmt.query_row(params![id.as_ref()], row_to_package)
            .map_err(|_| CoreError::NotFound {
                input: id.to_string(),
            })
    }

    /// Full-text search across name, description, tags, aliases, publisher.
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.manifest_version, p.name, p.description, p.publisher,
                    p.homepage, p.category, p.type, p.slug, p.license,
                    p.tags, p.aliases, p.dependencies, f.rank
             FROM packages_fts f
             JOIN packages p ON p.rowid = f.rowid
             WHERE packages_fts MATCH ?1
             ORDER BY f.rank",
        )?;

        let results = stmt
            .query_map(params![query], |row| {
                let rank: f64 = row.get(13)?;
                Ok(SearchResult {
                    package: row_to_package_at(row, 0)?,
                    rank,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// List packages matching filter criteria.
    pub fn filter(&self, filter: &CatalogFilter) -> Result<Vec<PackageSummary>, CoreError> {
        let mut sql = String::from(
            "SELECT id, manifest_version, name, description, publisher, homepage,
                    category, type, slug, license, tags, aliases, dependencies
             FROM packages WHERE 1=1",
        );
        let mut param_values: Vec<String> = Vec::new();

        if let Some(ref cat) = filter.category {
            param_values.push(cat.to_string());
            sql.push_str(&format!(" AND category = ?{}", param_values.len()));
        }
        if let Some(ref st) = filter.software_type {
            param_values.push(st.to_string());
            sql.push_str(&format!(" AND type = ?{}", param_values.len()));
        }
        sql.push_str(" ORDER BY name");

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();

        let results = stmt
            .query_map(params.as_slice(), row_to_package)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// List all packages (unfiltered).
    pub fn list_all(&self) -> Result<Vec<PackageSummary>, CoreError> {
        self.filter(&CatalogFilter::default())
    }

    /// Get all known versions for a package, newest first.
    pub fn versions(&self, id: &PackageId) -> Result<Vec<VersionEntry>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT package_id, version, url, sha256, discovered_at,
                    release_notes_url, pre_release
             FROM versions
             WHERE package_id = ?1
             ORDER BY discovered_at DESC",
        )?;

        let results = stmt
            .query_map(params![id.as_ref()], row_to_version)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Get the latest non-pre-release version for a package.
    pub fn latest_version(&self, id: &PackageId) -> Result<Option<VersionEntry>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT package_id, version, url, sha256, discovered_at,
                    release_notes_url, pre_release
             FROM versions
             WHERE package_id = ?1 AND pre_release = 0
             ORDER BY discovered_at DESC
             LIMIT 1",
        )?;

        let result = stmt
            .query_row(params![id.as_ref()], row_to_version)
            .optional()?;

        Ok(result)
    }

    /// Get catalog metadata.
    pub fn meta(&self) -> Result<CatalogMeta, CoreError> {
        Self::read_meta(&self.conn)
    }
}

// ---------------------------------------------------------------------------
// Row mapping helpers
// ---------------------------------------------------------------------------

fn row_to_package(row: &rusqlite::Row<'_>) -> rusqlite::Result<PackageSummary> {
    row_to_package_at(row, 0)
}

fn row_to_package_at(row: &rusqlite::Row<'_>, offset: usize) -> rusqlite::Result<PackageSummary> {
    let id_str: String = row.get(offset)?;
    let category_str: String = row.get(offset + 6)?;
    let type_str: String = row.get(offset + 7)?;
    let tags_json: Option<String> = row.get(offset + 10)?;
    let aliases_json: Option<String> = row.get(offset + 11)?;
    let deps_json: Option<String> = row.get(offset + 12)?;

    Ok(PackageSummary {
        id: id_str
            .parse()
            .unwrap_or_else(|_| PackageId::new("unknown").unwrap()),
        manifest_version: row.get::<_, u32>(offset + 1)?,
        name: row.get(offset + 2)?,
        description: row.get(offset + 3)?,
        publisher: row.get(offset + 4)?,
        homepage: row.get(offset + 5)?,
        category: category_str
            .parse()
            .unwrap_or(crate::types::Category::Capture),
        software_type: type_str
            .parse()
            .unwrap_or(crate::types::SoftwareType::Application),
        slug: row.get(offset + 8)?,
        license: row.get(offset + 9)?,
        tags: parse_json_vec(&tags_json),
        aliases: parse_json_vec(&aliases_json),
        dependencies: parse_json_vec(&deps_json),
    })
}

fn row_to_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<VersionEntry> {
    let pid_str: String = row.get(0)?;
    let discovered_str: String = row.get(4)?;
    let pre: i32 = row.get(6)?;

    Ok(VersionEntry {
        package_id: pid_str
            .parse()
            .unwrap_or_else(|_| PackageId::new("unknown").unwrap()),
        version: row.get(1)?,
        url: row.get(2)?,
        sha256: row.get(3)?,
        discovered_at: DateTime::parse_from_rfc3339(&discovered_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_default(),
        release_notes_url: row.get(5)?,
        pre_release: pre != 0,
    })
}

fn parse_json_vec(json: &Option<String>) -> Vec<String> {
    json.as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

/// Extension trait to get optional results from rusqlite.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
