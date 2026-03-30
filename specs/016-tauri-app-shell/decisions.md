# Decisions Report: 016-tauri-app-shell

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: Thin Tauri adapter, all logic in core
**Choice**: Tauri commands are ~10-line functions calling into astro-up-core.
**Reasoning**: CLI and GUI share logic. Tauri is the bridge, not the brain.

### D2: Single instance enforcement
**Choice**: tauri-plugin-single-instance. Second launch focuses existing window.

### D3: Typed events, not stringly
**Choice**: Rust structs for each event type. Frontend receives typed JSON.

### D4: Scoped file system permissions
**Choice**: Only grant access to app's own directories. No broad fs access.

## Clarify-Phase Decisions (Interactive)

### C1: Separate Ed25519 keys for catalog and app updates
**Finding**: User asked if we can reuse the minisign key for Tauri updates.
**Decision**: Separate keys. Both use Ed25519 underneath, but different trust boundaries:
- Catalog signing key → manifests repo CI secrets
- App update signing key → astro-up release CI secrets
Different repos, different rotation cadence. If one is compromised, the other isn't.
**Note**: minisign IS Ed25519. Tauri uses raw Ed25519. Same algorithm, different format/wrapping.

### C2: Configurable close button behavior
**Finding**: User wanted close button to be configurable.
**Decision**: `ui.close_action` config: `"minimize"` (default) or `"quit"`. Default keeps the app in tray for background update checks. Users who prefer close=quit can change it.

### C3: Light/dark/system theme toggle
**Finding**: User wanted a theme toggle, not system-only.
**Decision**: `ui.theme` config: `"system"` (default), `"light"`, `"dark"`. Exposed in settings UI. PrimeVue supports all three. Astrophotographers need dark mode at night, light mode during daytime setup.

### C4: API mirrors CLI operations, GUI presents differently
**Finding**: User noted GUI shouldn't have the same sections as CLI.
**Decision**: Tauri commands mirror CLI operations 1:1 (same core functions). But the GUI presents a single filterable software list, not separate pages per show variant. The `list_software` command takes a filter parameter (`all`, `installed`, `outdated`, `category:guiding`). The frontend handles the UI presentation.

## Questions I Would Have Asked

### Q1: Should the app minimize to tray on startup if autostart is enabled?
**My decision**: Yes — if autostart is enabled, the app starts minimized to tray (no window). The user opens it from the tray when needed. Background update checking runs silently.

### Q2: Should we support notification clicks opening the app?
**My decision**: Yes — when a "3 updates available" notification is shown, clicking it opens/focuses the main window filtered to outdated packages.
