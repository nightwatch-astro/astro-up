use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use tabled::Tabled;

use astro_up_core::adapters::SqliteLedgerStore;
use astro_up_core::catalog::{PackageId, PackageSummary, SqliteCatalogReader};

use crate::ShowFilter;
use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::output::table::print_table;

use crate::state::CliState;

/// Main show handler — dispatches to the appropriate sub-view.
pub async fn handle_show(
    state: &CliState,
    filter: Option<ShowFilter>,
    mode: &OutputMode,
) -> Result<()> {
    tracing::debug!(?filter, "entering handle_show");
    let result = match filter {
        None | Some(ShowFilter::All) => {
            let reader = state.open_catalog_reader_ensure().await?;
            show_all(&reader, mode)
        }
        Some(ShowFilter::Installed) => show_installed(state, mode).await,
        Some(ShowFilter::Outdated) => show_outdated(state, mode).await,
        Some(ShowFilter::Backups { package }) => show_backups(package.as_deref(), mode).await,
    };
    tracing::debug!(ok = result.is_ok(), "exiting handle_show");
    result
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
        println!("No packages in catalog.");
        return Ok(());
    }

    let rows: Vec<PackageRow> = packages.iter().map(PackageRow::from).collect();
    print_table(&rows)?;
    println!("\n{} packages in catalog", packages.len());
    Ok(())
}

/// Show installed packages from the ledger (FR-004, T017).
/// Reads persisted scan results — does NOT re-scan or download catalog.
async fn show_installed(state: &CliState, mode: &OutputMode) -> Result<()> {
    use astro_up_core::detect::scanner::LedgerStore;
    let ledger = SqliteLedgerStore::new(state.db_path.clone());
    let entries = ledger
        .list_acknowledged()
        .map_err(|e| eyre!("failed to read ledger: {e}"))?;

    if entries.is_empty() {
        if *mode == OutputMode::Json {
            return print_json(
                &serde_json::json!({"packages": [], "note": "no scan results — run astro-up scan first"}),
            );
        }
        if mode.should_print() {
            println!("No scan results available. Run `astro-up scan` first.");
        }
        return Ok(());
    }

    #[derive(Tabled, Serialize)]
    struct Row {
        #[tabled(rename = "Package")]
        id: String,
        #[tabled(rename = "Version")]
        version: String,
    }

    let rows: Vec<Row> = entries
        .iter()
        .map(|e| Row {
            id: e.package_id.clone(),
            version: e.version.to_string(),
        })
        .collect();

    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({"packages": rows}));
    }

    if !mode.should_print() {
        return Ok(());
    }

    print_table(&rows)?;
    println!("\n{} installed package(s)", rows.len());
    Ok(())
}

/// Show packages with available updates from ledger + catalog (FR-004, T018).
/// Reads persisted scan results from ledger, compares against catalog.
async fn show_outdated(state: &CliState, mode: &OutputMode) -> Result<()> {
    use astro_up_core::detect::scanner::LedgerStore;
    let ledger = SqliteLedgerStore::new(state.db_path.clone());
    let entries = ledger
        .list_acknowledged()
        .map_err(|e| eyre!("failed to read ledger: {e}"))?;

    if entries.is_empty() {
        if *mode == OutputMode::Json {
            return print_json(
                &serde_json::json!({"packages": [], "note": "no scan results — run astro-up scan first"}),
            );
        }
        if mode.should_print() {
            println!("No scan results available. Run `astro-up scan` first.");
        }
        return Ok(());
    }

    // Need catalog for version comparison — this is a read, not a download
    let reader = state
        .open_catalog_reader()
        .map_err(|_| eyre!("catalog not available — run astro-up scan first to sync"))?;

    #[derive(Tabled, Serialize)]
    struct Row {
        #[tabled(rename = "Package")]
        id: String,
        #[tabled(rename = "Installed")]
        installed: String,
        #[tabled(rename = "Latest")]
        latest: String,
    }

    let mut outdated = Vec::new();
    for entry in &entries {
        let pkg_id: Result<PackageId, _> = entry.package_id.parse();
        if let Ok(id) = pkg_id {
            if let Ok(Some(latest)) = reader.latest_version(&id) {
                let latest_ver = astro_up_core::types::Version::parse(&latest.version);
                if latest_ver > entry.version {
                    outdated.push(Row {
                        id: entry.package_id.clone(),
                        installed: entry.version.to_string(),
                        latest: latest.version.clone(),
                    });
                }
            }
        }
    }

    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({"packages": outdated}));
    }

    if !mode.should_print() {
        return Ok(());
    }

    if outdated.is_empty() {
        println!("All installed packages are up to date.");
        return Ok(());
    }

    print_table(&outdated)?;
    println!("\n{} update(s) available", outdated.len());
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
    tracing::debug!(package, "entering handle_show_detail");
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
        dependencies: pkg.dependencies,
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
    tracing::debug!(package, "exiting handle_show_detail");
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
        // List all backups — not directly supported, show message
        if *mode == OutputMode::Json {
            return print_json(&serde_json::json!({"backups": []}));
        }
        println!("Specify a package: `astro-up show backups <package>`");
        return Ok(());
    } else {
        service.list(package_id).await?
    };

    if *mode == OutputMode::Json {
        return print_json(&entries);
    }

    if entries.is_empty() {
        println!("No backups found for '{package_id}'.");
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
