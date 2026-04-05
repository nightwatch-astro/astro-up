use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::Version;

/// A discovered remote version with download URL and metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub url: String,
    #[serde(default)]
    pub asset_name: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub release_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub pre_release: bool,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn release_round_trip() {
        let release = Release {
            version: Version::parse("3.1.2"),
            url: "https://github.com/isbeorn/nina/releases/download/v3.1.2/NINA-3.1.2.exe".into(),
            asset_name: Some("NINA-3.1.2.exe".into()),
            sha256: Some("abc123".into()),
            release_date: None,
            changelog: Some("Bug fixes".into()),
            pre_release: false,
        };

        let json = serde_json::to_string(&release).unwrap();
        let back: Release = serde_json::from_str(&json).unwrap();
        assert_eq!(release.version, back.version);
        assert_eq!(release.url, back.url);
    }
}
