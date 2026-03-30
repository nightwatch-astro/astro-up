# Implementation Plan: Manifest Parsing and Catalog

**Branch**: `005-manifest-catalog` | **Date**: 2026-03-30 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/005-manifest-catalog/spec.md`

## Summary

Implement the catalog module in `astro-up-core` — fetch a signed SQLite catalog from GitHub Releases, verify its minisign signature, cache it locally with ETag-based conditional refresh, and expose query/search/filter operations via FTS5. The catalog is the foundation for all downstream features (check, update, install).

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: rusqlite 0.39 (bundled, upgrade from 0.35), minisign-verify 0.2, reqwest 0.13 (rustls-tls), sysinfo 0.38, serde/serde_json (existing), chrono (existing)
**Storage**: SQLite — read-only access to pre-compiled catalog.db from GitHub Releases
**Testing**: cargo test + insta snapshots + tempfile + test fixture catalog
**Target Platform**: Windows primary, cross-platform CI (macOS, Linux)
**Project Type**: Library module within `astro-up-core`
**Performance Goals**: <10ms catalog load (SC-001), <50ms FTS5 search (SC-002)
**Constraints**: Catalog ~100 packages, single-file SQLite, compile-time embedded public key
**Scale/Scope**: ~100 packages, ~500 version entries, one catalog file

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | New `catalog/` module in `astro-up-core`, no new crate |
| II. Platform Awareness | PASS | No Windows-specific code. reqwest + rusqlite are cross-platform. PID lock uses `sysinfo` (cross-platform) |
| III. Test-First | PASS | Integration tests with fixture catalog.db, insta snapshots for search results |
| IV. Thin Tauri Boundary | PASS | All logic in core. GUI/CLI will call `CatalogReader`/`CatalogManager` traits |
| V. Spec-Driven | PASS | Full spec with 14 FRs, 4 SCs, 9 clarifications |
| VI. Simplicity | PASS | No cache layers, no retry backoff escalation, no feature flags. One retry, one file, one schema version |

No violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/005-manifest-catalog/
├── plan.md              # This file
├── spec.md              # Feature specification
├── decisions.md         # Architecture decisions
├── research.md          # Phase 0: library research
├── data-model.md        # Phase 1: entity definitions
├── quickstart.md        # Phase 1: getting started guide
├── contracts/           # Phase 1: public API contracts
│   └── catalog-trait.rs # CatalogReader + CatalogManager traits
├── checklists/          # Requirement quality checklists
│   └── catalog.md       # Pre-plan checklist
└── tasks.md             # Phase 2: implementation tasks (next step)
```

### Source Code (repository root)

```text
crates/astro-up-core/src/
├── catalog/
│   ├── mod.rs           # Public re-exports, module doc
│   ├── types.rs         # PackageId, PackageSummary, VersionEntry, CatalogMeta, CatalogSidecar
│   ├── reader.rs        # SqliteCatalogReader — resolve, search, filter, list, versions
│   ├── manager.rs       # CatalogManager — ensure_catalog, refresh (orchestrates fetch+verify+lock)
│   ├── fetch.rs         # HTTP fetch — ETag conditional request, retry, download
│   ├── verify.rs        # Minisign signature verification
│   ├── lock.rs          # PID lockfile — acquire, release, stale detection
│   └── sidecar.rs       # catalog.db.meta JSON — read/write etag + fetched_at
├── error.rs             # Add CatalogError variants to CoreError
└── lib.rs               # Add `pub mod catalog;`

crates/astro-up-core/tests/
├── fixtures/catalog/
│   ├── catalog.db       # Test fixture (compiled from 3-5 manifests)
│   └── catalog.db.minisig  # Signed with test key
└── catalog_integration.rs   # Integration tests
```

**Structure Decision**: Single module in existing `astro-up-core` crate per Constitution Principle I. No new crates.

## Implementation Phases

### Phase A: Foundation (types, errors, lockfile)

- `PackageId` newtype with validation, `FromStr`, `Display`, serde
- `PackageSummary`, `VersionEntry`, `CatalogMeta`, `CatalogSidecar` structs
- `CatalogError` variants added to `CoreError`
- PID lockfile (`lock.rs`) — acquire, check stale, release on Drop
- Test: PackageId validation, lockfile acquire/release/stale

### Phase B: Catalog Reader (SQLite queries)

- `SqliteCatalogReader` — opens catalog.db read-only
- Schema version check from `meta` table
- `resolve()` — exact ID lookup
- `search()` — FTS5 query with rank ordering
- `filter()` — category/type filter via SQL WHERE
- `list_all()` — all packages
- `versions()` / `latest_version()` — version queries
- JSON decoding for `aliases`, `tags`, `dependencies` columns
- Test: all queries against fixture catalog, insta snapshots

### Phase C: Signature Verification

- `verify_catalog()` — minisign-verify with embedded public key
- One-shot verification (catalog is <1MB, no streaming needed)
- Test: valid sig passes, tampered data fails, missing sig fails

### Phase D: Fetch and Refresh

- `fetch_catalog()` — reqwest GET with ETag/If-None-Match
- Sidecar read/write (`catalog.db.meta`)
- TTL check from `fetched_at`
- One retry with 1-2s backoff on transient failure
- `CatalogManager` orchestration: TTL check → fetch → verify → store → update sidecar
- Test: mock HTTP server for 200/304/5xx, sidecar persistence

## Dependencies (new)

| Crate | Version | Purpose | blessed.rs/lib.rs |
|-------|---------|---------|-------------------|
| minisign-verify | 0.2 | Signature verification | lib.rs |
| reqwest | 0.13 | HTTP client (rustls-tls) | blessed.rs, lib.rs |
| sysinfo | 0.38 | Cross-platform PID check | lib.rs |
| rusqlite | 0.39 | SQLite (upgrade from 0.35) | blessed.rs, lib.rs |

## Risks

| Risk | Mitigation |
|------|------------|
| FTS5 tokenization of punctuated aliases | Compiler normalizes before indexing (FR-014, PR #79) |
| Catalog download on slow connections | Single retry + local fallback. Catalog is <1MB |
| Schema version drift | Exact match check (FR-013). Coordinated releases |
| Test fixture staleness | Generate from compiler, check in to repo |
