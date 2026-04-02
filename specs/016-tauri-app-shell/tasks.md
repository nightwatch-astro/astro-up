# Tasks: Tauri App Shell

**Input**: Design documents from `/specs/016-tauri-app-shell/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add dependencies, configure Tauri, create shared infrastructure

- [ ] T001 Add tauri-plugin-window-state, tauri-plugin-single-instance, tauri-plugin-updater, tauri-plugin-autostart, dashmap, uuid dependencies to `crates/astro-up-gui/Cargo.toml`
- [ ] T002 Add @tauri-apps/plugin-window-state, @tauri-apps/plugin-autostart frontend dependencies to `frontend/package.json`
- [ ] T003 Update `crates/astro-up-gui/tauri.conf.json`: set CSP to `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'`, configure window defaults (1024x768, centered), add plugin config sections, scope FS permissions to app data/config/cache/log dirs per FR-015
- [ ] T004 Create `crates/astro-up-gui/src/state.rs` with AppState struct: core (astro_up_core::App) + operations (DashMap<String, CancellationToken>) + OperationId type
- [ ] T005 Create `frontend/src/types/commands.ts` with TypeScript types for CoreEvent union, OperationId, CoreError, UpdateAvailable, ErrorLogEntry per contracts/frontend-types.ts

---

## Phase 2: Foundational

**Purpose**: Plugin registration and event infrastructure that ALL user stories depend on

- [ ] T006 Rewrite `crates/astro-up-gui/src/lib.rs`: register all plugins (window-state, single-instance, updater, autostart) in setup closure, initialize AppState with core crate, wire `generate_handler![]` with placeholder commands
- [ ] T007 Create `crates/astro-up-gui/src/commands.rs` with CoreError type implementing `Serialize` and `Into<tauri::InvokeError>`, plus a helper function `emit_event(app: &AppHandle, event: &Event)` that calls `app.emit("core-event", event)`
- [ ] T008 Update `frontend/src/main.ts`: change PrimeVue darkModeSelector from `"system"` to `".app-dark"`, add ToastService plugin registration
- [ ] T009 Create `frontend/src/composables/useCoreEvents.ts`: composable that calls `listen<CoreEvent>("core-event", callback)` from `@tauri-apps/api/event`, returns reactive event stream, auto-unlisten on unmount

**Checkpoint**: Foundation ready — plugins registered, event infra in place, CSP enforced

---

## Phase 3: User Story 1 — Launch Desktop Application (Priority: P1)

**Goal**: App launches fast, remembers window state, enforces single instance

**Independent Test**: Launch app, resize, close, relaunch — window state restored. Launch second instance — first instance focused.

- [ ] T010 [US1] Wire window-state plugin in lib.rs setup: call `StateFlags::ALL` restore on window creation, save on close. Handle invalid coordinates (removed monitor) by resetting to centered default.
- [ ] T011 [US1] Wire single-instance plugin in lib.rs setup: on second launch, focus existing window via `app.get_webview_window("main").set_focus()`
- [ ] T012 [US1] Add tracing instrumentation to lib.rs: log app startup timing, plugin registration, window creation at debug level per NFR-003 in `crates/astro-up-gui/src/lib.rs`
- [ ] T013 [US1] Update `frontend/src/App.vue`: add `<Toast />` component at root, show version in layout

**Checkpoint**: App launches with window state persistence and single instance

---

## Phase 4: User Story 2 — System Tray (Priority: P2)

**Goal**: Tray icon with badge, context menu, configurable close behavior

**Independent Test**: Minimize to tray, right-click menu works, close button respects config

- [ ] T014 [US2] Create `crates/astro-up-gui/src/tray.rs`: build tray with TrayIconBuilder, menu items (Show Window, Check for Updates, separator, Quit), wire menu event handler
- [ ] T015 [US2] Implement badge overlay in tray.rs: generate/load numbered icon variants (0-9, 9+), expose `set_badge_count(count: usize)` that calls `tray.set_icon()`
- [ ] T016 [US2] Wire close button behavior in lib.rs: intercept `on_window_event(WindowEvent::CloseRequested)`, read `ui.close_action` from config, minimize to tray (default) or quit
- [ ] T017 [US2] Implement close-during-operation prompt in lib.rs: when close requested and `state.operations` is non-empty, show native dialog "Cancel operations or continue in background?", handle each choice per FR-017
- [ ] T018 [US2] Create tray badge icon assets: generate numbered overlay images (0-9, 9+) as .png files in `crates/astro-up-gui/icons/badges/`

