# Implementation Plan: Configuration System

**Branch**: `004-configuration-system` | **Date**: 2026-03-30 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/004-configuration-system/spec.md`

## Summary

SQLite-backed configuration system for astro-up with `config get/set/list/reset` API. Three-layer precedence: compiled defaults → SQLite stored settings → CLI flags. AppConfig struct is the parameter registry. Both CLI and GUI use the same core API. Uses garde for validation and humantime for duration parsing.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: garde 0.22 (validation), humantime 2 (duration parsing), rusqlite (existing — bundled SQLite)
**Storage**: SQLite — same database file as catalog and ledger (single .db per app)
**Testing**: cargo test, insta (snapshot), rstest (parameterized), pretty_assertions, tempfile
**Target Platform**: Windows (primary), macOS/Linux (dev/CI)
**Project Type**: Library (astro-up-core) consumed by CLI and GUI binaries
**Performance Goals**: Config load + validate < 100ms (aspirational)
**Constraints**: No secrets in config, immutable after load, extensible for downstream specs
**Scale/Scope**: ~15 config fields across 6 sections, 3-layer precedence

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | New `config/` module in `astro-up-core`, no new crate |
| II. Platform Awareness | PASS | Config module is platform-agnostic — receives DB path from caller |
| III. Test-First | PASS | Integration tests with tempfile SQLite, insta snapshots |
| IV. Thin Tauri Boundary | PASS | Config lives in core, CLI and GUI both call same API |
| V. Spec-Driven | PASS | This plan implements spec 004 |
| VI. Simplicity | PASS | rusqlite already in workspace, no new crates except garde. ~300 LOC total. |

## Project Structure

### Documentation (this feature)

```text
specs/004-configuration-system/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── checklists/          # Archived (TOML-era)
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/astro-up-core/
├── Cargo.toml                    # +garde, +humantime (rusqlite already present)
├── src/
│   ├── lib.rs                    # +pub mod config
│   ├── config/
│   │   ├── mod.rs                # load_config(), re-exports
│   │   ├── model.rs              # AppConfig, section structs, LogLevel enum
│   │   ├── defaults.rs           # Default impls for all config structs
│   │   ├── store.rs              # ConfigStore: SQLite read/write
│   │   └── api.rs                # config_get/set/list/reset public API
│   └── ...existing modules...
└── tests/
    └── config/
        ├── mod.rs
        ├── defaults_test.rs      # SC-001: zero-config startup
        ├── store_test.rs         # FR-013/015: SQLite persistence
        ├── api_test.rs           # FR-014: get/set/list/reset
        ├── layering_test.rs      # SC-006: precedence test
        └── validation_test.rs    # SC-003/FR-005: invalid values
```

**Structure Decision**: Five source files in `config/`. Simpler than the TOML-era design (dropped tokens.rs, unknown_keys.rs, init.rs). Each file has a clear responsibility.

## Design Decisions

### D1: SQLite key-value table for persistence

Config settings stored as string key-value pairs in `config_settings` table. The same SQLite database file is used by catalog and ledger — one file to manage. Type conversion happens at the application layer, not in SQL.

### D2: Config loading pipeline

```
load_config(db_path: &Path, cli_overrides: &[(&str, &str)]) -> Result<AppConfig>
  1. AppConfig::default() with resolved platform paths
  2. Open SQLite, read config_settings rows
  3. Merge stored values over defaults (string → typed)
  4. Merge CLI flag overrides (highest precedence)
  5. Validate with garde
  6. Return immutable AppConfig
```

### D3: Duration handling

Duration fields stored as humantime strings in SQLite (`"24h"`, `"30s"`). Parsed via `humantime::parse_duration` on load and on `config set`. No humantime-serde needed — conversion is explicit at the API boundary.

### D4: Platform path resolution

The config module does NOT resolve platform directories. App startup code (CLI `main.rs` or Tauri `lib.rs`) resolves `ProjectDirs::from("", "", "astro-up")` and passes:
- `db_path` to `load_config`
- Resolved `cache_dir`, `data_dir` as default values for `PathsConfig`

This keeps the config module testable without platform dependencies.

### D5: Config set validates before persisting

`config_set` builds a temporary `AppConfig` with the proposed change, runs garde validation, and only persists if validation passes. This prevents invalid state in SQLite.

## Complexity Tracking

No constitution violations. No complexity justifications needed.
