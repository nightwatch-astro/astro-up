use color_eyre::eyre::{Result, eyre};
use serde::Deserialize;

use crate::output::OutputMode;
use crate::output::json::print_json;

const RELEASES_URL: &str = "https://api.github.com/repos/nightwatch-astro/astro-up/releases/latest";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// Check for and install CLI updates from GitHub Releases (T023-T024).
pub async fn handle_self_update(dry_run: bool, mode: &OutputMode) -> Result<()> {
    let current_str = env!("CARGO_PKG_VERSION");
    let current = semver::Version::parse(current_str)
        .map_err(|e| eyre!("failed to parse current version: {e}"))?;

    let client = reqwest::Client::builder()
        .user_agent(format!("astro-up/{current_str}"))
        .build()
        .map_err(|e| eyre!("failed to create HTTP client: {e}"))?;

    let release = match client.get(RELEASES_URL).send().await {
        Ok(resp) if resp.status().is_success() => Some(
            resp.json::<GithubRelease>()
                .await
                .map_err(|e| eyre!("failed to parse release response: {e}"))?,
        ),
        Ok(resp) => {
            tracing::warn!(status = %resp.status(), "GitHub Releases check failed");
            None
        }
        Err(e) => {
            tracing::warn!(error = %e, "GitHub Releases check failed (network)");
            None
        }
    };

    let latest_version = release.as_ref().and_then(|r| {
        let tag = r.tag_name.strip_prefix('v').unwrap_or(&r.tag_name);
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
        println!("Could not check for updates (network error). Try again later.");
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

    // Find the CLI binary asset for this platform
    let asset_name = cli_asset_name();
    let asset = release
        .as_ref()
        .and_then(|r| r.assets.iter().find(|a| a.name == asset_name));

    let Some(asset) = asset else {
        println!("No CLI binary found in release assets (expected '{asset_name}').");
        println!("Download manually from:");
        println!("  https://github.com/nightwatch-astro/astro-up/releases/latest");
        return Ok(());
    };

    println!("Downloading {asset_name}...");
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| eyre!("download failed: {e}"))?
        .bytes()
        .await
        .map_err(|e| eyre!("download failed: {e}"))?;

    // Replace the running binary using rename-and-replace pattern:
    // 1. Get current exe path
    // 2. Rename current → .old
    // 3. Write new binary to original path
    // 4. .old is cleaned up on next start
    let current_exe =
        std::env::current_exe().map_err(|e| eyre!("could not determine current exe path: {e}"))?;
    let old_exe = current_exe.with_extension("old");

    // Clean up any leftover .old from a previous update
    let _ = std::fs::remove_file(&old_exe);

    // Rename current → .old (Windows allows renaming a running exe)
    std::fs::rename(&current_exe, &old_exe)
        .map_err(|e| eyre!("failed to rename current binary: {e}"))?;

    // Write new binary
    if let Err(e) = std::fs::write(&current_exe, &bytes) {
        // Restore the old binary on failure
        let _ = std::fs::rename(&old_exe, &current_exe);
        return Err(eyre!("failed to write new binary: {e}"));
    }

    println!("Updated to v{latest}. Restart astro-up to use the new version.");
    Ok(())
}

/// Determine the expected CLI asset name for this platform.
fn cli_asset_name() -> String {
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };
    format!("astro-up-{arch}-pc-windows-msvc.exe")
}

/// Clean up leftover .old binary from a previous self-update.
/// Call this early in main() startup.
pub fn cleanup_old_binary() {
    if let Ok(exe) = std::env::current_exe() {
        let old = exe.with_extension("old");
        let _ = std::fs::remove_file(old);
    }
}
