use color_eyre::eyre::Result;

use crate::output::OutputMode;

pub async fn handle_backup(_package: &str, _mode: &OutputMode) -> Result<()> {
    Ok(())
}
