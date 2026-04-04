# Implementation Plan: CLI Command Wiring

**Branch**: `024-cli-wiring` | **Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/024-cli-wiring/spec.md`

## Summary

Wire all CLI command handlers to the astro-up-core engine, replacing stub implementations with real calls to Scanner, Orchestrator, DownloadManager, and BackupService. Add inline progress bars via indicatif, persist scan results to the ledger, implement self-update via GitHub Releases API, rename the binary to `astro-up`, and create comprehensive Windows CI integration tests.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: clap 4 (existing), indicatif 0.18 (new — replaces ratatui for progress), color-eyre 0.6 (existing), astro-up-core (existing)
**Storage**: SQLite via rusqlite (existing — ledger for scan results)
**Testing**: assert_cmd + predicates (existing), insta (existing), cargo test + Windows CI
**Target Platform**: Windows (primary), macOS/Linux (compile + platform guards)
**Project Type**: CLI binary
**Performance Goals**: Progress visible within 1s, Ctrl+C cancellation within 2s
**Constraints**: Thin wrappers only — all logic in astro-up-core
**Scale/Scope**: 9 commands, ~500 lines of new handler code, ~400 lines of tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | All logic stays in astro-up-core; CLI handlers are thin wrappers |
| II. Platform Awareness | PASS | Windows-gated detection/install; platform guards for non-Windows |
| III. Test-First | PASS | Integration tests with fixture catalog; Windows CI for real operations |
| IV. Thin Tauri Boundary | PASS | CLI mirrors GUI wiring pattern — same core APIs, different adapter |
| V. Spec-Driven | PASS | This spec defines all acceptance criteria |
| VI. Simplicity | PASS | indicatif over ratatui (simpler for inline progress); CliState is a plain struct |

## Project Structure

### Documentation (this feature)

```text
specs/024-cli-wiring/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── cli-state.rs     # CliState struct contract
└── tasks.md             # Phase 2 output (from /speckit.tasks)
```

### Source Code (repository root)

```text
crates/astro-up-cli/
├── Cargo.toml           # [[bin]] name = "astro-up", +indicatif, -ratatui (optional keep)
├── src/
│   ├── main.rs          # Entry point (unchanged)
│   ├── lib.rs           # Cli struct, Commands, run() — pass CliState to handlers
│   ├── state.rs         # NEW: CliState struct (catalog_manager, db_path, config, backup)
│   ├── commands/
│   │   ├── mod.rs       # confirm(), CliState init
│   │   ├── scan.rs      # Wire Scanner → table/JSON output
│   │   ├── show.rs      # Read from ledger (installed/outdated)
│   │   ├── install.rs   # Wire Orchestrator.install() + progress
│   │   ├── update.rs    # Wire Orchestrator.update() + progress
│   │   ├── backup.rs    # Wire BackupService with catalog lookup
│   │   ├── search.rs    # Already wired (no changes)
│   │   ├── restore.rs   # Already wired (no changes)
│   │   ├── config.rs    # Already wired (no changes)
│   │   └── self_update.rs # GitHub Releases API check + download
│   ├── output/
│   │   ├── mod.rs       # OutputMode (existing, already has Quiet)
│   │   ├── json.rs      # JSON printer (existing)
│   │   ├── table.rs     # Table printer (existing)
│   │   └── progress.rs  # Replace raw writes with indicatif ProgressBar
│   └── logging.rs       # Existing (no changes)
├── tests/
│   ├── cli_show.rs      # Existing tests
│   ├── cli_json.rs      # Existing tests
│   ├── cli_integration.rs # New comprehensive tests
│   └── fixtures/
│       └── test-catalog.db # Pre-built fixture catalog
└── tests/snapshots/     # insta snapshots
```

**Structure Decision**: Follows existing layout. Only new files: `state.rs`, `fixtures/test-catalog.db`. Existing files are modified in place.

## Complexity Tracking

No constitution violations. No complexity justification needed.
