# Implementation Plan: Install Orchestration Engine

**Branch**: `012-install-orchestration` | **Date**: 2026-03-31 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/012-install-orchestration/spec.md`

## Summary

Build the install orchestration engine in `astro-up-core` that coordinates the full update pipeline: version comparison → download → backup → install → verify. Extends the existing `Version` type with date and custom regex parsing, adds dependency resolution via topological sort, implements update policies, operation history logging, global locking, and running-process detection.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: semver (existing), chrono (existing), sysinfo (existing from spec 005), regex (new — custom version format), rusqlite (existing — operations table), tokio (existing — async runtime), tokio-util (existing from spec 010/011 — CancellationToken)
**Storage**: SQLite — operations table in existing app database
**Testing**: cargo test, insta (snapshots), rstest (fixtures), tempfile (filesystem), pretty_assertions
**Target Platform**: Windows primary, cross-platform CI (macOS/Linux compile + unit tests)
**Project Type**: Library crate (astro-up-core engine module)
**Performance Goals**: Single-package update orchestration < 3 minutes excluding download
**Constraints**: Sequential installs (Windows MSI mutex), global lock file for single-instance
**Scale/Scope**: ~50 packages in catalog, typical user has 5-15 installed

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | Engine lives in `astro-up-core/src/engine/` as a new module |
| II. Platform Awareness | PASS | Process detection and lock files use `cfg(windows)` where needed; sysinfo is cross-platform |
| III. Test-First | PASS | Integration tests for pipeline, unit tests for version parsing and dependency resolution |
| IV. Thin Tauri Boundary | PASS | All logic in core; CLI/GUI call engine methods |
| V. Spec-Driven | PASS | This plan implements spec 012 |
| VI. Simplicity | PASS | No abstractions beyond what the spec requires; sequential pipeline, no state machine framework |

## Project Structure

### Documentation (this feature)

```text
specs/012-install-orchestration/
├── spec.md
├── plan.md              # This file
├── decisions.md         # Prior decisions
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── engine-service.rs
└── tasks.md             # Phase 2 output (from /speckit.tasks)
```

### Source Code (repository root)

```text
crates/astro-up-core/src/
├── engine/
│   ├── mod.rs           # Module root, re-exports
│   ├── orchestrator.rs  # UpdateOrchestrator — main pipeline coordinator
│   ├── planner.rs       # UpdatePlanner — version compare, dependency resolve, plan building
│   ├── version_cmp.rs   # VersionFormat enum, date/custom parsers, PackageStatus
│   ├── policy.rs        # Policy enforcement logic (minor/major filtering, per-package overrides)
│   ├── history.rs       # OperationRecord, operations table read/write
│   ├── lock.rs          # Global lock file (PID-based, stale detection)
│   └── process.rs       # Running-process detection (sysinfo)
└── events.rs            # Extended with orchestration-level events (existing file)
```

**Structure Decision**: New `engine/` module in astro-up-core following Constitution I (modules-first). All new version comparison code (VersionFormat, date/custom parsers, PackageStatus) lives in `engine/version_cmp.rs`; existing `types/version.rs` is unchanged. No new crates.

### Test Structure

```text
crates/astro-up-core/tests/
├── engine_orchestrator.rs   # Integration: full pipeline with mock traits
├── engine_planner.rs        # Integration: dependency resolution, plan building
├── version_formats.rs       # Unit: semver, date, custom regex comparison
└── engine_policy.rs         # Unit: policy filtering logic
```

## Complexity Tracking

No violations. All code stays in `astro-up-core` as modules. No new crates, no repository pattern, no state machine framework.
