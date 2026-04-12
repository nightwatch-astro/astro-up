use std::time::Instant;

use chrono::Utc;
use tracing::{debug, info};

use crate::detect::{
    DetectionCache, DetectionError, DetectionResult, PackageDetection, PathResolver, ScanResult,
    run_chain, wmi_apps,
};
use crate::ledger::LedgerEntry;
use crate::types::{DetectionMethod, Software, Version};

/// Minimal interface for getting the list of packages to scan.
///
/// Will be implemented by the catalog module (spec 005) when available.
pub trait PackageSource {
    fn list_all(&self) -> Result<Vec<Software>, DetectionError>;

    /// Look up the latest non-pre-release version entry for a package.
    /// Returns `None` if the package has no version entries.
    fn latest_version(
        &self,
        _id: &crate::catalog::PackageId,
    ) -> Result<Option<crate::catalog::VersionEntry>, DetectionError> {
        Ok(None)
    }
}

/// Minimal interface for reading/writing ledger entries.
///
/// Will be implemented by the ledger/storage module when available.
pub trait LedgerStore {
    fn list_acknowledged(&self) -> Result<Vec<LedgerEntry>, DetectionError>;
    fn upsert_acknowledged(
        &self,
        package_id: &str,
        version: &Version,
    ) -> Result<(), DetectionError>;
    fn remove_acknowledged(&self, package_id: &str) -> Result<(), DetectionError>;
}

/// The main detection scanner. Orchestrates full catalog scans.
pub struct Scanner<P, L> {
    packages: P,
    ledger: L,
    cache: DetectionCache,
    resolver: PathResolver,
}

impl<P: PackageSource, L: LedgerStore> Scanner<P, L> {
    pub fn new(packages: P, ledger: L) -> Self {
        Self {
            packages,
            ledger,
            cache: DetectionCache::new(),
            resolver: PathResolver::new(),
        }
    }

    /// Try to match a package against WMI results. Returns `NotInstalled` if no match.
    fn wmi_fallback(
        id: &str,
        name: &str,
        aliases: &[String],
        wmi_programs: &[wmi_apps::InstalledProgram],
    ) -> DetectionResult {
        let wmi_result = wmi_apps::match_package(name, aliases, None, wmi_programs);

        if let Some(matched) = wmi_result {
            if let Some(version) = matched.version() {
                debug!(
                    package = %id,
                    wmi_name = matched.program.name,
                    version = %version,
                    strategy = ?matched.strategy,
                    "detected via WMI (fallback)"
                );
                DetectionResult::Installed {
                    version,
                    method: DetectionMethod::Wmi,
                    install_path: None,
                }
            } else {
                debug!(
                    package = %id,
                    wmi_name = matched.program.name,
                    "WMI matched but no version (fallback)"
                );
                DetectionResult::InstalledUnknownVersion {
                    method: DetectionMethod::Wmi,
                    install_path: None,
                }
            }
        } else {
            DetectionResult::NotInstalled
        }
    }

    /// Run a full scan across all catalog packages.
    ///
    /// Detection strategy:
    /// 1. WMI enumeration — query `Win32_InstalledWin32Program` once, match all packages
    /// 2. Legacy detection chain — PE file / registry fallback for unmatched packages
    #[tracing::instrument(skip_all)]
    pub async fn scan(&self) -> Result<ScanResult, DetectionError> {
        let start = Instant::now();
        let packages = self.packages.list_all()?;
        info!(package_count = packages.len(), "starting catalog scan");

        // Step 1: WMI enumeration (single system call, with timeout)
        //
        // Use std::thread::spawn + oneshot instead of spawn_blocking so the
        // thread is fully detached. spawn_blocking tasks block tokio runtime
        // shutdown — if WMI hangs (e.g., no WMI service on CI), the runtime
        // can never shut down, causing test timeouts.
        let (wmi_tx, wmi_rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let _ = wmi_tx.send(wmi_apps::enumerate_installed());
        });
        let wmi_programs =
            match tokio::time::timeout(std::time::Duration::from_secs(5), wmi_rx).await {
                Ok(Ok(Ok(scan))) => {
                    debug!(
                        wmi_count = scan.programs.len(),
                        wmi_ms = scan.duration.as_millis() as u64,
                        "WMI enumeration complete"
                    );
                    scan.programs
                }
                Ok(Ok(Err(e))) => {
                    debug!(error = %e, "WMI enumeration failed, using legacy detection only");
                    Vec::new()
                }
                _ => {
                    debug!("WMI enumeration timed out or panicked, using legacy detection only");
                    Vec::new()
                }
            };

