# Quickstart: 005-manifest-catalog

## New dependencies

Add to `crates/astro-up-core/Cargo.toml`:

```toml
[dependencies]
minisign-verify = "0.2"
reqwest = { version = "0.13", features = ["rustls-tls"], default-features = false }
sysinfo = "0.38"
tokio = { version = "1", features = ["fs"] }

# Also upgrade existing:
rusqlite = { version = "0.39", features = ["bundled"] }
```

## Module structure

```
crates/astro-up-core/src/
├── catalog/
│   ├── mod.rs          # Public API re-exports
│   ├── reader.rs       # CatalogReader impl (SQLite queries, FTS5 search)
│   ├── manager.rs      # CatalogManager impl (fetch, verify, refresh)
│   ├── fetch.rs        # HTTP fetch with ETag, retry, sidecar
│   ├── verify.rs       # Minisign signature verification
│   ├── lock.rs         # PID lockfile
│   ├── sidecar.rs      # catalog.db.meta JSON read/write
│   └── types.rs        # PackageSummary, VersionEntry, PackageId, etc.
```

## Key constants

```rust
// Embedded at compile time
const MINISIGN_PUBLIC_KEY: &str = "RWT3Z/NUV2mo2nf2YDHF/Iyz9NFR7+gkUHa0rTlAcIBBxg+eqG3LUItj";
const SUPPORTED_SCHEMA: &str = "1";
```

## Smoke test

```rust
#[tokio::test]
async fn test_catalog_loads() {
    // Uses a test fixture catalog.db + catalog.db.minisig
    let dir = tempfile::tempdir().unwrap();
    // Copy test fixtures to dir...
    let reader = SqliteCatalogReader::open(dir.path().join("catalog.db")).unwrap();
    let results = reader.search("nina").await.unwrap();
    assert!(!results.is_empty());
}
```

## Test fixtures

Create a small test catalog using the compiler from `astro-up-manifests`:

```fish
cd /path/to/astro-up-manifests
# Create a minimal test set (3-5 manifests)
cargo run --release --bin astro-up-compiler -- \
    --manifests test-fixtures/manifests \
    --versions test-fixtures/versions \
    --output test-fixtures/catalog.db
# Sign it
minisign -Sm test-fixtures/catalog.db -s /path/to/test.key
```

Store fixtures in `crates/astro-up-core/tests/fixtures/catalog/`.
