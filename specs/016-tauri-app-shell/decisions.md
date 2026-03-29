# Decisions Report: 016-tauri-app-shell
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: Thin Tauri adapter, all logic in core
**Choice**: Tauri commands are ~10-line functions that call into astro-up-core and format the response.
**Reasoning**: Business logic in core means CLI and GUI share the same logic. Tauri is only the bridge.

### D2: Ed25519 signing for self-update
**Choice**: Ed25519 (Tauri's default) over RSA or Authenticode.
**Reasoning**: Ed25519 is fast, small keys, and natively supported by tauri-plugin-updater. Authenticode (SignPath.io) is deferred.

### D3: Single instance enforcement
**Choice**: tauri-plugin-single-instance — second launch focuses existing window.
**Reasoning**: Multiple instances would conflict on config files, cache, and install operations.

### D4: System tray with update badge
**Choice**: Tray icon showing update count, context menu for quick actions.
**Reasoning**: Users leave the app running during imaging sessions. Tray presence enables background checks without window clutter.

## Clarify-Phase Decisions

### C1: WebView2 requirement handling
**Decision**: On Windows 10 without WebView2, show a native dialog (not WebView-based) explaining the requirement and offering a download link. The installer should bundle WebView2 bootstrapper.

### C2: Event bridge is typed, not stringly
**Decision**: Define Rust structs for each event type. Frontend receives typed JSON. No string-based event names with unstructured payloads.

### C3: Autostart is opt-in via settings
**Decision**: Not enabled by default. Users enable it in Settings. Controlled by config (spec 004) `updates.auto_start` field.

### C4: Tauri plugins scoped conservatively
**Decision**: Only grant file system access to the app's own directories. No broad fs access. Use Tauri's scoped permissions model.

## Questions I Would Have Asked

### Q1: Should the app minimize to tray on close?
**My decision**: Yes — close button minimizes to tray. Quit is via tray menu or File → Quit. This matches user expectations for tray apps.
**Impact if wrong**: Medium — some users expect close to quit. Configurable in settings.

### Q2: Should we support dark/light mode toggle or system-only?
**My decision**: System-only. PrimeVue Aura dark theme follows OS preference. No manual toggle.
**Impact if wrong**: Low — can add toggle later in the frontend spec.
