#![allow(clippy::unwrap_used, clippy::expect_used)]

use astro_up_core::types::Software;

const NINA_TOML: &str = r#"
id = "nina-app"
name = "N.I.N.A."
type = "application"
category = "capture"
publisher = "Stefan Berg (isbeorn)"
description = "Nighttime Imaging 'N' Astronomy — advanced capture sequencer"
homepage = "https://nighttime-imaging.eu"
license = "MPL-2.0"
aliases = ["nina"]
tags = ["sequencer", "dso", "ascom-compatible", "plate-solving", "autofocus"]
notes = "Requires .NET Desktop Runtime 8 and ASCOM Platform"
channel = "stable"
min_os_version = "10.0"

[dependencies]
requires = [
    { id = "ascom-platform", min_version = "6.6" },
    { id = "dotnet-desktop-8" },
]
optional = ["phd2-guider"]

[detection]
method = "registry"
registry_key = "SOFTWARE\\NINA"
registry_value = "DisplayVersion"

[detection.fallback]
method = "pe_file"
file_path = "{program_dir}/NINA/NINA.exe"

[install]
method = "inno_setup"
scope = "either"
elevation = "self"
upgrade_behavior = "install"
install_modes = ["interactive", "silent"]
success_codes = [3010]

[install.switches]
silent = ["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"]
interactive = ["/NORESTART"]
upgrade = ["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"]
install_location = "/DIR=<INSTALLPATH>"
log = "/LOG=<LOGPATH>"

[checkver]
github = "isbeorn/nina"
asset_pattern = "NINASetupBundle_*.zip"
tag_prefix = "Version-"
changelog_url = "https://github.com/isbeorn/nina/releases"

[backup]
config_paths = [
    "{config_dir}/NINA/Profiles",
    "{config_dir}/NINA/Settings",
]
"#;

const MINIMAL_TOML: &str = r#"
id = "test-app"
name = "Test App"
type = "application"
category = "capture"
"#;

#[test]
fn nina_manifest_deserializes() {
    let software: Software = toml::from_str(NINA_TOML).expect("NINA manifest should deserialize");
    assert_eq!(software.id.as_ref(), "nina-app");
    assert_eq!(software.name, "N.I.N.A.");
    assert_eq!(software.category.to_string(), "capture");
    assert_eq!(software.software_type.to_string(), "application");
    assert!(software.detection.is_some());
    assert!(software.install.is_some());
    assert!(software.checkver.is_some());
    assert!(software.backup.is_some());
    assert!(software.hardware.is_none());

    // Verify fallback chain
    let detection = software.detection.as_ref().unwrap();
    assert!(detection.fallback.is_some());
    let fallback = detection.fallback.as_ref().unwrap();
    assert_eq!(fallback.method.to_string(), "pe_file");
}

#[test]
fn nina_json_round_trip() {
    let software: Software = toml::from_str(NINA_TOML).expect("deserialize");
    let json = serde_json::to_string_pretty(&software).expect("serialize to JSON");
    let back: Software = serde_json::from_str(&json).expect("deserialize from JSON");
    assert_eq!(software, back);
}

#[test]
fn nina_snapshot() {
    let software: Software = toml::from_str(NINA_TOML).expect("deserialize");
    insta::assert_json_snapshot!(software);
}

#[test]
fn minimal_manifest_deserializes() {
    let software: Software =
        toml::from_str(MINIMAL_TOML).expect("minimal manifest should deserialize");
    assert_eq!(software.id.as_ref(), "test-app");
    assert!(software.detection.is_none());
    assert!(software.install.is_none());
    assert!(software.checkver.is_none());
    assert!(software.dependencies.is_none());
    assert!(software.hardware.is_none());
    assert!(software.backup.is_none());
}

#[test]
fn minimal_snapshot() {
    let software: Software = toml::from_str(MINIMAL_TOML).expect("deserialize");
    insta::assert_json_snapshot!(software);
}
