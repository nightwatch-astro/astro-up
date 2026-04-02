use std::io::{self, Write};

use astro_up_core::events::Event;

/// Simple text-based progress renderer for CLI operations.
///
/// Prints events to stderr as they arrive. The full ratatui TUI
/// (gauge + paragraph) will be implemented when the engine is fully wired.
pub fn render_event(event: &Event) {
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
            let _ = write!(out, "\r  [{id}] downloading... {pct}% ({speed:.0} B/s)");
            let _ = out.flush();
        }
        Event::DownloadComplete { id } => {
            let _ = writeln!(out, "\r  [{id}] download complete                      ");
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
