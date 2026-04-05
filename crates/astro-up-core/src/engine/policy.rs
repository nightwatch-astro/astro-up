//! Policy enforcement — minor/major filtering and per-package overrides.

use crate::engine::planner::SkipReason;
use crate::engine::version_cmp::{PackageStatus, VersionFormat};
use crate::types::PolicyLevel;

/// Apply update policy to a package status. Returns `None` if the update is
/// allowed, or `Some(SkipReason)` if blocked by policy.
///
/// Policy rules:
/// - `PolicyLevel::None` → block all updates
/// - `PolicyLevel::Manual` → block unless explicitly requested
/// - `PolicyLevel::Minor` → allow minor, block major (semver only; date/custom always allowed)
/// - `PolicyLevel::Major` → allow all updates
///
/// The `allow_major` flag (from `--allow-major` CLI) overrides `Minor` policy for this invocation.
pub fn apply_policy(
    status: &PackageStatus,
    policy: &PolicyLevel,
    allow_major: bool,
    format: &VersionFormat,
) -> Option<SkipReason> {
    match policy {
        PolicyLevel::None => Some(SkipReason::Disabled),
        PolicyLevel::Manual => Some(SkipReason::ManualOnly),
        PolicyLevel::Minor => {
            // Minor-only: block major upgrades for semver packages only
            if matches!(status, PackageStatus::MajorUpgradeAvailable { .. }) {
                // Major/minor distinction only applies to semver
                if matches!(format, VersionFormat::Semver) && !allow_major {
                    return Some(SkipReason::PolicyBlocked {
                        policy: PolicyLevel::Minor,
                    });
                }
            }
            None
        }
        PolicyLevel::Major => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::Version;

    fn update_available() -> PackageStatus {
        PackageStatus::UpdateAvailable {
            current: Version::parse("1.0.0"),
            available: Version::parse("1.5.0"),
        }
    }

    fn major_upgrade() -> PackageStatus {
        PackageStatus::MajorUpgradeAvailable {
            current: Version::parse("1.0.0"),
            available: Version::parse("2.0.0"),
        }
    }

    #[test]
    fn none_policy_blocks_all() {
        assert!(
            apply_policy(
                &update_available(),
                &PolicyLevel::None,
                false,
                &VersionFormat::Semver
            )
            .is_some()
        );
        assert_eq!(
            apply_policy(
                &update_available(),
                &PolicyLevel::None,
                false,
                &VersionFormat::Semver
            ),
            Some(SkipReason::Disabled)
        );
    }

    #[test]
    fn manual_policy_blocks_all() {
        assert_eq!(
            apply_policy(
                &update_available(),
                &PolicyLevel::Manual,
                false,
                &VersionFormat::Semver
            ),
            Some(SkipReason::ManualOnly)
        );
    }

    #[test]
    fn minor_policy_allows_minor_update() {
        assert!(
            apply_policy(
                &update_available(),
                &PolicyLevel::Minor,
                false,
                &VersionFormat::Semver
            )
            .is_none()
        );
    }

    #[test]
    fn minor_policy_blocks_major_semver() {
        assert_eq!(
            apply_policy(
                &major_upgrade(),
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
    fn minor_policy_allows_major_with_flag() {
        assert!(
            apply_policy(
                &major_upgrade(),
                &PolicyLevel::Minor,
                true,
                &VersionFormat::Semver
            )
            .is_none()
        );
    }

    #[test]
    fn minor_policy_allows_date_format_always() {
        // Date format: major/minor distinction doesn't apply
        assert!(
            apply_policy(
                &major_upgrade(),
                &PolicyLevel::Minor,
                false,
                &VersionFormat::Date
            )
            .is_none()
        );
    }

    #[test]
    fn minor_policy_allows_custom_format_always() {
        let custom = VersionFormat::Custom {
            pattern: r"(\d+)".to_string(),
        };
        assert!(apply_policy(&major_upgrade(), &PolicyLevel::Minor, false, &custom).is_none());
    }

    #[test]
    fn major_policy_allows_everything() {
        assert!(
            apply_policy(
                &major_upgrade(),
                &PolicyLevel::Major,
                false,
                &VersionFormat::Semver
            )
            .is_none()
        );
        assert!(
            apply_policy(
                &update_available(),
                &PolicyLevel::Major,
                false,
                &VersionFormat::Semver
            )
            .is_none()
        );
    }
}
