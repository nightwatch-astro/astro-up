//! Version format handling — semver, date, and custom regex parsers.

use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::types::version::{try_parse_lenient, Version};

/// Status of a package relative to the catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum PackageStatus {
    /// Installed version matches catalog latest.
    UpToDate,
    /// A newer version is available (minor/patch update).
    UpdateAvailable {
        current: Version,
        available: Version,
    },
    /// A newer major version is available (breaking change potential).
    MajorUpgradeAvailable {
        current: Version,
        available: Version,
    },
    /// Installed version is newer than the catalog's latest.
    NewerThanCatalog {
        current: Version,
        catalog_latest: Version,
    },
    /// Package is not installed.
    NotInstalled,
    /// Status cannot be determined.
    Unknown,
}

impl PackageStatus {
    /// Determine the status of a package by comparing installed and catalog versions.
    ///
    /// - If `installed` is `None`, returns [`PackageStatus::NotInstalled`].
    /// - If `catalog_latest` is `None`, returns [`PackageStatus::Unknown`].
    /// - Equal versions return [`PackageStatus::UpToDate`].
    /// - Installed newer than catalog returns [`PackageStatus::NewerThanCatalog`].
    /// - Catalog newer than installed returns [`PackageStatus::UpdateAvailable`]
    ///   (major upgrade distinction is deferred to T027).
    pub fn determine(
        installed: Option<&Version>,
        catalog_latest: Option<&Version>,
        format: &VersionFormat,
    ) -> Self {
        let (Some(current), Some(latest)) = (installed, catalog_latest) else {
            return match (installed, catalog_latest) {
                (None, _) => Self::NotInstalled,
                (_, None) => Self::Unknown,
                _ => unreachable!(),
            };
        };

        match current.compare_with_format(latest, format) {
            Ordering::Equal => Self::UpToDate,
            Ordering::Greater => Self::NewerThanCatalog {
                current: current.clone(),
                catalog_latest: latest.clone(),
            },
            Ordering::Less => {
                // T027 will add MajorUpgradeAvailable distinction here.
                Self::UpdateAvailable {
                    current: current.clone(),
                    available: latest.clone(),
                }
            }
        }
    }

    /// Returns `true` if this status represents a major version upgrade.
    ///
    /// Stub — always returns `false`. Full implementation in T027.
    pub fn is_major_upgrade(&self) -> bool {
        false
    }
}

impl fmt::Display for PackageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpToDate => write!(f, "up to date"),
            Self::UpdateAvailable {
                current,
                available,
            } => write!(f, "update available: {} -> {}", current.raw, available.raw),
            Self::MajorUpgradeAvailable {
                current,
                available,
            } => write!(
                f,
                "major upgrade available: {} -> {}",
                current.raw, available.raw
            ),
            Self::NewerThanCatalog {
                current,
                catalog_latest,
            } => write!(
                f,
                "newer than catalog: {} > {}",
                current.raw, catalog_latest.raw
            ),
            Self::NotInstalled => write!(f, "not installed"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Describes how version strings should be parsed and compared.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum VersionFormat {
    /// Default. Lenient semver parsing (v-prefix, 2-part, 4-part coercion).
    #[default]
    Semver,
    /// YYYY.MM.DD or YYYY-MM-DD chronological comparison.
    Date,
    /// Regex with capture groups, numeric group comparison.
    Custom {
        /// Regex pattern with capture groups for version extraction.
        pattern: String,
    },
}

impl fmt::Display for VersionFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Semver => write!(f, "semver"),
            Self::Date => write!(f, "date"),
            Self::Custom { pattern } => write!(f, "custom({pattern})"),
        }
    }
}

/// Compare two version strings according to the given [`VersionFormat`].
///
/// - **Semver**: uses lenient parsing via [`try_parse_lenient`]. Falls back to
///   lexicographic comparison if either string cannot be parsed.
/// - **Date**: stub — always returns [`Ordering::Equal`] (full implementation in T024).
/// - **Custom**: stub — always returns [`Ordering::Equal`] (full implementation in T025).
pub fn compare_versions(a: &str, b: &str, format: &VersionFormat) -> Ordering {
    match format {
        VersionFormat::Semver => compare_semver(a, b),
        VersionFormat::Date => {
            // Stub: full date comparison implemented in T024.
            Ordering::Equal
        }
        VersionFormat::Custom { .. } => {
            // Stub: full custom regex comparison implemented in T025.
            Ordering::Equal
        }
    }
}

