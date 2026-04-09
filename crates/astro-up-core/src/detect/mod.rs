pub mod ascom;
mod cache;
pub mod discovery;
mod file;
pub mod hardware;
mod path;
pub mod pe;
mod registry;
pub mod scanner;
pub(crate) mod search;
pub mod wmi_apps;
mod wmi_driver;

pub use cache::DetectionCache;
pub use hardware::{HardwareMatch, VidPid};
pub use path::PathResolver;
pub use scanner::{LedgerStore, PackageSource, Scanner};

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::DetectionMethod;
use crate::types::Version;

/// Outcome of detecting a single package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DetectionResult {
    Installed {
        version: Version,
        method: DetectionMethod,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        install_path: Option<String>,
    },
    InstalledUnknownVersion {
        method: DetectionMethod,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        install_path: Option<String>,
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
            Self::Installed { .. } | Self::InstalledUnknownVersion { .. }
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

/// Context for WMI app detection within the detection chain.
pub struct WmiContext<'a> {
    pub package_name: &'a str,
    pub aliases: &'a [String],
    pub programs: &'a [wmi_apps::InstalledProgram],
}

/// Execute a detection chain, stopping at the first successful result.
///
/// `ledger_path` is the install ledger's recorded executable path for the package,
/// used as a fallback by PE detection when the manifest path template cannot be resolved.
/// `wmi_ctx` provides WMI app data for `WmiApps` detection method.
pub async fn run_chain(
    config: &DetectionConfig,
    resolver: &PathResolver,
    ledger_path: Option<&str>,
    wmi_ctx: Option<&WmiContext<'_>>,
) -> DetectionResult {
    let result = run_single_method(config, resolver, ledger_path, wmi_ctx).await;

    match &result {
        DetectionResult::Installed { .. } | DetectionResult::InstalledUnknownVersion { .. } => {
            result
        }
        _ => match &config.fallback {
            Some(next) => Box::pin(run_chain(next, resolver, ledger_path, wmi_ctx)).await,
            None => result,
        },
    }
}

async fn run_single_method(
    config: &DetectionConfig,
    resolver: &PathResolver,
    ledger_path: Option<&str>,
    wmi_ctx: Option<&WmiContext<'_>>,
) -> DetectionResult {
    match config.method {
        DetectionMethod::Registry => registry::detect(config).await,
        DetectionMethod::PeFile => pe::detect(config, resolver, ledger_path).await,
        DetectionMethod::Wmi | DetectionMethod::DriverStore => wmi_driver::detect(config).await,
        DetectionMethod::WmiApps => {
            if let Some(ctx) = wmi_ctx {
                let matched =
                    wmi_apps::match_package(ctx.package_name, ctx.aliases, None, ctx.programs);
                if let Some(m) = matched {
                    if let Some(version) = m.version() {
                        DetectionResult::Installed {
                            version,
                            method: DetectionMethod::WmiApps,
                            install_path: None,
                        }
                    } else {
                        DetectionResult::InstalledUnknownVersion {
                            method: DetectionMethod::WmiApps,
                            install_path: None,
                        }
                    }
                } else {
                    DetectionResult::NotInstalled
                }
            } else {
                DetectionResult::Unavailable {
                    reason: "WMI app data not available".into(),
                }
            }
        }
        DetectionMethod::AscomProfile => ascom::detect(config).await,
        DetectionMethod::FileExists => file::detect_exists(config, resolver).await,
        DetectionMethod::ConfigFile => file::detect_config(config, resolver).await,
        DetectionMethod::Ledger => DetectionResult::Unavailable {
            reason: "ledger-only detection — version tracked via download ledger".into(),
        },
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::DetectionMethod;

    #[test]
    fn detection_result_installed_is_installed() {
        let result = DetectionResult::Installed {
            version: Version::parse("1.2.3"),
            method: DetectionMethod::Registry,
            install_path: None,
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
            install_path: Some("C:\\Program Files\\Test".into()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: DetectionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }
}
