# Feature Specification: CLI Interface

**Feature Branch**: `015-cli-interface`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 014 — astro-up-cli binary with clap + ratatui

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Show Software Status (Priority: P1)

A user runs `astro-up show` and sees a styled table of all catalog software with columns: name, category, installed version, latest version, and status. `show installed` filters to installed only. `show outdated` filters to packages with available updates.

**Why this priority**: The show command is the primary read interface — what do I have, what's available, what needs updating?

**Independent Test**: Run `show installed` with known software, verify correct versions and status.

**Acceptance Scenarios**:

1. **Given** 5 packages installed, 3 with updates, **When** `show` runs, **Then** all catalog entries shown with install status badges
2. **Given** `show installed` runs, **Then** only installed packages are shown
3. **Given** `show outdated` runs, **Then** only packages with available updates are shown
4. **Given** `show nina` runs, **Then** detailed info: name, version, category, detection method, backup count, last updated, dependencies
5. **Given** `--json` flag, **Then** all show variants output valid JSON

---

### User Story 2 - Install Software (Priority: P2)

A user runs `astro-up install nina` to install a package. If the package is already installed, the user is notified and offered to update instead.

**Why this priority**: Install is the primary write action for new packages.

**Independent Test**: Install a test package, verify it downloads and installs. Try installing an already-installed package, verify the update prompt.

**Acceptance Scenarios**:

1. **Given** NINA is not installed, **When** `install nina` runs, **Then** the orchestration pipeline runs (download → install → verify)
2. **Given** NINA 3.0 is already installed and 3.1 is available, **When** `install nina` runs, **Then** "NINA 3.0 is already installed. Update to 3.1? [y/N]"
3. **Given** NINA is already at the latest version, **When** `install nina` runs, **Then** "NINA 3.1 is already installed and up to date"
4. **Given** an unknown package ID, **When** `install foo` runs, **Then** "Package 'foo' not found in catalog. Did you mean: ..."

---

### User Story 3 - Update Software (Priority: P3)

A user runs `astro-up update nina` or `astro-up update --all` to update packages. Update only works on installed packages — not-installed packages get an error suggesting `install`.

**Why this priority**: Update is the most frequent action for existing users.

**Independent Test**: Update a test package with a known newer version. Verify download + install + verification.

**Acceptance Scenarios**:

1. **Given** NINA 3.0 installed with 3.1 available, **When** `update nina` runs, **Then** ratatui TUI shows download and install progress
2. **Given** NINA is not installed, **When** `update nina` runs, **Then** "NINA is not installed. Use `install nina` instead."
3. **Given** `update --all` with 3 outdated packages, **When** running, **Then** an update plan table is shown (package, current → target), user confirms, all 3 are updated in dependency order with progress
4. **Given** `--dry-run`, **When** updating, **Then** the plan is shown without executing
5. **Given** `--all --json`, **When** updating, **Then** structured JSON output with per-package results

---

### User Story 4 - Scan for Installed Software (Priority: P4)

A user runs `astro-up scan` for first-time setup or to refresh detection. The application runs all detection methods (registry, PE, WMI) and updates the cache.

**Why this priority**: Scan is the first action new users take — "what do I have?"

**Independent Test**: Run `scan` on a system with known software, verify all are detected.

**Acceptance Scenarios**:

1. **Given** NINA, PHD2, and ASCOM are installed, **When** `scan` runs, **Then** all three are detected with versions
2. **Given** `--json` flag, **Then** output is valid JSON with detection results per package
3. **Given** `scan` completes, **When** `show installed` runs immediately after, **Then** cached results are used (no re-scan)

---

### User Story 5 - Search Catalog (Priority: P5)

A user runs `astro-up search plate` and sees matching packages from the catalog via FTS5 full-text search.

**Why this priority**: Discovery — finding software by name, tag, or description.

**Independent Test**: Search for "guiding", verify PHD2 and MetaGuide appear.

**Acceptance Scenarios**:

1. **Given** `search plate`, **Then** PlateSolve, ASTAP, and All-Sky Plate Solver are shown (ranked by relevance)
2. **Given** `search nonexistent`, **Then** "No packages found for 'nonexistent'"
3. **Given** `search --json guiding`, **Then** valid JSON array of matching packages

---

### User Story 6 - Backup and Restore (Priority: P6)

A user manages backups via `astro-up backup nina` (create), `astro-up restore nina` (restore with picker), and `astro-up show backups nina` (list).

**Why this priority**: Backup/restore is the safety net for config changes.

**Independent Test**: Create a backup, list it, restore it.

**Acceptance Scenarios**:

1. **Given** `backup nina`, **Then** a backup archive is created and confirmed
2. **Given** `restore nina`, **Then** available backups are listed, user picks one, file change summary shown, confirm before overwriting
3. **Given** `show backups nina`, **Then** backups listed with date, version, size
4. **Given** `show backups` (no package), **Then** all backups across all packages shown

---

### User Story 7 - Configuration (Priority: P7)