**Checkpoint**: Tray icon with menu, badge, configurable close behavior

---

## Phase 5: User Story 3 — Tauri Commands (Priority: P3)

**Goal**: All 11 Tauri commands bridge frontend to core crate

**Independent Test**: Call each command from frontend, verify typed JSON responses and event streaming

- [ ] T019 [P] [US3] Implement read commands in `crates/astro-up-gui/src/commands.rs`: list_software, search_catalog, check_for_updates, get_config — each delegates to core, returns typed Result
- [ ] T020 [P] [US3] Implement write commands in `crates/astro-up-gui/src/commands.rs`: save_config — delegates to core config save
- [ ] T021 [US3] Implement long-running operation commands in `crates/astro-up-gui/src/commands.rs`: install_software, update_software, scan_installed, create_backup, restore_backup — each spawns tokio task, registers CancellationToken in AppState, emits events via `emit_event()`
- [ ] T022 [US3] Implement cancel_operation command in `crates/astro-up-gui/src/commands.rs`: look up token by OperationId in AppState, trigger cancellation, remove from map
- [ ] T023 [US3] Register all 11 commands in `generate_handler![]` in `crates/astro-up-gui/src/lib.rs`
- [ ] T024 [P] [US3] Create `frontend/src/composables/useInvoke.ts`: VueQuery wrappers — useQuery for reads (list_software, search_catalog, check_for_updates, get_config), useMutation for writes (install_software, update_software, save_config, create_backup, restore_backup, cancel_operation, scan_installed)
- [ ] T025 [US3] Create `frontend/src/stores/errorLog.ts`: reactive array of ErrorLogEntry capped at 100, addEntry/clearEntries functions. Wire useCoreEvents to push error events into the store and show PrimeVue toast via useToast()

**Checkpoint**: All commands callable from frontend, errors surfaced as toasts

---

## Phase 6: User Story 4 — Theme Support (Priority: P4)

**Goal**: System/light/dark theme toggle, persisted via config

**Independent Test**: Switch themes, verify UI updates. Restart, verify setting persists.

- [ ] T026 [US4] Create `frontend/src/composables/useTheme.ts`: read `ui.theme` from config (via get_config), manage `.app-dark` class on `document.documentElement`. System mode: listen to `matchMedia('(prefers-color-scheme: dark)')` changes. Light/dark: set class directly. Save changes via save_config.
- [ ] T027 [US4] Wire useTheme in `frontend/src/App.vue`: initialize theme on mount, expose toggle for settings UI

**Checkpoint**: Theme switching works across system/light/dark with persistence

---

## Phase 7: User Story 5 — Auto-Update (Priority: P5)

**Goal**: Self-update check on startup, toast notification, Ed25519 signature verification

**Independent Test**: Point updater endpoint to test server with newer version, verify toast appears

- [ ] T028 [US5] Configure updater plugin in `crates/astro-up-gui/tauri.conf.json`: set endpoints array to GitHub Releases URL, add Ed25519 public key, set check interval
- [ ] T029 [US5] Implement startup update check in `crates/astro-up-gui/src/lib.rs` setup: call updater check after plugins init, emit custom event `"update-available"` with version info if update found. Handle signature verification failure per FR-021.
- [ ] T030 [US5] Implement background update check timer in lib.rs: read `ui.check_interval` from config, spawn periodic task that calls updater check, updates tray badge count, shows system notification when window is hidden per FR-019
- [ ] T031 [US5] Handle update notification in frontend: listen for `"update-available"` event in App.vue, show toast with "Install" and "Dismiss" actions. Install triggers `@tauri-apps/plugin-updater` install + relaunch. Dismiss hides toast.

**Checkpoint**: Self-update flow works end-to-end

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Autostart, logging, quality checks

