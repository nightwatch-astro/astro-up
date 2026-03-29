use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    #[serde(default)]
    pub min_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyConfig {
    #[serde(default)]
    pub requires: Vec<Dependency>,
    #[serde(default)]
    pub optional: Vec<String>,
}