        // Build ledger path index for PE detection fallback (#215)
        let ledger_paths: std::collections::HashMap<String, String> = self
            .ledger
            .list_acknowledged()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| {
                let path = entry.install_path?.to_string_lossy().into_owned();
                Some((entry.package_id, path))
            })
            .collect();

        let mut results = Vec::with_capacity(packages.len());
        let mut errors = Vec::new();

        for pkg in &packages {
            let id = pkg.id.to_string();

            // Check cache first
            if let Some(cached) = self.cache.get(&id) {
                results.push(PackageDetection {
                    package_id: id,
                    result: cached,
                });
                continue;
            }

            // Step 2: Detection chain first (when config exists), WMI as fallback.
            // The detection chain uses manifest-defined methods (registry, PE, etc.)
            // which return accurate versions. WMI's Win32_InstalledWin32Program can
            // report internal/MSI versions that differ from the user-visible version
            // (e.g., PHD2 reports 0.6.4 via WMI but 2.6.14 via registry DisplayVersion).
            let result = if let Some(ref detection_config) = pkg.detection {
                let ledger_path = ledger_paths.get(&id).map(String::as_str);
                let wmi_ctx = crate::detect::WmiContext {
                    package_name: &pkg.name,
                    aliases: &pkg.aliases,
                    programs: &wmi_programs,
                };
                let chain_result = run_chain(
                    detection_config,
                    &self.resolver,
                    ledger_path,
                    Some(&wmi_ctx),
                )
                .await;

                match &chain_result {
                    DetectionResult::Installed { .. }
                    | DetectionResult::InstalledUnknownVersion { .. } => chain_result,
                    DetectionResult::Unavailable { reason } => {
                        errors.push(crate::detect::ScanError {
                            package_id: id.clone(),
                            method: detection_config.method.clone(),
                            error: reason.clone(),
                        });
                        // Detection chain failed — fall back to WMI
                        Self::wmi_fallback(&id, &pkg.name, &pkg.aliases, &wmi_programs)
                    }
                    DetectionResult::NotInstalled => {
                        // Detection chain says not installed — try WMI as second opinion
                        Self::wmi_fallback(&id, &pkg.name, &pkg.aliases, &wmi_programs)
                    }
                }
            } else {
                // No detection config — WMI is the only option
                let wmi_result = Self::wmi_fallback(&id, &pkg.name, &pkg.aliases, &wmi_programs);
                if wmi_result.is_installed() {
                    wmi_result
                } else {
                    debug!(package = %id, "no detection config and no WMI match");
                    continue;
                }
            };

            // Cache the result
            self.cache.insert(id.clone(), result.clone());

            results.push(PackageDetection {
                package_id: id,
                result,
            });
        }

        // Sync ledger: update Acknowledged entries
        self.sync_ledger(&results)?;

        let detected_count = results.iter().filter(|r| r.result.is_installed()).count();
        let elapsed = start.elapsed();
        info!(
            detected_count,
            error_count = errors.len(),
            duration_ms = elapsed.as_millis() as u64,
            "catalog scan complete"
        );

        Ok(ScanResult {
            results,
            errors,
            duration: elapsed,
            scanned_at: Utc::now(),
        })
    }

    /// Sync ledger with scan results.
    ///
    /// - New detections → insert Acknowledged entry
    /// - Changed versions → update Acknowledged entry
    /// - Gone detections → remove Acknowledged entry (only Acknowledged source)
    fn sync_ledger(&self, results: &[PackageDetection]) -> Result<(), DetectionError> {
        let existing = self.ledger.list_acknowledged()?;
        let existing_ids: std::collections::HashSet<&str> =
            existing.iter().map(|e| e.package_id.as_str()).collect();

        // Upsert installed packages
        for pd in results {
            match &pd.result {
                DetectionResult::Installed { version, .. } => {
                    self.ledger.upsert_acknowledged(&pd.package_id, version)?;
                }
                DetectionResult::InstalledUnknownVersion { .. } => {
                    let sentinel = Version::parse("0.0.0");
                    self.ledger.upsert_acknowledged(&pd.package_id, &sentinel)?;
                }
                _ => {}
            }
        }

        // Remove Acknowledged entries for packages no longer detected
        let detected_ids: std::collections::HashSet<&str> = results
            .iter()
            .filter(|pd| pd.result.is_installed())
            .map(|pd| pd.package_id.as_str())
            .collect();

        for existing_id in &existing_ids {
            if !detected_ids.contains(existing_id) {
                debug!(package = %existing_id, "removing stale Acknowledged entry");
                self.ledger.remove_acknowledged(existing_id)?;
            }
        }

        Ok(())
    }

    /// Get cached result for a single package.
    pub fn cached(&self, id: &str) -> Option<DetectionResult> {
        self.cache.get(id)
    }

    /// Invalidate cache for a specific package.
    pub fn invalidate(&self, id: &str) {
        self.cache.invalidate(id);
    }

    /// Invalidate entire cache.
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::unwrap_in_result)]
mod tests {
    use super::*;
    use crate::ledger::LedgerSource;
    use crate::types::{Category, DetectionConfig, DetectionMethod, SoftwareType};

