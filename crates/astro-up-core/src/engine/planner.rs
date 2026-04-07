//! Update planner — version comparison, dependency resolution, and plan building.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::catalog::PackageId;
use crate::error::CoreError;
use crate::types::{PolicyLevel, Software, Version};

use super::version_cmp::{PackageStatus, VersionFormat};

// ---------------------------------------------------------------------------
// PackageState (forward-declared stub — will move to a shared location once
// the detection/ledger modules define a canonical version)
// ---------------------------------------------------------------------------

/// Current state of a package in the user's environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageState {
    /// Package is installed and managed.
    Installed,
    /// Package was detected but not yet acknowledged by the user.
    Detected,
    /// Package is disabled (opted-out of updates).
    Disabled,
    /// Package is not present on the system.
    NotInstalled,
}

// ---------------------------------------------------------------------------
// SkipReason
// ---------------------------------------------------------------------------

/// Why a package was excluded from the update plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SkipReason {
    /// The installed version is already at or above the catalog target.
    UpToDate,
    /// The installed version is newer than anything in the catalog.
    NewerThanCatalog,
    /// A policy rule prevented the update.
    PolicyBlocked {
        /// The policy level that blocked the update.
        policy: PolicyLevel,
    },
    /// The package is configured for manual-only updates.
    ManualOnly,
    /// The package is disabled.
    Disabled,
    /// A dependency of this package failed or was skipped.
    DependencyFailed {
        /// The dependency that caused the skip.
        dep_id: PackageId,
    },
    /// The installed version is newer than catalog and `--allow-downgrade` was not set.
    DowngradeBlocked,
}

// ---------------------------------------------------------------------------
// SkippedPackage
// ---------------------------------------------------------------------------

/// A package that was evaluated but excluded from the update plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedPackage {
    /// Identifier of the skipped package.
    pub package_id: PackageId,
    /// Why the package was skipped.
    pub reason: SkipReason,
    /// Current state of the package at plan time.
    pub state: PackageState,
}

// ---------------------------------------------------------------------------
// PlannedUpdate
// ---------------------------------------------------------------------------

/// A single package update that the engine intends to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedUpdate {
    /// Identifier of the package to update.
    pub package_id: PackageId,
    /// Full software definition from the catalog.
    pub software: Software,
    /// Currently installed version.
    pub current_version: Version,
    /// Version the engine will update to.
    pub target_version: Version,
    /// Catalog entry for the target version (download URLs, hashes, etc.).
    pub version_entry: crate::catalog::VersionEntry,
    /// How the version string should be parsed and compared.
    pub version_format: VersionFormat,
    /// Whether a backup configuration exists for this package.
    pub has_backup_config: bool,
    /// Other packages that must be updated before this one.
    pub dependencies: Vec<PackageId>,
}

// ---------------------------------------------------------------------------
// UpdatePlan
// ---------------------------------------------------------------------------

/// The complete update plan produced by the planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlan {
    /// Packages that will be updated, in dependency order.
    pub items: Vec<PlannedUpdate>,
    /// Packages that were evaluated but excluded.
    pub skipped: Vec<SkippedPackage>,
    /// Non-fatal warnings encountered during planning.
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// CatalogEntry — input data for the planner
// ---------------------------------------------------------------------------

/// A single package from the catalog with its installed version, used as input
/// to the planner.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    /// Full software definition from the catalog.
    pub software: Software,
    /// Currently installed version, if any.
    pub installed_version: Option<Version>,
    /// Latest version available in the catalog.
    pub catalog_version: Version,
    /// Catalog entry for the target version.
    pub version_entry: crate::catalog::VersionEntry,
    /// How the version string should be parsed and compared.
    pub version_format: VersionFormat,
    /// Update policy for this package (default: Major = allow all).
    pub policy: crate::types::PolicyLevel,
}

// ---------------------------------------------------------------------------
// UpdatePlanner
// ---------------------------------------------------------------------------

/// Builds an [`UpdatePlan`] from catalog data and installed versions.
///
/// The planner compares installed versions against catalog versions, resolves
/// transitive dependencies, and produces a topologically sorted plan.
pub struct UpdatePlanner {
    entries: Vec<CatalogEntry>,
    allow_major: bool,
    allow_downgrade: bool,
}

impl UpdatePlanner {
    /// Create a new planner with the given catalog entries.
    pub fn new(entries: Vec<CatalogEntry>) -> Self {
        Self {
            entries,
            allow_major: false,
            allow_downgrade: false,
        }
    }

    /// Set the `--allow-major` override for this planning session.
    pub fn with_allow_major(mut self, allow: bool) -> Self {
        self.allow_major = allow;
        self
    }

    /// Set the `--allow-downgrade` override for this planning session.
    pub fn with_allow_downgrade(mut self, allow: bool) -> Self {
        self.allow_downgrade = allow;
        self
    }

