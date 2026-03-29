use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareConfig {
    #[serde(default)]
    pub vid_pid: Vec<String>,
    #[serde(default)]
    pub device_class: Option<String>,
    #[serde(default)]
    pub inf_provider: Option<String>,
}