A user runs `astro-up config init` to generate a default config file, or `astro-up config show` to see effective configuration.

**Why this priority**: Config management supports setup and debugging.

**Acceptance Scenarios**:

1. **Given** no config file exists, **When** `config init` runs, **Then** a documented config.toml is created
2. **Given** `config show`, **Then** the effective config (after layering) is displayed
3. **Given** `config show --json`, **Then** JSON output of effective config

### Edge Cases

- No subcommand provided: Show help with available commands
- Unknown subcommand: Error with "did you mean?" suggestion (clap's built-in)
- Ctrl+C during update: Cancel gracefully, report partial progress
- Terminal doesn't support colors: Detect and fall back to plain text
- Piped output (not a TTY): Auto-disable colors and TUI, use plain text
- `show` on first run with no cache: Auto-trigger `scan` first, then show results
- First run with no catalog: Auto-download catalog with progress feedback, then proceed

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide commands: `show`, `install`, `update`, `scan`, `search`, `backup`, `restore`, `config`, `self-update`
- **FR-002**: System MUST support `show` subvariants: `show` (all), `show installed`, `show outdated`, `show <package>`, `show backups [package]`
- **FR-003**: System MUST support global flags: `--verbose`, `--quiet`, `--config <path>`, `--json`
- **FR-004**: System MUST detect TTY vs pipe and auto-adjust output format
- **FR-005**: System MUST show download/install progress visually in interactive mode (progress bars, status updates)
- **FR-006**: System MUST support `--json` on ALL commands (both read and write) for CI/scripting
- **FR-007**: System MUST exit with code 0 (success), 1 (error), 2 (user cancelled)
- **FR-008**: System MUST support `--dry-run` on `install` and `update`
- **FR-009**: System MUST support `--allow-major` on `update` to permit major version upgrades
- **FR-010**: System MUST handle Ctrl+C gracefully, cancelling in-progress operations
- **FR-011**: System MUST be a standalone binary with no Tauri/WebView dependency
- **FR-012**: System MUST support `self-update` to update astro-up itself
- **FR-013**: `install` MUST defer to `update` if the package is already installed (with user confirmation)
- **FR-014**: `update` MUST error if the package is not installed (suggest `install`)
- **FR-015**: `show` on first run with no cache MUST auto-trigger `scan`
- **FR-016**: System MUST support `update --all --yes` for non-interactive bulk updates
- **FR-017**: On first run with no catalog database, System MUST auto-download the catalog with visible progress feedback ("Downloading catalog..." with progress bar), then proceed with the requested command
- **FR-018**: System MUST always write a structured log file to the data directory. `--verbose`/`--quiet` control terminal output verbosity only. On errors, the log file path MUST be shown to help users report issues.
- **FR-019**: `update` and `install` without `--yes` MUST show an update plan table (package name, current version, target version) and require explicit confirmation before proceeding. `--yes` skips the confirmation prompt.

### Key Entities

- **CliApp**: Top-level clap application with subcommand dispatch
- **OutputMode**: Enum of Interactive (TUI), Plain (no colors), Json (machine-readable)
- **ProgressRenderer**: Visual TUI for download/install progress (progress bars, spinners, status)

## Command Summary

```
astro-up show [all|installed|outdated|backups] [package] [--json]
astro-up install <package> [--dry-run] [--json]
astro-up update <package>|--all [--dry-run] [--allow-major] [--yes] [--json]
astro-up scan [--json]
astro-up search <query> [--json]
astro-up backup <package>
astro-up restore <package> [--path <filter>] [--yes]
astro-up config init|show [--json]
astro-up self-update [--dry-run]
```

Global flags: `--verbose`, `--quiet`, `--config <path>`, `--json`

## Clarifications

### Session 2026-04-02

- Q: How should the CLI handle first-run when no catalog database exists? → A: Auto-download catalog on first run with visible progress feedback ("Downloading catalog..."), then proceed with the requested command.
- Q: Should the CLI write a log file for troubleshooting? → A: Always write a structured log file to data dir; `--verbose` controls terminal verbosity only. Log file path shown on errors.
- Q: What should the confirmation flow look like for `update --all`? → A: Show update plan as a table (package, current version → target version), then ask for confirmation. `--yes` skips the prompt.
- Q: How should errors be presented to users? → A: Styled error with context and actionable suggestion, plus log file path. Use tracing-subscriber with dual layers (human stderr + structured JSON file).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All show/search commands complete in under 2 seconds for cached data
- **SC-002**: JSON output is valid and parseable by `jq` for all commands
- **SC-003**: TUI progress updates at least once per second during downloads
- **SC-004**: CLI is a single self-contained binary with no external runtime dependencies

## Assumptions

- Separate binary from the GUI (astro-up-cli vs astro-up-gui)
- ratatui for progress bars and tables during long operations, not a persistent TUI app
- All business logic shared with GUI via astro-up-core
- Depends on: spec 004 (config), spec 005 (catalog), spec 006 (detection), spec 010 (download), spec 011 (install), spec 012 (engine), spec 013 (backup)
