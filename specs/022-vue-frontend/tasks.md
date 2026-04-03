# Tasks: Vue Frontend Views

**Input**: Design documents from `/specs/022-vue-frontend/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1-US7)

## Phase 1: Setup

**Purpose**: Install dependencies, create project structure, configure routing

- [ ] T001 Install new frontend dependencies: `pnpm --dir frontend add vue-router@^4 @vueuse/core@^14 valibot@^1` in `frontend/package.json`
- [ ] T002 Create vue-router setup with hash mode and route definitions for all 5 pages in `frontend/src/router/index.ts`
- [ ] T003 Refactor `frontend/src/App.vue` to use router-view layout: sidebar + main (router-view) + status bar + log panel + ops dock
- [ ] T004 [P] Create TypeScript types mirroring Rust models in `frontend/src/types/package.ts`, `frontend/src/types/backup.ts`, `frontend/src/types/config.ts`, `frontend/src/types/operations.ts`
- [ ] T005 [P] Create mock data module with backup list, activity feed, and backup contents in `frontend/src/mocks/backups.ts`, `frontend/src/mocks/activity.ts`, `frontend/src/mocks/index.ts`
- [ ] T006 Update `frontend/src/main.ts` to register vue-router plugin
- [ ] T007 Update `frontend/src/App.test.ts` to provide router in test mounts

---

## Phase 2: Foundational (Layout Shell + Shared Components)

**Purpose**: Layout components and shared infrastructure that all views depend on

- [ ] T008 Create AppSidebar component with navigation items, active state from router, and update badge in `frontend/src/components/layout/AppSidebar.vue`
- [ ] T009 Create AppStatusBar component showing catalog count, sync time, installed count, update count, ops count, and log toggle in `frontend/src/components/layout/AppStatusBar.vue`
- [ ] T010 Create OperationsDock component with progress bar, expand/collapse, step log, cancel, 3s auto-dismiss in `frontend/src/components/layout/OperationsDock.vue`
- [ ] T011 Create LogPanel component with collapsible bottom panel, resizable (100px-60% viewport), log level filter dropdown, 1000-line buffer, monospace output in `frontend/src/components/layout/LogPanel.vue`
- [ ] T012 [P] Create ConfirmDialog component wrapping PrimeVue Dialog with title, icon, message, detail slot, confirm/cancel buttons, Escape/overlay dismiss in `frontend/src/components/shared/ConfirmDialog.vue`
- [ ] T013 [P] Create EmptyState component with icon, message, and optional action button in `frontend/src/components/shared/EmptyState.vue`
- [ ] T014 [P] Create FileTable component for backup file listings (action mode with overwrite/unchanged/new coloring, and preview mode with size/date) in `frontend/src/components/shared/FileTable.vue`
- [ ] T015 Create useOperations composable managing active operation state, single-op guard (FR-052), progress event listener, cancel support in `frontend/src/composables/useOperations.ts`
- [ ] T016 Create useKeyboard composable with Ctrl+F (search focus), Escape (close/collapse), Ctrl+1-5 (navigate), Ctrl+L (log), Ctrl+J (ops dock), Ctrl+, (settings) in `frontend/src/composables/useKeyboard.ts`
- [ ] T017 Extend existing useTheme composable to support dark/light/system preference from config in `frontend/src/composables/useTheme.ts`
- [ ] T018 Create useSearch composable with combined text search + category filter logic, relevance ranking (FR-053) in `frontend/src/composables/useSearch.ts`

**Checkpoint**: Layout shell functional — sidebar navigates between empty view stubs, status bar shows data, log panel toggles, keyboard shortcuts work

---

## Phase 3: User Story 1 — Browse and Search Catalog (Priority: P1)

**Goal**: Users can browse, search, and filter the software catalog

**Independent Test**: Load catalog page, type "nina", see N.I.N.A. appear. Click category chip, see filtered results. Click card, navigate to detail.

- [ ] T019 [US1] Create CategoryChips component with color-coded chips per category, active state, click handler in `frontend/src/components/catalog/CategoryChips.vue`
- [ ] T020 [US1] Create PackageCard component showing icon, name, publisher, 2-line description, category tag, status badge (installed/update/none) in `frontend/src/components/catalog/PackageCard.vue`
- [ ] T021 [US1] Create PackageGrid component with responsive auto-fill grid (minmax 280px) and loading skeleton state in `frontend/src/components/catalog/PackageGrid.vue`
- [ ] T022 [US1] Create CatalogView with search box, category chips, package grid, combined search+filter via useSearch composable, empty state for no results in `frontend/src/views/CatalogView.vue`
- [ ] T023 [US1] Wire CatalogView to `useSoftwareList` and `useCatalogSearch` from useInvoke.ts, add loading skeleton and error state with retry

**Checkpoint**: Catalog page fully functional — search, filter, cards with status, click navigates to detail

---

## Phase 4: User Story 2 — View Package Details (Priority: P1)

**Goal**: Users see full package info with tabbed content and contextual actions

**Independent Test**: Navigate to a package, see hero with correct action buttons, switch tabs, view backup config for packages with backup paths.

- [ ] T024 [US2] Create DetailHero component with icon, name, publisher, homepage link, description, contextual action buttons (Update/Install/Installed/Backup Now/Homepage) in `frontend/src/components/detail/DetailHero.vue`
- [ ] T025 [P] [US2] Create OverviewTab component with info grid (version, category, method, license, dependencies, detection, backup paths) in `frontend/src/components/detail/OverviewTab.vue`
- [ ] T026 [P] [US2] Create VersionsTab component with PrimeVue DataTable (version, discovered, pre-release, action button per row) in `frontend/src/components/detail/VersionsTab.vue`
- [ ] T027 [P] [US2] Create TechnicalTab component with detection section (method, details, last result) and installation section (method, scope, elevation, upgrade) in `frontend/src/components/detail/TechnicalTab.vue`
- [ ] T028 [US2] Create BackupTab component with manifest paths (read-only, MANIFEST badge), custom paths (add/remove), auto-backup toggles, backup history list with preview/delete in `frontend/src/components/detail/BackupTab.vue`
- [ ] T029 [US2] Create PackageDetailView with route param `:id`, hero, PrimeVue TabView (Overview/Versions/Backup/Technical), breadcrumb back navigation in `frontend/src/views/PackageDetailView.vue`
- [ ] T030 [US2] Wire Backup Now button (hero + installed page) to ConfirmDialog showing paths + version, then trigger create_backup via useCreateBackup

**Checkpoint**: Detail page fully functional — all tabs render, actions work, backup config editable

---

## Phase 5: User Story 3 — Manage Installed Software (Priority: P1)

**Goal**: Users see installed packages grouped by update status, can update individually or in bulk, scan, and backup

**Independent Test**: Open Installed page, see grouped list, click Update on a package (confirmation -> progress), click Update All, click Re-scan.

- [ ] T031 [US3] Create PackageRow component with icon, name, category, version display (arrow for updates / checkmark for current), action buttons (Backup Now, Update) in `frontend/src/components/installed/PackageRow.vue`
- [ ] T032 [US3] Create InstalledView with search filter, Update All button (confirmation listing all packages), Re-scan button (confirmation), grouped sections (Updates Available / Up to Date), PackageRow list in `frontend/src/views/InstalledView.vue`
- [ ] T033 [US3] Wire Update individual package: ConfirmDialog ("Update X from A to B?") -> useUpdateSoftware mutation -> ops dock progress -> cache invalidation
- [ ] T034 [US3] Wire Update All: ConfirmDialog listing all updatable packages with versions -> sequential update mutations -> ops dock batch progress
- [ ] T035 [US3] Wire Re-scan: ConfirmDialog -> useScanInstalled mutation -> ops dock progress -> cache invalidation of software + updates queries

**Checkpoint**: Installed page fully functional — grouped list, individual/bulk update, scan, backup per package

---

## Phase 6: User Story 7 — Operations Progress (Priority: P1)

**Goal**: Operations dock shows real-time progress, expandable detail, cancellable

**Independent Test**: Trigger any operation, see dock appear with progress, expand for detail, cancel, see auto-dismiss on complete.

- [ ] T036 [US7] Wire OperationsDock to useCoreEvents — map CoreEvent types to progress updates (download_progress, scan_progress, backup_progress, install_started/complete/failed). Also wire error events to toast notifications + errorLog store (FR-028) in `frontend/src/components/layout/OperationsDock.vue`
- [ ] T037 [US7] Wire LogPanel to listen for `core-event` and `log-event` Tauri channels, format entries as LogEntry, manage 1000-line buffer with auto-scroll in `frontend/src/components/layout/LogPanel.vue`
- [ ] T038 [US7] Integrate OperationsDock + LogPanel layering in App.vue — ops dock above log panel, both visible simultaneously, status bar at bottom

**Checkpoint**: Operations dock + log panel fully functional — real-time progress, expand/collapse, log viewer with level filter

---

## Phase 7: User Story 4 — Dashboard Overview (Priority: P2)

**Goal**: At-a-glance system summary with stats, update preview, quick actions, activity feed

**Independent Test**: Open dashboard, see 4 stat cards with correct counts, updates preview listing packages, quick action buttons trigger correct flows.

- [ ] T039 [US4] Create DashboardView with stat cards (installed, updates [clickable->Installed], last scan, backups [clickable->Backup]), updates preview list, quick actions (Scan, Update All), recent activity feed in `frontend/src/views/DashboardView.vue`
- [ ] T040 [US4] Wire dashboard stats to VueQuery queries (useSoftwareList, useUpdateCheck), wire quick action buttons to same confirmation flows as Installed page
- [ ] T041 [US4] Wire activity feed to mock data from `frontend/src/mocks/activity.ts`

**Checkpoint**: Dashboard fully functional — stats, update preview, quick actions, activity feed

---

## Phase 8: User Story 5 — Backup and Restore (Priority: P2)

**Goal**: Users can restore backups with file-level preview, browse/filter/preview/delete backups

**Independent Test**: Select app + backup, click Preview & Restore, see file table, confirm restore -> progress. Browse backups, filter by app, preview contents, delete with confirmation.

- [ ] T042 [US5] Create QuickRestore component with app dropdown, backup dropdown (updates on app change), Preview & Restore button in `frontend/src/components/backup/QuickRestore.vue`
- [ ] T043 [US5] Create RestorePreview component with summary counts (overwrite/new/unchanged), FileTable in action mode, Confirm Restore button (-> final confirmation "overwrite N files") in `frontend/src/components/backup/RestorePreview.vue`
- [ ] T044 [US5] Create BackupGroup component with app header (icon, name, count), backup items (version, files, size, date), preview (eye) + delete (trash) buttons in `frontend/src/components/backup/BackupGroup.vue`
- [ ] T045 [US5] Create BackupView with Quick Restore section, All Backups section with app filter dropdown, grouped backup list, empty state in `frontend/src/views/BackupView.vue`
- [ ] T046 [US5] Wire backup actions: preview contents (InfoDialog with FileTable), delete (ConfirmDialog "cannot be undone"), restore confirmation chain (preview -> final confirm -> ops dock)
- [ ] T047 [US5] Add mock data queries in useInvoke.ts: useBackupList, useBackupContents, useBackupPreview returning mock data from mocks/ module

**Checkpoint**: Backup page fully functional — quick restore with file preview, backup list with filter/group/preview/delete

---

## Phase 9: User Story 6 — Settings (Priority: P3)

**Goal**: Users can configure all app settings organized in 9 sections with validation

**Independent Test**: Open settings, navigate sections, modify values, save (with validation), reset to defaults, check for app updates, view/restore config snapshots.

- [ ] T048 [US6] Create valibot validation schemas for all config sections in `frontend/src/validation/config.ts`
- [ ] T049 [P] [US6] Create GeneralSection component with theme switcher, font size, auto-scan toggle, install scope/method dropdowns, update settings (auto-check, interval, auto-notify, auto-install) in `frontend/src/components/settings/GeneralSection.vue`
- [ ] T050 [P] [US6] Create StartupSection component with start-at-login, start-minimized, minimize-to-tray toggles in `frontend/src/components/settings/StartupSection.vue`
- [ ] T051 [P] [US6] Create NotificationsSection component with enable toggle, duration dropdown, per-type toggles (errors, warnings, update, complete) in `frontend/src/components/settings/NotificationsSection.vue`
- [ ] T052 [P] [US6] Create BackupSection component with scheduled backup toggle + schedule dropdown, retention settings (max per package, max size, max age) in `frontend/src/components/settings/BackupSection.vue`
- [ ] T053a [P] [US6] Create CatalogSection component with catalog URL and cache TTL in `frontend/src/components/settings/CatalogSection.vue`
- [ ] T053b [P] [US6] Create NetworkSection component with proxy, connection/request timeout, download speed limit in `frontend/src/components/settings/NetworkSection.vue`
- [ ] T053c [P] [US6] Create PathsSection component with download/cache dirs (browse button), keep installers toggle, purge after days, clear cache/downloads buttons (confirmation + size display) in `frontend/src/components/settings/PathsSection.vue`
- [ ] T054 [P] [US6] Create LoggingSection component with log level dropdown, log-to-file toggle, log file path in `frontend/src/components/settings/LoggingSection.vue`
- [ ] T055 [P] [US6] Create AboutSection component with version info, links, Check for App Updates button (spinner -> result dialog), config snapshot list with restore, and max snapshot count setting in `frontend/src/components/settings/AboutSection.vue`
- [ ] T056 [US6] Create configSnapshots store using @vueuse/useStorage for localStorage-based snapshots (save on every "Save Changes", keep last N) in `frontend/src/stores/configSnapshots.ts`
- [ ] T057 [US6] Create SettingsView with sidebar nav (9 sections), section content panels, Save Changes + Reset to Defaults buttons, validation on blur + submit, inline errors in `frontend/src/views/SettingsView.vue`
- [ ] T058 [US6] Wire settings: get_config on mount, save_config with valibot validation (FR-042/043), error toast on backend failure, config snapshot on save, theme/font apply immediately

**Checkpoint**: Settings fully functional — all 9 sections, validation, save/reset, config snapshots, app update check

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Consistent styling, responsive layout, accessibility, final integration

- [ ] T059 [P] Replace all native `<select>` elements with PrimeVue Dropdown components for consistent styling (FR-032) across all views
- [ ] T060 [P] Add responsive CSS: min window size enforcement, grid column collapse, info grid stacking, table horizontal scroll (FR-023, FR-024)
- [ ] T061 [P] Add truncation handling: package name ellipsis, description 2-line clamp, path horizontal scroll, tooltips on truncated text (edge case)
- [ ] T062 Add toast notification stacking (max 3, bottom-right, oldest auto-dismissed) via PrimeVue Toast config (FR-046)
- [ ] T063 Add WAI-ARIA tablist keyboard navigation for detail page tabs (FR-047)
- [ ] T064 Wire focus management: tab order sidebar -> main -> ops dock -> log panel, focus return on panel collapse (FR-048)
- [ ] T065 Add font size / UI scale CSS custom property that applies to all components (FR-035)
- [ ] T066 Run `pnpm --dir frontend lint` and `pnpm --dir frontend test` — fix all errors
- [ ] T067 Update `frontend/src/App.test.ts` and add view-level smoke tests for each page
- [ ] T068 Run quickstart.md validation: verify `just dev` launches with all views functional

---

## Task Dependencies

```toml
[graph]
T001 = { blocked_by = [] }
T002 = { blocked_by = ["T001"] }
T003 = { blocked_by = ["T002"] }
T004 = { blocked_by = [] }
T005 = { blocked_by = ["T004"] }
T006 = { blocked_by = ["T002"] }
T007 = { blocked_by = ["T003", "T006"] }

