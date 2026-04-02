# Feature Specification: Tauri App Shell

**Feature Branch**: `016-tauri-app-shell`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 015 — Tauri v2 application setup

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Launch Desktop Application (Priority: P1)

A user double-clicks the astro-up icon and the application window opens. The window remembers its size and position from the last session.

**Why this priority**: Application launch is the first user interaction. Must be fast and reliable.

**Independent Test**: Launch the app, resize window, close, relaunch — verify state is restored.

**Acceptance Scenarios**:

1. **Given** the app is installed, **When** launched, **Then** the main window opens within 2 seconds
2. **Given** a previous session resized the window, **When** relaunched, **Then** window position and size are restored
3. **Given** the app is already running, **When** launched again, **Then** the existing window is focused (single instance)

---

### User Story 2 - System Tray (Priority: P2)

The application runs in the system tray showing an update count badge. Right-clicking shows quick actions. The close button behavior is configurable — minimize to tray (default) or quit.

**Why this priority**: Tray presence enables background update checking without keeping the window open.

**Independent Test**: Minimize to tray, verify icon appears. Right-click, verify menu. Test close button in both modes.

**Acceptance Scenarios**:

1. **Given** the app is running, **When** minimized, **Then** it appears in the system tray
2. **Given** 3 updates are available, **When** checking the tray icon, **Then** a numeric overlay badge on the icon shows "3" (Windows tray icon overlay)
3. **Given** close action is "minimize" (default), **When** the user clicks the close button, **Then** the window hides and the tray icon remains
4. **Given** close action is "quit", **When** the user clicks the close button, **Then** the app exits completely
5. **Given** right-click on tray icon, **When** selecting "Check for Updates", **Then** update check runs

---

### User Story 3 - Tauri Commands (Priority: P3)

The frontend calls Rust functions via Tauri's `invoke()` mechanism. Commands mirror the CLI operations: list/filter software, install, update, scan, search, backup, restore, config read/write. Each command wraps astro-up-core functionality.

**Why this priority**: Commands bridge the frontend to the core logic. Every GUI feature depends on this.

**Independent Test**: Call each Tauri command from the frontend, verify correct data is returned.

**Acceptance Scenarios**:

1. **Given** the frontend calls `invoke('list_software', {filter: 'installed'})`, **Then** it receives a typed JSON array of installed software
2. **Given** the frontend calls `invoke('update_software', {id: 'nina'})`, **Then** progress events are emitted in real time
3. **Given** the frontend calls `invoke('search_catalog', {query: 'plate'})`, **Then** FTS5 search results are returned
4. **Given** the frontend calls `invoke('get_config')`, **Then** the effective configuration is returned as typed JSON

---

### User Story 4 - Theme Support (Priority: P4)

The user can choose between system theme (follows OS), light mode, or dark mode. The setting persists across sessions.

**Why this priority**: Astrophotographers image at night — dark mode is essential. But daytime use benefits from light mode.

**Independent Test**: Switch between themes, verify the UI updates. Restart, verify the setting persists.

**Acceptance Scenarios**:

1. **Given** theme is set to "system" (default), **When** OS is in dark mode, **Then** the app uses dark theme
2. **Given** theme is set to "dark", **When** OS is in light mode, **Then** the app still uses dark theme
3. **Given** theme is changed in settings, **When** saved, **Then** the theme updates immediately without restart

---

### User Story 5 - Auto-Update (Priority: P5)

On startup, the application checks for a new version of itself. If available, it notifies the user and offers to install the update.

**Why this priority**: Self-update keeps users on the latest version without manual intervention.

**Independent Test**: Point update endpoint to a test server with a newer version, verify update is offered.

**Acceptance Scenarios**:

1. **Given** a newer version is available, **When** the app starts, **Then** a toast notification shows "Update available: vX.Y.Z" with "Install" and "Dismiss" actions
2. **Given** the user clicks "Install", **When** the update downloads, **Then** the app restarts with the new version
3. **Given** the user clicks "Dismiss", **Then** the notification is removed; the update is offered again on next launch
4. **Given** no update is available, **Then** no notification is shown

