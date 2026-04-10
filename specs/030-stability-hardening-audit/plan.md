# Implementation Plan: Stability and Hardening Audit

**Branch**: `030-stability-hardening-audit` | **Date**: 2026-04-10 | **Spec**: [spec.md](spec.md)

## Summary

Systematic codebase hardening across 7 phases ordered by crash risk. Addresses a CRITICAL path traversal vulnerability in backup restore, HIGH-severity mutex poisoning cascade in Tauri commands, production `unwrap()` crashes, unguarded frontend invokes, code structure debt, logging gaps, and dependency bloat. No new features — all changes improve robustness of existing functionality.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+), TypeScript 5, Vue 3
**Primary Dependencies**: tauri 2, parking_lot (new), tokio, reqwest, rusqlite, vue-query 5, PrimeVue 4
**Storage**: SQLite via rusqlite (bundled)
**Testing**: cargo test + insta (snapshots), vitest (frontend)
**Target Platform**: Windows (primary), macOS/Linux (CI)
**Project Type**: Desktop app (Tauri v2)
**Performance Goals**: Path validation adds <1ms per operation (trivial string checks)
**Constraints**: No new user-facing features, no breaking changes to existing behavior
**Scale/Scope**: ~15K lines Rust, ~8K lines TypeScript/Vue across 3 crates + frontend

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | Decomposition keeps modules in `astro-up-core`; no new crates |
| II. Platform Awareness | PASS | FR-003 adds Windows reparse point/junction handling; `cfg(windows)` gated |
| III. Test-First | PASS | Integration tests for path traversal, panic recovery; `insta` for snapshot testing |
| IV. Thin Tauri Boundary | PASS | Path validation logic moves to core (not commands.rs); commands remain thin adapters |
| V. Spec-Driven | PASS | Full spec with 31 FRs, 10 SCs |
| VI. Simplicity | PASS | Removes complexity (dead abstraction, duplication). Validation is straightforward string/path checks |
| VII. Observability | PASS | FR-023–025 directly implement this principle. Constitution allows `unwrap()` on `Mutex::lock()` — after parking_lot migration, this exception becomes moot (no `Result` returned) |

No violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/030-stability-hardening-audit/
├── spec.md
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (minimal — no new entities)
├── quickstart.md        # Phase 1 output
├── checklists/
│   ├── requirements.md
│   └── hardening.md
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
  astro-up-core/src/
  ├── backup/
  │   ├── archive.rs          # FR-001–003,007: Path traversal fixes
  │   └── mod.rs
  ├── catalog/
  │   └── reader.rs           # FR-008: SQL parameterization
  ├── config/
  │   └── loader.rs           # FR-010: Config size validation
  ├── download/
  │   └── stream.rs           # FR-009: Download size limits
  ├── engine/
  │   ├── orchestrator.rs     # FR-020–022: Decompose into submodules
  │   ├── orchestrator/       # NEW: plan.rs, execute.rs, events.rs
  │   ├── history.rs          # FR-008: SQL LIMIT parameterization
  │   └── version_cmp.rs
  ├── detect/
  │   ├── scanner.rs          # FR-015: Remove unwrap()
  │   ├── mod.rs              # FR-015: Remove unwrap()
  │   └── search.rs           # FR-018: Log COM cleanup errors
  ├── install/
  │   └── process.rs          # FR-024: Structured error logging
  ├── lifecycle.rs
  └── validation.rs           # NEW: Shared path validation utilities

  astro-up-cli/src/
  ├── commands/
  │   └── self_update.rs      # FR-015: Remove unwrap()
  └── logging.rs              # FR-018: Log directory creation failure

  astro-up-gui/src/
  ├── commands.rs              # FR-004–006,011,014,019: Decompose + validate + parking_lot
  ├── commands/                # NEW: backup.rs, catalog.rs, operations.rs, config.rs
  ├── state.rs                 # FR-011: parking_lot::Mutex migration
  └── lib.rs                   # FR-012–013,025: Panic boundaries + log levels

