#![allow(clippy::unwrap_used, clippy::expect_used)]
//! JSON output validation tests.
//!
//! These tests invoke the binary which requires Windows.

#![cfg(target_os = "windows")]

use assert_cmd::Command;

fn cmd() -> Command {
    Command::cargo_bin("astro-up").unwrap()
}

#[test]
fn scan_json_is_valid() {
    let output = cmd().args(["--json", "scan"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON from scan --json: {e}\nOutput: {stdout}"));

    assert!(parsed.get("results").is_some(), "missing 'results' key");
    assert!(parsed.get("errors").is_some(), "missing 'errors' key");
}

#[test]
fn config_show_json_is_valid() {
    let output = cmd().args(["--json", "config", "show"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON from config show --json: {e}\nOutput: {stdout}"));

    assert!(
        parsed.get("catalog").is_some(),
        "missing 'catalog' key in config JSON"
    );
}

#[test]
fn self_update_json_is_valid() {
    let output = cmd().args(["--json", "self-update"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON from self-update --json: {e}\nOutput: {stdout}"));

    assert!(
        parsed.get("current_version").is_some(),
        "missing 'current_version' key"
    );
}

#[test]
fn self_update_dry_run_json_has_flag() {
    let output = cmd()
        .args(["--json", "self-update", "--dry-run"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(
        parsed.get("dry_run"),
        Some(&serde_json::Value::Bool(true)),
        "dry_run should be true"
    );
}
