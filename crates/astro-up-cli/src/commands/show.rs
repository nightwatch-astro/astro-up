use color_eyre::eyre::Result;

use crate::output::OutputMode;

pub async fn handle_show(_filter: Option<super::ShowFilter>, _mode: &OutputMode) -> Result<()> {
    Ok(())
}
