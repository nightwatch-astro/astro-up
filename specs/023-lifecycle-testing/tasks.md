# Tasks: Automated Package Lifecycle Testing & Detection Discovery

**Input**: Design documents from `/specs/023-lifecycle-testing/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Project initialization and dependency promotion

- [ ] T001 Promote `toml` from dev-dependency to regular dependency in `crates/astro-up-core/Cargo.toml` and add `serde_json` features needed for fallback_config serialization

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core modules that ALL user stories depend on

- [ ] T002 [P] Implement TOML manifest reader in `crates/astro-up-core/src/catalog/manifest.rs` — `ManifestReader::read()`, `read_by_id()`, `list_missing_detection()`. Add `pub mod manifest` to `crates/astro-up-core/src/catalog/mod.rs`. Include unit tests with sample TOML fixtures.
- [ ] T003 [P] Implement detection discovery module in `crates/astro-up-core/src/detect/discovery.rs` — types (`DiscoveryCandidate`, `DiscoveryConfidence`, `DiscoveryResult`, `ProbedLocation`), `DiscoveryScanner::new()` and `discover()` orchestration, `build_config()` for generating DetectionConfig with fallback chain (max depth 3). Add `pub mod discovery` to `crates/astro-up-core/src/detect/mod.rs`. Stub individual probe methods (registry, PE, file, etc.) returning empty results — they are wired in T004-T006.
- [ ] T004 Implement registry discovery probe in `crates/astro-up-core/src/detect/discovery.rs` — `probe_registry()`: enumerate all subkeys in 3 uninstall registry paths, match `DisplayName` against manifest `name` (primary) and package ID (fallback) using case-insensitive substring, extract DisplayVersion/InstallLocation/UninstallString/QuietUninstallString/Publisher. Return `Vec<DiscoveryCandidate>` with confidence based on version presence. Gate with `#[cfg(windows)]`.
- [ ] T005 [P] Implement PE discovery probe in `crates/astro-up-core/src/detect/discovery.rs` — `probe_pe_files()`: scan InstallLocation from registry results + common program dirs (`{program_files}\{name}\**\*.exe`, `{program_files_x86}\{name}\**\*.exe`), read PE headers via pelite for FileVersion and ProductName, convert absolute paths to `{program_files}` tokens via PathResolver. Gate with `#[cfg(windows)]` for directory scanning; PE parsing is cross-platform.
- [ ] T006 [P] Implement remaining discovery probes in `crates/astro-up-core/src/detect/discovery.rs` — `probe_file_exists()`, `probe_config_file()`, `probe_ascom()`, `probe_wmi()`, `probe_driver_store()`. Each returns `Vec<DiscoveryCandidate>` with appropriate confidence levels. Gate Windows-only probes with `#[cfg(windows)]`.
- [ ] T007 Implement lifecycle test runner in `crates/astro-up-core/src/lifecycle.rs` — `LifecycleRunner` with phase orchestration (download → install → detect → verify-install → uninstall → verify-removal → report), `LifecycleOptions`, `LifecycleReport`, `PhaseResult`, `PhaseStatus`, `LifecycleStatus` types. Include `resolve_latest_version()` (read `versions/{id}/` dir, sort by semver), `resolve_download_url()` (substitute `$version` in autoupdate URL), `config_to_toml()` (serialize DetectionConfig). Handle dry-run (skip install/uninstall), download_only (require install_dir, skip install/uninstall), timeouts, and cleanup-on-failure. Add `pub mod lifecycle` to `crates/astro-up-core/src/lib.rs`.

**Checkpoint**: Foundation ready — all core modules testable independently

---

## Phase 3: User Story 1 — Single Package Lifecycle Test (Priority: P1)

**Goal**: Run the full download → install → detect → uninstall cycle for one package and output a `[detection]` TOML config.

**Independent Test**: Run `astro-up lifecycle-test nina-app --manifest-path <path>` on a Windows runner and verify it produces a valid detection config.

