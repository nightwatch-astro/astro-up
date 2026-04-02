//! T036: Integration tests for the show command using assert_cmd.
//!
//! These tests run the actual binary and verify output structure.
//! They do NOT require a catalog (they test help output and argument parsing).

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("astro-up-cli").unwrap()
}

#[test]
fn show_help_lists_subcommands() {
    cmd()
        .args(["show", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("all"))
        .stdout(predicate::str::contains("installed"))
        .stdout(predicate::str::contains("outdated"))
        .stdout(predicate::str::contains("backups"));
}

#[test]
fn top_level_help_lists_all_commands() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("scan"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("backup"))
        .stdout(predicate::str::contains("restore"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("self-update"));
}

#[test]
fn version_flag_prints_version() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("astro-up"));
}

#[test]
fn install_requires_package_arg() {
    cmd()
        .arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn search_requires_query_arg() {
    cmd()
        .arg("search")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn config_requires_subcommand() {
    cmd()
        .arg("config")
        .assert()
        .failure()
        .stderr(predicate::str::contains("subcommand"));
}

#[test]
fn global_json_flag_accepted() {
    cmd().args(["--json", "scan"]).assert().success();
}

/// Normalize binary name across platforms (astro-up-cli.exe → astro-up-cli).
fn normalize_binary_name(s: &str) -> String {
    s.replace("astro-up-cli.exe", "astro-up-cli")
}

#[test]
fn snapshot_help_output() {
    let output = cmd().arg("--help").output().unwrap();
    let stdout = normalize_binary_name(&String::from_utf8_lossy(&output.stdout));
    insta::assert_snapshot!("cli_help", stdout);
}

#[test]
fn snapshot_show_help_output() {
    let output = cmd().args(["show", "--help"]).output().unwrap();
    let stdout = normalize_binary_name(&String::from_utf8_lossy(&output.stdout));
    insta::assert_snapshot!("show_help", stdout);
}
