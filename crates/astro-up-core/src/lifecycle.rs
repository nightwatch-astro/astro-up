//! Lifecycle test runner — orchestrates download, install, detect, uninstall phases
//! for a single package, producing a report and discovered detection config.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::catalog::manifest::ManifestReader;
use crate::detect::PathResolver;
use crate::detect::discovery::DiscoveryScanner;
use crate::detect::wmi_apps;
use crate::error::CoreError;
use crate::types::{DetectionConfig, Software, Version};

/// Status of an individual lifecycle phase.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Pass,
    Fail,
    Skipped,
}

/// Result of a single lifecycle phase.
#[derive(Debug, Clone, Serialize)]
pub struct PhaseResult {
    pub phase: String,
    pub status: PhaseStatus,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub duration: Duration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub logs: Vec<String>,
    pub warnings: Vec<String>,
}

/// Overall lifecycle test status.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    Pass,
    PartialPass,
    Fail,
}

/// WMI snapshot captured after install — used to enrich manifest detection config.
#[derive(Debug, Clone, Serialize)]
pub struct WmiSnapshot {
    /// Exact name as it appears in `Win32_InstalledWin32Program`.
    pub name: String,
    /// Version string from WMI.
    pub version: String,
    /// Vendor/publisher.
    pub vendor: String,
    /// ProgramId (maps to Uninstall registry key name).
    pub program_id: String,
    /// How the match was made.
    pub match_strategy: String,
}

/// Full lifecycle test report for one package.
#[derive(Debug, Clone, Serialize)]
pub struct LifecycleReport {
    pub package_id: String,
    pub version: String,
    pub phases: Vec<PhaseResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_config: Option<DetectionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wmi_snapshot: Option<WmiSnapshot>,
    pub overall_status: LifecycleStatus,
}

/// Options for running a lifecycle test.
pub struct LifecycleOptions {
    pub manifest_path: PathBuf,
    pub package_id: String,
    pub version: Option<String>,
    pub install_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub timeout: Duration,
}

