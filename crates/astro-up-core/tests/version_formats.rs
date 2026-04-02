//! Integration tests for `VersionFormat` comparisons, `Version::is_major_upgrade`,
//! and `PackageStatus::determine`.

use std::cmp::Ordering;

use astro_up_core::engine::version_cmp::{
    PackageStatus, VersionFormat, check_format_compatibility, compare_versions,
};
use astro_up_core::types::Version;

// ---------------------------------------------------------------------------
// 1. Semver strict — standard semver strings
// ---------------------------------------------------------------------------

#[test]
fn semver_strict_less() {
    assert_eq!(
        compare_versions("1.0.0", "2.0.0", &VersionFormat::Semver),
        Ordering::Less
    );
}

#[test]
fn semver_strict_greater() {
    assert_eq!(
        compare_versions("3.0.0", "2.0.0", &VersionFormat::Semver),
        Ordering::Greater
    );
}

#[test]
fn semver_strict_equal() {
    assert_eq!(
        compare_versions("1.2.3", "1.2.3", &VersionFormat::Semver),
        Ordering::Equal
    );
}

#[test]
fn semver_strict_minor_bump() {
    assert_eq!(
        compare_versions("1.0.0", "1.1.0", &VersionFormat::Semver),
        Ordering::Less
    );
}

#[test]
fn semver_strict_patch_bump() {
    assert_eq!(
        compare_versions("1.1.0", "1.1.1", &VersionFormat::Semver),
        Ordering::Less
    );
}

#[test]
fn semver_strict_prerelease_less_than_release() {
    // semver crate: 1.0.0-alpha < 1.0.0
    assert_eq!(
        compare_versions("1.0.0-alpha", "1.0.0", &VersionFormat::Semver),
        Ordering::Less
    );
}

// ---------------------------------------------------------------------------
// 2. Semver lenient — v-prefix, 2-part, space suffixes
// ---------------------------------------------------------------------------

#[test]
fn semver_lenient_v_prefix() {
    assert_eq!(
        compare_versions("v1.0.0", "v2.0.0", &VersionFormat::Semver),
        Ordering::Less
    );
}

#[test]
fn semver_lenient_v_prefix_mixed() {
    assert_eq!(
        compare_versions("v1.5.0", "1.5.0", &VersionFormat::Semver),
        Ordering::Equal
    );
}

#[test]
fn semver_lenient_two_part() {
    assert_eq!(
        compare_versions("3.1", "3.2", &VersionFormat::Semver),
        Ordering::Less
    );
}

#[test]
fn semver_lenient_two_part_equals_three_part() {
    assert_eq!(
        compare_versions("3.1", "3.1.0", &VersionFormat::Semver),
        Ordering::Equal
    );
}

#[test]
fn semver_lenient_space_suffix_trimmed() {
    // Version::parse trims whitespace; compare_versions uses raw strings,
    // but lenient parsing should handle trailing spaces.
    let a = Version::parse("1.2.3 ");
    let b = Version::parse("1.2.3");
    assert_eq!(
        a.compare_with_format(&b, &VersionFormat::Semver),
        Ordering::Equal
    );
}

// ---------------------------------------------------------------------------
// 3. Date format — YYYY.MM.DD and YYYY-MM-DD
// ---------------------------------------------------------------------------

#[test]
fn date_format_dot_separator() {
    assert_eq!(
        compare_versions("2024.01.15", "2025.06.01", &VersionFormat::Date),
        Ordering::Less,
    );
}

#[test]
fn date_format_dash_separator() {
    assert_eq!(
        compare_versions("2024-01-15", "2025-06-01", &VersionFormat::Date),
        Ordering::Less,
    );
}

#[test]
fn date_format_same_date() {
    assert_eq!(
        compare_versions("2024.01.01", "2024.01.01", &VersionFormat::Date),
        Ordering::Equal,
    );
}

// ---------------------------------------------------------------------------
// 4. Custom regex
// ---------------------------------------------------------------------------

#[test]
fn custom_regex_ordering() {
    let fmt = VersionFormat::Custom {
        pattern: r"(\d+)\.(\d+)".to_string(),
    };
    assert_eq!(compare_versions("1.0", "2.0", &fmt), Ordering::Less,);
}

