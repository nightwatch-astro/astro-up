# Tasks: Stability and Hardening Audit

**Input**: Design documents from `/specs/030-stability-hardening-audit/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US8)
- Includes exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add new dependency and create shared validation module skeleton

- [ ] T001 Add `parking_lot = "0.12"` to `crates/astro-up-gui/Cargo.toml` dependencies
- [ ] T002 Create `crates/astro-up-core/src/validation.rs` module with public function signatures for `validate_zip_entry()`, `validate_within_allowlist()`, `validate_backup_sources()` and register module in `crates/astro-up-core/src/lib.rs`

---

## Phase 2: Foundational (Path Validation Utilities)

**Purpose**: Implement shared validation functions used by US1 and US3. MUST complete before user story work begins.

- [ ] T003 Implement `validate_zip_entry(entry_name, allowed_root) -> Result<PathBuf>` in `crates/astro-up-core/src/validation.rs` — normalize path, reject `..` components, absolute paths, symlinks, Windows reparse points (`FILE_ATTRIBUTE_REPARSE_POINT` 0x400) and junctions via ZIP external attributes
- [ ] T004 Implement `validate_within_allowlist(path, allowed_dirs) -> Result<()>` in `crates/astro-up-core/src/validation.rs` — path component matching via `Path::starts_with()`, reject symlinks and mount points
- [ ] T005 Implement `validate_backup_sources(paths, max_aggregate_bytes) -> Result<Vec<PathBuf>>` in `crates/astro-up-core/src/validation.rs` — validate paths exist, aren't symlinks/mounts, total size under limit (1 GB default)
- [ ] T006 Add integration tests for validation utilities in `crates/astro-up-core/src/validation.rs` — test `..` traversal, absolute paths, symlink entries, Windows reparse points, path allowlist matching, aggregate size enforcement

**Checkpoint**: Validation utilities ready — US1 and US3 can proceed

---

## Phase 3: US1 - Safe Backup Restore (Priority: P1)

**Goal**: Backup restore validates every file path, rejects traversal attacks, overwrites existing files safely.

**Independent Test**: Create a test archive with `../../` entries, symlinks, and absolute paths. Restore and verify all malicious paths rejected, valid paths succeed.

- [ ] T007 [US1] Harden `resolve_restore_target()` in `crates/astro-up-core/src/backup/archive.rs` — replace naive path joining with `validate_zip_entry()`, handle existing files with overwrite (full replacement)
- [ ] T008 [US1] Fix path filter matching in `crates/astro-up-core/src/backup/archive.rs` — replace string `starts_with()` in restore path filter with `Path::starts_with()` component-based matching
- [ ] T009 [P] [US1] Parameterize SQL LIMIT clause in `crates/astro-up-core/src/engine/history.rs:126` — replace `write!(sql, " LIMIT {limit}")` with parameterized query placeholder
- [ ] T010 [P] [US1] Enforce maximum download file size in `crates/astro-up-core/src/download/stream.rs` — add `const MAX_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024`, check `Content-Length` header before download, add running byte counter for chunked/missing Content-Length responses
- [ ] T011 [P] [US1] Add config file size validation in `crates/astro-up-core/src/config/` loader — check file size before reading (reject > 10 MB), add size check before deserialization
- [ ] T012 [US1] Add integration tests for backup restore path traversal in `crates/astro-up-core/src/backup/` tests — test archives with `../../` entries, absolute paths, symlinks, reparse points, valid entries, mixed valid/invalid, and overwrite behavior

**Checkpoint**: Backup restore is safe — path traversal vulnerability eliminated

---

## Phase 4: US3 - Validated Command Inputs (Priority: P1)

**Goal**: All path-accepting Tauri commands validate inputs against application-controlled directory allowlist.

**Independent Test**: Invoke each command with paths outside the allowlist and verify rejection.

- [ ] T013 [US3] Add path validation to `clear_directory()` in `crates/astro-up-gui/src/commands.rs` — derive allowlist from runtime config (backup dir, per-package config paths, app cache dir), validate with `validate_within_allowlist()`
- [ ] T014 [US3] Add path validation to `create_backup()` in `crates/astro-up-gui/src/commands.rs` — validate source paths with `validate_backup_sources()` using 1 GB aggregate limit
- [ ] T015 [US3] Add path validation to `delete_backup()` in `crates/astro-up-gui/src/commands.rs` — validate archive path is within backup directory via `validate_within_allowlist()`

**Checkpoint**: All command inputs validated — no filesystem operations outside app directories

---

## Phase 5: US2 - Application Survives Command Failures (Priority: P1)

**Goal**: No single panic cascades to crash all subsequent commands. Critical tasks auto-restart.

**Independent Test**: Trigger a panic in a mock handler, verify subsequent commands execute normally.

- [ ] T016 [US2] Migrate `std::sync::Mutex` to `parking_lot::Mutex` in `crates/astro-up-gui/src/state.rs` — replace import, update type aliases. Remove all `.unwrap()` after `.lock()` in `crates/astro-up-gui/src/state.rs`
- [ ] T017 [US2] Remove all `.lock().unwrap()` patterns in `crates/astro-up-gui/src/commands.rs` (33 call sites) — `.lock()` now returns `MutexGuard` directly with `parking_lot::Mutex`
- [ ] T018 [US2] Add panic boundaries to all `tokio::spawn` and `std::thread::spawn` sites in `crates/astro-up-gui/src/lib.rs` and `crates/astro-up-core/src/download/stream.rs` — wrap async blocks with `catch_unwind(AssertUnwindSafe(...))`, log panic payload at `error!` level with task name
- [ ] T019 [US2] Implement critical task restart with sliding window in `crates/astro-up-gui/src/lib.rs` — track restart count per task (3 per 10 min), on budget exhaustion emit Tauri event `task-budget-exhausted` for frontend notification, continue in degraded mode
- [ ] T020 [P] [US2] Replace blocking `std::fs::*` with `tokio::fs::*` in async handlers in `crates/astro-up-gui/src/commands.rs` — update `remove_file()`, `remove_dir_all()` (~5 call sites)

**Checkpoint**: App survives panics — no cascading mutex poisoning

---

## Phase 6: US4 - Reliable Error Reporting (Priority: P2)

**Goal**: No silent failures or crashes from `unwrap()`. Users see error feedback. Logs have context.

**Independent Test**: Trigger error conditions in hardened code paths, verify error toasts and log entries.

- [ ] T021 [P] [US4] Replace production `unwrap()` calls in `crates/astro-up-core/src/detect/scanner.rs:368` (UUID parse), `crates/astro-up-core/src/detect/mod.rs:227-228` (JSON serde), and remaining ~12 production sites — use `?` propagation or `.map_err()` with context
- [ ] T022 [P] [US4] Replace `unwrap()` in `crates/astro-up-cli/src/commands/self_update.rs:99` (`latest_release`) — use `.ok_or_else(|| anyhow!("no release found"))?`
- [ ] T023 [US4] Wrap all `invoke()` calls in `frontend/src/components/shared/SurveyDialog.vue` (4 calls) with try/catch — log via logger utility, show error toast via PrimeVue toast
- [ ] T024 [P] [US4] Wrap all `invoke()` calls in `frontend/src/components/shared/AssetSelectionDialog.vue` (2 calls) and `frontend/src/views/SettingsView.vue` with try/catch + logger + toast
- [ ] T025 [US4] Audit all `.ok()` (58 sites) and `let _ =` (13 sites) patterns across all Rust crates — add `warn!` or `debug!` before discarding meaningful errors; document intentionally silent ones with inline comments. Priority targets: `crates/astro-up-core/src/detect/search.rs:137,156` (COM cleanup → `debug!`), `crates/astro-up-core/src/download/stream.rs` (event sends → `debug!`), `crates/astro-up-cli/src/logging.rs:34` (log dir → `warn!`)

**Checkpoint**: All errors reported or intentionally documented — no silent failures

---

## Phase 7: US7 - Frontend Lifecycle Safety (Priority: P2)

**Goal**: Rapid navigation doesn't cause stale data or state updates on unmounted components.

**Independent Test**: Mount component, trigger invoke, unmount before response, verify no state update.

- [ ] T026 [US7] Implement mounted-flag pattern in `frontend/src/composables/useInvoke.ts` or create a `useSafeInvoke` wrapper — `ref(true)` set to `false` on `onUnmounted`, check before applying result. Apply to direct `invoke()` calls in SurveyDialog, AssetSelectionDialog, SettingsView
- [ ] T027 [US7] Add stale update guards to `watch()` hooks in `frontend/src/App.vue`, `frontend/src/components/` (BackupGroup, LogPanel, OperationsDock, PackageDetailView) — increment request counter on each trigger, discard responses from outdated requests
- [ ] T028 [US7] Verify VueQuery stale handling — ensure `queryClient.cancelQueries()` is called on unmount for long-running queries (catalog sync) in `frontend/src/composables/useInvoke.ts`

**Checkpoint**: Frontend safe under rapid navigation — no stale data

---

## Phase 8: US5 - Simplified Codebase (Priority: P3) — DEFERRED

**Deferred to spec 031 (multi-crate refactoring)**. Module-level splitting is superseded by a full crate extraction that will decompose `astro-up-core` into domain-specific crates. Doing module splits now would be throwaway work.

- [ ] T029 [US5] DEFERRED — Consolidate command handlers → spec 031
- [ ] T030 [US5] DEFERRED — Decompose commands.rs → spec 031
- [ ] T031 [US5] DEFERRED — Decompose orchestrator → spec 031
- [ ] T032 [US5] DEFERRED — Simplify orchestrator trait → spec 031

---

## Phase 9: US6 - Complete Observability (Priority: P3)

**Goal**: Every async I/O function has tracing instrumentation with structured fields. Log levels consistent.

**Independent Test**: Run operations and verify tracing spans with structured fields appear in output.

- [ ] T033 [P] [US6] Add `#[tracing::instrument(skip_all, fields(...))]` to all public async I/O functions in `crates/astro-up-core/src/` — minimum fields: operations (`operation_id`, `package`), network (`url`, `duration_ms`), file I/O (`path`), database (`query_type`, `table`)
- [ ] T034 [P] [US6] Add `#[tracing::instrument(skip_all, fields(...))]` to all public async I/O functions in `crates/astro-up-cli/src/` and `crates/astro-up-gui/src/`
- [ ] T035 [P] [US6] Add structured context fields to error logs in `crates/astro-up-core/src/install/process.rs:77,192,288` (add `path`, `process_name`, `exit_code`), `crates/astro-up-core/src/download/stream.rs` (add `url`, `bytes_downloaded`), `crates/astro-up-core/src/detect/search.rs` (add `com_object_id`)
- [ ] T036 [US6] Fix log level inconsistencies — `crates/astro-up-gui/src/lib.rs:78,86` update check `debug!` → `info!`; `frontend/src/composables/useUpdateQueue.ts:117-119` replace empty catch with `logger.warn()`; standardize inline `addEntry()` calls to use `logger` utility in `frontend/src/App.vue`

