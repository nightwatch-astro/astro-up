# Feature Specification: Vue Frontend Views

**Feature Branch**: `022-vue-frontend`
**Created**: 2026-04-02
**Status**: Draft
**Input**: User description: "Build the Vue 3 frontend views for Astro-Up -- catalog browsing (search/filter/sort), package detail (versions, backup config, technical details), installed software management with updates, backup & restore with file-level preview, and settings. All views wired to existing Tauri commands from spec 016."
**Project**: Rust Migration
**Project Number**: 1
**Project ID**: PVT_kwDOECmZr84BTDgZ
**Design Reference**: `research/017-vue-frontend-design.md` (flows, components, behaviors), `research/017-vue-frontend-mockup.html` (interactive mockup)

## User Scenarios & Testing

### User Story 1 - Browse and Search the Software Catalog (Priority: P1)

A user opens Astro-Up and wants to discover what astrophotography software is available. They browse the catalog, filter by category (e.g., "Capture" or "Guiding"), and search by name or keyword. Each package card shows its name, publisher, description, category, and installation status (not installed, installed, or update available). Clicking a package navigates to its detail page.

**Why this priority**: Browsing the catalog is the entry point for all user workflows — install, update, and backup all start from discovering a package. Without this, the app has no value.

**Independent Test**: Can be fully tested by loading the catalog from the backend, rendering cards, filtering by category, and searching. Delivers the core discovery experience.

**Acceptance Scenarios**:

1. **Given** the catalog is loaded, **When** the user opens the Catalog page, **Then** all packages are displayed as cards in a responsive grid with name, publisher, truncated description, category tag, and status badge.
2. **Given** the catalog is displayed, **When** the user types "nina" in the search box, **Then** N.I.N.A. appears (search matches against package ID, name, description, publisher, category, and other metadata — case-insensitive).
3. **Given** the catalog is displayed, **When** the user clicks the "Capture" category chip, **Then** only packages in the Capture category are shown, and the chip is visually highlighted.
4. **Given** a category filter is active and the user types a search term, **When** both are applied, **Then** results are filtered by both category AND search term simultaneously.
5. **Given** a package card is displayed, **When** the user clicks it, **Then** they navigate to the package detail page with a breadcrumb back to the catalog.

---

### User Story 2 - View Package Details (Priority: P1)

A user clicks on a package and sees its full details organized in tabs: overview (version, category, method, license, dependencies), version history, backup configuration (for packages with backup paths), and technical details (detection and installation configuration). Action buttons in the hero section allow updating, installing, backing up, or visiting the homepage.

**Why this priority**: The detail page is where users make decisions (install, update, configure backup). It's the action hub for every package interaction.

**Independent Test**: Can be tested by navigating to a package detail page, verifying all tabs render correctly, and checking that action buttons are present based on package state.

**Acceptance Scenarios**:

1. **Given** the user navigates to a package detail, **When** the page loads, **Then** the hero section shows the package icon, name, publisher, description, and contextual action buttons (Update if outdated, Install if not installed, disabled if up-to-date, Backup Now if backup paths exist, Homepage if URL exists).
2. **Given** the user is on the detail page, **When** they switch between tabs, **Then** the correct content panel is displayed (Overview, Versions, Backup, Technical Details).
3. **Given** the package has backup paths, **When** the user views the Backup tab, **Then** they see manifest paths (read-only, tagged "MANIFEST"), custom paths (editable with add/remove), auto-backup toggles, and backup history with preview/delete actions.
4. **Given** the user clicks "Backup Now" in the hero section, **When** the confirmation dialog appears, **Then** it shows the paths that will be backed up and the current version, with Proceed/Cancel buttons.

---

### User Story 3 - Manage Installed Software and Updates (Priority: P1)

A user opens the Installed page to see all detected software grouped by update status. Packages with available updates are shown in a prominent "Updates Available" section with individual Update buttons. Up-to-date packages are listed below. The user can update individual packages, update all at once, trigger a system re-scan, or initiate a backup before updating.

**Why this priority**: Managing installed software and applying updates is the core value proposition of Astro-Up. Users need to see what's installed, what needs updating, and take action.

**Independent Test**: Can be tested by loading installed packages, verifying grouping, and triggering update/scan actions.

**Acceptance Scenarios**:

