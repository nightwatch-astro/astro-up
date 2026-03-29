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

## CI

Three parallel jobs on every PR:
1. **check-rust** (Ubuntu) — fmt, clippy, test + Swatinem/rust-cache
2. **check-frontend** (Ubuntu) — lint, test, build + pnpm cache
3. **check-windows** (Windows) — check, test — only on `crates/**` changes

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
