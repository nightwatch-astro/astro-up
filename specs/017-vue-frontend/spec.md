# Feature Specification: Vue Frontend

**Feature Branch**: `017-vue-frontend`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 016 — Vue 3 + PrimeVue + VueQuery frontend

## Stack

| Layer | Library | Purpose |
|-------|---------|---------|
| Framework | Vue 3 Composition API + TypeScript strict | Core |
| Components | PrimeVue 4 (Aura theme) | UI components, DataTable, forms, dialogs |
| Server state | @tanstack/vue-query 5 | Caching, loading, error handling for Tauri invoke() |
| Client state | Pinia | Filters, wizard state, UI preferences, operation progress |
| Forms | VeeValidate + Zod | Settings page, wizard form validation |
| Icons | unplugin-icons + Iconify (Lucide primary) | Tree-shaken icons, mix sets as needed |
| Utilities | VueUse | `useLocalStorage`, `useDark`, `useMediaQuery`, etc. |
| Animations | @vueuse/motion | List transitions, drawer slide-ins, progress overlays |
| Bridge | @tauri-apps/api | Typed invoke(), event listeners |

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Software Dashboard (Priority: P1)

A user opens the app and sees a single filterable DataTable of all software. Toolbar filter chips toggle between All / Installed / Outdated. A category dropdown narrows further. Free-text search uses FTS5. Each row shows name, category, version info, and a contextual action button (Install / Update / Up to date).

**Why this priority**: The dashboard is the main view users see 95% of the time.

**Independent Test**: Load the dashboard with test data, toggle filters, verify correct packages are shown.

**Acceptance Scenarios**:

1. **Given** 95 packages in catalog, 12 installed, 3 outdated, **When** dashboard loads, **Then** the DataTable shows all 95 with status indicators
2. **Given** the user clicks `[Installed(12)]` chip, **Then** the table filters to 12 installed packages
3. **Given** the user clicks `[Outdated(3)]` chip, **Then** the table filters to 3 packages with updates, badge shows count
4. **Given** the user types "plate" in search, **Then** FTS5 results show PlateSolve, ASTAP, All-Sky Plate Solver
5. **Given** the user clicks a row, **Then** the row expands showing detailed package info (publisher, license, homepage, dependencies, backup status)
6. **Given** a row for an uninstalled package, **Then** the action button shows `[Install]`
7. **Given** a row for an outdated package, **Then** the action button shows `[Update]`

---

### User Story 2 - Install and Update with Progress (Priority: P2)

A user clicks Install or Update and sees a progress overlay showing download and install status per package. Bulk operations show a queue with per-package progress.

**Why this priority**: Visual progress is the primary GUI value over CLI.

**Independent Test**: Trigger an update, verify progress overlay animates and completion is reported.

**Acceptance Scenarios**:

1. **Given** the user clicks `[Update]` on NINA, **Then** a progress overlay slides in showing download progress bar
2. **Given** `[Update All]` with 3 packages, **Then** the overlay shows a queue: current package progress + remaining count
3. **Given** an update completes, **Then** a toast notification confirms success and the row's status badge updates
4. **Given** an update fails, **Then** a toast shows the error with the semantic reason (e.g., "Package in use")
5. **Given** the user clicks cancel during download, **Then** the operation stops and a "cancelled" toast appears

---

### User Story 3 - First-Run Experience (Priority: P3)

On first launch, the user sees a welcome screen with three choices: Scan (detect existing software), Set Up My Rig (hardware-based bundle installer), or Skip (go straight to software list). This choice screen also accessible later via a "Setup Wizard" menu item.

**Why this priority**: First impression determines whether users keep the app.

**Independent Test**: Launch with no prior data, verify the welcome screen appears. Complete each path.

**Acceptance Scenarios**:

1. **Given** first launch, **When** the app opens, **Then** the welcome screen shows three options
2. **Given** the user selects "Scan my system", **Then** detection runs with a progress stepper, then the dashboard shows results
3. **Given** the user selects "Set up my rig", **Then** the hardware wizard opens (see User Story 4)
4. **Given** the user selects "Skip", **Then** the dashboard opens with an empty state and a "Scan" CTA
5. **Given** subsequent launches, **Then** the welcome screen is not shown — straight to dashboard

---

### User Story 4 - Hardware Setup Wizard (Priority: P4)

The user selects their use case (deep sky imaging, planetary, live stacking) and hardware (mount, camera, guider, focuser, filter wheel). The wizard recommends a software bundle (NINA + PHD2 + ASCOM Platform + hardware-specific drivers) and installs everything.

**Why this priority**: Killer feature for new astrophotographers — one-click rig setup.

**Independent Test**: Select "deep sky" + ZWO camera + iOptron mount, verify the correct bundle is recommended.

**Acceptance Scenarios**:

1. **Given** the wizard starts, **When** step 1, **Then** the user selects use case: Deep Sky / Planetary / Live Stacking / EAA
2. **Given** use case selected, **When** step 2, **Then** hardware selection with multiple items per category (users may have 2 cameras — imaging + guiding)
3. **Given** hardware selected, **When** step 3, **Then** a recommended bundle is shown: core apps (NINA, PHD2) + ASCOM Platform + specific drivers for selected hardware
4. **Given** the user reviews the bundle, **When** step 4, **Then** all packages are installed in dependency order with progress
5. **Given** the wizard completes, **Then** a summary shows what was installed and the user is taken to the dashboard
6. **Given** the wizard is accessed later via "Setup Wizard" menu, **Then** it shows previously selected hardware as defaults

---

