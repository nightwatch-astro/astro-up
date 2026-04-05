use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use crate::types::Version;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LedgerSource {
    AstroUp,
    Manual,
    Acknowledged,
}

/// A manual version record for packages that cannot be detected automatically.
///
/// Priority: auto-detection > ledger > unknown.
/// If auto-detection and ledger disagree, warn the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub package_id: String,
    pub version: Version,
    pub source: LedgerSource,
    pub recorded_at: DateTime<Utc>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub install_path: Option<std::path::PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_entry_round_trip() {
        let entry = LedgerEntry {
            package_id: "zwo-firmware".into(),
            version: Version::parse("1.2.3"),
            source: LedgerSource::Acknowledged,
            recorded_at: Utc::now(),
            notes: Some("User confirmed via USB update tool".into()),
            install_path: Some(std::path::PathBuf::from("C:\\Program Files\\ZWO")),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let back: LedgerEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.package_id, back.package_id);
        assert_eq!(entry.source, back.source);
        assert_eq!(entry.version, back.version);
    }
}