#[test]
fn custom_regex_greater() {
    let fmt = VersionFormat::Custom {
        pattern: r"v(\d+)".to_string(),
    };
    assert_eq!(compare_versions("v100", "v1", &fmt), Ordering::Greater,);
}

// ---------------------------------------------------------------------------
// 5. 4-part coercion — versions like 3.1.2.3001
// ---------------------------------------------------------------------------

#[test]
fn four_part_coercion_less() {
    // Lenient parser coerces to first 3 parts: 3.1.2 vs 3.1.3
    assert_eq!(
        compare_versions("3.1.2.3001", "3.1.3.0", &VersionFormat::Semver),
        Ordering::Less
    );
}

#[test]
fn four_part_coercion_equal_first_three() {
    // Both coerce to 3.1.2 — should be equal
    assert_eq!(
        compare_versions("3.1.2.100", "3.1.2.999", &VersionFormat::Semver),
        Ordering::Equal
    );
}

#[test]
fn four_part_coercion_greater() {
    assert_eq!(
        compare_versions("4.0.0.1", "3.9.9.9999", &VersionFormat::Semver),
        Ordering::Greater
    );
}

#[test]
fn four_part_vs_three_part() {
    assert_eq!(
        compare_versions("3.1.2.3001", "3.1.2", &VersionFormat::Semver),
        Ordering::Equal,
        "4-part version coerces to first 3 parts"
    );
}

// ---------------------------------------------------------------------------
// 6. Version::is_major_upgrade
// ---------------------------------------------------------------------------

#[test]
fn is_major_upgrade_true_when_major_differs() {
    let from = Version::parse("1.5.0");
    let to = Version::parse("2.0.0");
    assert!(Version::is_major_upgrade(&from, &to));
}

#[test]
fn is_major_upgrade_true_downgrade() {
    // Major differs regardless of direction
    let from = Version::parse("3.0.0");
    let to = Version::parse("1.0.0");
    assert!(Version::is_major_upgrade(&from, &to));
}

#[test]
fn is_major_upgrade_false_same_major() {
    let from = Version::parse("1.0.0");
    let to = Version::parse("1.9.9");
    assert!(!Version::is_major_upgrade(&from, &to));
}

#[test]
fn is_major_upgrade_false_equal() {
    let v = Version::parse("2.0.0");
    assert!(!Version::is_major_upgrade(&v, &v));
}

#[test]
fn is_major_upgrade_false_unparsable_from() {
    let from = Version {
        raw: "abc".into(),
        parsed: None,
    };
    let to = Version::parse("2.0.0");
    assert!(!Version::is_major_upgrade(&from, &to));
}

#[test]
fn is_major_upgrade_false_unparsable_to() {
    let from = Version::parse("1.0.0");
    let to = Version {
        raw: "xyz".into(),
        parsed: None,
    };
    assert!(!Version::is_major_upgrade(&from, &to));
}

#[test]
fn is_major_upgrade_false_both_unparsable() {
    let a = Version {
        raw: "abc".into(),
        parsed: None,
    };
    let b = Version {
        raw: "xyz".into(),
        parsed: None,
    };
    assert!(!Version::is_major_upgrade(&a, &b));
}

// ---------------------------------------------------------------------------
// 7. PackageStatus::determine — each variant
// ---------------------------------------------------------------------------

#[test]
fn status_not_installed() {
    let latest = Version::parse("1.0.0");
    let status = PackageStatus::determine(None, Some(&latest), &VersionFormat::Semver);
    assert_eq!(status, PackageStatus::NotInstalled);
}

#[test]
fn status_not_installed_both_none() {
    let status = PackageStatus::determine(None, None, &VersionFormat::Semver);
    assert_eq!(status, PackageStatus::NotInstalled);
}

#[test]
fn status_unknown_no_catalog() {
    let installed = Version::parse("1.0.0");
    let status = PackageStatus::determine(Some(&installed), None, &VersionFormat::Semver);
    assert_eq!(status, PackageStatus::Unknown);
}

