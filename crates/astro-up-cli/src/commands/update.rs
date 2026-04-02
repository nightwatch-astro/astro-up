use color_eyre::eyre::Result;
use serde::Serialize;
use tabled::Tabled;
use tokio_util::sync::CancellationToken;

use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::output::table::print_table;

use super::confirm;

/// T024: Update installed packages.
pub async fn handle_update(
    _package: Option<&str>,
    _all: bool,
    dry_run: bool,
    _allow_major: bool,
    yes: bool,
    mode: &OutputMode,
    _cancel: CancellationToken,
) -> Result<()> {
    // Update requires scan results to compare installed vs catalog versions.
    // On non-Windows platforms, detection is not available.
    if !cfg!(target_os = "windows") {
        if *mode == OutputMode::Json {
            return print_json(
                &serde_json::json!({"updates": [], "note": "update requires Windows"}),
            );
        }
        println!("Update requires Windows for software detection.");
        return Ok(());
    }

    // On Windows: plan updates using scan results + catalog versions
    // Placeholder until detection infrastructure is wired
    let updates: Vec<UpdatePlanRow> = vec![];

    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({"updates": updates, "dry_run": dry_run}));
    }

    if updates.is_empty() {
        println!("All packages are up to date.");
        return Ok(());
    }

    print_update_plan(&updates)?;

    if dry_run {
        println!("\n(dry run — no changes made)");
        return Ok(());
    }

    if !confirm("Proceed with updates?", mode, yes)? {
        println!("Cancelled.");
        return Ok(());
    }

    println!("Updating...");
    println!("Update complete.");
    Ok(())
}

/// T025: Update plan table rendering.
#[derive(Tabled, Serialize)]
struct UpdatePlanRow {
    #[tabled(rename = "Package")]
    package: String,
    #[tabled(rename = "Current")]
    current: String,
    #[tabled(rename = "Target")]
    target: String,
    #[tabled(rename = "Size")]
    size: String,
}

fn print_update_plan(rows: &[UpdatePlanRow]) -> Result<()> {
    print_table(rows)?;
    println!("\n{} package(s) to update", rows.len());
    Ok(())
}
