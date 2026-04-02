# Implementation Plan: CLI Interface

**Branch**: `015-cli-interface` | **Date**: 2026-04-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/015-cli-interface/spec.md`

## Summary

Implement the astro-up CLI binary: clap subcommands for all user operations (show, install, update, scan, search, backup, restore, config, self-update), ratatui TUI for interactive progress, JSON output mode for scripting, structured logging with tracing dual layers (stderr + JSON file), graceful Ctrl+C via tokio cancellation.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: clap 4 (derive, existing), ratatui 0.29 (TUI), color-eyre 0.6 (errors, existing), tracing-subscriber 0.3 (logging layers), tracing-appender 0.2 (file rotation), human-panic 2 (release panic handler), dialoguer 0.11 (confirmation prompts), tabled 0.17 (table rendering)
**Storage**: SQLite (existing — catalog, config, ledger via astro-up-core)
**Testing**: cargo test, insta (snapshot testing of CLI output), assert_cmd + predicates (integration tests)
**Target Platform**: Cross-platform (Windows primary, macOS/Linux CI)
**Project Type**: CLI binary (astro-up-cli crate)
**Performance Goals**: SC-001 show/search <2s cached, SC-003 TUI progress ≥1Hz
**Constraints**: Standalone binary (no Tauri/WebView), single self-contained executable
**Scale/Scope**: 7 user stories, 19 FRs, 4 SCs, ~10 subcommands

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | CLI is a thin binary crate — all logic in astro-up-core |
| II. Platform Awareness | PASS | Cross-platform, no cfg(windows) needed in CLI (core handles platform) |
| III. Test-First | PASS | insta snapshots for output, assert_cmd for integration, dialoguer mock for prompts |
| IV. Thin Tauri Boundary | PASS | CLI and GUI are parallel consumers of core — identical results |
| V. Spec-Driven | PASS | Full speckit pipeline |
| VI. Simplicity | PASS | clap derive for parsing, thin command handlers delegating to core |

No violations.

## Project Structure

### Documentation

```text
specs/015-cli-interface/
  spec.md, decisions.md, plan.md, research.md
  data-model.md, quickstart.md
  contracts/cli-commands.rs
  checklists/requirements.md
  tasks.md
```

### Source Code

```text
crates/astro-up-cli/
  src/
    main.rs           # Entry point: panic handler, tracing init, clap parse, dispatch
    lib.rs            # Cli struct, Commands enum, run()
    commands/
      mod.rs          # Command trait + shared helpers (OutputMode, confirm, table rendering)
      show.rs         # show [all|installed|outdated|backups] [package]
      install.rs      # install <package>
      update.rs       # update <package>|--all
      scan.rs         # scan
      search.rs       # search <query>
      backup.rs       # backup <package>
      restore.rs      # restore <package>
      config.rs       # config init|show
      self_update.rs  # self-update
    output/
      mod.rs          # OutputMode enum, format dispatch
      json.rs         # JSON serialization for all command outputs
      table.rs        # Styled table rendering (show, search, backups)
      progress.rs     # ratatui TUI for download/install progress
    logging.rs        # Dual-layer tracing setup (stderr + JSON file)
  tests/
    cli_show.rs       # Integration tests for show command
    cli_update.rs     # Integration tests for update command
    cli_json.rs       # JSON output validation across all commands
```

**Structure Decision**: Flat `commands/` module with one file per subcommand. Each command function takes parsed args + core services, returns `Result<()>`. Output formatting is separated into `output/` to keep command logic clean.

## Complexity Tracking

No constitution violations — section empty.

## Architecture Overview

### Layered Design

```
main.rs → lib.rs (clap parse) → commands/* (orchestrate) → astro-up-core (logic)
                                      ↓
                                  output/* (format)
                                      ↓
                              stdout/stderr/JSON file
```

### Key Design Decisions

1. **Command handlers are thin adapters**: Each command function calls core services (engine, scanner, catalog, backup) and passes results to output formatters. No business logic in the CLI crate.

2. **OutputMode drives all formatting**: Determined once at startup from `--json` flag + TTY detection. All commands receive it and branch on Interactive/Plain/Json.

3. **Tracing dual layers**: stderr layer respects `--verbose`/`--quiet`. File layer always writes JSON to `{data_dir}/logs/astro-up-{date}.log`. Non-blocking via tracing-appender.

4. **Ctrl+C cancellation**: Create a `CancellationToken` in main, install signal handler that trips it, pass token to engine operations.

5. **First-run bootstrap**: Before any command that needs data, check for catalog. If missing, auto-download with progress. Then check for scan cache. If missing, auto-scan.

6. **Confirmation prompts**: dialoguer for interactive prompts (install/update confirmation, restore picker). Skipped in JSON mode or with `--yes`.

### Dependency Graph

```
Phase 1 (Foundation) → Phase 2 (Read Commands) → Phase 3 (Write Commands) → Phase 4 (Polish)
                                                   ↑
                                          Phase 2 provides output/table.rs
                                          Phase 3 reuses output/progress.rs
```

- Phase 1: main.rs, logging, OutputMode, Cli struct — everything compiles and runs `--help`
- Phase 2: show, scan, search — read-only commands using catalog + scanner
- Phase 3: install, update, backup, restore — write commands using engine + backup service
- Phase 4: config, self-update, error polish, integration tests