    /// Build an update plan for all packages with available updates.
    pub fn plan_all(&self) -> Result<UpdatePlan, CoreError> {
        let mut items = Vec::new();
        let mut skipped = Vec::new();
        let mut warnings = Vec::new();

        for entry in &self.entries {
            let status = PackageStatus::determine(
                entry.installed_version.as_ref(),
                Some(&entry.catalog_version),
                &entry.version_format,
            );

            // Apply policy enforcement (FR-003)
            if let Some(skip_reason) = super::policy::apply_policy(
                &status,
                &entry.policy,
                self.allow_major,
                &entry.version_format,
            ) {
                skipped.push(SkippedPackage {
                    package_id: entry.software.id.clone(),
                    reason: skip_reason,
                    state: PackageState::Installed,
                });
                continue;
            }

            match &status {
                PackageStatus::UpdateAvailable { current, available }
                | PackageStatus::MajorUpgradeAvailable { current, available } => {
                    let deps = entry
                        .software
                        .dependencies
                        .as_ref()
                        .map(|d| {
                            d.requires
                                .iter()
                                .filter_map(|dep| PackageId::new(&dep.id).ok())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    items.push(PlannedUpdate {
                        package_id: entry.software.id.clone(),
                        software: entry.software.clone(),
                        current_version: current.clone(),
                        target_version: available.clone(),
                        version_entry: entry.version_entry.clone(),
                        version_format: entry.version_format.clone(),
                        has_backup_config: entry.software.backup.is_some(),
                        dependencies: deps,
                    });
                }
                PackageStatus::UpToDate => {
                    skipped.push(SkippedPackage {
                        package_id: entry.software.id.clone(),
                        reason: SkipReason::UpToDate,
                        state: if entry.installed_version.is_some() {
                            PackageState::Installed
                        } else {
                            PackageState::NotInstalled
                        },
                    });
                }
                PackageStatus::NewerThanCatalog {
                    current,
                    catalog_latest,
                } => {
                    if self.allow_downgrade {
                        // FR-012: allow downgrade when explicitly requested
                        items.push(PlannedUpdate {
                            package_id: entry.software.id.clone(),
                            software: entry.software.clone(),
                            current_version: current.clone(),
                            target_version: catalog_latest.clone(),
                            version_entry: entry.version_entry.clone(),
                            version_format: entry.version_format.clone(),
                            has_backup_config: entry.software.backup.is_some(),
                            dependencies: Vec::new(),
                        });
                    } else {
                        warnings.push(format!(
                            "{}: installed {} is newer than catalog {} (use --allow-downgrade to override)",
                            entry.software.id, current.raw, catalog_latest.raw
                        ));
                        skipped.push(SkippedPackage {
                            package_id: entry.software.id.clone(),
                            reason: SkipReason::DowngradeBlocked,
                            state: PackageState::Installed,
                        });
                    }
                }
                PackageStatus::NotInstalled => {
                    // Fresh install: plan as update from 0.0.0 to catalog version
                    let deps = entry
                        .software
                        .dependencies
                        .as_ref()
                        .map(|d| {
                            d.requires
                                .iter()
                                .filter_map(|dep| PackageId::new(&dep.id).ok())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    items.push(PlannedUpdate {
                        package_id: entry.software.id.clone(),
                        software: entry.software.clone(),
                        current_version: Version::parse("0.0.0"),
                        target_version: entry.catalog_version.clone(),
                        version_entry: entry.version_entry.clone(),
                        version_format: entry.version_format.clone(),
                        has_backup_config: false,
                        dependencies: deps,
                    });
                }
                PackageStatus::Unknown => {
                    // Unknown state — skip
                }
            }
        }

        // Dependency satisfaction check: verify all referenced deps exist
        // in either the plan items or the catalog entries (even if up-to-date).
        let known_ids: std::collections::HashSet<&crate::catalog::PackageId> =
            self.entries.iter().map(|e| &e.software.id).collect();
        for item in &items {
            for dep_id in &item.dependencies {
                if !known_ids.contains(dep_id) {
                    return Err(CoreError::MissingDependency {
                        dep_id: format!(
                            "{} (required by {})",
                            dep_id.as_ref(),
                            item.package_id.as_ref()
                        ),
                    });
                }
            }
        }

        let items = topological_sort(items)?;

        Ok(UpdatePlan {
            items,
            skipped,
            warnings,
        })
    }

    /// Build an update plan for specific packages only.
    pub fn plan_specific(&self, package_ids: &[PackageId]) -> Result<UpdatePlan, CoreError> {
        let full_plan = self.plan_all()?;

        let update_map: HashMap<&PackageId, &PlannedUpdate> =
            full_plan.items.iter().map(|u| (&u.package_id, u)).collect();

        let mut included: HashSet<PackageId> = HashSet::new();
        let mut queue: VecDeque<PackageId> = package_ids.iter().cloned().collect();

        while let Some(pkg_id) = queue.pop_front() {
            if !included.insert(pkg_id.clone()) {
                continue;
            }
            if let Some(update) = update_map.get(&pkg_id) {
                for dep_id in &update.dependencies {
                    if !included.contains(dep_id) {
                        queue.push_back(dep_id.clone());
                    }
                }
            }
        }

        let items: Vec<PlannedUpdate> = full_plan
            .items
            .into_iter()
            .filter(|u| included.contains(&u.package_id))
            .collect();

        let requested: HashSet<&PackageId> = package_ids.iter().collect();
        let skipped: Vec<SkippedPackage> = full_plan
            .skipped
            .into_iter()
            .filter(|s| requested.contains(&s.package_id))
            .collect();

        let warnings: Vec<String> = full_plan
            .warnings
            .into_iter()
            .filter(|w| included.iter().any(|id| w.contains(id.as_ref())))
            .collect();

        Ok(UpdatePlan {
            items,
            skipped,
            warnings,
        })
    }
}

// ---------------------------------------------------------------------------
// topological_sort — Kahn's algorithm
// ---------------------------------------------------------------------------

/// Sort planned updates in dependency order using Kahn's algorithm.
pub fn topological_sort(updates: Vec<PlannedUpdate>) -> Result<Vec<PlannedUpdate>, CoreError> {
    if updates.is_empty() {
        return Ok(updates);
    }

    let id_to_idx: HashMap<&PackageId, usize> = updates
        .iter()
        .enumerate()
        .map(|(i, u)| (&u.package_id, i))
        .collect();

    let n = updates.len();
    let mut in_degree = vec![0usize; n];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];

    for (idx, update) in updates.iter().enumerate() {
        for dep_id in &update.dependencies {
            if let Some(&dep_idx) = id_to_idx.get(dep_id) {
                dependents[dep_idx].push(idx);
                in_degree[idx] += 1;
            }
        }
    }

    let edge_count: usize = dependents.iter().map(|d| d.len()).sum();
    tracing::debug!(
        nodes = n,
        edges = edge_count,
        "dependency graph built for topological sort"
    );

    let mut queue: VecDeque<usize> = in_degree
        .iter()
        .enumerate()
        .filter(|&(_, &d)| d == 0)
        .map(|(i, _)| i)
        .collect();

    let mut order: Vec<usize> = Vec::with_capacity(n);

    while let Some(idx) = queue.pop_front() {
        order.push(idx);
        for &dep_idx in &dependents[idx] {
            in_degree[dep_idx] -= 1;
            if in_degree[dep_idx] == 0 {
                queue.push_back(dep_idx);
            }
        }
    }

    if order.len() != n {
        let in_cycle: Vec<String> = in_degree
            .iter()
            .enumerate()
            .filter(|&(_, &d)| d > 0)
            .map(|(i, _)| updates[i].package_id.as_ref().to_string())
            .collect();
        return Err(CoreError::DependencyCycle { path: in_cycle });
    }

    let mut indexed: Vec<Option<PlannedUpdate>> = updates.into_iter().map(Some).collect();
    #[allow(clippy::expect_used)] // topological sort guarantees each index is used once
    let sorted = order
        .into_iter()
        .map(|i| indexed[i].take().expect("each index used exactly once"))
        .collect();

    Ok(sorted)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use chrono::Utc;

    use crate::catalog::{PackageId, VersionEntry};
    use crate::types::{Category, Dependency, DependencyConfig, Software, SoftwareType, Version};

    use super::*;

    fn make_software(id: &str, deps: Vec<&str>) -> Software {
        let dep_config = if deps.is_empty() {
            None
        } else {
            Some(DependencyConfig {
                requires: deps
                    .into_iter()
                    .map(|d| Dependency {
                        id: d.to_string(),
                        min_version: None,
                    })
                    .collect(),
                optional: Vec::new(),
            })
        };
        Software {
            id: PackageId::new(id).unwrap(),
            slug: String::new(),
            name: id.to_string(),
            software_type: SoftwareType::Application,
            category: Category::Capture,
            os: Vec::new(),
            description: None,
            homepage: None,
            publisher: None,
            icon_url: None,
            license: None,
            license_url: None,
            aliases: Vec::new(),
            tags: Vec::new(),
            notes: None,
            docs_url: None,
            channel: None,
            min_os_version: None,
            manifest_version: None,
            detection: None,
            install: None,
            checkver: None,
            dependencies: dep_config,
            hardware: None,
            backup: None,
            versioning: None,
        }
    }

    fn make_entry(id: &str, installed: &str, catalog: &str, deps: Vec<&str>) -> CatalogEntry {
        let pkg_id = PackageId::new(id).unwrap();
        CatalogEntry {
            software: make_software(id, deps),
            installed_version: Some(Version::parse(installed)),
            catalog_version: Version::parse(catalog),
            version_entry: VersionEntry {
                package_id: pkg_id,
                version: catalog.to_string(),
                url: format!("https://example.com/{id}/{catalog}"),
                sha256: None,
                discovered_at: Utc::now(),
                release_notes_url: None,
                pre_release: false,
            },
            version_format: VersionFormat::Semver,
            policy: crate::types::PolicyLevel::Major,
        }
    }

    #[test]
    fn topo_sort_empty() {
        let result = topological_sort(Vec::new()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn topo_sort_single_item() {
        let entries = vec![make_entry("nina", "1.0.0", "2.0.0", vec![])];
        let planner = UpdatePlanner::new(entries);
        let plan = planner.plan_all().unwrap();
        assert_eq!(plan.items.len(), 1);
    }

    #[test]
    fn topo_sort_respects_dependency_order() {
        let entries = vec![
            make_entry("nina", "1.0.0", "2.0.0", vec!["ascom-platform"]),
            make_entry("ascom-platform", "6.0.0", "7.0.0", vec![]),
        ];
        let planner = UpdatePlanner::new(entries);
        let plan = planner.plan_all().unwrap();
        assert_eq!(plan.items.len(), 2);
        assert_eq!(
            plan.items[0].package_id,
            PackageId::new("ascom-platform").unwrap()
        );
        assert_eq!(plan.items[1].package_id, PackageId::new("nina").unwrap());
    }

    #[test]
    fn topo_sort_detects_cycle() {
        let entries = vec![
            make_entry("aa", "1.0.0", "2.0.0", vec!["bb"]),
            make_entry("bb", "1.0.0", "2.0.0", vec!["aa"]),
        ];
        let planner = UpdatePlanner::new(entries);
        let result = planner.plan_all();
        assert!(matches!(result, Err(CoreError::DependencyCycle { .. })));
    }

    #[test]
    fn plan_all_skips_up_to_date() {
        let entries = vec![
            make_entry("nina", "2.0.0", "2.0.0", vec![]),
            make_entry("phd2", "1.0.0", "2.0.0", vec![]),
        ];
        let planner = UpdatePlanner::new(entries);
        let plan = planner.plan_all().unwrap();
        assert_eq!(plan.items.len(), 1);
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].reason, SkipReason::UpToDate);
    }

    #[test]
    fn plan_all_warns_newer_than_catalog() {
        let entries = vec![make_entry("nina", "3.0.0", "2.0.0", vec![])];
        let planner = UpdatePlanner::new(entries);
        let plan = planner.plan_all().unwrap();
        assert!(plan.items.is_empty());
        assert_eq!(plan.warnings.len(), 1);
        assert!(plan.warnings[0].contains("newer"));
    }

    #[test]
    fn plan_specific_filters_to_requested() {
        let entries = vec![
            make_entry("nina", "1.0.0", "2.0.0", vec![]),
            make_entry("phd2", "1.0.0", "2.0.0", vec![]),
        ];
        let planner = UpdatePlanner::new(entries);
        let plan = planner
            .plan_specific(&[PackageId::new("nina").unwrap()])
            .unwrap();
        assert_eq!(plan.items.len(), 1);
        assert_eq!(plan.items[0].package_id, PackageId::new("nina").unwrap());
    }

    #[test]
    fn plan_specific_includes_transitive_deps() {
        let entries = vec![
            make_entry("nina", "1.0.0", "2.0.0", vec!["ascom-platform"]),
            make_entry("ascom-platform", "6.0.0", "7.0.0", vec!["dotnet-runtime"]),
            make_entry("dotnet-runtime", "6.0.0", "8.0.0", vec![]),
            make_entry("phd2", "1.0.0", "2.0.0", vec![]),
        ];
        let planner = UpdatePlanner::new(entries);
        let plan = planner
            .plan_specific(&[PackageId::new("nina").unwrap()])
            .unwrap();
        assert_eq!(plan.items.len(), 3);
        let ids: Vec<String> = plan
            .items
            .iter()
            .map(|u| u.package_id.as_ref().to_string())
            .collect();
        assert!(ids.contains(&"dotnet-runtime".to_string()));
        assert!(!ids.contains(&"phd2".to_string()));
    }

    #[test]
    fn plan_specific_empty_ids_returns_empty() {
        let entries = vec![make_entry("nina", "1.0.0", "2.0.0", vec![])];
        let planner = UpdatePlanner::new(entries);
        let plan = planner.plan_specific(&[]).unwrap();
        assert!(plan.items.is_empty());
    }
}
