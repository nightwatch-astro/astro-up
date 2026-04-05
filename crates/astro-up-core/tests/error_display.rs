#![allow(clippy::unwrap_used, clippy::expect_used)]

use astro_up_core::error::CoreError;
use astro_up_core::types::{CheckMethod, KnownExitCode};

#[test]
fn error_messages_are_readable() {
    let errors: Vec<CoreError> = vec![
        CoreError::ChecksumMismatch {
            expected: "abc123".into(),
            actual: "def456".into(),
        },
        CoreError::ManifestInvalid {
            id: "nina-app".into(),
            reason: "missing detection".into(),
        },
        CoreError::InstallerFailed {
            exit_code: 1,
            response: KnownExitCode::PackageInUse,
        },
        CoreError::ElevationRequired,
        CoreError::RebootRequired,
        CoreError::InstallerTimeout { timeout_secs: 600 },
        CoreError::InstallerBusy,
        CoreError::PackageInUse {
            process_name: "NINA.exe".into(),
        },
        CoreError::AlreadyInstalled {
            id: "nina-app".into(),
            version: "3.1.2".into(),
        },
        CoreError::MissingDependency {
            dep_id: "ascom-platform".into(),
        },
        CoreError::NotFound {
            input: "nonexistent".into(),
        },
        CoreError::VersionParseFailed {
            raw: "not-a-version".into(),
            cause: Box::new(std::io::Error::other("invalid format")),
        },
        CoreError::ProviderUnavailable {
            provider: CheckMethod::Github,
            cause: Box::new(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "connection refused",
            )),
        },
        CoreError::ManualDownloadRequired {
            id: "firecapture".into(),
            url: "https://example.com/download".into(),
            cause: Box::new(std::io::Error::other("captcha required")),
        },
    ];

    for err in &errors {
        let msg = err.to_string();
        assert!(!msg.is_empty(), "error message should not be empty");
        assert!(
            !msg.contains("CoreError"),
            "error message should not contain type name: {msg}"
        );
    }

    insta::assert_snapshot!(
        errors
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}