/// Semver comparison using lenient parsing. Falls back to raw string comparison
/// when either version cannot be parsed.
fn compare_semver(a: &str, b: &str) -> Ordering {
    match (try_parse_lenient(a), try_parse_lenient(b)) {
        (Some(ref va), Some(ref vb)) => Ord::cmp(va, vb),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => a.cmp(b),
    }
}

impl Version {
    /// Compare this version to another using the given [`VersionFormat`].
    ///
    /// This delegates to [`compare_versions`] using the raw version strings.
    pub fn compare_with_format(&self, other: &Version, format: &VersionFormat) -> Ordering {
        compare_versions(&self.raw, &other.raw, format)
    }

    /// Returns `true` if the major semver component differs between `from` and `to`.
    ///
    /// Only meaningful for semver-parseable versions. Returns `false` if either
    /// version cannot be parsed.
    pub fn is_major_upgrade(from: &Version, to: &Version) -> bool {
        match (&from.parsed, &to.parsed) {
            (Some(a), Some(b)) => a.major != b.major,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Semver comparison: basic ordering ---

    #[test]
    fn semver_basic_ordering() {
        assert_eq!(
            compare_versions("1.0.0", "2.0.0", &VersionFormat::Semver),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("2.0.0", "1.0.0", &VersionFormat::Semver),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.2.3", "1.2.3", &VersionFormat::Semver),
            Ordering::Equal
        );
    }

    #[test]
    fn semver_minor_and_patch() {
        assert_eq!(
            compare_versions("1.0.0", "1.1.0", &VersionFormat::Semver),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("1.1.0", "1.1.1", &VersionFormat::Semver),
            Ordering::Less
        );
    }

    // --- Semver with lenient parsing ---

    #[test]
    fn semver_v_prefix() {
        assert_eq!(
            compare_versions("v1.0.0", "v2.0.0", &VersionFormat::Semver),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("v1.5.0", "1.5.0", &VersionFormat::Semver),
            Ordering::Equal
        );
    }

    #[test]
    fn semver_two_part_versions() {
        assert_eq!(
            compare_versions("3.1", "3.2", &VersionFormat::Semver),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("3.1", "3.1.0", &VersionFormat::Semver),
            Ordering::Equal
        );
    }

    #[test]
    fn semver_four_part_versions() {
        // 4-part versions: first 3 components are used
        assert_eq!(
            compare_versions("3.1.2.3001", "3.1.3.0", &VersionFormat::Semver),
            Ordering::Less
        );
    }

    #[test]
    fn semver_unparseable_fallback() {
        // Both unparseable: lexicographic comparison
        assert_eq!(
            compare_versions("abc", "def", &VersionFormat::Semver),
            Ordering::Less
        );
        // Parsed beats unparsed
        assert_eq!(
            compare_versions("1.0.0", "zzz", &VersionFormat::Semver),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("zzz", "1.0.0", &VersionFormat::Semver),
            Ordering::Less
        );
    }

    // --- Date format returns Equal (stub) ---

    #[test]
    fn date_format_stub_returns_equal() {
        assert_eq!(
            compare_versions("2024.01.15", "2025.06.01", &VersionFormat::Date),
            Ordering::Equal
        );
        assert_eq!(
            compare_versions("2024-01-15", "2025-06-01", &VersionFormat::Date),
            Ordering::Equal
        );
    }

    // --- Custom format returns Equal (stub) ---

    #[test]
    fn custom_format_stub_returns_equal() {
        let format = VersionFormat::Custom {
            pattern: r"(\d+)\.(\d+)".to_string(),
        };
        assert_eq!(compare_versions("1.0", "2.0", &format), Ordering::Equal);
    }

    // --- VersionFormat Display ---

    #[test]
    fn version_format_display() {
        assert_eq!(VersionFormat::Semver.to_string(), "semver");
        assert_eq!(VersionFormat::Date.to_string(), "date");
        assert_eq!(
            VersionFormat::Custom {
                pattern: r"\d+".to_string()
            }
            .to_string(),
            r"custom(\d+)"
        );
    }

    // --- VersionFormat Default ---

    #[test]
    fn version_format_default_is_semver() {
        assert_eq!(VersionFormat::default(), VersionFormat::Semver);
    }

    // --- VersionFormat serde round-trip ---

    #[test]
    fn version_format_serde_round_trip() {
        let formats = [
            VersionFormat::Semver,
            VersionFormat::Date,
            VersionFormat::Custom {
                pattern: r"(\d+)".to_string(),
            },
        ];
        for format in &formats {
            let json = serde_json::to_string(format).unwrap();
            let back: VersionFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, format);
        }
    }

    // --- Version::compare_with_format ---

    #[test]
    fn version_compare_with_format() {
        let v1 = Version::parse("1.0.0");
        let v2 = Version::parse("2.0.0");
        assert_eq!(
            v1.compare_with_format(&v2, &VersionFormat::Semver),
            Ordering::Less
        );
    }

    // --- Version::is_major_upgrade ---

    #[test]
    fn is_major_upgrade_true() {
        let from = Version::parse("1.5.0");
        let to = Version::parse("2.0.0");
        assert!(Version::is_major_upgrade(&from, &to));
    }

    #[test]
    fn is_major_upgrade_false_same_major() {
        let from = Version::parse("1.0.0");
        let to = Version::parse("1.9.0");
        assert!(!Version::is_major_upgrade(&from, &to));
    }

    #[test]
    fn is_major_upgrade_false_unparseable() {
        let from = Version {
            raw: "abc".into(),
            parsed: None,
        };
        let to = Version::parse("2.0.0");
        assert!(!Version::is_major_upgrade(&from, &to));
    }

    // --- PackageStatus::determine ---

    #[test]
    fn status_not_installed_when_no_installed_version() {
        let latest = Version::parse("1.0.0");
        let status = PackageStatus::determine(None, Some(&latest), &VersionFormat::Semver);
        assert_eq!(status, PackageStatus::NotInstalled);
    }

    #[test]
    fn status_unknown_when_no_catalog_version() {
        let installed = Version::parse("1.0.0");
        let status = PackageStatus::determine(Some(&installed), None, &VersionFormat::Semver);
        assert_eq!(status, PackageStatus::Unknown);
    }

    #[test]
    fn status_up_to_date_when_equal() {
        let installed = Version::parse("1.2.3");
        let latest = Version::parse("1.2.3");
        let status =
            PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
        assert_eq!(status, PackageStatus::UpToDate);
    }

    #[test]
    fn status_update_available_when_older() {
        let installed = Version::parse("1.0.0");
        let latest = Version::parse("1.5.0");
        let status =
            PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
        assert_eq!(
            status,
            PackageStatus::UpdateAvailable {
                current: installed,
                available: latest,
            }
        );
    }

    #[test]
    fn status_update_available_even_for_major_bump() {
        // T027 will change this to MajorUpgradeAvailable
        let installed = Version::parse("1.0.0");
        let latest = Version::parse("2.0.0");
        let status =
            PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
        assert_eq!(
            status,
            PackageStatus::UpdateAvailable {
                current: installed,
                available: latest,
            }
        );
    }

    #[test]
    fn status_newer_than_catalog() {
        let installed = Version::parse("3.0.0");
        let latest = Version::parse("2.5.0");
        let status =
            PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
        assert_eq!(
            status,
            PackageStatus::NewerThanCatalog {
                current: installed,
                catalog_latest: latest,
            }
        );
    }

    #[test]
    fn status_not_installed_when_both_none() {
        let status = PackageStatus::determine(None, None, &VersionFormat::Semver);
        assert_eq!(status, PackageStatus::NotInstalled);
    }

    // --- PackageStatus::is_major_upgrade stub ---

    #[test]
    fn is_major_upgrade_stub_always_false() {
        let statuses = [
            PackageStatus::UpToDate,
            PackageStatus::UpdateAvailable {
                current: Version::parse("1.0.0"),
                available: Version::parse("2.0.0"),
            },
            PackageStatus::MajorUpgradeAvailable {
                current: Version::parse("1.0.0"),
                available: Version::parse("2.0.0"),
            },
            PackageStatus::NotInstalled,
            PackageStatus::Unknown,
        ];
        for status in &statuses {
            assert!(!status.is_major_upgrade());
        }
    }

    // --- PackageStatus Display ---

    #[test]
    fn package_status_display() {
        assert_eq!(PackageStatus::UpToDate.to_string(), "up to date");
        assert_eq!(PackageStatus::NotInstalled.to_string(), "not installed");
        assert_eq!(PackageStatus::Unknown.to_string(), "unknown");
        assert_eq!(
            PackageStatus::UpdateAvailable {
                current: Version::parse("1.0.0"),
                available: Version::parse("2.0.0"),
            }
            .to_string(),
            "update available: 1.0.0 -> 2.0.0"
        );
        assert_eq!(
            PackageStatus::NewerThanCatalog {
                current: Version::parse("3.0.0"),
                catalog_latest: Version::parse("2.0.0"),
            }
            .to_string(),
            "newer than catalog: 3.0.0 > 2.0.0"
        );
    }
}
