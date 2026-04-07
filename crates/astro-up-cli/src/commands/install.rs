use std::time::Duration;

use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use tabled::Tabled;
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
use crate::output::table::print_table;
use crate::state::CliState;

use super::{confirm, forward_events};

/// Install a package from the catalog (T010-T012).
pub async fn handle_install(
    state: &CliState,
    package: &str,
    dry_run: bool,
    yes: bool,
    mode: &OutputMode,
    cancel: CancellationToken,
) -> Result<()> {
    let reader = state.open_catalog_reader_ensure().await?;

    let id: PackageId = package
        .parse()
        .map_err(|_| eyre!("invalid package id: {package}"))?;

    let Ok(pkg) = reader.resolve(&id) else {
        let results = reader.search(package)?;
        if results.is_empty() {
            return Err(eyre!("package '{package}' not found in catalog"));
        }
        if mode.should_print() {
            println!("Package '{package}' not found. Did you mean:");
            for r in results.iter().take(5) {
                println!("  {} ({})", r.package.id, r.package.name);
            }
        }
        return Ok(());
    };

    let latest = reader
        .latest_version(&id)?
        .ok_or_else(|| eyre!("no versions available for '{package}'"))?;

    let plan = InstallPlan {
        package_id: pkg.id.to_string(),
        name: pkg.name.clone(),
        version: latest.version.clone(),
        category: pkg.category.to_string(),
    };

    if *mode == OutputMode::Json && dry_run {
        return print_json(&serde_json::json!({"plan": plan, "dry_run": true}));
    }

    if mode.should_print() {
        print_table(&[&plan])?;
    }

    if dry_run {
        if mode.should_print() {
            println!("\n(dry run — no changes made)");
        }
        return Ok(());
    }

    if !confirm("Proceed with install?", mode, yes)? {
        if mode.should_print() {
            println!("Cancelled.");
        }
        return Ok(());
    }

    // Set up event channel and progress renderer
    let (event_tx, rx) = broadcast::channel::<Event>(64);
    let (_bar, event_handle) = forward_events(rx, *mode);

    // Create orchestrator (same pattern as GUI's run_orchestrated_operation)
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

    let pkg_ids = vec![id];
    let request = UpdateRequest {
        packages: pkg_ids,
        allow_major: false,
        allow_downgrade: false,
        dry_run: false,
        confirmed: true,
    };

    let orch_plan = orchestrator
        .plan(request)
        .await
        .map_err(|e| eyre!("planning failed: {e}"))?;

    let on_event: EventCallback = Box::new(|_event| {
        // Events already forwarded via broadcast channel
    });

    // Asset selector: prompt user when multiple download options exist
    let asset_selector: astro_up_core::engine::orchestrator::AssetSelector =
        Box::new(|package_name, assets| {
            if assets.len() <= 1 {
                return Some(0);
            }
            let items: Vec<String> = assets
                .iter()
                .map(|a| format!("{} ({:.1} MB)", a.name, a.size as f64 / 1024.0 / 1024.0))
                .collect();
            println!("Multiple download options for {package_name}:");
            dialoguer::Select::new()
                .with_prompt("Choose variant")
                .items(&items)
                .default(0)
                .interact_opt()
                .ok()
                .flatten()
        });

    let result = orchestrator
        .execute(orch_plan, on_event, Some(asset_selector), cancel)
        .await
        .map_err(|e| eyre!("install failed: {e}"))?;

    // Drop the event handle to let the forwarder finish
    drop(event_handle);

    // Post-install verification: re-run detection for this package (T012)
    let verify_packages = CatalogPackageSource::new(state.catalog_path().to_path_buf());
    let verify_ledger = SqliteLedgerStore::new(state.db_path.clone());
    let scanner = astro_up_core::detect::scanner::Scanner::new(verify_packages, verify_ledger);
    let verify_result = scanner.scan().await;

    let verified = verify_result
        .as_ref()
        .ok()
        .and_then(|sr| sr.results.iter().find(|r| r.package_id == package))
        .is_some_and(|r| r.result.is_installed());

    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({
            "plan": plan,
            "result": result,
            "verified": verified,
        }));
    }

    if mode.should_print() {
        if verified {
            println!(
                "Install complete: {} {} (verified)",
                pkg.name, latest.version
            );
        } else {
            println!(
                "Install complete: {} {} (detection could not verify installation)",
                pkg.name, latest.version
            );
        }
    }
    Ok(())
}

#[derive(Tabled, Serialize)]
struct InstallPlan {
    #[tabled(rename = "Package")]
    package_id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Category")]
    category: String,
}