- [ ] T008 [US1] Add `LifecycleTest` variant to `Commands` enum in `crates/astro-up-cli/src/lib.rs` with args: `package_id` (positional), `--manifest-path` (required), `--version` (optional), `--install-dir` (optional), `--dry-run` (flag), `--json` (global). Wire to handler in `run()`.
- [ ] T009 [US1] Implement lifecycle-test CLI handler in `crates/astro-up-cli/src/commands/lifecycle_test.rs` — parse args into `LifecycleOptions`, call `LifecycleRunner::run()`, output human-readable phase table or JSON based on `--json` flag. Write to `GITHUB_STEP_SUMMARY` if env var is set. Exit codes: 0/1/2/3/4 per FR-032. Add `pub mod lifecycle_test` to `crates/astro-up-cli/src/commands/mod.rs`.
- [ ] T010 [US1] Add snapshot tests for lifecycle-test CLI output in `crates/astro-up-cli/tests/cli_lifecycle_test.rs` — test JSON report structure and TOML detection config output using insta snapshots with mock manifest data.

**Checkpoint**: Single package lifecycle test works end-to-end on Windows via CLI

---

## Phase 4: User Story 2 — Dry Run & Matrix Sweep (Priority: P2)

**Goal**: Support dry-run mode for portable/data packages and matrix sweep for bulk discovery.

**Independent Test**: Run `--dry-run` for a portable package; trigger matrix sweep for 3 packages.

- [ ] T011 [US2] Add dry-run integration test in `crates/astro-up-core/tests/lifecycle_dry_run.rs` — verify that dry-run skips install/uninstall, still probes PE headers on downloaded .exe, and reports FileExists for non-PE files. Use a temp dir with a sample PE file.
- [ ] T012 [US2] Implement single-package GitHub Actions workflow in `.github/workflows/lifecycle-test.yml` — workflow_dispatch with inputs (`package_id` required, `version` optional, `dry_run` boolean). Checkout both repos (astro-up + manifests via Nightwatch app token). Build CLI binary. Run lifecycle-test. Upload JSON report as artifact. Always-run cleanup step with `continue-on-error: true`. Job summary from CLI output.
- [ ] T013 [US2] Add matrix sweep mode to `.github/workflows/lifecycle-test.yml` — prepare job that runs `ManifestReader::list_missing_detection()` (via a small shell script scanning TOML files for `[install]` without `[detection]`) and outputs JSON array. Test job uses `strategy.matrix` with `fromJson()`, `max-parallel: 5`, `fail-fast: false`. Each matrix job runs the single-package flow.
- [ ] T014 [US2] Add cross-repo PR creation step to `.github/workflows/lifecycle-test.yml` — after successful lifecycle test, use `gh` CLI with app token to: create branch `lifecycle/{package_id}` in manifests repo, update the manifest TOML with discovered `[detection]` section, create PR (or force-push + update existing PR if branch exists). PR body: package name, version, phase summary table, TOML code fence, workflow run link.

**Checkpoint**: Dry-run works for portables; matrix sweep discovers detection for multiple packages

---

## Phase 5: User Story 4 — Install Path Ledger Recording (Priority: P3)

**Goal**: Record discovered install path in the ledger after every successful install.

**Independent Test**: Install a package via CLI, verify ledger entry has correct install_path.

- [ ] T015 [US4] Modify `InstallerService::install()` in `crates/astro-up-core/src/install/mod.rs` — after successful install, run detection chain using the package's detection config (if available) to extract `install_path` from `DetectionResult::Installed`. Fallback chain: detection install_path → registry InstallLocation → default dir. Pass install_path to `ledger::record_install()`.
- [ ] T016 [US4] Update `record_install()` in `crates/astro-up-core/src/install/ledger.rs` to accept and forward `install_path: Option<PathBuf>` parameter. Update callers in `crates/astro-up-cli/src/commands/install.rs` and `crates/astro-up-gui/src/commands.rs` to pass detection config for install path resolution.