# Phase 2: Foundational
T008 = { blocked_by = ["T003", "T006"] }
T009 = { blocked_by = ["T003"] }
T010 = { blocked_by = ["T003", "T015"] }
T011 = { blocked_by = ["T003"] }
T012 = { blocked_by = ["T003"] }
T013 = { blocked_by = ["T003"] }
T014 = { blocked_by = ["T004"] }
T015 = { blocked_by = ["T004"] }
T016 = { blocked_by = ["T003", "T008"] }
T017 = { blocked_by = ["T003"] }
T018 = { blocked_by = ["T004"] }

# Phase 3: US1 — Catalog
T019 = { blocked_by = ["T018"] }
T020 = { blocked_by = ["T004", "T013"] }
T021 = { blocked_by = ["T020"] }
T022 = { blocked_by = ["T019", "T021", "T018"] }
T023 = { blocked_by = ["T022"] }

# Phase 4: US2 — Detail
T024 = { blocked_by = ["T012", "T015"] }
T025 = { blocked_by = ["T004"] }
T026 = { blocked_by = ["T004"] }
T027 = { blocked_by = ["T004"] }
T028 = { blocked_by = ["T004", "T014", "T005"] }
T029 = { blocked_by = ["T024", "T025", "T026", "T027", "T028"] }
T030 = { blocked_by = ["T012", "T015", "T024"] }

