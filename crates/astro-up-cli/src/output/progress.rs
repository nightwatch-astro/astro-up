use std::io::{self, Write};

use indicatif::{ProgressBar, ProgressStyle};

use astro_up_core::events::Event;

use super::OutputMode;

/// Create an indicatif progress bar for download tracking.
pub fn create_download_bar(total_bytes: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_bytes);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )
        .unwrap()
        .progress_chars("=> "),
    );
    pb
}

/// Render a core event based on the output mode.
///
/// - Interactive: uses indicatif progress bar (passed in)
/// - Plain: line-by-line text to stderr
/// - Quiet: suppressed
/// - Json: caller handles JSON streaming separately
pub fn render_event(event: &Event, mode: &OutputMode, download_bar: Option<&ProgressBar>) {
    match mode {
        OutputMode::Interactive => render_interactive(event, download_bar),
        OutputMode::Plain => render_plain(event),
        OutputMode::Quiet | OutputMode::Json => {}
    }
}

/// Interactive mode: use indicatif progress bar for downloads, eprintln for status.
fn render_interactive(event: &Event, download_bar: Option<&ProgressBar>) {
    match event {
        Event::PlanReady { total, skipped } => {
            eprintln!("Plan: {total} package(s) to process, {skipped} skipped");
        }
        Event::PackageStarted {
            package_id,
            step_count,
        } => {
            eprintln!("  [{package_id}] starting ({step_count} steps)...");
        }
        Event::DownloadStarted { id, .. } => {
            if let Some(pb) = download_bar {
                pb.set_message(format!("[{id}] downloading"));
            } else {
                eprintln!("  [{id}] downloading...");
            }
        }
        Event::DownloadProgress {
            bytes_downloaded,
            total_bytes,
            ..
        } => {
            if let Some(pb) = download_bar {
                if pb.length() != Some(*total_bytes) && *total_bytes > 0 {
                    pb.set_length(*total_bytes);
                }
                pb.set_position(*bytes_downloaded);
            }
        }
        Event::DownloadComplete { id } => {
            if let Some(pb) = download_bar {
                pb.finish_with_message(format!("[{id}] download complete"));
            } else {
                eprintln!("  [{id}] download complete");
            }
        }
        Event::InstallStarted { id } => {
            eprintln!("  [{id}] installing...");
        }
        Event::InstallComplete { id } => {
            eprintln!("  [{id}] installed successfully");
        }
        Event::InstallFailed { id, error } => {
            eprintln!("  [{id}] install failed: {error}");
        }
        Event::PackageComplete {
            package_id, status, ..
        } => {
            eprintln!("  [{package_id}] {status}");
        }
        Event::PackageSkipped {
            package_id, reason, ..
        } => {
            eprintln!("  [{package_id}] skipped: {reason}");
        }
        Event::ProcessBlocking {
            package_id,
            process_name,
            pid,
        } => {
            eprintln!("  [{package_id}] blocked by {process_name} (PID {pid})");
        }
        Event::OrchestrationComplete {
            succeeded,
            failed,
            skipped,
        } => {
            eprintln!("\nDone: {succeeded} succeeded, {failed} failed, {skipped} skipped");
        }
        _ => {}
    }
}

/// Plain mode: line-by-line text to stderr (no ANSI, no carriage returns).
fn render_plain(event: &Event) {
    let stderr = io::stderr();
    let mut out = stderr.lock();

    match event {
        Event::PlanReady { total, skipped } => {
            let _ = writeln!(
                out,
                "Plan: {total} package(s) to process, {skipped} skipped"
            );
        }
        Event::PackageStarted {
            package_id,
            step_count,
        } => {
            let _ = writeln!(out, "  [{package_id}] starting ({step_count} steps)...");
        }
        Event::DownloadStarted { id, .. } => {
            let _ = writeln!(out, "  [{id}] downloading...");
        }
        Event::DownloadProgress {
            id,
            progress,
            speed,
            ..
        } => {
            let pct = (progress * 100.0) as u32;
            let _ = writeln!(out, "  [{id}] {pct}% ({speed:.0} B/s)");
        }
        Event::DownloadComplete { id } => {
            let _ = writeln!(out, "  [{id}] download complete");
        }
        Event::InstallStarted { id } => {
            let _ = writeln!(out, "  [{id}] installing...");
        }
        Event::InstallComplete { id } => {
            let _ = writeln!(out, "  [{id}] installed successfully");
        }
        Event::InstallFailed { id, error } => {
            let _ = writeln!(out, "  [{id}] install failed: {error}");
        }
        Event::PackageComplete {
            package_id, status, ..
        } => {
            let _ = writeln!(out, "  [{package_id}] {status}");
        }
        Event::PackageSkipped {
            package_id, reason, ..
        } => {
            let _ = writeln!(out, "  [{package_id}] skipped: {reason}");
        }
        Event::ProcessBlocking {
            package_id,
            process_name,
            pid,
        } => {
            let _ = writeln!(
                out,
                "  [{package_id}] blocked by {process_name} (PID {pid})"
            );
        }
        Event::OrchestrationComplete {
            succeeded,
            failed,
            skipped,
        } => {
            let _ = writeln!(
                out,
                "\nDone: {succeeded} succeeded, {failed} failed, {skipped} skipped"
            );
        }
        _ => {}
    }
}
