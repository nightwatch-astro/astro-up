#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for the update planner — dependency resolution, ordering, and plan building.

use astro_up_core::catalog::{PackageId, VersionEntry};
use astro_up_core::engine::planner::{CatalogEntry, SkipReason, UpdatePlanner};
use astro_up_core::engine::version_cmp::VersionFormat;
use astro_up_core::error::CoreError;
use astro_up_core::types::{
    Category, Dependency, DependencyConfig, Software, SoftwareType, Version,
};
use chrono::Utc;

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
    CatalogEntry {
        software: make_software(id, deps),
        installed_version: Some(Version::parse(installed)),
        catalog_version: Version::parse(catalog),
        version_entry: VersionEntry {
            package_id: PackageId::new(id).unwrap(),
            version: catalog.to_string(),
            url: format!("https://example.com/{id}/{catalog}"),
            sha256: None,
            discovered_at: Utc::now(),
            release_notes_url: None,
            pre_release: false,
            assets: Vec::new(),
        },
        version_format: VersionFormat::Semver,
        policy: astro_up_core::types::PolicyLevel::Major,
    }
}

/// 5 packages, 2 with dependencies — verify topological order.
#[test]
fn five_packages_with_dependencies_topo_order() {
    let entries = vec![
        make_entry("nina", "1.0.0", "2.0.0", vec!["ascom-platform"]),
        make_entry("phd2", "2.0.0", "3.0.0", vec!["ascom-platform"]),
        make_entry("ascom-platform", "6.0.0", "7.0.0", vec![]),
        make_entry("sharpcap", "4.0.0", "4.1.0", vec![]),
        make_entry("stellarium", "1.0.0", "1.2.0", vec![]),
    ];
    let planner = UpdatePlanner::new(entries);
    let plan = planner.plan_all().unwrap();

    assert_eq!(plan.items.len(), 5);

    let ids: Vec<String> = plan
        .items
        .iter()
        .map(|u| u.package_id.as_ref().to_string())
        .collect();

    // ascom-platform must come before nina and phd2
    let ascom_pos = ids.iter().position(|s| s == "ascom-platform").unwrap();
    let nina_pos = ids.iter().position(|s| s == "nina").unwrap();
    let phd2_pos = ids.iter().position(|s| s == "phd2").unwrap();
    assert!(ascom_pos < nina_pos, "ascom-platform must come before nina");
    assert!(ascom_pos < phd2_pos, "ascom-platform must come before phd2");
}

/// Cycle detection produces DependencyCycle error.
#[test]
fn cycle_detection() {
    let entries = vec![
        make_entry("pkg-a", "1.0.0", "2.0.0", vec!["pkg-b"]),
        make_entry("pkg-b", "1.0.0", "2.0.0", vec!["pkg-c"]),
        make_entry("pkg-c", "1.0.0", "2.0.0", vec!["pkg-a"]),
    ];
    let planner = UpdatePlanner::new(entries);
    let result = planner.plan_all();

    match result {
        Err(CoreError::DependencyCycle { path }) => {
            assert_eq!(path.len(), 3, "cycle should contain all 3 packages");
        }
        other => panic!("expected DependencyCycle, got: {other:?}"),
    }
}

/// Mix of up-to-date and updateable packages.
#[test]
fn mixed_up_to_date_and_updateable() {
    let entries = vec![
        make_entry("nina", "2.0.0", "2.0.0", vec![]), // up to date
        make_entry("phd2", "1.0.0", "2.0.0", vec![]), // has update
        make_entry("sharpcap", "3.0.0", "2.0.0", vec![]), // newer than catalog
    ];
    let planner = UpdatePlanner::new(entries);
    let plan = planner.plan_all().unwrap();

    assert_eq!(plan.items.len(), 1, "only phd2 needs update");
    assert_eq!(plan.items[0].package_id, PackageId::new("phd2").unwrap());
    assert_eq!(plan.skipped.len(), 2);

    let skip_reasons: Vec<_> = plan.skipped.iter().map(|s| &s.reason).collect();
    assert!(skip_reasons.contains(&&SkipReason::UpToDate));
    assert!(skip_reasons.contains(&&SkipReason::DowngradeBlocked));
    assert_eq!(plan.warnings.len(), 1, "newer-than-catalog warning");
}

/// plan_specific resolves transitive dependencies.
#[test]
fn plan_specific_transitive_deps() {
    let entries = vec![
        make_entry("nina", "1.0.0", "2.0.0", vec!["ascom-platform"]),
        make_entry("ascom-platform", "6.0.0", "7.0.0", vec!["dotnet"]),
        make_entry("dotnet", "6.0.0", "8.0.0", vec![]),
        make_entry("phd2", "1.0.0", "2.0.0", vec![]),
    ];
    let planner = UpdatePlanner::new(entries);
    let plan = planner
        .plan_specific(&[PackageId::new("nina").unwrap()])
        .unwrap();

    // nina + ascom-platform + dotnet, but NOT phd2
    assert_eq!(plan.items.len(), 3);
    let ids: Vec<String> = plan
        .items
        .iter()
        .map(|u| u.package_id.as_ref().to_string())
        .collect();
    assert!(!ids.contains(&"phd2".to_string()));

    // Order: dotnet < ascom-platform < nina
    let dotnet_pos = ids.iter().position(|s| s == "dotnet").unwrap();
    let ascom_pos = ids.iter().position(|s| s == "ascom-platform").unwrap();
    let nina_pos = ids.iter().position(|s| s == "nina").unwrap();
    assert!(dotnet_pos < ascom_pos);
    assert!(ascom_pos < nina_pos);
}

/// Missing dependency not in catalog → MissingDependency error.
#[test]
fn missing_dependency_errors() {
    // nina depends on "unknown-dep" which is not in the catalog
    let entries = vec![make_entry("nina", "1.0.0", "2.0.0", vec!["unknown-dep"])];
    let planner = UpdatePlanner::new(entries);
    let result = planner.plan_all();
    assert!(
        matches!(
            result,
            Err(astro_up_core::error::CoreError::MissingDependency { .. })
        ),
        "expected MissingDependency, got: {result:?}"
    );
}

/// Satisfied dependency (up-to-date) proceeds without error.
#[test]
fn satisfied_dependency_proceeds() {
    // nina depends on ascom-platform, which is up-to-date
    let entries = vec![
        make_entry("nina", "1.0.0", "2.0.0", vec!["ascom-platform"]),
        make_entry("ascom-platform", "7.0.0", "7.0.0", vec![]),
    ];
    let planner = UpdatePlanner::new(entries);
    let plan = planner.plan_all().unwrap();
    // Only nina needs update (ascom-platform is skipped as up-to-date)
    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.items[0].package_id, PackageId::new("nina").unwrap());
}
