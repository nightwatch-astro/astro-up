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

use color_eyre::eyre::Result;
use dialoguer::Confirm;

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
