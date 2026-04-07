use std::path::Path;

use chrono::Utc;

use crate::ledger::{LedgerEntry, LedgerSource};
use crate::types::Version;

/// Creates a `LedgerEntry` for a successful install.
///
/// The caller is responsible for persisting this entry to the SQLite store.
pub fn record_install(
    package_id: &str,
    version: &Version,
    install_path: Option<&Path>,
) -> LedgerEntry {
    tracing::debug!(
        package_id,
        version = %version,
        install_path = ?install_path,
        "recording install ledger entry"
    );
    LedgerEntry {
        package_id: package_id.to_string(),
        version: version.clone(),
        source: LedgerSource::AstroUp,
        recorded_at: Utc::now(),
        notes: None,
        install_path: install_path.map(Path::to_path_buf),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn record_install_basic() {
        let version = Version::parse("3.1.2");
        let path = std::path::Path::new("C:\\Programs\\NINA");
        let entry = record_install("nina-app", &version, Some(path));

        assert_eq!(entry.package_id, "nina-app");
        assert_eq!(entry.version, version);
        assert_eq!(entry.source, LedgerSource::AstroUp);
        assert_eq!(entry.install_path, Some(path.to_path_buf()));
        assert!(entry.notes.is_none());
    }

    #[test]
    fn record_install_no_path() {
        let version = Version::parse("1.0.0");
        let entry = record_install("phd2", &version, None);

        assert_eq!(entry.package_id, "phd2");
        assert!(entry.install_path.is_none());
    }
}
