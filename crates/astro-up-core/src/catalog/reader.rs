//! SQLite catalog reader — resolve, search, filter, list, versions.

use std::fmt::Write as _;
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

        let package_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM packages", [], |row| row.get(0))
            .unwrap_or(0);
        tracing::info!(
            schema_version = %meta.schema_version,
            package_count,
            "catalog opened"
        );

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
        tracing::debug!(package_id = %id, "resolving package from catalog");
        let mut stmt = self.conn.prepare(
            "SELECT id, manifest_version, name, description, publisher, homepage,
                    category, type, slug, license, tags, aliases, dependencies, icon_base64
             FROM packages WHERE id = ?1",
        )?;

        let result = stmt
            .query_row(params![id.as_ref()], row_to_package)
            .map_err(|_| CoreError::NotFound {
                input: id.to_string(),
            })?;
        tracing::trace!(package_id = %id, name = %result.name, "package resolved");
        Ok(result)
    }

    /// Full-text search across name, description, tags, aliases, publisher.
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, CoreError> {
        tracing::debug!(query, "searching catalog");
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.manifest_version, p.name, p.description, p.publisher,
                    p.homepage, p.category, p.type, p.slug, p.license,
                    p.tags, p.aliases, p.dependencies, p.icon_base64, f.rank
             FROM packages_fts f
             JOIN packages p ON p.rowid = f.rowid
             WHERE packages_fts MATCH ?1
             ORDER BY f.rank",
        )?;

        let results = stmt
            .query_map(params![query], |row| {
                let rank: f64 = row.get(14)?;
                Ok(SearchResult {
                    package: row_to_package_at(row, 0)?,
                    rank,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        tracing::debug!(
            query,
            result_count = results.len(),
            "catalog search complete"
        );
        Ok(results)
    }

    /// List packages matching filter criteria.
    pub fn filter(&self, filter: &CatalogFilter) -> Result<Vec<PackageSummary>, CoreError> {
        let mut sql = String::from(
            "SELECT id, manifest_version, name, description, publisher, homepage,
                    category, type, slug, license, tags, aliases, dependencies, icon_base64
             FROM packages WHERE 1=1",
        );
        let mut param_values: Vec<String> = Vec::new();

        if let Some(ref cat) = filter.category {
            param_values.push(cat.to_string());
            let _ = write!(sql, " AND category = ?{}", param_values.len());
        }
        if let Some(ref st) = filter.software_type {
            param_values.push(st.to_string());
            let _ = write!(sql, " AND type = ?{}", param_values.len());
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
        let results = self.filter(&CatalogFilter::default())?;
        tracing::debug!(count = results.len(), "listed all catalog packages");
        Ok(results)
    }

    /// Get all known versions for a package, newest first.
    pub fn versions(&self, id: &PackageId) -> Result<Vec<VersionEntry>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT package_id, version, url, sha256, discovered_at,
                    release_notes_url, pre_release, assets
             FROM versions WHERE package_id = ?1 ORDER BY discovered_at DESC",
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
                    release_notes_url, pre_release, assets
             FROM versions WHERE package_id = ?1 AND pre_release = 0
             ORDER BY discovered_at DESC LIMIT 1",
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

    /// Get detection config for a package from the operational `detection` table.
    ///
    /// Returns `None` if the table doesn't exist (old catalog) or the package has no detection config.
    /// Handles both old-schema catalogs (with `path`, `fallback_method`, `fallback_path` columns)
    /// and new-schema catalogs (with `file_path`, `version_regex`, `product_code`, `upgrade_code`,
    /// `inf_provider`, `device_class`, `inf_name`, `fallback_config` columns) gracefully.
    pub fn detection_config(
        &self,
        id: &PackageId,
    ) -> Result<Option<crate::types::DetectionConfig>, CoreError> {
        let result = self
            .conn
            .query_row(
                "SELECT method, file_path, registry_key, registry_value,
                        version_regex, product_code, upgrade_code,
                        inf_provider, device_class, inf_name, fallback_config
                 FROM detection WHERE package_id = ?1",
                params![id.as_ref()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, Option<String>>(10)?,
                    ))
                },
            )
            .optional()
            .unwrap_or(None); // Table or columns may not exist in old catalogs

        let Some((
            method_str,
            file_path,
            registry_key,
            registry_value,
            version_regex,
            product_code,
            upgrade_code,
            inf_provider,
            device_class,
            inf_name,
            fallback_config_json,
        )) = result
        else {
            return Ok(None);
        };

        let method = parse_detection_method(&method_str);
        let Some(method) = method else {
            tracing::warn!(package = %id, method = %method_str, "unknown detection method");
            return Ok(None);
        };

        let fallback = fallback_config_json.and_then(|json| {
            serde_json::from_str::<crate::types::DetectionConfig>(&json)
                .map(Box::new)
                .map_err(|e| {
                    tracing::warn!(
                        package = %id,
                        error = %e,
                        "failed to deserialize fallback_config"
                    );
                    e
                })
                .ok()
        });

        Ok(Some(crate::types::DetectionConfig {
            method,
            registry_key,
            registry_value,
            file_path,
            version_regex,
            product_code,
            upgrade_code,
            inf_provider,
            device_class,
            inf_name,
            fallback,
        }))
    }

    /// Read install config from the catalog's `install` table.
    pub fn install_config(
        &self,
        id: &PackageId,
    ) -> Result<Option<crate::types::InstallConfig>, CoreError> {
        let result = self
            .conn
            .query_row(
                "SELECT method, scope, elevation, switches, exit_codes, success_codes
                 FROM install WHERE package_id = ?1",
                params![id.as_ref()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, i32>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .optional()
            .unwrap_or(None); // Table may not exist in old catalogs

        let Some((
            method_str,
            scope_str,
            elevation_int,
            switches_json,
            exit_codes_json,
            success_codes_json,
        )) = result
        else {
            return Ok(None);
        };

        let method = parse_install_method(&method_str);
        let scope = scope_str.and_then(|s| s.parse().ok());
        let elevation = if elevation_int != 0 {
            Some(crate::types::Elevation::Required)
        } else {
            None
        };

        let switches = switches_json
            .and_then(|json| {
                serde_json::from_str::<std::collections::HashMap<String, String>>(&json).ok()
            })
            .and_then(|map| {
                let silent: Vec<String> = map
                    .get("silent")
                    .map(|s| s.split_whitespace().map(String::from).collect())
                    .unwrap_or_default();
                let interactive: Vec<String> = map
                    .get("interactive")
                    .map(|s| s.split_whitespace().map(String::from).collect())
                    .unwrap_or_default();
                let log = map.get("log").cloned();
                let install_location = map.get("install_dir").cloned();
                if !silent.is_empty()
                    || !interactive.is_empty()
                    || log.is_some()
                    || install_location.is_some()
                {
                    Some(crate::types::InstallerSwitches {
                        silent,
                        interactive,
                        upgrade: Vec::new(),
                        install_location,
                        log,
                        custom: Vec::new(),
                    })
                } else {
                    None
                }
            });

        let success_codes: Vec<i32> = success_codes_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let known_exit_codes: std::collections::HashMap<String, crate::types::KnownExitCode> =
            exit_codes_json
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();

        Ok(Some(crate::types::InstallConfig {
            method,
            scope,
            elevation,
            upgrade_behavior: None,
            install_modes: Vec::new(),
            success_codes,
            pre_install: Vec::new(),
            post_install: Vec::new(),
            switches,
            known_exit_codes,
            timeout: None,
        }))
    }

    /// List all packages with their detection configs (for scanning).
    ///
    /// Returns `Software` objects with detection config populated from the `detection` table.
    pub fn list_all_with_detection(&self) -> Result<Vec<crate::types::Software>, CoreError> {
        let summaries = self.list_all()?;
        let mut software = Vec::with_capacity(summaries.len());

        for summary in summaries {
            let detection = self.detection_config(&summary.id)?;
            let install = self.install_config(&summary.id)?;
            software.push(crate::types::Software {
                id: summary.id,
                slug: summary.slug,
                name: summary.name,
                software_type: summary.software_type,
                category: summary.category,
                os: vec![],
                description: summary.description,
                homepage: summary.homepage,
                publisher: summary.publisher,
                icon_url: None,
                license: summary.license,
                license_url: None,
                aliases: summary.aliases,
                tags: summary.tags,
                notes: None,
                docs_url: None,
                channel: None,
                min_os_version: None,
                manifest_version: Some(summary.manifest_version),
                detection,
                install,
                checkver: None,
                dependencies: None,
                hardware: None,
                backup: None,
                versioning: None,
            });
        }

        Ok(software)
    }
}

// ---------------------------------------------------------------------------
// Detection method parsing
// ---------------------------------------------------------------------------

fn parse_detection_method(method_str: &str) -> Option<crate::types::DetectionMethod> {
    match method_str {
        "registry" => Some(crate::types::DetectionMethod::Registry),
        "file" | "pe_file" => Some(crate::types::DetectionMethod::PeFile),
        "wmi" => Some(crate::types::DetectionMethod::Wmi),
        "driver_store" => Some(crate::types::DetectionMethod::DriverStore),
        "ascom_profile" => Some(crate::types::DetectionMethod::AscomProfile),
        "file_exists" => Some(crate::types::DetectionMethod::FileExists),
        "config_file" => Some(crate::types::DetectionMethod::ConfigFile),
        "ledger" => Some(crate::types::DetectionMethod::Ledger),
        _ => None,
    }
}

fn parse_install_method(method_str: &str) -> crate::types::InstallMethod {
    match method_str {
        "exe" => crate::types::InstallMethod::Exe,
        "msi" => crate::types::InstallMethod::Msi,
        "inno_setup" => crate::types::InstallMethod::InnoSetup,
        "nullsoft" | "nsis" => crate::types::InstallMethod::Nullsoft,
        "wix" => crate::types::InstallMethod::Wix,
        "burn" => crate::types::InstallMethod::Burn,
        "zip" => crate::types::InstallMethod::Zip,
        "zip_wrap" => crate::types::InstallMethod::ZipWrap,
        "portable" => crate::types::InstallMethod::Portable,
        "download_only" => crate::types::InstallMethod::DownloadOnly,
        _ => crate::types::InstallMethod::Exe, // default fallback
    }
}

// ---------------------------------------------------------------------------
// Row mapping helpers
// ---------------------------------------------------------------------------

fn row_to_package(row: &rusqlite::Row<'_>) -> rusqlite::Result<PackageSummary> {
    row_to_package_at(row, 0)
}

#[allow(clippy::unwrap_in_result)] // "unknown" is a known-valid PackageId constant
fn row_to_package_at(row: &rusqlite::Row<'_>, offset: usize) -> rusqlite::Result<PackageSummary> {
    let id_str: String = row.get(offset)?;
    let category_str: String = row.get(offset + 6)?;
    let type_str: String = row.get(offset + 7)?;
    let tags_json: Option<String> = row.get(offset + 10)?;
    let aliases_json: Option<String> = row.get(offset + 11)?;
    let deps_json: Option<String> = row.get(offset + 12)?;

    #[allow(clippy::unwrap_used)] // "unknown" is a valid PackageId constant
    let fallback_id = PackageId::new("unknown").unwrap();
    Ok(PackageSummary {
        id: id_str.parse().unwrap_or(fallback_id),
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
        icon_base64: row.get(offset + 13)?,
    })
}

#[allow(clippy::unwrap_in_result)] // "unknown" is a known-valid PackageId constant
fn row_to_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<VersionEntry> {
    let pid_str: String = row.get(0)?;
    let discovered_str: String = row.get(4)?;
    let pre: i32 = row.get(6)?;
    let assets_json: Option<String> = row.get(7).unwrap_or(None);

    let assets: Vec<crate::catalog::types::ReleaseAsset> = assets_json
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();

    #[allow(clippy::unwrap_used)] // "unknown" is a valid PackageId constant
    let fallback_id = PackageId::new("unknown").unwrap();
    Ok(VersionEntry {
        package_id: pid_str.parse().unwrap_or(fallback_id),
        version: row.get(1)?,
        url: row.get(2)?,
        sha256: row.get(3)?,
        discovered_at: DateTime::parse_from_rfc3339(&discovered_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_default(),
        release_notes_url: row.get(5)?,
        pre_release: pre != 0,
        assets,
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
