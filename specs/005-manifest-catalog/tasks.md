# Tasks: Manifest Parsing and Catalog

**Input**: Design documents from `/specs/005-manifest-catalog/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included — Constitution III mandates integration tests over mocks.

**Organization**: Tasks grouped by user story. Each story is independently testable after its phase completes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Add dependencies and create module structure

- [ ] T001 Upgrade rusqlite from 0.35 to 0.39 in `crates/astro-up-core/Cargo.toml` and verify existing config tests pass
- [ ] T002 Add new dependencies to `crates/astro-up-core/Cargo.toml`: minisign-verify 0.2, reqwest 0.13 (features: rustls-tls, default-features = false), sysinfo 0.38, tokio (features: fs)
- [ ] T003 Create catalog module structure in `crates/astro-up-core/src/catalog/` with mod.rs, types.rs, reader.rs, manager.rs, fetch.rs, verify.rs, lock.rs, sidecar.rs — add `pub mod catalog;` to `crates/astro-up-core/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Types and error variants that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 [P] Implement `PackageId` newtype with validation (regex `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`, 2-50 chars), `FromStr`, `Display`, `Serialize`/`Deserialize`, `Hash`, `Eq`, `Ord`, `AsRef<str>` in `crates/astro-up-core/src/catalog/types.rs`
- [ ] T005 [P] Implement `PackageSummary` struct (read from packages table: id, name, slug, description, publisher, homepage, category, software_type, license, aliases, tags, dependencies, manifest_version) in `crates/astro-up-core/src/catalog/types.rs`
- [ ] T006 [P] Implement `VersionEntry` struct (read from versions table: package_id, version, url, sha256, discovered_at, release_notes_url, pre_release) in `crates/astro-up-core/src/catalog/types.rs`
- [ ] T007 [P] Implement `CatalogMeta` struct (schema_version, compiled_at), `CatalogSidecar` struct (etag, fetched_at), `SearchResult` struct (package + rank), `CatalogFilter` struct (category, software_type), `FetchResult` enum (Updated, Unchanged, FallbackToLocal) in `crates/astro-up-core/src/catalog/types.rs`
- [ ] T008 [P] Add catalog error variants to `CoreError` in `crates/astro-up-core/src/error.rs`: CatalogFetchFailed, CatalogSignatureInvalid, CatalogSignatureMissing, CatalogSchemaUnsupported, CatalogNotAvailable, CatalogCorrupted, CatalogLocked(pid), InvalidPackageId
- [ ] T009 [P] Implement PID lockfile in `crates/astro-up-core/src/catalog/lock.rs`: `PidLock::acquire(path)` checks for stale PID (sysinfo), writes current PID, releases on `Drop`. Test: acquire, double-acquire fails, stale lock recovery
- [ ] T010 [P] Implement sidecar read/write in `crates/astro-up-core/src/catalog/sidecar.rs`: `CatalogSidecar::load(path)` and `CatalogSidecar::save(path)` for JSON `catalog.db.meta` with etag + fetched_at. Test: round-trip, missing file returns None
- [ ] T011 [P] Create test fixture catalog: use astro-up-compiler from manifests repo to compile 3-5 test manifests into `crates/astro-up-core/tests/fixtures/catalog/catalog.db`, sign with test key, store `catalog.db.minisig` alongside. Add test public key constant.
- [ ] T012 Unit tests for `PackageId` validation in `crates/astro-up-core/src/catalog/types.rs` (valid IDs, invalid IDs, edge cases — too short, too long, uppercase, leading hyphen)

**Checkpoint**: Foundation ready — types, errors, lockfile, sidecar, test fixtures in place

---

## Phase 3: User Story 1 — Load Software Catalog (Priority: P1) MVP

**Goal**: Fetch signed SQLite catalog from GitHub Releases, verify signature, store locally with ETag-based conditional refresh

**Independent Test**: Start with no local catalog, fetch succeeds, catalog loads. Start with local catalog within TTL, no network request made. Start with expired TTL, conditional fetch returns 304 or 200+verify.

