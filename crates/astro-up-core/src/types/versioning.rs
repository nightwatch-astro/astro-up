use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PolicyLevel {
    Minor,
    Major,
    Manual,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionOverride {
    #[serde(default)]
    pub install_dir: Option<String>,
    #[serde(default)]
    pub registry_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersioningConfig {
    #[serde(default)]
    pub side_by_side: bool,
    #[serde(default)]
    pub major_version_pattern: Option<String>,
    #[serde(default)]
    pub overrides: HashMap<String, VersionOverride>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePolicy {
    pub default: PolicyLevel,
    #[serde(default)]
    pub per_package: HashMap<String, PolicyLevel>,
}
