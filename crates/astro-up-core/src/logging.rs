use std::path::Path;

use tracing::{debug, warn};

/// Delete log files in `log_dir` older than `max_age_days`.
/// Skips pruning if `max_age_days` is 0. Best-effort — errors are logged but not propagated.
pub fn prune_old_logs(log_dir: &Path, max_age_days: u32) {
    if max_age_days == 0 {
        return;
    }

    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(u64::from(max_age_days) * 86400);

    let entries = match std::fs::read_dir(log_dir) {
        Ok(e) => e,
        Err(e) => {
            debug!(path = %log_dir.display(), error = %e, "cannot read log dir for pruning");
            return;
        }
    };

    let mut pruned = 0u32;
    for entry in entries.flatten() {
        let path = entry.path();

        // Only prune files that look like rotated logs (contain "astro-up.log")
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.contains("astro-up.log") {
            continue;
        }

        let Ok(modified) = entry.metadata().and_then(|m| m.modified()) else {
            continue;
        };

        if modified < cutoff {
            if let Err(e) = std::fs::remove_file(&path) {
                warn!(path = %path.display(), error = %e, "failed to prune old log file");
            } else {
                pruned += 1;
            }
        }
    }

    if pruned > 0 {
        debug!(pruned, max_age_days, "pruned old log files");
    }
}