1. **Given** installed packages are loaded, **When** the Installed page renders, **Then** packages are grouped into "Updates Available" (yellow header) and "Up to Date" (green header) sections.
2. **Given** an update is available, **When** the user clicks "Update" on a package, **Then** a confirmation dialog shows the package name, current version, and target version. On confirmation, the operations dock shows progress.
3. **Given** multiple updates are available, **When** the user clicks "Update All", **Then** a confirmation dialog lists all packages with their version transitions. On confirmation, the operations dock shows batch progress.
4. **Given** the user clicks "Re-scan", **When** the confirmation dialog is accepted, **Then** the operations dock shows scanning progress.
5. **Given** a package has backup paths, **When** the user clicks "Backup Now" next to it, **Then** a confirmation dialog shows the backup paths and version before starting.

---

### User Story 4 - Dashboard Overview (Priority: P2)

A user opens Astro-Up and sees a dashboard summarizing their system: how many packages are installed, how many updates are available, when the last scan was, and how many backups exist. The updates section shows exactly which packages need updating with version arrows. Quick action buttons let the user scan or update all without navigating to other pages. A recent activity feed shows past operations.

**Why this priority**: The dashboard provides at-a-glance awareness and quick actions. Important but not essential — users can accomplish everything through the Catalog and Installed pages.

**Independent Test**: Can be tested by verifying stats display correctly, updates preview lists the right packages, and quick action buttons trigger the correct flows.

**Acceptance Scenarios**:

1. **Given** the app launches, **When** the Dashboard loads, **Then** four stat cards show: installed count (with total catalog size), updates available count (clickable, navigates to Installed), last scan time, and backup count with total size.
2. **Given** updates are available, **When** the Dashboard renders, **Then** an "Updates Available" section lists each updatable package with icon, name, category, version arrow, and individual Update button.
3. **Given** the user clicks "Scan Installed", **When** the confirmation dialog is accepted, **Then** the operations dock shows scanning progress.
4. **Given** the dashboard is visible, **When** the user looks at the status bar, **Then** it shows catalog package count, last sync time, installed count, update count, operation status, and a log panel toggle.
5. **Given** the dashboard backups stat card is displayed, **When** the user clicks it, **Then** they navigate to the Backup page.

---

### User Story 5 - Backup and Restore (Priority: P2)

A user navigates to the Backup page to restore a previous configuration. They select an application and backup version from dropdowns, preview the restore (seeing exactly which files would be overwritten, which are unchanged, and which are new — with sizes and dates), and confirm the restore. They can also browse all backups grouped by application, preview backup contents, and delete old backups.

**Why this priority**: Backup/restore is a safety net for updates and configuration changes. Critical functionality but secondary to the install/update workflow.

**Independent Test**: Can be tested by selecting a backup, viewing the preview table, confirming restore, and verifying the operations dock shows progress.

**Acceptance Scenarios**:

1. **Given** backups exist, **When** the user selects an application in the Quick Restore dropdown, **Then** the backup dropdown populates with available backups for that application (showing version, date, file count, size).
2. **Given** a backup is selected, **When** the user clicks "Preview & Restore", **Then** a file-level table appears showing each file with its action (Overwrite/Unchanged/New), current file details (size, date), and backup file details (size, date). Summary counts are shown above the table.
3. **Given** the restore preview is visible, **When** the user clicks "Confirm Restore", **Then** a final confirmation dialog states how many files will be overwritten and warns the action cannot be undone. On confirmation, the operations dock shows restore progress.
4. **Given** the All Backups list is displayed, **When** the user selects a filter from the dropdown, **Then** the list filters to show only backups for that application. Backups are grouped by application with headers showing icon, name, and count.
5. **Given** a backup item, **When** the user clicks the preview (eye) icon, **Then** a modal shows backup metadata and a file listing with names, sizes, and dates.
6. **Given** a backup item, **When** the user clicks the delete (trash) icon, **Then** a confirmation dialog warns the deletion is permanent. On confirmation, the backup is removed.

---

### User Story 6 - Configure Application Settings (Priority: P3)

A user opens Settings to configure Astro-Up behavior. Settings are organized in sections: General (update checking), Backup (scheduled backups, retention policies), Catalog (source URL, cache), Network (proxy, timeouts), Paths (download/cache directories), Logging, and About. Changes are saved explicitly. The About section shows app version and lets the user check for app updates.