### Edge Cases

- WebView2 not installed (Windows 10 without Edge): Show native dialog with WebView2 download link.
- WebView2 present but outdated or corrupted: Attempt WebView2 bootstrapper download; if that fails, show native dialog.
- Update endpoint unreachable: Skip self-update check silently, log warning.
- Update download fails: Retry once, then inform user. Never leave app in broken state.
- Self-update signature verification fails: Reject update, delete downloaded file, inform user via toast.
- Self-update fails mid-install: Atomic replace ensures current binary is never corrupted; inform user of failure.
- Multiple monitors with different DPI: Window respects per-monitor DPI scaling.
- Window state restore with invalid coordinates (removed monitor): Reset to centered on primary monitor at default size.
- First launch (no prior state): Open centered on primary monitor at default size (1024x768), config initialized with defaults.
- Core operation fails mid-stream (network, permission, hash mismatch): Show toast notification with error summary and "View Details" link to error log panel.
- User closes window during active operation: Prompt with "cancel or continue in background?" choice.
- User cancels operation: CancellationToken triggers graceful cleanup — partial downloads removed, ledger entries rolled back, incomplete backup archives deleted.
- Concurrent operations: Multiple operations may run simultaneously (e.g., installing one package while scanning). Each gets its own CancellationToken and event stream. The UI shows a stacked progress view.
- User declines self-update: Dismiss notification; ask again on next launch (no permanent snooze).
- Background check finds updates while window is hidden: Show a system notification with update count; clicking it opens/focuses the main window.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use Tauri v2 for the desktop application shell
- **FR-002**: System MUST expose Tauri commands as thin adapters — all business logic lives in astro-up-core
- **FR-003**: System MUST forward all `astro-up-core::Event` variants to the frontend as typed Tauri events (adjacently tagged JSON: `{"type": "...", "data": {...}}`)
- **FR-004**: System MUST persist window state (position, size) via tauri-plugin-window-state
- **FR-005**: System MUST enforce single instance via tauri-plugin-single-instance
- **FR-006**: System MUST provide system tray with context menu: "Show Window", "Check for Updates", separator, "Quit"
- **FR-007**: System MUST display an update count badge on the tray icon when updates are available
- **FR-008**: System MUST support configurable close button behavior: minimize to tray (default) or quit
- **FR-009**: System MUST support self-update via tauri-plugin-updater with Ed25519 signatures
- **FR-010**: System MUST publish an update endpoint JSON on GitHub Releases containing: `version`, `notes`, `pub_date`, and per-platform `url` + `signature` fields
- **FR-011**: System MUST require WebView2 runtime and show native error dialog if missing; if present but outdated/corrupted, attempt WebView2 bootstrapper download
- **FR-012**: System MUST support autostart via tauri-plugin-autostart (opt-in, configurable); when autostart is enabled, launch minimized to tray (no window)
- **FR-013**: System MUST support three theme modes: system (default), light, dark — configurable in settings
- **FR-014**: System MUST use separate Ed25519 key pair for app updates (not shared with catalog minisign key); key generation and rotation documented in release runbook
- **FR-015**: System MUST scope file system permissions to: app data dir (`directories::data_dir()/astro-up`), app config dir, app cache dir, and app log dir only
- **FR-016**: System MUST allow users to cancel any in-progress operation from the GUI via CancellationToken
- **FR-017**: System MUST prompt "cancel or continue in background?" when closing the window during an active operation
- **FR-018**: System MUST display errors as toast notifications with a "View Details" link to an error log panel; the panel retains the last 100 entries per session
- **FR-019**: System MUST check for software updates every 6 hours in the background (matching catalog refresh cadence), configurable via `ui.check_interval`; when updates are found while the window is hidden, show a system notification with update count
- **FR-020**: System MUST apply a Content Security Policy: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'` — no external resource loading
- **FR-021**: When a self-update signature verification fails, the update MUST be rejected, the downloaded file deleted, and the user informed via toast notification
- **FR-022**: When a self-update fails mid-install, the system MUST retain the current binary and inform the user; the updater uses atomic replace (temp file + rename) so partial writes never corrupt the running binary
- **FR-023**: When a cancelled operation has already begun writing state, CancellationToken triggers graceful cleanup: partial downloads removed, ledger entries rolled back, backup archives deleted if incomplete

