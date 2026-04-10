mod api;
mod defaults;
mod model;
mod store;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use garde::Validate;
use tracing::{debug, warn};

use crate::error::CoreError;

pub use api::{config_get, config_list, config_reset, config_set};
pub use model::{
    AppConfig, BackupPolicyConfig, BackupSchedule, CatalogConfig, FontSize, InstallMethod,
    InstallScope, LogConfig, LogLevel, NetworkConfig, NotificationsConfig, PathsConfig,
    StartupConfig, TelemetryConfig, ThemeMode, UiConfig, UpdateConfig,
};
pub use store::ConfigStore;

/// Load configuration with 3-layer precedence: defaults → SQLite → CLI flags.
pub fn load_config(
    db_path: &Path,
    default_paths: PathsConfig,
    log_file: PathBuf,
    cli_overrides: &[(&str, &str)],
) -> Result<AppConfig, CoreError> {
    debug!(db_path = %db_path.display(), overrides = cli_overrides.len(), "loading configuration");

    // Layer 1: compiled defaults with caller-provided platform paths
    let mut config = AppConfig::with_paths(default_paths, log_file);

    // Layer 2: SQLite stored overrides (skip unknown keys gracefully — they may
    // come from a newer version that wrote config keys this version doesn't know).
    let store = open_store(db_path)?;
    let stored = store.list()?;
    merge_overrides_lenient(&mut config, &stored);

    // Layer 3: CLI flag overrides (highest precedence)
    let cli_pairs: Vec<(String, String)> = cli_overrides
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    merge_overrides(&mut config, &cli_pairs)?;

    // Validate
    if let Err(e) = config.validate() {
        warn!(error = %e, "config validation failed, check stored overrides");
        return Err(CoreError::from(e));
    }

    debug!(
        theme = %config.ui.theme,
        log_level = %config.logging.level,
        "configuration loaded successfully"
    );

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
    warn!(
        error = %original_err,
        corrupt_path = %corrupt_path.display(),
        "config database corrupt, renaming and starting fresh"
    );
    if let Err(e) = std::fs::rename(db_path, &corrupt_path) {
        warn!(path = %db_path.display(), error = %e, "failed to rename corrupt config database");
    }
    let conn = rusqlite::Connection::open(db_path)?;
    Ok(ConfigStore::new(conn)?)
}

/// Merge key-value overrides into an AppConfig. Errors on unknown keys.
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

/// Merge key-value overrides, skipping unknown keys with a warning.
/// Used for DB-stored config which may contain keys from a newer version.
fn merge_overrides_lenient(config: &mut AppConfig, overrides: &[(String, String)]) {
    for (key, value) in overrides {
        if !config.is_known_key(key) {
            warn!(key, "ignoring unknown config key from database");
            continue;
        }
        if let Err(e) = set_field(config, key, value) {
            warn!(key, error = %e, "ignoring unparsable config value from database");
        }
    }
}

