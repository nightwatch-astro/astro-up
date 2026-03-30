//! Integration tests for the catalog module.

use std::path::Path;

/// Test public key matching the keypair used to sign the fixture catalog.
const TEST_PUBLIC_KEY: &str = "RWQK6Ny4IewwF5A+6bI/YNv08w/kZ7hy3xVAv+SWPT11w+7RvatDV0bg";

fn fixture_dir() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/catalog"
    ))
}

fn fixture_catalog() -> std::path::PathBuf {
    fixture_dir().join("catalog.db")
}

fn fixture_sig() -> std::path::PathBuf {
    fixture_dir().join("catalog.db.minisig")
}

// =========================================================================
// T014: Signature verification tests
// =========================================================================

mod verify_tests {
    use super::*;
    use astro_up_core::catalog::verify::verify_catalog_with_key;

    #[test]
    fn valid_signature_passes() {
        let result = verify_catalog_with_key(&fixture_catalog(), &fixture_sig(), TEST_PUBLIC_KEY);
        assert!(result.is_ok());
    }

    #[test]
    fn tampered_data_fails() {
        let dir = tempfile::tempdir().unwrap();
        let tampered = dir.path().join("catalog.db");
        let sig = dir.path().join("catalog.db.minisig");

        // Copy fixture catalog, modify one byte
        let mut data = std::fs::read(fixture_catalog()).unwrap();
        data[100] ^= 0xFF;
        std::fs::write(&tampered, &data).unwrap();
        std::fs::copy(fixture_sig(), &sig).unwrap();

        let result = verify_catalog_with_key(&tampered, &sig, TEST_PUBLIC_KEY);
        assert!(result.is_err());
    }

    #[test]
    fn missing_signature_file_fails() {
        let dir = tempfile::tempdir().unwrap();
        let missing_sig = dir.path().join("nonexistent.minisig");

        let result = verify_catalog_with_key(&fixture_catalog(), &missing_sig, TEST_PUBLIC_KEY);
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("signature file missing"),
            "expected 'signature file missing', got: {err}"
        );
    }
}

// =========================================================================
// Reader — search, filter, resolve, versions tests (T023, T030)
// =========================================================================

mod reader_tests {
    use super::*;
    use astro_up_core::catalog::reader::SqliteCatalogReader;
    use astro_up_core::catalog::types::{CatalogFilter, PackageId};
    use astro_up_core::types::Category;

    fn open_fixture() -> SqliteCatalogReader {
        SqliteCatalogReader::open(&fixture_catalog()).unwrap()
    }

    #[test]
    fn open_and_check_schema() {
        let reader = open_fixture();
        let meta = reader.meta().unwrap();
        assert_eq!(meta.schema_version, "1");
    }

    #[test]
    fn resolve_known_id() {
        let reader = open_fixture();
        let id: PackageId = "nina".parse().unwrap();
        let pkg = reader.resolve(&id).unwrap();
        assert_eq!(pkg.name, "N.I.N.A.");
        assert_eq!(pkg.slug, "N.I.N.A.");
    }

    #[test]
    fn resolve_unknown_id() {
        let reader = open_fixture();
        let id: PackageId = "nonexistent".parse().unwrap();
        let result = reader.resolve(&id);
        assert!(result.is_err());
    }

    #[test]
    fn search_by_name() {
        let reader = open_fixture();
        let results = reader.search("NINA").unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].package.id.as_ref(), "nina");
    }

    #[test]
    fn search_by_description_keyword() {
        let reader = open_fixture();
        let results = reader.search("autoguiding").unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].package.id.as_ref(), "phd2");
    }

    #[test]
    fn search_empty_result() {
        let reader = open_fixture();
        let results = reader.search("zzzznonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn filter_by_category() {
        let reader = open_fixture();
        let results = reader
            .filter(&CatalogFilter {
                category: Some(Category::Guiding),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id.as_ref(), "phd2");
    }

    #[test]
    fn filter_capture_returns_multiple() {
        let reader = open_fixture();
        let results = reader
            .filter(&CatalogFilter {
                category: Some(Category::Capture),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn list_all_returns_all_packages() {
        let reader = open_fixture();
        let results = reader.list_all().unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn versions_returns_newest_first() {
        let reader = open_fixture();
        let id: PackageId = "nina".parse().unwrap();
        let versions = reader.versions(&id).unwrap();
        assert_eq!(versions.len(), 3);
        assert!(versions[0].discovered_at > versions[1].discovered_at);
    }

    #[test]
    fn latest_version_excludes_pre_releases() {
        let reader = open_fixture();
        let id: PackageId = "nina".parse().unwrap();
        let latest = reader.latest_version(&id).unwrap().unwrap();
        assert!(!latest.pre_release);
        assert_eq!(latest.version, "3.1.2.9001");
    }

    #[test]
    fn latest_version_returns_none_for_unknown() {
        let reader = open_fixture();
        let id: PackageId = "nonexistent".parse().unwrap();
        let latest = reader.latest_version(&id).unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn aliases_decoded_from_json() {
        let reader = open_fixture();
        let id: PackageId = "nina".parse().unwrap();
        let pkg = reader.resolve(&id).unwrap();
        assert!(pkg.aliases.contains(&"nina".to_string()));
        assert!(pkg.aliases.contains(&"nighttimeimaging".to_string()));
    }

    #[test]
    fn dependencies_decoded_from_json() {
        let reader = open_fixture();
        let id: PackageId = "sharpcap".parse().unwrap();
        let pkg = reader.resolve(&id).unwrap();
        assert_eq!(pkg.dependencies, vec!["ascom-platform"]);
    }
}