**Checkpoint**: Full observability — all I/O functions instrumented, log levels consistent

---

## Phase 10: US8 - Optimized Dependencies (Priority: P3)

**Goal**: Trimmed feature flags, up-to-date packages.

**Independent Test**: `cargo check` passes with trimmed features; `pnpm outdated` shows no minor/patch updates.

- [ ] T037 [P] [US8] Replace `tokio = { features = ["full"] }` with `features = ["rt-multi-thread", "time", "sync", "macros"]` in `crates/astro-up-cli/Cargo.toml` — run `cargo check -p astro-up-cli` to verify
- [ ] T038 [P] [US8] Run `pnpm update` in `frontend/` for minor/patch version updates — verify with `pnpm build` and `pnpm test`

**Checkpoint**: Dependencies optimized — no unnecessary features compiled

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all user stories

- [ ] T039 Run `cargo clippy -- -D warnings` across all crates to verify zero `unwrap()` in production paths and no new warnings
- [ ] T040 Run `cargo fmt --check` and `pnpm lint` to verify formatting compliance
- [ ] T041 Run full test suite (`just test`) and verify all existing + new tests pass
- [ ] T042 Validate quickstart.md verification checklist — run each check item manually

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
blocked_by = []

# Phase 2: Foundational
[graph.T003]
blocked_by = ["T002"]

