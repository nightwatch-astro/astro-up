pub mod backup;
pub mod config;
pub mod install;
pub mod restore;
pub mod scan;
pub mod search;
pub mod self_update;
pub mod show;
pub mod update;

// Re-export subcommand enums used by command handlers.
pub use crate::{ConfigAction, ShowFilter};

use color_eyre::eyre::{Result, eyre};
use dialoguer::Confirm;

use astro_up_core::catalog::{CatalogManager, SqliteCatalogReader};
use astro_up_core::config::CatalogConfig;

use crate::output::OutputMode;

/// Prompt user for confirmation. Returns `true` immediately in JSON mode or if `--yes`.
pub fn confirm(prompt: &str, mode: &OutputMode, yes: bool) -> Result<bool> {
    if yes || *mode == OutputMode::Json {
        return Ok(true);
    }
    Ok(Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()?)
}

/// T015: Ensure catalog is available, downloading if needed.
/// Returns a ready-to-use catalog reader.
pub async fn ensure_catalog() -> Result<SqliteCatalogReader> {
    let data_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up")
        .map(|dirs| dirs.data_dir().to_owned())
        .ok_or_else(|| eyre!("could not determine data directory"))?;

    std::fs::create_dir_all(&data_dir)?;

    let config = CatalogConfig::default();
    let manager = CatalogManager::new(&data_dir, config);

    let result = manager.ensure_catalog().await?;
    tracing::info!(?result, "catalog status");

    let reader = manager.open_reader()?;
    Ok(reader)
}
