# Quickstart: Repository Scaffolding

## Prerequisites

- Rust (via rustup) — `rust-toolchain.toml` auto-installs the correct version
- Node.js v22+
- pnpm (`npm install -g pnpm`)
- Tauri system dependencies:
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  - **Windows**: Visual Studio Build Tools (C++ workload), WebView2 (pre-installed on Windows 11)

## Setup

```bash
git clone https://github.com/nightwatch-astro/astro-up.git
cd astro-up
just setup
```

`just setup` installs frontend dependencies and verifies the Rust toolchain.

## Development

```bash
just dev
```

Opens the Tauri development window with Vue hot-reload. Frontend changes reflect in ~2 seconds. Rust changes trigger a recompile and app restart.

## Quality Checks

```bash
just check    # All checks (matches CI)
just test     # Rust + Vue tests only
just fmt      # Auto-format Rust code
just lint     # Clippy + ESLint
```

## Build

```bash
just build    # Produces platform installer via Tauri NSIS bundler
```

## Project Layout

```
crates/
  astro-up-core/    # Shared library — all business logic
  astro-up-cli/     # CLI binary (clap + ratatui)
  astro-up-gui/     # Tauri desktop app
frontend/           # Vue 3 + PrimeVue + VueQuery
```