- [ ] T032 Wire autostart plugin in lib.rs: read `ui.autostart` from config on startup, enable/disable via plugin API. When autostart enabled, launch minimized to tray per FR-012
- [ ] T033 Add tracing spans for all Tauri commands in commands.rs: log command name, params (redacted), duration, result status at debug level per NFR-003
- [ ] T034 Run `cargo fmt` and `cargo clippy -- -D warnings` on `crates/astro-up-gui/`
- [ ] T035 Run `pnpm --dir frontend lint` and `pnpm --dir frontend build` (vue-tsc type check)
- [ ] T036 Run `cargo test -p astro-up-gui` and `pnpm --dir frontend test`
- [ ] T037 Validate NFR-001/002: launch app, hide window to tray, measure RSS memory (<50MB) and CPU usage (<5% sustained) over 60 seconds using `sysinfo` or OS tools

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational
- **US2 (Phase 4)**: Depends on Foundational
- **US3 (Phase 5)**: Depends on Foundational + US1 (App.vue with Toast)
- **US4 (Phase 6)**: Depends on US3 (needs get_config/save_config commands)
- **US5 (Phase 7)**: Depends on US2 (tray badge) + US3 (toast/error infra)
- **Polish (Phase 8)**: Depends on all user stories

### User Story Dependencies

- **US1 (P1)**: Independent after Foundational — window + single instance only
- **US2 (P2)**: Independent after Foundational — tray is standalone
- **US3 (P3)**: Depends on US1 (App.vue Toast component setup)
- **US4 (P4)**: Depends on US3 (config commands needed for theme persistence)
- **US5 (P5)**: Depends on US2 (tray badge) + US3 (toast infra)

### Parallel Opportunities

- T001 + T002: different files (Cargo.toml vs package.json)
- T019 + T020 + T024: read commands, write commands, and frontend wrappers in different files
- T014-T018 within US2: T018 (icon assets) parallel with T014-T015 (Rust code)
- US1 + US2 can run in parallel after Foundational

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: US1 — App launches, window state persists, single instance
4. **STOP and VALIDATE**: Test launch, resize, close, relaunch cycle

### Incremental Delivery

1. Setup + Foundational → foundation ready
2. US1 → App launches correctly (MVP)
3. US2 → Tray icon with menu and close behavior
4. US3 → All commands work, errors shown as toasts
5. US4 → Theme toggle
6. US5 → Self-update
7. Polish → Autostart, logging, quality gates

---

## Task Dependencies

<!-- Machine-readable. Generated by /speckit.tasks, updated by /speckit.iterate.apply -->
<!-- Do not edit manually unless you also update GitHub issue dependencies -->

```toml
[graph]

[graph.T001]
blocked_by = []

[graph.T002]
blocked_by = []

[graph.T003]
blocked_by = []

[graph.T004]
blocked_by = []

[graph.T005]
blocked_by = []

[graph.T006]
blocked_by = ["T001", "T003", "T004"]

[graph.T007]
blocked_by = ["T006"]

[graph.T008]
blocked_by = ["T002"]

[graph.T009]
blocked_by = ["T002", "T005"]

[graph.T010]
blocked_by = ["T006"]

[graph.T011]
blocked_by = ["T006"]

[graph.T012]
blocked_by = ["T006"]

[graph.T013]
blocked_by = ["T008"]

[graph.T014]
blocked_by = ["T006"]

[graph.T015]
blocked_by = ["T014"]

[graph.T016]
blocked_by = ["T014"]

[graph.T017]
blocked_by = ["T016", "T004"]

[graph.T018]
blocked_by = []

[graph.T019]
blocked_by = ["T007"]

[graph.T020]
blocked_by = ["T007"]

[graph.T021]
blocked_by = ["T007", "T004"]

[graph.T022]
blocked_by = ["T021"]

[graph.T023]
blocked_by = ["T019", "T020", "T021", "T022"]

[graph.T024]
blocked_by = ["T009", "T005"]

[graph.T025]
blocked_by = ["T009", "T024"]

[graph.T026]
blocked_by = ["T024"]

[graph.T027]
blocked_by = ["T026", "T013"]

[graph.T028]
blocked_by = ["T003"]

[graph.T029]
blocked_by = ["T006", "T028"]

[graph.T030]
blocked_by = ["T029", "T015"]

[graph.T031]
blocked_by = ["T025", "T029"]

[graph.T032]
blocked_by = ["T006", "T019"]

[graph.T033]
blocked_by = ["T023"]

[graph.T034]
blocked_by = ["T033"]

[graph.T035]
blocked_by = ["T027"]

[graph.T036]
blocked_by = ["T034", "T035"]

[graph.T037]
blocked_by = ["T036"]
```
