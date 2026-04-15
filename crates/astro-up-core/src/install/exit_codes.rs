use crate::install::types::ExitCodeOutcome;
use crate::types::{InstallConfig, KnownExitCode};

/// Well-known Windows exit codes with universal meaning.
const EXIT_CODE_SUCCESS: i32 = 0;
const EXIT_CODE_ELEVATION_REQUIRED: i32 = 740;
/// HRESULT E_ACCESSDENIED (0x80070005) — Burn bootstrappers return this when
/// they detect insufficient privileges. Treat the same as exit code 740.
const EXIT_CODE_ACCESS_DENIED: i32 = -2_147_024_891; // 0x80070005 as i32
const EXIT_CODE_REBOOT_REQUIRED: i32 = 3010;
const EXIT_CODE_REBOOT_INITIATED: i32 = 1641;

/// Interprets an installer exit code using the following precedence:
///
/// 1. Exit code 0 → Success
/// 2. Exit code in `success_codes` → Success (semantic meaning is informational)
/// 3. Exit code in `known_exit_codes` → map to semantic outcome
/// 4. Well-known Windows codes (740 = elevation, 3010/1641 = reboot)
/// 5. Else → Failed with raw code
pub fn interpret_exit_code(code: i32, config: &InstallConfig) -> ExitCodeOutcome {
    // 1. Zero is always success
    if code == EXIT_CODE_SUCCESS {
        return ExitCodeOutcome::Success;
    }

    // 2. success_codes takes priority — treat as success even if also in known_exit_codes
    if config.success_codes.contains(&code) {
        return ExitCodeOutcome::Success;
    }

    // 3. Per-manifest known_exit_codes mapping
    let code_str = code.to_string();
    if let Some(known) = config.known_exit_codes.get(&code_str) {
        return match known {
            KnownExitCode::RebootRequired | KnownExitCode::SuccessRebootInitiated => {
                ExitCodeOutcome::SuccessRebootRequired
            }
            other => ExitCodeOutcome::Failed {
                code,
                semantic: Some(other.clone()),
            },
        };
    }

    // 4. Well-known Windows universal codes
    match code {
        EXIT_CODE_ELEVATION_REQUIRED | EXIT_CODE_ACCESS_DENIED => {
            ExitCodeOutcome::ElevationRequired
        }
        EXIT_CODE_REBOOT_REQUIRED | EXIT_CODE_REBOOT_INITIATED => {
            ExitCodeOutcome::SuccessRebootRequired
        }
        _ => ExitCodeOutcome::Failed {
            code,
            semantic: None,
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn base_config() -> InstallConfig {
        InstallConfig::default()
    }

    #[test]
    fn zero_is_success() {
        assert_eq!(
            interpret_exit_code(0, &base_config()),
            ExitCodeOutcome::Success
        );
    }

    #[test]
    fn success_codes_override() {
        let mut config = base_config();
        config.success_codes = vec![42, 99];
        assert_eq!(interpret_exit_code(42, &config), ExitCodeOutcome::Success);
        assert_eq!(interpret_exit_code(99, &config), ExitCodeOutcome::Success);
    }

    #[test]
    fn success_codes_take_priority_over_known_exit_codes() {
        let mut config = base_config();
        config.success_codes = vec![1];
        config
            .known_exit_codes
            .insert("1".into(), KnownExitCode::PackageInUse);
        // success_codes wins — treat as success
        assert_eq!(interpret_exit_code(1, &config), ExitCodeOutcome::Success);
    }

    #[test]
    fn known_exit_code_package_in_use() {
        let mut config = base_config();
        config
            .known_exit_codes
            .insert("1".into(), KnownExitCode::PackageInUse);
        assert_eq!(
            interpret_exit_code(1, &config),
            ExitCodeOutcome::Failed {
                code: 1,
                semantic: Some(KnownExitCode::PackageInUse),
            }
        );
    }

    #[test]
    fn known_exit_code_reboot_required() {
        let mut config = base_config();
        config
            .known_exit_codes
            .insert("5".into(), KnownExitCode::RebootRequired);
        assert_eq!(
            interpret_exit_code(5, &config),
            ExitCodeOutcome::SuccessRebootRequired
        );
    }

    #[test]
    fn known_exit_code_already_installed() {
        let mut config = base_config();
        config
            .known_exit_codes
            .insert("1638".into(), KnownExitCode::AlreadyInstalled);
        assert_eq!(
            interpret_exit_code(1638, &config),
            ExitCodeOutcome::Failed {
                code: 1638,
                semantic: Some(KnownExitCode::AlreadyInstalled),
            }
        );
    }

    #[test]
    fn universal_elevation_740() {
        assert_eq!(
            interpret_exit_code(740, &base_config()),
            ExitCodeOutcome::ElevationRequired
        );
    }

    #[test]
    fn universal_reboot_3010() {
        assert_eq!(
            interpret_exit_code(3010, &base_config()),
            ExitCodeOutcome::SuccessRebootRequired
        );
    }

    #[test]
    fn universal_reboot_1641() {
        assert_eq!(
            interpret_exit_code(1641, &base_config()),
            ExitCodeOutcome::SuccessRebootRequired
        );
    }

    #[test]
    fn unknown_exit_code() {
        assert_eq!(
            interpret_exit_code(999, &base_config()),
            ExitCodeOutcome::Failed {
                code: 999,
                semantic: None,
            }
        );
    }

    #[test]
    fn all_known_exit_code_variants_mapped() {
        let mut config = base_config();
        let mappings = vec![
            ("10", KnownExitCode::PackageInUse),
            ("11", KnownExitCode::PackageInUseByApplication),
            ("12", KnownExitCode::CancelledByUser),
            ("13", KnownExitCode::MissingDependency),
            ("14", KnownExitCode::DiskFull),
            ("15", KnownExitCode::InsufficientMemory),
            ("16", KnownExitCode::NetworkError),
            ("17", KnownExitCode::ContactSupport),
            ("18", KnownExitCode::RestartRequired),
        ];
        for (code_str, known) in &mappings {
            config
                .known_exit_codes
                .insert((*code_str).into(), known.clone());
        }
        for (code_str, known) in &mappings {
            let code: i32 = code_str.parse().unwrap();
            let result = interpret_exit_code(code, &config);
            match result {
                ExitCodeOutcome::Failed { semantic, .. } => {
                    assert_eq!(semantic, Some(known.clone()));
                }
                _ => panic!("Expected Failed for {code_str}"),
            }
        }
    }
}
