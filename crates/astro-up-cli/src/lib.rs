pub mod commands;
pub mod logging;
pub mod output;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use tokio_util::sync::CancellationToken;

use crate::output::OutputMode;

#[derive(Parser)]
#[command(
    name = "astro-up",
    version,
    about = "Astrophotography software manager"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format: JSON for scripting
    #[arg(long, global = true)]
    pub json: bool,

    /// Increase verbosity (show debug output)
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show software status
    #[command(args_conflicts_with_subcommands = true)]
    Show {
        /// Show details for a specific package
        package: Option<String>,

        #[command(subcommand)]
        filter: Option<ShowFilter>,
    },

    /// Install a package
    Install {
        package: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short)]
        yes: bool,
    },

    /// Update installed packages
    Update {
        package: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        allow_major: bool,
        #[arg(long, short)]
        yes: bool,
    },

    /// Scan for installed software
    Scan,

    /// Search the catalog
    Search { query: String },

    /// Create a backup
    Backup { package: String },

    /// Restore from a backup
    Restore {
        package: String,
        #[arg(long)]
        path: Option<String>,
        #[arg(long, short)]
        yes: bool,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Update astro-up itself
    SelfUpdate {
        #[arg(long)]
        dry_run: bool,
    },

    /// Run lifecycle test for a package (download, install, detect, uninstall)
    LifecycleTest {
        /// Package ID from the manifests repo
        package: String,
        /// Path to the manifests repo checkout
        #[arg(long)]
        manifest_path: PathBuf,
        /// Specific version to test (default: latest)
        #[arg(long)]
        version: Option<String>,
        /// Install directory for download_only packages
        #[arg(long)]
        install_dir: Option<PathBuf>,
        /// Download and probe only, skip install/uninstall
        #[arg(long)]
        dry_run: bool,
        /// Write JSON report to file
        #[arg(long)]
        report_file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum ShowFilter {
    /// Show all catalog packages (default)
    All,
    /// Show only installed packages
    Installed,
    /// Show packages with available updates
    Outdated,
    /// Show backups
    Backups { package: Option<String> },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Generate default config file
    Init,
    /// Show effective configuration
    Show,
}

pub async fn run(cli: Cli, cancel: CancellationToken) -> Result<()> {
    let mode = OutputMode::detect(cli.json, cli.quiet);

    match cli.command {
        Commands::Show { package, filter } => {
            if let Some(ref pkg) = package {
                let reader = commands::ensure_catalog().await?;
                commands::show::handle_show_detail(&reader, pkg, &mode)
            } else {
                commands::show::handle_show(filter, &mode).await
            }
        }
        Commands::Install {
            ref package,
            dry_run,
            yes,
        } => commands::install::handle_install(package, dry_run, yes, &mode, cancel).await,
        Commands::Update {
            ref package,
            all,
            dry_run,
            allow_major,
            yes,
        } => {
            commands::update::handle_update(
                package.as_deref(),
                all,
                dry_run,
                allow_major,
                yes,
                &mode,
                cancel,
            )
            .await
        }
        Commands::Scan => commands::scan::handle_scan(&mode).await,
        Commands::Search { ref query } => commands::search::handle_search(query, &mode).await,
        Commands::Backup { ref package } => commands::backup::handle_backup(package, &mode).await,
        Commands::Restore {
            ref package,
            ref path,
            yes,
        } => commands::restore::handle_restore(package, path.as_deref(), yes, &mode).await,
        Commands::Config { action } => commands::config::handle_config(action, &mode).await,
        Commands::SelfUpdate { dry_run } => {
            commands::self_update::handle_self_update(dry_run, &mode).await
        }
        Commands::LifecycleTest {
            ref package,
            ref manifest_path,
            ref version,
            ref install_dir,
            dry_run,
            ref report_file,
        } => {
            commands::lifecycle_test::handle_lifecycle_test(
                package,
                manifest_path,
                version.as_deref(),
                install_dir.as_deref(),
                dry_run,
                report_file.as_deref(),
                &mode,
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_show_command() {
        let cli = Cli::parse_from(["astro-up", "show"]);
        assert!(matches!(
            cli.command,
            Commands::Show {
                package: None,
                filter: None
            }
        ));
    }

    #[test]
    fn cli_parses_show_package_detail() {
        let cli = Cli::parse_from(["astro-up", "show", "nina"]);
        assert!(
            matches!(cli.command, Commands::Show { package: Some(ref p), filter: None } if p == "nina")
        );
    }

    #[test]
    fn cli_parses_install_command() {
        let cli = Cli::parse_from(["astro-up", "install", "nina"]);
        assert!(matches!(cli.command, Commands::Install { ref package, .. } if package == "nina"));
    }

    #[test]
    fn cli_parses_global_json_flag() {
        let cli = Cli::parse_from(["astro-up", "--json", "scan"]);
        assert!(cli.json);
    }

    #[test]
    fn cli_parses_config_subcommands() {
        let cli = Cli::parse_from(["astro-up", "config", "init"]);
        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Init
            }
        ));
    }
}
