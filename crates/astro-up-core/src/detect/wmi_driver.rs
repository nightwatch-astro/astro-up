use tracing::debug;

use crate::detect::DetectionResult;
use crate::types::DetectionConfig;
#[cfg(windows)]
use crate::types::{DetectionMethod, Version};

/// Detect driver versions via WMI Win32_PnPSignedDriver queries.
///
/// Filters by DriverProviderName, DeviceClass, and InfName (AND logic).
/// 10-second timeout enforced via tokio::time::timeout.
#[tracing::instrument(skip_all)]
pub async fn detect(config: &DetectionConfig) -> DetectionResult {
    debug!(
        method = "wmi",
        inf_provider = ?config.inf_provider,
        device_class = ?config.device_class,
        inf_name = ?config.inf_name,
        "detect_wmi entry"
    );
    #[cfg(windows)]
    {
        detect_windows(config).await
    }
    #[cfg(not(windows))]
    {
        let _ = config;
        DetectionResult::Unavailable {
            reason: "WMI detection requires Windows".into(),
        }
    }
}

#[cfg(windows)]
async fn detect_windows(config: &DetectionConfig) -> DetectionResult {
    use std::time::Duration;

    use serde::Deserialize;
    use tracing::warn;

    #[derive(Deserialize, Debug)]
    #[allow(non_snake_case, dead_code)]
    struct PnPSignedDriver {
        DriverProviderName: Option<String>,
        DeviceClass: Option<String>,
        InfName: Option<String>,
        DriverVersion: Option<String>,
        DeviceID: Option<String>,
    }

    // Build WHERE clause from config filters (AND logic)
    let mut conditions = Vec::new();
    if let Some(ref provider) = config.inf_provider {
        conditions.push(format!("DriverProviderName = '{provider}'"));
    }
    if let Some(ref class) = config.device_class {
        conditions.push(format!("DeviceClass = '{class}'"));
    }
    if let Some(ref inf) = config.inf_name {
        conditions.push(format!("InfName = '{inf}'"));
    }

    if conditions.is_empty() {
        return DetectionResult::Unavailable {
            reason: "WMI detection requires at least one filter (inf_provider, device_class, or inf_name)".into(),
        };
    }

    let query = format!(
        "SELECT DriverProviderName, DeviceClass, InfName, DriverVersion, DeviceID FROM Win32_PnPSignedDriver WHERE {}",
        conditions.join(" AND ")
    );

    let result = tokio::time::timeout(Duration::from_secs(10), async {
        let query = query.clone();
        tokio::task::spawn_blocking(move || {
            let con =
                wmi::WMIConnection::new().map_err(|e| format!("WMI connection failed: {e}"))?;
            let drivers: Vec<PnPSignedDriver> = con
                .raw_query(&query)
                .map_err(|e| format!("WMI query failed: {e}"))?;
            Ok::<_, String>(drivers)
        })
        .await
        .map_err(|e| format!("spawn failed: {e}"))?
    })
    .await;

    match result {
        Ok(Ok(drivers)) => {
            if let Some(driver) = drivers.first() {
                if let Some(ref ver) = driver.DriverVersion {
                    if !ver.trim().is_empty() {
                        return DetectionResult::Installed {
                            version: Version::parse(ver.trim()),
                            method: DetectionMethod::Wmi,
                            install_path: None,
                        };
                    }
                }
                DetectionResult::InstalledUnknownVersion {
                    method: DetectionMethod::Wmi,
                    install_path: None,
                }
            } else {
                DetectionResult::NotInstalled
            }
        }
        Ok(Err(ref e)) => {
            warn!(method = "wmi", error = %e, "WMI query failed");
            DetectionResult::Unavailable { reason: e.clone() }
        }
        Err(_) => {
            warn!(method = "wmi", "WMI query timed out (10s)");
            DetectionResult::Unavailable {
                reason: "WMI query timed out (10s)".into(),
            }
        }
    }
}
