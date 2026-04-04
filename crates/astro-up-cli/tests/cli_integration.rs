//! Comprehensive CLI integration tests.
//!
//! Cross-platform tests run everywhere. Windows-specific tests are gated
//! with `#[cfg(target_os = "windows")]` and exercise detection, install,
//! and update paths that require Windows APIs.

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("astro-up-cli").unwrap()
}

// =========================================================================
// Help & version (cross-platform)
// =========================================================================

#[test]
fn help_shows_docs_url() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("nightwatch-astro.github.io"));
}

#[test]
fn short_help_shows_about() {
    cmd()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Astrophotography software manager",
        ));
}

#[test]
fn version_includes_semver() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"astro-up \d+\.\d+\.\d+").unwrap());
}

// =========================================================================
// Subcommand help (cross-platform)
// =========================================================================

#[test]
fn install_help() {
    cmd()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run"))
        .stdout(predicate::str::contains("yes"));
}

#[test]
fn update_help() {
    cmd()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--allow-major"));
}

#[test]
fn backup_help() {
    cmd()
        .args(["backup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PACKAGE"));
}

#[test]
fn restore_help() {
    cmd()
        .args(["restore", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PACKAGE"))
        .stdout(predicate::str::contains("--yes"));
}

#[test]
fn self_update_help() {
    cmd()
        .args(["self-update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"));
}

// =========================================================================
// Argument validation (cross-platform)
// =========================================================================

#[test]
fn install_missing_package_fails() {
    cmd()
        .arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn search_missing_query_fails() {
    cmd()
        .arg("search")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn backup_missing_package_fails() {
    cmd()
        .arg("backup")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn restore_missing_package_fails() {
    cmd()
        .arg("restore")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn config_missing_subcommand_fails() {
    cmd()
        .arg("config")
        .assert()
        .failure()
        .stderr(predicate::str::contains("subcommand"));
}

#[test]
fn unknown_command_fails() {
    cmd().arg("frobnicate").assert().failure();
}

// =========================================================================
// JSON output mode (cross-platform)
// =========================================================================

#[test]
fn json_scan_returns_valid_json() {
    let output = cmd().args(["--json", "scan"]).output().unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(parsed.get("results").is_some());
    assert!(parsed.get("errors").is_some());
}

#[test]
fn json_config_show_returns_valid_json() {
    let output = cmd().args(["--json", "config", "show"]).output().unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    // Config should have known sections
    assert!(parsed.get("ui").is_some() || parsed.get("catalog").is_some());
}

#[test]
fn json_self_update_returns_valid_json() {
    let output = cmd().args(["--json", "self-update"]).output().unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(parsed.get("current_version").is_some());
}

#[test]
fn json_self_update_dry_run_returns_valid_json() {
    let output = cmd()
        .args(["--json", "self-update", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["dry_run"], true);
}

#[test]
fn json_update_returns_valid_json() {
    let output = cmd().args(["--json", "update"]).output().unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(parsed.get("updates").is_some());
}

#[test]
fn json_show_installed_returns_valid_json() {
    let output = cmd()
        .args(["--json", "show", "installed"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(parsed.get("packages").is_some());
}

#[test]
fn json_show_outdated_returns_valid_json() {
    let output = cmd().args(["--json", "show", "outdated"]).output().unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(parsed.get("packages").is_some());
}

#[test]
fn json_show_backups_returns_valid_json() {
    let output = cmd().args(["--json", "show", "backups"]).output().unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(parsed.get("backups").is_some());
}

// =========================================================================
// Quiet mode (cross-platform)
// =========================================================================

#[test]
fn quiet_scan_suppresses_output() {
    cmd()
        .args(["--quiet", "scan"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn quiet_self_update_suppresses_output() {
    cmd()
        .args(["--quiet", "self-update"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn quiet_config_show_suppresses_output() {
    cmd()
        .args(["--quiet", "config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn quiet_show_installed_suppresses_output() {
    cmd()
        .args(["--quiet", "show", "installed"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn quiet_show_outdated_suppresses_output() {
    cmd()
        .args(["--quiet", "show", "outdated"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn quiet_update_suppresses_output() {
    cmd()
        .args(["--quiet", "update"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =========================================================================
// Config commands (cross-platform)
// =========================================================================

#[test]
fn config_show_prints_toml() {
    cmd()
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[ui]"))
        .stdout(predicate::str::contains("[catalog]"))
        .stdout(predicate::str::contains("[network]"));
}

#[test]
fn self_update_prints_version() {
    cmd()
        .arg("self-update")
        .assert()
        .success()
        .stdout(predicate::str::contains("astro-up"));
}

#[test]
fn self_update_dry_run_mentions_dry_run() {
    cmd()
        .args(["self-update", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));
}

// =========================================================================
// Show subcommands do NOT require catalog download (bug fix)
// =========================================================================

#[test]
fn show_installed_does_not_fail_without_catalog() {
    // This should succeed immediately without trying to download the catalog
    cmd().args(["show", "installed"]).assert().success();
}

#[test]
fn show_outdated_does_not_fail_without_catalog() {
    cmd().args(["show", "outdated"]).assert().success();
}

#[test]
fn show_backups_does_not_fail_without_catalog() {
    cmd().args(["show", "backups"]).assert().success();
}

// =========================================================================
// Platform guard messages (cross-platform — non-Windows shows message)
// =========================================================================

#[cfg(not(target_os = "windows"))]
mod non_windows {
    use super::*;

    #[test]
    fn scan_shows_platform_message() {
        cmd()
            .arg("scan")
            .assert()
            .success()
            .stdout(predicate::str::contains("requires Windows"));
    }

    #[test]
    fn update_shows_platform_message() {
        cmd()
            .arg("update")
            .assert()
            .success()
            .stdout(predicate::str::contains("requires Windows"));
    }

    #[test]
    fn json_scan_includes_platform_note() {
        let output = cmd().args(["--json", "scan"]).output().unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(parsed["note"], "detection requires Windows");
    }

    #[test]
    fn json_update_includes_platform_note() {
        let output = cmd().args(["--json", "update"]).output().unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(parsed["note"], "update requires Windows");
    }
}

// =========================================================================
// Windows-specific tests (run in CI on windows-latest runner)
// =========================================================================

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    #[test]
    fn scan_runs_on_windows() {
        // On Windows, scan should succeed (even with empty results)
        cmd().arg("scan").assert().success();
    }

    #[test]
    fn json_scan_on_windows_has_no_platform_note() {
        let output = cmd().args(["--json", "scan"]).output().unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(parsed.get("note").is_none() || parsed["note"].is_null());
    }

    #[test]
    fn update_runs_on_windows() {
        cmd().arg("update").assert().success();
    }

    #[test]
    fn restore_no_backups_found() {
        cmd()
            .args(["restore", "nonexistent-package", "--yes"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No backups found"));
    }

    #[test]
    fn show_backups_for_package_no_results() {
        cmd()
            .args(["show", "backups", "nonexistent-package"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No backups found"));
    }
}
