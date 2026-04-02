// Contract: CLI Commands — spec 015
// This is a design contract, not compilable code.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

// --- Top-level CLI ---

#[derive(Parser)]
#[command(name = "astro-up", version, about = "Astrophotography software manager")]
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
    Show {
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
    /// Show detailed info for a package (positional, not a subcommand)
    // Note: clap handles this via `show <package>` as a default argument
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Generate default config file
    Init,
    /// Show effective configuration
    Show,
}

// --- Output Mode ---

pub enum OutputMode {
    /// TTY with colors, tables, ratatui TUI
    Interactive,
    /// No colors, no TUI (piped output or --quiet)
    Plain,
    /// Structured JSON to stdout
    Json,
}

impl OutputMode {
    pub fn detect(cli: &Cli) -> Self {
        if cli.json {
            OutputMode::Json
        } else if !std::io::stdout().is_terminal() || cli.quiet {
            OutputMode::Plain
        } else {
            OutputMode::Interactive
        }
    }
}

// --- Command handler signature ---
// Each command handler follows this pattern:

// async fn handle_show(filter: ShowFilter, mode: &OutputMode, services: &AppServices) -> Result<()>;
// async fn handle_install(package: &str, dry_run: bool, yes: bool, mode: &OutputMode, services: &AppServices) -> Result<()>;
// etc.

// AppServices bundles core dependencies:
pub struct AppServices {
    pub catalog: Catalog,       // spec 005
    pub scanner: Scanner,       // spec 006
    pub engine: Orchestrator,   // spec 012
    pub backup: BackupService,  // spec 013
    pub config: AppConfig,      // spec 004
}
