# Quickstart: Tauri App Shell

## Prerequisites

- Rust toolchain (stable, via `rust-toolchain.toml`)
- Node.js + pnpm
- Tauri CLI: `pnpm add -D @tauri-apps/cli`
- Tauri signer (for updater key generation): `npx @tauri-apps/cli signer generate`

## Development

```sh
just dev     # Starts Vite dev server + Tauri window with hot-reload
just build   # Production build (NSIS installer)
just test    # Run Rust + Vue tests
just check   # All quality checks
```

## Architecture

```
Frontend (Vue 3 + PrimeVue)
    ↕ invoke() / events
Tauri Shell (astro-up-gui crate — thin adapter)
    ↕ function calls
Core Logic (astro-up-core crate)
    ↕ rusqlite / reqwest / tokio
SQLite + Network + File System
```

## Key Files

| File | Purpose |
|------|---------|
| `crates/astro-up-gui/src/lib.rs` | Plugin registration, commands, tray setup |
| `crates/astro-up-gui/src/main.rs` | App entry point |
| `crates/astro-up-gui/src/commands.rs` | Tauri command implementations |
| `crates/astro-up-gui/src/tray.rs` | System tray setup and event handling |
| `crates/astro-up-gui/src/state.rs` | AppState and operation management |
| `crates/astro-up-gui/tauri.conf.json` | Tauri configuration (window, CSP, plugins) |
| `frontend/src/main.ts` | Vue app bootstrap, PrimeVue + VueQuery setup |
| `frontend/src/composables/useInvoke.ts` | VueQuery wrappers for Tauri commands |
| `frontend/src/composables/useCoreEvents.ts` | Event listener composable |
| `frontend/src/composables/useTheme.ts` | Theme management (system/light/dark) |
| `frontend/src/stores/errorLog.ts` | Error log panel state (last 100 entries) |

## Adding a New Command

1. Add function in `astro-up-core` (if not exists)
2. Add `#[tauri::command]` wrapper in `commands.rs`
3. Register in `generate_handler![]` in `lib.rs`
4. Add TypeScript types in `frontend/src/types/commands.ts`
5. Create VueQuery wrapper in `useInvoke.ts`

## Testing

- **Rust unit tests**: `cargo test -p astro-up-gui`
- **Command tests**: Test each command function directly (no Tauri runtime needed for core logic)
- **Frontend tests**: `pnpm --dir frontend test` (Vitest + vue-test-utils)
- **Snapshot tests**: `insta` for Rust JSON output, Vitest snapshots for Vue components