    struct MockPackages(Vec<Software>);

    impl PackageSource for MockPackages {
        fn list_all(&self) -> Result<Vec<Software>, DetectionError> {
            Ok(self.0.clone())
        }
    }

    struct MockLedger {
        entries: std::sync::Mutex<Vec<LedgerEntry>>,
    }

    impl MockLedger {
        fn new() -> Self {
            Self {
                entries: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl LedgerStore for MockLedger {
        fn list_acknowledged(&self) -> Result<Vec<LedgerEntry>, DetectionError> {
            Ok(self
                .entries
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.source == LedgerSource::Acknowledged)
                .cloned()
                .collect())
        }

        fn upsert_acknowledged(
            &self,
            package_id: &str,
            version: &Version,
        ) -> Result<(), DetectionError> {
            let mut entries = self.entries.lock().unwrap();
            if let Some(existing) = entries
                .iter_mut()
                .find(|e| e.package_id == package_id && e.source == LedgerSource::Acknowledged)
            {
                existing.version = version.clone();
            } else {
                entries.push(LedgerEntry {
                    package_id: package_id.into(),
                    version: version.clone(),
                    source: LedgerSource::Acknowledged,
                    recorded_at: Utc::now(),
                    notes: None,
                    install_path: None,
                });
            }
            Ok(())
        }

        fn remove_acknowledged(&self, package_id: &str) -> Result<(), DetectionError> {
            let mut entries = self.entries.lock().unwrap();
            entries.retain(|e| {
                !(e.package_id == package_id && e.source == LedgerSource::Acknowledged)
            });
            Ok(())
        }
    }

    fn test_software(id: &str, detection: Option<DetectionConfig>) -> Software {
        Software {
            id: id.parse().unwrap(),
            slug: id.into(),
            name: id.into(),
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
            detection,
            checkver: None,
            install: None,
            dependencies: None,
            versioning: None,
            hardware: None,
            backup: None,
        }
    }

    #[tokio::test]
    async fn scan_with_pe_detection() {
        let packages = MockPackages(vec![test_software(
            "test-app",
            Some(DetectionConfig {
                method: DetectionMethod::PeFile,
                file_path: Some("tests/fixtures/test.exe".into()),
                registry_key: None,
                registry_value: None,
                version_regex: None,
                product_code: None,
                upgrade_code: None,
                inf_provider: None,
                device_class: None,
                inf_name: None,
                fallback: None,
            }),
        )]);

        let ledger = MockLedger::new();
        let scanner = Scanner::new(packages, ledger);
        let result = scanner.scan().await.unwrap();

        assert_eq!(result.results.len(), 1);
        assert!(result.results[0].result.is_installed());
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn scan_skips_no_detection_config() {
        let packages = MockPackages(vec![test_software("no-config", None)]);
        let ledger = MockLedger::new();
        let scanner = Scanner::new(packages, ledger);
        let result = scanner.scan().await.unwrap();

        assert!(result.results.is_empty());
    }

    #[tokio::test]
    async fn scan_uses_cache_on_second_call() {
        let packages = MockPackages(vec![test_software(
            "test-app",
            Some(DetectionConfig {
                method: DetectionMethod::PeFile,
                file_path: Some("tests/fixtures/test.exe".into()),
                registry_key: None,
                registry_value: None,
                version_regex: None,
                product_code: None,
                upgrade_code: None,
                inf_provider: None,
                device_class: None,
                inf_name: None,
                fallback: None,
            }),
        )]);

        let ledger = MockLedger::new();
        let scanner = Scanner::new(packages, ledger);

        // First scan
        let r1 = scanner.scan().await.unwrap();
        assert_eq!(r1.results.len(), 1);

        // Second scan should use cache
        let r2 = scanner.scan().await.unwrap();
        assert_eq!(r2.results.len(), 1);
        assert!(r2.results[0].result.is_installed());
    }

    #[tokio::test]
    async fn ledger_sync_creates_acknowledged_entry() {
        let packages = MockPackages(vec![test_software(
            "test-app",
            Some(DetectionConfig {
                method: DetectionMethod::PeFile,
                file_path: Some("tests/fixtures/test.exe".into()),
                registry_key: None,
                registry_value: None,
                version_regex: None,
                product_code: None,
                upgrade_code: None,
                inf_provider: None,
                device_class: None,
                inf_name: None,
                fallback: None,
            }),
        )]);

        let ledger = MockLedger::new();
        let scanner = Scanner::new(packages, ledger);
        scanner.scan().await.unwrap();

        let entries = scanner.ledger.list_acknowledged().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].package_id, "test-app");
        assert_eq!(entries[0].source, LedgerSource::Acknowledged);
    }

