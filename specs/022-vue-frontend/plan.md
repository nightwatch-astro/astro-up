# Implementation Plan: Vue Frontend Views

**Branch**: `022-vue-frontend` | **Date**: 2026-04-03 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/022-vue-frontend/spec.md`
**Design Reference**: `research/017-vue-frontend-design.md`, `research/017-vue-frontend-mockup.html`

## Summary

Build the complete Vue 3 frontend for Astro-Up: 5 page views (Dashboard, Catalog, Installed, Package Detail, Backup, Settings), a persistent sidebar, status bar, collapsible log panel, operations dock, confirmation dialogs, and keyboard shortcuts. All views wire to existing Tauri commands via the `useInvoke.ts` service layer, using mock data where backend commands are stubs or missing. PrimeVue Aura dark theme with vue-router for navigation.

## Technical Context

**Language/Version**: TypeScript 5, Vue 3 (Composition API + `<script setup>`)
**Primary Dependencies**: PrimeVue 4 (Aura), @tanstack/vue-query 5, vue-router 4, @vueuse/core 14, valibot (validation)
**Storage**: Tauri commands -> SQLite (via astro-up-core); localStorage for config snapshots
**Testing**: Vitest 3, @vue/test-utils 2, jsdom
**Target Platform**: Windows (Tauri v2 / WebView2), dev on macOS
**Project Type**: Desktop app (Tauri) frontend
**Performance Goals**: 500ms page load (cached), 1s with Tauri round-trip (SC-008)
**Constraints**: Min window 800x600, single operation at a time, PrimeVue component library
**Scale/Scope**: ~42 packages in catalog, 6 views, 9 settings sections, 55 FRs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First Crate Layout | N/A | Frontend spec — no Rust crate changes |
| II. Platform Awareness | PASS | Windows primary, dev on macOS. Tauri handles platform abstraction |
| III. Test-First with Integration Tests | PASS | Vitest + vue-test-utils for component tests. Integration tests via Tauri command mocking |
| IV. Thin Tauri Boundary | PASS | All business logic in astro-up-core. Frontend is a thin adapter via useInvoke.ts |
| V. Spec-Driven Development | PASS | Full spec with 55 FRs, design doc, mockup |
| VI. Simplicity | PASS | Using existing dependencies (PrimeVue, VueQuery). Adding only vue-router, vueuse, valibot — all standard Vue ecosystem |

No violations.

## Project Structure

### Documentation (this feature)

```text
specs/022-vue-frontend/
├── plan.md              # This file
├── research.md          # Phase 0: library research
├── data-model.md        # Phase 1: TypeScript types and state model
├── quickstart.md        # Phase 1: dev setup guide
└── tasks.md             # Phase 2: implementation tasks
```

### Source Code (repository root)

```text
frontend/src/
├── main.ts                      # App entry (existing, add router)
├── App.vue                      # Layout shell (existing, replace with router-view)
├── App.test.ts                  # (existing)
├── styles.css                   # Global styles (existing)
├── router/
│   └── index.ts                 # Vue Router setup (hash mode for Tauri)
├── views/
│   ├── DashboardView.vue        # FR-011: stats, updates preview, activity
│   ├── CatalogView.vue          # FR-002-004: search, filter, grid
│   ├── InstalledView.vue        # FR-006-007: grouped list, actions
│   ├── PackageDetailView.vue    # FR-005: hero, tabs (overview, versions, backup, technical)
│   ├── BackupView.vue           # FR-012-016: quick restore, backup list
│   └── SettingsView.vue         # FR-018-020, FR-030-040: 9 sections
├── components/
│   ├── layout/
│   │   ├── AppSidebar.vue       # FR-001: nav, badge
│   │   ├── AppStatusBar.vue     # FR-034: catalog info, ops count, log toggle
│   │   ├── OperationsDock.vue   # FR-021-022, FR-029, FR-050: progress, expand
│   │   └── LogPanel.vue         # FR-033, FR-049: resizable log viewer
│   ├── catalog/
│   │   ├── PackageCard.vue      # FR-002: card with status badge
│   │   ├── PackageGrid.vue      # Grid container with responsive columns
│   │   └── CategoryChips.vue    # FR-004: filter chips
│   ├── installed/
│   │   └── PackageRow.vue       # FR-006: row with actions
│   ├── detail/
│   │   ├── DetailHero.vue       # FR-005: icon, name, actions
│   │   ├── OverviewTab.vue      # Info grid
│   │   ├── VersionsTab.vue      # Version table
│   │   ├── BackupTab.vue        # FR-025-026: paths, toggles, history
│   │   └── TechnicalTab.vue     # Detection + install config
│   ├── backup/
│   │   ├── QuickRestore.vue     # FR-012: dropdowns, preview
│   │   ├── RestorePreview.vue   # FR-013: file table with actions
│   │   └── BackupGroup.vue      # FR-015: grouped backup list
│   ├── settings/
│   │   ├── GeneralSection.vue   # FR-035-037, FR-039: theme, font, install, updates
│   │   ├── StartupSection.vue   # FR-030: start at login, minimize
│   │   ├── NotificationsSection.vue # FR-031: toast config
│   │   ├── BackupSection.vue    # Scheduled backup, retention
│   │   ├── CatalogSection.vue   # URL, cache TTL
│   │   ├── NetworkSection.vue   # Proxy, timeouts
│   │   ├── PathsSection.vue     # FR-038: dirs, clear cache/downloads
│   │   ├── LoggingSection.vue   # Level, file
│   │   └── AboutSection.vue     # FR-020, FR-040: version, config snapshots
│   └── shared/
│       ├── ConfirmDialog.vue    # FR-017, FR-051: generic confirmation
│       ├── FileTable.vue        # FR-013, FR-016: backup file listing
│       └── EmptyState.vue       # FR-045: reusable empty/error state
├── composables/
│   ├── useInvoke.ts             # (existing) Tauri command wrappers
│   ├── useCoreEvents.ts         # (existing) Event listener
│   ├── useTheme.ts              # (existing, extend for FR-035)
│   ├── useErrorLog.ts           # (existing) -> stores/errorLog.ts
│   ├── useKeyboard.ts           # FR-041: keyboard shortcuts via @vueuse/core 14
│   ├── useOperations.ts         # FR-021, FR-052: operation state, single-op guard
│   └── useSearch.ts             # FR-003-004: combined search + filter logic
├── stores/
│   ├── errorLog.ts              # (existing)
│   └── configSnapshots.ts       # FR-040: localStorage-based config snapshots
├── mocks/
│   ├── backups.ts               # Mock backup list, contents, preview data
│   ├── activity.ts              # Mock recent activity
│   └── index.ts                 # Mock data registry
├── types/
│   ├── commands.ts              # (existing) CoreEvent types
│   ├── package.ts               # Package, PackageSummary, VersionEntry
│   ├── backup.ts                # BackupListEntry, FileChangeSummary, BackupContents
│   ├── config.ts                # AppConfig sections, validation schemas
│   └── operations.ts            # Operation state types
└── validation/
    └── config.ts                # FR-042-043: valibot schemas for config validation
