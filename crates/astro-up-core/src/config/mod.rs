mod api;
mod defaults;
mod model;
mod store;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use garde::Validate;

use crate::error::CoreError;

pub use api::{config_get, config_list, config_reset, config_set};
pub use model::{
    AppConfig, CatalogConfig, LogConfig, LogLevel, NetworkConfig, PathsConfig, TelemetryConfig,
    UpdateConfig,
};
pub use store::ConfigStore;

/// Load configuration with 3-layer precedence: defaults → SQLite → CLI flags.
pub fn load_config(
    db_path: &Path,
    default_paths: PathsConfig,
    log_file: PathBuf,
    cli_overrides: &[(&str, &str)],
) -> Result<AppConfig, CoreError> {
    // Layer 1: compiled defaults with caller-provided platform paths
    let mut config = AppConfig::with_paths(default_paths, log_file);

    // Layer 2: SQLite stored overrides
    let store = open_store(db_path)?;
    let stored = store.list()?;
    merge_overrides(&mut config, &stored)?;

    // Layer 3: CLI flag overrides (highest precedence)
    let cli_pairs: Vec<(String, String)> = cli_overrides
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    merge_overrides(&mut config, &cli_pairs)?;

    // Validate
    config.validate().map_err(CoreError::from)?;

    Ok(config)
}

/// Open a ConfigStore, handling corruption by renaming the corrupt file and starting fresh.
fn open_store(db_path: &Path) -> Result<ConfigStore, CoreError> {
    match rusqlite::Connection::open(db_path) {
        Ok(conn) => match ConfigStore::new(conn) {
            Ok(store) => Ok(store),
            Err(e) => recover_corrupt(db_path, e),
        },
        Err(e) => recover_corrupt(db_path, e),
    }
}

fn recover_corrupt(
    db_path: &Path,
    original_err: rusqlite::Error,
) -> Result<ConfigStore, CoreError> {
    let corrupt_path = db_path.with_extension("corrupt");
    tracing::warn!(
        "Config database corrupt ({}), renaming to {} and starting fresh",
        original_err,
        corrupt_path.display()
    );
    let _ = std::fs::rename(db_path, &corrupt_path);
    let conn = rusqlite::Connection::open(db_path)?;
    Ok(ConfigStore::new(conn)?)
}

/// Merge key-value overrides into an AppConfig.
fn merge_overrides(
    config: &mut AppConfig,
    overrides: &[(String, String)],
) -> Result<(), CoreError> {
    for (key, value) in overrides {
        if !config.is_known_key(key) {
            return Err(CoreError::ConfigUnknownKey {
                key: key.clone(),
                valid_keys: config.known_keys(),
            });
        }
        set_field(config, key, value)?;
    }
    Ok(())
}

/// Set a single field on AppConfig by dot-path key.
pub(crate) fn set_field(config: &mut AppConfig, key: &str, value: &str) -> Result<(), CoreError> {
    let parse_err = |expected: &str| CoreError::ConfigParse {
        key: key.to_string(),
        expected: expected.to_string(),
        got: value.to_string(),
    };

    match key {
        "catalog.url" => config.catalog.url = value.to_string(),
        "catalog.cache_ttl" => {
            config.catalog.cache_ttl =
                parse_duration(value).map_err(|_| parse_err("duration (e.g. 24h, 30s)"))?;
        }
        "paths.download_dir" => config.paths.download_dir = PathBuf::from(value),
        "paths.cache_dir" => config.paths.cache_dir = PathBuf::from(value),
        "paths.data_dir" => config.paths.data_dir = PathBuf::from(value),
        "paths.keep_installers" => {
            config.paths.keep_installers = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "paths.purge_installers_after_days" => {
            config.paths.purge_installers_after_days = value
                .parse::<u32>()
                .map_err(|_| parse_err("integer days (0 = disabled)"))?;
        }
        "network.proxy" => {
            config.network.proxy = if value.is_empty() || value == "none" {
                None
            } else {
                Some(value.to_string())
            };
        }
        "network.timeout" => {
            config.network.timeout =
                parse_duration(value).map_err(|_| parse_err("duration (e.g. 30s, 1m)"))?;
        }
        "network.connect_timeout" => {
            config.network.connect_timeout =
                parse_duration(value).map_err(|_| parse_err("duration (e.g. 10s, 5s)"))?;
        }
        "network.user_agent" => config.network.user_agent = value.to_string(),
        "network.download_speed_limit" => {
            config.network.download_speed_limit = value
                .parse::<u64>()
                .map_err(|_| parse_err("integer bytes/sec (0 = unlimited)"))?;
        }
        "updates.auto_check" => {
            config.updates.auto_check = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "updates.check_interval" => {
            config.updates.check_interval =
                parse_duration(value).map_err(|_| parse_err("duration (e.g. 24h, 6h)"))?;
        }
        "logging.level" => {
            config.logging.level = LogLevel::from_str(value)
                .map_err(|_| parse_err("log level (error/warn/info/debug/trace)"))?;
        }
        "logging.log_to_file" => {
            config.logging.log_to_file = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "logging.log_file" => config.logging.log_file = PathBuf::from(value),
        "telemetry.enabled" => {
            config.telemetry.enabled = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        _ => {
            return Err(CoreError::ConfigUnknownKey {
                key: key.to_string(),
                valid_keys: config.known_keys(),
            });
        }
    }
    Ok(())
}

fn parse_duration(s: &str) -> Result<Duration, humantime::DurationError> {
    humantime::parse_duration(s)
}

/// Get the string representation of a config field value.
pub(crate) fn get_field_value(config: &AppConfig, key: &str) -> Option<String> {
    match key {
        "catalog.url" => Some(config.catalog.url.clone()),
        "catalog.cache_ttl" => {
            Some(humantime::format_duration(config.catalog.cache_ttl).to_string())
        }
        "paths.download_dir" => Some(config.paths.download_dir.display().to_string()),
        "paths.cache_dir" => Some(config.paths.cache_dir.display().to_string()),
        "paths.data_dir" => Some(config.paths.data_dir.display().to_string()),
        "paths.keep_installers" => Some(config.paths.keep_installers.to_string()),
        "paths.purge_installers_after_days" => {
            Some(config.paths.purge_installers_after_days.to_string())
        }
        "network.proxy" => Some(
            config
                .network
                .proxy
                .clone()
                .unwrap_or_else(|| "none".to_string()),
        ),
        "network.timeout" => Some(humantime::format_duration(config.network.timeout).to_string()),
        "network.connect_timeout" => {
            Some(humantime::format_duration(config.network.connect_timeout).to_string())
        }
        "network.user_agent" => Some(config.network.user_agent.clone()),
        "network.download_speed_limit" => Some(config.network.download_speed_limit.to_string()),
        "updates.auto_check" => Some(config.updates.auto_check.to_string()),
        "updates.check_interval" => {
            Some(humantime::format_duration(config.updates.check_interval).to_string())
        }
        "logging.level" => Some(config.logging.level.to_string()),
        "logging.log_to_file" => Some(config.logging.log_to_file.to_string()),
        "logging.log_file" => Some(config.logging.log_file.display().to_string()),
        "telemetry.enabled" => Some(config.telemetry.enabled.to_string()),
        _ => None,
    }
}