# Phase 5: US3 — Installed
T031 = { blocked_by = ["T004", "T012", "T015"] }
T032 = { blocked_by = ["T031", "T018"] }
T033 = { blocked_by = ["T032", "T012", "T015"] }
T034 = { blocked_by = ["T033"] }
T035 = { blocked_by = ["T032", "T015"] }

# Phase 6: US7 — Operations
T036 = { blocked_by = ["T010", "T015"] }
T037 = { blocked_by = ["T011"] }
T038 = { blocked_by = ["T036", "T037", "T009"] }

# Phase 7: US4 — Dashboard
T039 = { blocked_by = ["T008", "T009", "T013"] }
T040 = { blocked_by = ["T039", "T012", "T015"] }
T041 = { blocked_by = ["T039", "T005"] }

# Phase 8: US5 — Backup
T042 = { blocked_by = ["T005", "T014"] }
T043 = { blocked_by = ["T014", "T012"] }
T044 = { blocked_by = ["T005", "T014"] }
T045 = { blocked_by = ["T042", "T043", "T044"] }
T046 = { blocked_by = ["T045", "T012"] }
T047 = { blocked_by = ["T005"] }

# Phase 9: US6 — Settings
T048 = { blocked_by = ["T004"] }
T049 = { blocked_by = ["T048", "T017"] }
T050 = { blocked_by = ["T048"] }
T051 = { blocked_by = ["T048"] }
T052 = { blocked_by = ["T048"] }
T053a = { blocked_by = ["T048"] }
T053b = { blocked_by = ["T048"] }
T053c = { blocked_by = ["T048", "T012"] }
T054 = { blocked_by = ["T048"] }
T055 = { blocked_by = ["T048", "T056"] }
T056 = { blocked_by = ["T001"] }
T057 = { blocked_by = ["T049", "T050", "T051", "T052", "T053a", "T053b", "T053c", "T054", "T055"] }
T058 = { blocked_by = ["T057", "T015"] }

