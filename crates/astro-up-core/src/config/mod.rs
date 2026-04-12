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
    ScanInterval, StartupConfig, ThemeMode, UiConfig, UpdateConfig,
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

/// Maximum config database file size (10 MB). Typical configs are <100 KB.
const MAX_CONFIG_DB_BYTES: u64 = 10 * 1024 * 1024;

/// Open a ConfigStore, handling corruption by renaming the corrupt file and starting fresh.
fn open_store(db_path: &Path) -> Result<ConfigStore, CoreError> {
    // Validate file size before opening to prevent OOM on corrupt/bloated databases
    if db_path.exists() {
        if let Ok(metadata) = std::fs::metadata(db_path) {
            if metadata.len() > MAX_CONFIG_DB_BYTES {
                return Err(CoreError::Validation(format!(
                    "config database exceeds maximum size ({} bytes > {MAX_CONFIG_DB_BYTES} bytes): {}",
                    metadata.len(),
                    db_path.display()
                )));
            }
        }
    }

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
        "ui.scan_interval" => {
            config.ui.scan_interval = model::ScanInterval::from_str(value)
                .map_err(|_| parse_err("scan interval (manual/on_startup/hourly/daily/weekly)"))?;
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
        "ui.survey_threshold" => {
            config.ui.survey_threshold = value
                .parse::<u32>()
                .map_err(|_| parse_err("integer (minimum operations before survey)"))?;
        }
        "ui.survey_dismissed_at" => {
            config.ui.survey_dismissed_at = if value.is_empty() || value == "none" {
                None
            } else {
                Some(value.to_string())
            };
        }
        "ui.survey_completed_at" => {
            config.ui.survey_completed_at = if value.is_empty() || value == "none" {
                None
            } else {
                Some(value.to_string())
            };
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
        _ => {
            return Err(CoreError::ConfigUnknownKey {
                key: key.to_string(),
                valid_keys: config.known_keys(),
            });
        }
    }
    Ok(())
}

/// Check if the user is eligible for the feedback survey.
///
/// Eligible when:
/// - Successful operation count >= threshold
/// - Survey was never completed (completed_at is None)
/// - Survey was never dismissed, or dismissal was >30 days ago
pub fn check_survey_eligible(
    conn: &rusqlite::Connection,
    config: &UiConfig,
) -> Result<bool, CoreError> {
    use chrono::{DateTime, Utc};

    // Already completed — never show again
    if config.survey_completed_at.is_some() {
        debug!("survey already completed, not eligible");
        return Ok(false);
    }

    // Dismissed within the last 30 days — snooze
    if let Some(ref dismissed_str) = config.survey_dismissed_at {
        if let Ok(dismissed) = DateTime::parse_from_rfc3339(dismissed_str) {
            let days_since = (Utc::now() - dismissed.with_timezone(&Utc)).num_days();
            if days_since <= 30 {
                debug!(days_since, "survey snoozed, not eligible");
                return Ok(false);
            }
        }
    }

    // Check operation count
    let count = crate::engine::history::count_successful_operations(conn)?;
    let eligible = count >= u64::from(config.survey_threshold);
    debug!(
        count,
        threshold = config.survey_threshold,
        eligible,
        "survey eligibility check"
    );
    Ok(eligible)
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
        "ui.scan_interval" => Some(config.ui.scan_interval.to_string()),
        "ui.default_install_scope" => Some(config.ui.default_install_scope.to_string()),
        "ui.default_install_method" => Some(config.ui.default_install_method.to_string()),
        "ui.auto_check_updates" => Some(config.ui.auto_check_updates.to_string()),
        "ui.check_interval" => {
            Some(humantime::format_duration(config.ui.check_interval).to_string())
        }
        "ui.auto_notify_updates" => Some(config.ui.auto_notify_updates.to_string()),
        "ui.survey_threshold" => Some(config.ui.survey_threshold.to_string()),
        "ui.survey_dismissed_at" => config.ui.survey_dismissed_at.clone(),
        "ui.survey_completed_at" => config.ui.survey_completed_at.clone(),
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
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use chrono::{Duration as ChronoDuration, Utc};

    fn setup_db_with_ops(count: usize) -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::engine::history::create_table(&conn).unwrap();

        for i in 0..count {
            let record = crate::engine::history::OperationRecord {
                id: 0,
                package_id: format!("pkg-{i}"),
                operation_type: crate::engine::history::OperationType::Install,
                from_version: None,
                to_version: Some("1.0.0".into()),
                status: crate::engine::history::OperationStatus::Success,
                duration_ms: 1000,
                error_message: None,
                created_at: Utc::now(),
            };
            crate::engine::history::record_operation(&conn, &record).unwrap();
        }
        conn
    }

    #[test]
    fn survey_eligible_when_threshold_met() {
        let conn = setup_db_with_ops(3);
        let config = UiConfig::default();
        assert!(check_survey_eligible(&conn, &config).unwrap());
    }

    #[test]
    fn survey_not_eligible_below_threshold() {
        let conn = setup_db_with_ops(2);
        let config = UiConfig::default();
        assert!(!check_survey_eligible(&conn, &config).unwrap());
    }

    #[test]
    fn survey_not_eligible_when_completed() {
        let conn = setup_db_with_ops(5);
        let config = UiConfig {
            survey_completed_at: Some(Utc::now().to_rfc3339()),
            ..UiConfig::default()
        };
        assert!(!check_survey_eligible(&conn, &config).unwrap());
    }

    #[test]
    fn survey_not_eligible_when_snoozed_recently() {
        let conn = setup_db_with_ops(5);
        let dismissed = Utc::now() - ChronoDuration::days(10);
        let config = UiConfig {
            survey_dismissed_at: Some(dismissed.to_rfc3339()),
            ..UiConfig::default()
        };
        assert!(!check_survey_eligible(&conn, &config).unwrap());
    }

    #[test]
    fn survey_eligible_when_snooze_expired() {
        let conn = setup_db_with_ops(5);
        let dismissed = Utc::now() - ChronoDuration::days(31);
        let config = UiConfig {
            survey_dismissed_at: Some(dismissed.to_rfc3339()),
            ..UiConfig::default()
        };
        assert!(check_survey_eligible(&conn, &config).unwrap());
    }

    #[test]
    fn survey_not_eligible_at_snooze_boundary() {
        let conn = setup_db_with_ops(5);
        let dismissed = Utc::now() - ChronoDuration::days(29);
        let config = UiConfig {
            survey_dismissed_at: Some(dismissed.to_rfc3339()),
            ..UiConfig::default()
        };
        assert!(!check_survey_eligible(&conn, &config).unwrap());
    }
}
