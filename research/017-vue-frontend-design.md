# 017 Vue Frontend — Design Document

Reference mockup: `research/017-vue-frontend-mockup.html`

## Window

- Default size: 1024x768
- Minimum size: 800x600 (enforced by Tauri `minWidth`/`minHeight`)
- Window state (position, size) persisted by `tauri-plugin-window-state`

## Layout

Sidebar (220px fixed) + main content area. Sidebar contains:
- App title + version
- Navigation: Dashboard, Catalog, Installed, Backup, Settings
- Badge on Installed showing update count
- Footer: catalog package count + last sync time

Bottom-docked operations panel appears during long-running operations (install, update, backup, restore, scan). Shows spinner, label, progress bar, percentage, cancel button. Auto-dismisses on completion.

## Responsive Behavior

At minimum width (800px), the main content area is ~580px (800 - 220 sidebar). Design targets:
- Catalog grid: `auto-fill, minmax(280px, 1fr)` — collapses to 2 columns at min width, 1 column not expected
- Info grids (detail page): 2 columns, collapse to 1 at narrow widths via `@media (max-width: 900px)`
- Stats grid (dashboard): 4 columns, collapse to 2 at narrow widths
- Settings layout: sidebar nav (180px) + panel — at min width the panel is ~380px, sufficient for all form fields
- Tables (version history, file preview): horizontal scroll via `overflow-x: auto` on container
- Sidebar: fixed 220px, never collapses (min window width guarantees sufficient space)

## Pages

### Dashboard

| Element | Behavior |
|---------|----------|
| Stats grid (4 cards) | Installed count, Updates available (clickable -> Installed), Last scan time, Backups count + size |
| Updates preview | Lists each updatable package: icon, name, category, version arrow, individual Update button |
| "View all installed" link | Navigates to Installed page |
| Scan Installed button | Confirmation dialog -> ops dock progress |
| Update All button | Confirmation listing all packages with versions -> ops dock progress |
| Recent Activity | Static list of recent operations (update, install, scan) with timestamps |

### Catalog

| Element | Behavior |
|---------|----------|
| Search box | Filters against: id, name, description, publisher, category, method, license, dependencies. Case-insensitive. Interacts with category filter (both applied simultaneously) |
| Grid/List toggle | Visual only (grid implemented) |
| Category chips | All, Capture, Guiding, Platesolving, Focusing, Planetarium, Viewers, Drivers, Prerequisites, USB, Equipment. Color-coded. Combines with search |
| Package cards | Icon, name, publisher, description (2-line clamp), category tag, status badge. Click -> Detail page |
| Status badges | Green dot + version = installed. Yellow dot + version arrow = update available. Gray dot = not installed |

### Installed

| Element | Behavior |
|---------|----------|
| Search box | Filters installed list by text match |
| Update All button | Confirmation dialog listing packages -> ops dock |
| Re-scan button | Confirmation dialog -> ops dock |
| Updates Available group | Yellow header, rows with version arrow, Update button (with confirmation), Backup Now button (for packages with backup paths) |
| Up to Date group | Green header, rows showing current version with checkmark |
| Row click | Navigates to Detail page (back button returns to Installed) |
| Backup Now button | Confirmation dialog showing paths to back up, version -> ops dock |
| Update button | Confirmation dialog "Update X from A to B?" -> ops dock |

### Detail (Package)

Reached from Catalog or Installed. Breadcrumb returns to origin page.

**Hero section**: Large icon, package name, publisher + homepage link, description, action buttons:
- Update to X.X.X (if update available, with confirmation)
- Install X.X.X (if not installed)
- Installed (X.X.X) (disabled, if up to date)
- Backup Now (if package has backup paths, with confirmation dialog)
- Homepage (external link)

**Tabs**:

| Tab | Content |
|-----|---------|
| Overview | Info grid: installed version, latest version, category tag, install method, detection method, license, dependencies, backup paths |
| Versions | Table: version, discovered date, pre-release flag, action button (Update/Install/Current) |
| Backup (if applicable) | Backup paths (manifest = read-only with MANIFEST badge, custom paths = editable with add/remove). Auto-backup toggles (backup before update, include in scheduled). History list with preview/delete per entry. Backup Now button |
| Technical Details | Detection section (method, details, last result). Installation section (method, scope, elevation, upgrade behavior) |

### Backup & Restore

