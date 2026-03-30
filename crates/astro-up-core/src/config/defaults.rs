use std::path::PathBuf;
use std::time::Duration;

use super::model::{
    CatalogConfig, LogConfig, LogLevel, NetworkConfig, UpdateConfig,
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

// PathsConfig: all fields are PathBuf::default() — use #[derive(Default)] on the struct

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

// TelemetryConfig: all fields are false — use #[derive(Default)] on the struct
// AppConfig: all fields are Default — use #[derive(Default)] on the struct
