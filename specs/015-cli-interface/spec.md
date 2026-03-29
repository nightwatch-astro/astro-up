# Feature Specification: CLI Interface

**Feature Branch**: `015-cli-interface`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 014 — astro-up-cli binary with clap + ratatui

## User Scenarios & Testing *(mandatory)*

### User Story 1 - List Installed Software (Priority: P1)

A user runs `astro-up list` and sees a styled table of installed astrophotography software with names, installed versions, latest versions, and update status (up-to-date, update available, major upgrade).

**Why this priority**: Listing is the most basic operation and the entry point for all other CLI interactions.

**Independent Test**: Run `list` with known installed software, verify output shows correct versions and status.

**Acceptance Scenarios**:

1. **Given** 5 packages are installed, **When** `astro-up list` runs, **Then** a styled table shows all 5 with name, installed version, latest version, and status indicator
2. **Given** `--json` flag is passed, **When** listing, **Then** output is valid JSON array with the same fields
3. **Given** `--category guiding` is passed, **When** listing, **Then** only guiding software is shown

---

### User Story 2 - Check for Updates (Priority: P2)

A user runs `astro-up check` to see which packages have updates available without installing anything. The output shows current vs latest version for each package with an update.

**Why this priority**: Check is the read-only precursor to update — users want to see what's available before committing.

**Independent Test**: Run `check` with known outdated packages, verify they're listed with correct version info.

**Acceptance Scenarios**:

1. **Given** 3 of 10 packages have updates, **When** `check` runs, **Then** only the 3 with updates are shown
2. **Given** all packages are up to date, **When** `check` runs, **Then** "All packages are up to date" is displayed
3. **Given** `check nina-app` targets a specific package, **When** running, **Then** only NINA's status is shown

---

### User Story 3 - Update with Progress (Priority: P3)

A user runs `astro-up update --all` and sees a ratatui TUI with download progress bars, install status, and a summary at the end.

**Why this priority**: Visual progress during updates is critical for user confidence during long operations.

**Independent Test**: Run `update` on a test package, verify progress bar shows download and install stages.

**Acceptance Scenarios**:

1. **Given** an update is available, **When** `update nina-app` runs, **Then** download progress and install status are shown in a TUI
2. **Given** `--quiet` flag, **When** updating, **Then** minimal output (just success/failure) with no TUI
3. **Given** `--dry-run` flag, **When** updating, **Then** the plan is shown without executing

---

### User Story 4 - Scan for Installed Software (Priority: P4)

A user runs `astro-up scan` for first-time setup. The application detects all installed astrophotography software on the system and reports what it found.

**Why this priority**: Initial detection is the first interaction new users have after installation.

**Independent Test**: Run `scan` on a system with known software, verify all are detected.

**Acceptance Scenarios**:

1. **Given** NINA, PHD2, and ASCOM are installed, **When** `scan` runs, **Then** all three are detected with their versions
2. **Given** `--json` flag, **When** scanning, **Then** output is valid JSON

---

### User Story 5 - Configuration Management (Priority: P5)

A user runs `astro-up config init` to generate a default config file, or `astro-up config show` to see the effective configuration.

**Why this priority**: Config management CLI commands support the configuration system (spec 004).

**Independent Test**: Run `config init`, verify a documented TOML file is generated.

**Acceptance Scenarios**:

1. **Given** no config file exists, **When** `config init` runs, **Then** a documented config.toml is created at the default path
2. **Given** a config exists, **When** `config show` runs, **Then** the effective config (after layering) is displayed

### Edge Cases

- No subcommand provided: Show help with available commands.
- Unknown subcommand: Error with "did you mean?" suggestion.
- Ctrl+C during update: Cancel gracefully, report partial progress.
- Terminal doesn't support colors: Detect and fall back to plain text.
- Piped output (not a TTY): Auto-disable colors and TUI, use plain text.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide subcommands: `list`, `check`, `update`, `scan`, `add`, `remove`, `config` (init/show), `self-update`
- **FR-002**: System MUST support global flags: `--verbose` (debug logging), `--quiet` (minimal output), `--config <path>`, `--json` (machine-readable output)
- **FR-003**: System MUST detect TTY vs pipe and auto-adjust output format (colors, TUI vs plain text)
- **FR-004**: System MUST show download/install progress via ratatui TUI in interactive mode
- **FR-005**: System MUST support `--json` flag on all read commands for machine-readable output
- **FR-006**: System MUST exit with code 0 (success), 1 (error), or 2 (user cancelled)
- **FR-007**: System MUST support `--dry-run` on `update` to show the plan without executing
- **FR-008**: System MUST support package-specific operations (e.g., `check nina-app`, `update phd2`)
- **FR-009**: System MUST support `--allow-major` flag on `update` to permit major version upgrades
- **FR-010**: System MUST handle Ctrl+C gracefully, cancelling in-progress operations cleanly
- **FR-011**: System MUST be a standalone binary with no Tauri/WebView dependency
- **FR-012**: System MUST support `self-update` to update astro-up itself

### Key Entities

- **CliApp**: Top-level clap application with subcommand dispatch
- **OutputMode**: Enum of Interactive (TUI), Plain (no colors), Json (machine-readable)
- **ProgressRenderer**: ratatui-based TUI for download/install progress

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All subcommands complete in under 2 seconds for cached data operations (list, config show)
- **SC-002**: JSON output is valid and parseable by `jq` for all read commands
- **SC-003**: TUI progress updates at least once per second during downloads
- **SC-004**: CLI binary size is under 10MB (no WebView dependency)

## Assumptions

- The CLI is a separate binary from the GUI (astro-up-cli vs astro-up-gui)
- ratatui is used for TUI elements (progress bars, tables), not a full TUI app framework
- The CLI shares all logic with the GUI via astro-up-core — no duplicated business logic
- Depends on: spec 004 (config), spec 005 (catalog), spec 006-007 (detection), spec 008 (providers), spec 010 (download), spec 011 (install), spec 012 (engine), spec 013 (backup), spec 014 (custom tools)