impl Default for LifecycleOptions {
    fn default() -> Self {
        Self {
            manifest_path: PathBuf::new(),
            package_id: String::new(),
            version: None,
            install_dir: None,
            dry_run: false,
            timeout: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// The lifecycle test runner.
pub struct LifecycleRunner;

impl LifecycleRunner {
    /// Run the full lifecycle test for a package.
    pub async fn run(options: &LifecycleOptions) -> Result<LifecycleReport, CoreError> {
        let software = ManifestReader::read_by_id(&options.manifest_path, &options.package_id)?;

        let version_str = match &options.version {
            Some(v) => v.clone(),
            None => Self::resolve_latest_version(&options.manifest_path, &options.package_id)?
                .to_string(),
        };

        let mut phases = Vec::new();
        let mut discovered_config = None;

        // Phase 1: Download
        let download_result = Self::run_download(&software, &version_str, options).await;
        let download_ok = matches!(download_result.status, PhaseStatus::Pass);
        let download_path = if download_ok {
            download_result.logs.first().cloned().map(PathBuf::from)
        } else {
            None
        };
        phases.push(download_result);

        if !download_ok {
            return Ok(Self::build_report(
                &options.package_id,
                &version_str,
                phases,
                None,
                None,
            ));
        }

        let is_download_only = software
            .install
            .as_ref()
            .is_some_and(|i| i.method == crate::types::InstallMethod::DownloadOnly);

        if options.dry_run || is_download_only {
            // Dry-run / download_only: probe without install
            let detect_result = Self::run_discovery(&software, &download_path).await;
            if let Some(ref config) = detect_result
                .logs
                .first()
                .and_then(|s| serde_json::from_str::<DetectionConfig>(s).ok())
            {
                discovered_config = Some(config.clone());
            }
            phases.push(detect_result);
        } else {
            // Full lifecycle: install → detect → verify → uninstall → verify-removal

            // Phase 2: Install
            let install_result =
                Self::run_install(&software, download_path.as_deref(), options).await;
            let install_ok = matches!(install_result.status, PhaseStatus::Pass);
            phases.push(install_result);

            if install_ok {
                // Phase 3: WMI snapshot — capture how the OS sees this package
                let (wmi_phase, wmi_snap) =
                    Self::run_wmi_snapshot(&software.name, &software.aliases).await;
                phases.push(wmi_phase);

                // Phase 4: File search — verify Windows Search finds the EXE
                let search_phase = Self::run_file_search(&software.detection).await;
                phases.push(search_phase);

                // Phase 5: Detect
                let detect_result = Self::run_discovery(&software, &None).await;
                if let Some(ref config) = detect_result
                    .logs
                    .first()
                    .and_then(|s| serde_json::from_str::<DetectionConfig>(s).ok())
                {
                    discovered_config = Some(config.clone());
                }
                phases.push(detect_result);

                // Phase 6: Verify install
                let verify_result =
                    Self::run_verify_install(&discovered_config, &version_str).await;
                phases.push(verify_result);

                // Phase 7: Uninstall
                let uninstall_result = Self::run_uninstall(&options.package_id).await;
                phases.push(uninstall_result);

                // Phase 8: Verify removal
                let removal_result = Self::run_verify_removal(&software).await;
                phases.push(removal_result);

                return Ok(Self::build_report(
                    &options.package_id,
                    &version_str,
                    phases,
                    discovered_config,
                    wmi_snap,
                ));
            } else {
                // Install failed — attempt cleanup
                let cleanup = Self::run_uninstall(&options.package_id).await;
                if !matches!(cleanup.status, PhaseStatus::Pass) {
                    tracing::warn!(
                        package = %options.package_id,
                        "cleanup after failed install also failed"
                    );
                }
                phases.push(PhaseResult {
                    phase: "detect".into(),
                    status: PhaseStatus::Skipped,
                    duration: Duration::ZERO,
                    exit_code: None,
                    logs: vec![],
                    warnings: vec!["skipped: install failed".into()],
                });
            }
        }

        Ok(Self::build_report(
            &options.package_id,
            &version_str,
            phases,
            discovered_config,
            None,
        ))
    }

    /// Resolve the latest version from `versions/{package_id}/` directory.
    pub fn resolve_latest_version(
        manifest_path: &Path,
        package_id: &str,
    ) -> Result<Version, CoreError> {
        let versions_dir = manifest_path.join("versions").join(package_id);
        if !versions_dir.is_dir() {
            return Err(CoreError::NotFound {
                input: format!("versions directory for '{package_id}'"),
            });
        }

        let mut versions: Vec<(Version, String)> = Vec::new();
        for entry in std::fs::read_dir(&versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let version = Version::parse(stem);
                    versions.push((version, stem.to_string()));
                }
            }
        }

        if versions.is_empty() {
            return Err(CoreError::NotFound {
                input: format!("no versions found for '{package_id}'"),
            });
        }

        // Sort by semver (Version::parse produces comparable versions)
        versions.sort_by(|a, b| a.0.cmp(&b.0));
        // SAFETY: versions is non-empty (checked at function entry or populated by loop above)
        #[allow(clippy::unwrap_used)]
        Ok(versions.last().unwrap().0.clone())
    }

    /// Resolve download URL from autoupdate template + version.
    pub fn resolve_download_url(software: &Software, version: &str) -> Result<String, CoreError> {
        let url = software
            .checkver
            .as_ref()
            .and_then(|cv| cv.autoupdate.as_ref())
            .and_then(|au| au.url.as_ref())
            .ok_or_else(|| CoreError::NotFound {
                input: format!("autoupdate URL for '{}'", software.id.as_ref()),
            })?;

        Ok(url.replace("$version", version))
    }

    /// Serialize a DetectionConfig to TOML string.
    pub fn config_to_toml(config: &DetectionConfig) -> String {
        // Use serde to serialize, then wrap in [detection] section
        match toml::to_string_pretty(config) {
            Ok(toml_str) => format!("[detection]\n{toml_str}"),
            Err(e) => format!("# serialization error: {e}"),
        }
    }

    // -- Phase implementations --

    /// Orchestration stub for the download phase.
    ///
    /// Resolves the download URL from the manifest's autoupdate template but does not
    /// invoke `DownloadManager` directly. The GitHub Actions workflow handles actual
    /// binary downloads on Windows runners using the resolved URL. This method sets up
    /// phase tracking and returns the URL as the first log entry for downstream phases.
    async fn run_download(
        software: &Software,
        version: &str,
        _options: &LifecycleOptions,
    ) -> PhaseResult {
        let start = Instant::now();

        let url = match Self::resolve_download_url(software, version) {
            Ok(url) => url,
            Err(e) => {
                return PhaseResult {
                    phase: "download".into(),
                    status: PhaseStatus::Fail,
                    duration: start.elapsed(),
                    exit_code: None,
                    logs: vec![],
                    warnings: vec![format!("cannot resolve download URL: {e}")],
                };
            }
        };

        // For now, report the URL. Actual download uses DownloadManager
        // which requires network config — wired in the CLI handler.
        PhaseResult {
            phase: "download".into(),
            status: PhaseStatus::Pass,
            duration: start.elapsed(),
            exit_code: None,
            logs: vec![url], // First log entry is the resolved URL
            warnings: vec![],
        }
    }

    /// Orchestration stub for the install phase.
    ///
    /// Does not invoke `InstallerService` directly because the lifecycle test workflow
    /// runs the actual installer binary on Windows runners via the CLI. This method
    /// provides phase tracking structure — on non-Windows it returns `Skipped`, on
    /// Windows it returns `Pass` to allow the subsequent detection/verify phases to run.
    async fn run_install(
        _software: &Software,
        _download_path: Option<&Path>,
        _options: &LifecycleOptions,
    ) -> PhaseResult {
        let start = Instant::now();

        // Install uses InstallerService — wired in the CLI handler.
        // This is a placeholder for the orchestration structure.
        #[cfg(not(windows))]
        {
            PhaseResult {
                phase: "install".into(),
                status: PhaseStatus::Skipped,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![],
                warnings: vec!["install requires Windows".into()],
            }
        }

        #[cfg(windows)]
        {
            PhaseResult {
                phase: "install".into(),
                status: PhaseStatus::Pass,
                duration: start.elapsed(),
                exit_code: Some(0),
                logs: vec![],
                warnings: vec![],
            }
        }
    }

    /// Capture WMI snapshot after install — records how the OS sees this package.
    ///
    /// This data is used to enrich the manifest with exact `program_id` and
    /// `match_name` values for reliable detection without fuzzy matching.
    async fn run_wmi_snapshot(
        package_name: &str,
        aliases: &[String],
    ) -> (PhaseResult, Option<WmiSnapshot>) {
        let start = Instant::now();

        let wmi_result = wmi_apps::enumerate_installed();
        let programs = match wmi_result {
            Ok(scan) => scan.programs,
            Err(e) => {
                return (
                    PhaseResult {
                        phase: "wmi-snapshot".into(),
                        status: PhaseStatus::Skipped,
                        duration: start.elapsed(),
                        exit_code: None,
                        logs: vec![],
                        warnings: vec![format!("WMI unavailable: {e}")],
                    },
                    None,
                );
            }
        };

        let matched = wmi_apps::match_package(package_name, aliases, None, &programs);

        match matched {
            Some(m) => {
                let snapshot = WmiSnapshot {
                    name: m.program.name.clone(),
                    version: m.program.version.clone(),
                    vendor: m.program.vendor.clone(),
                    program_id: m.program.program_id.clone(),
                    match_strategy: format!("{:?}", m.strategy),
                };
                let log_json =
                    serde_json::to_string(&snapshot).unwrap_or_else(|_| "{}".to_string());
                (
                    PhaseResult {
                        phase: "wmi-snapshot".into(),
                        status: PhaseStatus::Pass,
                        duration: start.elapsed(),
                        exit_code: None,
                        logs: vec![log_json],
                        warnings: vec![],
                    },
                    Some(snapshot),
                )
            }
            None => (
                PhaseResult {
                    phase: "wmi-snapshot".into(),
                    status: PhaseStatus::Fail,
                    duration: start.elapsed(),
                    exit_code: None,
                    logs: vec![],
                    warnings: vec![format!(
                        "package '{package_name}' not found in WMI after install"
                    )],
                },
                None,
            ),
        }
    }

    /// Verify that Windows Search can find the package's EXE/DLL.
    ///
    /// Extracts the filename from the detection config's `file_path` template
    /// and queries the Windows Search index. This validates the search-based
    /// file discovery path used by PE detection.
    async fn run_file_search(detection: &Option<DetectionConfig>) -> PhaseResult {
        let start = Instant::now();

        let Some(config) = detection else {
            return PhaseResult {
                phase: "file-search".into(),
                status: PhaseStatus::Skipped,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![],
                warnings: vec!["no detection config — nothing to search for".into()],
            };
        };

        let Some(ref template) = config.file_path else {
            return PhaseResult {
                phase: "file-search".into(),
                status: PhaseStatus::Skipped,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![],
                warnings: vec!["detection has no file_path — search not applicable".into()],
            };
        };

        let Some(filename) = std::path::Path::new(template)
            .file_name()
            .and_then(|f| f.to_str())
        else {
            return PhaseResult {
                phase: "file-search".into(),
                status: PhaseStatus::Fail,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![],
                warnings: vec![format!("cannot extract filename from template: {template}")],
            };
        };

        match crate::detect::search::find_file(filename) {
            Ok(Some(found_path)) => PhaseResult {
                phase: "file-search".into(),
                status: PhaseStatus::Pass,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![format!("found: {found_path}")],
                warnings: vec![],
            },
            Ok(None) => PhaseResult {
                phase: "file-search".into(),
                status: PhaseStatus::Fail,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![],
                warnings: vec![format!("'{filename}' not found in Windows Search index")],
            },
            Err(e) => PhaseResult {
                phase: "file-search".into(),
                status: PhaseStatus::Skipped,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![],
                warnings: vec![format!("Windows Search unavailable: {e}")],
            },
        }
    }

    async fn run_discovery(software: &Software, _download_path: &Option<PathBuf>) -> PhaseResult {
        let start = Instant::now();
        let resolver = PathResolver::new();
        let scanner = DiscoveryScanner::new(resolver);

        let result = scanner.discover(software).await;

        let config_json = result
            .best_config
            .as_ref()
            .and_then(|c| serde_json::to_string(c).ok());

        let status = if result.best_config.is_some() {
            PhaseStatus::Pass
        } else {
            PhaseStatus::Fail
        };

        let mut logs: Vec<String> = Vec::new();
        if let Some(json) = config_json {
            logs.push(json);
        }
        for loc in &result.probed_locations {
            logs.push(format!("{}: {} → {}", loc.method, loc.location, loc.result));
        }

        PhaseResult {
            phase: "detect".into(),
            status,
            duration: start.elapsed(),
            exit_code: None,
            logs,
            warnings: if result.candidates.is_empty() {
                vec!["no detection signatures found".into()]
            } else {
                vec![]
            },
        }
    }

    async fn run_verify_install(
        config: &Option<DetectionConfig>,
        expected_version: &str,
    ) -> PhaseResult {
        let start = Instant::now();

        let Some(config) = config else {
            return PhaseResult {
                phase: "verify-install".into(),
                status: PhaseStatus::Skipped,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![],
                warnings: vec!["no detection config to verify against".into()],
            };
        };

        // Run detection with the discovered config
        let resolver = PathResolver::new();
        let result = crate::detect::run_chain(config, &resolver, None).await;

        let (status, logs) = match &result {
            crate::detect::DetectionResult::Installed { version, .. } => {
                let matches = version.to_string() == expected_version;
                (
                    if matches {
                        PhaseStatus::Pass
                    } else {
                        PhaseStatus::Fail
                    },
                    vec![format!(
                        "detected version: {version}, expected: {expected_version}"
                    )],
                )
            }
            crate::detect::DetectionResult::InstalledUnknownVersion { .. } => (
                PhaseStatus::Pass,
                vec!["installed but version unknown".into()],
            ),
            crate::detect::DetectionResult::NotInstalled => (
                PhaseStatus::Fail,
                vec!["detection reports not installed after install".into()],
            ),
            crate::detect::DetectionResult::Unavailable { reason } => (
                PhaseStatus::Fail,
                vec![format!("detection unavailable: {reason}")],
            ),
        };

        PhaseResult {
            phase: "verify-install".into(),
            status,
            duration: start.elapsed(),
            exit_code: None,
            logs,
            warnings: vec![],
        }
    }

    async fn run_uninstall(package_id: &str) -> PhaseResult {
        let start = Instant::now();

        #[cfg(not(windows))]
        {
            let _ = package_id;
            PhaseResult {
                phase: "uninstall".into(),
                status: PhaseStatus::Skipped,
                duration: start.elapsed(),
                exit_code: None,
                logs: vec![],
                warnings: vec!["uninstall requires Windows".into()],
            }
        }

        #[cfg(windows)]
        {
            use crate::install::uninstall;

            let command = uninstall::find_uninstall_command(package_id);
            match command {
                Some(cmd) => match uninstall::run_uninstall(&cmd, true).await {
                    Ok(()) => PhaseResult {
                        phase: "uninstall".into(),
                        status: PhaseStatus::Pass,
                        duration: start.elapsed(),
                        exit_code: Some(0),
                        logs: vec![format!("uninstall command: {cmd}")],
                        warnings: vec![],
                    },
                    Err(e) => PhaseResult {
                        phase: "uninstall".into(),
                        status: PhaseStatus::Fail,
                        duration: start.elapsed(),
                        exit_code: None,
                        logs: vec![format!("uninstall command: {cmd}")],
                        warnings: vec![format!("uninstall failed: {e}")],
                    },
                },
                None => PhaseResult {
                    phase: "uninstall".into(),
                    status: PhaseStatus::Fail,
                    duration: start.elapsed(),
                    exit_code: None,
                    logs: vec![],
                    warnings: vec!["no uninstall command found in registry".into()],
                },
            }
        }
    }

    async fn run_verify_removal(software: &Software) -> PhaseResult {
        let start = Instant::now();
        let resolver = PathResolver::new();
        let scanner = DiscoveryScanner::new(resolver);

        let result = scanner.discover(software).await;

        let mut warnings = Vec::new();
        if !result.candidates.is_empty() {
            for c in &result.candidates {
                warnings.push(format!(
                    "leftover: {} — {:?}",
                    c.method,
                    c.install_path.as_deref().unwrap_or("unknown")
                ));
            }
        }

        PhaseResult {
            phase: "verify-removal".into(),
            status: if result.candidates.is_empty() {
                PhaseStatus::Pass
            } else {
                PhaseStatus::Fail
            },
            duration: start.elapsed(),
            exit_code: None,
            logs: vec![],
            warnings,
        }
    }

    fn build_report(
        package_id: &str,
        version: &str,
        phases: Vec<PhaseResult>,
        discovered_config: Option<DetectionConfig>,
        wmi_snapshot: Option<WmiSnapshot>,
    ) -> LifecycleReport {
        let has_fail = phases.iter().any(|p| matches!(p.status, PhaseStatus::Fail));
        let has_pass = phases.iter().any(|p| matches!(p.status, PhaseStatus::Pass));
        let has_warnings = phases.iter().any(|p| !p.warnings.is_empty());

        let overall_status = if has_fail {
            LifecycleStatus::Fail
        } else if has_warnings {
            LifecycleStatus::PartialPass
        } else if has_pass {
            LifecycleStatus::Pass
        } else {
            LifecycleStatus::Fail
        };

        LifecycleReport {
            package_id: package_id.to_string(),
            version: version.to_string(),
            phases,
            discovered_config,
            wmi_snapshot,
            overall_status,
        }
    }
}

fn serialize_duration_ms<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u64(duration.as_millis() as u64)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn resolve_download_url_substitution() {
        use crate::catalog::PackageId;
        use crate::types::{AutoupdateConfig, Category, CheckverConfig, SoftwareType};

        let software = Software {
            id: PackageId::try_from("nina-app".to_string()).unwrap(),
            slug: "nina".into(),
            name: "N.I.N.A.".into(),
            software_type: SoftwareType::Application,
            category: Category::Capture,
            os: vec![],
            description: None,
            homepage: None,
            publisher: None,
            icon_url: None,
            license: None,
            license_url: None,
            aliases: vec![],
            tags: vec![],
            notes: None,
            docs_url: None,
            channel: None,
            min_os_version: None,
            manifest_version: None,
            detection: None,
            install: None,
            checkver: Some(CheckverConfig {
                provider: None,
                github: None,
                owner: None,
                repo: None,
                url: None,
                regex: None,
                jsonpath: None,
                asset_pattern: None,
                tag_prefix: None,
                changelog_url: None,
                autoupdate: Some(AutoupdateConfig {
                    url: Some(
                        "https://github.com/nina/releases/download/v$version/NINA-$version.exe"
                            .into(),
                    ),
                    hash: None,
                }),
                hash: None,
            }),
            dependencies: None,
            hardware: None,
            backup: None,
            versioning: None,
        };

        let url = LifecycleRunner::resolve_download_url(&software, "3.1.2").unwrap();
        assert_eq!(
            url,
            "https://github.com/nina/releases/download/v3.1.2/NINA-3.1.2.exe"
        );
    }

