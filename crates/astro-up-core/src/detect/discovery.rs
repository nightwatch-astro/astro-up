//! Detection discovery — blind probing of all detection methods to discover
//! detection signatures for packages without a `[detection]` config.

use serde::Serialize;

use crate::types::{DetectionConfig, DetectionMethod, Software, Version};

use super::PathResolver;

/// Confidence level for a discovery candidate, ordered from low to high.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryConfidence {
    Low,    // WMI, ASCOM, DriverStore
    Medium, // Registry without version, FileExists
    High,   // Registry with version, PeFile with version
}

/// A single detection signature found during blind probing.
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryCandidate {
    pub method: DetectionMethod,
    pub config: DetectionConfig,
    pub confidence: DiscoveryConfidence,
    pub version: Option<Version>,
    pub install_path: Option<String>,
    pub display_name: Option<String>,
    pub registry_key_name: Option<String>,
}

/// Debug info for a single probe attempt.
#[derive(Debug, Clone, Serialize)]
pub struct ProbedLocation {
    pub method: DetectionMethod,
    pub location: String,
    pub result: String,
}

/// Full discovery result for one package.
#[derive(Debug, Clone, Serialize)]
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
    pub fn new(resolver: PathResolver) -> Self {
        Self { resolver }
    }

    /// Run all detection methods for a package, rank results, generate config.
    pub async fn discover(&self, software: &Software) -> DiscoveryResult {
        let mut candidates = Vec::new();
        let mut probed = Vec::new();

        // Run all probes, collecting candidates and probe locations
        let (mut reg_candidates, mut reg_probed) = self.probe_registry(software).await;
        candidates.append(&mut reg_candidates);
        probed.append(&mut reg_probed);

        let (mut pe_candidates, mut pe_probed) =
            self.probe_pe_files(software, &candidates).await;
        candidates.append(&mut pe_candidates);
        probed.append(&mut pe_probed);

        let (mut file_candidates, mut file_probed) = self.probe_file_exists(software).await;
        candidates.append(&mut file_candidates);
        probed.append(&mut file_probed);

        let (mut config_candidates, mut config_probed) = self.probe_config_file(software).await;
        candidates.append(&mut config_candidates);
        probed.append(&mut config_probed);

        let (mut ascom_candidates, mut ascom_probed) = self.probe_ascom(software).await;
        candidates.append(&mut ascom_candidates);
        probed.append(&mut ascom_probed);

        let (mut wmi_candidates, mut wmi_probed) = self.probe_wmi(software).await;
        candidates.append(&mut wmi_candidates);
        probed.append(&mut wmi_probed);

        let (mut driver_candidates, mut driver_probed) = self.probe_driver_store(software).await;
        candidates.append(&mut driver_candidates);
        probed.append(&mut driver_probed);

        // Sort by confidence (high first), then by version presence
        candidates.sort_by(|a, b| {
            b.confidence.cmp(&a.confidence).then_with(|| {
                let a_has_ver = a.version.is_some();
                let b_has_ver = b.version.is_some();
                b_has_ver.cmp(&a_has_ver)
            })
        });

        let best_config = Self::build_config(&candidates);

        DiscoveryResult {
            candidates,
            best_config,
            probed_locations: probed,
        }
    }

    /// Generate a DetectionConfig from the best candidate, with fallback from second-best.
    /// Max depth: 3 levels (primary + 2 fallbacks). Skips candidates with duplicate methods.
    pub fn build_config(candidates: &[DiscoveryCandidate]) -> Option<DetectionConfig> {
        if candidates.is_empty() {
            return None;
        }

        // Collect up to 3 candidates with distinct methods
        let mut distinct: Vec<&DiscoveryCandidate> = Vec::with_capacity(3);
        for c in candidates {
            if distinct.len() >= 3 {
                break;
            }
            if !distinct.iter().any(|d| d.method == c.method) {
                distinct.push(c);
            }
        }

        let mut config = distinct[0].config.clone();

        if distinct.len() > 1 {
            let mut fallback = distinct[1].config.clone();

            if distinct.len() > 2 {
                fallback.fallback = Some(Box::new(distinct[2].config.clone()));
            }

            config.fallback = Some(Box::new(fallback));
        }

        Some(config)
    }

    // -- Probe methods (stubbed for now, implemented in T004-T006) --

    #[cfg(windows)]
    async fn probe_registry(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // T004: Implement registry discovery probe
        (vec![], vec![])
    }

    #[cfg(not(windows))]
    async fn probe_registry(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        (vec![], vec![])
    }

    #[cfg(windows)]
    async fn probe_pe_files(
        &self,
        _software: &Software,
        _registry_candidates: &[DiscoveryCandidate],
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // T005: Implement PE discovery probe
        (vec![], vec![])
    }

    #[cfg(not(windows))]
    async fn probe_pe_files(
        &self,
        _software: &Software,
        _registry_candidates: &[DiscoveryCandidate],
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        (vec![], vec![])
    }

    async fn probe_file_exists(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // T006: Implement file exists probe
        (vec![], vec![])
    }

    async fn probe_config_file(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // T006: Implement config file probe
        (vec![], vec![])
    }

    #[cfg(windows)]
    async fn probe_ascom(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // T006: Implement ASCOM probe
        (vec![], vec![])
    }

    #[cfg(not(windows))]
    async fn probe_ascom(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        (vec![], vec![])
    }

    #[cfg(windows)]
    async fn probe_wmi(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // T006: Implement WMI probe
        (vec![], vec![])
    }

    #[cfg(not(windows))]
    async fn probe_wmi(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        (vec![], vec![])
    }

    #[cfg(windows)]
    async fn probe_driver_store(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // T006: Implement driver store probe
        (vec![], vec![])
    }

    #[cfg(not(windows))]
    async fn probe_driver_store(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        (vec![], vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(
        method: DetectionMethod,
        confidence: DiscoveryConfidence,
        version: Option<&str>,
    ) -> DiscoveryCandidate {
        DiscoveryCandidate {
            method: method.clone(),
            config: DetectionConfig {
                method,
                registry_key: None,
                registry_value: None,
                file_path: None,
                version_regex: None,
                product_code: None,
                upgrade_code: None,
                inf_provider: None,
                device_class: None,
                inf_name: None,
                fallback: None,
            },
            confidence,
            version: version.map(Version::parse),
            install_path: None,
            display_name: None,
            registry_key_name: None,
        }
    }

    #[test]
    fn build_config_empty() {
        assert!(DiscoveryScanner::build_config(&[]).is_none());
    }

    #[test]
    fn build_config_single_candidate() {
        let candidates = vec![make_candidate(
            DetectionMethod::Registry,
            DiscoveryConfidence::High,
            Some("1.0.0"),
        )];
        let config = DiscoveryScanner::build_config(&candidates).unwrap();
        assert_eq!(config.method, DetectionMethod::Registry);
        assert!(config.fallback.is_none());
    }

    #[test]
    fn build_config_with_fallback() {
        let candidates = vec![
            make_candidate(
                DetectionMethod::Registry,
                DiscoveryConfidence::High,
                Some("1.0.0"),
            ),
            make_candidate(
                DetectionMethod::PeFile,
                DiscoveryConfidence::High,
                Some("1.0.0"),
            ),
        ];
        let config = DiscoveryScanner::build_config(&candidates).unwrap();
        assert_eq!(config.method, DetectionMethod::Registry);
        let fallback = config.fallback.unwrap();
        assert_eq!(fallback.method, DetectionMethod::PeFile);
        assert!(fallback.fallback.is_none());
    }

    #[test]
    fn build_config_max_depth_3() {
        let candidates = vec![
            make_candidate(
                DetectionMethod::Registry,
                DiscoveryConfidence::High,
                Some("1.0.0"),
            ),
            make_candidate(
                DetectionMethod::PeFile,
                DiscoveryConfidence::High,
                Some("1.0.0"),
            ),
            make_candidate(
                DetectionMethod::FileExists,
                DiscoveryConfidence::Medium,
                None,
            ),
            make_candidate(DetectionMethod::Wmi, DiscoveryConfidence::Low, None),
        ];
        let config = DiscoveryScanner::build_config(&candidates).unwrap();
        assert_eq!(config.method, DetectionMethod::Registry);
        let fb1 = config.fallback.unwrap();
        assert_eq!(fb1.method, DetectionMethod::PeFile);
        let fb2 = fb1.fallback.unwrap();
        assert_eq!(fb2.method, DetectionMethod::FileExists);
        assert!(fb2.fallback.is_none()); // Max depth 3, no 4th level
    }

    #[test]
    fn build_config_skips_duplicate_method() {
        let candidates = vec![
            make_candidate(
                DetectionMethod::Registry,
                DiscoveryConfidence::High,
                Some("1.0.0"),
            ),
            make_candidate(
                DetectionMethod::Registry,
                DiscoveryConfidence::Medium,
                None,
            ),
            make_candidate(
                DetectionMethod::PeFile,
                DiscoveryConfidence::High,
                Some("1.0.0"),
            ),
        ];
        let config = DiscoveryScanner::build_config(&candidates).unwrap();
        assert_eq!(config.method, DetectionMethod::Registry);
        // Second candidate is same method, so fallback should be PeFile (third)
        let fallback = config.fallback.unwrap();
        assert_eq!(fallback.method, DetectionMethod::PeFile);
    }
}
