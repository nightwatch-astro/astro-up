use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use tabled::Tabled;

use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::output::table::print_table;

use super::confirm;

use crate::state::CliState;

/// T028: Restore from a backup archive.
pub async fn handle_restore(
    _state: &CliState,
    package: &str,
    _path: Option<&str>,
    yes: bool,
    mode: &OutputMode,
) -> Result<()> {
    tracing::debug!(package, yes, "entering handle_restore");
    let data_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up")
        .map(|dirs| dirs.data_dir().to_owned())
        .ok_or_else(|| eyre!("could not determine data directory"))?;

    let backup_dir = data_dir.join("backups");
    let service = astro_up_core::backup::BackupService::new(backup_dir, 0);

    let entries = service.list(package).await?;

    if entries.is_empty() {
        if *mode == OutputMode::Json {
            return print_json(&serde_json::json!({"package": package, "backups": []}));
        }
        if mode.should_print() {
            println!("No backups found for '{package}'.");
        }
        return Ok(());
    }

    if *mode == OutputMode::Json {
        return print_json(&entries);
    }

    // Show available backups
    let rows: Vec<BackupRow> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| BackupRow {
            index: format!("{}", i + 1),
            version: e.version.raw.clone(),
            date: e.created_at.format("%Y-%m-%d %H:%M").to_string(),
            files: e.file_count,
        })
        .collect();

    print_table(&rows)?;

    // Auto-select latest if --yes
    let selected = if yes {
        &entries[0]
    } else {
        // Interactive: use dialoguer to pick
        let selection = dialoguer::Select::new()
            .with_prompt("Select backup to restore")
            .items(
                &entries
                    .iter()
                    .map(|e| {
                        format!(
                            "{} ({})",
                            e.version.raw,
                            e.created_at.format("%Y-%m-%d %H:%M")
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .default(0)
            .interact()?;
        &entries[selection]
    };

    // Show restore preview
    let preview = service.restore_preview(&selected.archive_path).await?;
    println!(
        "\nRestore preview: {} overwritten, {} new, {} unchanged",
        preview.overwritten.len(),
        preview.new_files.len(),
        preview.unchanged.len()
    );

    if !confirm("Proceed with restore?", mode, yes)? {
        println!("Cancelled.");
        return Ok(());
    }

    let request = astro_up_core::backup::types::RestoreRequest {
        archive_path: selected.archive_path.clone(),
        path_filter: _path.map(String::from),
        current_version: None,
        event_tx: tokio::sync::broadcast::channel(16).0,
    };

    service.restore(&request).await?;
    println!("Restored {} v{}", package, selected.version.raw);
    tracing::debug!(package, version = %selected.version.raw, "exiting handle_restore");
    Ok(())
}

#[derive(Tabled, Serialize)]
struct BackupRow {
    #[tabled(rename = "#")]
    index: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Files")]
    files: u32,
}
