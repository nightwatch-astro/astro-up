use std::path::PathBuf;
use std::time::Duration;

use super::model::{
    BackupPolicyConfig, BackupSchedule, CatalogConfig, FontSize, InstallMethod, InstallScope,
    LogConfig, LogLevel, NetworkConfig, NotificationsConfig, PathsConfig, ScanInterval, ThemeMode,
    UiConfig, UpdateConfig,
};

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            font_size: FontSize::default(),
            auto_scan_on_launch: false,
            scan_interval: ScanInterval::default(),
            default_install_scope: InstallScope::default(),
            default_install_method: InstallMethod::default(),
            auto_check_updates: true,
            check_interval: Duration::from_secs(86400), // 24h
            auto_notify_updates: true,
            survey_threshold: 3,
            survey_dismissed_at: None,
            survey_completed_at: None,
        }
    }
}

// StartupConfig: all fields false — uses #[derive(Default)] on the struct

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            display_duration: 5,
            show_errors: true,
            show_warnings: true,
            show_update_available: true,
            show_operation_complete: true,
        }
    }
}

impl Default for BackupPolicyConfig {
    fn default() -> Self {
        Self {
            scheduled_enabled: false,
            schedule: BackupSchedule::default(),
            max_per_package: 5,
            max_total_size_mb: 0,
            max_age_days: 0,
        }
    }
}

impl Default for CatalogConfig {
    fn default() -> Self {
        Self {
            url: "https://github.com/nightwatch-astro/astro-up-manifests/releases/download/catalog/latest/catalog.db".into(),
            cache_ttl: Duration::from_secs(86400), // 24h
        }
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            download_dir: PathBuf::default(),
            cache_dir: PathBuf::default(),
            data_dir: PathBuf::default(),
            portable_apps_dir: PathBuf::default(),
            keep_installers: true,
            purge_installers_after_days: 30,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            connect_timeout: Duration::from_secs(10),
            timeout: Duration::from_secs(30),
            user_agent: format!("astro-up/{}", env!("CARGO_PKG_VERSION")),
            download_speed_limit: 0,
        }
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            check_interval: Duration::from_secs(86400), // 24h
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            log_to_file: false,
            log_file: PathBuf::default(),
            max_age_days: 365,
        }
    }
}

// AppConfig: all fields are Default — use #[derive(Default)] on the struct