```

**Structure Decision**: Extends existing `frontend/src/` with views, components (grouped by feature), composables, mocks, and validation directories. No new crates. No monorepo changes.

## Technology Decisions

### vue-router (NEW — not yet installed)

Hash mode (`createWebHashHistory`) for Tauri — no server for HTML5 history mode. Routes:
- `/` — Dashboard
- `/catalog` — Catalog
- `/catalog/:id` — Package Detail
- `/installed` — Installed
- `/backup` — Backup & Restore
- `/settings` — Settings (with query param for section)

### valibot (NEW — not yet installed)

Chosen over Zod for config validation (FR-042/043). Reasons:
- ~10x smaller bundle (tree-shakeable, ~1KB vs ~13KB)
- Same TypeScript inference quality
- Functional API matches Vue composition style
- Desktop app doesn't need Zod's ecosystem (no tRPC, no Next.js)

### @vueuse/core 14 (NEW — not yet installed)

For keyboard shortcuts (`useMagicKeys`), localStorage (`useStorage`), and resize observer (`useResizeObserver`). Standard Vue ecosystem utility — 200+ composables, tree-shakeable.

### Existing dependencies (no changes)

- **PrimeVue 4**: DataTable, Toast, Dialog, Dropdown, InputText, Checkbox, Slider, TabView, ProgressBar, Skeleton, ScrollPanel
- **@tanstack/vue-query 5**: useQuery, useMutation, queryClient.invalidateQueries — already wired in useInvoke.ts
- **@tauri-apps/api**: invoke, listen — already wired
- **@tauri-apps/plugin-updater**: check() for app updates — already installed
- **@tauri-apps/plugin-autostart**: for start-at-login setting — already installed
- **@tauri-apps/plugin-window-state**: for window position persistence — already installed

## Complexity Tracking

No constitution violations. No complexity justifications needed.
