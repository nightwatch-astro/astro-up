#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Snapshot tests for lifecycle-test CLI command output structures.
//!
//! These tests verify JSON report structure and TOML detection config output
//! using insta snapshots with mock manifest data. They run on all platforms
//! since they test serialization, not Windows-specific detection.

use astro_up_core::lifecycle::{LifecycleReport, LifecycleStatus, PhaseResult, PhaseStatus};
use astro_up_core::types::{DetectionConfig, DetectionMethod};
use std::time::Duration;

fn sample_report() -> LifecycleReport {
    LifecycleReport {
        package_id: "nina-app".into(),
        version: "3.1.2".into(),
        phases: vec![
            PhaseResult {
                phase: "download".into(),
                status: PhaseStatus::Pass,
                duration: Duration::from_millis(1234),
                exit_code: None,
                logs: vec![
                    "https://github.com/nina/releases/download/v3.1.2/NINA-3.1.2.exe".into(),
                ],
                warnings: vec![],
            },
            PhaseResult {
                phase: "detect".into(),
                status: PhaseStatus::Pass,
                duration: Duration::from_millis(567),
                exit_code: None,
                logs: vec![
                    r#"{"method":"registry","registry_key":"NINA 2","registry_value":"DisplayVersion"}"#
                        .into(),
                ],
                warnings: vec![],
            },
        ],
        discovered_config: Some(DetectionConfig {
            method: DetectionMethod::Registry,
            registry_key: Some("NINA 2".into()),
            registry_value: Some("DisplayVersion".into()),
            file_path: None,
            version_regex: None,
            product_code: None,
            upgrade_code: None,
            inf_provider: None,
            device_class: None,
            inf_name: None,
            fallback: None,
        }),
        overall_status: LifecycleStatus::Pass,
    }
}

#[test]
fn snapshot_lifecycle_report_json() {
    let report = sample_report();
    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("lifecycle_report_json", json);
}

#[test]
fn snapshot_lifecycle_report_toml_config() {
    let config = DetectionConfig {
        method: DetectionMethod::Registry,
        registry_key: Some("NINA 2".into()),
        registry_value: Some("DisplayVersion".into()),
        file_path: None,
        version_regex: None,
        product_code: None,
        upgrade_code: None,
        inf_provider: None,
        device_class: None,
        inf_name: None,
        fallback: Some(Box::new(DetectionConfig {
            method: DetectionMethod::PeFile,
            file_path: Some(r"C:\Program Files\NINA\NINA.exe".into()),
            registry_key: None,
            registry_value: None,
            version_regex: None,
            product_code: None,
            upgrade_code: None,
            inf_provider: None,
            device_class: None,
            inf_name: None,
            fallback: None,
        })),
    };

    let toml = astro_up_core::lifecycle::LifecycleRunner::config_to_toml(&config);
    insta::assert_snapshot!("lifecycle_detection_toml", toml);
}

#[test]
fn snapshot_lifecycle_fail_report_json() {
    let report = LifecycleReport {
        package_id: "test-pkg".into(),
        version: "1.0.0".into(),
        phases: vec![
            PhaseResult {
                phase: "download".into(),
                status: PhaseStatus::Pass,
                duration: Duration::from_millis(100),
                exit_code: None,
                logs: vec!["https://example.com/test.exe".into()],
                warnings: vec![],
            },
            PhaseResult {
                phase: "detect".into(),
                status: PhaseStatus::Fail,
                duration: Duration::from_millis(50),
                exit_code: None,
                logs: vec![],
                warnings: vec!["no detection signatures found".into()],
            },
        ],
        discovered_config: None,
        overall_status: LifecycleStatus::Fail,
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("lifecycle_fail_report_json", json);
}