frontend/src/
  ├── composables/
  │   ├── useInvoke.ts         # FR-027: Mounted-flag pattern
  │   ├── useUpdateQueue.ts    # FR-026: Fix empty catch
  │   └── useCoreEvents.ts
  ├── components/shared/
  │   ├── SurveyDialog.vue     # FR-017: Add error handling to invoke()
  │   └── AssetSelectionDialog.vue  # FR-017: Add error handling
  ├── views/
  │   └── SettingsView.vue     # FR-017: Guard invoke() calls
  └── App.vue                  # FR-025,028: Log levels + stale guard
```

**Structure Decision**: No new crates. Path validation utilities added as `core/src/validation.rs` module (Constitution I: modules-first). GUI commands decomposed into submodules under `commands/` directory. Orchestrator decomposed into submodules under `engine/orchestrator/`.

## Phase A: Path Traversal & Input Validation

**Priority**: CRITICAL/HIGH | **Files**: 8 | **Risk**: HIGH (security-critical changes)

### A1: Path Validation Utilities (`core/src/validation.rs`)

New module providing shared path validation functions used by backup, commands, and download:

- `validate_zip_entry(entry_name, allowed_root) -> Result<PathBuf>`: Normalizes path, rejects `..`, absolute paths, symlinks, Windows reparse points/junctions. Returns canonical path within `allowed_root`.
- `validate_within_allowlist(path, allowed_dirs) -> Result<()>`: Checks path is within one of the allowed directories using path component matching (not string prefix).
- `validate_backup_sources(paths, max_aggregate_bytes) -> Result<Vec<PathBuf>>`: Validates paths exist, aren't symlinks/mounts, total size under limit.

### A2: Backup Restore Hardening (`core/src/backup/archive.rs`)

- `resolve_restore_target()`: Replace naive path joining with `validate_zip_entry()`. Reject entries with `..`, absolute paths, symlinks, reparse points.
- Path filter matching: Replace `starts_with()` string comparison with path component comparison via `Path::starts_with()` (stdlib method operates on components, not bytes).
- Existing files at restore targets: overwrite (full replacement per FR-002).

### A3: Command Path Validation (`gui/src/commands.rs` → `commands/backup.rs`)

- `clear_directory()`: Derive allowlist from runtime config (backup dir, per-package config paths, app cache dir). Validate with `validate_within_allowlist()`.
- `create_backup()`: Validate sources with `validate_backup_sources()`, 1 GB aggregate limit.
- `delete_backup()`: Validate path is within backup directory via `validate_within_allowlist()`.

### A4: SQL Parameterization (`core/src/engine/history.rs`, `core/src/catalog/reader.rs`)

- `history.rs:126`: Replace `write!(sql, " LIMIT {limit}")` with parameterized `" LIMIT ?N"` and add limit to params vector.
- Verify `catalog/reader.rs` already parameterized (audit confirmed safe — no changes needed).

### A5: Download Size Enforcement (`core/src/download/stream.rs`)

- When `Content-Length` present: reject if > 2 GB before starting download.
- When `Content-Length` absent: add running byte counter during streaming, abort at 2 GB.
- Constant: `const MAX_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;`

### A6: Config Size Validation (`core/src/config/`)

- At config load time: check file size before reading. Reject configs > 10 MB (generous limit; typical configs are <100 KB).

## Phase B: Mutex Poisoning & Concurrency

**Priority**: HIGH | **Files**: 4 | **Risk**: MEDIUM (behavioral change in lock semantics)

### B1: parking_lot Migration (`gui/src/state.rs`, `gui/src/commands.rs`, `gui/src/lib.rs`)

**Research confirmed**: `parking_lot::Mutex` is a drop-in replacement for `std::sync::Mutex` in Tauri state. `lock()` returns `MutexGuard` directly (no `Result`), eliminating all 33 `.lock().unwrap()` call sites.

Migration:
1. Add `parking_lot = "0.12"` to `astro-up-gui/Cargo.toml`
2. Replace `use std::sync::Mutex` with `use parking_lot::Mutex` in `state.rs`
3. Remove all `.unwrap()` after `.lock()` calls (33 sites in `commands.rs`, `state.rs`, `lib.rs`)
4. No changes to `app.manage()` calls — Tauri's `State<'_, T>` only requires `T: Send + Sync + 'static`

### B2: Panic Boundaries on Spawned Tasks (`gui/src/lib.rs`, `core/src/download/stream.rs`)

For all 8 `tokio::spawn` / `std::thread::spawn` sites:
- Wrap async block in `catch_unwind` (requires `AssertUnwindSafe` for futures)
- Log panic payload at `error!` level with task name
- Critical tasks (event forwarding, progress streaming): restart with sliding window tracker (3 per 10 min). On budget exhaustion: emit Tauri event for frontend notification, continue without task.
- Non-critical tasks: log and let die.

### B3: Async File Operations (`gui/src/commands.rs`)

Replace `std::fs::remove_file()` / `std::fs::remove_dir_all()` in async command handlers with `tokio::fs::*` equivalents (~5 call sites).

## Phase C: Error Handling

**Priority**: HIGH/MEDIUM | **Files**: ~12 | **Risk**: LOW (replacing panics with graceful errors)

### C1: Production `unwrap()` Elimination (Rust)

| File | Line(s) | Pattern | Fix |
|------|---------|---------|-----|
| `detect/scanner.rs` | 368 | `id.parse().unwrap()` | `id.parse().map_err(\|e\| ...)?` |
| `detect/mod.rs` | 227–228 | JSON serde round-trip `.unwrap()` | `serde_json::to_value(&v).map_err(...)? ` |
| `cli/commands/self_update.rs` | 99 | `latest_release.unwrap()` | `.ok_or_else(\|\| anyhow!("no release found"))?` |
| Other ~12 sites | various | `.unwrap()` outside tests | Case-by-case: `?` or `.unwrap_or_else()` with logging |

### C2: Frontend `invoke()` Error Handling (Vue)

- `SurveyDialog.vue` (4 calls): Wrap in try/catch, log via logger, show error toast
- `AssetSelectionDialog.vue` (2 calls): Same pattern
- `SettingsView.vue`: Audit and wrap unguarded calls
- Pattern: `try { await invoke(...) } catch (e) { logger.error(...); toast.add({ severity: 'error', ... }) }`

### C3: Silent Error Audit (`.ok()` / `let _ =`)

Audit all 58 `.ok()` and 13 `let _ =` sites. For each:
- If meaningful failure (data loss, resource leak): add `warn!` or `debug!` before discarding
- If non-meaningful (stderr write, UI cleanup on shutdown): document with inline comment
- Specific targets: `detect/search.rs:137,156` (COM cleanup → `debug!`), `download/stream.rs` (event sends → `debug!`), `cli/logging.rs:34` (log dir creation → `warn!`)

## Phase D: Code Structure

**Priority**: HIGH/MEDIUM | **Files**: 4-6 | **Risk**: MEDIUM (large refactors, regression risk)

### D1: Command Handler Consolidation (`gui/src/commands.rs`)

Replace three near-identical handlers (`install_software`, `update_software`, `update_all`) with:

```
enum OperationType { Install, Update, UpdateAll }

