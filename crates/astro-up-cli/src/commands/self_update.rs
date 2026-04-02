use color_eyre::eyre::Result;

use crate::output::OutputMode;

pub async fn handle_self_update(_dry_run: bool, _mode: &OutputMode) -> Result<()> {
    Ok(())
}
