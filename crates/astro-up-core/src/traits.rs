use std::path::Path;

use crate::error::CoreError;
use crate::release::Release;
use crate::types::{
    CheckverConfig, DetectionConfig, DetectionMethod, InstallConfig, InstallMethod, Version,
};

/// Options for an install operation.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub asset_path: String,
    pub config: InstallConfig,
    pub quiet: bool,
}

/// Callback for download progress: (bytes_downloaded, total_bytes, speed_bytes_per_sec)
pub type ProgressCallback = Box<dyn Fn(u64, u64, f64) + Send>;

/// Options for a download operation.
pub struct DownloadOptions {
    pub on_progress: Option<ProgressCallback>,
    pub checksum: Option<String>,
    pub resume: bool,
}

/// Detects the installed version of software on the local system.
#[trait_variant::make(DetectorDyn: Send)]
pub trait Detector {
    async fn detect(&self, cfg: &DetectionConfig) -> Result<Version, CoreError>;
    fn supports(&self, method: &DetectionMethod) -> bool;
}

/// Checks for the latest version of software from a remote source.
#[trait_variant::make(ProviderDyn: Send)]
pub trait Provider {
    fn name(&self) -> &str;
    async fn latest_release(&self, cfg: &CheckverConfig) -> Result<Release, CoreError>;
    async fn list_releases(
        &self,
        cfg: &CheckverConfig,
        limit: usize,
    ) -> Result<Vec<Release>, CoreError>;
}

/// Installs or updates software on the local system.
#[trait_variant::make(InstallerDyn: Send)]
pub trait Installer {
    async fn install(&self, opts: &InstallOptions) -> Result<(), CoreError>;
    fn supports(&self, method: &InstallMethod) -> bool;
}

/// Downloads files with progress reporting and verification.
#[trait_variant::make(DownloaderDyn: Send)]
pub trait Downloader {
    async fn download(
        &self,
        url: &str,
        dest: &Path,
        opts: &DownloadOptions,
    ) -> Result<(), CoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDetector;

    impl Detector for MockDetector {
        async fn detect(&self, _cfg: &DetectionConfig) -> Result<Version, CoreError> {
            Ok(Version::parse("1.0.0"))
        }

        fn supports(&self, _method: &DetectionMethod) -> bool {
            true
        }
    }

    #[test]
    fn mock_detector_supports() {
        let detector = MockDetector;
        assert!(detector.supports(&DetectionMethod::Registry));
    }

    #[tokio::test]
    async fn mock_detector_detect() {
        let detector = MockDetector;
        let cfg = DetectionConfig {
            method: DetectionMethod::Registry,
            registry_key: None,
            registry_value: None,
            file_path: None,
            version_regex: None,
            product_code: None,
            upgrade_code: None,
            fallback: None,
        };
        let version = detector.detect(&cfg).await.unwrap();
        assert_eq!(version.raw, "1.0.0");
    }
}
