use std::path::Path;

use color_eyre::eyre::{Result, eyre};

use astro_up_core::lifecycle::{
    LifecycleOptions, LifecycleReport, LifecycleRunner, LifecycleStatus, PhaseStatus,
};

use crate::output::OutputMode;
use crate::output::json::print_json;

/// Handle the lifecycle-test CLI subcommand.
pub async fn handle_lifecycle_test(
    package_id: &str,
    manifest_path: &Path,
    version: Option<&str>,
    install_dir: Option<&Path>,
    catalog_path: Option<&Path>,
    dry_run: bool,
    report_file: Option<&Path>,
    mode: &OutputMode,
) -> Result<()> {
    tracing::debug!(
        package_id,
        ?version,
        dry_run,
        "entering handle_lifecycle_test"
    );
    if !manifest_path.join("manifests").is_dir() {
        return Err(eyre!(
            "manifest path '{}' does not contain a 'manifests/' directory",
            manifest_path.display()
        ));
    }

    let software =
        astro_up_core::catalog::manifest::ManifestReader::read_by_id(manifest_path, package_id)
            .map_err(|e| eyre!("{e}"))?;

    let is_download_only = software
        .install
        .as_ref()
        .is_some_and(|i| i.method == astro_up_core::types::InstallMethod::DownloadOnly);

    if is_download_only && install_dir.is_none() {
        return Err(eyre!(
            "package '{package_id}' uses download_only — --install-dir is required"
        ));
    }

    let options = LifecycleOptions {
        manifest_path: manifest_path.to_owned(),
        package_id: package_id.to_string(),
        version: version.map(String::from),
        install_dir: install_dir.map(std::borrow::ToOwned::to_owned),
        catalog_path: catalog_path.map(std::borrow::ToOwned::to_owned),
        dry_run,
        ..Default::default()
    };

    let report = LifecycleRunner::run(&options)
        .await
        .map_err(|e| eyre!("{e}"))?;

    if let Some(path) = report_file {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(path, &json)?;
        if *mode != OutputMode::Json {
            println!("Report written to {}", path.display());
        }
    }

    write_job_summary(&report);

    if *mode == OutputMode::Json {
        print_json(&report)?;
    } else {
        print_human_report(&report);
    }

    tracing::debug!(
        package_id,
        overall_status = ?report.overall_status,
        "exiting handle_lifecycle_test"
    );
    match report.overall_status {
        LifecycleStatus::Pass | LifecycleStatus::PartialPass => Ok(()),
        LifecycleStatus::Fail => {
            let code = exit_code_for(&report);
            std::process::exit(code);
        }
    }
}

fn print_human_report(report: &LifecycleReport) {
    println!("Lifecycle test: {} v{}", report.package_id, report.version);
    println!("{}", "-".repeat(60));

    for phase in &report.phases {
        let tag = match phase.status {
            PhaseStatus::Pass => "PASS",
            PhaseStatus::Fail => "FAIL",
            PhaseStatus::Skipped => "SKIP",
        };
        println!(
            "  [{tag}] {:<20} ({}ms)",
            phase.phase,
            phase.duration.as_millis()
        );
        for w in &phase.warnings {
            println!("        warning: {w}");
        }
    }

    println!("{}", "-".repeat(60));
    let overall = match report.overall_status {
        LifecycleStatus::Pass => "PASS",
        LifecycleStatus::PartialPass => "PARTIAL",
        LifecycleStatus::Fail => "FAIL",
    };
    println!("Overall: {overall}");

    if let Some(ref config) = report.discovered_config {
        println!();
        println!("Discovered detection config:");
        println!("{}", LifecycleRunner::config_to_toml(config));
    }
}

fn exit_code_for(report: &LifecycleReport) -> i32 {
    for phase in &report.phases {
        if matches!(phase.status, PhaseStatus::Fail) {
            return match phase.phase.as_str() {
                "download" => 4,
                "install" => 1,
                "detect" => 2,
                "uninstall" | "verify-install" | "verify-removal" => 3,
                _ => 1,
            };
        }
    }
    1
}

fn write_job_summary(report: &LifecycleReport) {
    let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY") else {
        return;
    };

    use std::io::Write;
    let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        return;
    };

    let overall = match report.overall_status {
        LifecycleStatus::Pass => "PASS",
        LifecycleStatus::PartialPass => "PARTIAL",
        LifecycleStatus::Fail => "FAIL",
    };

    let _ = writeln!(
        f,
        "## Lifecycle Test: {} v{} — {overall}\n",
        report.package_id, report.version
    );
    let _ = writeln!(f, "| Phase | Status | Duration |");
    let _ = writeln!(f, "|-------|--------|----------|");

    for phase in &report.phases {
        let tag = match phase.status {
            PhaseStatus::Pass => "PASS",
            PhaseStatus::Fail => "FAIL",
            PhaseStatus::Skipped => "SKIP",
        };
        let _ = writeln!(
            f,
            "| {} | {} | {}ms |",
            phase.phase,
            tag,
            phase.duration.as_millis()
        );
    }

    if let Some(ref config) = report.discovered_config {
        let _ = writeln!(f, "\n### Discovered Detection Config\n");
        let _ = writeln!(
            f,
            "```toml\n{}\n```",
            LifecycleRunner::config_to_toml(config)
        );
    }
}