### Tauri Command Contracts

Each command wraps a core crate function. Return types are serialized as JSON to the frontend.

| Command | Parameters | Returns | Error |
|---------|-----------|---------|-------|
| `list_software` | `filter: String` (all, installed, outdated, category:{name}) | `Vec<SoftwareEntry>` | `CoreError` |
| `install_software` | `id: String` | `OperationId` (events stream progress) | `CoreError` |
| `update_software` | `id: String` | `OperationId` (events stream progress) | `CoreError` |
| `scan_installed` | — | `ScanResult` | `CoreError` |
| `search_catalog` | `query: String` | `Vec<SearchResult>` | `CoreError` |
| `check_for_updates` | — | `Vec<UpdateAvailable>` | `CoreError` |
| `create_backup` | `paths: Vec<String>` | `BackupInfo` | `CoreError` |
| `restore_backup` | `archive: String, filter: Option<Vec<String>>` | `RestoreResult` | `CoreError` |
| `get_config` | — | `AppConfig` | `CoreError` |
| `save_config` | `config: AppConfig` | `()` | `CoreError` |
| `cancel_operation` | `operation_id: String` | `()` | `CoreError` |

All long-running commands (`install_software`, `update_software`, `scan_installed`, `create_backup`, `restore_backup`) emit `Event` variants via the Tauri event bus. The frontend subscribes to a single `"core-event"` channel.

### New Config Keys (spec 004)

| Key | Default | Description |
|-----|---------|-------------|
| `ui.close_action` | `"minimize"` | Close button behavior: "minimize" or "quit" |
| `ui.theme` | `"system"` | Theme mode: "system", "light", "dark" |
| `ui.autostart` | `false` | Start with Windows |
| `ui.check_interval` | `"6h"` | Background update check interval |

### Key Entities

- **TauriCommand**: A `#[tauri::command]` function bridging `invoke()` to core crate logic
- **TauriEvent**: Typed event emitted from Rust to frontend (progress, status changes, update notification)
- **UpdateEndpoint**: JSON file on GitHub Releases with latest version, download URL, Ed25519 signature

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Application cold start to first meaningful paint (WebView loaded, main UI rendered) in under 3 seconds
- **SC-002**: Tauri command round-trip (invoke → response) under 100ms for read operations that don't hit the network (list_software, get_config, search_catalog)
- **SC-003**: Real-time events reach the frontend within 50ms of emission from the core crate
- **SC-004**: Self-update downloads and installs without manual intervention beyond accepting the notification

### Non-Functional Requirements

- **NFR-001**: Tray-resident mode (window hidden) MUST use under 50MB RSS memory
- **NFR-002**: Background update checks MUST NOT cause visible CPU spikes (< 5% sustained)
- **NFR-003**: GUI layer MUST use the same tracing infrastructure as CLI (spec 015): stderr for user-visible warnings, file-based JSON logs for debug. Tauri command invocations and event emissions logged at debug level.
- **NFR-004**: Windows-only application; Linux/macOS: compile but show "Windows required" at launch. System tray unavailability is not a concern for the target platform.

## Assumptions

- Tauri v2 is the application framework
- Windows 10+ required (WebView2 included by default on 11, optional on 10)
- The Tauri shell is a thin adapter — all business logic in astro-up-core
- Separate Ed25519 key pair for app updates (different secret from catalog signing)
- Depends on: spec 004 (config), spec 012 (engine for operations)

## Clarifications

### Session 2026-04-02

- Q: Can users cancel in-progress operations from the GUI, and what happens on window close during an operation? → A: Users can cancel any operation; closing window during operation prompts "cancel or background?"
- Q: How are errors from core operations surfaced in the GUI? → A: Toast notifications with "View Details" link to an error log panel
- Q: How often does the app check for software updates in the background? → A: Every 6 hours (matches catalog refresh cadence), configurable via `ui.check_interval`