/// Set a single field on AppConfig by dot-path key.
pub(crate) fn set_field(config: &mut AppConfig, key: &str, value: &str) -> Result<(), CoreError> {
    let parse_err = |expected: &str| CoreError::ConfigParse {
        key: key.to_string(),
        expected: expected.to_string(),
        got: value.to_string(),
    };

    match key {
        // ui section
        "ui.theme" => {
            config.ui.theme = model::ThemeMode::from_str(value)
                .map_err(|_| parse_err("theme (system/dark/light)"))?;
        }
        "ui.font_size" => {
            config.ui.font_size = model::FontSize::from_str(value)
                .map_err(|_| parse_err("font size (small/medium/large)"))?;
        }
        "ui.auto_scan_on_launch" => {
            config.ui.auto_scan_on_launch = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "ui.default_install_scope" => {
            config.ui.default_install_scope = model::InstallScope::from_str(value)
                .map_err(|_| parse_err("scope (user/machine)"))?;
        }
        "ui.default_install_method" => {
            config.ui.default_install_method = model::InstallMethod::from_str(value)
                .map_err(|_| parse_err("method (silent/interactive)"))?;
        }
        "ui.auto_check_updates" => {
            config.ui.auto_check_updates = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "ui.check_interval" => {
            config.ui.check_interval =
                parse_duration(value).map_err(|_| parse_err("duration (e.g. 24h, 6h)"))?;
        }
        "ui.auto_notify_updates" => {
            config.ui.auto_notify_updates = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "ui.auto_install_updates" => {
            config.ui.auto_install_updates = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        // startup section
        "startup.start_at_login" => {
            config.startup.start_at_login = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "startup.start_minimized" => {
            config.startup.start_minimized = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "startup.minimize_to_tray_on_close" => {
            config.startup.minimize_to_tray_on_close = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        // notifications section
        "notifications.enabled" => {
            config.notifications.enabled = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "notifications.display_duration" => {
            config.notifications.display_duration = value
                .parse::<u32>()
                .map_err(|_| parse_err("integer seconds (0 = never auto-dismiss)"))?;
        }
        "notifications.show_errors" => {
            config.notifications.show_errors = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "notifications.show_warnings" => {
            config.notifications.show_warnings = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "notifications.show_update_available" => {
            config.notifications.show_update_available = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "notifications.show_operation_complete" => {
            config.notifications.show_operation_complete = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        // backup_policy section
        "backup_policy.scheduled_enabled" => {
            config.backup_policy.scheduled_enabled = value
                .parse::<bool>()
                .map_err(|_| parse_err("boolean (true/false)"))?;
        }
        "backup_policy.schedule" => {
            config.backup_policy.schedule = model::BackupSchedule::from_str(value)
                .map_err(|_| parse_err("schedule (daily/weekly/monthly)"))?;
        }
        "backup_policy.max_per_package" => {
            config.backup_policy.max_per_package = value
                .parse::<u32>()
                .map_err(|_| parse_err("integer (0 = unlimited)"))?;
        }
        "backup_policy.max_total_size_mb" => {
            config.backup_policy.max_total_size_mb = value
                .parse::<u32>()
                .map_err(|_| parse_err("integer MB (0 = unlimited)"))?;
        }
        "backup_policy.max_age_days" => {
            config.backup_policy.max_age_days = value
                .parse::<u32>()
                .map_err(|_| parse_err("integer days (0 = never expire)"))?;
        }
        // catalog section
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
        "logging.max_age_days" => {
            config.logging.max_age_days = value
                .parse::<u32>()
                .map_err(|_| parse_err("number (days, 0 = never)"))?;
        }
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
        // ui section
        "ui.theme" => Some(config.ui.theme.to_string()),
        "ui.font_size" => Some(config.ui.font_size.to_string()),
        "ui.auto_scan_on_launch" => Some(config.ui.auto_scan_on_launch.to_string()),
        "ui.default_install_scope" => Some(config.ui.default_install_scope.to_string()),
        "ui.default_install_method" => Some(config.ui.default_install_method.to_string()),
        "ui.auto_check_updates" => Some(config.ui.auto_check_updates.to_string()),
        "ui.check_interval" => {
            Some(humantime::format_duration(config.ui.check_interval).to_string())
        }
        "ui.auto_notify_updates" => Some(config.ui.auto_notify_updates.to_string()),
        "ui.auto_install_updates" => Some(config.ui.auto_install_updates.to_string()),
        // startup section
        "startup.start_at_login" => Some(config.startup.start_at_login.to_string()),
        "startup.start_minimized" => Some(config.startup.start_minimized.to_string()),
        "startup.minimize_to_tray_on_close" => {
            Some(config.startup.minimize_to_tray_on_close.to_string())
        }
        // notifications section
        "notifications.enabled" => Some(config.notifications.enabled.to_string()),
        "notifications.display_duration" => Some(config.notifications.display_duration.to_string()),
        "notifications.show_errors" => Some(config.notifications.show_errors.to_string()),
        "notifications.show_warnings" => Some(config.notifications.show_warnings.to_string()),
        "notifications.show_update_available" => {
            Some(config.notifications.show_update_available.to_string())
        }
        "notifications.show_operation_complete" => {
            Some(config.notifications.show_operation_complete.to_string())
        }
        // backup_policy section
        "backup_policy.scheduled_enabled" => {
            Some(config.backup_policy.scheduled_enabled.to_string())
        }
        "backup_policy.schedule" => Some(config.backup_policy.schedule.to_string()),
        "backup_policy.max_per_package" => Some(config.backup_policy.max_per_package.to_string()),
        "backup_policy.max_total_size_mb" => {
            Some(config.backup_policy.max_total_size_mb.to_string())
        }
        "backup_policy.max_age_days" => Some(config.backup_policy.max_age_days.to_string()),
        // catalog section
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
        "logging.max_age_days" => Some(config.logging.max_age_days.to_string()),
        "telemetry.enabled" => Some(config.telemetry.enabled.to_string()),
        _ => None,
    }
}