[graph.T004]
blocked_by = ["T002"]

[graph.T005]
blocked_by = ["T002"]

[graph.T006]
blocked_by = ["T003", "T004", "T005"]

# Phase 3: US1 - Safe Backup Restore
[graph.T007]
blocked_by = ["T003"]

[graph.T008]
blocked_by = ["T003"]

[graph.T009]
blocked_by = []

[graph.T010]
blocked_by = []

[graph.T011]
blocked_by = []

[graph.T012]
blocked_by = ["T007", "T008"]

# Phase 4: US3 - Validated Command Inputs
[graph.T013]
blocked_by = ["T004"]

[graph.T014]
blocked_by = ["T005"]

[graph.T015]
blocked_by = ["T004"]

# Phase 5: US2 - App Survives Failures
[graph.T016]
blocked_by = ["T001"]

[graph.T017]
blocked_by = ["T016"]

[graph.T018]
blocked_by = []

[graph.T019]
blocked_by = ["T018"]

[graph.T020]
blocked_by = ["T017"]

# Phase 6: US4 - Reliable Error Reporting
[graph.T021]
blocked_by = []

[graph.T022]
blocked_by = []

[graph.T023]
blocked_by = []

[graph.T024]
blocked_by = []

[graph.T025]
blocked_by = []

