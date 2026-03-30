use crate::detect::DetectionResult;
use crate::types::DetectionConfig;

#[cfg(windows)]
pub async fn detect(_config: &DetectionConfig) -> DetectionResult {
    todo!("T025: implement ASCOM Profile detection")
}

#[cfg(not(windows))]
pub async fn detect(_config: &DetectionConfig) -> DetectionResult {
    DetectionResult::Unavailable {
        reason: "ASCOM detection requires Windows".into(),
    }
}
