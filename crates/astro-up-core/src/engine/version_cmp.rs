//! Version format handling — semver, date, and custom regex parsers.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::sync::{LazyLock, Mutex};

use chrono::NaiveDate;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::types::version::{Version, try_parse_lenient};

/// Cache for compiled regexes used in custom version formats.
static REGEX_CACHE: LazyLock<Mutex<HashMap<String, Regex>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Status of a package relative to the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    ///   or [`PackageStatus::MajorUpgradeAvailable`] when the major semver
    ///   component differs (semver format only).
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
                if matches!(format, VersionFormat::Semver)
                    && Version::is_major_upgrade(current, latest)
                {
                    Self::MajorUpgradeAvailable {
                        current: current.clone(),
                        available: latest.clone(),
                    }
                } else {
                    Self::UpdateAvailable {
                        current: current.clone(),
                        available: latest.clone(),
                    }
                }
            }
        }
    }

    /// Returns `true` if this status represents a major version upgrade.
    pub fn is_major_upgrade(&self) -> bool {
        matches!(self, Self::MajorUpgradeAvailable { .. })
    }
}

impl fmt::Display for PackageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpToDate => write!(f, "up to date"),
            Self::UpdateAvailable { current, available } => {
                write!(f, "update available: {} -> {}", current.raw, available.raw)
            }
            Self::MajorUpgradeAvailable { current, available } => write!(
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

/// Parse a date version string into a [`NaiveDate`].
///
/// Supports `YYYY.MM.DD` and `YYYY-MM-DD` formats. Trailing text after the date
/// (e.g. `2024.01.15-beta`) is ignored.
pub fn parse_date(raw: &str) -> Option<NaiveDate> {
    // Try dot-separated first (YYYY.MM.DD), then dash-separated (YYYY-MM-DD).
    // We only look at the first 10 characters (or up to the first non-date char)
    // to handle trailing text.
    for sep in ['.', '-'] {
        let parts: Vec<&str> = raw.splitn(4, sep).collect();
        if parts.len() >= 3 {
            let year: i32 = parts[0].parse().ok()?;
            let month: u32 = parts[1].parse().ok()?;
            // The day part may have trailing text (e.g. "15-beta" or "15.1").
            // Take only leading digits.
            let day_str = parts[2]
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap_or("");
            let day: u32 = day_str.parse().ok()?;
            return NaiveDate::from_ymd_opt(year, month, day);
        }
    }
    None
}

/// Compare two version strings according to the given [`VersionFormat`].
///
/// - **Semver**: uses lenient parsing via [`try_parse_lenient`]. Falls back to
///   lexicographic comparison if either string cannot be parsed.
/// - **Date**: parses dates via [`parse_date`] and compares chronologically.
///   Falls back to lexicographic comparison if either date cannot be parsed.
/// - **Custom**: compiles regex (cached), extracts capture groups as numeric
///   components, and compares them component-by-component. Falls back to
///   lexicographic comparison if the regex is invalid or doesn't match.
pub fn compare_versions(a: &str, b: &str, format: &VersionFormat) -> Ordering {
    match format {
        VersionFormat::Semver => compare_semver(a, b),
        VersionFormat::Date => compare_date(a, b),
        VersionFormat::Custom { pattern } => compare_custom(a, b, pattern),
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

/// Date comparison using [`parse_date`]. Falls back to lexicographic comparison
/// when either date cannot be parsed.
fn compare_date(a: &str, b: &str) -> Ordering {
    match (parse_date(a), parse_date(b)) {
        (Some(da), Some(db)) => da.cmp(&db),
        _ => a.cmp(b),
    }
}

/// Parse a version string using a custom regex pattern.
///
/// Extracts all capture groups (numbered 1..N) and parses them as `u64`.
/// Returns `None` if the regex doesn't match or any capture group is not a valid integer.
pub fn parse_custom(raw: &str, re: &Regex) -> Option<Vec<u64>> {
    let caps = re.captures(raw)?;
    let mut components = Vec::new();
    for i in 1..caps.len() {
        let s = caps.get(i)?.as_str();
        components.push(s.parse::<u64>().ok()?);
    }
    if components.is_empty() {
        return None;
    }
    Some(components)
}

/// Custom regex comparison. Compiles and caches the regex, then extracts
/// numeric capture groups from both version strings and compares them
/// component-by-component. Falls back to string comparison when the regex
/// is invalid or doesn't match either version.
fn compare_custom(a: &str, b: &str, pattern: &str) -> Ordering {
    let re = {
        let Ok(mut cache) = REGEX_CACHE.lock() else {
            return a.cmp(b);
        };
        if let Some(cached) = cache.get(pattern) {
            cached.clone()
        } else {
            match Regex::new(pattern) {
                Ok(re) => {
                    cache.insert(pattern.to_string(), re.clone());
                    re
                }
                Err(_) => return a.cmp(b),
            }
        }
    };

    match (parse_custom(a, &re), parse_custom(b, &re)) {
        (Some(ref ca), Some(ref cb)) => ca.cmp(cb),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => a.cmp(b),
    }
}

/// Check whether a raw version string is compatible with the expected format.
///
/// Returns `Some(warning_message)` when the string does not match the expected
/// format and the comparison will fall back to raw string ordering. Returns
/// `None` when the string parses successfully under the given format.
pub fn check_format_compatibility(raw: &str, format: &VersionFormat) -> Option<String> {
    match format {
        VersionFormat::Semver => {
            if try_parse_lenient(raw).is_none() {
                Some(format!(
                    "version \"{raw}\" does not match semver format; \
                     comparison will use raw string ordering"
                ))
            } else {
                None
            }
        }
        VersionFormat::Date => {
            if parse_date(raw).is_none() {
                Some(format!(
                    "version \"{raw}\" does not match date format (YYYY.MM.DD / YYYY-MM-DD); \
                     comparison will use raw string ordering"
                ))
            } else {
                None
            }
        }
        VersionFormat::Custom { pattern } => {
            let re = {
                let Ok(mut cache) = REGEX_CACHE.lock() else {
                    return Some(format!(
                        "version \"{raw}\" could not be checked against custom pattern \
                         \"{pattern}\"; regex cache is poisoned"
                    ));
                };
                if let Some(cached) = cache.get(pattern.as_str()) {
                    cached.clone()
                } else {
                    match Regex::new(pattern) {
                        Ok(re) => {
                            cache.insert(pattern.clone(), re.clone());
                            re
                        }
                        Err(_) => {
                            return Some(format!(
                                "custom version pattern \"{pattern}\" is not valid regex; \
                                 comparison will use raw string ordering"
                            ));
                        }
                    }
                }
            };
            if parse_custom(raw, &re).is_none() {
                Some(format!(
                    "version \"{raw}\" does not match custom pattern \"{pattern}\"; \
                     comparison will use raw string ordering"
                ))
            } else {
                None
            }
        }
    }
}

impl Version {
    /// Compare this version to another using the given [`VersionFormat`].
    ///
    /// This delegates to [`compare_versions`] using the raw version strings.
    pub fn compare_with_format(&self, other: &Self, format: &VersionFormat) -> Ordering {
        compare_versions(&self.raw, &other.raw, format)
    }

    /// Returns `true` if the major semver component differs between `from` and `to`.
    ///
    /// Only meaningful for semver-parseable versions. Returns `false` if either
    /// version cannot be parsed.
    pub fn is_major_upgrade(from: &Self, to: &Self) -> bool {
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
    fn semver_unparsable_fallback() {
        // Both unparsable: lexicographic comparison
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

    // --- parse_date ---

    #[test]
    fn parse_date_dot_separated() {
        assert_eq!(
            parse_date("2024.01.15"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
    }

    #[test]
    fn parse_date_dash_separated() {
        assert_eq!(
            parse_date("2024-01-15"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
    }

    #[test]
    fn parse_date_trailing_text_dot() {
        assert_eq!(
            parse_date("2024.03.10-beta"),
            Some(NaiveDate::from_ymd_opt(2024, 3, 10).unwrap())
        );
    }

    #[test]
    fn parse_date_trailing_text_dash() {
        assert_eq!(
            parse_date("2024-03-10-rc1"),
            Some(NaiveDate::from_ymd_opt(2024, 3, 10).unwrap())
        );
    }

    #[test]
    fn parse_date_invalid_returns_none() {
        assert_eq!(parse_date("not-a-date"), None);
        assert_eq!(parse_date("abc"), None);
        assert_eq!(parse_date(""), None);
    }

    #[test]
    fn parse_date_invalid_day_returns_none() {
        // February 30 does not exist
        assert_eq!(parse_date("2024.02.30"), None);
    }

    // --- Date format comparison ---

    #[test]
    fn date_format_basic_ordering() {
        assert_eq!(
            compare_versions("2024.01.15", "2025.06.01", &VersionFormat::Date),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("2025.06.01", "2024.01.15", &VersionFormat::Date),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("2024.01.15", "2024.01.15", &VersionFormat::Date),
            Ordering::Equal
        );
    }

    #[test]
    fn date_format_dash_separator() {
        assert_eq!(
            compare_versions("2024-01-15", "2025-06-01", &VersionFormat::Date),
            Ordering::Less
        );
    }

    #[test]
    fn date_format_mixed_separators() {
        // Both get parsed to the same date regardless of separator
        assert_eq!(
            compare_versions("2024.01.15", "2024-01-15", &VersionFormat::Date),
            Ordering::Equal
        );
    }

    #[test]
    fn date_format_with_trailing_text() {
        assert_eq!(
            compare_versions("2024.01.15-beta", "2025.06.01", &VersionFormat::Date),
            Ordering::Less
        );
    }

    #[test]
    fn date_format_unparsable_fallback() {
        // Both unparsable: lexicographic comparison
        assert_eq!(
            compare_versions("abc", "def", &VersionFormat::Date),
            Ordering::Less
        );
        // One parsable, one not: falls back to string comparison
        assert_eq!(
            compare_versions("2024.01.15", "zzz", &VersionFormat::Date),
            Ordering::Less
        );
    }

    #[test]
    fn date_format_same_year_different_month() {
        assert_eq!(
            compare_versions("2024.01.01", "2024.12.01", &VersionFormat::Date),
            Ordering::Less
        );
    }

    #[test]
    fn date_format_same_month_different_day() {
        assert_eq!(
            compare_versions("2024.06.01", "2024.06.30", &VersionFormat::Date),
            Ordering::Less
        );
    }

    // --- Custom format returns Equal (stub) ---

    #[test]
    fn custom_format_basic_ordering() {
        let format = VersionFormat::Custom {
            pattern: r"(\d+)\.(\d+)".to_string(),
        };
        assert_eq!(compare_versions("1.0", "2.0", &format), Ordering::Less);
        assert_eq!(compare_versions("2.0", "1.0", &format), Ordering::Greater);
        assert_eq!(compare_versions("1.0", "1.0", &format), Ordering::Equal);
    }

    #[test]
    fn custom_format_multi_component() {
        let format = VersionFormat::Custom {
            pattern: r"(\d+)\.(\d+)\.(\d+)".to_string(),
        };
        assert_eq!(compare_versions("1.2.3", "1.2.4", &format), Ordering::Less);
        assert_eq!(
            compare_versions("1.3.0", "1.2.9", &format),
            Ordering::Greater
        );
    }

    #[test]
    fn custom_format_single_group() {
        let format = VersionFormat::Custom {
            pattern: r"v(\d+)".to_string(),
        };
        assert_eq!(compare_versions("v10", "v9", &format), Ordering::Greater);
    }

    #[test]
    fn custom_format_no_match_falls_back_to_string() {
        let format = VersionFormat::Custom {
            pattern: r"(\d+)\.(\d+)".to_string(),
        };
        assert_eq!(
            compare_versions("abc", "def", &format),
            Ordering::Less // string comparison
        );
    }

    #[test]
    fn custom_format_invalid_regex_falls_back_to_string() {
        let format = VersionFormat::Custom {
            pattern: r"([invalid".to_string(),
        };
        assert_eq!(compare_versions("a", "b", &format), Ordering::Less);
    }

    #[test]
    fn parse_custom_extracts_groups() {
        let re = Regex::new(r"(\d+)\.(\d+)").unwrap();
        assert_eq!(parse_custom("3.14", &re), Some(vec![3, 14]));
    }

    #[test]
    fn parse_custom_returns_none_on_no_match() {
        let re = Regex::new(r"(\d+)\.(\d+)").unwrap();
        assert_eq!(parse_custom("abc", &re), None);
    }

    #[test]
    fn parse_custom_returns_none_on_non_numeric_group() {
        let re = Regex::new(r"([a-z]+)-(\d+)").unwrap();
        // First group is non-numeric, should return None
        assert_eq!(parse_custom("abc-123", &re), None);
    }

    #[test]
    fn parse_custom_no_capture_groups() {
        let re = Regex::new(r"\d+\.\d+").unwrap();
        assert_eq!(parse_custom("1.2", &re), None);
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
    fn is_major_upgrade_false_unparsable() {
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
    fn status_major_upgrade_available_for_major_bump() {
        let installed = Version::parse("1.0.0");
        let latest = Version::parse("2.0.0");
        let status =
            PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
        assert_eq!(
            status,
            PackageStatus::MajorUpgradeAvailable {
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

    // --- PackageStatus::is_major_upgrade ---

    #[test]
    fn is_major_upgrade_true_for_major_upgrade_available() {
        let status = PackageStatus::MajorUpgradeAvailable {
            current: Version::parse("1.0.0"),
            available: Version::parse("2.0.0"),
        };
        assert!(status.is_major_upgrade());
    }

    #[test]
    fn is_major_upgrade_false_for_other_variants() {
        let statuses = [
            PackageStatus::UpToDate,
            PackageStatus::UpdateAvailable {
                current: Version::parse("1.0.0"),
                available: Version::parse("1.5.0"),
            },
            PackageStatus::NewerThanCatalog {
                current: Version::parse("3.0.0"),
                catalog_latest: Version::parse("2.0.0"),
            },
            PackageStatus::NotInstalled,
            PackageStatus::Unknown,
        ];
        for status in &statuses {
            assert!(!status.is_major_upgrade(), "expected false for {status}");
        }
    }

    // --- PackageStatus::determine — major vs minor distinction ---

    #[test]
    fn status_minor_update_stays_update_available() {
        let installed = Version::parse("1.0.0");
        let latest = Version::parse("1.5.0");
        let status =
            PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
        assert!(matches!(status, PackageStatus::UpdateAvailable { .. }));
        assert!(!status.is_major_upgrade());
    }

    #[test]
    fn status_patch_update_stays_update_available() {
        let installed = Version::parse("2.1.0");
        let latest = Version::parse("2.1.5");
        let status =
            PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
        assert!(matches!(status, PackageStatus::UpdateAvailable { .. }));
    }

    #[test]
    fn status_major_bump_with_unparsable_stays_update_available() {
        let installed = Version {
            raw: "abc".into(),
            parsed: None,
        };
        let latest = Version::parse("2.0.0");
        let status =
            PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
        assert!(matches!(status, PackageStatus::UpdateAvailable { .. }));
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

    // --- check_format_compatibility ---

    #[test]
    fn compatibility_semver_with_valid_string() {
        assert!(check_format_compatibility("1.2.3", &VersionFormat::Semver).is_none());
        assert!(check_format_compatibility("v2.0", &VersionFormat::Semver).is_none());
    }

    #[test]
    fn compatibility_semver_with_unparsable_string() {
        let warning = check_format_compatibility("not-a-version", &VersionFormat::Semver);
        assert!(warning.is_some());
        let msg = warning.unwrap();
        assert!(msg.contains("not-a-version"));
        assert!(msg.contains("semver"));
    }

    #[test]
    fn compatibility_date_with_valid_string() {
        assert!(check_format_compatibility("2024.01.15", &VersionFormat::Date).is_none());
        assert!(check_format_compatibility("2024-06-01", &VersionFormat::Date).is_none());
    }

    #[test]
    fn compatibility_date_with_unparsable_string() {
        let warning = check_format_compatibility("not-a-date", &VersionFormat::Date);
        assert!(warning.is_some());
        let msg = warning.unwrap();
        assert!(msg.contains("not-a-date"));
        assert!(msg.contains("date format"));
    }

    #[test]
    fn compatibility_custom_with_matching_string() {
        let fmt = VersionFormat::Custom {
            pattern: r"(\d+)\.(\d+)".to_string(),
        };
        assert!(check_format_compatibility("3.14", &fmt).is_none());
    }

    #[test]
    fn compatibility_custom_with_non_matching_string() {
        let fmt = VersionFormat::Custom {
            pattern: r"(\d+)\.(\d+)".to_string(),
        };
        let warning = check_format_compatibility("abc-xyz", &fmt);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("custom pattern"));
    }

    #[test]
    fn compatibility_custom_with_invalid_regex() {
        let fmt = VersionFormat::Custom {
            pattern: r"([invalid".to_string(),
        };
        let warning = check_format_compatibility("anything", &fmt);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("not valid regex"));
    }
}
