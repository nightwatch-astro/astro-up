# Tasks: CLI Command Wiring

**Input**: Design documents from `/specs/024-cli-wiring/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Dependencies, binary rename, shared state infrastructure

- [ ] T001 Add indicatif 0.18 (with tokio feature) to `crates/astro-up-cli/Cargo.toml`, add `[[bin]] name = "astro-up" path = "src/main.rs"` section, remove anyhow from dependencies
- [ ] T002 Create CliState struct in `crates/astro-up-cli/src/state.rs` — holds data_dir, db_path, config, catalog_manager, backup_service (mirror GUI's AppState pattern per contracts/cli-state.rs)
- [ ] T003 Update `crates/astro-up-cli/src/lib.rs` run() to initialize CliState once and pass &CliState to all command handlers; update all handler signatures in commands/mod.rs

---

## Phase 2: Foundational (Progress Renderer)

**Purpose**: Event-driven progress display used by all operation commands

- [ ] T004 Rewrite `crates/astro-up-cli/src/output/progress.rs` — replace raw stderr writes with indicatif ProgressBar for Interactive mode (ProgressStyle template: `{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})`), keep text fallback for Plain mode, suppress for Quiet mode, stream JSON events for Json mode
- [ ] T005 Add event forwarding helper in `crates/astro-up-cli/src/commands/mod.rs` — spawn a tokio task that receives from broadcast::Receiver<Event> and calls the progress renderer based on OutputMode; return the JoinHandle so callers can await it

**Checkpoint**: Progress infrastructure ready — operation commands can now use it

---

## Phase 3: User Story 1 — Scan for Installed Software (Priority: P1)

**Goal**: `astro-up scan` calls core Scanner, persists results to ledger, displays as table/JSON

**Independent Test**: Run `astro-up scan` on Windows with astrophotography software installed; verify packages appear with correct versions

- [ ] T006 [US1] Wire scan command in `crates/astro-up-cli/src/commands/scan.rs` — create CatalogPackageSource + SqliteLedgerStore from CliState, instantiate Scanner, call scanner.scan(), persist results to ledger, display as table (tabled) or JSON
- [ ] T007 [US1] Add ScanResultRow struct to `crates/astro-up-cli/src/commands/scan.rs` with Tabled + Serialize derives (package_id, name, version, method, status columns)
- [ ] T008 [US1] Update snapshot test in `crates/astro-up-cli/tests/snapshots/` for scan help output if signature changed
- [ ] T009 [US1] Add Windows integration test in `crates/astro-up-cli/tests/cli_integration.rs` — `#[cfg(target_os = "windows")]` test that scan produces valid table output with at least the column headers

**Checkpoint**: `astro-up scan` works end-to-end on Windows, persists to ledger

---

## Phase 4: User Story 2 — Install a Package with Progress (Priority: P1)

**Goal**: `astro-up install <id>` resolves from catalog, downloads with progress bar, installs, verifies detection

**Independent Test**: Run `astro-up install astap` on clean Windows; verify ASTAP downloads, installs silently, and is detected

