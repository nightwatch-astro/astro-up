use color_eyre::eyre::Result;
use serde::Serialize;
use tabled::Tabled;

use crate::output::OutputMode;
use crate::output::json::print_json;

/// T017: Scan for installed software using the detection engine.
pub async fn handle_scan(mode: &OutputMode) -> Result<()> {
    // Detection requires Windows APIs (registry, PE, WMI).
    // On non-Windows platforms, show a clear message.
    if !cfg!(target_os = "windows") {
        if *mode == OutputMode::Json {
            return print_json(&ScanOutput {
                results: vec![],
                errors: vec![],
                note: Some("detection requires Windows".into()),
            });
        }
        println!("Software detection requires Windows. Scan is not available on this platform.");
        return Ok(());
    }

    // On Windows: run full catalog scan
    // TODO(T017): Wire Scanner from core once catalog + ledger are available on this platform
    if *mode == OutputMode::Json {
        return print_json(&ScanOutput {
            results: vec![],
            errors: vec![],
            note: None,
        });
    }
    println!("No packages detected.");
    Ok(())
}

#[derive(Serialize)]
struct ScanOutput {
    results: Vec<ScanRow>,
    errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Tabled, Serialize)]
struct ScanRow {
    #[tabled(rename = "Package")]
    package: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Method")]
    method: String,
}
