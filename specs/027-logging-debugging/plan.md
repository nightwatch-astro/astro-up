# Implementation Plan: Logging and Debugging

**Branch**: `027-logging-debugging` | **Date**: 2026-04-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/027-logging-debugging/spec.md`

## Summary

Add comprehensive structured logging and error handling across the entire codebase (Rust backend + Vue frontend). Update constitution with Principle VII (Observability) and CLAUDE.md with logging conventions. Instrument all public functions in core with `#[tracing::instrument]`, triage dangerous `unwrap()`/`let _ =`/`.ok()` patterns, add VueQuery error handlers, global Vue error boundary, and frontend structured logging utility.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+), TypeScript 5, Vue 3
**Primary Dependencies**: tracing 0.1, tracing-subscriber 0.3 (fmt, json, env-filter), tracing-appender 0.2, PrimeVue 4 (toast), @tanstack/vue-query 5
**Storage**: N/A (no new storage — logging is file/stderr/LogPanel)
**Testing**: `cargo test`, `cargo clippy -- -D warnings`, `pnpm lint`, `pnpm vue-tsc --noEmit`
**Target Platform**: Windows primary (cross-platform CI on macOS/Linux)
**Project Type**: Desktop app (Tauri v2) + CLI
**Performance Goals**: N/A — logging overhead must not be perceptible to users
**Constraints**: Tight-loop functions use `trace!` events, not per-call spans. No new crate dependencies.
**Scale/Scope**: ~65 files modified across 3 Rust crates + Vue frontend

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First Crate Layout | PASS | No new crates. All changes within existing modules. |
| II. Platform Awareness | PASS | Logging is cross-platform. `cfg(windows)` paths get same treatment. |
| III. Test-First Integration Tests | PASS | Verification via clippy + existing tests + grep audits. No new test infrastructure needed. |
| IV. Thin Tauri Boundary | PASS | GUI commands add boundary logging only. Core owns detail logging. Anti-duplication enforced. |
| V. Spec-Driven Development | PASS | This is the spec. |
| VI. Simplicity | PASS | Uses existing infrastructure (tracing, toast, LogPanel). No new abstractions except one frontend logger utility. |
| VII. Observability (NEW) | N/A | This spec creates Principle VII. |

No violations. No complexity justification needed.

## Project Structure

### Documentation (this feature)

```text
specs/027-logging-debugging/
├── spec.md
├── plan.md              # This file
├── research.md          # Phase 0 (minimal — no new libraries)
├── data-model.md        # Phase 1 (N/A — no new data entities)
├── checklists/
│   ├── requirements.md
│   └── observability.md
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
# Governance files (Phase A)
.specify/memory/constitution.md    # Add Principle VII
CLAUDE.md                          # Add Logging & Error Handling section

# Rust backend — Core crate (Phase B + C)
crates/astro-up-core/src/
├── catalog/
│   ├── fetch.rs          # Instrument fetch_catalog, add timing
│   ├── manager.rs        # Enhance existing instrument, add timing
│   ├── reader.rs         # Add package count logging
│   ├── verify.rs         # Add signature result logging
│   └── lock.rs           # Add warn on PID parse .ok() failures
├── detect/
│   ├── scanner.rs        # Instrument scan(), add entry/exit/duration
│   └── discovery.rs      # Add debug per method, trace per .ok()
├── download/
│   └── mod.rs            # Instrument download(), add entry/exit/bytes/duration
├── install/
│   ├── mod.rs            # Verify existing instrument complete
│   └── process.rs        # CRITICAL: instrument spawn_*, add entry/exit/error
├── engine/
│   ├── orchestrator.rs   # Fill gaps: get_history, dependency resolution
│   ├── lock.rs           # Add debug on PID parse failure
│   └── version_cmp.rs    # Add trace on version parse
├── backup/
│   └── archive.rs        # Add trace per-file, warn on skip
├── config/
│   └── mod.rs            # Add debug on load, warn on validation fallback
└── lifecycle.rs          # Add info on check, warn on stale

# Rust backend — CLI crate (Phase D)
crates/astro-up-cli/src/
├── main.rs               # Info on command dispatch
├── state.rs              # Debug on state init
└── commands/*.rs          # Debug entry/exit per command

# Rust backend — GUI crate (Phase D)
crates/astro-up-gui/src/
├── commands.rs            # Fill 5 missing commands' logging
└── lib.rs                 # Fill get_version logging

# Vue frontend (Phase E + F)
frontend/src/
├── App.vue                       # Add onErrorCaptured
├── main.ts                       # Add app.config.errorHandler, QueryClient defaults
├── utils/
│   └── logger.ts                 # NEW: structured logging utility
├── composables/
│   ├── useInvoke.ts              # Add onError to 8 mutations + debug logging
│   ├── useOperations.ts          # Add debug lifecycle logging
│   └── useCoreEvents.ts          # Add debug event setup logging
├── views/*.vue                   # Add debug route/action logging
├── components/
│   ├── settings/
│   │   ├── AboutSection.vue      # Replace alert() with toast
│   │   └── CatalogSection.vue    # Replace console.error with toast+errorLog
│   └── detail/
│       └── DetailHero.vue        # Add debug on install/update clicks
```

