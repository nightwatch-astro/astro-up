use color_eyre::eyre::Result;
use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    serde_json::to_writer_pretty(std::io::stdout().lock(), value)?;
    println!();
    Ok(())
}
