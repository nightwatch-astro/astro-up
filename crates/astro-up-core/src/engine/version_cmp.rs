//! Version format handling — semver, date, and custom regex parsers.

use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::types::version::{try_parse_lenient, Version};

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
}
