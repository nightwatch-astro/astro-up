// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareConfig {
    #[serde(default)]
    pub vid_pid: Vec<String>,
    #[serde(default)]
    pub device_class: Option<String>,
    #[serde(default)]
    pub inf_provider: Option<String>,
}
