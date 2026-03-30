use std::path::PathBuf;
use std::time::Duration;

use super::model::{CatalogConfig, LogConfig, LogLevel, NetworkConfig, PathsConfig, UpdateConfig};

impl Default for CatalogConfig {
    fn default() -> Self {
        Self {
            url: "https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db".into(),
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
        }
    }
}

// TelemetryConfig: all fields are false — use #[derive(Default)] on the struct
// AppConfig: all fields are Default — use #[derive(Default)] on the struct