**Why this priority**: Settings are configure-once, use-forever. Most users will use defaults. Needed for completeness but lowest interaction frequency.

**Independent Test**: Can be tested by navigating through settings sections, modifying values, and saving.

**Acceptance Scenarios**:

1. **Given** the user opens Settings, **When** the page loads, **Then** a sidebar shows all sections (General, Startup & Window, Notifications, Backup, Catalog, Network, Paths, Logging, About) and clicking a section shows its content.
2. **Given** the user is in the Backup section, **When** they configure retention, **Then** they can set max backups per package (including unlimited), max total size, and max age.
3. **Given** the user is in the Startup & Window section, **When** they view the options, **Then** they can toggle start at login, start minimized, and minimize to tray on close.
4. **Given** the user is in the Notifications section, **When** they view the options, **Then** they can enable/disable toasts, set display duration, and choose which notification types to show.
5. **Given** the user clicks "Check for App Updates" in About, **When** the check completes, **Then** the result shows either "up to date" or an available update with install/dismiss options.
6. **Given** the user clicks "Reset to Defaults", **When** the confirmation dialog is accepted, **Then** all settings return to their default values.
7. **Given** the user is in General settings, **When** they change the theme to "Light", **Then** the entire UI immediately switches to light mode.
8. **Given** the user is in General settings, **When** they change font size to "Large", **Then** all UI text scales up immediately.
9. **Given** the user presses Ctrl+F on any page, **When** a search box exists on that page, **Then** it receives focus. The webview find-in-page is not triggered.

---

### User Story 7 - Operations Progress and Cancellation (Priority: P1)

During any long-running operation (install, update, backup, restore, scan), a docked panel appears at the bottom of the window showing the operation label, a progress bar, percentage, and a cancel button. The panel auto-dismisses when the operation completes. Users can continue browsing while operations run.

**Why this priority**: Without progress feedback and cancellation, users have no visibility into or control over running operations. This is foundational UX.

**Independent Test**: Can be tested by triggering any operation and verifying the dock appears, updates progress, and dismisses on completion or cancel.

**Acceptance Scenarios**:

1. **Given** an operation starts (install, update, backup, restore, scan), **When** the operations dock appears, **Then** it shows a spinner, operation label, progress bar, percentage, and cancel button.
2. **Given** an operation is in progress, **When** progress events arrive from the backend, **Then** the progress bar and percentage update in real-time.
3. **Given** an operation is in progress, **When** the user clicks Cancel, **Then** the operation is cancelled via the backend and the dock dismisses.
4. **Given** an operation completes, **When** the completion event arrives, **Then** the progress bar fills to 100%, the label updates to show completion, and the dock auto-dismisses after a brief delay.
5. **Given** the operations dock is visible, **When** the user clicks to expand it, **Then** a detail panel opens showing step-by-step progress for the current operation.
6. **Given** the user clicks the Log toggle in the status bar (or presses Ctrl+L), **When** the log panel opens, **Then** it shows live application logs that can be filtered by log level for the current session without changing the persisted setting.

---

### Edge Cases

- What happens when the catalog is empty or fails to load? Distinct states: empty (0 packages) shows "No packages in catalog." Failed load shows "Failed to load catalog. [Retry]" with error banner. Each page defines its own empty state message with optional action (e.g., "No installed packages. [Scan now?]").
- What happens when a search returns no results? Show a "no packages found" message with a suggestion to clear filters.
- What happens when the user tries to start any operation while another is running? Show a toast: "An operation is already in progress." The new operation is rejected (not queued).
- What happens when the backend returns an error during an operation? Show an error toast with the backend message + inline error banner on the affected page with a retry button. Error logged to error log store. Retry is manual only (no auto-retry, no retry count limit).
- What happens when the window is at minimum size? Layout adapts: grids reduce columns, info panels stack, tables scroll horizontally.
- What happens when a package has no backup paths? The Backup tab and Backup Now button are hidden for that package.
- What happens when no backups exist? The Backup page shows an empty state for the All Backups list and disabled Quick Restore dropdowns.
- What happens when navigating between pages? Search/filter state is preserved within the session, cleared on app restart.
- What happens when the user clicks the nav item for the page they're already on? No-op.
- What happens when config save fails (file locked, permissions)? Error toast with backend message. Settings form stays open with values preserved for retry.
- What happens when the user clicks "Clear Cache" or "Clear Downloads" while an operation is running? Buttons are disabled while an operation is active.
- What happens with very long package names, descriptions, or paths? Names: truncate with ellipsis at container width. Descriptions: 2-line clamp. Paths: horizontal scroll in monospace containers. Tooltips on all truncated text.
- What happens when the log panel receives a high volume of log lines? Buffer limited to 1000 lines in memory. Older lines discarded. User can scroll within buffer. Log file handles persistence.

