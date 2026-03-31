use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum InstallMethod {
    Exe,
    Msi,
    InnoSetup,
    Nullsoft,
    Wix,
    Burn,
    Zip,
    ZipWrap,
    Portable,
    DownloadOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Scope {
    Machine,
    User,
    Either,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Elevation {
    Required,
    Prohibited,
    #[serde(rename = "self")]
    #[strum(serialize = "self")]
    Self_,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UpgradeBehavior {
    Install,
    UninstallPrevious,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum KnownExitCode {
    PackageInUse,
    PackageInUseByApplication,
    RebootRequired,
    CancelledByUser,
    AlreadyInstalled,
    MissingDependency,
    DiskFull,
    InsufficientMemory,
    NetworkError,
    ContactSupport,
    RestartRequired,
    SuccessRebootInitiated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallerSwitches {
    #[serde(default)]
    pub silent: Vec<String>,
    #[serde(default)]
    pub interactive: Vec<String>,
    #[serde(default)]
    pub upgrade: Vec<String>,
    #[serde(default)]
    pub install_location: Option<String>,
    #[serde(default)]
    pub log: Option<String>,
    #[serde(default)]
    pub custom: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallConfig {
    pub method: InstallMethod,
    #[serde(default)]
    pub scope: Option<Scope>,
    #[serde(default)]
    pub elevation: Option<Elevation>,
    #[serde(default)]
    pub upgrade_behavior: Option<UpgradeBehavior>,
    #[serde(default)]
    pub install_modes: Vec<String>,
    #[serde(default)]
    pub success_codes: Vec<i32>,
    #[serde(default)]
    pub pre_install: Vec<String>,
    #[serde(default)]
    pub post_install: Vec<String>,
    #[serde(default)]
    pub switches: Option<InstallerSwitches>,
    #[serde(default)]
    pub known_exit_codes: HashMap<String, KnownExitCode>,
    /// Per-manifest timeout override in seconds. Default: 600 (10 min). Valid range: 10-3600.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}
