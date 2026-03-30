use crate::detect::DetectionResult;
use crate::types::DetectionConfig;

#[cfg(windows)]
pub async fn detect(_config: &DetectionConfig) -> DetectionResult {
    todo!("T018: implement WMI driver detection")
}

#[cfg(not(windows))]
pub async fn detect(_config: &DetectionConfig) -> DetectionResult {
    DetectionResult::Unavailable {
        reason: "WMI detection requires Windows".into(),
    }
}
