# Astro-Up

Astrophotography software manager for Windows — install, detect, and update imaging software.

## Stack

| Layer | Technology |
|-------|-----------|
| Core | Rust 2024 edition, `astro-up-core` crate |
| CLI | `astro-up-cli` — clap 4, ratatui, color-eyre |
| GUI | `astro-up-gui` — Tauri v2 |
| Frontend | Vue 3 + PrimeVue 4 (Aura dark) + VueQuery 5 + Vite 6 |
| Storage | SQLite via rusqlite (bundled) |
| Async | tokio |
| Logging | tracing + tracing-subscriber |
| Metrics | metrics + metrics-util |

## Project Structure

```
crates/
  astro-up-core/    # Shared library — types, detection, download, install, engine
  astro-up-cli/     # CLI binary (clap + ratatui)
  astro-up-gui/     # Tauri desktop app (lib.rs for commands, main.rs for app init)
frontend/           # Vue 3 + PrimeVue + VueQuery
.github/workflows/  # CI (ci.yml) + Release (release.yml)
specs/              # Speckit feature specifications
research/           # Architecture decisions and migration plan
```

## Development

```sh
just setup    # Install frontend deps, verify toolchain
just dev      # Start Tauri dev server (Vite + hot-reload)
just build    # Production Tauri build
just test     # Rust + Vue tests
just check    # All quality checks (matches CI)
just fmt      # Auto-format Rust
just lint     # Clippy + ESLint
```

## Coding Standards

### Rust
- `cargo fmt` — enforced in CI
- `cargo clippy -- -D warnings` — zero warnings policy
- Edition 2024, stable channel (pinned in `rust-toolchain.toml`)
- `thiserror` for library errors, `anyhow`/`color-eyre` for application errors
- `trait_variant::make` for async traits needing dyn dispatch
- `std::sync::LazyLock` instead of `once_cell`
- Snapshot testing with `insta`

### Vue / TypeScript
- ESLint flat config with `vue` + `typescript-eslint`
- TypeScript strict mode, `vue-tsc` for type checking
- PrimeVue Aura dark theme, `darkModeSelector: 'system'`
- VueQuery for server state (wrapping Tauri `invoke()`)

### Logging & Error Handling
- `#[tracing::instrument(skip_all, fields(...))]` on all public async/sync-with-I/O functions
- Structured fields: `operation_id` + `package` for operations, `url` + `duration_ms` for network, `path` for file I/O
- Tight loops: `trace!` event macros, NOT per-call span creation
- No `unwrap()` in I/O/network/DB/process paths — propagate with `?` + `.map_err()`
- `let _ =` and `.ok()` require `warn!`/`debug!` when suppressing meaningful failures
- No passwords or tokens in structured log fields (paths and package names are fine)
- CLI/GUI: boundary logging only — MUST NOT re-log what core already reports
- Frontend: VueQuery `onError` → toast + errorLog on all mutations, no `console.log`/`alert()`
- Frontend logging: use `logger` utility writing to LogPanel store, not browser console

## CI