**Checkpoint**: Install path recorded for all successful installs

---

## Phase 6: Catalog Schema Expansion (Cross-Cutting)

**Goal**: Expand the detection table to support all DetectionConfig fields, including full recursive fallback chain.

**Independent Test**: Round-trip test: write detection config to catalog, read it back, verify all fields preserved.

- [ ] T017 [P] Update catalog reader `detection_config()` in `crates/astro-up-core/src/catalog/reader.rs` — expand SQL SELECT to read new columns (file_path, version_regex, product_code, upgrade_code, inf_provider, device_class, inf_name, fallback_config). Deserialize `fallback_config` JSON blob into `Box<DetectionConfig>`. Handle missing columns gracefully (old catalogs).
- [ ] T018 [P] Update test fixture in `crates/astro-up-core/tests/create_fixture_catalog.rs` — update detection table CREATE TABLE statement with new columns. Add test data with fallback_config JSON. Rename `path` column to `file_path`, replace `fallback_method`/`fallback_path` with `fallback_config TEXT`.
- [ ] T019 Update manifests repo compiler schema in `nightwatch-astro/astro-up-manifests` — modify `crates/compiler/src/schema.rs` (detection table DDL) and `crates/compiler/src/compile.rs` (read all TOML `[detection]` fields, serialize fallback as JSON blob in `fallback_config` column). Submit as PR to manifests repo.
- [ ] T020 Add round-trip integration test in `crates/astro-up-core/tests/catalog_detection_roundtrip.rs` — create a fixture catalog with full detection configs (all field types, multi-level fallback), read via `SqliteCatalogReader::detection_config()`, verify all fields match including deserialized fallback chain.

**Checkpoint**: Full DetectionConfig survives TOML → compile → SQLite → read round-trip

---

## Phase 7: User Story 5 — Regression Testing (Priority: P3)

**Goal**: The lifecycle workflow doubles as an install/uninstall regression test suite.

**Independent Test**: Run lifecycle test for a known-good package, verify all phases pass with detailed report.

- [ ] T021 [US5] Add regression test documentation and known-good package list in `specs/023-lifecycle-testing/regression-packages.md` — list 3-5 packages that should always pass (e.g., packages with known-good install/uninstall cycles), document expected detection methods and install paths for each.
- [ ] T022 [US5] Add `--report-file <path>` flag to lifecycle-test CLI in `crates/astro-up-cli/src/commands/lifecycle_test.rs` — write the full JSON report to a file in addition to stdout. Used by CI for artifact upload and diff comparisons between runs.

**Checkpoint**: Lifecycle test works as regression suite with persistent reports

---

## Phase 8: Polish & Cross-Cutting Concerns

- [ ] T023 [P] Add CI job for lifecycle-test compilation in `.github/workflows/ci.yml` — ensure the lifecycle-test subcommand compiles on all platforms (clippy, test). Windows-only integration tests gated behind `#[cfg(windows)]`.
- [ ] T024 Run quickstart.md validation — verify all documented commands work, exit codes are correct, workflow inputs match implementation.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1. BLOCKS all user stories.
- **Phase 3 (US1 - Single Test)**: Depends on Phase 2
- **Phase 4 (US2 - Dry Run & Matrix)**: Depends on Phase 3 (workflow uses CLI built in US1)
- **Phase 5 (US4 - Ledger Recording)**: Depends on Phase 2 only (independent of US1/US2)
- **Phase 6 (Schema Expansion)**: Depends on Phase 2 only (independent of US1/US2/US4)
- **Phase 7 (US5 - Regression)**: Depends on Phase 3 (needs working lifecycle-test CLI)
- **Phase 8 (Polish)**: Depends on all previous phases

### User Story Dependencies

