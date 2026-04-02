# Research: CLI Interface

**Date**: 2026-04-02 | **Spec**: [spec.md](spec.md)

## Technology Decisions

### CLI Framework: clap 4 (derive)

- **Decision**: Keep existing clap 4 with derive macros
- **Rationale**: Already in the project, industry standard, excellent error messages, built-in help/version/suggestions
- **Alternatives**: argh (Google, minimal), bpaf (functional), pico-args (zero-dep) — all too minimal for this scope

### TUI Progress: ratatui 0.29

- **Decision**: ratatui for download/install progress rendering
- **Rationale**: Best Rust TUI library (blessed.rs). Temporary TUI during operations, not a persistent app. Gauge, Table, and Paragraph widgets cover all needs.
- **Alternatives**: indicatif (simpler progress bars — but we need tables too), console (too basic)
- **Pattern**: Create a Terminal, draw frames in a loop driven by event channel, restore terminal on exit/panic

### Table Output: tabled 0.17

- **Decision**: tabled for styled table output in show/search/backups commands
- **Rationale**: Simple API for rendering tabular data to stdout with style support. Does not require a full TUI — works with plain print. blessed.rs listed.
- **Alternatives**: comfy-table (similar, fewer downloads), ratatui Table (requires full TUI setup for simple list output — overkill)

### Confirmation Prompts: dialoguer 0.11

- **Decision**: dialoguer for interactive confirmation and selection
- **Rationale**: Standard Rust library for CLI prompts (blessed.rs). Confirm, Select, FuzzySelect. Respects `--yes` / non-TTY by skipping prompts.
- **Alternatives**: inquire (similar API, fewer downloads), requestty (unmaintained)

### Error Display: color-eyre 0.6

- **Decision**: Keep existing color-eyre, switch from anyhow
- **Rationale**: Already in stack. Styled backtraces, `.wrap_err()` context chains, `.suggestion()` for actionable hints. Integrates with tracing via tracing-error.
- **Alternatives**: miette (overkill — designed for parsers/compilers with source snippets), anyhow (no styled output)

### Logging: tracing-subscriber 0.3 + tracing-appender 0.2

- **Decision**: Dual-layer tracing setup
- **Rationale**: stderr layer (human, respects --verbose) + file layer (JSON, always DEBUG). tracing-appender provides daily rotation and non-blocking writes.
- **Pattern**:
  ```rust
  let file_appender = tracing_appender::rolling::daily(log_dir, "astro-up.log");
  let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

  tracing_subscriber::registry()
      .with(stderr_layer.with_filter(verbosity))
      .with(json_file_layer.with_writer(non_blocking))
      .init();
  ```
- **Alternatives**: env_logger (no file output), slog (different ecosystem), log4rs (heavyweight)

### Panic Handler: human-panic 2

- **Decision**: human-panic for release builds
- **Rationale**: Replaces scary backtraces with user-friendly "something went wrong" message and crash report file. Standard for CLI tools.
- **Pattern**: `#[cfg(not(debug_assertions))] human_panic::setup_panic!();` in main()

### Integration Testing: assert_cmd + predicates

- **Decision**: assert_cmd for CLI integration tests
- **Rationale**: Standard approach for testing CLI binaries (blessed.rs). Run the binary as a subprocess, assert on stdout/stderr/exit code.
- **Alternatives**: trycmd (snapshot-based, less flexible), escargot (lower level)

## Crate Compatibility

All dependencies are compatible with Rust 2024 edition (1.85+) and the existing workspace:

| Crate | Version | blessed.rs | lib.rs | Downloads |
|-------|---------|------------|--------|-----------|
| clap | 4 | Yes | Yes | 363M |
| ratatui | 0.29 | Yes | Yes | 27M |
| color-eyre | 0.6 | Yes | Yes | 46M |
| tracing-subscriber | 0.3 | Yes | Yes | 363M |
| tracing-appender | 0.2 | Yes | Yes | 63M |
| dialoguer | 0.11 | Yes | Yes | 52M |
| tabled | 0.17 | Yes | Yes | 11M |
| human-panic | 2 | Yes | Yes | 8M |
| assert_cmd | 2 | Yes | Yes | 31M |
