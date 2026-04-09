use color_eyre::eyre::Result;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize dual-layer tracing:
/// - stderr: compact human-readable, respects verbosity
/// - file: JSON, daily rotation to `{log_dir}/` (best-effort — falls back to stderr-only)
///
/// Returns the `WorkerGuard` which must be kept alive for the duration of the program.
pub fn init(
    verbose: bool,
    quiet: bool,
    log_dir: &std::path::Path,
    max_age_days: u32,
) -> Result<WorkerGuard> {
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

    // Prune old log files before creating new ones
    astro_up_core::logging::prune_old_logs(log_dir, max_age_days);

    // Try to set up file logging — fall back to stderr-only if dir creation fails.
    let file_result = std::fs::create_dir_all(log_dir)
        .ok()
        .map(|()| tracing_appender::rolling::daily(log_dir, "astro-up.log"))
        .map(tracing_appender::non_blocking);

    if let Some((non_blocking, guard)) = file_result {
        let file_layer = fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_filter(EnvFilter::new("debug"));

        tracing_subscriber::registry()
            .with(stderr_layer)
            .with(file_layer)
            .init();

        Ok(guard)
    } else {
        // No file logging — create a dummy guard from /dev/null
        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::sink());
        drop(non_blocking);

        tracing_subscriber::registry().with(stderr_layer).init();

        Ok(guard)
    }
}
