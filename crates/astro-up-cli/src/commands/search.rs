use color_eyre::eyre::Result;
use serde::Serialize;
use tabled::Tabled;

use crate::output::OutputMode;
use crate::output::json::print_json;
use crate::output::table::print_table;

use crate::state::CliState;

/// T019: Search the catalog via FTS5.
pub async fn handle_search(state: &CliState, query: &str, mode: &OutputMode) -> Result<()> {
    tracing::debug!(query, "entering handle_search");
    let reader = state.open_catalog_reader_ensure().await?;
    let results = reader.search(query)?;

    if *mode == OutputMode::Json {
        let rows: Vec<SearchRow> = results.iter().map(SearchRow::from).collect();
        return print_json(&rows);
    }

    if results.is_empty() {
        if mode.should_print() {
            println!("No packages matching '{query}'.");
        }
        return Ok(());
    }

    if mode.should_print() {
        let rows: Vec<SearchRow> = results.iter().map(SearchRow::from).collect();
        print_table(&rows)?;
        println!("\n{} result(s)", results.len());
    }
    tracing::debug!(query, results = results.len(), "exiting handle_search");
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