    #[test]
    fn resolve_latest_version_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        let versions_dir = dir.path().join("versions").join("nina-app");
        std::fs::create_dir_all(&versions_dir).unwrap();
        std::fs::write(versions_dir.join("3.0.0.json"), "{}").unwrap();
        std::fs::write(versions_dir.join("3.1.2.json"), "{}").unwrap();
        std::fs::write(versions_dir.join("2.9.0.json"), "{}").unwrap();

        let version = LifecycleRunner::resolve_latest_version(dir.path(), "nina-app").unwrap();
        assert_eq!(version.to_string(), "3.1.2");
    }

    #[test]
    fn resolve_latest_version_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let result = LifecycleRunner::resolve_latest_version(dir.path(), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn config_to_toml_output() {
        let config = DetectionConfig {
            method: crate::types::DetectionMethod::Registry,
            registry_key: Some("NINA 2".into()),
            registry_value: Some("DisplayVersion".into()),
            file_path: None,
            version_regex: None,
            product_code: None,
            upgrade_code: None,
            inf_provider: None,
            device_class: None,
            inf_name: None,
            fallback: None,
        };

        let toml = LifecycleRunner::config_to_toml(&config);
        assert!(toml.starts_with("[detection]"));
        assert!(toml.contains("registry"));
        assert!(toml.contains("NINA 2"));
    }
}
