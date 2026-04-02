//! Unit tests for policy enforcement logic.

use astro_up_core::engine::planner::SkipReason;
use astro_up_core::engine::policy::apply_policy;
use astro_up_core::engine::version_cmp::{PackageStatus, VersionFormat};
use astro_up_core::types::{PolicyLevel, Version};

fn minor_update() -> PackageStatus {
    PackageStatus::UpdateAvailable {
        current: Version::parse("1.0.0"),
        available: Version::parse("1.5.0"),
    }
}

fn major_update() -> PackageStatus {
    PackageStatus::MajorUpgradeAvailable {
        current: Version::parse("1.0.0"),
        available: Version::parse("2.0.0"),
    }
}

#[test]
fn minor_only_blocks_major_semver() {
    assert_eq!(
        apply_policy(
            &major_update(),
            &PolicyLevel::Minor,
            false,
            &VersionFormat::Semver
        ),
        Some(SkipReason::PolicyBlocked {
            policy: PolicyLevel::Minor
        })
    );
}

#[test]
fn minor_only_allows_date_custom() {
    assert!(
        apply_policy(
            &major_update(),
            &PolicyLevel::Minor,
            false,
            &VersionFormat::Date
        )
        .is_none()
    );
    let custom = VersionFormat::Custom {
        pattern: r"(\d+)".to_string(),
    };
    assert!(apply_policy(&major_update(), &PolicyLevel::Minor, false, &custom).is_none());
}

#[test]
fn allow_major_overrides_minor_policy() {
    assert!(
        apply_policy(
            &major_update(),
            &PolicyLevel::Minor,
            true,
            &VersionFormat::Semver
        )
        .is_none()
    );
}

#[test]
fn manual_blocks_all() {
    assert_eq!(
        apply_policy(
            &minor_update(),
            &PolicyLevel::Manual,
            false,
            &VersionFormat::Semver
        ),
        Some(SkipReason::ManualOnly)
    );
}

#[test]
fn none_blocks_all() {
    assert_eq!(
        apply_policy(
            &minor_update(),
            &PolicyLevel::None,
            false,
            &VersionFormat::Semver
        ),
        Some(SkipReason::Disabled)
    );
}

#[test]
fn major_policy_allows_everything() {
    assert!(
        apply_policy(
            &major_update(),
            &PolicyLevel::Major,
            false,
            &VersionFormat::Semver
        )
        .is_none()
    );
    assert!(
        apply_policy(
            &minor_update(),
            &PolicyLevel::Major,
            false,
            &VersionFormat::Semver
        )
        .is_none()
    );
}

#[test]
fn minor_policy_allows_minor_update() {
    assert!(
        apply_policy(
            &minor_update(),
            &PolicyLevel::Minor,
            false,
            &VersionFormat::Semver
        )
        .is_none()
    );
}
