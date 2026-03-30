use std::path::PathBuf;
use std::time::Duration;

use garde::Validate;

use astro_up_core::config::{AppConfig, PathsConfig};

#[test]
fn zero_timeout_fails_validation() {
    let mut config = AppConfig::with_paths(
        PathsConfig {
            download_dir: PathBuf::from("/tmp/dl"),
            cache_dir: PathBuf::from("/tmp/cache"),
            data_dir: PathBuf::from("/tmp/data"),
            ..PathsConfig::default()
        },
        PathBuf::from("/tmp/app.log"),
    );
    config.network.timeout = Duration::ZERO;

    let result = config.validate();
    assert!(result.is_err());
    let report = result.unwrap_err().to_string();
    assert!(
        report.contains("duration must be positive"),
        "got: {report}"
    );
}

#[test]
fn empty_user_agent_fails_validation() {
    let mut config = AppConfig::with_paths(
        PathsConfig {
            download_dir: PathBuf::from("/tmp/dl"),
            cache_dir: PathBuf::from("/tmp/cache"),
            data_dir: PathBuf::from("/tmp/data"),
            ..PathsConfig::default()
        },
        PathBuf::from("/tmp/app.log"),
    );
    config.network.user_agent = String::new();

    let result = config.validate();
    assert!(result.is_err());
    let report = result.unwrap_err().to_string();
    assert!(report.contains("length"), "got: {report}");
}

#[test]
fn check_interval_under_one_minute_fails() {
    let mut config = AppConfig::with_paths(
        PathsConfig {
            download_dir: PathBuf::from("/tmp/dl"),
            cache_dir: PathBuf::from("/tmp/cache"),
            data_dir: PathBuf::from("/tmp/data"),
            ..PathsConfig::default()
        },
        PathBuf::from("/tmp/app.log"),
    );
    config.updates.check_interval = Duration::from_secs(30);

    let result = config.validate();
    assert!(result.is_err());
    let report = result.unwrap_err().to_string();
    assert!(report.contains("at least 1 minute"), "got: {report}");
}

#[test]
fn invalid_proxy_url_fails_validation() {
    let mut config = AppConfig::with_paths(
        PathsConfig {
            download_dir: PathBuf::from("/tmp/dl"),
            cache_dir: PathBuf::from("/tmp/cache"),
            data_dir: PathBuf::from("/tmp/data"),
            ..PathsConfig::default()
        },
        PathBuf::from("/tmp/app.log"),
    );
    config.network.proxy = Some("not-a-url".to_string());

    let result = config.validate();
    assert!(result.is_err());
    let report = result.unwrap_err().to_string();
    assert!(report.contains("url"), "got: {report}");
}

#[test]
fn valid_defaults_pass_validation() {
    let config = AppConfig::with_paths(
        PathsConfig {
            download_dir: PathBuf::from("/tmp/dl"),
            cache_dir: PathBuf::from("/tmp/cache"),
            data_dir: PathBuf::from("/tmp/data"),
            ..PathsConfig::default()
        },
        PathBuf::from("/tmp/app.log"),
    );

    assert!(config.validate().is_ok());
}
