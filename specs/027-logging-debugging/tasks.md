# Tasks: Logging and Debugging

**Input**: Design documents from `/specs/027-logging-debugging/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US5)
- Exact file paths included in descriptions

---

## Phase 1: Setup (Governance)

**Purpose**: Establish logging standards that all subsequent tasks follow

- [x] T001 [P] Add Constitution Principle VII (Observability) to `.specify/memory/constitution.md` — define log level semantics, instrument requirements, error suppression rules, sensitive data exclusion. Bump version to 1.1.0.
- [x] T002 [P] Add Logging & Error Handling section to `CLAUDE.md` under `## Coding Standards` — instrument conventions, structured fields, unwrap rules, frontend error handling rules.

**Checkpoint**: Governance in place — all subsequent tasks follow these standards.

---

## Phase 2: Foundational (Core Instrumentation Infrastructure)

**Purpose**: No new infrastructure needed. Existing tracing + LogPanel is the foundation. This phase instruments the core library modules that all user stories depend on.

**Note**: All Phase 2 tasks are parallelizable — they touch different files within the core crate.

- [x] T003 [P] [US1] Instrument `catalog/` module: add `#[tracing::instrument(skip_all, fields(...))]` to `fetch_catalog()` in `crates/astro-up-core/src/catalog/fetch.rs` (entry with URL, exit with status 304/200/error + duration), enhance timing/result logging in `manager.rs`, add package count on open in `reader.rs`, add signature result in `verify.rs`, add `warn!` on stale/corrupt lockfile PID parse failures in `lock.rs`.
- [x] T004 [P] [US1] Instrument `detect/` module: add `info!` entry/exit with package count and duration to `scan()` in `crates/astro-up-core/src/detect/scanner.rs`, add `debug!` per detection method and `warn!` on unexpected failures in `discovery.rs`, add `trace!` before each `.ok()` in the detection chain with method name + raw error. Scope: instrumentation + detection-chain `.ok()` patterns only; `unwrap()` triage is handled by T017.
- [x] T005 [P] [US1] Instrument `download/` module: add `#[tracing::instrument]` to `download()` in `crates/astro-up-core/src/download/mod.rs` with `info!` entry (URL, expected hash), `info!` exit (bytes, duration, cached/fresh), add retry count to existing hash mismatch warn, add `debug!` to `purge()` with file count deleted.
- [x] T006 [P] [US1] Instrument `install/` module: verify existing instrument in `crates/astro-up-core/src/install/mod.rs` is complete, add `#[tracing::instrument]` to `spawn_simple()` and `spawn_with_job_object()` in `process.rs` with `info!` entry (exe path, args, timeout), `info!` exit (exit code, duration), `warn!` on timeout, `error!` on spawn failure, `debug!` for Windows Job Object creation.
- [x] T007 [P] [US1] Instrument `engine/` module: add `debug!` entry/exit to `get_history()` in `crates/astro-up-core/src/engine/orchestrator.rs`, add `debug!` with graph stats to dependency resolution, add `warn!` to `list_acknowledged_packages()` `.ok().and_then()` chain, add `debug!` on PID parse failure in `lock.rs`, add `trace!` on version parse with raw input in `version_cmp.rs`.
- [x] T008 [P] [US1] Instrument `backup/` and remaining modules: add `trace!` per-file and `warn!` on skip in `crates/astro-up-core/src/backup/archive.rs`, add `debug!` on config load and `warn!` on validation fallback in `config/mod.rs`, add `info!` on lifecycle check and `warn!` on stale state in `lifecycle.rs`.

**Checkpoint**: All core modules instrumented with structured logging at appropriate levels.

---

## Phase 3: User Story 1 — Developer Debugging a Failed Installation (Priority: P1)

**Goal**: Complete operation traceability from command entry to root cause through structured log output with correlated `operation_id`.

**Independent Test**: Trigger an install with an invalid installer path. Verify logs show full trace with same `operation_id` from entry through failure.

### Implementation

