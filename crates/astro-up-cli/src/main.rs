use std::process::ExitCode;

use clap::Parser;
use tokio_util::sync::CancellationToken;

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> ExitCode {
    if cfg!(not(target_os = "windows")) {
        eprintln!(
            "astro-up requires Windows. Astrophotography software detection, installation, and management depend on Windows APIs (registry, PE headers, WMI)."
        );
        return ExitCode::from(1);
    }

    // Clean up leftover .old binary from a previous self-update
    astro_up_cli::commands::self_update::cleanup_old_binary();

    #[cfg(not(debug_assertions))]
    human_panic::setup_panic!();

    if let Err(e) = color_eyre::install() {
        eprintln!("error: failed to install error handler: {e}");
        return ExitCode::from(1);
    }

    let cli = astro_up_cli::Cli::parse();

    let data_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up").map_or_else(
        || std::path::PathBuf::from("."),
        |dirs| dirs.data_dir().to_path_buf(),
    );
    let log_dir = data_dir.join("logs");

    // Load log config early for max_age_days (best-effort)
    let max_age_days = {
        let db_path = data_dir.join("astro-up.db");
        let paths = astro_up_core::config::PathsConfig {
            data_dir: data_dir.clone(),
            ..astro_up_core::config::PathsConfig::default()
        };
        astro_up_core::config::load_config(&db_path, paths, log_dir.join("astro-up.log"), &[])
            .map(|c| c.logging.max_age_days)
            .unwrap_or(365)
    };

    let _log_guard =
        match astro_up_cli::logging::init(cli.verbose, cli.quiet, &log_dir, max_age_days) {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("error: failed to initialize logging: {e}");
                return ExitCode::from(1);
            }
        };

    tracing::info!(version = astro_up_core::version(), "starting astro-up");

    let subcommand = match &cli.command {
        astro_up_cli::Commands::Show { .. } => "show",
        astro_up_cli::Commands::Install { .. } => "install",
        astro_up_cli::Commands::Update { .. } => "update",
        astro_up_cli::Commands::Scan => "scan",
        astro_up_cli::Commands::Search { .. } => "search",
        astro_up_cli::Commands::Backup { .. } => "backup",
        astro_up_cli::Commands::Restore { .. } => "restore",
        astro_up_cli::Commands::Config { .. } => "config",
        astro_up_cli::Commands::SelfUpdate { .. } => "self-update",
        astro_up_cli::Commands::LifecycleTest { .. } => "lifecycle-test",
    };
    tracing::info!(command = subcommand, "dispatching command");

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancel_clone.cancel();
        }
    });

    let result = astro_up_cli::run(cli, cancel.clone()).await;

    match result {
        Ok(()) => {
            if cancel.is_cancelled() {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            if cancel.is_cancelled() {
                ExitCode::from(2)
            } else {
                eprintln!("{e:?}");
                eprintln!("\nLog directory: {}", log_dir.display());
                ExitCode::from(1)
            }
        }
    }
}
