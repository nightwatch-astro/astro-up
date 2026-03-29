# Feature Specification: Tauri App Shell

**Feature Branch**: `016-tauri-app-shell`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 015 — Tauri v2 application setup

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Launch Desktop Application (Priority: P1)

A user double-clicks the astro-up icon and the application window opens with the software dashboard. The window remembers its size and position from the last session.

**Why this priority**: Application launch is the first user interaction. Must be fast and reliable.

**Independent Test**: Launch the app, resize window, close, relaunch — verify window state is restored.

**Acceptance Scenarios**:

1. **Given** the app is installed, **When** launched, **Then** the main window opens within 2 seconds
2. **Given** a previous session resized the window, **When** relaunched, **Then** window position and size are restored
3. **Given** the app is already running, **When** launched again, **Then** the existing window is focused (single instance)

---

### User Story 2 - System Tray (Priority: P2)

The application runs in the system tray showing an update count badge. Right-clicking shows quick actions: check for updates, open window, quit.

**Why this priority**: Tray presence enables background update checking without keeping the window open.

**Independent Test**: Minimize to tray, verify icon appears. Right-click, verify menu items work.

**Acceptance Scenarios**:

1. **Given** the app is running, **When** minimized, **Then** it appears in the system tray
2. **Given** 3 updates are available, **When** checking the tray icon, **Then** a badge shows "3"
3. **Given** right-click on tray icon, **When** selecting "Check for Updates", **Then** update check runs

---

### User Story 3 - Tauri Commands (Priority: P3)

The frontend calls Rust functions via Tauri's `invoke()` mechanism. Each command wraps astro-up-core functionality: list software, check updates, update, scan, config read/write.

**Why this priority**: Commands bridge the frontend to the core logic. Every GUI feature depends on this.

**Independent Test**: Call each Tauri command from the frontend, verify correct data is returned.

**Acceptance Scenarios**:

1. **Given** the frontend calls `invoke('list_software')`, **When** the core returns data, **Then** the frontend receives a typed JSON array of software entries
2. **Given** the frontend calls `invoke('update_software', {id: 'nina-app'})`, **When** the update runs, **Then** progress events are emitted to the frontend in real time

---

### User Story 4 - Auto-Update (Priority: P4)

On startup, the application checks for a new version of itself. If available, it notifies the user and offers to install the update (download + restart).

**Why this priority**: Self-update keeps users on the latest version without manual intervention.

**Independent Test**: Point the update endpoint to a test server with a newer version, verify update is offered.

**Acceptance Scenarios**:

1. **Given** a newer version is available, **When** the app starts, **Then** a notification offers the update
2. **Given** the user accepts, **When** the update downloads, **Then** the app restarts with the new version
3. **Given** no update is available, **When** checking, **Then** no notification is shown

### Edge Cases

- WebView2 not installed (Windows 10 without Edge): Show error with download link for WebView2 runtime.
- Update endpoint unreachable: Skip self-update check silently, log a warning.
- Update download fails mid-download: Retry once, then inform user. Never leave the app in a broken state.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use Tauri v2 for the desktop application shell
- **FR-002**: System MUST expose Tauri commands for: list_software, check_updates, update_software, scan_installed, get_config, save_config, add_custom_tool, remove_custom_tool
- **FR-003**: System MUST emit Tauri events for real-time progress (download, install, scan)
- **FR-004**: System MUST persist window state (position, size) via tauri-plugin-window-state
- **FR-005**: System MUST enforce single instance via tauri-plugin-single-instance
- **FR-006**: System MUST provide system tray with update badge and context menu
- **FR-007**: System MUST support self-update via tauri-plugin-updater with Ed25519 signatures
- **FR-008**: System MUST provide an update endpoint JSON for the updater (hosted on GitHub Releases)
- **FR-009**: System MUST require WebView2 runtime and report a clear error if missing
- **FR-010**: System MUST support autostart via tauri-plugin-autostart (opt-in, configurable)

### Key Entities

- **TauriCommand**: A #[tauri::command] function bridging frontend invoke() to core crate logic
- **TauriEvent**: A typed event emitted from Rust to the frontend (progress, status changes)
- **UpdateEndpoint**: JSON file describing the latest version, download URL, and Ed25519 signature

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Application cold start to interactive window in under 3 seconds
- **SC-002**: Tauri command round-trip (invoke → response) completes in under 100ms for cached operations
- **SC-003**: Real-time events reach the frontend within 50ms of being emitted from Rust
- **SC-004**: Self-update downloads and installs without manual intervention

## Assumptions

- Tauri v2 is the application framework (not Electron, not Wails)
- Windows 10+ is required (WebView2 included by default on Windows 11, optional on 10)
- The Tauri shell is a thin adapter — all business logic lives in astro-up-core
- Ed25519 key pair is generated during CI setup and the public key is embedded in the app
- Depends on: spec 004 (config), spec 012 (engine for update operations)
