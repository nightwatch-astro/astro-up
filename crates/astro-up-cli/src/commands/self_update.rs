use color_eyre::eyre::{Result, eyre};

use crate::output::OutputMode;
use crate::output::json::print_json;

/// Check for and install CLI updates from GitHub Releases.
///
/// Uses the `self_update` crate's lower-level API to fetch release metadata,
/// then downloads the raw `.exe` asset and replaces the running binary via
/// `self_replace`. We avoid the high-level `Update::update()` because our
/// release assets are bare executables, not archives.
pub async fn handle_self_update(dry_run: bool, mode: &OutputMode) -> Result<()> {
    let current_str = env!("CARGO_PKG_VERSION");
    let current = semver::Version::parse(current_str)
        .map_err(|e| eyre!("failed to parse current version: {e}"))?;

    // Fetch the release list (blocking — runs on a thread-pool thread).
    let releases = tokio::task::spawn_blocking(|| {
        self_update::backends::github::ReleaseList::configure()
            .repo_owner("nightwatch-astro")
            .repo_name("astro-up")
            .build()
            .and_then(|list| list.fetch())
    })
    .await
    .map_err(|e| eyre!("task join error: {e}"))?;

    let releases = match releases {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "GitHub Releases check failed");

            if *mode == OutputMode::Json {
                return print_json(&serde_json::json!({
                    "current_version": current_str,
                    "latest_version": null,
                    "status": "check_failed",
                    "dry_run": dry_run,
                }));
            }
            if mode.should_print() {
                println!("astro-up {current_str}");
                println!("Could not check for updates (network error). Try again later.");
            }
            return Ok(());
        }
    };

    let latest_release = releases.first();
    let latest_version = latest_release.and_then(|r| {
        let tag = r.version.strip_prefix('v').unwrap_or(&r.version);
        semver::Version::parse(tag).ok()
    });

    let status = match &latest_version {
        Some(latest) if latest > &current => "update_available",
        Some(_) => "up_to_date",
        None => "check_failed",
    };

    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({
            "current_version": current_str,
            "latest_version": latest_version.as_ref().map(|v| v.to_string()),
            "status": status,
            "dry_run": dry_run,
        }));
    }

    if !mode.should_print() {
        return Ok(());
    }

    println!("astro-up {current_str}");

    let Some(latest) = &latest_version else {
        println!("Could not determine latest version. Try again later.");
        return Ok(());
    };

    if latest <= &current {
        println!("You are running the latest version.");
        return Ok(());
    }

    println!("Update available: v{latest}");

    if dry_run {
        println!("(dry run -- no changes made)");
        return Ok(());
    }

    // Find the CLI binary asset. Our release publishes `astro-up.exe`.
    let asset_name = "astro-up.exe";
    let release = latest_release.unwrap(); // safe — latest_version is Some
    let asset = release.assets.iter().find(|a| a.name == asset_name);

    let Some(asset) = asset else {
        println!("No CLI binary found in release assets (expected '{asset_name}').");
        println!("Download manually from:");
        println!("  https://github.com/nightwatch-astro/astro-up/releases/latest");
        return Ok(());
    };

    println!("Downloading {asset_name}...");

    // Download the raw binary into a temp file, then self-replace.
    let download_url = asset.download_url.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let tmp_dir = tempfile::Builder::new()
            .prefix("astro-up-update")
            .tempdir()
            .map_err(|e| eyre!("failed to create temp dir: {e}"))?;
        let tmp_path = tmp_dir.path().join(asset_name);

        self_update::Download::from_url(&download_url)
            .set_header(
                reqwest::header::ACCEPT,
                "application/octet-stream".parse().unwrap(),
            )
            .download_to(
                &std::fs::File::create(&tmp_path)
                    .map_err(|e| eyre!("failed to create temp file: {e}"))?,
            )
            .map_err(|e| eyre!("download failed: {e}"))?;

        self_update::self_replace::self_replace(&tmp_path)
            .map_err(|e| eyre!("failed to replace binary: {e}"))?;

        Ok(())
    })
    .await
    .map_err(|e| eyre!("task join error: {e}"))??;

    println!("Updated to v{latest}. Restart astro-up to use the new version.");
    Ok(())
}

/// Clean up leftover `.old` binary from a previous self-update.
/// Call this early in main() startup.
pub fn cleanup_old_binary() {
    if let Ok(exe) = std::env::current_exe() {
        let old = exe.with_extension("old");
        let _ = std::fs::remove_file(old);
    }
}
