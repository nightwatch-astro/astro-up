//! Catalog sidecar — JSON metadata file (ETag, fetched_at).

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// JSON sidecar stored as `catalog.db.meta` alongside the catalog file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogSidecar {
    pub etag: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

impl CatalogSidecar {
    /// Load sidecar from disk. Returns `None` if the file doesn't exist.
    pub fn load(path: &Path) -> Result<Option<Self>, std::io::Error> {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                let sidecar: Self =
                    serde_json::from_str(&contents).map_err(std::io::Error::other)?;
                Ok(Some(sidecar))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Save sidecar to disk.
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Build the sidecar path from a catalog path (e.g., `catalog.db` → `catalog.db.meta`).
    pub fn path_for(catalog_path: &Path) -> std::path::PathBuf {
        let mut p = catalog_path.as_os_str().to_owned();
        p.push(".meta");
        std::path::PathBuf::from(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("catalog.db.meta");

        let sidecar = CatalogSidecar {
            etag: Some("\"abc123\"".into()),
            fetched_at: Utc.with_ymd_and_hms(2026, 3, 30, 12, 0, 0).unwrap(),
        };

        sidecar.save(&path).unwrap();
        let loaded = CatalogSidecar::load(&path).unwrap().unwrap();
        assert_eq!(loaded, sidecar);
    }

    #[test]
    fn missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.meta");
        let result = CatalogSidecar::load(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn path_for_catalog() {
        let p = CatalogSidecar::path_for(Path::new("/data/catalog.db"));
        assert_eq!(p, Path::new("/data/catalog.db.meta"));
    }
}
