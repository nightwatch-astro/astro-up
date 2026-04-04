use astro_up_core::detect::pe;

#[test]
fn pe_fixture_extracts_version() {
    let result = pe::read_pe_version_sync("tests/fixtures/test.exe");
    match result {
        astro_up_core::detect::DetectionResult::Installed {
            version, method, ..
        } => {
            assert_eq!(version.raw, "3.2.1");
            assert_eq!(method, astro_up_core::types::DetectionMethod::PeFile);
        }
        other => panic!("expected Installed, got {other:?}"),
    }
}

#[test]
fn pe_missing_file_returns_not_installed() {
    let result = pe::read_pe_version_sync("tests/fixtures/nonexistent.exe");
    assert!(matches!(
        result,
        astro_up_core::detect::DetectionResult::NotInstalled
    ));
}

#[test]
fn pe_invalid_file_returns_unavailable() {
    // Use Cargo.toml as a non-PE file
    let result = pe::read_pe_version_sync("Cargo.toml");
    assert!(matches!(
        result,
        astro_up_core::detect::DetectionResult::Unavailable { .. }
    ));
}