async fn run_operation(
    app: AppHandle,
    operation: OperationType,
    packages: Vec<String>,
    state: /* ... */
) -> Result<OperationResult, String> { ... }
```

Each original handler becomes a thin wrapper calling `run_operation()` with the appropriate enum variant.

### D2: Commands Module Decomposition (`gui/src/commands.rs` → `gui/src/commands/`)

After consolidation, split 1093-line `commands.rs` into:
- `commands/mod.rs` — re-exports, shared types
- `commands/backup.rs` — `create_backup`, `restore_backup`, `delete_backup`, `clear_directory`
- `commands/catalog.rs` — `get_catalog`, `sync_catalog`, `search_catalog`
- `commands/operations.rs` — `run_operation` + `install_software`, `update_software`, `update_all`
- `commands/config.rs` — `get_config`, `update_config`

Each file targets <300 lines.

### D3: Orchestrator Decomposition (`core/src/engine/orchestrator.rs`)

Decompose `run_orchestrated_operation()` (~160 lines) into:
- `build_operation_plan()` — dependency resolution, version comparison, plan construction
- `execute_plan()` — iterate packages, dispatch download/install/update
- `forward_events()` — broadcast channel setup, event type mapping

Simplify orchestrator trait: inline into `UpdateOrchestrator` struct (single implementation). Remove unused type parameters. Address 7 `#[allow(dead_code)]` fields: remove fields not needed for current functionality, add TODO comments for fields needed by upcoming specs (T014–T016).

