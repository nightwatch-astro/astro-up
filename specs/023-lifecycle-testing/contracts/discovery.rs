// Contract: Detection Discovery Scanner
// Location: crates/astro-up-core/src/detect/discovery.rs

use crate::detect::{DetectionResult, PathResolver};
use crate::types::{DetectionConfig, DetectionMethod, Software, Version};

/// Confidence level for a discovery candidate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiscoveryConfidence {
    Low,    // WMI, ASCOM, DriverStore
    Medium, // Registry without version, FileExists
    High,   // Registry with version, PeFile with version
}

/// A single detection signature found during blind probing.
#[derive(Debug, Clone)]
pub struct DiscoveryCandidate {
    pub method: DetectionMethod,
    pub config: DetectionConfig,
    pub confidence: DiscoveryConfidence,
    pub version: Option<Version>,
    pub install_path: Option<String>,
}

/// Debug info for a single probe attempt.
#[derive(Debug, Clone, Serialize)]
pub struct ProbedLocation {
    pub method: DetectionMethod,
    pub location: String,
    pub result: String,
}

/// Full discovery result for one package.
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub candidates: Vec<DiscoveryCandidate>,
    pub best_config: Option<DetectionConfig>,
    pub probed_locations: Vec<ProbedLocation>,
}

/// The discovery scanner — probes all detection methods blind.
pub struct DiscoveryScanner {
    resolver: PathResolver,
}

impl DiscoveryScanner {
    pub fn new(resolver: PathResolver) -> Self;

    /// Run all detection methods for a package, rank results, generate config.
    /// `software` provides the manifest name and publisher for matching.
    pub async fn discover(&self, software: &Software) -> DiscoveryResult;

    /// Generate a DetectionConfig from the best candidate, with fallback from second-best.
    pub fn build_config(candidates: &[DiscoveryCandidate]) -> Option<DetectionConfig>;
}
