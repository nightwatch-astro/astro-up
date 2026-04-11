# Feature Specification: Stability and Hardening Audit

**Feature Branch**: `030-stability-hardening-audit`
**Created**: 2026-04-10
**Status**: Draft
**Type**: implementation

## Clarifications

### Session 2026-04-10

- Q: What happens when critical task restart budget is exhausted (3 panics in 10 min)? → A: Alert user via UI notification, continue in degraded mode. User decides whether to restart the app.
- CHK002: FR-001 expanded to also reject absolute paths (not just `..` traversal)
- CHK006: FR-009 expanded to handle missing Content-Length via streaming byte counter
- CHK008: FR-004 allowlist defined as config-derived (backup dir, per-package config paths, app cache dir)
- CHK009: FR-005 aggregate backup size limit set to 1 GB
- CHK015: FR-019 updated to consolidate first, then decompose by domain — resolves tension with FR-022
- CHK026: FR-003 expanded to reject Windows reparse points and junctions alongside symlinks
- CHK027: FR-002 expanded — restore overwrites existing files (full replacement, not merge)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Safe Backup Restore (Priority: P1)

A user restores a configuration backup from an archive. The application validates every file path extracted from the archive, rejecting any path containing `..` components, absolute paths, symlinks, or Windows reparse points/junctions. Files are only written within the application's designated configuration directories, preventing malicious or malformed archives from writing to arbitrary locations on the filesystem. Existing files at restore targets are overwritten (full replacement).

**Why this priority**: Path traversal during backup restore is the single highest-severity finding (CRITICAL). A malicious or corrupted archive could overwrite arbitrary files on the user's system.

**Independent Test**: Create a test archive containing entries with `..` path components and symlinks. Attempt restore and verify all malicious paths are rejected while legitimate paths succeed.

**Acceptance Scenarios**:

1. **Given** a backup archive containing a file entry `../../Windows/System32/evil.dll`, **When** the user restores the backup, **Then** the restore rejects that entry with a logged warning and completes successfully for all valid entries
2. **Given** a backup archive with valid entries within expected config directories, **When** the user restores the backup, **Then** all files are restored to their correct locations without error
3. **Given** a backup archive containing symlink entries, **When** the user restores the backup, **Then** symlink entries are rejected and logged

---

### User Story 2 - Application Survives Command Failures (Priority: P1)

A user performs multiple operations (install, update, backup, restore) in sequence. If one operation encounters an unexpected internal error (panic), the application remains responsive for subsequent operations. No single failing command renders the application unusable until restart.

**Why this priority**: Mutex poisoning from panics in Tauri command handlers cascades to crash ALL subsequent commands. This is the highest-impact stability issue affecting every user interaction.

**Independent Test**: Trigger a panic in a mock command handler, then verify that subsequent commands still execute successfully without the application hanging or crashing.

**Acceptance Scenarios**:

1. **Given** a Tauri command panics while processing, **When** the user invokes another command, **Then** the second command executes normally without hanging
2. **Given** a background task (event forwarding, progress streaming) panics, **When** the user continues using the application, **Then** the failed task is logged and restarted automatically
3. **Given** multiple concurrent operations, **When** one operation fails with an internal error, **Then** other in-progress operations continue unaffected

---

### User Story 3 - Validated Command Inputs (Priority: P1)

A user interacts with backup, restore, and directory management features through the GUI. All file paths received from the frontend are validated against an allowlist of application-controlled directories before any filesystem operations occur. The application never deletes, creates, or modifies files outside its designated directories based on user-supplied paths.

**Why this priority**: Multiple Tauri commands accept arbitrary paths from the frontend without validation. Combined with the path traversal finding, this represents a systemic input validation gap.

**Independent Test**: Invoke each path-accepting Tauri command with paths outside the allowlist (e.g., `/etc/passwd`, `C:\Windows\System32`) and verify all are rejected with appropriate error messages.

**Acceptance Scenarios**:

1. **Given** a `clear_directory` command with a path outside app-controlled directories, **When** the command executes, **Then** it returns an error and no files are deleted
2. **Given** a `create_backup` command with paths containing symlinks or mount points, **When** the command executes, **Then** it rejects invalid paths and logs a warning
3. **Given** a `delete_backup` command with a path not in the backup directory, **When** the command executes, **Then** it returns an error and no file is deleted

