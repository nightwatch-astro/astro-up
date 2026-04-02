# Tasks: CLI Interface

**Input**: Design documents from `/specs/015-cli-interface/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/cli-commands.rs, research.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US7)
- Paths relative to `crates/astro-up-cli/`

## Phase 1: Setup

**Purpose**: Add dependencies, wire module structure, basic entry point

- [ ] T001 Add dependencies to `crates/astro-up-cli/Cargo.toml`: ratatui 0.29, color-eyre 0.6, tracing-subscriber 0.3 (features: fmt, json, env-filter), tracing-appender 0.2, human-panic 2, dialoguer 0.11, tabled 0.17, tokio (features: full), serde_json
- [ ] T002 Create module structure: `src/commands/mod.rs`, `src/output/mod.rs`, `src/logging.rs` with empty module declarations
- [ ] T003 Wire `pub mod commands;`, `pub mod output;`, `pub mod logging;` in `src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: OutputMode, logging, Cli struct, signal handling — MUST complete before any command work

- [ ] T004 [P] Implement `OutputMode` enum (Interactive/Plain/Json) with `detect()` in `src/output/mod.rs` — TTY detection via `std::io::IsTerminal`, respects `--json` and `--quiet` flags
- [ ] T005 [P] Implement dual-layer tracing in `src/logging.rs`: stderr layer (compact, respects verbosity), JSON file layer via `tracing_appender::rolling::daily` to `{data_dir}/logs/`, non-blocking writer. Return the `WorkerGuard` to keep in scope.
- [ ] T006 Expand `Cli` struct in `src/lib.rs` with full `Commands` enum per contract: Show, Install, Update, Scan, Search, Backup, Restore, Config, SelfUpdate with all flags (`--json`, `--verbose`, `--quiet`, `--config`, `--dry-run`, `--allow-major`, `--yes`)
- [ ] T007 Update `src/main.rs`: install `human_panic::setup_panic!()` (release only), install `color_eyre`, init logging from T005, create `CancellationToken`, install Ctrl+C handler that trips the token, parse CLI, dispatch to `run()`
- [ ] T008 Implement command dispatch in `src/lib.rs` `run()`: match on `Commands` variants, pass `OutputMode` + `CancellationToken` to each handler. Stub all handlers to return `Ok(())`.
- [ ] T009 [P] Implement JSON output helper in `src/output/json.rs`: `pub fn print_json<T: Serialize>(value: &T) -> Result<()>` that serializes to stdout
- [ ] T010 [P] Implement table output helper in `src/output/table.rs`: `pub fn print_table<T: Tabled>(rows: &[T]) -> Result<()>` using tabled with styled headers
- [ ] T011 [P] Implement confirmation helper in `src/commands/mod.rs`: `pub fn confirm(prompt: &str, mode: &OutputMode, yes: bool) -> Result<bool>` — returns true immediately in Json mode or if `--yes`, uses dialoguer otherwise

**Checkpoint**: `cargo run -p astro-up-cli -- --help` shows all subcommands. Log file created on startup. All commands stub to Ok(()).

---

## Phase 3: User Story 1 — Show Software Status (Priority: P1)

**Goal**: `astro-up show` displays styled table of catalog software with install status.

**Independent Test**: Run `show installed` with known data, verify table output and JSON mode.

- [ ] T012 [US1] Implement `handle_show()` in `src/commands/show.rs`: parse ShowFilter (all/installed/outdated), load catalog + scan results from core, filter packages, render via OutputMode
- [ ] T013 [US1] Implement `show <package>` detail view in `src/commands/show.rs`: single package with name, version, category, detection method, backup count, dependencies
- [ ] T014 [US1] Implement `show backups [package]` in `src/commands/show.rs`: list backups via `BackupService::list()`, render as table with date, version, file count, size
- [ ] T015 [P] [US1] Implement first-run bootstrap in `src/commands/mod.rs`: `ensure_catalog()` checks catalog exists, auto-downloads with progress if missing; `ensure_scan_cache()` auto-triggers scan if no cached results
- [ ] T016 [US1] Wire show command in `src/lib.rs` dispatch to call `handle_show()` with bootstrap

**Checkpoint**: `astro-up show`, `show installed`, `show outdated`, `show <pkg>`, `show backups` all work. `--json` outputs valid JSON.

---

## Phase 4: User Story 4 — Scan for Installed Software (Priority: P4)

**Goal**: `astro-up scan` runs detection and shows results.

**Note**: Phase order ≠ priority order. Scan is implemented before US2/US3 because install/update depend on scan data being available.

**Independent Test**: Run `scan`, verify detection results displayed.

- [ ] T017 [US4] Implement `handle_scan()` in `src/commands/scan.rs`: create Scanner from core, run scan, display results as table (package, version, method), emit JSON if `--json`
- [ ] T018 [US4] Wire scan command in `src/lib.rs` dispatch

