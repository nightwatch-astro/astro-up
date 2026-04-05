use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CheckMethod {
    Github,
    Gitlab,
    DirectUrl,
    HttpHead,
    HtmlScrape,
    BrowserScrape,
    PeDownload,
    Manual,
    RuntimeScrape,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum HashMode {
    Extract,
    Json,
    Download,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashConfig {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default)]
    pub jsonpath: Option<String>,
    #[serde(default)]
    pub mode: Option<HashMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoupdateConfig {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub hash: Option<HashConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckverConfig {
    #[serde(default)]
    pub provider: Option<CheckMethod>,
    /// GitHub shorthand: "owner/repo"
    #[serde(default)]
    pub github: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default)]
    pub jsonpath: Option<String>,
    #[serde(default)]
    pub asset_pattern: Option<String>,
    #[serde(default)]
    pub tag_prefix: Option<String>,
    #[serde(default)]
    pub changelog_url: Option<String>,
    #[serde(default)]
    pub autoupdate: Option<AutoupdateConfig>,
    #[serde(default)]
    pub hash: Option<HashConfig>,
}
