# Feature Specification: Vue Frontend

**Feature Branch**: `017-vue-frontend`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 016 — Vue 3 + PrimeVue + VueQuery frontend

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Software Dashboard (Priority: P1)

A user opens the app and sees a sortable, filterable DataTable of all detected software with columns: name, category, installed version, latest version, status. Status badges show at-a-glance whether each package is up-to-date, has an update, or needs attention.

**Why this priority**: The dashboard is the main view users see 95% of the time.

**Independent Test**: Load the dashboard with test data, verify sorting, filtering, and status badges render correctly.

**Acceptance Scenarios**:

1. **Given** 30 installed packages, **When** the dashboard loads, **Then** all are shown in a DataTable with correct status badges
2. **Given** the user clicks a column header, **When** sorting, **Then** the table sorts by that column
3. **Given** the user types in the search box, **When** filtering, **Then** the table filters by name, category, or tags in real time

---

### User Story 2 - Update with Progress (Priority: P2)

A user clicks "Update All" or selects specific packages to update. A progress overlay shows download and install status for each package with progress bars.

**Why this priority**: Visual progress is the primary value of the GUI over the CLI.

**Independent Test**: Trigger an update, verify progress bars animate and completion is reported.

**Acceptance Scenarios**:

1. **Given** 3 packages have updates, **When** "Update All" is clicked, **Then** a progress panel shows each package's download/install status
2. **Given** a download is in progress, **When** the progress bar updates, **Then** it shows bytes/total, speed, and estimated time
3. **Given** an update completes, **When** the summary shows, **Then** success/failure per package is listed

---

### User Story 3 - Settings Page (Priority: P3)

A user navigates to Settings and sees a form with configuration options: download directory, proxy, GitHub token, update check interval, log level, telemetry opt-in. Changes are saved to the TOML config file.

**Why this priority**: GUI config editing is the primary way most users will customize the app.

**Independent Test**: Change a setting, save, reload — verify the change persists.

**Acceptance Scenarios**:

1. **Given** the Settings page, **When** the user changes download_dir, **Then** the new path is shown in the input
2. **Given** changes are made, **When** "Save" is clicked, **Then** config.toml is updated and a success toast appears
3. **Given** an invalid value is entered, **When** saving, **Then** the field shows a validation error

---

### User Story 4 - Custom Tools Management (Priority: P4)

A user navigates to the Custom Tools view, pastes a GitHub URL, and the app guides them through adding a custom tool (asset selection, manifest preview, confirmation).

**Why this priority**: GUI makes custom tool addition more accessible than CLI commands.

**Independent Test**: Add a custom tool via the GUI, verify it appears in the dashboard.

**Acceptance Scenarios**:

1. **Given** a GitHub URL is pasted, **When** "Add" is clicked, **Then** release assets are fetched and shown in a selection dialog
2. **Given** a custom tool is registered, **When** viewing Custom Tools, **Then** it appears with an option to remove

### Edge Cases

- Empty dashboard (fresh install, no software detected): Show a welcome message with "Run Scan" button.
- Slow network during update: Progress bar shows stalled state, doesn't freeze the UI.
- Window resize: Dashboard table adapts responsively. No horizontal scroll at common resolutions.
- Dark mode: PrimeVue Aura dark theme follows system preference via `darkModeSelector: 'system'`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use Vue 3 Composition API with TypeScript strict mode
- **FR-002**: System MUST use PrimeVue 4 DataTable for the software dashboard
- **FR-003**: System MUST use PrimeVue Badge for software status indicators
- **FR-004**: System MUST use PrimeVue ProgressBar for download/install progress
- **FR-005**: System MUST use PrimeVue Toast for notifications (success, error, info)
- **FR-006**: System MUST use VueQuery 5 composables wrapping Tauri invoke() calls
- **FR-007**: System MUST support three views: Dashboard, Settings, Custom Tools
- **FR-008**: System MUST use PrimeVue Aura dark theme with system dark mode detection
- **FR-009**: System MUST receive real-time Tauri events and update the UI reactively
- **FR-010**: System MUST validate settings form inputs before saving
- **FR-011**: System MUST show a GitHub token input with password masking in Settings
- **FR-012**: System MUST be responsive at common resolutions (1280x720 minimum)

### Key Entities

- **SoftwareRow**: DataTable row with name, category, installed_version, latest_version, status badge
- **StatusBadge**: Visual indicator — green (up-to-date), blue (update), orange (major), red (error), gray (not installed)
- **UpdateProgress**: Per-package progress state (downloading %, installing, complete, failed)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Dashboard renders 100 packages in under 500ms
- **SC-002**: Real-time progress updates reflect in the UI within 100ms of event emission
- **SC-003**: Settings form validates and saves in under 1 second
- **SC-004**: All views are usable at 1280x720 without horizontal scrolling

## Assumptions

- No client-side routing library — simple `ref('dashboard')` state for 3 views
- VueQuery handles caching, loading states, and error states for all Tauri invoke() calls
- The frontend has no direct file system access — all operations go through Tauri commands
- PrimeVue provides all UI components — no additional component library needed
- Depends on: spec 016 (Tauri commands and events)
