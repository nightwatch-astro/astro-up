# Research: CLI Command Wiring

**Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)

## Technology Decisions

### Progress Display: indicatif (replace ratatui for this use case)

- **Decision**: Use `indicatif` for inline progress bars instead of ratatui
- **Rationale**: ratatui is a full-screen TUI framework — it takes over the entire terminal with an "alternate screen". For simple progress bars during download/install, this is overkill and disrupts the normal stdout flow. `indicatif` provides inline progress bars that work alongside regular println! output, exactly like cargo, rustup, and pip.
- **Alternatives**: ratatui (full-screen TUI — wrong abstraction), raw `\r` writes (current approach — no multi-bar, no ETA formatting)
- **Action**: Replace ratatui with indicatif in Cargo.toml. Keep the existing `progress.rs` text renderer as the Plain mode fallback.
- **Note**: ratatui stays as a dependency for potential future interactive TUI (e.g., package browser) but is not used in this spec.

### Binary Name: Cargo.toml `[[bin]]` section

- **Decision**: Add `[[bin]] name = "astro-up" path = "src/main.rs"` to Cargo.toml
- **Rationale**: `cargo install` uses the `[[bin]]` name, not the package name. The `#[command(name = "astro-up")]` in clap only affects `--version` and help output, not the actual binary name.
- **Alternative**: Rename the package to `astro-up` — rejected because it conflicts with `astro-up-core` and `astro-up-gui` naming convention.

### CLI State: Shared struct mirroring GUI's AppState

- **Decision**: Create a `CliState` struct in `commands/mod.rs` that holds catalog_manager, db_path, config, backup_service — same services the GUI's AppState uses
- **Rationale**: The GUI wires core services via `AppState` managed by Tauri. The CLI needs the same services but without Tauri's state management. A simple struct initialized once in `run()` and passed to command handlers avoids the current pattern of each handler creating its own `ensure_catalog()` / `ProjectDirs` / etc.
- **Pattern**: Initialize in `run()`, pass `&CliState` to each handler

### Self-Update: reqwest + GitHub Releases API

- **Decision**: Direct GitHub Releases API call via reqwest (already a dependency via astro-up-core)
- **Rationale**: The GUI uses tauri-plugin-updater which is Tauri-specific. The CLI needs a standalone implementation. GitHub's Releases API is simple: GET `/repos/{owner}/{repo}/releases/latest`, compare `tag_name` against current version, download the asset if newer.
- **Alternative**: self_update crate — rejected (unmaintained, last release 2023)

### Fixture Catalog: Pre-built SQLite for CI

- **Decision**: Ship a minimal `test-catalog.db` in `crates/astro-up-cli/tests/fixtures/`
- **Rationale**: Integration tests need a catalog to exercise show/search/install --dry-run without network access. The fixture contains 3-5 packages with detection configs, version entries, and backup configs.
- **Alternative**: Mock the CatalogManager — rejected per constitution (prefer integration tests over mocks)

## Crate Compatibility

| Crate | Version | Status | Notes |
|-------|---------|--------|-------|
| indicatif | 0.17 | blessed.rs | 180M downloads, standard progress bar lib |
| clap | 4 | existing | `[[bin]]` rename only |
| dialoguer | 0.11 | existing | No changes needed |
| color-eyre | 0.6 | existing | No changes needed |
| tabled | 0.17 | existing | No changes needed |
| assert_cmd | 2 | existing | Windows integration tests |
| reqwest | 0.13 | existing (via core) | For self-update GitHub API calls |
