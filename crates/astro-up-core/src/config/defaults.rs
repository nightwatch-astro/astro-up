use std::path::PathBuf;
use std::time::Duration;

use super::model::{
    AppConfig, CatalogConfig, LogConfig, LogLevel, NetworkConfig, PathsConfig, TelemetryConfig,
    UpdateConfig,
};

impl Default for CatalogConfig {
    fn default() -> Self {
        Self {
            url: "https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db".into(),
            cache_ttl: Duration::from_secs(86400), // 24h
            offline: false,
        }
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            download_dir: PathBuf::default(),
            cache_dir: PathBuf::default(),
            data_dir: PathBuf::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            timeout: Duration::from_secs(30),
            user_agent: format!("astro-up/{}", env!("CARGO_PKG_VERSION")),
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

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            catalog: CatalogConfig::default(),
            paths: PathsConfig::default(),
            network: NetworkConfig::default(),
            updates: UpdateConfig::default(),
            logging: LogConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}
