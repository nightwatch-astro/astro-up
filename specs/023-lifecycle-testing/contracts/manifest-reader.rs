// Contract: TOML Manifest Reader
// Location: crates/astro-up-core/src/catalog/manifest.rs (new module)

use crate::error::CoreError;
use crate::types::Software;
use std::path::Path;

/// Reads raw TOML manifests from the manifests repo checkout.
pub struct ManifestReader;

impl ManifestReader {
    /// Read a single manifest TOML file and deserialize to Software.
    pub fn read(path: &Path) -> Result<Software, CoreError>;

    /// Read a manifest by package ID from the manifests directory.
    /// Looks for `{manifests_root}/manifests/{package_id}.toml`.
    pub fn read_by_id(manifests_root: &Path, package_id: &str) -> Result<Software, CoreError>;

    /// List all package IDs that have [install] but no [detection] section.
    /// Used by matrix sweep to find packages needing discovery.
    pub fn list_missing_detection(manifests_root: &Path) -> Result<Vec<String>, CoreError>;
}