# Phase 7: US7 - Frontend Lifecycle Safety
[graph.T026]
blocked_by = ["T023", "T024"]

[graph.T027]
blocked_by = []

[graph.T028]
blocked_by = []

# Phase 8: US5 - DEFERRED to spec 031
[graph.T029]
blocked_by = ["DEFERRED"]

[graph.T030]
blocked_by = ["DEFERRED"]

[graph.T031]
blocked_by = ["DEFERRED"]

[graph.T032]
blocked_by = ["DEFERRED"]

# Phase 9: US6 - Complete Observability
[graph.T033]
blocked_by = []

[graph.T034]
blocked_by = []

[graph.T035]
blocked_by = []

[graph.T036]
blocked_by = []

# Phase 10: US8 - Optimized Dependencies
[graph.T037]
blocked_by = []

[graph.T038]
blocked_by = []

# Phase 11: Polish
[graph.T039]
blocked_by = ["T021", "T022", "T025", "T033", "T034"]

[graph.T040]
blocked_by = ["T039"]

[graph.T041]
blocked_by = ["T040"]

[graph.T042]
blocked_by = ["T041"]
```

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: T003–T005 depend on T002 (module skeleton)
- **US1 (Phase 3)**: T007/T008 depend on T003 (zip validation). T009–T011 are independent.
- **US3 (Phase 4)**: Depends on T004/T005 (allowlist/backup validation)
- **US2 (Phase 5)**: T016 depends on T001 (parking_lot dep). T017 depends on T016.
- **US4 (Phase 6)**: Fully independent — can run in parallel with US1/US2/US3
- **US7 (Phase 7)**: T026 depends on T023/T024 (invoke wrapping from US4)
- **US5 (Phase 8)**: **MUST wait for commands.rs changes** from Phases 4+5 (T013–T015, T017, T020)
- **US6 (Phase 9)**: T034 depends on T030 (commands decomposition). Others independent.
- **US8 (Phase 10)**: Fully independent
- **Polish (Phase 11)**: Depends on all implementation tasks

### Critical Path

T002 → T003 → T007/T008 → T012 (US1 complete)
T001 → T016 → T017 → T020 → T029 → T030 → T034 (commands.rs sequential chain)

### Parallel Opportunities

| Parallel group | Tasks | Reason |
|----------------|-------|--------|
| Setup | T001, T002 | Different files |
| Foundational | T003, T004, T005 | Same file but independent functions |
| US1 core | T009, T010, T011 | Different files in core crate |
| US4 Rust | T021, T022 | Different crates |
| US4 Frontend | T023, T024 | Different Vue files |
| US6 instrumentation | T033, T034, T035, T036 | Different crates/files |
| US8 | T037, T038 | Rust vs frontend |
| Cross-phase | US4 (T021–T025) can run alongside US1/US2/US3 | No file overlap |

## Implementation Strategy

### MVP (P1 stories only)

1. Setup + Foundational → T001–T006
2. US1 (Safe Backup Restore) → T007–T012
3. US3 (Validated Command Inputs) → T013–T015
4. US2 (App Survives Failures) → T016–T020
5. **STOP and VALIDATE**: All CRITICAL/HIGH findings resolved

### Incremental Delivery

1. P1 stories (above) → Security/stability hardened
2. US4 + US7 (P2) → Error reporting + frontend safety
3. US5 + US6 + US8 (P3) → Code quality + observability + deps
4. Polish → Full validation

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- commands.rs has a strict sequential chain: Phase 4 (validation) → Phase 5 (parking_lot) → Phase 8 (decomposition)
- T025 (silent error audit) is the broadest task — touches ~71 sites across all crates
- T033/T034 (tracing instrumentation) touch many files but are additive-only (low conflict risk)
