# Development Setup

## Prerequisites

- **Rust** (stable, 2024 edition) — via [mise](https://mise.jdx.dev/) or [rustup](https://rustup.rs/)
- **Node.js 22+** and **pnpm** — for the Vue frontend
- **Tauri CLI**: `cargo install tauri-cli`
- **just**: task runner — `cargo install just`

On Ubuntu (for CI/cross-compile), also install:
```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## Getting Started

```sh
git clone git@github.com:nightwatch-astro/astro-up.git
cd astro-up
just setup     # Install frontend deps, verify toolchain
just dev       # Start Tauri dev server (Vite + hot-reload)
just test      # Rust + Vue tests
just check     # All quality checks (matches CI)
```

## Project Structure

```
crates/
  astro-up-core/    # Shared library — types, detection, download, install
  astro-up-cli/     # CLI binary (clap + ratatui)
  astro-up-gui/     # Tauri desktop app
frontend/           # Vue 3 + PrimeVue + VueQuery
docs/               # This documentation site (VitePress)
specs/              # Speckit feature specifications
```

## Development Workflow

### Branching

- Feature branches off `main`, merged with `--no-ff`
- Branch naming: `feat/*`, `fix/*`, `refactor/*`, `docs/*`, `chore/*`

### Conventional Commits

```
feat(engine): add dependency resolution
fix(detect): handle missing registry key
docs(spec): update spec 006 requirements
refactor(config): simplify path resolution
```

### Testing

- Integration tests over mocks
- `insta` for snapshot testing
- `cargo clippy -- -D warnings` — zero warnings policy
- ESLint + `vue-tsc` for frontend

### Pre-commit Hooks

Never skip hooks (`--no-verify` is banned). If a hook fails, fix the issue.

## Build Commands

```sh
just setup    # Install deps, verify toolchain
just dev      # Tauri dev server with hot-reload
just build    # Production Tauri build
just test     # Rust + Vue tests
just check    # All quality checks
just fmt      # Auto-format Rust
just lint     # Clippy + ESLint
```