- [ ] T009 [US1] Add unique `operation_id` generation and propagation to all top-level operations in `crates/astro-up-core/src/engine/orchestrator.rs` — `plan()` and `execute()` must generate UUID and pass to child spans via tracing fields. Implements FR-008a.
- [ ] T010 [P] [US1] Add `debug!` entry/exit logging to all CLI command handlers in `crates/astro-up-cli/src/commands/*.rs` with subcommand name and parsed args. Add `info!` on command dispatch in `main.rs`. Add `debug!` on state init in `state.rs`. MUST NOT duplicate core logging — boundary only (FR-013).
- [ ] T011 [P] [US1] Fill logging gaps in 5 GUI Tauri commands in `crates/astro-up-gui/src/commands.rs`: add `debug!` entry/exit to `get_versions` (line ~166), `list_backups` (line ~724) with count, `backup_preview` (line ~735); add `info!` exit to `delete_backup` (line ~749). Add `debug!` to `get_version` in `lib.rs`. Boundary logging only (FR-014).

**Checkpoint**: US1 complete — developer can trace any operation through structured logs with operation_id correlation.

---

## Phase 4: User Story 2 — End User Sees Meaningful Errors in GUI (Priority: P1)

**Goal**: Every GUI operation failure surfaces as a toast notification + error log entry. No silent failures.

**Independent Test**: Disconnect network, trigger catalog sync. Verify toast error appears and error log panel contains details.

### Implementation

- [x] T012 [P] [US2] Add global Vue error boundary: add `onErrorCaptured` handler in `frontend/src/App.vue` that logs to errorLog store and shows toast. Add `app.config.errorHandler` in `frontend/src/main.ts` for uncaught errors. Include rate limiting: max 3 toasts per 5 seconds (FR-015).
- [x] T013 [P] [US2] Add `onError` callbacks to all 8 VueQuery mutations missing them in `frontend/src/composables/useInvoke.ts`: `useSyncCatalog`, `useSaveConfig`, `useInstallSoftware`, `useUpdateSoftware`, `useUpdateAll`, `useScanInstalled`, `useCreateBackup`, `useRestoreBackup`, `useCancelOperation`. Each gets `toast.add({ severity: "error", ... })` + `errorLog.addEntry()`. Add global `QueryClient` `onError` default as safety net in main.ts or query client setup (FR-016).
- [x] T014 [P] [US2] Replace 3 `alert()` calls with PrimeVue toast in `frontend/src/components/settings/AboutSection.vue`: "You are on the latest version" → toast info (line ~29), "Failed to check for updates" → toast error (line ~32), "Update failed" → toast error (line ~49). Inject `useToast()` (FR-017).
- [x] T015 [P] [US2] Fix CatalogSection error handling: replace `console.error("Catalog re-download failed:", e)` with `toast.add({ severity: "error", ... })` + `errorLog.addEntry()` in `frontend/src/components/settings/CatalogSection.vue` (line ~25). Inject `useToast()` and `useErrorLog()` (FR-018).

**Checkpoint**: US2 complete — every GUI operation failure shows toast + error log entry. Zero alert() calls remain.

---

## Phase 5: User Story 3 — Project Governance for Future Code (Priority: P2)

**Goal**: Constitution and coding standards updated so future code follows logging conventions.

**Independent Test**: Read updated constitution and CLAUDE.md — verify rules are concrete and enforceable.

**Note**: T001 and T002 (Phase 1) already implement this user story. This phase is a validation checkpoint only.

**Checkpoint**: US3 complete — governance documents updated in Phase 1.

---

## Phase 6: User Story 4 — Silent Error Elimination in Backend (Priority: P2)

**Goal**: Triage and fix dangerous error suppression patterns. Every suppressed error either propagated or logged.

**Independent Test**: `rg "unwrap()" --type rust` on in-scope files shows 80%+ reduction. `rg "let _ =" --type rust` — remaining instances have adjacent log statements.

### Implementation

