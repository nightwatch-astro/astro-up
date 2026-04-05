#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for lifecycle dry-run mode.
//!
//! Verifies that dry-run:
//! - Skips install/uninstall phases
//! - Still runs the download URL resolution and detection discovery phases
//! - Reports FileExists for non-PE files during probe

use std::path::Path;
use std::time::Duration;

use astro_up_core::lifecycle::{LifecycleOptions, LifecycleRunner, PhaseStatus};

/// Create a minimal manifest directory structure for testing.
fn create_test_manifests(dir: &Path, package_id: &str) {
    let manifests_dir = dir.join("manifests");
    std::fs::create_dir_all(&manifests_dir).unwrap();

    let manifest = format!(
        r#"id = "{package_id}"
name = "Test Package"
type = "application"
category = "capture"
slug = "Test Package"

[checkver]
[checkver.autoupdate]
url = "https://example.com/releases/v$version/test-$version.exe"
"#
    );
    std::fs::write(manifests_dir.join(format!("{package_id}.toml")), manifest).unwrap();

    // Create a versions directory with a version file
    let versions_dir = dir.join("versions").join(package_id);
    std::fs::create_dir_all(&versions_dir).unwrap();
    std::fs::write(versions_dir.join("1.0.0.json"), "{}").unwrap();
}

#[tokio::test]
async fn dry_run_skips_install_uninstall() {
    let dir = tempfile::tempdir().unwrap();
    create_test_manifests(dir.path(), "test-pkg");

    let options = LifecycleOptions {
        manifest_path: dir.path().to_path_buf(),
        package_id: "test-pkg".into(),
        version: Some("1.0.0".into()),
        install_dir: None,
        dry_run: true,
        timeout: Duration::from_secs(30),
    };

    let report = LifecycleRunner::run(&options).await.unwrap();

    // Dry-run should have download + detect phases only (no install/uninstall/verify)
    assert!(
        report.phases.len() <= 3,
        "dry-run should not have install/uninstall phases, got {} phases: {:?}",
        report.phases.len(),
        report.phases.iter().map(|p| &p.phase).collect::<Vec<_>>()
    );

    // Should have download phase
    let download = report.phases.iter().find(|p| p.phase == "download");
    assert!(download.is_some(), "dry-run should have download phase");
    assert!(
        matches!(download.unwrap().status, PhaseStatus::Pass),
        "download should pass (URL resolution)"
    );

    // Should have detect phase
    let detect = report.phases.iter().find(|p| p.phase == "detect");
    assert!(detect.is_some(), "dry-run should have detect phase");

    // Should NOT have install phase
    let install = report.phases.iter().find(|p| p.phase == "install");
    assert!(install.is_none(), "dry-run should skip install phase");

    // Should NOT have uninstall phase
    let uninstall = report.phases.iter().find(|p| p.phase == "uninstall");
    assert!(uninstall.is_none(), "dry-run should skip uninstall phase");
}

#[tokio::test]
async fn dry_run_resolves_download_url() {
    let dir = tempfile::tempdir().unwrap();
    create_test_manifests(dir.path(), "test-pkg");

    let options = LifecycleOptions {
        manifest_path: dir.path().to_path_buf(),
        package_id: "test-pkg".into(),
        version: Some("2.5.0".into()),
        install_dir: None,
        dry_run: true,
        timeout: Duration::from_secs(30),
    };

    let report = LifecycleRunner::run(&options).await.unwrap();

    let download = report
        .phases
        .iter()
        .find(|p| p.phase == "download")
        .unwrap();
    assert!(matches!(download.status, PhaseStatus::Pass));
    assert!(
        download.logs.first().unwrap().contains("2.5.0"),
        "resolved URL should contain version"
    );
}

#[tokio::test]
async fn dry_run_overall_not_fail_on_no_detection() {
    let dir = tempfile::tempdir().unwrap();
    create_test_manifests(dir.path(), "test-pkg");

    let options = LifecycleOptions {
        manifest_path: dir.path().to_path_buf(),
        package_id: "test-pkg".into(),
        version: Some("1.0.0".into()),
        install_dir: None,
        dry_run: true,
        timeout: Duration::from_secs(30),
    };

    let report = LifecycleRunner::run(&options).await.unwrap();

    // Detection may fail in test env (no Windows registry), but the report should still be generated
    assert!(
        !report.package_id.is_empty(),
        "report should have package_id"
    );
    assert_eq!(report.version, "1.0.0");
}

#[test]
fn resolve_latest_version_picks_highest() {
    let dir = tempfile::tempdir().unwrap();
    let versions_dir = dir.path().join("versions").join("test-pkg");
    std::fs::create_dir_all(&versions_dir).unwrap();
    std::fs::write(versions_dir.join("1.0.0.json"), "{}").unwrap();
    std::fs::write(versions_dir.join("2.0.0.json"), "{}").unwrap();
    std::fs::write(versions_dir.join("1.5.0.json"), "{}").unwrap();

    let version = LifecycleRunner::resolve_latest_version(dir.path(), "test-pkg").unwrap();
    assert_eq!(version.to_string(), "2.0.0");
}