**Tauri CI requirements:**
- Ubuntu Rust jobs MUST install system deps: `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
- `crates/astro-up-gui/icons/icon.ico` and `icon.png` MUST be tracked in git (Tauri build.rs requires icon.ico for Windows resource file)
- Path-conditional jobs MUST use `dorny/paths-filter@v3`, never `contains()` on event URLs

Four jobs (fast path + conditional slow paths):
1. **check-rust** (Ubuntu) — fmt, clippy, test for core + cli only (~30s, no Tauri deps)
2. **check-gui** (Ubuntu) — clippy, test for gui crate with Tauri system deps — only on `crates/astro-up-gui/**` changes
3. **check-frontend** (Ubuntu) — lint, test, build + pnpm cache
4. **check-windows** (Windows) — check, test — only on `crates/**` changes

Semantic PR titles enforced (`amannn/action-semantic-pull-request`).

## Release

- `release-plz` for automated version management and changelog
- Delegates to `nightwatch-astro/.github` shared workflow
- Trusted publishing to crates.io via OIDC
- Conventional commits: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `chore`

## Architecture Principles

1. **Modules-first crate layout** — extract modules to crates only when compile times justify
2. **Platform awareness** — `cfg(windows)` for Windows-only code, cross-platform CI
3. **Test-first** — integration tests over mocks, `insta` for snapshots
4. **Thin Tauri boundary** — all logic in `astro-up-core`, GUI is a thin adapter
5. **Spec-driven** — every feature has a speckit specification
6. **Simplicity** — YAGNI, no premature abstractions

## Branch Strategy

- Feature branches off `main`, merged with `--no-ff`
- Branch protection: require CI + 1 review
- Conventional commits required

## Active Technologies
- Rust 2024 edition + serde 1, serde_json 1, thiserror 2, strum 0.26, trait-variant 0.1, semver 1, chrono 0.4 (003-core-domain-types)
- N/A (types only — storage is in catalog/engine specs) (003-core-domain-types)
- Rust 2024 edition (1.85+) + config 0.15 (layered config), garde 0.22 (validation), humantime-serde 1.1 (durations), directories 6.0 (platform paths), toml 0.9 (raw parsing for unknown key detection — already a dev-dependency) (004-configuration-system)
- N/A — config is file-based TOML, loaded once at startup (004-configuration-system)
- Rust 2024 edition (1.85+) + garde 0.22 (validation), humantime 2 (duration parsing), rusqlite (existing — bundled SQLite) (004-configuration-system)
- SQLite — same database file as catalog and ledger (single .db per app) (004-configuration-system)
- Rust 2024 edition (1.85+) + rusqlite 0.35 (bundled, existing), minisign-verify 0.2, reqwest 0.13 (rustls-tls), serde/serde_json (existing), chrono (existing) (005-manifest-catalog)
- SQLite — read-only access to pre-compiled catalog.db from GitHub Releases (005-manifest-catalog)
- Rust 2024 edition (1.85+) + pelite 0.10 (PE), winreg 0.56 (registry), wmi 0.14 (WMI/hardware), directories 6.0 (platform paths) (006-registry-pe-detection)
- SQLite (existing — ledger entries for Acknowledged packages) (006-registry-pe-detection)
- Rust 2024 edition (1.85+) + reqwest 0.13 (existing), sha2 0.10 (new), tokio-util 0.7 (new — CancellationToken) (010-download-manager)
- N/A (file-based — `.part` files and final installers on disk) (010-download-manager)
- Rust 2024 edition (1.85+) + okio 1 (+ process feature), zip 2, windows 0.62 (cfg(windows)), tokio-util 0.7 (CancellationToken, existing) (011-installer-execution)
- SQLite (existing, ledger entries via rusqlite) (011-installer-execution)
- Rust 2024 edition (1.85+) + semver (existing), chrono (existing), sysinfo (existing from spec 005), regex (new — custom version format), rusqlite (existing — operations table), tokio (existing — async runtime), tokio-util (existing from spec 010/011 — CancellationToken) (012-install-orchestration)
- SQLite — operations table in existing app database (012-install-orchestration)
- Rust 2024 edition (1.85+) + clap 4 (derive), ratatui 0.29 (TUI), color-eyre 0.6 (errors), tracing-subscriber 0.3 (logging layers), tracing-appender 0.2 (file rotation), human-panic 2 (panic handler), dialoguer 0.11 (prompts), tabled 0.17 (tables) (015-cli-interface)
- SQLite (existing — catalog, config, ledger via astro-up-core) (015-cli-interface)
- Rust 2024 edition + tauri 2, tauri-plugin-{window-state,single-instance,updater,autostart,dialog}, dashmap 6, directories 6, uuid 1 (016-tauri-app-shell)
- Vue 3 + PrimeVue 4 (Aura), @tanstack/vue-query 5, @tauri-apps/{api,plugin-window-state,plugin-autostart,plugin-updater} 2, TypeScript 5 (016-tauri-app-shell)
- SQLite (existing — catalog + config via astro-up-core, rusqlite 0.39) (016-tauri-app-shell)
- Rust 2024 edition (1.85+), TypeScript 5, Vue 3 + racing 0.1, tracing-subscriber 0.3 (fmt, json, env-filter), tracing-appender 0.2, PrimeVue 4 (toast), @tanstack/vue-query 5 (027-logging-debugging)
- N/A (no new storage — logging is file/stderr/LogPanel) (027-logging-debugging)

- Vue 3 + vue-router 4 (hash mode), PrimeVue 4 (Aura), @tanstack/vue-query 5, @vueuse/core 14, valibot 1, TypeScript 5 (022-vue-frontend)
- localStorage for config snapshots, mock data layer for stubbed Tauri commands (022-vue-frontend)

- Rust 2024 edition (1.85+) + pelite 0.10 (PE, existing), winreg (registry, existing), wmi (WMI, existing), reqwest (downloads, existing), toml 0.9 (manifest reading, promoted from dev-dep), serde_json (fallback serialization, existing) (023-lifecycle-testing)
- SQLite (existing — catalog detection table expansion, ledger install_path recording) (023-lifecycle-testing)

## Recent Changes
- 022-vue-frontend: Complete Vue 3 frontend — 6 views (Dashboard, Catalog, Installed, Detail, Backup, Settings), 30+ components, vue-router hash mode, valibot validation, mock data layer, keyboard shortcuts, operations dock + log panel
- 016-tauri-app-shell: Tauri v2 shell — 11 commands (6 wired to core), system tray with badge, theme switching, error toasts, self-update check, close-during-operation prompt
- 015-cli-interface: CLI scaffold — all 9 commands, dual-layer tracing, OutputMode, exit codes, JSON output, confirmation helpers
- 003-core-domain-types: Added Rust 2024 edition + serde 1, serde_json 1, thiserror 2, strum 0.26, trait-variant 0.1, semver 1, chrono 0.4
