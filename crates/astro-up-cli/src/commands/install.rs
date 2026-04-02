use color_eyre::eyre::Result;
use tokio_util::sync::CancellationToken;

use crate::output::OutputMode;

pub async fn handle_install(
    _package: &str,
    _dry_run: bool,
    _yes: bool,
    _mode: &OutputMode,
    _cancel: CancellationToken,
) -> Result<()> {
    Ok(())
}