## Phase E: Logging

**Priority**: MEDIUM | **Files**: ~15 | **Risk**: LOW (additive changes)

### E1: Tracing Instrumentation

Add `#[tracing::instrument(skip_all, fields(...))]` to all public async functions with I/O across all three crates. Minimum fields per category:

| Category | Required fields |
|----------|----------------|
| Operations | `operation_id`, `package` |
| Network | `url`, `duration_ms` |
| File I/O | `path` |
| Database | `query_type`, `table` |

### E2: Structured Error Context

Update `error!` / `warn!` calls in:
- `install/process.rs:77,192,288` — add `path`, `process_name`, `exit_code`
- `download/stream.rs` — add `url`, `bytes_downloaded`
- `detect/search.rs` — add `com_object_id` for COM cleanup

### E3: Log Level Fixes

- `gui/src/lib.rs:78,86` — update check: `debug!` → `info!`
- Frontend: `useUpdateQueue.ts:117-119` — replace empty catch with `logger.warn()`
- Frontend: standardize inline `addEntry()` to use `logger` utility

## Phase F: Frontend Lifecycle

**Priority**: MEDIUM | **Files**: ~6 | **Risk**: LOW

### F1: Mounted-Flag Pattern (`composables/useInvoke.ts`)

Create a composable wrapper or modify existing `useInvoke`:

```typescript
const isMounted = ref(true)
onUnmounted(() => { isMounted.value = false })

// Before applying invoke result:
if (!isMounted.value) return
```

Apply to all direct `invoke()` call sites outside VueQuery (SurveyDialog, AssetSelectionDialog, SettingsView).

### F2: Stale Update Guards

For `watch()` hooks in App.vue, BackupGroup, LogPanel, OperationsDock, PackageDetailView:
- Add request counter. Increment on each trigger. Compare counter at response time. Discard if stale.

### F3: VueQuery Stale Handling

VueQuery already handles most staleness via query keys. Verify `queryClient.cancelQueries()` is called on component unmount for long-running queries (catalog sync).

## Phase G: Dependencies

**Priority**: LOW | **Files**: 2 | **Risk**: LOW

### G1: Tokio Feature Trimming

`astro-up-cli/Cargo.toml`: Replace `features = ["full"]` with `features = ["rt-multi-thread", "time", "sync", "macros"]`. Verify with `cargo check`.

### G2: NPM Package Updates

Run `pnpm update` for minor/patch versions. Major upgrades (TypeScript 6, vue-tsc 3) deferred to separate evaluation.

## Implementation Order

Phases execute in order A → B → C → D → E → F → G. Within each phase, tasks are independent unless noted.

**Cross-phase dependencies**:
- Phase D (commands decomposition) should happen AFTER Phase A (path validation in commands) and Phase B (parking_lot in commands) to avoid merge conflicts
- Phase E (logging) can partially overlap with Phase C (error handling) since both touch error paths

**Testing strategy**:
- Phase A: Integration tests with malicious ZIP archives, path traversal payloads, oversized downloads
- Phase B: Tests triggering panics in mock handlers, verifying subsequent commands succeed
- Phase C: Tests for each hardened code path with error injection
- Phase D: Existing tests must pass after refactoring (no behavioral changes)
- Phase E: Verify tracing spans appear in test output
- Phase F: Component tests with rapid mount/unmount cycles
- Phase G: `cargo check` + `pnpm build` after dependency changes
