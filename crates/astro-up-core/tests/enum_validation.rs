#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::str::FromStr;

use astro_up_core::types::{
    Category, CheckMethod, DetectionMethod, InstallMethod, KnownExitCode, SoftwareType,
};
use rstest::rstest;
use strum::IntoEnumIterator;

// T026: Parameterized Category tests
#[rstest]
#[case("capture", Category::Capture)]
#[case("guiding", Category::Guiding)]
#[case("platesolving", Category::Platesolving)]
#[case("equipment", Category::Equipment)]
#[case("focusing", Category::Focusing)]
#[case("planetarium", Category::Planetarium)]
#[case("viewers", Category::Viewers)]
#[case("prerequisites", Category::Prerequisites)]
#[case("usb", Category::Usb)]
#[case("driver", Category::Driver)]
fn category_from_str(#[case] input: &str, #[case] expected: Category) {
    let parsed = Category::from_str(input).unwrap();
    assert_eq!(parsed, expected);
    assert_eq!(parsed.to_string(), input);
}

#[test]
fn category_invalid_string_fails() {
    let result = Category::from_str("astronomy");
    assert!(result.is_err());
}

#[test]
fn category_serde_round_trip() {
    for cat in Category::iter() {
        let json = serde_json::to_string(&cat).unwrap();
        let back: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(cat, back);
    }
}

// T027: Parameterized tests for other enums
#[rstest]
#[case("registry", DetectionMethod::Registry)]
#[case("pe_file", DetectionMethod::PeFile)]
#[case("wmi", DetectionMethod::Wmi)]
#[case("driver_store", DetectionMethod::DriverStore)]
#[case("ascom_profile", DetectionMethod::AscomProfile)]
#[case("file_exists", DetectionMethod::FileExists)]
#[case("config_file", DetectionMethod::ConfigFile)]
fn detection_method_round_trip(#[case] input: &str, #[case] expected: DetectionMethod) {
    let parsed = DetectionMethod::from_str(input).unwrap();
    assert_eq!(parsed, expected);
    assert_eq!(parsed.to_string(), input);
}

#[rstest]
#[case("exe", InstallMethod::Exe)]
#[case("msi", InstallMethod::Msi)]
#[case("inno_setup", InstallMethod::InnoSetup)]
#[case("nullsoft", InstallMethod::Nullsoft)]
#[case("zip", InstallMethod::Zip)]
#[case("zip_wrap", InstallMethod::ZipWrap)]
#[case("portable", InstallMethod::Portable)]
#[case("download_only", InstallMethod::DownloadOnly)]
fn install_method_round_trip(#[case] input: &str, #[case] expected: InstallMethod) {
    let parsed = InstallMethod::from_str(input).unwrap();
    assert_eq!(parsed, expected);
    assert_eq!(parsed.to_string(), input);
}

#[rstest]
#[case("github", CheckMethod::Github)]
#[case("gitlab", CheckMethod::Gitlab)]
#[case("direct_url", CheckMethod::DirectUrl)]
#[case("http_head", CheckMethod::HttpHead)]
#[case("html_scrape", CheckMethod::HtmlScrape)]
#[case("browser_scrape", CheckMethod::BrowserScrape)]
#[case("pe_download", CheckMethod::PeDownload)]
#[case("manual", CheckMethod::Manual)]
#[case("runtime_scrape", CheckMethod::RuntimeScrape)]
fn check_method_round_trip(#[case] input: &str, #[case] expected: CheckMethod) {
    let parsed = CheckMethod::from_str(input).unwrap();
    assert_eq!(parsed, expected);
    assert_eq!(parsed.to_string(), input);
}

// T028: EnumIter test for Category
#[test]
fn category_iter_returns_all_10_variants() {
    let variants: Vec<Category> = Category::iter().collect();
    assert_eq!(variants.len(), 10);
    assert_eq!(variants[0], Category::Capture);
    assert_eq!(variants[9], Category::Driver);
}

#[test]
fn software_type_iter_returns_all_6_variants() {
    assert_eq!(SoftwareType::iter().count(), 6);
}

#[test]
fn known_exit_code_iter_returns_all_12_variants() {
    assert_eq!(KnownExitCode::iter().count(), 12);
}