---

### User Story 4 - Reliable Error Reporting (Priority: P2)

A user encounters an error during software installation or catalog sync. Instead of a silent failure or application crash, they see a clear error message. The application logs sufficient context (file paths, package names, operation IDs) for troubleshooting without exposing internal implementation details to the user.

**Why this priority**: Multiple `unwrap()` calls in production code can crash the app on malformed data, and 15 unguarded `invoke()` calls in the frontend fail silently. Users currently get no feedback when operations fail in certain code paths.

**Independent Test**: Trigger error conditions in each hardened code path (malformed UUID, missing release, network failure) and verify appropriate error messages appear in the UI and structured details appear in logs.

**Acceptance Scenarios**:

1. **Given** a malformed package ID in the detection scanner, **When** the scanner processes it, **Then** the error is logged with the malformed ID and the scanner continues processing other packages
2. **Given** a frontend survey dialog invoke fails, **When** the error occurs, **Then** the user sees an error toast and the failure is logged
3. **Given** an error path that previously used `let _ =` to discard the result, **When** the error occurs, **Then** a warning or debug log entry is emitted before the result is discarded

---

### User Story 5 - Simplified Codebase for Maintainability (Priority: P3)

A developer working on the codebase finds that large monolithic files have been decomposed into focused modules, duplicated command handlers have been consolidated, and over-abstracted traits have been simplified. New contributors can navigate the code structure without mental overhead from 1000+ line files.

**Why this priority**: Code structure issues don't affect end users directly but slow development velocity and increase bug risk. Lower priority than crash/security fixes.

**Independent Test**: Verify that no source file exceeds 500 lines, duplicate command handlers are consolidated into a single parameterized function, and the orchestrator trait is simplified or inlined.

**Acceptance Scenarios**:

1. **Given** the `commands.rs` file, **When** a developer inspects it, **Then** the three duplicate command handlers (`install_software`, `update_software`, `update_all`) are consolidated into a single generic operation handler
2. **Given** the `orchestrator.rs` file, **When** a developer inspects it, **Then** `run_orchestrated_operation()` is decomposed into focused functions (plan-building, execution-dispatch, event-forwarding) each under 50 lines
3. **Given** the orchestrator trait, **When** a developer inspects it, **Then** either the trait has been inlined into its single implementation or the unused type parameters have been removed

---

### User Story 6 - Complete Observability (Priority: P3)

An operator diagnosing a production issue can trace any user operation through the logs. Every async I/O function emits structured tracing spans. Error paths always log context before returning. Log levels are consistent: info for user-visible actions, debug for internals, warn for recoverable issues, error for action-required failures.

**Why this priority**: Logging completeness is essential for post-incident diagnosis but doesn't prevent incidents directly.

**Independent Test**: Trigger a representative sample of operations (catalog sync, software install, backup create) and verify that tracing spans appear for all I/O operations with structured fields (path, package, duration_ms).

**Acceptance Scenarios**:

1. **Given** a public async function performing I/O, **When** it executes, **Then** a tracing span is emitted with appropriate structured fields
2. **Given** an error path that previously swallowed errors silently, **When** the error occurs, **Then** a log entry at warn or debug level is emitted before the error is discarded
3. **Given** the update check in the GUI, **When** it runs, **Then** it logs at info level (not debug)

---

### User Story 7 - Frontend Lifecycle Safety (Priority: P2)

A user navigates rapidly between views while operations are in progress. Pending Tauri invoke calls from unmounted components are cancelled or their results are safely ignored. No stale data from earlier requests overwrites fresh data in the UI.

**Why this priority**: Race conditions from rapid navigation can cause incorrect data display or state corruption. Lower than crash/security but affects data correctness.

**Independent Test**: Mount a component, trigger an invoke call, unmount the component before the call returns, and verify no state update occurs on the unmounted component and no error is thrown.

**Acceptance Scenarios**:

1. **Given** a component with a pending invoke call, **When** the component unmounts, **Then** the result is discarded without error
2. **Given** two rapid catalog sync requests, **When** the first response arrives after the second, **Then** the stale response is discarded
3. **Given** a watch() hook tracking config changes, **When** multiple rapid changes occur, **Then** only the latest state is applied

---

### User Story 8 - Optimized Dependencies (Priority: P3)

A developer building the project benefits from faster compile times and smaller binaries due to trimmed dependency feature flags and up-to-date packages. No unnecessary features are compiled.

**Why this priority**: Dependency optimization is lowest risk and lowest impact. No user-visible changes.

**Independent Test**: Build the project and verify that tokio features are explicitly listed (not "full"), and all npm packages are at their latest compatible versions.

**Acceptance Scenarios**:

1. **Given** the CLI crate's Cargo.toml, **When** a developer inspects tokio features, **Then** only required features are listed (not "full")
2. **Given** the frontend package.json, **When** `pnpm outdated` is run, **Then** no minor/patch updates are pending

### Edge Cases

- What happens when a ZIP archive contains entries with mixed valid and invalid paths? (Partial restore with per-entry error reporting)
- What happens when a Mutex-holding task is cancelled via CancellationToken mid-operation? (Lock is released by Drop, no poisoning with `parking_lot`)
- What happens when a spawned task panics during application shutdown? (Panic is caught, logged, and shutdown continues)
- What happens when component unmount races with invoke response arrival? (Mounted-flag pattern ensures result is discarded)
- What happens when the SQL LIMIT value is negative or extremely large? (Parameterized query lets SQLite handle bounds)
- What happens when a download server reports Content-Length larger than the max allowed? (Download is rejected before starting)
- What happens when a download server omits Content-Length entirely? (Stream with running byte counter, abort at 2 GB limit)
- What happens when a restore target file already exists? (Overwrite — restore is full replacement, not merge)
- What happens when a ZIP entry contains a Windows reparse point or junction? (Rejected alongside symlinks)
- What happens when a critical task exhausts its restart budget (3 panics in 10 min)? (User is alerted via UI notification, app continues in degraded mode without the failed task)

## Requirements *(mandatory)*

### Functional Requirements

**Phase A: Path Traversal & Input Validation (CRITICAL/HIGH)**

- **FR-001**: System MUST normalize all file paths extracted from ZIP archives, rejecting entries containing `..` path components or absolute paths (entries whose resolved path falls outside the restore target directory)
- **FR-002**: System MUST validate that restored files are written only within their original configuration directories. When a file already exists at the restore target, the system MUST overwrite it (restore is a full replacement, not a merge)
- **FR-003**: System MUST reject ZIP entries that are symlinks during backup restore. On Windows, where symlinks require elevated privileges, the system MUST also reject reparse points and junction entries
- **FR-004**: System MUST validate `clear_directory` paths against an allowlist derived from the application's runtime configuration: backup directory, per-package config paths from the catalog, and the app cache directory. Paths outside these directories MUST be rejected
- **FR-005**: System MUST validate `create_backup` source paths exist, are not symlinks or mount points, and enforce an aggregate size limit of 1 GB per backup operation
- **FR-006**: System MUST validate `delete_backup` paths are within the designated backup directory
- **FR-007**: System MUST use path component matching (not string prefix) for backup restore path filters
- **FR-008**: System MUST parameterize all SQL query values including LIMIT clauses
- **FR-009**: ~~Removed~~ — No artificial download size limit. Star database downloads (ASTAP W08) can exceed 20 GB. The existing disk space check (require 2x available) is sufficient protection.
- **FR-010**: System MUST validate configuration file size at load time before deserialization

**Phase B: Mutex Poisoning & Concurrency (HIGH)**

- **FR-011**: System MUST use non-poisoning locks for all shared state in Tauri command handlers
- **FR-012**: System MUST wrap all spawned tasks and threads in panic boundaries that log the panic and prevent silent task death
- **FR-013**: System MUST restart critical spawned tasks (event forwarding, progress streaming) on panic, with a sliding window limit (3 restarts per 10 minutes). When the restart budget is exhausted, the system MUST alert the user via a UI notification and continue in degraded mode without the failed task
- **FR-014**: System MUST use async file operations in all async Tauri command handlers (no blocking `std::fs` in async context)