#[test]
fn status_up_to_date() {
    let v = Version::parse("1.2.3");
    let status = PackageStatus::determine(Some(&v), Some(&v), &VersionFormat::Semver);
    assert_eq!(status, PackageStatus::UpToDate);
}

#[test]
fn status_update_available_minor() {
    let installed = Version::parse("1.0.0");
    let latest = Version::parse("1.5.0");
    let status = PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
    assert_eq!(
        status,
        PackageStatus::UpdateAvailable {
            current: installed,
            available: latest,
        }
    );
}

#[test]
fn status_major_upgrade_available() {
    let installed = Version::parse("1.0.0");
    let latest = Version::parse("2.0.0");
    let status = PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Semver);
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
    let catalog = Version::parse("2.5.0");
    let status = PackageStatus::determine(Some(&installed), Some(&catalog), &VersionFormat::Semver);
    assert_eq!(
        status,
        PackageStatus::NewerThanCatalog {
            current: installed,
            catalog_latest: catalog,
        }
    );
}

#[test]
fn status_with_date_format_update_available() {
    let installed = Version::parse("2024.01.15");
    let latest = Version::parse("2025.06.01");
    let status = PackageStatus::determine(Some(&installed), Some(&latest), &VersionFormat::Date);
    // Date format: non-semver, so always UpdateAvailable (never MajorUpgradeAvailable)
    assert!(matches!(status, PackageStatus::UpdateAvailable { .. }));
}

#[test]
fn status_with_custom_format_update_available() {
    let fmt = VersionFormat::Custom {
        pattern: r"(\d+)".to_string(),
    };
    let installed = Version::parse("1");
    let latest = Version::parse("99");
    let status = PackageStatus::determine(Some(&installed), Some(&latest), &fmt);
    // Custom format: non-semver, so always UpdateAvailable
    assert!(matches!(status, PackageStatus::UpdateAvailable { .. }));
}

#[test]
fn status_is_major_upgrade_true_for_major() {
    let status = PackageStatus::MajorUpgradeAvailable {
        current: Version::parse("1.0.0"),
        available: Version::parse("2.0.0"),
    };
    assert!(status.is_major_upgrade());
}

#[test]
fn status_is_major_upgrade_false_for_others() {
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
    for s in &statuses {
        assert!(!s.is_major_upgrade(), "expected false for {s}");
    }
}

// ---------------------------------------------------------------------------
// 8. Snapshot tests — edge case version comparisons
// ---------------------------------------------------------------------------

/// Helper: format a comparison result for snapshot output.
fn cmp_line(a: &str, b: &str, format: &VersionFormat) -> String {
    let result = compare_versions(a, b, format);
    let symbol = match result {
        Ordering::Less => "<",
        Ordering::Equal => "==",
        Ordering::Greater => ">",
    };
    format!("{a:>20} {symbol} {b:<20} (format: {format})")
}

#[test]
fn snapshot_semver_lenient_two_part_vs_three_part() {
    // v3.1 vs 3.1.0 — 2-part should coerce to 3-part
    let lines = [
        cmp_line("v3.1", "3.1.0", &VersionFormat::Semver),
        cmp_line("3.1.0", "v3.1", &VersionFormat::Semver),
        cmp_line("v3.1", "3.2.0", &VersionFormat::Semver),
        cmp_line("3.1", "3.1.0", &VersionFormat::Semver),
    ];
    insta::assert_snapshot!(lines.join("\n"));
}

#[test]
fn snapshot_date_mixed_separators() {
    // 2026.03.29 vs 2026-03-29 — different separators, same date
    let lines = [
        cmp_line("2026.03.29", "2026-03-29", &VersionFormat::Date),
        cmp_line("2026-03-29", "2026.03.29", &VersionFormat::Date),
        cmp_line("2026.03.29", "2026-04-01", &VersionFormat::Date),
        cmp_line("2025-12-31", "2026.01.01", &VersionFormat::Date),
    ];
    insta::assert_snapshot!(lines.join("\n"));
}

