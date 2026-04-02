use color_eyre::eyre::Result;
use tokio_util::sync::CancellationToken;

use crate::output::OutputMode;

pub async fn handle_update(
    _package: Option<&str>,
    _all: bool,
    _dry_run: bool,
    _allow_major: bool,
    _yes: bool,
    _mode: &OutputMode,
    _cancel: CancellationToken,
) -> Result<()> {
    Ok(())
}