- **US1** (Single Test): Needs foundational modules → sequential after Phase 2
- **US2** (Dry Run + Matrix): Needs US1's CLI → sequential after Phase 3
- **US4** (Ledger Recording): Independent of US1/US2 → can parallel with Phase 3
- **Schema Expansion** (Cross-cutting): Independent of US1/US2/US4 → can parallel with Phase 3
- **US5** (Regression): Needs US1's CLI → sequential after Phase 3

### Parallel Opportunities

- T002, T003 can run in parallel (manifest reader + discovery types)
- T005, T006 can run in parallel (PE probe + other probes) after T003
- Phase 5 (US3) and Phase 6 (US4) can run in parallel with Phase 3 (US1)
- T017, T018 can run in parallel (reader update + fixture update)
- T023 can run in parallel with any phase

---

## Parallel Example: Phase 2 (Foundational)

```
Wave 1: T001 (setup)
Wave 2: T002 (manifest reader) || T003 (discovery types + orchestration)
Wave 3: T004 (registry probe — needs T003) || T005 (PE probe — needs T003) || T006 (other probes — needs T003)
Wave 4: T007 (lifecycle runner — needs T002, T003)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002-T007)
3. Complete Phase 3: US1 Single Package Test (T008-T010)
4. **STOP and VALIDATE**: Run lifecycle-test on Windows for a known package
5. Deploy CLI, test manually

### Incremental Delivery

1. Setup + Foundational → Core modules ready
2. US1 (Single Test) → CLI works end-to-end → **MVP**
3. US4 (Schema) can parallel with US2 → Detection table supports all fields
4. US2 (Dry Run + Matrix) → Workflow automation → Full CI integration
5. US3 (Ledger) → Install path tracking → Backup system enabled
6. US5 (Regression) → Regression suite → Ongoing quality gate

---

## Task Dependencies

```toml
[graph]
# Phase 1: Setup
[graph.T001]
blocked_by = []

# Phase 2: Foundational
[graph.T002]
blocked_by = ["T001"]

[graph.T003]
blocked_by = ["T001"]

[graph.T004]
blocked_by = ["T003"]

[graph.T005]
blocked_by = ["T003"]

[graph.T006]
blocked_by = ["T003"]

[graph.T007]
blocked_by = ["T002", "T003"]

# Phase 3: US1 — Single Package Test
[graph.T008]
blocked_by = ["T007"]

[graph.T009]
blocked_by = ["T007", "T008"]

[graph.T010]
blocked_by = ["T009"]

# Phase 4: US2 — Dry Run & Matrix Sweep
[graph.T011]
blocked_by = ["T007"]

[graph.T012]
blocked_by = ["T009"]

[graph.T013]
blocked_by = ["T012"]

[graph.T014]
blocked_by = ["T012"]

# Phase 5: US4 — Install Path Ledger
[graph.T015]
blocked_by = ["T007"]

[graph.T016]
blocked_by = ["T015"]

# Phase 6: Catalog Schema Expansion
[graph.T017]
blocked_by = ["T007"]

[graph.T018]
blocked_by = ["T007"]

[graph.T019]
blocked_by = ["T017"]

[graph.T020]
blocked_by = ["T017", "T018"]

# Phase 7: US5 — Regression Testing
[graph.T021]
blocked_by = ["T009"]

[graph.T022]
blocked_by = ["T009"]

# Phase 8: Polish
[graph.T023]
blocked_by = ["T009"]

[graph.T024]
blocked_by = ["T012", "T016", "T020", "T022"]
```

## Notes

- [P] tasks = different files, no dependencies on incomplete parallel tasks
- [Story] label maps task to specific user story for traceability
- Windows-only tasks (T004, T005, T006, T015, T016) gated with `#[cfg(windows)]`
- Cross-repo task T019 requires separate PR to nightwatch-astro/astro-up-manifests
- Commit after each task or logical group
