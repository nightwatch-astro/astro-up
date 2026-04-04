# Feature Specification: CLI Command Wiring

**Feature Branch**: `024-cli-wiring`
**Created**: 2026-04-04
**Status**: Draft
**Input**: User description: "Wire all CLI commands to astro-up-core engine with progress display and integration tests"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan for Installed Software (Priority: P1)

A user runs `astro-up scan` on their Windows imaging PC to see which astrophotography software is installed. The CLI calls the core detection engine, scans the system, and displays results as a table showing package name, version, and detection method.

**Why this priority**: Scanning is the foundation — every other command (update, backup, show installed) depends on scan results.

**Independent Test**: Run `astro-up scan` on a Windows machine with N.I.N.A. and PHD2 installed. Verify both appear in output with correct versions.

**Acceptance Scenarios**:

1. **Given** a Windows machine with astrophotography software installed, **When** the user runs `astro-up scan`, **Then** a table of detected packages is shown with name, version, and detection method
2. **Given** a Windows machine with no astrophotography software, **When** the user runs `astro-up scan`, **Then** the message "No packages detected" is shown
3. **Given** a non-Windows machine, **When** the user runs `astro-up scan`, **Then** a clear message explains detection requires Windows
4. **Given** any machine, **When** the user runs `astro-up --json scan`, **Then** valid JSON with `results` and `errors` arrays is output

---

### User Story 2 - Install a Package with Progress (Priority: P1)

A user runs `astro-up install nina-app` to install software. The CLI resolves the package from the catalog, downloads the installer with a visible progress bar, runs the installer, and verifies detection afterward.

**Why this priority**: Installing is the core value proposition — users choose Astro-Up to avoid manual download-and-install cycles.

**Independent Test**: Run `astro-up install astap` on a clean Windows machine. Verify ASTAP is downloaded, installed silently, and detected afterward.

**Acceptance Scenarios**:

1. **Given** a valid package ID, **When** the user runs `astro-up install <id>`, **Then** the download progress, install status, and verification result are displayed
2. **Given** a valid package ID, **When** the user runs `astro-up install <id> --dry-run`, **Then** the install plan is shown without downloading or installing
3. **Given** an invalid package ID, **When** the user runs `astro-up install unknown-pkg`, **Then** an error with fuzzy search suggestions is shown
4. **Given** an install in progress, **When** the user presses Ctrl+C, **Then** the download is cancelled cleanly and partial files are preserved for resume
5. **Given** a terminal, **When** the install is running, **Then** a progress bar shows download percentage, speed, and estimated time remaining
6. **Given** a piped or non-TTY context, **When** the install is running, **Then** line-by-line progress updates are printed instead of a progress bar

---

### User Story 3 - Update Installed Packages (Priority: P1)

A user runs `astro-up update --all` to update all outdated packages. The CLI scans for installed versions, compares against the catalog, plans updates, and executes them with progress feedback.

**Why this priority**: Keeping software up to date is the primary ongoing use case after initial setup.

**Independent Test**: Install an older version of a package, run `astro-up update --all`, verify it detects and updates the outdated version.

**Acceptance Scenarios**:

1. **Given** installed packages with updates available, **When** the user runs `astro-up update --all`, **Then** an update plan is shown and executed after confirmation
2. **Given** a specific outdated package, **When** the user runs `astro-up update nina-app`, **Then** only that package is updated
3. **Given** all packages are up to date, **When** the user runs `astro-up update --all`, **Then** the message "All packages are up to date" is shown
4. **Given** an update in progress, **When** the user runs with `--dry-run`, **Then** the plan is shown without executing

---

### User Story 4 - Show Installed and Outdated Packages (Priority: P2)

A user runs `astro-up show installed` or `astro-up show outdated` to see their current software status without re-running a full scan.

**Why this priority**: Important for user awareness but depends on scan being wired first.

**Independent Test**: Run `astro-up scan` then `astro-up show installed`. Verify the output matches scan results.

**Acceptance Scenarios**:

1. **Given** a previous scan has been performed, **When** the user runs `astro-up show installed`, **Then** a table of installed packages with versions is shown
2. **Given** a previous scan has been performed, **When** the user runs `astro-up show outdated`, **Then** only packages with available updates are shown
3. **Given** no scan has ever been performed (empty ledger), **When** the user runs `astro-up show installed`, **Then** a message suggests running `astro-up scan` first

---

### User Story 5 - Create a Backup (Priority: P2)

A user runs `astro-up backup nina-app` to back up N.I.N.A.'s configuration before a risky upgrade.

**Why this priority**: Backups are a safety net for updates but require catalog backup config to be populated.

**Independent Test**: Install N.I.N.A., create some profiles, run `astro-up backup nina-app`, verify a backup archive is created.

**Acceptance Scenarios**:

1. **Given** a package with backup config in the catalog, **When** the user runs `astro-up backup <id>`, **Then** the configuration is backed up and the archive path is shown
2. **Given** a package without backup config, **When** the user runs `astro-up backup <id>`, **Then** a message explains no backup paths are configured for this package

---

### User Story 6 - Self-Update (Priority: P3)

A user runs `astro-up self-update` to check for and install a newer version of Astro-Up itself.

**Why this priority**: Important for long-term maintenance but the GUI already handles this via the updater plugin.

