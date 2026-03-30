use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use crate::catalog::PackageId;

use super::backup::BackupConfig;
use super::checkver::CheckverConfig;
use super::dependency::DependencyConfig;
use super::detection::DetectionConfig;
use super::hardware::HardwareConfig;
use super::install::InstallConfig;
use super::versioning::VersioningConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SoftwareType {
    Application,
    Driver,
    Runtime,
    Database,
    UsbDriver,
    Resource,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Category {
    Capture,
    Guiding,
    Platesolving,
    Equipment,
    Focusing,
    Planetarium,
    Viewers,
    Prerequisites,
    Usb,
    Driver,
}

/// The central aggregate — a complete software package definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Software {
    pub id: PackageId,
    #[serde(default)]
    pub slug: String,
    pub name: String,
    #[serde(rename = "type")]
    pub software_type: SoftwareType,
    pub category: Category,
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub license_url: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub docs_url: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub min_os_version: Option<String>,
    #[serde(default)]
    pub manifest_version: Option<u32>,

    // Nested configs — all optional
    #[serde(default)]
    pub detection: Option<DetectionConfig>,
    #[serde(default)]
    pub install: Option<InstallConfig>,
    #[serde(default)]
    pub checkver: Option<CheckverConfig>,
    #[serde(default)]
    pub dependencies: Option<DependencyConfig>,
    #[serde(default)]
    pub hardware: Option<HardwareConfig>,
    #[serde(default)]
    pub backup: Option<BackupConfig>,
    #[serde(default)]
    pub versioning: Option<VersioningConfig>,
}