**Phase C: Error Handling (HIGH/MEDIUM)**

- **FR-015**: System MUST NOT use `unwrap()` in production code paths outside of compile-time constants and test code
- **FR-016**: System MUST propagate errors with `?` or handle them with contextual logging and graceful degradation
- **FR-017**: All frontend `invoke()` calls MUST be wrapped in error handling that logs the failure and shows appropriate user feedback
- **FR-018**: System MUST log at warn or debug level before discarding error results via `.ok()` or `let _ =`, unless the discarded result is non-meaningful (e.g., stderr write)

**Phase D: Code Structure (HIGH/MEDIUM)**

- **FR-019**: System MUST consolidate duplicate command handlers into a single parameterized operation handler, then decompose `commands.rs` into domain-focused modules (backup commands, catalog commands, operation commands) each under 500 lines
- **FR-020**: System MUST decompose `run_orchestrated_operation()` into focused sub-functions (plan-building, execution-dispatch, event-forwarding) each under 50 lines
- **FR-021**: System MUST simplify the orchestrator abstraction by inlining the single-implementation trait or reducing type parameters
- **FR-022**: System MUST decompose files exceeding 500 lines into domain-focused modules

**Phase E: Logging (MEDIUM)**

- **FR-023**: All public async functions performing I/O MUST have `#[tracing::instrument]` with appropriate `skip_all` and structured fields
- **FR-024**: All error logs MUST include structured context fields (path, package, operation_id, duration_ms as applicable)
- **FR-025**: Log levels MUST be consistent: info for user-visible actions, debug for internals, warn for recoverable issues, error for action-required failures
- **FR-026**: Frontend catch blocks MUST use the logger utility (not inline `addEntry()` or empty catches)

**Phase F: Frontend Lifecycle (MEDIUM)**

- **FR-027**: Components MUST cancel or ignore results from pending invoke calls when unmounting (mounted-flag pattern)
- **FR-028**: Watch hooks MUST guard against stale updates using request identifiers or timestamps
- **FR-029**: VueQuery queries MUST invalidate stale responses when newer requests have been dispatched

**Phase G: Dependencies (LOW)**

- **FR-030**: Cargo dependency feature flags MUST list only required features (no "full" feature sets)
- **FR-031**: Frontend packages MUST be updated to latest compatible minor/patch versions

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero path traversal vectors — all path-accepting functions validate inputs against allowlists, confirmed by test cases with malicious paths
- **SC-002**: Application remains operational after any single command panic — no cascading failures from mutex poisoning
- **SC-003**: Zero `unwrap()` calls in production code paths (excluding compile-time constants and test code)
- **SC-004**: 100% of frontend `invoke()` calls have error handling with user feedback
- **SC-005**: 100% of public async I/O functions have tracing instrumentation
- **SC-006**: No source file exceeds 500 lines (excluding generated code and tests)
- **SC-007**: All error-discarding patterns (`let _ =`, `.ok()`) are preceded by a log statement or documented as intentionally silent
- **SC-008**: All spawned tasks have panic boundaries with logging
- **SC-009**: No blocking file operations in async command handlers
- **SC-010**: All SQL query values are parameterized

## Assumptions

- Test code (`#[cfg(test)]` blocks, `tests/` directories) is excluded from `unwrap()` auditing — `unwrap()` is acceptable in tests
- The 2 GB max download size is sufficient for all known astrophotography software installers
- `parking_lot::Mutex` is an acceptable replacement for `std::sync::Mutex` in all Tauri state contexts (it is `Send + Sync` and works with Tauri's state management)
- The mounted-flag pattern (checking a boolean ref before applying invoke results) is sufficient for Tauri invoke lifecycle safety, since Tauri's IPC does not support AbortController
- TypeScript 6 and vue-tsc 3 major version upgrades are evaluated but deferred to a separate spec if they require migration effort
- Files exceeding 500 lines where the excess is primarily test code are exempt from the decomposition requirement
- The sliding window restart limit (3 per 10 minutes) for critical tasks is a reasonable default; it may need tuning based on production telemetry