    #[tokio::test]
    async fn ledger_sync_removes_stale_entry() {
        // Pre-populate ledger with a package that won't be detected
        let ledger = MockLedger::new();
        ledger
            .upsert_acknowledged("gone-app", &Version::parse("1.0.0"))
            .unwrap();

        // Scan with different package
        let packages = MockPackages(vec![test_software(
            "test-app",
            Some(DetectionConfig {
                method: DetectionMethod::PeFile,
                file_path: Some("tests/fixtures/test.exe".into()),
                registry_key: None,
                registry_value: None,
                version_regex: None,
                product_code: None,
                upgrade_code: None,
                inf_provider: None,
                device_class: None,
                inf_name: None,
                fallback: None,
            }),
        )]);

        let scanner = Scanner::new(packages, ledger);
        scanner.scan().await.unwrap();

        let entries = scanner.ledger.list_acknowledged().unwrap();
        // "gone-app" should be removed, "test-app" should be added
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].package_id, "test-app");
    }

    #[test]
    fn ledger_sync_creates_entry_for_unknown_version() {
        let ledger = MockLedger::new();
        let packages = MockPackages(vec![]);
        let scanner = Scanner::new(packages, ledger);

        let results = vec![PackageDetection {
            package_id: "astap".into(),
            result: DetectionResult::InstalledUnknownVersion {
                method: DetectionMethod::Registry,
                install_path: Some("C:\\Program Files\\ASTAP".into()),
            },
        }];

        scanner.sync_ledger(&results).unwrap();

        let entries = scanner.ledger.list_acknowledged().unwrap();
        assert_eq!(
            entries.len(),
            1,
            "InstalledUnknownVersion must create a ledger entry"
        );
        assert_eq!(entries[0].package_id, "astap");
        assert_eq!(
            entries[0].version.raw, "0.0.0",
            "sentinel version for unknown"
        );
    }

    #[test]
    fn ledger_sync_removes_stale_when_unknown_version_present() {
        let ledger = MockLedger::new();
        // Pre-populate with a package that will no longer be detected
        ledger
            .upsert_acknowledged("gone-app", &Version::parse("1.0.0"))
            .unwrap();

        let packages = MockPackages(vec![]);
        let scanner = Scanner::new(packages, ledger);

        // Only "astap" is detected (with unknown version)
        let results = vec![PackageDetection {
            package_id: "astap".into(),
            result: DetectionResult::InstalledUnknownVersion {
                method: DetectionMethod::Registry,
                install_path: None,
            },
        }];

        scanner.sync_ledger(&results).unwrap();

        let entries = scanner.ledger.list_acknowledged().unwrap();
        assert_eq!(entries.len(), 1, "stale entry should be removed");
        assert_eq!(entries[0].package_id, "astap");
    }

    #[tokio::test]
    async fn ledger_sync_preserves_astroup_entries() {
        let ledger = MockLedger::new();
        // Pre-populate with an AstroUp-sourced entry (user installed it)
        {
            let mut entries = ledger.entries.lock().unwrap();
            entries.push(LedgerEntry {
                package_id: "user-installed-app".into(),
                version: Version::parse("2.0.0"),
                source: LedgerSource::AstroUp,
                recorded_at: Utc::now(),
                notes: None,
                install_path: None,
            });
        }

        // Scan finds nothing (no packages in catalog)
        let packages = MockPackages(vec![]);
        let scanner = Scanner::new(packages, ledger);
        scanner.scan().await.unwrap();

        // AstroUp entry must NOT be removed (only Acknowledged entries are auto-removed)
        let all_entries = scanner.ledger.entries.lock().unwrap();
        assert_eq!(all_entries.len(), 1);
        assert_eq!(all_entries[0].package_id, "user-installed-app");
        assert_eq!(all_entries[0].source, LedgerSource::AstroUp);
    }
}
