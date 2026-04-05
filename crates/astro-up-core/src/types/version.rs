use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A version with lenient semver parsing.
///
/// Tries to parse as strict semver first. If that fails, applies coercion:
/// - 4-part (`3.1.2.3001`) → strip build component → `3.1.2`
/// - 2-part (`3.1`) → pad with `.0` → `3.1.0`
/// - Suffix (`6.6 SP2`) → treat suffix as pre-release → `6.6.0-SP2`
///
/// Always preserves the original `raw` string for display.
#[derive(Debug, Clone)]
pub struct Version {
    pub raw: String,
    pub parsed: Option<semver::Version>,
}

impl Version {
    pub fn parse(input: &str) -> Self {
        let raw = input.trim().to_string();
        let parsed = try_parse_lenient(&raw);
        Self { raw, parsed }
    }
}

pub(crate) fn try_parse_lenient(raw: &str) -> Option<semver::Version> {
    // Try strict semver first
    if let Ok(v) = semver::Version::parse(raw) {
        return Some(v);
    }

    let trimmed = raw.trim();

    // Strip leading 'v' or 'V'
    let s = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);

    // Try again after stripping prefix
    if let Ok(v) = semver::Version::parse(s) {
        return Some(v);
    }

    // Split on spaces — treat suffix as pre-release
    let (version_part, suffix) = match s.split_once(' ') {
        Some((v, s)) => (v, Some(s)),
        None => (s, None),
    };

    // Split on dots
    let parts: Vec<&str> = version_part.split('.').collect();

    let (major, minor, patch) = match parts.len() {
        1 => {
            let major = parts[0].parse::<u64>().ok()?;
            (major, 0, 0)
        }
        2 => {
            let major = parts[0].parse::<u64>().ok()?;
            let minor = parts[1].parse::<u64>().ok()?;
            (major, minor, 0)
        }
        3 => {
            let major = parts[0].parse::<u64>().ok()?;
            let minor = parts[1].parse::<u64>().ok()?;
            let patch = parts[2].parse::<u64>().ok()?;
            (major, minor, patch)
        }
        4.. => {
            // 4+ parts: take first 3, ignore rest
            let major = parts[0].parse::<u64>().ok()?;
            let minor = parts[1].parse::<u64>().ok()?;
            let patch = parts[2].parse::<u64>().ok()?;
            (major, minor, patch)
        }
        _ => return None,
    };

    let mut v = semver::Version::new(major, minor, patch);

    if let Some(suf) = suffix {
        let cleaned: String = suf
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '.')
            .collect();
        if !cleaned.is_empty() {
            v.pre = semver::Prerelease::new(&cleaned).unwrap_or_default();
        }
    }

    Some(v)
}

impl FromStr for Version {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        match (&self.parsed, &other.parsed) {
            (Some(a), Some(b)) => a == b,
            _ => self.raw == other.raw,
        }
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.parsed, &other.parsed) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => self.raw.cmp(&other.raw),
        }
    }
}

impl Serialize for Version {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::parse(&raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("1.0.0", 1, 0, 0)]
    #[case("3.1.2", 3, 1, 2)]
    #[case("v2.0.1", 2, 0, 1)]
    #[case("3.1", 3, 1, 0)]
    #[case("5", 5, 0, 0)]
    #[case("3.1.2.3001", 3, 1, 2)]
    #[case("4.1.12288.0", 4, 1, 12288)]
    fn parse_valid_versions(
        #[case] input: &str,
        #[case] major: u64,
        #[case] minor: u64,
        #[case] patch: u64,
    ) {
        let v = Version::parse(input);
        assert_eq!(v.raw, input);
        let parsed = v.parsed.expect("should parse");
        assert_eq!(parsed.major, major);
        assert_eq!(parsed.minor, minor);
        assert_eq!(parsed.patch, patch);
    }

    #[test]
    fn parse_suffix_as_prerelease() {
        let v = Version::parse("6.6 SP2");
        assert_eq!(v.raw, "6.6 SP2");
        let parsed = v.parsed.expect("should parse");
        assert_eq!(parsed.major, 6);
        assert_eq!(parsed.minor, 6);
        assert_eq!(parsed.patch, 0);
        assert!(!parsed.pre.is_empty());
    }

    #[test]
    fn ordering_parsed() {
        let v1 = Version::parse("1.0.0");
        let v2 = Version::parse("2.0.0");
        let v3 = Version::parse("1.1.0");
        assert!(v1 < v3);
        assert!(v3 < v2);
    }

    #[test]
    fn ordering_raw_fallback() {
        let v1 = Version {
            raw: "abc".into(),
            parsed: None,
        };
        let v2 = Version {
            raw: "def".into(),
            parsed: None,
        };
        assert!(v1 < v2);
    }

    #[test]
    fn parsed_beats_unparsed() {
        let parsed = Version::parse("1.0.0");
        let raw = Version {
            raw: "zzz".into(),
            parsed: None,
        };
        assert!(parsed > raw);
    }

    #[test]
    fn json_round_trip() {
        let v = Version::parse("3.1.2");
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"3.1.2\"");
        let back: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}
