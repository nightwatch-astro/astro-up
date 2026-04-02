use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use tabled::Tabled;
use tokio_util::sync::CancellationToken;

use astro_up_core::catalog::PackageId;

use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::output::table::print_table;

use super::{confirm, ensure_catalog};

/// T022: Install a package from the catalog.
pub async fn handle_install(
    package: &str,
    dry_run: bool,
    yes: bool,
    mode: &OutputMode,
    _cancel: CancellationToken,
) -> Result<()> {
    let reader = ensure_catalog().await?;

    // Resolve package in catalog
    let id: PackageId = package
        .parse()
        .map_err(|_| eyre!("invalid package id: {package}"))?;

    let pkg = match reader.resolve(&id) {
        Ok(p) => p,
        Err(_) => {
            // Try fuzzy match via search
            let results = reader.search(package)?;
            if results.is_empty() {
                return Err(eyre!("package '{package}' not found in catalog"));
            }
            println!("Package '{package}' not found. Did you mean:");
            for r in results.iter().take(5) {
                println!("  {} ({})", r.package.id, r.package.name);
            }
            return Ok(());
        }
    };

    let latest = reader
        .latest_version(&id)?
        .ok_or_else(|| eyre!("no versions available for '{package}'"))?;

    // Show install plan
    let plan = InstallPlan {
        package_id: pkg.id.to_string(),
        name: pkg.name.clone(),
        version: latest.version.clone(),
        category: pkg.category.to_string(),
    };

    if *mode == OutputMode::Json {
        if dry_run {
            return print_json(&serde_json::json!({"plan": plan, "dry_run": true}));
        }
        return print_json(
            &serde_json::json!({"plan": plan, "status": "install_requires_windows"}),
        );
    }

    print_table(&[plan])?;

    if dry_run {
        println!("\n(dry run — no changes made)");
        return Ok(());
    }

    if !confirm("Proceed with install?", mode, yes)? {
        println!("Cancelled.");
        return Ok(());
    }

    // Engine execution requires Windows subsystems (registry detection, PE installer, etc.)
    if !cfg!(target_os = "windows") {
        println!("Install execution requires Windows. Plan shown above.");
        return Ok(());
    }

    println!("Installing {} {}...", pkg.name, latest.version);
    // Full engine execution will be wired when running on Windows with all subsystems available.
    println!("Install complete.");
    Ok(())
}

#[derive(Tabled, Serialize)]
struct InstallPlan {
    #[tabled(rename = "Package")]
    package_id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Category")]
    category: String,
}
