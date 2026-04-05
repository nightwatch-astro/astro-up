//! Catalog types — PackageId, PackageSummary, VersionEntry, and supporting types.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::types::{Category, SoftwareType};

// ---------------------------------------------------------------------------
// PackageId
// ---------------------------------------------------------------------------

/// Validated package identifier.
///
/// Must match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`, length 2–50.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackageId(String);

impl PackageId {
    /// Validate and construct a `PackageId`.
    pub fn new(s: &str) -> Result<Self, CoreError> {
        Self::validate(s)?;
        Ok(Self(s.to_owned()))
    }

    fn validate(s: &str) -> Result<(), CoreError> {
        if s.len() < 2 || s.len() > 50 {
            return Err(CoreError::InvalidPackageId {
                input: s.to_owned(),
                reason: format!("length must be 2–50, got {}", s.len()),
            });
        }
        // ^[a-z][a-z0-9]*(-[a-z0-9]+)*$
        let bytes = s.as_bytes();
        if !bytes[0].is_ascii_lowercase() {
            return Err(CoreError::InvalidPackageId {
                input: s.to_owned(),
                reason: "must start with a lowercase letter".into(),
            });
        }
        let mut prev_hyphen = false;
        for &b in &bytes[1..] {
            match b {
                b'a'..=b'z' | b'0'..=b'9' => prev_hyphen = false,
                b'-' => {
                    if prev_hyphen {
                        return Err(CoreError::InvalidPackageId {
                            input: s.to_owned(),
                            reason: "consecutive hyphens not allowed".into(),
                        });
                    }
                    prev_hyphen = true;
                }
                _ => {
                    return Err(CoreError::InvalidPackageId {
                        input: s.to_owned(),
                        reason: format!(
                            "invalid character '{}', only lowercase alphanumeric and hyphens allowed",
                            b as char
                        ),
                    });
                }
            }
        }
        if prev_hyphen {
            return Err(CoreError::InvalidPackageId {
                input: s.to_owned(),
                reason: "must not end with a hyphen".into(),
            });
        }
        Ok(())
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for PackageId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for PackageId {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for PackageId {
    type Error = CoreError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::validate(&s)?;
        Ok(Self(s))
    }
}

impl From<PackageId> for String {
    fn from(id: PackageId) -> Self {
        id.0
    }
}

// ---------------------------------------------------------------------------
// PackageSummary
// ---------------------------------------------------------------------------

/// Query-only view of a package from the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSummary {
    pub id: PackageId,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub homepage: Option<String>,
    pub category: Category,
    pub software_type: SoftwareType,
    pub license: Option<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub manifest_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_base64: Option<String>,
}

// ---------------------------------------------------------------------------
// VersionEntry
// ---------------------------------------------------------------------------

/// A discovered version for a package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionEntry {
    pub package_id: PackageId,
    pub version: String,
    pub url: String,
    pub sha256: Option<String>,
    pub discovered_at: DateTime<Utc>,
    pub release_notes_url: Option<String>,
    pub pre_release: bool,
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Catalog metadata from the `meta` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogMeta {
    pub schema_version: String,
    pub compiled_at: DateTime<Utc>,
}

/// Search result with FTS5 relevance rank.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub package: PackageSummary,
    pub rank: f64,
}

/// Filter criteria for catalog queries.
#[derive(Debug, Clone, Default)]
pub struct CatalogFilter {
    pub category: Option<Category>,
    pub software_type: Option<SoftwareType>,
}

/// Result of a catalog fetch operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchResult {
    /// New catalog downloaded and verified.
    Updated,
    /// Server returned 304 — local catalog is current.
    Unchanged,
    /// Fetch failed but local catalog is available.
    FallbackToLocal { reason: String },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn valid_package_ids() {
        for id in &[
            "ab",
            "nina",
            "phd2",
            "ascom-platform",
            "all-sky-plate-solver",
            "a1",
        ] {
            assert!(PackageId::new(id).is_ok(), "expected valid: {id}");
        }
    }

    #[test]
    fn invalid_too_short() {
        assert!(PackageId::new("a").is_err());
    }

    #[test]
    fn invalid_too_long() {
        let long = "a".repeat(51);
        assert!(PackageId::new(&long).is_err());
    }

    #[test]
    fn invalid_uppercase() {
        assert!(PackageId::new("Nina").is_err());
        assert!(PackageId::new("NINA").is_err());
    }

    #[test]
    fn invalid_starts_with_digit() {
        assert!(PackageId::new("2nina").is_err());
    }

    #[test]
    fn invalid_starts_with_hyphen() {
        assert!(PackageId::new("-nina").is_err());
    }

    #[test]
    fn invalid_ends_with_hyphen() {
        assert!(PackageId::new("nina-").is_err());
    }

    #[test]
    fn invalid_consecutive_hyphens() {
        assert!(PackageId::new("nina--app").is_err());
    }

    #[test]
    fn invalid_special_chars() {
        assert!(PackageId::new("nina.app").is_err());
        assert!(PackageId::new("nina_app").is_err());
        assert!(PackageId::new("nina app").is_err());
    }

    #[test]
    fn display_and_as_ref() {
        let id = PackageId::new("nina").unwrap();
        assert_eq!(id.to_string(), "nina");
        assert_eq!(id.as_ref(), "nina");
    }

    #[test]
    fn from_str_roundtrip() {
        let id: PackageId = "phd2".parse().unwrap();
        assert_eq!(String::from(id), "phd2");
    }

    #[test]
    fn serde_roundtrip() {
        let id = PackageId::new("ascom-platform").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"ascom-platform\"");
        let back: PackageId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn max_length_valid() {
        let id = format!("a{}", "b".repeat(49));
        assert_eq!(id.len(), 50);
        assert!(PackageId::new(&id).is_ok());
    }
}