- [ ] T016 [P] [US4] Triage and fix dangerous `unwrap()` in `crates/astro-up-core/src/install/process.rs` — replace unwraps in Windows process spawning, FFI calls, and Job Object operations with `?` + `.map_err()` using appropriate `CoreError` variants. Log at `error!` on failure (FR-009).
- [ ] T017 [P] [US4] Triage and fix dangerous `unwrap()` in `crates/astro-up-core/src/detect/discovery.rs` — replace unwraps in PE file parsing, registry access, and WMI queries with `?` or `.ok()` + `trace!` logging. Focus on runtime-fallible `unwrap()` paths only; `.ok()` detection-chain logging was already added by T004 (FR-009).
- [ ] T018 [P] [US4] Triage and fix dangerous `unwrap()` in `crates/astro-up-core/src/backup/archive.rs` — replace unwraps in ZIP operations and file I/O with `?` + descriptive error context. Also audit remaining high-risk files: `engine/lock.rs`, `catalog/fetch.rs`, `download/mod.rs` (FR-009).
- [ ] T019 [P] [US4] Fix silent `let _ =` patterns: add `debug!` on send failure for UI event emissions in `crates/astro-up-gui/src/commands.rs` (~15 instances). Add `warn!` for file deletion and DB writes in `crates/astro-up-core/src/download/mod.rs` and `backup/mod.rs` (FR-011).
- [ ] T020 [P] [US4] Fix silent `.ok()` patterns in non-detection files: add `debug!` to PID parsing in `crates/astro-up-core/src/engine/lock.rs`, add `debug!` to version comparison in `engine/version_cmp.rs`, add `trace!` to Windows handle cleanup in `install/process.rs`. Detection-chain `.ok()` patterns in `discovery.rs` were already addressed by T004 (FR-012).

**Checkpoint**: US4 complete — dangerous error suppression eliminated. Remaining patterns have adjacent log statements.

---

## Phase 7: User Story 5 — Frontend Structured Logging (Priority: P3)

**Goal**: Frontend logging utility + debug-level logging for key user actions and composable lifecycles.

**Independent Test**: Open app, navigate views, trigger install. LogPanel at `debug` shows route transitions, mutation triggers, operation lifecycle.

### Implementation

- [x] T021 [US5] Create frontend logging utility in `frontend/src/utils/logger.ts` — export `logger` object with `debug(context, message)`, `info(context, message)`, `warn(context, message)`, `error(context, message)` methods. Writes to LogPanel store via existing `useLogPanel().addLog()` infrastructure (FR-019).
- [ ] T022 [P] [US5] Add debug logging to composables: `frontend/src/composables/useOperations.ts` (debug on start/complete/fail/cancel with operation details), `useCoreEvents.ts` (debug on event listener setup, warn on listener failure), `useInvoke.ts` (debug on mutation trigger with command name) (FR-021).
- [ ] T023 [P] [US5] Add debug logging to views and key components: route navigation in `frontend/src/views/*.vue` (onMounted debug), install/update/backup button clicks in `frontend/src/components/detail/DetailHero.vue` and `PackageRow.vue`, settings saves in `SettingsView.vue`. Use logger utility from T021 (FR-020).

**Checkpoint**: US5 complete — frontend actions visible in LogPanel at debug level.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Verification and cleanup

- [ ] T024 Run `cargo clippy -- -D warnings` across all crates — fix any warnings from new tracing imports or changed signatures.
- [ ] T025 Run `cargo test` across all crates — fix any test failures from unwrap removals or signature changes.
- [ ] T026 Run `pnpm lint && pnpm vue-tsc --noEmit` in frontend — fix any TypeScript or ESLint errors.
- [ ] T027 Run grep audits to verify success criteria: `rg "unwrap()" --type rust -c` on in-scope files (SC-003), `rg "let _ =" --type rust` remaining instances (SC-004), `rg "alert(" frontend/` zero results (SC-006).

**Checkpoint**: All quality checks pass. Feature complete.

---

