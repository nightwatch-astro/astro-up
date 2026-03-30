//! Catalog module public API contract.
//!
//! This file defines the trait and types that the catalog module exposes
//! to the rest of astro-up-core and to CLI/GUI consumers.

use crate::types::software::{Category, SoftwareType};

/// Query-only view of a package from the catalog.
/// Does NOT include operational fields (detection, install, etc.).
pub struct PackageSummary {
    pub id: PackageId,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub homepage: Option<String>,
    pub category: Category,
    pub software_type: SoftwareType,
    pub license: Option<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub manifest_version: u32,
}

/// A discovered version for a package.
pub struct VersionEntry {
    pub package_id: PackageId,
    pub version: String,
    pub url: String,
    pub sha256: Option<String>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub release_notes_url: Option<String>,
    pub pre_release: bool,
}

/// Search results with relevance ranking.
pub struct SearchResult {
    pub package: PackageSummary,
    pub rank: f64,
}

/// Filter criteria for catalog queries.
pub struct CatalogFilter {
    pub category: Option<Category>,
    pub software_type: Option<SoftwareType>,
}

/// Result of a catalog fetch operation.
pub enum FetchResult {
    /// New catalog downloaded and verified.
    Updated,
    /// Server returned 304 — local catalog is current.
    Unchanged,
    /// Fetch failed but local catalog is available.
    FallbackToLocal { reason: String },
}

/// The catalog API exposed to CLI/GUI consumers.
#[trait_variant::make(Send)]
pub trait CatalogReader {
    /// Resolve a single package by exact ID.
    async fn resolve(&self, id: &PackageId) -> Result<PackageSummary, CatalogError>;

    /// Full-text search across name, description, tags, aliases, publisher.
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, CatalogError>;

    /// List packages matching filter criteria.
    async fn filter(&self, filter: &CatalogFilter) -> Result<Vec<PackageSummary>, CatalogError>;

    /// List all packages (unfiltered).
    async fn list_all(&self) -> Result<Vec<PackageSummary>, CatalogError>;

    /// Get all known versions for a package, newest first.
    async fn versions(&self, id: &PackageId) -> Result<Vec<VersionEntry>, CatalogError>;

    /// Get the latest non-pre-release version for a package.
    async fn latest_version(&self, id: &PackageId) -> Result<Option<VersionEntry>, CatalogError>;
}

/// The catalog lifecycle manager (fetch, verify, refresh).
#[trait_variant::make(Send)]
pub trait CatalogManager {
    /// Ensure a valid catalog is available. Fetches if needed (TTL expired or no local copy).
    async fn ensure_catalog(&self) -> Result<FetchResult, CatalogError>;

    /// Force a refresh regardless of TTL.
    async fn refresh(&self) -> Result<FetchResult, CatalogError>;
}
