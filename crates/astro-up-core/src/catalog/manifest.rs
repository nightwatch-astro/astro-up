//! TOML manifest reader — reads raw manifests from the manifests repo checkout.

use std::path::Path;

use crate::error::CoreError;
use crate::types::Software;

/// Reads raw TOML manifest files from a manifests repo checkout.
pub struct ManifestReader;

impl ManifestReader {
    /// Read a single manifest TOML file and deserialize to [`Software`].
    pub fn read(path: &Path) -> Result<Software, CoreError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str::<Software>(&content).map_err(|e| CoreError::ManifestInvalid {
            id: path.display().to_string(),
            reason: e.to_string(),
        })
    }

    /// Read a manifest by package ID from the manifests directory.
    ///
    /// Looks for `{manifests_root}/manifests/{package_id}.toml`.
    pub fn read_by_id(manifests_root: &Path, package_id: &str) -> Result<Software, CoreError> {
        let path = manifests_root
            .join("manifests")
            .join(format!("{package_id}.toml"));
        if !path.exists() {
            return Err(CoreError::NotFound {
                input: format!("manifest for '{package_id}' at {}", path.display()),
            });
        }
        Self::read(&path)
    }

    /// List all package IDs that have `[install]` but no `[detection]` section.
    ///
    /// Performs a lightweight TOML parse — checks for section headers without
    /// full deserialization, for speed.
    pub fn list_missing_detection(manifests_root: &Path) -> Result<Vec<String>, CoreError> {
        let manifests_dir = manifests_root.join("manifests");
        if !manifests_dir.is_dir() {
            return Err(CoreError::NotFound {
                input: format!("manifests directory at {}", manifests_dir.display()),
            });
        }

        let mut missing = Vec::new();

        for entry in std::fs::read_dir(&manifests_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "toml") {
                let content = std::fs::read_to_string(&path)?;
                let has_install = content.contains("[install]");
                let has_detection = content.contains("[detection]");

                if has_install && !has_detection {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        missing.push(stem.to_string());
                    }
                }
            }
        }

        missing.sort();
        Ok(missing)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;

    fn sample_manifest() -> &'static str {
        r#"
id = "nina-app"
name = "N.I.N.A."
slug = "nina"
type = "application"
category = "capture"

[install]
method = "inno_setup"

[checkver]
provider = "github"
github = "nightwatch-astro/nina"

[checkver.autoupdate]
url = "https://github.com/nightwatch-astro/nina/releases/download/v$version/NINA-$version-setup.exe"
"#
    }

    #[test]
    fn read_manifest_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nina-app.toml");
        fs::write(&path, sample_manifest()).unwrap();

        let software = ManifestReader::read(&path).unwrap();
        assert_eq!(software.name, "N.I.N.A.");
        assert_eq!(software.id.as_ref(), "nina-app");
        assert!(software.install.is_some());
        assert!(software.detection.is_none());
    }

    #[test]
    fn read_by_id() {
        let dir = tempfile::tempdir().unwrap();
        let manifests_dir = dir.path().join("manifests");
        fs::create_dir(&manifests_dir).unwrap();
        fs::write(manifests_dir.join("nina-app.toml"), sample_manifest()).unwrap();

        let software = ManifestReader::read_by_id(dir.path(), "nina-app").unwrap();
        assert_eq!(software.name, "N.I.N.A.");
    }

    #[test]
    fn read_by_id_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let manifests_dir = dir.path().join("manifests");
        fs::create_dir(&manifests_dir).unwrap();

        let result = ManifestReader::read_by_id(dir.path(), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn list_missing_detection() {
        let dir = tempfile::tempdir().unwrap();
        let manifests_dir = dir.path().join("manifests");
        fs::create_dir(&manifests_dir).unwrap();

        // Has install, no detection → should be listed
        fs::write(manifests_dir.join("nina-app.toml"), sample_manifest()).unwrap();

        // Has install AND detection → should NOT be listed
        let with_detection = format!(
            "{}\n[detection]\nmethod = \"registry\"\nregistry_key = \"NINA 2\"\n",
            sample_manifest()
        );
        fs::write(manifests_dir.join("phd2-app.toml"), with_detection).unwrap();

        // No install → should NOT be listed
        fs::write(
            manifests_dir.join("firmware.toml"),
            "id = \"firmware\"\nname = \"FW\"\nslug = \"fw\"\ntype = \"resource\"\ncategory = \"equipment\"\n",
        )
        .unwrap();

        let missing = ManifestReader::list_missing_detection(dir.path()).unwrap();
        assert_eq!(missing, vec!["nina-app"]);
    }
}