- [ ] T013 [US1] Implement minisign signature verification in `crates/astro-up-core/src/catalog/verify.rs`: `verify_catalog(catalog_path, sig_path)` using embedded `MINISIGN_PUBLIC_KEY` constant and `minisign_verify::PublicKey::from_base64()`. One-shot verify (catalog <1MB).
- [ ] T014 [US1] Integration test for signature verification in `crates/astro-up-core/tests/catalog_integration.rs`: valid signature passes, tampered data fails, missing .minisig fails
- [ ] T015 [US1] Implement HTTP fetch with ETag in `crates/astro-up-core/src/catalog/fetch.rs`: `fetch_catalog(url, etag) -> FetchResult` using reqwest with `If-None-Match` header, handle 200 (download + return new etag) and 304 (unchanged). Include one retry with 1-2s backoff on transient failure (timeout, 5xx).
- [ ] T016 [US1] Implement `SqliteCatalogReader::open(path)` in `crates/astro-up-core/src/catalog/reader.rs`: open catalog.db with `SQLITE_OPEN_READ_ONLY`, read `meta` table, check `schema_version` matches `SUPPORTED_SCHEMA = "1"`, return error if mismatch
- [ ] T017 [US1] Implement `CatalogManager` in `crates/astro-up-core/src/catalog/manager.rs`: orchestrate TTL check (from sidecar `fetched_at`) → acquire PID lock → fetch (conditional) → verify signature → store catalog + sidecar → release lock. Handle: fresh install (no local), TTL expired (conditional fetch), network failure (fallback to local), no local + no network (error).
- [ ] T018 [US1] Integration test for `CatalogManager` in `crates/astro-up-core/tests/catalog_integration.rs`: test with local fixture catalog (TTL not expired → no fetch), test with expired TTL (mock HTTP or use test server), test fallback when no network
- [ ] T019 [US1] Wire public API in `crates/astro-up-core/src/catalog/mod.rs`: re-export `CatalogManager`, `SqliteCatalogReader`, `PackageId`, `PackageSummary`, `VersionEntry`, `CatalogFilter`, `SearchResult`, `FetchResult`

**Checkpoint**: Catalog loads from local SQLite, fetches with ETag, verifies signatures. US1 is MVP-complete.

---

## Phase 4: User Story 2 — Search and Filter Software (Priority: P2)

**Goal**: FTS5 full-text search across name, description, tags, aliases, publisher. Filter by category and type.

**Independent Test**: Load fixture catalog, search for "nina" → N.I.N.A. appears. Filter by "guiding" → only guiding software. Search for nonexistent term → empty result.

- [ ] T020 [US2] Implement `SqliteCatalogReader::search(query) -> Vec<SearchResult>` in `crates/astro-up-core/src/catalog/reader.rs`: FTS5 MATCH query on `packages_fts`, JOIN with `packages` table, ORDER BY rank. Decode JSON columns (aliases, tags, dependencies) from TEXT.
- [ ] T021 [US2] Implement `SqliteCatalogReader::filter(CatalogFilter) -> Vec<PackageSummary>` in `crates/astro-up-core/src/catalog/reader.rs`: SQL WHERE on category and/or software_type columns
- [ ] T022 [US2] Implement `SqliteCatalogReader::list_all() -> Vec<PackageSummary>` in `crates/astro-up-core/src/catalog/reader.rs`: SELECT all from packages, decode JSON columns
- [ ] T023 [US2] Integration tests for search and filter in `crates/astro-up-core/tests/catalog_integration.rs`: search by name, search by alias (normalized), filter by category, filter by type, combined search+filter, empty results, insta snapshots for search ranking

**Checkpoint**: Search and filter working against fixture catalog. US2 independently testable.

---

## Phase 5: User Story 3 — Signature Verification (Priority: P3)

**Goal**: Comprehensive error handling and edge cases for signature verification

**Independent Test**: Valid sig accepted, invalid sig rejected (keeps previous catalog), missing sig reported with clear error.

Note: Core verification is implemented in T013-T014 (US1). This phase adds the edge cases and error UX.

- [ ] T024 [US3] Implement catalog replacement logic in `crates/astro-up-core/src/catalog/manager.rs`: on signature failure with existing valid local catalog, keep previous catalog and report warning. On signature failure with no previous catalog, return `CatalogSignatureInvalid` error.
- [ ] T025 [US3] Implement corrupt catalog detection in `crates/astro-up-core/src/catalog/reader.rs`: run `PRAGMA integrity_check` on open, if corrupt delete local file + sidecar and re-fetch. If re-fetch also fails, return `CatalogCorrupted` error.
- [ ] T026 [US3] Integration tests for signature edge cases in `crates/astro-up-core/tests/catalog_integration.rs`: tampered catalog with valid previous → keeps previous, corrupt SQLite → re-fetch, missing .minisig → clear error message

**Checkpoint**: All signature verification scenarios covered. US3 independently testable.

---

## Phase 6: User Story 4 — Resolve Software by ID (Priority: P4)

**Goal**: Exact ID lookup and version queries

