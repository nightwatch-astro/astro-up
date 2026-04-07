# Feature Specification: Logging and Debugging

**Feature Branch**: `027-logging-debugging`
**Created**: 2026-04-07
**Status**: Draft
**Type**: implementation
**Project**: Astro-Up
**Project Number**: 3
**Project ID**: PVT_kwDOECmZr84BT6uK
**Input**: User description: "Comprehensive logging, error handling, and observability across the entire codebase (Rust backend + Vue frontend). Every action gets appropriate structured logging (debug/trace minimum, info+debug ideal). Every error properly handled — no silent swallowing. Every exception/non-standard behavior gets error or warn logging. Constitution updated with Principle VII (Observability). CLAUDE.md updated with logging conventions."

## User Scenarios & Testing

### User Story 1 - Developer Debugging a Failed Installation (Priority: P1)

A developer investigating why a software installation failed can trace the entire operation lifecycle through structured log output. Every step — catalog lookup, download, hash verification, installer execution, and ledger update — produces log entries with consistent fields (`package`, `operation_id`, `duration_ms`) that can be correlated.

**Why this priority**: Debugging failed operations is the most common developer need. Without logging in critical paths (process spawning, downloads), root cause analysis requires code reading instead of log reading.

**Independent Test**: Trigger a software installation with an invalid installer path. Verify that logs show entry, each substep, the specific failure point, and exit — all with the same `operation_id`.

**Acceptance Scenarios**:

1. **Given** a developer runs an install operation, **When** the installer fails with a non-zero exit code, **Then** logs show the full operation trace from command entry through process spawn, exit code, and error propagation — all at `info` or `error` level.
2. **Given** a developer enables `debug` verbosity, **When** any operation runs, **Then** additional context appears: method selection, timing, cache decisions, retry attempts.
3. **Given** a developer enables `trace` verbosity, **When** a detection scan runs, **Then** per-package, per-method probe results are visible including raw values and silent fallback decisions.

---

### User Story 2 - End User Sees Meaningful Errors in GUI (Priority: P1)

When an operation fails in the Tauri GUI, the user sees a clear error toast notification and can find details in the log panel. No operations fail silently — every mutation error surfaces to the user through the existing toast and error log infrastructure.

**Why this priority**: 8 of 9 VueQuery mutations currently have no error handling. Users experience silent failures with no feedback, making the application appear broken.

**Independent Test**: Disconnect network, trigger catalog sync from the GUI. Verify a toast error appears and the error log panel contains the failure details.

**Acceptance Scenarios**:

1. **Given** a user triggers any operation (install, update, backup, scan, sync), **When** the operation fails, **Then** a toast notification with severity "error" appears and an entry is added to the error log.
2. **Given** a user is on the Settings page, **When** the update check fails, **Then** a toast notification appears (not a browser `alert()` dialog).
3. **Given** a Vue component throws an unhandled error, **When** the error propagates, **Then** a global error boundary catches it, logs it, and shows a toast — the app does not crash.

---

### User Story 3 - Project Governance for Future Code (Priority: P2)

The project constitution and coding standards are updated so that every future spec and code contribution follows consistent logging and error handling conventions. New code that lacks appropriate logging or silently swallows errors is caught during review.

**Why this priority**: Without governance, the codebase will regress. Standards must be documented alongside the code changes.

**Independent Test**: Read the updated constitution and CLAUDE.md. Verify that logging level definitions, instrument requirements, and error handling rules are concrete and enforceable.

**Acceptance Scenarios**:

1. **Given** a new spec is being planned, **When** the constitution is consulted, **Then** Principle VII (Observability) defines concrete rules for logging levels, instrument usage, and error suppression.
2. **Given** a developer writes a new public async function, **When** they check CLAUDE.md coding standards, **Then** the logging conventions section specifies required structured logging, structured fields, and error handling patterns.

---

### User Story 4 - Silent Error Elimination in Backend (Priority: P2)

All error suppression patterns (`let _ =`, `.ok()`, bare `unwrap()` in I/O paths) are triaged and either replaced with proper error propagation or annotated with logging so failures are observable.

**Why this priority**: 679 `unwrap()` calls in core, 76 `let _ =` patterns, and 65 `.ok()` calls represent a large surface of unobservable failures. The most dangerous are in I/O, network, and Windows API paths.

**Independent Test**: Run `rg "unwrap()" --type rust` on I/O files (process.rs, discovery.rs, archive.rs) and confirm the count is reduced. Run `rg "let _ =" --type rust` and confirm remaining instances have adjacent log statements.

**Acceptance Scenarios**:

