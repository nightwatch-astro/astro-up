use color_eyre::eyre::{Result, eyre};

use crate::CatalogAction;
use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::state::CliState;

pub async fn handle_catalog(
    state: &CliState,
    action: CatalogAction,
    mode: &OutputMode,
) -> Result<()> {
    tracing::debug!(?action, "entering handle_catalog");

    let result = match action {
        CatalogAction::Sync => state.catalog_manager.ensure_catalog().await,
        CatalogAction::Refresh => state.catalog_manager.refresh().await,
    };

    let fetch_result = result.map_err(|e| eyre!("catalog operation failed: {e}"))?;

    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({
            "result": format!("{fetch_result:?}"),
            "path": state.catalog_path().display().to_string(),
        }));
    }

    if mode.should_print() {
        let reader = state.open_catalog_reader()?;
        let count = reader.list_all()?.len();
        match action {
            CatalogAction::Sync => println!("Catalog synced ({fetch_result:?}). {count} packages."),
            CatalogAction::Refresh => {
                println!("Catalog refreshed ({fetch_result:?}). {count} packages.");
            }
        }
    }

    Ok(())
}
