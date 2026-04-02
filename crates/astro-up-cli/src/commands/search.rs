use color_eyre::eyre::Result;
use serde::Serialize;
use tabled::Tabled;

use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::output::table::print_table;

use super::ensure_catalog;

/// T019: Search the catalog via FTS5.
pub async fn handle_search(query: &str, mode: &OutputMode) -> Result<()> {
    let reader = ensure_catalog().await?;
    let results = reader.search(query)?;

    if *mode == OutputMode::Json {
        let rows: Vec<SearchRow> = results.iter().map(SearchRow::from).collect();
        return print_json(&rows);
    }

    if results.is_empty() {
        println!("No packages matching '{query}'.");
        return Ok(());
    }

    let rows: Vec<SearchRow> = results.iter().map(SearchRow::from).collect();
    print_table(&rows)?;
    println!("\n{} result(s)", results.len());
    Ok(())
}

#[derive(Tabled, Serialize)]
struct SearchRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Description")]
    description: String,
}

impl From<&astro_up_core::catalog::SearchResult> for SearchRow {
    fn from(r: &astro_up_core::catalog::SearchResult) -> Self {
        Self {
            id: r.package.id.to_string(),
            name: r.package.name.clone(),
            category: r.package.category.to_string(),
            description: r
                .package
                .description
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect(),
        }
    }
}
