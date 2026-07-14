// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    #[serde(default)]
    pub min_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyConfig {
    #[serde(default)]
    pub requires: Vec<Dependency>,
    #[serde(default)]
    pub optional: Vec<String>,
}
