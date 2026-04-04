use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use tabled::Tabled;

use astro_up_core::catalog::{PackageId, PackageSummary, SqliteCatalogReader};

use crate::ShowFilter;
use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::output::table::print_table;

use super::ensure_catalog;

/// T012: Main show handler — dispatches to the appropriate sub-view.
pub async fn handle_show(filter: Option<ShowFilter>, mode: &OutputMode) -> Result<()> {
    match filter {
        None | Some(ShowFilter::All) => {
            let reader = ensure_catalog().await?;
            show_all(&reader, mode)
        }
        Some(ShowFilter::Installed) => show_installed(mode),
        Some(ShowFilter::Outdated) => show_outdated(mode),
        Some(ShowFilter::Backups { package }) => show_backups(package.as_deref(), mode).await,
    }
}

// ---------------------------------------------------------------------------
// T012: Show all / installed / outdated
// ---------------------------------------------------------------------------

#[derive(Tabled, Serialize)]
struct PackageRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Type")]
    software_type: String,
}

impl From<&PackageSummary> for PackageRow {
    fn from(p: &PackageSummary) -> Self {
        Self {
            id: p.id.to_string(),
            name: p.name.clone(),
            category: p.category.to_string(),
            software_type: p.software_type.to_string(),
        }
    }
}

fn show_all(reader: &SqliteCatalogReader, mode: &OutputMode) -> Result<()> {
    let packages = reader.list_all()?;

    if *mode == OutputMode::Json {
        return print_json(&packages);
    }

    if packages.is_empty() {
        if mode.should_print() {
            println!("No packages in catalog.");
        }
        return Ok(());
    }

    if mode.should_print() {
        let rows: Vec<PackageRow> = packages.iter().map(PackageRow::from).collect();
        print_table(&rows)?;
        println!("\n{} packages in catalog", packages.len());
    }
    Ok(())
}

fn show_installed(mode: &OutputMode) -> Result<()> {
    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({"packages": [], "note": "scan not yet available"}));
    }
    if mode.should_print() {
        println!("No scan results available. Run `astro-up scan` first.");
    }
    Ok(())
}

fn show_outdated(mode: &OutputMode) -> Result<()> {
    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({"packages": [], "note": "scan not yet available"}));
    }
    if mode.should_print() {
        println!("No scan results available. Run `astro-up scan` first.");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// T013: Show package detail view
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct PackageDetail {
    id: String,
    name: String,
    category: String,
    software_type: String,
    description: Option<String>,
    publisher: Option<String>,
    homepage: Option<String>,
    license: Option<String>,
    tags: Vec<String>,
    aliases: Vec<String>,
    dependencies: Vec<String>,
    latest_version: Option<String>,
}

pub fn handle_show_detail(
    reader: &SqliteCatalogReader,
    package: &str,
    mode: &OutputMode,
) -> Result<()> {
    let id: PackageId = package
        .parse()
        .map_err(|_| eyre!("invalid package id: {package}"))?;
    let pkg = reader.resolve(&id)?;
    let latest = reader.latest_version(&id)?;

    let detail = PackageDetail {
        id: pkg.id.to_string(),
        name: pkg.name.clone(),
        category: pkg.category.to_string(),
        software_type: pkg.software_type.to_string(),
        description: pkg.description.clone(),
        publisher: pkg.publisher.clone(),
        homepage: pkg.homepage.clone(),
        license: pkg.license.clone(),
        tags: pkg.tags.clone(),
        aliases: pkg.aliases.clone(),
        dependencies: pkg.dependencies.clone(),
        latest_version: latest.map(|v| v.version),
    };

    if *mode == OutputMode::Json {
        return print_json(&detail);
    }

    println!("{} ({})", detail.name, detail.id);
    println!("Category:    {}", detail.category);
    println!("Type:        {}", detail.software_type);
    if let Some(ref desc) = detail.description {
        println!("Description: {desc}");
    }
    if let Some(ref publisher) = detail.publisher {
        println!("Publisher:   {publisher}");
    }
    if let Some(ref homepage) = detail.homepage {
        println!("Homepage:    {homepage}");
    }
    if let Some(ref license) = detail.license {
        println!("License:     {license}");
    }
    if let Some(ref ver) = detail.latest_version {
        println!("Latest:      {ver}");
    }
    if !detail.tags.is_empty() {
        println!("Tags:        {}", detail.tags.join(", "));
    }
    if !detail.aliases.is_empty() {
        println!("Aliases:     {}", detail.aliases.join(", "));
    }
    if !detail.dependencies.is_empty() {
        println!("Depends on:  {}", detail.dependencies.join(", "));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// T014: Show backups
// ---------------------------------------------------------------------------

#[derive(Tabled, Serialize)]
struct BackupRow {
    #[tabled(rename = "Package")]
    package: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Files")]
    file_count: u32,
    #[tabled(rename = "Size")]
    size: String,
}

async fn show_backups(package: Option<&str>, mode: &OutputMode) -> Result<()> {
    let data_dir = data_dir()?;
    let backup_dir = data_dir.join("backups");
    let service = astro_up_core::backup::BackupService::new(backup_dir, 0);

    let package_id = package.unwrap_or("*");
    let entries = if package_id == "*" {
        if *mode == OutputMode::Json {
            return print_json(&serde_json::json!({"backups": []}));
        }
        if mode.should_print() {
            println!("Specify a package: `astro-up show backups <package>`");
        }
        return Ok(());
    } else {
        service.list(package_id).await?
    };

    if *mode == OutputMode::Json {
        return print_json(&entries);
    }

    if entries.is_empty() {
        if mode.should_print() {
            println!("No backups found for '{package_id}'.");
        }
        return Ok(());
    }

    if !mode.should_print() {
        return Ok(());
    }

    let rows: Vec<BackupRow> = entries
        .iter()
        .map(|e| BackupRow {
            package: e.package_id.clone(),
            version: e.version.raw.clone(),
            date: e.created_at.format("%Y-%m-%d %H:%M").to_string(),
            file_count: e.file_count,
            size: format_bytes(e.total_size),
        })
        .collect();
    print_table(&rows)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn data_dir() -> Result<std::path::PathBuf> {
    directories::ProjectDirs::from("com", "nightwatch", "astro-up")
        .map(|dirs| dirs.data_dir().to_owned())
        .ok_or_else(|| eyre!("could not determine data directory"))
}
