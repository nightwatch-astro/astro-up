use std::path::PathBuf;

use color_eyre::eyre::{Result, eyre};
use tokio::sync::broadcast;

use astro_up_core::backup::archive::create_backup;
use astro_up_core::backup::types::BackupRequest;
use astro_up_core::catalog::PackageId;
use astro_up_core::events::Event;
use astro_up_core::types::Version;

use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::state::CliState;

/// Create a backup for a package's configuration (T020-T021).
pub async fn handle_backup(state: &CliState, package: &str, mode: &OutputMode) -> Result<()> {
    tracing::debug!(package, "entering handle_backup");
    let reader = state.open_catalog_reader_ensure().await?;

    let _id: PackageId = package
        .parse()
        .map_err(|_| eyre!("invalid package id: {package}"))?;

    // Need full Software struct (not PackageSummary) to access backup config
    let all_software = reader
        .list_all_with_detection()
        .map_err(|e| eyre!("failed to read catalog: {e}"))?;
    let pkg = all_software
        .iter()
        .find(|s| s.id.as_ref() == package)
        .ok_or_else(|| eyre!("package '{package}' not found in catalog"))?;

    let backup_config = match pkg.backup.as_ref() {
        Some(cfg) if !cfg.config_paths.is_empty() => cfg,
        _ => {
            if *mode == OutputMode::Json {
                return print_json(&serde_json::json!({
                    "package": package,
                    "status": "no_config_paths",
                }));
            }
            if mode.should_print() {
                println!("No backup paths configured for '{package}'.");
            }
            return Ok(());
        }
    };

    // Determine version — fall back to "unknown" if not detected
    let version = Version::parse("0.0.0");

    let (event_tx, _rx) = broadcast::channel::<Event>(16);

    let config_paths: Vec<PathBuf> = backup_config
        .config_paths
        .iter()
        .map(PathBuf::from)
        .collect();

    let request = BackupRequest {
        package_id: package.to_string(),
        version,
        config_paths,
        event_tx,
    };

    let backup_dir = state.data_dir.join("backups");
    let metadata = create_backup(&request, &backup_dir)
        .await
        .map_err(|e| eyre!("backup failed: {e}"))?;

    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({
            "package": package,
            "status": "created",
            "file_count": metadata.file_count,
            "total_size": metadata.total_size,
        }));
    }

    if mode.should_print() {
        println!(
            "Backup created for '{}': {} files, {} bytes",
            package, metadata.file_count, metadata.total_size
        );
    }
    tracing::debug!(
        package,
        file_count = metadata.file_count,
        "exiting handle_backup"
    );
    Ok(())
}