1. **Given** a process spawning operation, **When** the Windows API call fails, **Then** the error is propagated with proper error handling and logged at `error` level — not swallowed silently.
2. **Given** an event emission to the UI via a fire-and-forget send, **When** the send fails, **Then** a `debug` log records the failure.
3. **Given** a detection probe uses optional error conversion for enrichment, **When** the probe fails, **Then** a `trace` log records which method failed and the raw error.

---

### User Story 5 - Frontend Structured Logging (Priority: P3)

The Vue frontend has a structured logging utility that writes to the existing LogPanel infrastructure. Key user actions (navigation, button clicks, operation triggers) are logged at `debug` level, giving developers visibility into user behavior during issue investigation.

**Why this priority**: Frontend currently has 1 `console.error()` total. While the LogPanel and operations tracking exist, composables and views produce no diagnostic output.

**Independent Test**: Open the app, navigate between views, trigger an install. Open the LogPanel at `debug` filter level and verify route transitions, button clicks, and mutation triggers appear.

**Acceptance Scenarios**:

1. **Given** a frontend logging utility exists, **When** a developer imports it, **Then** it provides `debug()`, `info()`, `warn()`, `error()` methods that write to the LogPanel store.
2. **Given** a user navigates to a different view, **When** the route changes, **Then** a `debug` entry appears in the LogPanel.
3. **Given** a user clicks "Install", **When** the mutation fires, **Then** a `debug` entry with the command name and package appears in the LogPanel.

---

### Edge Cases

- What happens when tracing subscriber initialization fails? Falls back to stderr-only (already implemented in CLI).
- What happens when the LogPanel store exceeds capacity? Existing 1000-entry cap truncates oldest entries.
- What happens when structured logging is applied to a function with non-serializable fields? Use selective field inclusion for chosen values only.
- What happens when frontend logging is called outside a Vue component context? Logger utility must work in both component and composable contexts.
- What happens when an error suppression fix changes a function signature from infallible to fallible? Callers must be updated to handle the new error type.
- What happens when log output is written before the tracing subscriber is initialized (early startup) or after it is torn down (shutdown)? Early log calls MUST use stderr fallback; shutdown logging MUST flush pending spans before exit.
- What happens when disk is full and file log rotation fails? File logging degrades gracefully to stderr-only; no panic or crash.
- What happens when multiple errors fire rapidly in the Vue error boundary (error storm)? The error boundary MUST rate-limit toast notifications (max 3 per 5 seconds) to prevent toast flooding. All errors still logged to error log store.
- What happens when concurrent operations (multiple downloads/installs) produce interleaved logs? Each operation MUST carry a unique `operation_id` field for correlation.

## Requirements

### Functional Requirements

#### Governance

- **FR-001**: Constitution MUST include a new Principle VII (Observability) defining log level semantics (`error`, `warn`, `info`, `debug`, `trace`), instrument requirements, and error suppression rules.
- **FR-002**: Project coding standards MUST include a Logging & Error Handling section with concrete conventions for both Rust and Vue code.

#### Rust Backend — Instrumentation

- **FR-003**: All public async functions in the core library MUST have structured logging with context fields. Required minimum fields per operation type: operations require `operation_id` + `package`; network calls require `url` + `duration_ms`; file I/O requires `path`. Sync public functions with I/O are included; private helpers are excluded unless they perform I/O directly.
- **FR-004**: All operation boundaries (catalog fetch, download, install, scan, backup, restore) MUST log at `info` level on entry and exit with summary statistics (count, duration, result).
- **FR-005**: All developer-context decisions (method selection, cache hit/miss, retry logic, timing) MUST log at `debug` level.
- **FR-006**: Per-item details in loops (detection probes, file operations, version comparisons) MUST log at `trace` level. Functions called in tight loops MUST NOT use per-call span creation; use event macros (`trace!`) instead to avoid performance overhead.
- **FR-007**: Process spawning functions MUST log entry (executable path, timeout), exit (exit code, duration), and errors (spawn failure, timeout).
- **FR-008**: Download operations MUST log entry (URL, expected hash), exit (bytes, duration, cached/fresh), and retries (attempt count, reason).
- **FR-008a**: Concurrent operations MUST use unique `operation_id` fields in all log entries for correlation. Each top-level operation (install, update, scan, backup) generates an ID that propagates to all child spans.

#### Rust Backend — Error Handling