### User Story 5 - Settings Page (Priority: P5)

A user navigates to Settings and sees a form with grouped configuration options. Changes are validated and saved to the TOML config file.

**Why this priority**: GUI config editing is the primary way most users customize the app.

**Independent Test**: Change a setting, save, reload — verify persistence.

**Acceptance Scenarios**:

1. **Given** the Settings page opens, **Then** settings are grouped: Paths, Network, Updates, UI, Backup
2. **Given** the user changes download directory, **When** "Save" is clicked, **Then** config is updated and a success toast appears
3. **Given** an invalid value, **When** saving, **Then** Zod validation highlights the field with an error message
4. **Given** the theme toggle, **When** switched from System to Dark, **Then** the theme changes immediately

---

### User Story 6 - Backup and Restore (Priority: P6)

The software list row expansion includes backup/restore actions. Users can trigger manual backups and restore from the package detail view.

**Why this priority**: Backup/restore is a safety net best accessed in context of the package.

**Acceptance Scenarios**:

1. **Given** an expanded row for NINA, **Then** "Backup Now" and "Restore..." buttons are visible
2. **Given** the user clicks "Restore...", **Then** a dialog shows available backups with date, version, size
3. **Given** the user selects a backup, **Then** file change summary is shown before confirmation

### Edge Cases

- Empty dashboard (fresh install, no scan): Show centered welcome message with "Scan" or "Set Up My Rig" buttons
- Slow network during update: Progress bar shows stalled state, UI remains responsive
- Window resize: DataTable adapts. Minimum usable width: 1024px
- Theme: System/Light/Dark via `ui.theme` config. PrimeVue Aura theme supports all three.
- Multiple hardware per category in wizard: Allow adding multiple cameras (imaging + guiding), multiple mounts, etc.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use Vue 3 Composition API with TypeScript strict mode
- **FR-002**: System MUST use PrimeVue 4 DataTable as the core software list component with virtual scrolling, row expansion, row selection, and column sorting
- **FR-003**: System MUST use toolbar filter chips (All / Installed / Outdated) with badge counts
- **FR-004**: System MUST use PrimeVue category dropdown for category filtering
- **FR-005**: System MUST use FTS5 search via Tauri invoke for free-text filtering
- **FR-006**: System MUST show per-row contextual action buttons (Install / Update / Up to date)
- **FR-007**: System MUST show a progress overlay/drawer for install/update operations with per-package progress
- **FR-008**: System MUST use PrimeVue Toast for operation notifications (success, error, info)
- **FR-009**: System MUST provide a first-run welcome screen with: Scan / Set Up My Rig / Skip
- **FR-010**: System MUST provide a hardware setup wizard with: use case → hardware selection → bundle recommendation → install
- **FR-011**: System MUST allow multiple hardware items per category in the wizard (multiple cameras, mounts, etc.)
- **FR-012**: System MUST include ASCOM Platform and appropriate drivers in all wizard bundles
- **FR-013**: System MUST provide a Settings page with grouped, validated form fields
- **FR-014**: System MUST support three theme modes: System / Light / Dark with immediate switching
- **FR-015**: System MUST use VueQuery composables wrapping all Tauri invoke() calls
- **FR-016**: System MUST use Pinia for client-side state (filters, wizard, preferences, progress)
- **FR-017**: System MUST use VeeValidate + Zod for settings and wizard form validation
- **FR-018**: System MUST receive real-time Tauri events and update UI reactively
- **FR-019**: System MUST expose backup/restore actions in the package detail row expansion
- **FR-020**: System MUST make the setup wizard accessible from the main UI (not just first-run)
- **FR-021**: System MUST be responsive at 1024px minimum width

### Key Entities

- **SoftwareRow**: DataTable row — name, category, installed version, latest version, status badge, action button
- **StatusBadge**: Green (up to date), Blue (update available), Orange (major upgrade), Red (error), Gray (not installed), Purple (newer than catalog)
- **FilterState**: Pinia store — active filter chip, selected categories, search query
- **WizardState**: Pinia store — selected use case, hardware list, recommended bundle, install progress
- **UseCase**: Deep Sky / Planetary / Live Stacking / EAA — maps to recommended software bundles
- **HardwareSelection**: Category (mount/camera/guider/focuser/filter) + model + quantity

## Use Case → Bundle Mapping

| Use Case | Core Apps | Recommended |
|----------|-----------|-------------|
| Deep Sky | NINA, PHD2, ASCOM Platform | ASTAP or PlateSolve, Stellarium |
| Planetary | SharpCap or FireCapture, ASCOM Platform | PIPP, AutoStakkert, RegiStax |
| Live Stacking | SharpCap, ASCOM Platform | Stellarium |
| EAA (Electronic Assisted Astronomy) | SharpCap, ASCOM Platform | Stellarium |

All bundles include ASCOM Platform + drivers for selected hardware.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Dashboard renders 95 packages in under 500ms
- **SC-002**: Filter chip toggling re-renders in under 100ms
- **SC-003**: Real-time progress updates reflect within 100ms of event emission
- **SC-004**: Setup wizard completes a full rig install with progress feedback
- **SC-005**: All views usable at 1024px without horizontal scrolling

## Assumptions

- No client-side routing library — Pinia-driven view state (dashboard / settings / wizard)
- VueQuery handles caching and loading states for all Tauri invoke() calls
- No direct file system access from frontend — all operations through Tauri commands
- PrimeVue provides all UI components — no additional component library
- Bundle recommendations are hardcoded in v1 — configurable bundle definitions deferred
- Depends on: spec 016 (Tauri commands and events)
