# Research: Tauri App Shell

**Date**: 2026-04-02 | **Spec**: 016-tauri-app-shell

## R1: Tauri v2 Command & Event Patterns

**Decision**: Use `#[tauri::command]` async functions with `AppHandle` for event emission via `Emitter` trait.

**Rationale**: Tauri v2 uses `app.emit("event-name", payload)` for global events. Payloads must implement `Clone + Serialize`. Our `astro-up-core::Event` already satisfies this — emit all variants on a single `"core-event"` channel.

**Alternatives considered**:
- Per-event-type channels: Rejected — adds frontend complexity for no benefit; single channel with tagged union is idiomatic
- WebSocket sidecar: Rejected — Tauri's IPC is already optimized for this

**Key pattern** (from Tauri v2 docs):
```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
async fn install_software(app: AppHandle, id: String) -> Result<OperationId, CoreError> {
    // Forward core events to frontend
    app.emit("core-event", &event).unwrap();
}
```

Frontend listening:
```typescript
import { listen } from '@tauri-apps/api/event';
const unlisten = await listen<CoreEvent>('core-event', (event) => {
    // event.payload is typed CoreEvent
});
```

## R2: Plugin Configuration

**Decision**: Register all plugins in `lib.rs` via `tauri::Builder::default().plugin(...)`. Configure in `tauri.conf.json` under `plugins` key.

**Plugins and Cargo features**:

| Plugin | Crate | Feature |
|--------|-------|---------|
| window-state | `tauri-plugin-window-state` | — |
| single-instance | `tauri-plugin-single-instance` | — |
| updater | `tauri-plugin-updater` | `#[cfg(desktop)]` |
| autostart | `tauri-plugin-autostart` | — |

**Frontend packages** (npm):
- `@tauri-apps/plugin-window-state` — `saveWindowState`, `restoreStateCurrent`, `StateFlags`
- `@tauri-apps/plugin-autostart` — `enable`, `disable`, `isEnabled`

## R3: System Tray

**Decision**: Use Tauri v2 `TrayIconBuilder` API in `setup()` closure with menu items and event handler.

**Rationale**: Tauri v2 moved from `SystemTray` to `TrayIconBuilder`. Menu is built with `Menu::with_items()` and `MenuItem`. Badge overlay requires dynamically swapping the tray icon (no native badge API on Windows).

**Key pattern**:
```rust
use tauri::tray::{TrayIconBuilder, MenuItemBuilder};
use tauri::menu::{Menu, MenuItem};

TrayIconBuilder::new()
    .menu(&menu)
    .on_menu_event(|app, event| { /* handle clicks */ })
    .build(app)?;
```

**Badge approach**: Generate icon variants at build time (0-9, 9+) as .ico overlays. Swap tray icon dynamically via `tray.set_icon()`.

## R4: Self-Update with Ed25519

**Decision**: Use `tauri-plugin-updater` with Ed25519 key pair. Public key in `tauri.conf.json`, private key in CI secrets.

**Rationale**: Tauri v2 updater requires Ed25519 by default. Generate keys with `tauri signer generate`. The updater checks a JSON endpoint for `version`, `notes`, `pub_date`, and per-platform `url` + `signature`.

**Update endpoint format** (GitHub Releases):
```json
{
    "version": "1.2.0",
    "notes": "Bug fixes and performance improvements",
    "pub_date": "2026-04-02T00:00:00Z",
    "platforms": {
        "windows-x86_64": {
            "url": "https://github.com/.../astro-up_1.2.0_x64-setup.nsis.zip",
            "signature": "..."
        }
    }
}
```

**Key generation**: `npx @tauri-apps/cli signer generate -w ~/.tauri/astro-up.key`

## R5: PrimeVue Theme Switching

**Decision**: Use `darkModeSelector` with CSS class toggle, not `'system'` media query.

**Rationale**: The spec requires three modes (system/light/dark). Using `'system'` only supports auto-detection. Instead, use a CSS class selector (e.g., `.app-dark`) and manage it in Vue:
- `system` mode: listen to `window.matchMedia('(prefers-color-scheme: dark)')` and toggle class
- `dark` mode: always add class
- `light` mode: never add class

**Key pattern**:
```typescript
// main.ts
app.use(PrimeVue, {
    theme: {
        preset: Aura,
        options: {
            darkModeSelector: '.app-dark',
            cssLayer: false,
        },
    },
});

// ThemeManager composable
function applyTheme(mode: 'system' | 'light' | 'dark') {
    if (mode === 'dark') document.documentElement.classList.add('app-dark');
    else if (mode === 'light') document.documentElement.classList.remove('app-dark');
    else { /* watch matchMedia */ }
}
```

## R6: Toast Notifications for Errors

**Decision**: Use PrimeVue `Toast` + `useToast()` composable. Requires `ToastService` plugin registration.

**Pattern**:
```typescript
import ToastService from 'primevue/toastservice';
app.use(ToastService);

// In component:
const toast = useToast();
toast.add({ severity: 'error', summary: 'Install Failed', detail: error.message, life: 5000 });
```

## R7: VueQuery + Tauri Invoke

**Decision**: Wrap `invoke()` calls in `useQuery` (reads) and `useMutation` (writes/operations).

**Rationale**: VueQuery handles caching, loading states, and error boundaries. Tauri `invoke()` rejects on error natively — no manual mapping needed.

**Pattern**:
```typescript
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useMutation } from '@tanstack/vue-query';

const { data, isLoading } = useQuery({
    queryKey: ['software', filter],
    queryFn: () => invoke<SoftwareEntry[]>('list_software', { filter }),
});

const installMutation = useMutation({
    mutationFn: (id: string) => invoke<string>('install_software', { id }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['software'] }),
});
```
