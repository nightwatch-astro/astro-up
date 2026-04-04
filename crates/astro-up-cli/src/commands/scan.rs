use color_eyre::eyre::Result;
use serde::Serialize;
use tabled::Tabled;

use astro_up_core::adapters::{CatalogPackageSource, SqliteLedgerStore};
use astro_up_core::detect::scanner::Scanner;

use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::output::table::print_table;
use crate::state::CliState;

/// Wire scan command to core Scanner (T006).
pub async fn handle_scan(state: &CliState, mode: &OutputMode) -> Result<()> {
    // Ensure catalog is available
    state.open_catalog_reader_ensure().await?;

    let packages = CatalogPackageSource::new(state.catalog_path().to_path_buf());
    let ledger = SqliteLedgerStore::new(state.db_path.clone());
    let scanner = Scanner::new(packages, ledger);

    let scan_result = scanner
        .scan()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("scan failed: {e}"))?;

    // Persist scan results to ledger (FR-004)
    {
        use astro_up_core::detect::scanner::LedgerStore;
        let persist_ledger = SqliteLedgerStore::new(state.db_path.clone());
        for r in &scan_result.results {
            if let astro_up_core::detect::DetectionResult::Installed { version, .. } = &r.result {
                let _ = persist_ledger.upsert_acknowledged(&r.package_id, version);
            }
        }
    }

    use astro_up_core::detect::DetectionResult;

    let rows: Vec<ScanRow> = scan_result
        .results
        .iter()
        .map(|r| match &r.result {
            DetectionResult::Installed {
                version, method, ..
            } => ScanRow {
                package: r.package_id.clone(),
                version: version.to_string(),
                method: format!("{method:?}"),
                status: "Installed".into(),
            },
            DetectionResult::InstalledUnknownVersion { method, .. } => ScanRow {
                package: r.package_id.clone(),
                version: "unknown".into(),
                method: format!("{method:?}"),
                status: "Installed".into(),
            },
            DetectionResult::NotInstalled => ScanRow {
                package: r.package_id.clone(),
                version: "-".into(),
                method: "-".into(),
                status: "Not found".into(),
            },
            DetectionResult::Unavailable { reason } => ScanRow {
                package: r.package_id.clone(),
                version: "-".into(),
                method: "-".into(),
                status: format!("Unavailable: {reason}"),
            },
        })
        .collect();

    if *mode == OutputMode::Json {
        return print_json(&ScanOutput {
            results: rows,
            errors: scan_result
                .errors
                .iter()
                .map(|e| format!("{}: {}", e.package_id, e.error))
                .collect(),
            note: None,
        });
    }

    if !mode.should_print() {
        return Ok(());
    }

    let installed_count = rows.iter().filter(|r| r.status == "Installed").count();

    if rows.is_empty() {
        println!("No packages in catalog.");
    } else {
        print_table(&rows)?;
        println!(
            "\n{} detected, {} not found, {} errors",
            installed_count,
            rows.len() - installed_count,
            scan_result.errors.len()
        );
    }
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
    #[tabled(rename = "Status")]
    status: String,
}