| Element | Behavior |
|---------|----------|
| Quick Restore | Two dropdowns: Application (packages with backups) + Backup (versions for selected app, updates on app change). "Preview & Restore" button shows file-level detail table |
| Restore preview table | Columns: File (monospace), Action (color-coded: yellow=Overwrite, gray=Unchanged, green=New), Current (size + date), Backup (size + date). Summary counts above table |
| Confirm Restore | Simple "This will overwrite N files. Cannot be undone." confirmation (files already visible in preview) -> ops dock |
| Cancel | Hides preview |
| All Backups | Filter dropdown (All Applications / per app). Grouped by application with collapsible headers showing icon + name + count |
| Backup items | Version, file count, size, date. Preview (eye icon) + Delete (trash icon, with confirmation) |
| Preview contents | Modal showing backup metadata + file table (name, size, modified date) |
| Delete backup | Confirmation: "Delete backup for X vY? This cannot be undone." |

### Settings

Sidebar navigation: General, Backup, Catalog, Network, Paths, Logging, About.

| Section | Fields |
|---------|--------|
| General | Auto-check for updates (toggle), Check interval (dropdown: 30min/1h/6h/24h) |
| Backup | Scheduled backup (toggle + schedule: daily/weekly/monthly). Retention: max backups per package (dropdown: unlimited/3/5/10/20), max total size (MB input), delete after N days |
| Catalog | Catalog URL (text), Cache TTL (dropdown: 1h/6h/24h/7d) |
| Network | HTTP Proxy (text), Connection timeout, Request timeout, Download speed limit |
| Paths | Download directory (with browse), Cache directory (with browse), Keep installers (toggle), Purge after N days |
| Logging | Log level (dropdown: Error-Trace), Log to file (toggle), Log file path |
| About | Version, Catalog version, Database, License, Links (GitHub/Docs/Report Issue), Check for App Updates button |

| Action | Behavior |
|--------|----------|
| Save Changes | (mock only) |
| Reset to Defaults | Confirmation: "Reset all settings to defaults? Configuration will be lost." |
| Check for App Updates | Spinner for 1.5s -> "Astro-Up is up to date (v0.1.0)" result |

## Confirmation Dialogs

All destructive or significant actions show a confirmation dialog:

| Action | Dialog |
|--------|--------|
| Scan Installed | "Scan your system for installed astrophotography software?" |
| Update All | Lists all packages with version arrows |
| Update individual | "Update {name} from {old} to {new}?" |
| Backup Now | Shows paths to back up + version |
| Confirm Restore | "This will overwrite N files. Cannot be undone." |
| Delete backup | "Delete backup for {name} v{ver}? Cannot be undone." |
| Reset to Defaults | "Reset all settings to defaults? Configuration will be lost." |

## Component Mapping (Vue)

Suggested Vue component structure:

```
App.vue                    # Layout shell, Toast, update banner
views/
  DashboardView.vue        # Stats, updates preview, quick actions, activity
  CatalogView.vue          # Search + filter + grid
  InstalledView.vue        # Grouped list with actions
  PackageDetailView.vue    # Hero + tabbed content
  BackupView.vue           # Quick restore + backup list
  SettingsView.vue         # Sectioned settings form
components/
  AppSidebar.vue           # Navigation + footer
  OperationsDock.vue       # Bottom progress panel
  ConfirmDialog.vue        # Generic confirmation modal
  InfoDialog.vue           # Generic info modal (preview contents)
  PackageCard.vue          # Catalog grid card
  PackageRow.vue           # Installed list row
  BackupGroup.vue          # Grouped backup list per app
  FileTable.vue            # Restore preview / backup contents table
  CategoryChips.vue        # Filter chip row
composables/
  useSearch.ts             # Search + filter logic
  useCoreEvents.ts         # (existing) Tauri event listener
  useTheme.ts              # (existing) Theme switching
  useErrorLog.ts           # (existing) Error log store
stores/
  operations.ts            # Active operations state (pinia or reactive)
```

## Tauri Commands Used

| Command | Pages |
|---------|-------|
| `list_software(filter)` | Catalog, Dashboard |
| `search_catalog(query)` | Catalog |
| `check_for_updates()` | Dashboard, Installed |
| `get_config()` | Settings |
| `save_config(config)` | Settings |
| `scan_installed()` | Dashboard, Installed |
| `install_software(id)` | Detail |
| `update_software(id)` | Detail, Installed, Dashboard |
| `create_backup(paths)` | Detail (Backup tab), Installed |
| `restore_backup(archive, filter)` | Backup |
| `cancel_operation(id)` | Operations dock |

## Deferred (not in this spec)

- Backup policies backend: #507 (auto-backup, scheduled, retention, custom path validation)
- Wiring scan/install/update to real core: #503, #504, #505
- Release pipeline: #464, #465