**Checkpoint**: `astro-up scan` and `astro-up scan --json` work.

---

## Phase 5: User Story 5 — Search Catalog (Priority: P5)

**Goal**: `astro-up search <query>` finds packages via FTS5.

**Independent Test**: Search for a known term, verify results.

- [ ] T019 [US5] Implement `handle_search()` in `src/commands/search.rs`: query catalog FTS5, display results as table, support `--json`
- [ ] T020 [US5] Wire search command in `src/lib.rs` dispatch

**Checkpoint**: `astro-up search <term>` and `--json` work.

---

## Phase 6: User Story 2 — Install Software (Priority: P2)

**Goal**: `astro-up install <package>` runs the orchestration pipeline with progress.

**Independent Test**: Install a test package, verify download + install. Try already-installed package, verify update prompt.

- [ ] T021 [P] [US2] Implement ratatui progress TUI in `src/output/progress.rs`: create `ProgressRenderer` that subscribes to core `Event` channel, renders download progress bar + install status via ratatui `Gauge` + `Paragraph`. Handle terminal setup/restore.
- [ ] T022 [US2] Implement `handle_install()` in `src/commands/install.rs`: check if package exists in catalog (suggest fuzzy matches if not), check if already installed (offer update via confirm), show plan table, run orchestration engine with progress renderer, display result. Support `--dry-run` (show plan only) and `--json` per FR-006/FR-008.
- [ ] T023 [US2] Wire install command in `src/lib.rs` dispatch with `CancellationToken`

**Checkpoint**: `astro-up install <pkg>` shows plan, confirms, downloads with progress, installs. `--dry-run` shows plan only. `--json` outputs structured result.

---

## Phase 7: User Story 3 — Update Software (Priority: P3)

**Goal**: `astro-up update <package>` or `update --all` updates installed packages.

**Independent Test**: Update a package with a known newer version.

- [ ] T024 [US3] Implement `handle_update()` in `src/commands/update.rs`: if `--all`, plan all outdated packages; if single, plan one. Show update plan table (package, current → target). Confirm unless `--yes`. Support `--allow-major`, `--dry-run`. Run engine with progress, display results.
- [ ] T025 [US3] Implement update plan table rendering in `src/output/table.rs`: `print_update_plan()` showing package name, current version, target version, size estimate
- [ ] T026 [US3] Wire update command in `src/lib.rs` dispatch with `CancellationToken`

**Checkpoint**: `astro-up update nina`, `update --all`, `--dry-run`, `--allow-major`, `--yes`, `--json` all work.

---

## Phase 8: User Story 6 — Backup and Restore (Priority: P6)

**Goal**: `astro-up backup <pkg>` and `astro-up restore <pkg>` manage config backups.

**Independent Test**: Create backup, list it, restore it.

- [ ] T027 [US6] Implement `handle_backup()` in `src/commands/backup.rs`: create backup via `BackupService`, display result (archive path, file count, size). Support `--json` output per FR-006.
- [ ] T028 [US6] Implement `handle_restore()` in `src/commands/restore.rs`: list available backups, let user pick via dialoguer Select (or `--yes` for latest), show `restore_preview()` file change summary, confirm, execute restore. Support `--json` output per FR-006.
- [ ] T029 [US6] Wire backup and restore commands in `src/lib.rs` dispatch

**Checkpoint**: `backup nina`, `restore nina`, `restore nina --path Profiles` work.

---

## Phase 9: User Story 7 — Configuration (Priority: P7)

**Goal**: `astro-up config init` and `config show` manage configuration.

**Independent Test**: Init config, show effective config.

- [ ] T030 [US7] Implement `handle_config()` in `src/commands/config.rs`: `init` generates default config.toml at platform config dir, `show` loads effective config and displays as TOML (or JSON with `--json`)
- [ ] T031 [US7] Wire config command in `src/lib.rs` dispatch

**Checkpoint**: `config init`, `config show`, `config show --json` work.

---

## Phase 10: Self-Update (FR-012)

**Purpose**: `astro-up self-update` checks for and installs CLI updates.

- [ ] T032 Implement `handle_self_update()` in `src/commands/self_update.rs`: check GitHub Releases for latest astro-up version, compare with current. If newer: download to temp file, swap via atomic rename (Unix) or rename-on-reboot (Windows). Support `--dry-run`. Support `--json` per FR-006.
- [ ] T033 Wire self-update command in `src/lib.rs` dispatch

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Error polish, exit codes, integration tests, fmt/clippy