- [ ] T010 [US2] Wire install command in `crates/astro-up-cli/src/commands/install.rs` — use CliState to create Orchestrator (same pattern as GUI's run_orchestrated_operation), subscribe to event channel via T005 helper, execute with cancel_token, handle download retry (3 attempts), report install failures immediately
- [ ] T011 [US2] Add install plan display — before executing, show tabled plan (package, version, size) and call confirm() unless --yes or --dry-run
- [ ] T012 [US2] Add post-install verification — after install succeeds, re-run detection for the package and report success/failure
- [ ] T013 [US2] Add Windows integration test in `crates/astro-up-cli/tests/cli_integration.rs` — test `install nonexistent-pkg` shows fuzzy search suggestions, test `install <id> --dry-run` shows plan without executing

**Checkpoint**: `astro-up install` works end-to-end with progress bars on Windows

---

## Phase 5: User Story 3 — Update Installed Packages (Priority: P1)

**Goal**: `astro-up update --all` scans, compares against catalog, plans updates, executes with progress

**Independent Test**: Install older version of package, run `astro-up update --all`, verify it detects and updates

- [ ] T014 [US3] Wire update command in `crates/astro-up-cli/src/commands/update.rs` — scan via Scanner (or read ledger), compare installed vs catalog latest, build UpdateRequest, create Orchestrator, show plan, execute after confirmation
- [ ] T015 [US3] Handle single-package update (`astro-up update nina-app`) — resolve package, check if installed (from ledger), compare version, update if outdated
- [ ] T016 [US3] Add integration test in `crates/astro-up-cli/tests/cli_integration.rs` — test `update --dry-run` shows plan, test `--json update` returns valid JSON with updates array

**Checkpoint**: `astro-up update` works end-to-end with progress on Windows

---

## Phase 6: User Story 4 — Show Installed and Outdated (Priority: P2)

**Goal**: `show installed` and `show outdated` read from ledger without re-scanning

**Independent Test**: Run `scan` then `show installed` — verify output matches; run `show outdated` — verify only outdated packages shown

- [ ] T017 [US4] Wire `show installed` in `crates/astro-up-cli/src/commands/show.rs` — read from SqliteLedgerStore via CliState, filter for installed packages, display as table with name/version/method columns
- [ ] T018 [US4] Wire `show outdated` in `crates/astro-up-cli/src/commands/show.rs` — read installed from ledger, compare each against catalog latest_version, show only packages where installed < latest
- [ ] T019 [US4] Add integration test — verify `show installed` succeeds with empty ledger (shows "run scan first" message), verify JSON output has packages array

**Checkpoint**: `show installed` and `show outdated` work from ledger data

---

## Phase 7: User Story 5 — Create a Backup (Priority: P2)

**Goal**: `astro-up backup <id>` looks up backup config from catalog, calls BackupService

**Independent Test**: Install N.I.N.A., run `astro-up backup nina-app`, verify archive created

- [ ] T020 [US5] Wire backup command in `crates/astro-up-cli/src/commands/backup.rs` — use CliState catalog_manager to look up package backup config (config_paths), resolve install_path from ledger, call BackupService.create() with event channel for progress
- [ ] T021 [US5] Handle missing backup config — if package has no backup_paths in catalog, show clear message; if package not in ledger, suggest running scan first
- [ ] T022 [US5] Add integration test — verify `backup nonexistent-pkg` produces appropriate error, verify `--json backup` returns valid JSON

**Checkpoint**: `astro-up backup` creates archives using catalog backup configs

---

## Phase 8: User Story 6 — Self-Update (Priority: P3)

**Goal**: `astro-up self-update` checks GitHub Releases for newer version, downloads and applies

**Independent Test**: Run `astro-up self-update --dry-run`, verify it reports current vs latest version

- [ ] T023 [US6] Implement GitHub Releases version check in `crates/astro-up-cli/src/commands/self_update.rs` — GET `https://api.github.com/repos/nightwatch-astro/astro-up/releases/latest` via reqwest, parse tag_name, compare against CARGO_PKG_VERSION using semver
- [ ] T024 [US6] Implement update download and apply — download the appropriate asset for the platform, replace the running binary (or prompt user for manual steps on Windows where the binary is locked)
- [ ] T025 [US6] Add integration test — verify `self-update --dry-run` returns valid output with current_version, verify `--json self-update` has current_version and status fields

**Checkpoint**: `astro-up self-update` checks and applies updates

---

## Phase 9: User Story 7 — Binary Name (Priority: P3)

**Goal**: Binary installs as `astro-up` not `astro-up-cli`

- [ ] T026 [US7] Update all assert_cmd test helpers in `crates/astro-up-cli/tests/` — change `Command::cargo_bin("astro-up-cli")` to `Command::cargo_bin("astro-up")` across cli_show.rs, cli_json.rs, cli_integration.rs
- [ ] T027 [US7] Update all insta snapshots that reference the binary name `astro-up-cli` in `crates/astro-up-cli/tests/snapshots/`
- [ ] T028 [US7] Verify `cargo install --path crates/astro-up-cli` installs as `astro-up` binary

**Checkpoint**: Binary is `astro-up` everywhere

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Test fixture, CI, cleanup

- [ ] T029 Create fixture catalog `crates/astro-up-cli/tests/fixtures/test-catalog.db` — pre-built SQLite with 3-5 packages (including detection configs, version entries, backup configs) for offline CI testing
- [ ] T030 Add Windows CI integration test job in `.github/workflows/ci.yml` — run `cargo test -p astro-up-cli` on windows-latest with the fixture catalog, exercise scan/install/update --dry-run
- [ ] T031 [P] Add cancellation integration test in `crates/astro-up-cli/tests/cli_integration.rs` — start a long operation, send SIGINT, verify exit code 2 and clean shutdown
- [ ] T032 [P] Update CLI help text in snapshot tests and docs/reference/cli.md to reflect newly wired commands (remove "requires Windows" stubs, update descriptions)
- [ ] T033 Run `cargo clippy -p astro-up-cli -- -D warnings` and `cargo test -p astro-up-cli` — fix any warnings or failures

---

## Task Dependencies

<!-- Machine-readable. Generated by /speckit.tasks, updated by /speckit.iterate.apply -->
<!-- Do not edit manually unless you also update GitHub issue dependencies -->

```toml
[graph]

# Phase 1: Setup
[graph.T001]
blocked_by = []

[graph.T002]
blocked_by = ["T001"]

[graph.T003]
blocked_by = ["T002"]

# Phase 2: Progress renderer
[graph.T004]
blocked_by = ["T001"]

[graph.T005]
blocked_by = ["T004"]

# US1: Scan
[graph.T006]
blocked_by = ["T003", "T005"]

[graph.T007]
blocked_by = ["T006"]

[graph.T008]
blocked_by = ["T006"]

[graph.T009]
blocked_by = ["T006"]

# US2: Install
[graph.T010]
blocked_by = ["T003", "T005"]

[graph.T011]
blocked_by = ["T010"]

[graph.T012]
blocked_by = ["T010"]

[graph.T013]
blocked_by = ["T010"]

# US3: Update
[graph.T014]
blocked_by = ["T006", "T010"]

[graph.T015]
blocked_by = ["T014"]

[graph.T016]
blocked_by = ["T014"]

# US4: Show installed/outdated
[graph.T017]
blocked_by = ["T006"]

[graph.T018]
blocked_by = ["T017"]

[graph.T019]
blocked_by = ["T017"]

# US5: Backup
[graph.T020]
blocked_by = ["T003"]

[graph.T021]
blocked_by = ["T020"]

[graph.T022]
blocked_by = ["T020"]

# US6: Self-update
[graph.T023]
blocked_by = ["T003"]

[graph.T024]
blocked_by = ["T023"]

[graph.T025]
blocked_by = ["T023"]

# US7: Binary name
[graph.T026]
blocked_by = ["T001"]

[graph.T027]
blocked_by = ["T026"]

[graph.T028]
blocked_by = ["T027"]

# Polish
[graph.T029]
blocked_by = ["T006"]

[graph.T030]
blocked_by = ["T029"]

[graph.T031]
blocked_by = ["T005"]

[graph.T032]
blocked_by = ["T006", "T010", "T014", "T017", "T020", "T023"]

[graph.T033]
blocked_by = ["T032"]
```

## Parallel Opportunities

After Phase 1 (Setup) + T004/T005 (progress renderer):
- **US1 (Scan)** and **US2 (Install)** can start in parallel (different files)
- **US5 (Backup)** and **US6 (Self-update)** can start in parallel with US1/US2
- **US7 (Binary name)** can start immediately after T001

After US1 completes:
- **US3 (Update)** can start (depends on scan wiring)
- **US4 (Show)** can start (depends on scan/ledger wiring)

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Progress renderer (T004-T005)
3. Complete Phase 3: Scan (T006-T009)
4. Complete Phase 4: Install (T010-T013)
5. **STOP and VALIDATE**: `astro-up scan` and `astro-up install` work end-to-end on Windows

### Incremental Delivery

5. Add Update (Phase 5) → validates scan + orchestrator integration
6. Add Show installed/outdated (Phase 6) → validates ledger persistence
7. Add Backup (Phase 7) → validates catalog backup config lookup
8. Add Self-update (Phase 8) → standalone, no core dependencies
9. Binary rename (Phase 9) → polish
10. CI + fixture catalog (Phase 10) → quality gate
