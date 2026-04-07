use std::time::Instant;

use chrono::Utc;
use tracing::{debug, info};

use crate::detect::{
    DetectionCache, DetectionError, DetectionResult, PackageDetection, PathResolver, ScanResult,
    run_chain,
};
use crate::ledger::LedgerEntry;
use crate::types::{Software, Version};

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

    /// Run a full scan across all catalog packages.
    #[tracing::instrument(skip_all)]
    pub async fn scan(&self) -> Result<ScanResult, DetectionError> {
        let start = Instant::now();
        let packages = self.packages.list_all()?;
        info!(package_count = packages.len(), "starting catalog scan");

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
            let Some(ref detection_config) = pkg.detection else {
                debug!(package = %pkg.id, "no detection config, skipping");
                continue;
            };

            // Check cache first
            let id = pkg.id.to_string();

            // Check cache first
            if let Some(cached) = self.cache.get(&id) {
                results.push(PackageDetection {
                    package_id: id,
                    result: cached,
                });
                continue;
            }

            let ledger_path = ledger_paths.get(&id).map(String::as_str);
            let result = run_chain(detection_config, &self.resolver, ledger_path).await;

            // Report per-package errors for Unavailable results
            if let DetectionResult::Unavailable { ref reason } = result {
                errors.push(crate::detect::ScanError {
                    package_id: id.clone(),
                    method: detection_config.method.clone(),
                    error: reason.clone(),
                });
            }

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
            if let DetectionResult::Installed { ref version, .. } = pd.result {
                self.ledger.upsert_acknowledged(&pd.package_id, version)?;
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
