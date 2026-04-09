pub mod backup;
pub mod catalog;
pub mod config;
pub mod install;
pub mod lifecycle_test;
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
use indicatif::ProgressBar;
use tokio::sync::broadcast;

use astro_up_core::events::Event;

use crate::output::OutputMode;
use crate::output::progress::{create_download_bar, render_event};

/// Prompt user for confirmation. Returns `true` immediately in JSON/Quiet mode or if `--yes`.
pub fn confirm(prompt: &str, mode: &OutputMode, yes: bool) -> Result<bool> {
    if yes || *mode == OutputMode::Json || *mode == OutputMode::Quiet {
        return Ok(true);
    }
    Ok(Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()?)
}

/// Spawn a background task that forwards core events to the progress renderer.
/// Returns the download progress bar (for Interactive mode) and a `JoinHandle`.
pub fn forward_events(
    mut rx: broadcast::Receiver<Event>,
    mode: OutputMode,
) -> (Option<ProgressBar>, tokio::task::JoinHandle<()>) {
    let download_bar = if mode == OutputMode::Interactive {
        Some(create_download_bar(0))
    } else {
        None
    };
    let bar_clone = download_bar.clone();

    let handle = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            render_event(&event, &mode, bar_clone.as_ref());
        }
    });

    (download_bar, handle)
}