**Structure Decision**: Cross-cutting changes across existing files. One new file: `frontend/src/utils/logger.ts`. No structural changes.

## Phase 0: Research

No NEEDS CLARIFICATION items. All technologies are already in use. No new dependencies.

Key implementation patterns (from existing codebase):
- **Instrument pattern**: `#[tracing::instrument(skip_all, fields(package = %name, operation_id = %id))]` — used in `catalog/manager.rs`, `backup/mod.rs`
- **Error context pattern**: `.map_err(|e| CoreError::CatalogFetchFailed { reason: e.to_string() })` — used throughout core
- **GUI boundary pattern**: `tracing::info!(operation = "install", package = %name, "starting install")` — used in `commands.rs`
- **FrontendLogLayer**: Custom tracing layer in `log_layer.rs` that forwards events to Vue via Tauri events

## Phase 1: Design

### Data Model

N/A — no new data entities. This spec adds observability to existing operations.

### Contracts

No new external interfaces. The frontend logging utility is an internal contract:

**`logger.ts` interface**:
```typescript
interface Logger {
  debug(context: string, message: string): void
  info(context: string, message: string): void
  warn(context: string, message: string): void
  error(context: string, message: string): void
}
```

Writes to LogPanel store via `useLogPanel().addLog()` (existing infrastructure).

### Quickstart

After implementation:
1. Run `just dev` — start Tauri dev server
2. Open LogPanel (bottom dock) and set filter to `debug`
3. Trigger any operation (sync catalog, install, scan)
4. Verify structured logs appear with `operation_id`, `package`, `duration_ms` fields
5. Trigger an error (disconnect network, sync catalog) — verify toast appears
6. Run `just check` — verify clippy/lint/test all pass

### Implementation Approach

**6 phases, ~20 tasks:**

| Phase | Description | Parallelizable | Files |
|-------|-------------|---------------|-------|
| A | Governance: constitution + CLAUDE.md | Yes (2 tasks) | 2 |
| B | Core instrumentation: 6 module groups | Yes (6 tasks) | ~20 |
| C | Error handling triage: unwrap/let_/ok | Yes (3 tasks) | ~15 |
| D | CLI + GUI boundary logging | Partially (2 tasks) | ~10 |
| E | Frontend error handling | Yes (4 tasks) | ~6 |
| F | Frontend structured logging | Sequential (3 tasks) | ~8 |

**Anti-duplication rule**: Core functions own detail logging. CLI/GUI add boundary logging only (command name, operation_id, duration, final result). Never re-log what core already reports.

**Tight-loop rule**: Functions in detection chains, per-file archive ops, and per-registry-key scans use `trace!` event macros, NOT `#[tracing::instrument]` spans, to avoid performance overhead.

**Error triage approach for Phase C**: Not all 679 unwraps need fixing. Categories:
- **Safe (skip)**: Mutex::lock, const regex, test code, infallible conversions (~600)
- **Fix (~40-60)**: File I/O, HTTP, SQLite, process spawn, Windows API (registry, WMI, PE, Job Objects)
- Replace with `?` + `.map_err()` or `.context()` for descriptive errors

### Constitution Re-check (Post-Design)

| Principle | Status |
|-----------|--------|
| I–VI | PASS (unchanged from pre-design) |
| VII (new) | Will be created by Phase A |

No violations introduced by design decisions.
