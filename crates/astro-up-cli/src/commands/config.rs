use color_eyre::eyre::Result;

use crate::output::OutputMode;

pub async fn handle_config(_action: super::ConfigAction, _mode: &OutputMode) -> Result<()> {
    Ok(())
}
