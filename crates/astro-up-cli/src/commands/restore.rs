use color_eyre::eyre::Result;

use crate::output::OutputMode;

pub async fn handle_restore(
    _package: &str,
    _path: Option<&str>,
    _yes: bool,
    _mode: &OutputMode,
) -> Result<()> {
    Ok(())
}
