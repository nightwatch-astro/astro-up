use std::time::Duration;

use color_eyre::eyre::{Result, eyre};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use astro_up_core::adapters::{CatalogPackageSource, SqliteLedgerStore};
use astro_up_core::backup::BackupService;
use astro_up_core::catalog::PackageId;
use astro_up_core::download::DownloadManager;
use astro_up_core::engine::orchestrator::{
    EventCallback, Orchestrator, UpdateOrchestrator, UpdateRequest,
};
use astro_up_core::events::Event;
use astro_up_core::install::InstallerService;

use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::state::CliState;

use super::{confirm, forward_events};

/// Update installed packages (T014-T015).
pub async fn handle_update(
    state: &CliState,
    package: Option<&str>,
    all: bool,
    dry_run: bool,
    allow_major: bool,
    yes: bool,
    mode: &OutputMode,
    cancel: CancellationToken,
) -> Result<()> {
    // Determine which packages to update
    let pkg_ids: Vec<PackageId> = if let Some(pkg) = package {
        vec![
            pkg.parse()
                .map_err(|_| eyre!("invalid package id: {pkg}"))?,
        ]
    } else if all {
        // Update all — pass empty list, orchestrator plans for all outdated
        vec![]
    } else {
        if mode.should_print() {
            println!("Specify a package or use --all to update everything.");
        }
        return Ok(());
    };

    state.open_catalog_reader_ensure().await?;

    let (event_tx, rx) = broadcast::channel::<Event>(64);
    let (_bar, event_handle) = forward_events(rx, *mode);

    let catalog_path = state.catalog_path().to_path_buf();
    let packages = CatalogPackageSource::new(catalog_path);
    let ledger = SqliteLedgerStore::new(state.db_path.clone());
    let downloader = DownloadManager::new(&state.config.network, event_tx)
        .map_err(|e| eyre!("failed to create download manager: {e}"))?;
    let installer =
        InstallerService::new(Duration::from_secs(600), state.data_dir.join("installs"));
    let backup_dir = state.data_dir.join("backups");
    let backup = BackupService::new(backup_dir, 5);
    let db = state.open_db()?;
    let lock_path = state.data_dir.join("orchestration.lock");

    let download_dir = if state.config.paths.download_dir.as_os_str().is_empty() {
        std::env::temp_dir().join("astro-up").join("downloads")
    } else {
        state.config.paths.download_dir.clone()
    };
    let orchestrator = UpdateOrchestrator::new(
        &lock_path,
        packages,
        ledger,
        downloader,
        installer,
        backup,
        db,
        download_dir,
    )
    .map_err(|e| eyre!("failed to create orchestrator: {e}"))?;

    let request = UpdateRequest {
        packages: pkg_ids,
        allow_major,
        allow_downgrade: false,
        dry_run,
        confirmed: yes,
    };

    let plan = orchestrator
        .plan(request)
        .await
        .map_err(|e| eyre!("planning failed: {e}"))?;

    // Show plan summary
    let plan_json = serde_json::to_value(&plan).unwrap_or_default();
    if *mode == OutputMode::Json && dry_run {
        return print_json(&serde_json::json!({"plan": plan_json, "dry_run": true}));
    }

    if dry_run {
        if *mode == OutputMode::Json {
            return print_json(&serde_json::json!({"plan": plan_json, "dry_run": true}));
        }
        if mode.should_print() {
            println!("Update plan (dry run):");
            println!(
                "{}",
                serde_json::to_string_pretty(&plan_json).unwrap_or_default()
            );
            println!("\n(dry run — no changes made)");
        }
        return Ok(());
    }

    if !confirm("Proceed with updates?", mode, yes)? {
        if mode.should_print() {
            println!("Cancelled.");
        }
        return Ok(());
    }

    let on_event: EventCallback = Box::new(|_event| {});

    let result = orchestrator
        .execute(plan, on_event, None, cancel)
        .await
        .map_err(|e| eyre!("update failed: {e}"))?;

    drop(event_handle);

    if *mode == OutputMode::Json {
        return print_json(&result);
    }

    if mode.should_print() {
        println!("Update complete.");
    }
    Ok(())
}
