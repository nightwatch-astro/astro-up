use clap::Parser;

#[derive(Parser)]
#[command(name = "astro-up", version, about = "Astrophotography software manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Show version information
    Version,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) | None => {
            println!("astro-up {}", astro_up_core::version());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_version_command() {
        let cli = Cli::parse_from(["astro-up", "version"]);
        assert!(matches!(cli.command, Some(Commands::Version)));
    }
}
