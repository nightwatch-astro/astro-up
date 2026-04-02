use std::process::ExitCode;

use clap::Parser;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> ExitCode {
    #[cfg(not(debug_assertions))]
    human_panic::setup_panic!();

    if let Err(e) = color_eyre::install() {
        eprintln!("error: failed to install error handler: {e}");
        return ExitCode::from(1);
    }

    let cli = astro_up_cli::Cli::parse();

    let log_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up")
        .map(|dirs| dirs.data_dir().join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let _log_guard = match astro_up_cli::logging::init(cli.verbose, &log_dir) {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("error: failed to initialize logging: {e}");
            return ExitCode::from(1);
        }
    };

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");
        cancel_clone.cancel();
    });

    match astro_up_cli::run(cli, cancel).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e:?}");
            if let Some(log_path) = log_dir.join("astro-up.log").to_str() {
                eprintln!("\nLog file: {log_path}");
            }
            ExitCode::from(1)
        }
    }
}
