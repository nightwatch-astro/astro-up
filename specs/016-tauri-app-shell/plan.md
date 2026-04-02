# Implementation Plan: Tauri App Shell

**Branch**: `016-tauri-app-shell` | **Date**: 2026-04-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/016-tauri-app-shell/spec.md`

## Summary

Build the Tauri v2 desktop application shell as a thin adapter over `astro-up-core`. This includes: 11 Tauri commands mirroring CLI operations, real-time event forwarding, system tray with update badge, window state persistence, single instance enforcement, self-update with Ed25519 signatures, configurable theme (system/light/dark), autostart, and operation cancellation. The frontend uses Vue 3 + PrimeVue 4 + VueQuery 5 to present a reactive UI with toast error notifications and an error log panel.

## Technical Context

**Language/Version**: Rust 2024 edition (stable) + TypeScript 5
**Primary Dependencies**: tauri 2, tauri-plugin-window-state, tauri-plugin-single-instance, tauri-plugin-updater, tauri-plugin-autostart, dashmap, Vue 3, PrimeVue 4, @tanstack/vue-query 5, @tauri-apps/api 2
**Storage**: SQLite (existing via astro-up-core)
**Testing**: cargo test + insta (Rust), vitest + vue-test-utils (frontend)
**Target Platform**: Windows 10+ (compiles on macOS/Linux for CI)
**Project Type**: desktop-app (Tauri v2)
**Performance Goals**: <3s cold start, <100ms command round-trip, <50ms event latency
**Constraints**: <50MB RSS in tray mode, Windows tray icon overlay for badges, CSP: `default-src 'self'`
**Scale/Scope**: ~95 packages in catalog, single user, single window

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First Crate Layout | PASS | All logic in `astro-up-core`, GUI crate is thin adapter |
| II. Platform Awareness | PASS | `#[cfg(desktop)]` for plugins, `#[cfg(windows)]` for tray specifics |
| III. Test-First with Integration Tests | PASS | Command functions testable without Tauri runtime; insta for snapshots |
| IV. Thin Tauri Boundary | PASS | Commands are ~10-line adapters; spec FR-002 enforces this |
| V. Spec-Driven Development | PASS | Full spec with 23 FRs, 4 NFRs, 4 SCs |
| VI. Simplicity | PASS | No abstractions beyond what commands need; YAGNI |

## Project Structure

### Documentation (this feature)

```text
specs/016-tauri-app-shell/
├── plan.md              # This file
├── spec.md              # Feature specification
├── decisions.md         # Clarify-phase decisions
├── research.md          # Phase 0 research findings
├── data-model.md        # Entity model and state transitions
├── quickstart.md        # Developer quickstart
├── contracts/
│   ├── tauri-commands.rs   # Rust command signatures
│   └── frontend-types.ts  # TypeScript event/command types
├── checklists/
│   ├── requirements.md     # Spec quality checklist
│   └── comprehensive.md    # Requirements completeness checklist
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/astro-up-gui/
├── Cargo.toml           # +tauri-plugin-*, dashmap
├── tauri.conf.json      # Window, CSP, plugins, updater config
├── icons/               # App icon + tray badge overlays (0-9, 9+)
├── build.rs             # Existing (icon resource for Windows)
└── src/
    ├── main.rs          # Entry point (existing)
    ├── lib.rs           # Plugin registration, command handler, tray setup
    ├── commands.rs      # 11 Tauri command implementations
    ├── state.rs         # AppState (core + operation tokens)
    └── tray.rs          # System tray builder, menu, badge overlay

frontend/
├── src/
│   ├── main.ts          # Vue bootstrap (update PrimeVue config for theme)
│   ├── App.vue          # Root component (+ Toast, router placeholder)
│   ├── types/
│   │   └── commands.ts  # TypeScript types for commands and events
│   ├── composables/
│   │   ├── useInvoke.ts      # VueQuery wrappers for all Tauri commands
│   │   ├── useCoreEvents.ts  # Event listener composable (core-event channel)
│   │   └── useTheme.ts       # Theme manager (system/light/dark)
│   └── stores/
│       └── errorLog.ts       # Error log panel state (last 100 entries)
└── package.json         # +@tauri-apps/plugin-window-state, +@tauri-apps/plugin-autostart
```

**Structure Decision**: Extends existing `crates/astro-up-gui/` and `frontend/` directories. New Rust modules (`commands.rs`, `state.rs`, `tray.rs`) follow the modules-first principle. Frontend adds composables and stores directories for Vue state management.

## Implementation Phases

### Phase A: Setup & Foundation
- Add Tauri plugin dependencies to Cargo.toml and package.json
- Create `state.rs` with `AppState` struct (core + operations DashMap)
- Update `tauri.conf.json`: CSP, window defaults, plugin config
- Register all plugins in `lib.rs` setup closure
- Update `main.ts` PrimeVue config for CSS class-based dark mode

### Phase B: Commands & Events
- Create `commands.rs` with all 11 command implementations
- Implement event forwarding: core `Event` → `app.emit("core-event", &event)`
- Create `cancel_operation` command with CancellationToken lookup
- Register commands in `generate_handler![]`
- Create frontend TypeScript types (`types/commands.ts`)
- Create `useInvoke.ts` composable with VueQuery wrappers
- Create `useCoreEvents.ts` composable for event subscription

### Phase C: System Tray & Window Management
- Create `tray.rs` with menu builder (Show Window, Check for Updates, separator, Quit)
- Implement badge overlay (dynamic icon swap)
- Wire close button behavior: intercept `close_requested`, check active operations, prompt or hide
- Window state persistence via plugin (automatic with `StateFlags.ALL`)
- Single instance enforcement with focus-on-second-launch

### Phase D: Theme, Autostart & Error Handling
- Create `useTheme.ts` composable (system/light/dark with matchMedia listener)
- Wire theme to PrimeVue via `.app-dark` class toggle
- Register autostart plugin, wire to config `ui.autostart`
- Create `errorLog.ts` store (reactive array, capped at 100)
- Add `<Toast />` to App.vue, wire error events to toast + error log
- Implement background update check timer (FR-019)

### Phase E: Self-Update & Polish
- Configure updater plugin with Ed25519 public key
- Implement update check on startup, toast notification with Install/Dismiss
- Wire update endpoint JSON generation into release workflow
- First-run defaults (window centered, 1024x768)
- NFR validation: memory profiling in tray mode, event latency check

## Complexity Tracking

No constitution violations — no complexity justification needed.
