# Research: Vue Frontend Views

## Library Decisions

### vue-router 4

- **Decision**: Use vue-router 4 with `createWebHashHistory()`
- **Rationale**: Hash mode required for Tauri — no server for HTML5 history mode. Already the standard Vue router.
- **Alternatives**: Manual page switching (current approach in mockup) — rejected because it doesn't support deep linking, back/forward buttons, or URL-based navigation state.
- **Version**: ^4 (latest)
- **Tauri note**: Hash mode works out of the box. No special Tauri config needed.

### Validation: valibot over Zod

- **Decision**: Use valibot for config/form validation
- **Rationale**: ~1KB tree-shaken vs ~13KB for Zod. Same TypeScript inference. Functional API matches Vue composition style. No ecosystem lock-in needed (no tRPC/Next.js).
- **Alternatives**: Zod (larger bundle, overkill for form validation), hand-written validation (error-prone, no type inference), joi (not TypeScript-native)
- **Version**: ^1

### Keyboard shortcuts: @vueuse/core

- **Decision**: Use `useMagicKeys` from @vueuse/core
- **Rationale**: Already need @vueuse for useStorage (config snapshots) and useResizeObserver (log panel). useMagicKeys provides reactive keyboard state with combo support. No extra dependency.
- **Alternatives**: Dedicated keyboard library (mousetrap, hotkeys-js) — unnecessary when @vueuse covers it. Native addEventListener — works but verbose for combos.
- **Version**: ^14

### PrimeVue components needed

From the mockup and spec, we'll use these PrimeVue components:

| Component | Usage |
|-----------|-------|
| DataTable | Version history, file table, backup contents |
| Toast | Notifications (FR-028, FR-046) |
| Dialog | Confirmation dialogs (FR-017) |
| Dropdown | Category filter, settings dropdowns, backup selects (FR-032) |
| InputText | Search boxes, settings text fields |
| InputNumber | Numeric settings fields |
| Checkbox | Backup path selection |
| ToggleSwitch | Settings toggles |
| TabView/TabPanel | Detail page tabs |
| ProgressBar | Operations dock progress |
| Skeleton | Loading states (FR-044) |
| ScrollPanel | Log panel scrolling |
| Tag | Category badges, status badges |
| Badge | Sidebar update count |
| Button | All action buttons |
| Breadcrumb | Detail page back navigation |
| Chip | Category filter chips (or custom) |

### VueQuery patterns

The existing `useInvoke.ts` already implements the correct patterns:
- `useQuery` for reads with reactive query keys
- `useMutation` with `onSuccess` -> `invalidateQueries` for writes
- Query keys: `["software"]`, `["catalog-search", query]`, `["updates"]`, `["config"]`

New queries needed:
- `["backups"]` — mock data, list all backups
- `["backup-contents", archive]` — mock data, file listing
- `["backup-preview", archive]` — mock data, restore diff

Cache invalidation on operation complete:
- Scan -> invalidate `["software"]`, `["updates"]`
- Install/Update -> invalidate `["software"]`, `["updates"]`
- Backup -> invalidate `["backups"]`
- Restore -> invalidate `["backups"]`
- Config save -> invalidate `["config"]`

## Open Questions Resolved

### How to handle Ctrl+F override

WebView2 intercepts Ctrl+F for find-in-page. In Tauri v2, use `event.preventDefault()` in a global keydown handler registered before the webview's handler. The Tauri window config can also disable built-in shortcuts via `"windowEffects"` but the simpler approach is frontend-side preventDefault.

### Theme switching mechanism

The existing `useTheme.ts` uses `window.matchMedia` for system detection and a CSS class toggle. PrimeVue Aura supports dark/light via the `darkModeSelector` option (already set to `.app-dark`). To add user choice: store preference in config (`get_config`/`save_config`), apply class on load, and watch for system changes only when set to "system" mode.

### Log panel data source

Tauri's tracing-subscriber can emit logs to the frontend via the event system. The `listen("log-event", ...)` pattern mirrors the existing `listen("core-event", ...)`. A new Tauri command or plugin would be needed to stream logs. For MVP, the log panel subscribes to `core-event` and `log-event` channels and displays them — if `log-event` isn't implemented yet, it shows only core events.

### Config snapshots in localStorage

localStorage limit is ~5-10MB per origin in WebView2. A full config JSON is ~2-5KB. Storing 10 snapshots = ~50KB — well within limits. Use `useStorage` from @vueuse for reactive localStorage access.
