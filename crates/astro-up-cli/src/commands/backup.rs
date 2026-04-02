use color_eyre::eyre::{Result, eyre};

use crate::output::OutputMode;
use crate::output::json::print_json;

/// T027: Create a backup for a package's configuration.
pub async fn handle_backup(package: &str, mode: &OutputMode) -> Result<()> {
    let data_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up")
        .map(|dirs| dirs.data_dir().to_owned())
        .ok_or_else(|| eyre!("could not determine data directory"))?;

    let backup_dir = data_dir.join("backups");
    std::fs::create_dir_all(&backup_dir)?;

    // BackupService needs config_paths from the catalog's backup config.
    // For now, we inform the user that backup requires the package to have
    // a backup configuration in the catalog manifest.
    if *mode == OutputMode::Json {
        return print_json(
            &serde_json::json!({"package": package, "status": "backup_not_configured"}),
        );
    }

    println!("Backup for '{package}' requires a backup configuration in the catalog.");
    println!("Backup directory: {}", backup_dir.display());
    Ok(())
}
