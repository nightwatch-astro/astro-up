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

        let (mut pe_candidates, mut pe_probed) = self.probe_pe_files(software, &candidates).await;
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

    // -- Probe methods --

    // T004: Registry discovery probe
    #[cfg(windows)]
    async fn probe_registry(
        &self,
        software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        use winreg::RegKey;
        use winreg::enums::*;

        let mut candidates = Vec::new();
        let mut probed = Vec::new();

        let search_name = software.name.to_lowercase();
        let search_id = software.id.as_ref().to_lowercase();

        let search_paths: &[(winreg::HKEY, u32, &str)] = &[
            (
                HKEY_LOCAL_MACHINE,
                KEY_READ | KEY_WOW64_64KEY,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_LOCAL_MACHINE,
                KEY_READ | KEY_WOW64_32KEY,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_CURRENT_USER,
                KEY_READ,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
        ];

        for (hive, flags, path) in search_paths {
            let root = RegKey::predef(*hive);
            let Ok(uninstall_key) = root.open_subkey_with_flags(path, *flags) else {
                probed.push(ProbedLocation {
                    method: DetectionMethod::Registry,
                    location: path.to_string(),
                    result: "not_found".into(),
                });
                continue;
            };

            for subkey_name in uninstall_key.enum_keys().flatten() {
                let Ok(subkey) = uninstall_key.open_subkey(&subkey_name) else {
                    continue;
                };

                let display_name: String = subkey.get_value("DisplayName").unwrap_or_default();
                if display_name.is_empty() {
                    continue;
                }

                let display_lower = display_name.to_lowercase();

                // Match: manifest name (primary), package ID (fallback)
                let matched =
                    display_lower.contains(&search_name) || display_lower.contains(&search_id);

                let location = format!("{path}\\{subkey_name}");
                probed.push(ProbedLocation {
                    method: DetectionMethod::Registry,
                    location: location.clone(),
                    result: if matched { "found" } else { "not_found" }.into(),
                });

                if !matched {
                    continue;
                }

                let version_str: Option<String> = subkey
                    .get_value::<String, _>("DisplayVersion")
                    .ok()
                    .filter(|s| !s.trim().is_empty());
                let install_location: Option<String> = subkey
                    .get_value::<String, _>("InstallLocation")
                    .ok()
                    .filter(|s| !s.trim().is_empty());

                let version = version_str.as_deref().map(|s| Version::parse(s.trim()));
                let confidence = if version.is_some() {
                    DiscoveryConfidence::High
                } else {
                    DiscoveryConfidence::Medium
                };

                let registry_key =
                    format!(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{subkey_name}");

                candidates.push(DiscoveryCandidate {
                    method: DetectionMethod::Registry,
                    config: DetectionConfig {
                        method: DetectionMethod::Registry,
                        registry_key: Some(subkey_name.clone()),
                        registry_value: Some("DisplayVersion".into()),
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
                    version,
                    install_path: install_location.map(|s| s.trim().to_string()),
                    display_name: Some(display_name),
                    registry_key_name: Some(subkey_name),
                });
            }
        }

        (candidates, probed)
    }

    #[cfg(not(windows))]
    async fn probe_registry(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        (vec![], vec![])
    }

    // T005: PE discovery probe
    #[cfg(windows)]
    async fn probe_pe_files(
        &self,
        software: &Software,
        registry_candidates: &[DiscoveryCandidate],
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        let mut candidates = Vec::new();
        let mut probed = Vec::new();

        // Collect directories to search: InstallLocation from registry + common dirs
        let mut search_dirs: Vec<String> = Vec::new();

        for reg in registry_candidates {
            if let Some(ref path) = reg.install_path {
                search_dirs.push(path.clone());
            }
        }

        // Add common program dirs
        let name = &software.name;
        let slug = &software.slug;
        for token in ["{program_files}", "{program_files_x86}"] {
            for dir_name in [name.as_str(), slug.as_str()] {
                let template = format!("{token}\\{dir_name}");
                if let Some(expanded) = self.resolver.expand(&template) {
                    if std::path::Path::new(&expanded).is_dir() {
                        search_dirs.push(expanded);
                    }
                }
            }
        }

        search_dirs.dedup();

        for dir in &search_dirs {
            let dir_path = std::path::Path::new(dir);
            if !dir_path.is_dir() {
                probed.push(ProbedLocation {
                    method: DetectionMethod::PeFile,
                    location: dir.clone(),
                    result: "not_found".into(),
                });
                continue;
            }

            // Find .exe files (non-recursive, top level only for speed)
            let entries: Vec<_> = match std::fs::read_dir(dir_path) {
                Ok(entries) => entries
                    .flatten()
                    .filter(|e| {
                        e.path()
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
                    })
                    .collect(),
                Err(_) => {
                    probed.push(ProbedLocation {
                        method: DetectionMethod::PeFile,
                        location: dir.clone(),
                        result: "error: cannot read directory".into(),
                    });
                    continue;
                }
            };

            for entry in entries {
                let exe_path = entry.path();
                let exe_str = exe_path.to_string_lossy().to_string();

                let pe_result = tokio::task::spawn_blocking({
                    let exe_path = exe_path.clone();
                    move || -> Option<(Option<Version>, Option<String>)> {
                        let data = std::fs::read(&exe_path).ok()?;
                        let pe = pelite::PeFile::from_bytes(&data).ok()?;
                        let resources = pe.resources().ok()?;
                        let version_info = resources.version_info().ok()?;

                        // Try binary version first
                        let version = version_info.fixed().map(|fixed| {
                            let v = fixed.dwFileVersion;
                            Version::parse(&format!("{}.{}.{}", v.Major, v.Minor, v.Patch))
                        });

                        // Try ProductName from string info
                        let product_name = version_info.translation().first().and_then(|&lang| {
                            version_info
                                .value(lang, "ProductName")
                                .map(|s| s.trim_end_matches('\0').to_string())
                        });

                        Some((version, product_name))
                    }
                })
                .await
                .ok()
                .flatten();

                let Some((version, _product_name)) = pe_result else {
                    probed.push(ProbedLocation {
                        method: DetectionMethod::PeFile,
                        location: exe_str,
                        result: "error: cannot read PE".into(),
                    });
                    continue;
                };

                let confidence = if version.is_some() {
                    DiscoveryConfidence::High
                } else {
                    DiscoveryConfidence::Medium
                };

                // Tokenize the path for the config
                let tokenized = self.resolver.tokenize(&exe_str);

                probed.push(ProbedLocation {
                    method: DetectionMethod::PeFile,
                    location: exe_str,
                    result: "found".into(),
                });

                candidates.push(DiscoveryCandidate {
                    method: DetectionMethod::PeFile,
                    config: DetectionConfig {
                        method: DetectionMethod::PeFile,
                        file_path: Some(tokenized),
                        registry_key: None,
                        registry_value: None,
                        version_regex: None,
                        product_code: None,
                        upgrade_code: None,
                        inf_provider: None,
                        device_class: None,
                        inf_name: None,
                        fallback: None,
                    },
                    confidence,
                    version,
                    install_path: Some(dir.clone()),
                    display_name: None,
                    registry_key_name: None,
                });
            }
        }

        (candidates, probed)
    }

    #[cfg(not(windows))]
    async fn probe_pe_files(
        &self,
        _software: &Software,
        _registry_candidates: &[DiscoveryCandidate],
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        (vec![], vec![])
    }

    // T006: Remaining probes — file_exists, config_file, ascom, wmi, driver_store

    async fn probe_file_exists(
        &self,
        software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        let mut candidates = Vec::new();
        let mut probed = Vec::new();

        // Check common locations for the package executable
        let name = &software.name;
        let slug = &software.slug;
        let patterns = [
            format!("{{program_files}}\\{name}\\{slug}.exe"),
            format!("{{program_files}}\\{name}\\{name}.exe"),
            format!("{{program_files_x86}}\\{name}\\{slug}.exe"),
            format!("{{local_app_data}}\\{name}\\{slug}.exe"),
        ];

        for pattern in &patterns {
            let Some(expanded) = self.resolver.expand(pattern) else {
                continue;
            };
            let exists = std::path::Path::new(&expanded).exists();
            probed.push(ProbedLocation {
                method: DetectionMethod::FileExists,
                location: expanded.clone(),
                result: if exists { "found" } else { "not_found" }.into(),
            });

            if exists {
                let tokenized = self.resolver.tokenize(&expanded);
                candidates.push(DiscoveryCandidate {
                    method: DetectionMethod::FileExists,
                    config: DetectionConfig {
                        method: DetectionMethod::FileExists,
                        file_path: Some(tokenized),
                        registry_key: None,
                        registry_value: None,
                        version_regex: None,
                        product_code: None,
                        upgrade_code: None,
                        inf_provider: None,
                        device_class: None,
                        inf_name: None,
                        fallback: None,
                    },
                    confidence: DiscoveryConfidence::Medium,
                    version: None,
                    install_path: Some(expanded),
                    display_name: None,
                    registry_key_name: None,
                });
            }
        }

        (candidates, probed)
    }

    async fn probe_config_file(
        &self,
        _software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // ConfigFile detection requires knowing the config path and version regex.
        // Discovery can't infer these — they must be provided in the manifest.
        // No-op for blind discovery.
        (vec![], vec![])
    }

    #[cfg(windows)]
    async fn probe_ascom(
        &self,
        software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        use winreg::RegKey;
        use winreg::enums::*;

        let mut candidates = Vec::new();
        let mut probed = Vec::new();

        let search_name = software.name.to_lowercase();

        // ASCOM drivers register under HKLM\SOFTWARE\ASCOM\{DeviceType} Drivers
        let device_types = [
            "Camera",
            "Telescope",
            "Focuser",
            "FilterWheel",
            "Dome",
            "Rotator",
            "Switch",
            "SafetyMonitor",
            "ObservingConditions",
        ];

        let root = RegKey::predef(HKEY_LOCAL_MACHINE);
        for device_type in device_types {
            let path = format!(r"SOFTWARE\ASCOM\{device_type} Drivers");
            let Ok(drivers_key) = root.open_subkey_with_flags(&path, KEY_READ) else {
                continue;
            };

            for subkey_name in drivers_key.enum_keys().flatten() {
                let Ok(subkey) = drivers_key.open_subkey(&subkey_name) else {
                    continue;
                };

                // Check if the driver name matches
                let driver_name: String = subkey.get_value("").unwrap_or_default();
                let matches = driver_name.to_lowercase().contains(&search_name)
                    || subkey_name.to_lowercase().contains(&search_name);

                let location = format!("{path}\\{subkey_name}");
                probed.push(ProbedLocation {
                    method: DetectionMethod::AscomProfile,
                    location: location.clone(),
                    result: if matches { "found" } else { "not_found" }.into(),
                });

                if matches {
                    candidates.push(DiscoveryCandidate {
                        method: DetectionMethod::AscomProfile,
                        config: DetectionConfig {
                            method: DetectionMethod::AscomProfile,
                            registry_key: Some(format!("{device_type} Drivers\\{subkey_name}")),
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
                        confidence: DiscoveryConfidence::Low,
                        version: None,
                        install_path: None,
                        display_name: Some(driver_name),
                        registry_key_name: Some(subkey_name),
                    });
                }
            }
        }

        (candidates, probed)
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
        software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        let mut candidates = Vec::new();
        let mut probed = Vec::new();

        let search_name = software.name.to_lowercase();
        let publisher = software.publisher.as_deref().unwrap_or("").to_lowercase();

        let wmi_result: Result<Vec<std::collections::HashMap<String, wmi::Variant>>, _> =
            tokio::time::timeout(std::time::Duration::from_secs(10), async {
                tokio::task::spawn_blocking(move || {
                    let com = wmi::COMLibrary::new().ok()?;
                    let wmi_con = wmi::WMIConnection::new(com).ok()?;
                    wmi_con
                        .raw_query("SELECT DriverProviderName, DriverVersion, DeviceClass, InfName FROM Win32_PnPSignedDriver")
                        .ok()
                })
                .await
                .ok()?
            })
            .await;

        let drivers = match wmi_result {
            Ok(Some(drivers)) => drivers,
            _ => {
                probed.push(ProbedLocation {
                    method: DetectionMethod::Wmi,
                    location: "Win32_PnPSignedDriver".into(),
                    result: "error: WMI query failed or timed out".into(),
                });
                return (candidates, probed);
            }
        };

        for driver in &drivers {
            let provider = driver
                .get("DriverProviderName")
                .and_then(|v| match v {
                    wmi::Variant::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("");

            let matches = provider.to_lowercase().contains(&search_name)
                || (!publisher.is_empty() && provider.to_lowercase().contains(&publisher));

            if !matches {
                continue;
            }

            let version_str = driver.get("DriverVersion").and_then(|v| match v {
                wmi::Variant::String(s) => Some(s.clone()),
                _ => None,
            });
            let device_class = driver.get("DeviceClass").and_then(|v| match v {
                wmi::Variant::String(s) => Some(s.clone()),
                _ => None,
            });
            let inf_name = driver.get("InfName").and_then(|v| match v {
                wmi::Variant::String(s) => Some(s.clone()),
                _ => None,
            });

            probed.push(ProbedLocation {
                method: DetectionMethod::Wmi,
                location: format!("WMI: {provider}"),
                result: "found".into(),
            });

            candidates.push(DiscoveryCandidate {
                method: DetectionMethod::Wmi,
                config: DetectionConfig {
                    method: DetectionMethod::Wmi,
                    inf_provider: Some(provider.to_string()),
                    device_class: device_class.clone(),
                    inf_name: inf_name.clone(),
                    registry_key: None,
                    registry_value: None,
                    file_path: None,
                    version_regex: None,
                    product_code: None,
                    upgrade_code: None,
                    fallback: None,
                },
                confidence: DiscoveryConfidence::Low,
                version: version_str.map(|s| Version::parse(&s)),
                install_path: None,
                display_name: Some(provider.to_string()),
                registry_key_name: None,
            });
        }

        (candidates, probed)
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
        software: &Software,
    ) -> (Vec<DiscoveryCandidate>, Vec<ProbedLocation>) {
        // DriverStore detection uses WMI backend with InfName filter.
        // Discovery delegates to probe_wmi which already captures InfName.
        // No separate probe needed — WMI results with InfName are sufficient.
        let _ = software;
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
            make_candidate(DetectionMethod::Registry, DiscoveryConfidence::Medium, None),
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
