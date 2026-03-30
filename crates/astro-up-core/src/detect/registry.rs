use crate::detect::DetectionResult;
use crate::types::DetectionConfig;

#[cfg(windows)]
pub async fn detect(_config: &DetectionConfig) -> DetectionResult {
    todo!("T010: implement registry detection")
}

#[cfg(not(windows))]
pub async fn detect(_config: &DetectionConfig) -> DetectionResult {
    DetectionResult::Unavailable {
        reason: "registry detection requires Windows".into(),
    }
}
