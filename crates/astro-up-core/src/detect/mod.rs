mod ascom;
mod cache;
mod file;
mod hardware;
mod path;
pub mod pe;
mod registry;
mod wmi_driver;

pub use cache::DetectionCache;
pub use path::PathResolver;

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::DetectionMethod;
use crate::types::Version;

/// Outcome of detecting a single package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DetectionResult {
    Installed {
        version: Version,
        method: DetectionMethod,
    },
    InstalledUnknownVersion {
        method: DetectionMethod,
    },
    NotInstalled,
    Unavailable {
        reason: String,
    },
}

impl DetectionResult {
    pub fn is_installed(&self) -> bool {
        matches!(
            self,
            DetectionResult::Installed { .. } | DetectionResult::InstalledUnknownVersion { .. }
        )
    }
}

/// Per-package detection outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDetection {
    pub package_id: String,
    pub result: DetectionResult,
}

/// A non-fatal error during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanError {
    pub package_id: String,
    pub method: DetectionMethod,
    pub error: String,
}

/// Result of a full catalog scan.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub results: Vec<PackageDetection>,
    pub errors: Vec<ScanError>,
    #[serde(with = "humantime_serde_compat")]
    pub duration: Duration,
    pub scanned_at: DateTime<Utc>,
}

/// Detection system errors (fatal — abort the scan).
#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("catalog unavailable: {0}")]
    CatalogError(String),

    #[error("ledger error: {0}")]
    LedgerError(String),

    #[error("WMI connection failed: {0}")]
    WmiConnectionError(String),
}

// -- Chain runner --

use crate::types::DetectionConfig;

/// Execute a detection chain, stopping at the first successful result.
pub async fn run_chain(config: &DetectionConfig, resolver: &PathResolver) -> DetectionResult {
    let result = run_single_method(config, resolver).await;

    match &result {
        DetectionResult::Installed { .. } | DetectionResult::InstalledUnknownVersion { .. } => {
            result
        }
        _ => match &config.fallback {
            Some(next) => Box::pin(run_chain(next, resolver)).await,
            None => result,
        },
    }
}

async fn run_single_method(config: &DetectionConfig, resolver: &PathResolver) -> DetectionResult {
    match config.method {
        DetectionMethod::Registry => registry::detect(config).await,
        DetectionMethod::PeFile => pe::detect(config, resolver).await,
        DetectionMethod::Wmi | DetectionMethod::DriverStore => wmi_driver::detect(config).await,
        DetectionMethod::AscomProfile => ascom::detect(config).await,
        DetectionMethod::FileExists => file::detect_exists(config, resolver).await,
        DetectionMethod::ConfigFile => file::detect_config(config, resolver).await,
    }
}

/// Serde compat for Duration (serialize as seconds f64).
mod humantime_serde_compat {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(duration: &Duration, s: S) -> Result<S::Ok, S::Error> {
        duration.as_secs_f64().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let secs = f64::deserialize(d)?;
        Ok(Duration::from_secs_f64(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DetectionMethod;

    #[test]
    fn detection_result_installed_is_installed() {
        let result = DetectionResult::Installed {
            version: Version::parse("1.2.3"),
            method: DetectionMethod::Registry,
        };
        assert!(result.is_installed());
    }

    #[test]
    fn detection_result_not_installed() {
        assert!(!DetectionResult::NotInstalled.is_installed());
    }

    #[test]
    fn detection_result_unavailable() {
        let result = DetectionResult::Unavailable {
            reason: "not on Windows".into(),
        };
        assert!(!result.is_installed());
    }

    #[test]
    fn detection_result_serde_round_trip() {
        let result = DetectionResult::Installed {
            version: Version::parse("3.2.1"),
            method: DetectionMethod::PeFile,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: DetectionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }
}