#[test]
fn snapshot_semver_space_suffix() {
    // "3.1 HF2" vs "3.1 HF3" — suffix treated as pre-release
    let lines = [
        cmp_line("3.1 HF2", "3.1 HF3", &VersionFormat::Semver),
        cmp_line("3.1 HF3", "3.1 HF2", &VersionFormat::Semver),
        cmp_line("3.1 HF2", "3.1 HF2", &VersionFormat::Semver),
        cmp_line("3.1 HF2", "3.1.0", &VersionFormat::Semver),
        cmp_line("6.6 SP2", "6.6 SP3", &VersionFormat::Semver),
    ];
    insta::assert_snapshot!(lines.join("\n"));
}

#[test]
fn snapshot_four_part_vs_three_part() {
    // 4-part coercion: 3.1.2.3001 vs 3.1.2
    let lines = [
        cmp_line("3.1.2.3001", "3.1.2", &VersionFormat::Semver),
        cmp_line("3.1.2.3001", "3.1.3", &VersionFormat::Semver),
        cmp_line("4.1.12288.0", "4.1.12288", &VersionFormat::Semver),
        cmp_line("3.1.2.3001", "3.1.2.9999", &VersionFormat::Semver),
    ];
    insta::assert_snapshot!(lines.join("\n"));
}

#[test]
fn snapshot_newer_than_catalog() {
    // Installed is newer than catalog latest
    let installed = Version::parse("3.2.0");
    let catalog = Version::parse("3.1.5");
    let status = PackageStatus::determine(Some(&installed), Some(&catalog), &VersionFormat::Semver);
    insta::assert_debug_snapshot!(status);
}

#[test]
fn snapshot_newer_than_catalog_date_format() {
    let installed = Version::parse("2026.04.01");
    let catalog = Version::parse("2026.03.15");
    let status = PackageStatus::determine(Some(&installed), Some(&catalog), &VersionFormat::Date);
    insta::assert_debug_snapshot!(status);
}

#[test]
fn snapshot_format_mismatch_semver_with_date_string() {
    // Semver format given a date-like string — should trigger compatibility warning
    let lines = [
        cmp_line("2026.03.29", "2026.04.01", &VersionFormat::Semver),
        cmp_line("2026.03.29", "3.1.0", &VersionFormat::Semver),
        cmp_line("3.1.0", "2026.03.29", &VersionFormat::Semver),
    ];
    insta::assert_snapshot!(lines.join("\n"));
}

#[test]
fn snapshot_check_format_compatibility_warnings() {
    let cases: Vec<String> = vec![
        // Semver format with non-semver strings
        format!(
            "semver + 'not-a-version': {:?}",
            check_format_compatibility("not-a-version", &VersionFormat::Semver)
        ),
        format!(
            "semver + '1.2.3': {:?}",
            check_format_compatibility("1.2.3", &VersionFormat::Semver)
        ),
        format!(
            "semver + 'v3.1': {:?}",
            check_format_compatibility("v3.1", &VersionFormat::Semver)
        ),
        // Date format with non-date strings
        format!(
            "date + '3.1.0': {:?}",
            check_format_compatibility("3.1.0", &VersionFormat::Date)
        ),
        format!(
            "date + '2026.03.29': {:?}",
            check_format_compatibility("2026.03.29", &VersionFormat::Date)
        ),
        format!(
            "date + '2026-03-29': {:?}",
            check_format_compatibility("2026-03-29", &VersionFormat::Date)
        ),
        format!(
            "date + 'not-a-date': {:?}",
            check_format_compatibility("not-a-date", &VersionFormat::Date)
        ),
        // Custom format with invalid regex
        format!(
            "custom(invalid) + 'anything': {:?}",
            check_format_compatibility(
                "anything",
                &VersionFormat::Custom {
                    pattern: "([invalid".to_string(),
                }
            )
        ),
        // Custom format with non-matching string
        format!(
            "custom(digits) + 'abc': {:?}",
            check_format_compatibility(
                "abc",
                &VersionFormat::Custom {
                    pattern: r"(\d+)\.(\d+)".to_string(),
                }
            )
        ),
        // Custom format with matching string
        format!(
            "custom(digits) + '3.14': {:?}",
            check_format_compatibility(
                "3.14",
                &VersionFormat::Custom {
                    pattern: r"(\d+)\.(\d+)".to_string(),
                }
            )
        ),
    ];
    insta::assert_snapshot!(cases.join("\n"));
}