# Phase 10: Polish
T059 = { blocked_by = ["T023", "T029", "T032", "T045", "T057"] }
T060 = { blocked_by = ["T023", "T029", "T032", "T045", "T057"] }
T061 = { blocked_by = ["T023", "T029", "T032"] }
T062 = { blocked_by = ["T003"] }
T063 = { blocked_by = ["T029"] }
T064 = { blocked_by = ["T008", "T010", "T011"] }
T065 = { blocked_by = ["T049"] }
T066 = { blocked_by = ["T059", "T060", "T061", "T062", "T063", "T064", "T065"] }
T067 = { blocked_by = ["T066"] }
T068 = { blocked_by = ["T067"] }
```

## Parallel Opportunities

**Phase 1**: T004 + T005 can run parallel to T001-T003 (types/mocks don't need router)
**Phase 2**: T012 + T013 + T014 parallel (shared components in different files). T015 + T016 + T017 + T018 parallel (composables in different files)
**Phase 4**: T025 + T026 + T027 parallel (tab components are independent)
**Phase 9**: T049-T055 all parallel (settings section components are independent)
**Phase 10**: T059 + T060 + T061 parallel (different CSS/component concerns)

## Implementation Strategy

### MVP First (Phases 1-3)
1. Setup + Foundational -> layout shell working
2. Catalog page -> can browse and search packages
3. **STOP and VALIDATE**: Catalog loads real data from backend

### Core Flow (add Phases 4-6)
4. Detail page -> can view package info and take actions
5. Installed page -> can manage updates
6. Operations dock -> can see progress

### Full Feature (add Phases 7-9)
7. Dashboard -> at-a-glance overview
8. Backup page -> restore with preview
9. Settings -> full configuration

### Polish (Phase 10)
10. Consistent styling, responsive, accessibility, tests
