use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DetectionMethod {
    Registry,
    PeFile,
    Wmi,
    DriverStore,
    AscomProfile,
    FileExists,
    ConfigFile,
    /// Package is tracked by the ledger only (e.g., firmware flashed to hardware).
    /// No auto-detection possible — version is recorded when Astro-Up handles the download.
    Ledger,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub method: DetectionMethod,
    #[serde(default)]
    pub registry_key: Option<String>,
    #[serde(default)]
    pub registry_value: Option<String>,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub version_regex: Option<String>,
    #[serde(default)]
    pub product_code: Option<String>,
    #[serde(default)]
    pub upgrade_code: Option<String>,
    /// WMI: filter by DriverProviderName (e.g., "ZWO")
    #[serde(default)]
    pub inf_provider: Option<String>,
    /// WMI: filter by DeviceClass
    #[serde(default)]
    pub device_class: Option<String>,
    /// WMI: filter by InfName
    #[serde(default)]
    pub inf_name: Option<String>,
    #[serde(default)]
    pub fallback: Option<Box<DetectionConfig>>,
}
