use color_eyre::eyre::Result;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize dual-layer tracing:
/// - stderr: compact human-readable, respects verbosity
/// - file: JSON, daily rotation to `{log_dir}/`
///
/// Returns the `WorkerGuard` which must be kept alive for the duration of the program.
pub fn init(verbose: bool, quiet: bool, log_dir: &std::path::Path) -> Result<WorkerGuard> {
    let stderr_filter = if quiet {
        EnvFilter::new("error")
    } else if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .compact()
        .with_filter(stderr_filter);

    let file_appender = tracing_appender::rolling::daily(log_dir, "astro-up.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_filter(EnvFilter::new("debug"));

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}