## Requirements

### Functional Requirements

- **FR-001**: Application MUST display a persistent sidebar with navigation to Dashboard, Catalog, Installed, Backup, and Settings pages. The Installed nav item MUST show a badge with the number of available updates. Sidebar footer stats are removed — status information lives in the status bar.
- **FR-002**: Catalog page MUST display all packages as cards in a responsive grid with package name, publisher, truncated description, category tag (color-coded), and installation status badge.
- **FR-003**: Catalog MUST support text search that matches against package ID, name, description, publisher, category, install method, license, and dependencies — case-insensitive.
- **FR-004**: Catalog MUST support category filtering via chips. Search and category filter MUST combine (intersection).
- **FR-005**: Package detail page MUST show a hero section with contextual action buttons based on package state, and tabbed content (Overview, Versions, Backup if applicable, Technical Details).
- **FR-006**: Installed page MUST group packages into "Updates Available" and "Up to Date" sections with appropriate visual distinction.
- **FR-007**: Installed page MUST support text search filtering across the installed list.
- **FR-008**: All update actions (individual and bulk) MUST show a confirmation dialog listing what will be updated before proceeding.
- **FR-009**: Scan actions MUST show a confirmation dialog before starting.
- **FR-010**: Backup Now action MUST show a confirmation dialog displaying the paths that will be backed up and the current version.
- **FR-011**: Dashboard MUST show stats (installed count, updates available, last scan time, backup count/size — backups stat MUST be clickable and navigate to the Backup page), a list of updatable packages with version transitions, quick action buttons, and a recent activity feed.
- **FR-012**: Backup page MUST provide Quick Restore with application and backup version dropdowns. Selecting an application MUST update the backup dropdown.
- **FR-013**: Restore preview MUST show a file-level table with action (Overwrite/Unchanged/New — color-coded), current file details (size, date), and backup file details (size, date), with summary counts.
- **FR-014**: Restore confirmation MUST warn that the action will overwrite files and cannot be undone.
- **FR-015**: Backup list MUST be grouped by application and filterable by application via a dropdown.
- **FR-016**: Backup preview MUST show backup metadata and a file listing with names, sizes, and dates in a modal.
- **FR-017**: All destructive actions (delete backup, restore, reset settings) MUST show a confirmation dialog before executing.
- **FR-018**: Settings MUST be organized in 9 sections with a sidebar navigation: General (includes theme, font size, install defaults, scan-on-launch, and update settings), Startup & Window, Notifications, Backup, Catalog, Network, Paths, Logging, About.
- **FR-019**: Settings MUST support saving changes explicitly and resetting to defaults (with confirmation).
- **FR-020**: About section MUST show app version, catalog version, database version, license, links, and a "Check for App Updates" button that shows a loading state then result.
- **FR-021**: Operations dock MUST appear during long-running operations showing label, progress bar, percentage, and cancel button. It MUST auto-dismiss on completion.
- **FR-022**: Operations dock MUST update in real-time based on backend progress events received via the Tauri event system.
- **FR-023**: Application MUST enforce a minimum window size that prevents layout breakage (sidebar + content must remain usable).
- **FR-024**: Layout MUST adapt gracefully at small window sizes: grids reduce columns, info panels stack vertically, tables scroll horizontally.
- **FR-025**: Package detail Backup tab MUST show manifest paths as read-only and allow users to add/remove custom backup paths.
- **FR-026**: Package detail Backup tab MUST provide per-package toggles for "Backup before update" and "Include in scheduled backups".
- **FR-027**: All frontend views MUST use the existing Tauri commands defined in spec 016. Where commands are stubs or missing (backup list/preview/delete, detection wiring), the frontend MUST use mock data and be structured so real data can be swapped in without UI changes.
- **FR-028**: Error events from the backend MUST be displayed as toast notifications and logged to the error log store.
- **FR-029**: Operations dock MUST be expandable to show detailed operation steps/log lines.
- **FR-030**: Settings > Startup & Window MUST include: start at login, start minimized, minimize to tray on close.
- **FR-031**: Settings > Notifications MUST include: enable/disable toast notifications, toast display duration, notification types to show (errors, warnings, update available, operation complete).
- **FR-032**: All dropdown/select controls MUST use a consistent styled appearance rather than native browser selects.
- **FR-033**: Application MUST provide a collapsible bottom panel log viewer (not a separate page). The log viewer MUST show live application logs with a session-level log level filter (Error, Warn, Info, Debug, Trace) that does not affect the persisted log level. The panel MUST be resizable and accessible anytime via the status bar, independent of running operations. When the operations dock is active, it appears as a separate bar above the log panel — both are visible simultaneously.
- **FR-034**: Application MUST display a persistent status bar at the bottom showing: catalog package count, last sync time, installed count, update count, running operation count/status, and a toggle to open the log panel. This replaces the sidebar footer stats.
- **FR-035**: Settings > General MUST include: theme switcher (dark/light/system), font size / UI scale adjustment.
- **FR-036**: Settings > General MUST include: default install scope (user/machine), default install method (silent/interactive).
- **FR-037**: Settings > General MUST include: auto-scan on launch toggle (off by default — new software is rarely installed).
- **FR-038**: Settings > Paths MUST include actions to clear cache and clear downloads (with confirmation showing space to be freed).
- **FR-039**: Settings > General MUST include update settings: auto-check for catalog updates, check interval, auto-notify for package updates, auto-install package updates toggle.
- **FR-040**: Settings > About MUST include config backup/restore: a snapshot is automatically saved every time the user explicitly saves settings. The last N snapshots are retained (configurable). Users can view and restore any previous config snapshot.
- **FR-041**: Application MUST support keyboard shortcuts for common actions: Ctrl+F (focus search, overrides webview find-in-page), Escape (close dialog / collapse log panel), Ctrl+, (open settings), Ctrl+1-5 (navigate pages), Ctrl+L (toggle log panel), Ctrl+J (toggle operations dock). All shortcuts use Ctrl on Windows. Webview default shortcuts (zoom, copy/paste) are preserved.
- **FR-044**: All data-dependent pages MUST show loading skeletons while data is being fetched. Dashboard MUST load progressively (stats appear as each query returns).
- **FR-045**: All pages MUST define empty states (no data) and error states (load failure with retry button) as distinct UI states.
- **FR-046**: Toast notifications MUST stack vertically from bottom-right, max 3 visible. Oldest auto-dismissed when limit exceeded.
- **FR-047**: Tab keyboard navigation MUST follow WAI-ARIA tablist pattern (arrow keys to switch, Enter to activate).
- **FR-048**: Focus order MUST follow: sidebar, main content, operations dock, log panel. Focus returns to main content when a panel collapses.
- **FR-049**: Log panel MUST be resizable via drag handle on top edge. Min height: 100px. Max height: 60% of viewport.
- **FR-050**: Operations dock expand/collapse MUST toggle on click anywhere on the bar. Default: collapsed (slim bar). Expanded: shows step-by-step log. Auto-dismisses 3 seconds after operation completes.
- **FR-051**: Confirmation dialogs MUST dismiss on Escape key, overlay click, or Cancel button.
- **FR-052**: Only one operation may run at a time. Starting a new operation while one is active MUST show a toast: "An operation is already in progress." The new operation is rejected, not queued.
- **FR-042**: Settings forms MUST validate all inputs before saving: URLs must be valid format, durations must be positive number + unit, paths must contain valid Windows path characters with no path traversal (`..`), numeric values must be within field-specific ranges. Dropdowns are constrained input and need no validation. Validation triggers on blur (immediate feedback) and on submit (catch-all). Invalid fields show an inline error message below the field in red and prevent save.
- **FR-043**: The frontend MUST validate the complete configuration object against expected types and constraints (mirroring the Rust `AppConfig` model) before calling `save_config`, rejecting malformed data before it reaches the backend. If `save_config` fails despite frontend validation, an error toast with the backend message MUST be shown and the form MUST remain open with values preserved for retry.
- **FR-053**: Text search results MUST be relevance-ranked: exact ID match first, then name match, then description/metadata match. Browse mode (no search active) MUST display packages alphabetically by name.
- **FR-054**: When an operation completes, the frontend MUST refresh data relevant to that operation (e.g., scan complete refreshes installed list, backup complete refreshes backup list). Data shapes from Tauri commands MUST be defined as TypeScript types mirroring the backend models.
- **FR-055**: Config snapshots MUST store the full config object serialized as JSON, including a timestamp and config version. Custom backup path validation on the frontend is limited to format checks (non-empty, valid Windows path characters, no `..`); full path resolution and existence checks are deferred to the backend (#507).

### Key Entities

- **Package**: Software available in the catalog — has ID, name, publisher, description, category, version info, installation status, detection/install config, backup paths.
- **Backup**: Archived configuration for a package — has package ID, version, creation date, file count, total size, individual file entries.
- **Operation**: A running long-term action (install, update, backup, restore, scan) — has ID, label, progress percentage, cancellation capability.
- **AppConfig**: Application settings organized in 9 sections (general, startup & window, notifications, backup, catalog, network, paths, logging, about — including config snapshots).

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can locate a specific package via search or filter within 3 interactions (type query, optionally select category, find result).
- **SC-002**: Users can navigate from the catalog to a package detail page and back in under 2 seconds.
- **SC-003**: All destructive actions (update, restore, delete backup, reset settings) require explicit user confirmation before executing.
- **SC-004**: Users can see real-time progress for all long-running operations and cancel them at any time.
- **SC-005**: Users can identify all available updates from the dashboard without navigating to another page.
- **SC-006**: Users can preview exactly which files a restore operation will affect (overwrite, add, leave unchanged) before confirming.
- **SC-007**: The application remains usable at minimum window size with no overlapping or clipped content.
- **SC-008**: Pages with cached/local data load within 500ms. Pages requiring Tauri command round-trips load within 1 second.

## Clarifications

### Session 2026-04-03

- Q: How do the log panel and operations dock coexist when both are visible? → A: Operations dock is a separate bar above the log panel — both visible simultaneously. Expanding the operations dock pushes the log panel down.
- Q: What triggers a config snapshot? → A: Snapshot on every explicit "Save Changes" in Settings.
- Q: Should we consolidate the 10 settings sections? → A: Merge Updates into General (9 sections total).
- Q: How do update counts stay consistent across status bar, sidebar badge, and dashboard? → A: Single shared data source — computed once, displayed in all three locations.

## Assumptions

- The Tauri commands from spec 016 are available and functional. Commands are categorized by data readiness:
  - **Real data**: `list_software`, `search_catalog`, `get_config`, `save_config`, `get_version`
  - **Stubs (return empty/mock)**: `scan_installed`, `check_for_updates`, `install_software`, `update_software`. Frontend shows empty states for these.
  - **Missing (frontend-only mock)**: `list_backups`, `get_backup_contents`, `preview_restore`, `delete_backup`, `get_config_snapshots`, `restore_config_snapshot` — deferred to #508
- Mock data MUST live in a separate `mocks/` module, not hardcoded in components. Each Tauri command is wrapped in a service layer composable so mock implementations can be swapped for real ones via a single module replacement.
- Mock data shapes MUST match the expected real API types exactly (defined as TypeScript interfaces).
- Stub command behavior: return empty arrays (no packages found, no updates). No error simulation by default.
- Mock backup behavior: delete removes from local mock array (session only), preview shows mock file list, no actual files touched.
- Config snapshots are saved to local storage on each "Save Changes" for MVP. Backend command (#508) replaces this later.
- The PrimeVue Aura dark theme and existing frontend dependencies (Vue 3, VueQuery, PrimeVue 4) from spec 016 are used.
- Backend operations emit progress events via the Tauri event system using the CoreEvent types defined in spec 016.
- Backup policies backend (auto-backup scheduling, retention enforcement, custom path validation) is deferred to #507 — the frontend shows the UI controls but they connect to configuration only, not automated behavior.
- The catalog database is pre-populated — the frontend does not handle initial catalog setup or downloads.
- Only one operation runs at a time — concurrent operation handling is out of scope (FR-052).
- The "recent activity" feed on the dashboard shows hardcoded mock data for now — a persistent activity log is a future feature.