**Independent Test**: Run `astro-up self-update --dry-run`, verify it checks GitHub Releases and reports current vs latest version.

**Acceptance Scenarios**:

1. **Given** a newer version exists on GitHub Releases, **When** the user runs `astro-up self-update`, **Then** the update is downloaded and applied
2. **Given** the user is on the latest version, **When** the user runs `astro-up self-update`, **Then** "You are running the latest version" is shown
3. **Given** any version, **When** the user runs `astro-up self-update --dry-run`, **Then** the version comparison is shown without applying any update

---

### User Story 7 - Binary Name (Priority: P3)

A user installs the CLI and can invoke it as `astro-up` rather than `astro-up-cli`.

**Why this priority**: Polish item that improves user experience but doesn't affect functionality.

**Independent Test**: After installation, verify the binary is available as `astro-up` in PATH.

**Acceptance Scenarios**:

1. **Given** the CLI is installed, **When** the user types `astro-up`, **Then** the CLI launches

---

### Edge Cases

- What happens when the catalog is unavailable during install/update? (Graceful error with retry suggestion)
- What happens when a download fails? (Auto-retry up to 3 attempts before reporting failure)
- What happens when an install fails? (Report immediately — no auto-retry, as install failures may need manual intervention like elevation or closing a running process)
- What happens when an installer requires elevation but the user doesn't have admin? (Clear error message)
- What happens when a download is interrupted and resumed? (Partial file preserved, resume on retry)
- What happens when `--quiet` is combined with `--json`? (JSON takes precedence)
- What happens when scan finds a package not in the catalog? (Shown as "Unknown" with detected version)
- What happens when a package has no versions in the catalog? (Error with clear message)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `scan` command MUST call the core Scanner, display results as a table, and support `--json` output
- **FR-002**: `install` command MUST resolve packages from the catalog, download with progress (auto-retry up to 3 attempts on failure), execute the installer (report failures immediately, no auto-retry), and verify detection
- **FR-003**: `update` command MUST scan for installed versions, compare against catalog, show a plan, and execute after confirmation
- **FR-004**: `scan` MUST persist results to the ledger database; `show installed` and `show outdated` MUST read from the ledger without requiring a catalog download or fresh scan
- **FR-005**: `backup` command MUST look up backup config from the catalog and call the backup service
- **FR-006**: `self-update` command MUST check GitHub Releases for newer versions and apply updates
- **FR-007**: Progress display MUST show progress bars in interactive terminals, line-by-line updates in piped/plain mode, and nothing in quiet mode
- **FR-008**: All commands MUST subscribe to the core event broadcast channel for progress updates
- **FR-009**: All commands MUST respect output mode: Interactive (progress bars), Plain (line-by-line), Quiet (suppress), JSON (structured output)
- **FR-010**: Ctrl+C MUST propagate through cancellation to stop in-progress operations cleanly
- **FR-011**: The installed binary name MUST be `astro-up`
- **FR-012**: All confirmation prompts MUST be skippable with `--yes` and auto-confirmed in non-interactive mode
- **FR-013**: Windows CI MUST run integration tests exercising scan, install, and update with a fixture catalog
- **FR-014**: Integration tests MUST cover `--quiet`, `--json`, and `--verbose` modes for each wired command
- **FR-015**: Integration tests MUST verify cancellation propagates correctly

### Key Entities

- **OutputMode**: Controls rendering — Interactive (progress bars), Plain (line-by-line), Quiet (silent), JSON (structured)
- **Event**: Typed notifications from core (DownloadProgress, ScanProgress, InstallStarted, etc.) that drive the progress display
- **CancellationToken**: Shared token between the signal handler and core operations for clean shutdown
- **Fixture Catalog**: Pre-built test catalog database used in CI for offline integration testing

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 9 CLI commands produce meaningful output when run on a Windows machine with the catalog available
- **SC-002**: Download progress is visible within 1 second of starting a download in interactive mode
- **SC-003**: Ctrl+C cancels any in-progress operation within 2 seconds
- **SC-004**: `--quiet` mode produces zero stdout output for all commands (errors go to stderr)
- **SC-005**: `--json` mode produces valid, parseable JSON for all commands
- **SC-006**: Windows CI integration tests pass for scan, install, and update using a fixture catalog
- **SC-007**: The binary installs as `astro-up` and is invocable by that name

## Clarifications

### Session 2026-04-04

- Q: Should the CLI auto-retry failed downloads and installs? → A: Auto-retry downloads only (up to 3 attempts); report install failures immediately since they may need manual intervention (elevation, closing running processes).
- Q: Should scan results persist across invocations? → A: Yes, persist to the ledger database. `show installed`/`show outdated` read from the ledger without requiring a fresh scan.

## Assumptions

- The core library already has working implementations for Scanner, Orchestrator, DownloadManager, BackupService, and the Event system — this spec only wires them into CLI handlers
- Detection, install, and update operations are Windows-only by design — non-Windows platforms show a clear platform guard message
- The fixture catalog for integration tests will be a pre-built database checked into the test fixtures directory
- The GUI's Tauri commands serve as the reference implementation for how to call core APIs
- Self-update uses GitHub Releases API directly (not the Tauri updater plugin)
- CLI handlers remain thin wrappers — all logic stays in astro-up-core