**Independent Test**: Resolve "nina" → returns PackageSummary. Resolve "nonexistent" → NotFound error. Get versions for "nina" → returns all versions newest first.

- [ ] T027 [US4] Implement `SqliteCatalogReader::resolve(id: &PackageId) -> PackageSummary` in `crates/astro-up-core/src/catalog/reader.rs`: exact match on `packages.id`, return `CoreError::NotFound` if missing
- [ ] T028 [US4] Implement `SqliteCatalogReader::versions(id: &PackageId) -> Vec<VersionEntry>` in `crates/astro-up-core/src/catalog/reader.rs`: all versions for package ordered by `discovered_at` DESC
- [ ] T029 [US4] Implement `SqliteCatalogReader::latest_version(id: &PackageId) -> Option<VersionEntry>` in `crates/astro-up-core/src/catalog/reader.rs`: latest non-pre-release version (ORDER BY discovered_at DESC, WHERE pre_release = 0, LIMIT 1)
- [ ] T030 [US4] Integration tests for resolve and versions in `crates/astro-up-core/tests/catalog_integration.rs`: resolve known ID, resolve unknown ID, versions list order, latest_version excludes pre-releases, insta snapshots

**Checkpoint**: All 4 user stories complete and independently testable.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Error messages, logging, performance validation

- [ ] T031 [P] Add tracing instrumentation to all catalog operations in `crates/astro-up-core/src/catalog/*.rs`: `#[tracing::instrument]` on public methods, `tracing::info!` for fetch/verify/load events, `tracing::warn!` for fallback scenarios
- [ ] T032 [P] Implement actionable error messages (FR-010) in `crates/astro-up-core/src/error.rs`: ensure all Catalog* error variants have user-facing messages with remediation hints (e.g., "please update astro-up", "check your network")
- [ ] T033 Add performance benchmark for SC-001 and SC-002 in `crates/astro-up-core/tests/catalog_integration.rs`: assert catalog open <10ms, FTS5 search <50ms against fixture catalog
- [ ] T034 Run `cargo clippy -- -D warnings` and `cargo fmt --check` on all new code, fix any issues
- [ ] T035 Run full test suite (`just check`) and verify CI compatibility

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — MVP delivery point
- **US2 (Phase 4)**: Depends on Phase 2 + T016 (reader open) — can start after T016
- **US3 (Phase 5)**: Depends on T013, T017 (verify + manager) — hardening of US1
- **US4 (Phase 6)**: Depends on Phase 2 + T016 (reader open) — can start after T016
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (P1)**: No story dependencies. Self-contained. MVP.
- **US2 (P2)**: Needs T016 (reader.open) from US1. Can start in parallel after T016 completes.
- **US3 (P3)**: Needs T013+T017 from US1. Hardening — best done after US1 complete.
- **US4 (P4)**: Needs T016 (reader.open) from US1. Can start in parallel with US2.

### Parallel Opportunities

Within Phase 2:
- T004, T005, T006, T007 (all types) can run in parallel
- T008 (errors), T009 (lock), T010 (sidecar), T011 (fixtures), T012 (tests) can run in parallel

After Phase 2:
- US2 and US4 can start in parallel once T016 is done
- US3 should wait for US1 completion (builds on its verification logic)

---

## Parallel Example: Phase 2

```
# Launch all type definitions together:
Task: T004 "PackageId newtype in catalog/types.rs"
Task: T005 "PackageSummary struct in catalog/types.rs"
Task: T006 "VersionEntry struct in catalog/types.rs"
Task: T007 "Supporting types in catalog/types.rs"

# Launch all infrastructure together:
Task: T008 "Error variants in error.rs"
Task: T009 "PID lockfile in catalog/lock.rs"
Task: T010 "Sidecar in catalog/sidecar.rs"
Task: T011 "Test fixtures"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T012)
3. Complete Phase 3: User Story 1 (T013-T019)
4. **STOP and VALIDATE**: Catalog loads, fetches, verifies — independently testable
5. Checkpoint commit

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 → Catalog loads → MVP!
3. Add US2 → Search and filter working
4. Add US3 → Signature edge cases hardened
5. Add US4 → ID resolution and version queries
6. Polish → Logging, error UX, performance validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Fixture catalog must be generated from astro-up-manifests compiler with test key signing
- JSON column decoding (aliases, tags, dependencies) is needed in reader.rs — use serde_json::from_str on the TEXT values
- The `catalog.offline` config field is being removed (issue #118) — do NOT use it
- FTS5 alias normalization (stripping punctuation) is a compiler-side change (PR #79) — client searches normalized text
