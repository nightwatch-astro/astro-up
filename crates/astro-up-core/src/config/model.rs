use std::path::PathBuf;
use std::time::Duration;

use garde::Validate;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Log level for the application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_tracing_level(&self) -> tracing::Level {
        match self {
            Self::Error => tracing::Level::ERROR,
            Self::Warn => tracing::Level::WARN,
            Self::Info => tracing::Level::INFO,
            Self::Debug => tracing::Level::DEBUG,
            Self::Trace => tracing::Level::TRACE,
        }
    }
}

fn validate_positive_duration(value: &Duration, _ctx: &()) -> garde::Result {
    if value.is_zero() {
        return Err(garde::Error::new("duration must be positive"));
    }
    Ok(())
}

fn validate_min_one_minute(value: &Duration, _ctx: &()) -> garde::Result {
    if *value < Duration::from_secs(60) {
        return Err(garde::Error::new("duration must be at least 1 minute"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct CatalogConfig {
    #[garde(url)]
    pub url: String,
    #[garde(custom(validate_positive_duration))]
    pub cache_ttl: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct PathsConfig {
    pub download_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
    pub keep_installers: bool,
    pub purge_installers_after_days: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct NetworkConfig {
    #[garde(inner(url))]
    pub proxy: Option<String>,
    #[garde(custom(validate_positive_duration))]
    pub connect_timeout: Duration,
    #[garde(custom(validate_positive_duration))]
    pub timeout: Duration,
    #[garde(length(min = 1))]
    pub user_agent: String,
    pub download_speed_limit: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct UpdateConfig {
    pub auto_check: bool,
    #[garde(custom(validate_min_one_minute))]
    pub check_interval: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct LogConfig {
    pub level: LogLevel,
    pub log_to_file: bool,
    pub log_file: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct TelemetryConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Validate)]
pub struct AppConfig {
    #[garde(dive)]
    pub catalog: CatalogConfig,
    #[garde(dive)]
    pub paths: PathsConfig,
    #[garde(dive)]
    pub network: NetworkConfig,
    #[garde(dive)]
    pub updates: UpdateConfig,
    #[garde(dive)]
    pub logging: LogConfig,
    #[garde(dive)]
    pub telemetry: TelemetryConfig,
}

impl AppConfig {
    /// Discover all valid dot-path keys by introspecting the struct via serde.
    pub fn known_keys(&self) -> Vec<String> {
        let value = serde_json::to_value(self).expect("AppConfig is always serializable");
        collect_keys(&value, "")
    }

    /// Check if a dot-path key is valid.
    pub fn is_known_key(&self, key: &str) -> bool {
        self.known_keys().iter().any(|k| k == key)
    }

    /// Create default config with caller-provided platform paths.
    pub fn with_paths(paths: PathsConfig, log_file: PathBuf) -> Self {
        Self {
            paths,
            logging: LogConfig {
                log_file,
                ..LogConfig::default()
            },
            ..Self::default()
        }
    }
}

/// Recursively collect dot-path keys from a serde_json::Value.
/// Duration fields serialize as `{secs, nanos}` objects — treat them as leaf nodes.
fn collect_keys(value: &serde_json::Value, prefix: &str) -> Vec<String> {
    match value {
        serde_json::Value::Object(map) => {
            // Detect Duration objects: {secs: N, nanos: N}
            if map.contains_key("secs") && map.contains_key("nanos") && map.len() == 2 {
                return vec![prefix.to_string()];
            }
            map.iter()
                .flat_map(|(k, v)| {
                    let key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    if v.is_object() {
                        collect_keys(v, &key)
                    } else {
                        vec![key]
                    }
                })
                .collect()
        }
        _ => vec![prefix.to_string()],
    }
}
