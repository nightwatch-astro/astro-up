# Feature Specification: GUI File/Directory Browser Picker

**Feature Branch**: `025-file-picker`
**Created**: 2026-04-04
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Project ID**: PVT_kwDOECmZr84BTDgZ
**Input**: User description: "GUI file/directory browser picker for all path fields in Settings"
**Closes**: #690

## User Scenarios & Testing

### User Story 1 - Browse for a Directory (Priority: P1)

A user opens Settings and wants to change the download directory. Instead of manually typing or pasting a path, they click a browse button next to the Download Directory field. A native OS directory picker dialog opens. They navigate to their preferred folder, select it, and the path populates the input field automatically. The new path is saved via the existing auto-save mechanism.

**Why this priority**: This is the core interaction — most path fields are directories. Covers `download_dir`, `cache_dir`, and `data_dir`.

**Independent Test**: Can be fully tested by clicking the browse button next to any directory field in PathsSection, selecting a folder, and verifying the input field updates and the config persists after page reload.

**Acceptance Scenarios**:

1. **Given** the Settings view is open on the Paths tab, **When** the user clicks the browse button next to "Download Directory", **Then** a native directory picker dialog opens
2. **Given** the directory picker is open, **When** the user selects a folder and confirms, **Then** the selected path appears in the input field and auto-saves to config
3. **Given** the directory picker is open, **When** the user cancels the dialog, **Then** the input field retains its previous value and no save is triggered

---

### User Story 2 - Browse for a Log File (Priority: P2)

A user wants to change the log file location. They navigate to Settings > Logging, where the log file path is displayed. They click a browse button to open a native file picker filtered to `.log` files. After selecting a file (or typing a new filename), the path updates and saves.

**Why this priority**: Covers the file picker variant (as opposed to directory picker). Only one field (`log_file`) uses this pattern, but it completes the feature.

**Independent Test**: Can be tested by clicking the browse button next to the log file field, selecting or creating a `.log` file, and verifying the path updates and persists.

**Acceptance Scenarios**:

1. **Given** the Settings view is open on the Logging tab with "Log to file" enabled, **When** the user clicks the browse button next to the log file path, **Then** a native file save dialog opens with a `.log` file filter
2. **Given** the file picker is open, **When** the user selects or names a `.log` file and confirms, **Then** the selected path appears in the input field and auto-saves to config
3. **Given** the file picker is open, **When** the user cancels, **Then** the field retains its previous value

---

### Edge Cases

- What happens when the user selects a path that doesn't exist yet? The system accepts the path — directory/file creation is handled at runtime by the core, not at config time.
- What happens when the user selects a path they don't have write access to? The system accepts the path — permission errors surface at runtime when the path is actually used.
- What happens when the log file picker is used but "Log to file" is disabled? The browse button is only visible when "Log to file" is enabled, matching the existing conditional display.

## Requirements

### Functional Requirements

- **FR-001**: Each directory path field (`download_dir`, `cache_dir`, `data_dir`) MUST have a browse button that opens a native OS directory picker dialog
- **FR-002**: The log file path field (`log_file`) MUST have a browse button that opens a native file save dialog with a `.log` file extension filter
- **FR-003**: When the user selects a path in the dialog, the corresponding input field MUST update with the selected path
- **FR-004**: After a path is selected via the browse dialog, the config MUST auto-save using the existing debounced save flow
- **FR-005**: When the user cancels the dialog, the input field MUST retain its previous value with no save triggered
- **FR-006**: The `data_dir` field MUST be rendered in PathsSection as an editable input with a browse button (it exists in config but is not currently displayed)
- **FR-007**: The `log_file` field in LoggingSection MUST become an editable InputText (replacing the current read-only display) when "Log to file" is enabled
- **FR-008**: Browse buttons MUST use the PrimeVue InputGroup pattern (InputText + Button side by side)
- **FR-009**: The directory picker dialogs SHOULD open to the currently configured path as the default location (if the path exists)

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can select a directory for any path field in under 5 seconds (click browse, navigate, confirm)
- **SC-002**: Users can select a log file with the `.log` filter applied without manually typing paths
- **SC-003**: All 4 path fields (`download_dir`, `cache_dir`, `data_dir`, `log_file`) have functional browse buttons
- **SC-004**: Selected paths persist across application restarts via the existing config save mechanism
- **SC-005**: Cancelling the dialog leaves the field unchanged 100% of the time

## Assumptions

- The Tauri dialog plugin is already installed on the Rust side and permissions are configured. Only the frontend npm package needs to be added.
- The existing auto-save debounce (500ms deep watch) in SettingsView handles saving after browse selection — no new save logic is needed.
- `data_dir` uses the same display and interaction pattern as `download_dir` and `cache_dir`.
- The log file picker uses a "save" dialog (allowing new filenames) rather than an "open" dialog (requiring existing files).
- Windows is the primary target, but the dialog API is cross-platform so no platform-specific code is needed.
