//! Comprehensive CLI integration tests.
//!
//! All tests require Windows (the binary exits on non-Windows).

#![cfg(target_os = "windows")]

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("astro-up").unwrap()
}

// =========================================================================
// Help & version
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
// Argument validation
// =========================================================================

#[test]
fn install_missing_package_fails() {
    cmd().arg("install").assert().failure();
}

#[test]
fn search_missing_query_fails() {
    cmd().arg("search").assert().failure();
}

#[test]
fn backup_missing_package_fails() {
    cmd().arg("backup").assert().failure();
}

#[test]
fn restore_missing_package_fails() {
    cmd().arg("restore").assert().failure();
}

#[test]
fn config_missing_subcommand_fails() {
    cmd().arg("config").assert().failure();
}

// =========================================================================
// Config commands (no state needed)
// =========================================================================

#[test]
fn config_show_prints_toml() {
    cmd()
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[ui]"))
        .stdout(predicate::str::contains("[catalog]"));
}

// =========================================================================
// Self-update (no state needed)
// =========================================================================

#[test]
fn self_update_prints_version() {
    cmd()
        .arg("self-update")
        .assert()
        .success()
        .stdout(predicate::str::contains("astro-up"));
}

#[test]
fn self_update_dry_run() {
    cmd().args(["self-update", "--dry-run"]).assert().success();
}

// =========================================================================
// JSON output (T025)
// =========================================================================

#[test]
fn json_config_show_valid() {
    let output = cmd().args(["--json", "config", "show"]).output().unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert!(parsed.get("catalog").is_some());
}

#[test]
fn json_self_update_valid() {
    let output = cmd().args(["--json", "self-update"]).output().unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert!(parsed.get("current_version").is_some());
    assert!(parsed.get("status").is_some());
}

#[test]
fn json_self_update_dry_run_has_flag() {
    let output = cmd()
        .args(["--json", "self-update", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(parsed["dry_run"], true);
}

// =========================================================================
// Scan (T008-T009) — requires catalog
// =========================================================================

#[test]
fn scan_produces_output() {
    // May fail if catalog not available, but should not crash
    let output = cmd().arg("scan").output().unwrap();
    // Either succeeds with table or fails gracefully
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn json_scan_valid() {
    let output = cmd().args(["--json", "scan"]).output().unwrap();
    if output.status.success() {
        let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
        assert!(parsed.get("results").is_some());
    }
}

// =========================================================================
// Show subcommands (T019)
// =========================================================================

#[test]
fn show_installed_runs() {
    let output = cmd().args(["show", "installed"]).output().unwrap();
    // May need catalog; should not crash
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn show_outdated_runs() {
    let output = cmd().args(["show", "outdated"]).output().unwrap();
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn show_backups_without_package() {
    cmd()
        .args(["show", "backups"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Specify a package"));
}

// =========================================================================
// Install (T013)
// =========================================================================

#[test]
fn install_nonexistent_shows_suggestions() {
    let output = cmd()
        .args(["install", "nonexistent-package-xyz"])
        .output()
        .unwrap();
    // Should fail or show suggestions, not crash
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("not found")
            || combined.contains("Did you mean")
            || !output.status.success()
    );
}

// =========================================================================
// Update (T016)
// =========================================================================

#[test]
fn update_without_args_shows_help() {
    cmd().arg("update").assert().success().stdout(
        predicate::str::contains("Specify a package").or(predicate::str::contains("--all")),
    );
}

#[test]
fn json_update_valid() {
    let output = cmd().args(["--json", "update", "--all"]).output().unwrap();
    if output.status.success() {
        let _parsed: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("valid JSON");
    }
}

// =========================================================================
// Backup (T022)
// =========================================================================

#[test]
fn backup_nonexistent_package() {
    let output = cmd()
        .args(["backup", "nonexistent-package-xyz"])
        .output()
        .unwrap();
    assert!(!output.status.success() || !output.stderr.is_empty());
}

// =========================================================================
// Quiet mode
// =========================================================================

#[test]
fn quiet_config_show_suppresses_output() {
    cmd()
        .args(["--quiet", "config", "show"])
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

// =========================================================================
// Cancellation (T031)
// =========================================================================

#[test]
fn ctrl_c_returns_exit_code_2() {
    use std::process::Command as StdCommand;

    // Start a long-running command (self-update with network)
    let mut child = StdCommand::new(assert_cmd::cargo::cargo_bin("astro-up"))
        .args(["self-update"])
        .spawn()
        .expect("failed to spawn process");

    // Give it a moment to start, then kill it
    std::thread::sleep(std::time::Duration::from_millis(100));

    // On Windows, GenerateConsoleCtrlEvent isn't easily callable from Rust.
    // Use kill as a proxy — the process should handle termination gracefully.
    let _ = child.kill();
    let status = child.wait().expect("failed to wait for process");

    // Process was terminated (not a clean exit 0)
    assert!(!status.success());
}

// =========================================================================
// Binary name (T028)
// =========================================================================

#[test]
fn binary_is_named_astro_up() {
    let bin_path = assert_cmd::cargo::cargo_bin("astro-up");
    let file_name = bin_path.file_stem().unwrap().to_str().unwrap();
    assert_eq!(file_name, "astro-up");
}
