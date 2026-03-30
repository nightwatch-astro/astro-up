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
2. **Given** 3 updates are available, **When** checking the tray icon, **Then** a badge shows "3"
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

1. **Given** a newer version is available, **When** the app starts, **Then** a notification offers the update
2. **Given** the user accepts, **When** the update downloads, **Then** the app restarts with the new version
3. **Given** no update is available, **Then** no notification is shown

### Edge Cases

- WebView2 not installed (Windows 10 without Edge): Show native dialog with WebView2 download link.
- Update endpoint unreachable: Skip self-update check silently, log warning.
- Update download fails: Retry once, then inform user. Never leave app in broken state.
- Multiple monitors with different DPI: Window respects per-monitor DPI scaling.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use Tauri v2 for the desktop application shell
- **FR-002**: System MUST expose Tauri commands for: list_software (with filter), install_software, update_software, scan_installed, search_catalog, create_backup, restore_backup, get_config, save_config
- **FR-003**: System MUST emit typed Tauri events for real-time progress (download, install, scan)
- **FR-004**: System MUST persist window state (position, size) via tauri-plugin-window-state
- **FR-005**: System MUST enforce single instance via tauri-plugin-single-instance
- **FR-006**: System MUST provide system tray with update badge and context menu
- **FR-007**: System MUST support configurable close button behavior: minimize to tray (default) or quit
- **FR-008**: System MUST support self-update via tauri-plugin-updater with Ed25519 signatures
- **FR-009**: System MUST publish an update endpoint JSON on GitHub Releases for the updater
- **FR-010**: System MUST require WebView2 runtime and show native error dialog if missing
- **FR-011**: System MUST support autostart via tauri-plugin-autostart (opt-in, configurable)
- **FR-012**: System MUST support three theme modes: system (default), light, dark — configurable in settings
- **FR-013**: System MUST use separate Ed25519 key pair for app updates (not shared with catalog minisign key)
- **FR-014**: System MUST scope file system permissions to the app's own directories only
- **FR-015**: Tauri commands MUST be thin adapters — all business logic lives in astro-up-core

### New Config Keys (spec 004)

| Key | Default | Description |
|-----|---------|-------------|
| `ui.close_action` | `"minimize"` | Close button behavior: "minimize" or "quit" |
| `ui.theme` | `"system"` | Theme mode: "system", "light", "dark" |
| `ui.autostart` | `false` | Start with Windows |

### Key Entities

- **TauriCommand**: A `#[tauri::command]` function bridging `invoke()` to core crate logic
- **TauriEvent**: Typed event emitted from Rust to frontend (progress, status changes, update notification)
- **UpdateEndpoint**: JSON file on GitHub Releases with latest version, download URL, Ed25519 signature

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Application cold start to interactive window in under 3 seconds
- **SC-002**: Tauri command round-trip (invoke → response) under 100ms for cached operations
- **SC-003**: Real-time events reach the frontend within 50ms of emission
- **SC-004**: Self-update downloads and installs without manual intervention

## Assumptions

- Tauri v2 is the application framework
- Windows 10+ required (WebView2 included by default on 11, optional on 10)
- The Tauri shell is a thin adapter — all business logic in astro-up-core
- Separate Ed25519 key pair for app updates (different secret from catalog signing)
- Depends on: spec 004 (config), spec 012 (engine for operations)
