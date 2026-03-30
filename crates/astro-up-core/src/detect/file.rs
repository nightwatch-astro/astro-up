use crate::detect::{DetectionResult, PathResolver};
use crate::types::DetectionConfig;

pub async fn detect_exists(_config: &DetectionConfig, _resolver: &PathResolver) -> DetectionResult {
    todo!("T015: implement file_exists detection")
}

pub async fn detect_config(_config: &DetectionConfig, _resolver: &PathResolver) -> DetectionResult {
    todo!("T015: implement config_file detection")
}