- [ ] T034 Implement exit code handling in `src/main.rs`: map `Result` to exit code 0 (success), 1 (error), 2 (cancelled via Ctrl+C or declined prompt)
- [ ] T035 [P] Implement styled error output: on error, print context chain via color-eyre, append "Log file: {path}" pointing to today's log
- [ ] T036 [P] Write insta snapshot tests for `show` command output in `tests/cli_show.rs` using `assert_cmd`: run binary with test fixture catalog, snapshot stdout
- [ ] T037 [P] Write insta snapshot tests for `scan --json` and `show --json` in `tests/cli_json.rs`: validate JSON structure with `serde_json::from_str`
- [ ] T038 Run `cargo fmt` and `cargo clippy -- -D warnings` across workspace, fix any issues
- [ ] T039 Run `cargo test -p astro-up-cli` to verify all tests pass, update broken snapshot tests

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies, start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1, BLOCKS all command phases
- **Phase 3 (US1 Show)**: Depends on Phase 2 — first user-facing command
- **Phase 4 (US4 Scan)**: Depends on Phase 2 — moved ahead because US2/US3 need scan data
- **Phase 5 (US5 Search)**: Depends on Phase 2
- **Phase 6 (US2 Install)**: Depends on Phase 2 + T021 (progress TUI)
- **Phase 7 (US3 Update)**: Depends on Phase 6 (reuses progress TUI + install patterns)
- **Phase 8 (US6 Backup)**: Depends on Phase 2
- **Phase 9 (US7 Config)**: Depends on Phase 2
- **Phase 10 (Self-Update)**: Depends on Phase 2
- **Phase 11 (Polish)**: After all command phases

### User Story Dependencies

```
Phase 2 (Foundational)
  |-- US1 (Show) .............. read-only, first command
  |-- US4 (Scan) .............. detection, feeds show
  |-- US5 (Search) ............ catalog query
  |-- US2 (Install) ........... write, needs progress TUI
  |     `-- US3 (Update) ...... extends install patterns
  |-- US6 (Backup/Restore) .... independent
  |-- US7 (Config) ............ independent
  `-- Self-Update ............. independent
```

### Parallel Opportunities

- T004, T005, T009, T010, T011: foundational output helpers (different files)
- T015: bootstrap can be built alongside show implementation
- T021: progress TUI can be built alongside US1 show
- T036, T037: integration tests are independent of each other
- US5, US6, US7, Self-Update can all proceed in parallel after Phase 2

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T011)
3. Complete Phase 3: US1 Show (T012-T016)
4. STOP and VALIDATE: `astro-up show` renders table, `--json` works

### Incremental Delivery

1. Setup + Foundational: all commands stub, logging works, --help renders
2. US1 Show: first real output — see installed software
3. US4 Scan: detect software on the system
4. US5 Search: find software in catalog
5. US2 Install: first write operation with TUI progress
6. US3 Update: most frequent user action
7. US6 Backup/Restore: safety net
8. US7 Config: setup and debugging
9. Self-Update: maintenance
10. Polish: tests, error handling, fmt/clippy

---

## Task Dependencies

```toml
[graph]
T001 = { blocked_by = [] }
T002 = { blocked_by = ["T001"] }
T003 = { blocked_by = ["T002"] }
T004 = { blocked_by = ["T003"] }
T005 = { blocked_by = ["T003"] }
T006 = { blocked_by = ["T003"] }
T007 = { blocked_by = ["T005", "T006"] }
T008 = { blocked_by = ["T006", "T007"] }
T009 = { blocked_by = ["T003"] }
T010 = { blocked_by = ["T003"] }
T011 = { blocked_by = ["T003"] }
T012 = { blocked_by = ["T004", "T008", "T009", "T010"] }
T013 = { blocked_by = ["T012"] }
T014 = { blocked_by = ["T012"] }
T015 = { blocked_by = ["T004"] }
T016 = { blocked_by = ["T012", "T015"] }
T017 = { blocked_by = ["T004", "T008", "T010"] }
T018 = { blocked_by = ["T017"] }
T019 = { blocked_by = ["T004", "T008", "T010"] }
T020 = { blocked_by = ["T019"] }
T021 = { blocked_by = ["T004"] }
T022 = { blocked_by = ["T008", "T011", "T021"] }
T023 = { blocked_by = ["T022"] }
T024 = { blocked_by = ["T022", "T010"] }
T025 = { blocked_by = ["T010"] }
T026 = { blocked_by = ["T024"] }
T027 = { blocked_by = ["T004", "T008"] }
T028 = { blocked_by = ["T011", "T027"] }
T029 = { blocked_by = ["T027", "T028"] }
T030 = { blocked_by = ["T004", "T008"] }
T031 = { blocked_by = ["T030"] }
T032 = { blocked_by = ["T008"] }
T033 = { blocked_by = ["T032"] }
T034 = { blocked_by = ["T007"] }
T035 = { blocked_by = ["T005", "T034"] }
T036 = { blocked_by = ["T016"] }
T037 = { blocked_by = ["T016", "T018"] }
T038 = { blocked_by = ["T033", "T034", "T035"] }
T039 = { blocked_by = ["T038"] }
```

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story
- Fully cross-platform, no cfg(windows) in CLI crate (core handles platform)
- ratatui TUI is temporary during operations, not persistent
- Commit after each completed task or logical group