- **FR-009**: Silent `unwrap()` calls in I/O, network, database, and process paths MUST be replaced with proper error propagation and descriptive error context. In-scope paths: file read/write, HTTP requests, SQLite queries, process spawning, Windows API calls (registry, WMI, PE parsing, Job Objects). Out-of-scope: pure computation, string formatting, collection indexing with known bounds.
- **FR-010**: `unwrap()` MUST remain permitted for: mutex locking, compile-time constants, regex compilation, and test code.
- **FR-011**: Error suppression patterns (`let _ =`) MUST log at `debug` or `warn` when the suppressed result represents a meaningful failure. A "meaningful failure" is one where the suppressed error indicates data loss, resource leak, or broken user expectation (e.g., failed file deletion, failed DB write, failed lock release). Fire-and-forget event emissions are acceptable at `debug` level.
- **FR-012**: Optional error conversion patterns (`.ok()`) in critical paths MUST log at `debug` or `trace` with the raw error before converting. Critical paths: lock file operations, database queries, version parsing that affects upgrade decisions, detection probes that determine installed software state. Non-critical: optional UI enrichment, supplementary metadata collection.
- **FR-012a**: Structured log fields MUST NOT contain passwords, API tokens, or authentication credentials. User filesystem paths and package names are acceptable (logging is local-only).

#### Rust Backend — CLI & GUI Boundaries

- **FR-013**: CLI command handlers MUST log at `debug` on entry (with subcommand name and parsed args) and exit (with result summary). CLI MUST NOT duplicate logging already performed by core functions — boundary logging only (command dispatch, final result).
- **FR-014**: All GUI command handlers MUST have entry, exit, and error logging. Commands currently missing logging MUST be filled. GUI boundary logging covers: command name, operation_id assignment, duration, and error mapping. Core-level detail (download progress, detection steps) is logged by core, not re-logged by GUI.

#### Vue Frontend — Error Handling

- **FR-015**: A global error boundary MUST catch unhandled component errors, logging them and showing a user notification. When errors fire rapidly, toast notifications MUST be rate-limited (max 3 per 5 seconds) to prevent flooding; all errors still recorded in the error log store regardless of rate limiting.
- **FR-016**: All data mutation operations MUST have error callbacks that show notifications and write to the error log store. When both FR-015 (global boundary) and FR-016 (mutation callback) could fire for the same error, the mutation callback takes precedence and the global boundary MUST NOT duplicate the notification.
- **FR-017**: All browser `alert()` calls MUST be replaced with in-app toast notifications.
- **FR-018**: Direct `console.error()` calls for operation failures MUST be replaced with error log store entries and toast notifications.

#### Vue Frontend — Structured Logging

- **FR-019**: A frontend logging utility MUST exist that writes to the LogPanel store with `debug`, `info`, `warn`, `error` methods. Each method accepts a `context` string (component/composable name) and a `message` string.
- **FR-020**: Key user actions MUST be logged at `debug` level via the logging utility. Key actions are: route navigation, operation-triggering button clicks (install, update, backup, scan, sync), and settings saves. Search/filter keystrokes are excluded.
- **FR-021**: Data-fetching composables MUST log lifecycle events at `debug` level.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every operation failure (install, update, backup, scan, sync) produces a visible error notification in the GUI within 2 seconds of the error occurring (measured from backend error to toast display).
- **SC-002**: A developer can trace any failed operation from user action to root cause using only log output, without reading source code. Verified by: triggering 3 representative failure scenarios (network error, invalid installer, corrupt catalog) and confirming each produces a complete trace with correlated `operation_id`.
- **SC-003**: The number of silent `unwrap()` calls in I/O, network, and process paths (as defined in FR-009 scope) is reduced by at least 80% from the pre-implementation baseline. Baseline measured by `rg "unwrap()" --type rust` in the in-scope files before work begins.
- **SC-004**: Every `let _ =` and `.ok()` pattern either has a log statement within 3 lines, or has an inline code comment explaining why silent suppression is intentional.
- **SC-005**: The constitution contains Principle VII (Observability) with concrete, reviewable rules. Coding standards contain a Logging & Error Handling section. Both are verifiable by reading the documents and checking that each rule is specific enough to produce a pass/fail judgment during code review.
- **SC-006**: Zero browser `alert()` calls remain in the frontend codebase.
- **SC-007**: All GUI command handlers have entry, exit, and error logging coverage.

## Assumptions

- The existing logging infrastructure (tracing crate ecosystem) is sufficient — no new logging dependencies are needed for the backend.
- The existing Vue LogPanel component (1000-entry cap, level filtering) and errorLog store are reused — no new UI components needed.
- The existing toast notification service is available in all components that need error display.
- Error suppression triage focuses on runtime-fallible paths only — compile-time infallible patterns (mutex locks, constant regex, known-bounds indexing) are excluded.
- Frontend logging writes to the LogPanel store, not to the browser console — the browser console is not the target output.
- The anti-duplication principle applies: core functions own their logging; CLI/GUI adapters MUST NOT re-log what core already reports.
- This spec builds on logging infrastructure established by spec 015 (CLI dual-layer tracing) and spec 016 (GUI FrontendLogLayer). Those specs are complete; this spec extends their patterns to the full codebase.
- Startup/shutdown logging: early calls before subscriber init fall back to stderr; shutdown MUST flush pending spans.
