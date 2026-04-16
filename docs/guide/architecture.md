# Architecture

Astro-Up is organized into three crates sharing a common core library.

## Crate Overview

```
crates/astro-up-core/src/
  types/          Domain types (Package, Version, DetectionConfig, InstallConfig)
  config/         Layered TOML config with garde validation
  catalog/        SQLite catalog reader, manifest fetching, minisign verification
  detect/         Version detection (registry, PE, file, ASCOM, WMI, driver store)
  download/       HTTP downloads with resume, progress, SHA-256 verification
  install/        Silent installers (InnoSetup, NSIS, MSI, ZIP), elevation handling
  engine/         Orchestration, dependency resolution, planning
  events/         Typed event system for UI communication
  ledger/         SQLite ledger for install history

crates/astro-up-cli/src/
  commands/       CLI command handlers (scan, list, check, install, update, etc.)
  output/         Output formatting (plain text, JSON, progress bars)
  logging/        Dual-layer tracing (stderr + file)

crates/astro-up-gui/src/
  commands.rs     Tauri command handlers (thin wrappers around core)
  state.rs        App state management
  tray.rs         System tray with badge
  log_layer.rs    Tracing layer forwarding to frontend

frontend/src/
  views/          5 views (Dashboard, Catalog, Installed, Detail, Settings)
  components/     30+ Vue components
  composables/    VueQuery hooks wrapping Tauri invoke()
  types/          TypeScript types matching Rust serde output
```

## Key Design Decisions

### Modules-First Crate Layout
All logic in `astro-up-core` as modules. Extract to separate crates only when compile times justify it.

### Thin Tauri Boundary
The GUI crate is a thin adapter — all business logic lives in `astro-up-core`. Tauri commands are one-liners that delegate to core.

### Typed Event System
Core emits `Event` variants (adjacently tagged for clean TypeScript consumption). The GUI forwards these to Vue via Tauri's event system. The CLI renders them as progress bars.

### SQLite Everywhere
Catalog, config, and ledger all use SQLite via rusqlite (bundled). Single database file per app.

### Platform-Agnostic Core
The core types and interfaces don't import Windows-specific APIs directly. Platform-specific code is behind `cfg(windows)`.

## Data Flow

```
User Action (GUI click or CLI command)
  -> Engine (orchestration)
    -> Catalog (lookup package, versions)
    -> Detect (scan local system)
    -> Download (fetch installer)
    -> Install (run installer)
    -> Ledger (record result)
  -> Event (notify UI)
```

## Frontend

The GUI uses:
- **Tauri v2** for Rust <-> JavaScript bridging
- **Vue 3** with Composition API + TypeScript
- **PrimeVue 4** (Aura dark theme) for UI components
- **VueQuery 5** for server state (wrapping Tauri `invoke()`)
- **vue-router** (hash mode) for navigation
