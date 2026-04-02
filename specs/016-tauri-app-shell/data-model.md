# Data Model: Tauri App Shell

**Date**: 2026-04-02 | **Spec**: 016-tauri-app-shell

## Entities

### TauriCommand

Thin adapter wrapping `astro-up-core` functions. Each command is a `#[tauri::command]` async function.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Command identifier (snake_case, matches `invoke()` call) |
| parameters | Struct | Deserialized from frontend JSON |
| return_type | Result<T, CoreError> | Serialized back as JSON |

### CoreEvent (existing — `astro-up-core::Event`)

Already defined in `crates/astro-up-core/src/events.rs`. Adjacently tagged enum with 25+ variants covering download, install, backup, restore, scan, and orchestration events. No changes needed — forwarded as-is to frontend via `app.emit("core-event", &event)`.

### OperationId

Unique identifier for a long-running operation. Used by frontend to correlate events and cancel.

| Field | Type | Description |
|-------|------|-------------|
| id | String | UUID v4 |

### AppState (Tauri managed state)

| Field | Type | Description |
|-------|------|-------------|
| core | astro_up_core::App | Core application instance (catalog, config, engine) |
| operations | DashMap<String, CancellationToken> | Active operation tokens keyed by OperationId |

### ErrorLogEntry (frontend-only)

| Field | Type | Description |
|-------|------|-------------|
| timestamp | DateTime | When the error occurred |
| severity | String | "error" / "warning" |
| summary | String | Short error message (toast text) |
| detail | String | Full error context |

Stored in Vue reactive state, capped at 100 entries per session.

### UpdateEndpoint (GitHub Releases JSON)

| Field | Type | Description |
|-------|------|-------------|
| version | String | Semver of latest release |
| notes | String | Changelog / release notes |
| pub_date | String | ISO 8601 timestamp |
| platforms | Map<String, PlatformAsset> | Per-platform download info |

### PlatformAsset

| Field | Type | Description |
|-------|------|-------------|
| url | String | Download URL for the installer |
| signature | String | Ed25519 signature of the asset |

## State Transitions

### Operation Lifecycle

```
Idle → Started → [Progress]* → Complete | Failed | Cancelled
```

- `Started`: OperationId created, CancellationToken registered in AppState
- `Progress`: Events emitted via `core-event` channel
- `Complete`: Token removed from AppState, frontend notified
- `Failed`: Token removed, error emitted, toast shown
- `Cancelled`: Token triggered, cleanup runs, token removed

### Window Close During Operation

```
CloseRequested → Prompt("cancel or background?")
  → Cancel: trigger all active CancellationTokens → cleanup → exit
  → Background: hide window → tray icon stays → operations continue
```

### Theme State

```
system | light | dark  (persisted in config as ui.theme)
  → system: listen to OS matchMedia, toggle .app-dark class
  → dark: always .app-dark
  → light: never .app-dark
```
