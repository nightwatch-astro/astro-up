use clap::Parser;
use color_eyre::eyre::Result;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(not(debug_assertions))]
    human_panic::setup_panic!();

    color_eyre::install()?;

    let cli = astro_up_cli::Cli::parse();

    let log_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up")
        .map(|dirs| dirs.data_dir().join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let _log_guard = astro_up_cli::logging::init(cli.verbose, &log_dir)?;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");
        cancel_clone.cancel();
    });

    astro_up_cli::run(cli, cancel).await
}
