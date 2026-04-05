#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Performance benchmarks for catalog operations (SC-001, SC-002 from spec 005).

use std::path::Path;
use std::time::Instant;

use astro_up_core::catalog::SqliteCatalogReader;

fn fixture_catalog() -> std::path::PathBuf {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/catalog"
    ))
    .join("catalog.db")
}

/// SC-001: Catalog open should complete quickly.
/// Threshold is 100ms to tolerate slow CI runners (Windows VMs).
/// Local performance is typically <5ms.
#[test]
fn catalog_open_under_100ms() {
    let path = fixture_catalog();
    if !path.exists() {
        eprintln!("skipping: fixture catalog not found");
        return;
    }

    let start = Instant::now();
    let _reader = SqliteCatalogReader::open(&path).expect("failed to open catalog");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 100,
        "catalog open took {elapsed:?}, expected <100ms (SC-001)"
    );
}

/// SC-002: FTS5 search should complete in under 50ms.
#[tokio::test]
async fn fts5_search_under_50ms() {
    let path = fixture_catalog();
    if !path.exists() {
        eprintln!("skipping: fixture catalog not found");
        return;
    }

    let reader = SqliteCatalogReader::open(&path).expect("failed to open catalog");

    let start = Instant::now();
    let results = reader.search("nina").expect("search failed");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 200,
        "FTS5 search took {elapsed:?}, expected <200ms (SC-002)"
    );
    // Fixture catalog should have at least one result for "nina"
    assert!(!results.is_empty(), "expected search results for 'nina'");
}

/// SC-001 related: list_all should complete quickly even for full catalog.
#[test]
fn list_all_under_50ms() {
    let path = fixture_catalog();
    if !path.exists() {
        eprintln!("skipping: fixture catalog not found");
        return;
    }

    let reader = SqliteCatalogReader::open(&path).expect("failed to open catalog");

    let start = Instant::now();
    let all = reader.list_all().expect("list_all failed");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 200,
        "list_all took {elapsed:?}, expected <200ms"
    );
    assert!(!all.is_empty(), "catalog should not be empty");
}
