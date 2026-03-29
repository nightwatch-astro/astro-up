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
    #[serde(default)]
    pub fallback: Option<Box<DetectionConfig>>,
}