## Task Dependencies

<!-- Machine-readable. Generated by /speckit.tasks, updated by /speckit.iterate.apply -->
<!-- Do not edit manually unless you also update GitHub issue dependencies -->

```toml
[graph]
# Phase 1: Setup — no blockers
[graph.T001]
blocked_by = []

[graph.T002]
blocked_by = []

# Phase 2: Foundational — after governance
[graph.T003]
blocked_by = ["T001"]

[graph.T004]
blocked_by = ["T001"]

[graph.T005]
blocked_by = ["T001"]

[graph.T006]
blocked_by = ["T001"]

[graph.T007]
blocked_by = ["T001"]

[graph.T008]
blocked_by = ["T001"]

# Phase 3: US1 — after core instrumentation
[graph.T009]
blocked_by = ["T003", "T004", "T005", "T006", "T007", "T008"]

[graph.T010]
blocked_by = ["T001"]

[graph.T011]
blocked_by = ["T001"]

# Phase 4: US2 — independent of backend work
[graph.T012]
blocked_by = []

[graph.T013]
blocked_by = []

[graph.T014]
blocked_by = []

[graph.T015]
blocked_by = []

# Phase 6: US4 — after core instrumentation (uses same files)
[graph.T016]
blocked_by = ["T006"]

[graph.T017]
blocked_by = ["T004"]

[graph.T018]
blocked_by = ["T008"]

[graph.T019]
blocked_by = ["T003", "T005", "T008", "T011"]

[graph.T020]
blocked_by = ["T004", "T006", "T007"]

# Phase 7: US5 — T021 first, then T022/T023
[graph.T021]
blocked_by = []

[graph.T022]
blocked_by = ["T021"]

[graph.T023]
blocked_by = ["T021"]

# Phase 8: Polish — after all implementation
[graph.T024]
blocked_by = ["T003", "T004", "T005", "T006", "T007", "T008", "T009", "T010", "T011", "T016", "T017", "T018", "T019", "T020"]

[graph.T025]
blocked_by = ["T024"]

[graph.T026]
blocked_by = ["T012", "T013", "T014", "T015", "T022", "T023"]

[graph.T027]
blocked_by = ["T024", "T025", "T026"]
```

## Parallel Opportunities

### Backend parallel group 1 (Phase 1 — immediate):
```
T001 (constitution) || T002 (CLAUDE.md)
```

### Backend parallel group 2 (Phase 2 — after T001):
```
T003 (catalog) || T004 (detect) || T005 (download) || T006 (install) || T007 (engine) || T008 (backup+config+lifecycle)
```

### Frontend parallel group (Phase 4 — immediate, no backend dependency):
```
T012 (error boundary) || T013 (mutation errors) || T014 (alert replacement) || T015 (catalog error)
```

### Error triage parallel group (Phase 6 — after respective Phase 2 tasks):
```
T016 (process unwraps) || T017 (discovery unwraps) || T018 (archive unwraps) || T019 (let_ patterns) || T020 (.ok() patterns)
```

### Frontend logging parallel (Phase 7 — after T021):
```
T022 (composable logging) || T023 (view/component logging)
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Phase 1: T001 + T002 (governance) — immediate
2. Phase 2: T003–T008 (core instrumentation) — parallel
3. Phase 3: T009–T011 (operation traceability) — after Phase 2
4. Phase 4: T012–T015 (GUI error handling) — parallel with Phase 2+3
5. **STOP and VALIDATE**: Trigger operations, verify logs + toasts

### Full Delivery

6. Phase 6: T016–T020 (error suppression triage) — parallel
7. Phase 7: T021–T023 (frontend logging) — sequential then parallel
8. Phase 8: T024–T027 (verification) — sequential

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Phase 5 (US3 governance) is covered by Phase 1 tasks T001/T002
- Backend and frontend work can proceed in parallel throughout
- Error triage (Phase 6) must follow instrumentation (Phase 2) to avoid editing same files twice
- Commit after each task or logical group
