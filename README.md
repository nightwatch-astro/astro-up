# Astro-Up

Astrophotography software manager for Windows — install, detect, and update imaging software from one place.

**[Documentation](https://nightwatch-astro.github.io/astro-up/)** | **[Download](https://github.com/nightwatch-astro/astro-up/releases/latest)**

## Features

- **Curated catalog** of 90+ astrophotography packages (ASCOM drivers, capture apps, plate solvers, planetariums, and more)
- **Auto-detection** via Windows registry, PE headers, ASCOM profiles, WMI, and known paths
- **Silent install & update** with SHA-256 verification and automatic config backup
- **Modern GUI** (Tauri v2 + Vue 3) and full-featured **CLI** for automation
- **Configurable** via GUI Settings panel, CLI flags, or TOML config file

## Quick Start

### GUI

Download and install from [Releases](https://github.com/nightwatch-astro/astro-up/releases/latest), then launch from the Start Menu.

### CLI

```sh
astro-up sync          # Sync the catalog
astro-up scan          # Detect installed software
astro-up list          # Browse the catalog
astro-up check         # Check for updates
astro-up update --all  # Update everything
```

## Development

```sh
just setup    # Install frontend deps, verify toolchain
just dev      # Start Tauri dev server with hot-reload
just test     # Rust + Vue tests
just check    # All quality checks (matches CI)
```

See the [Development Setup](https://nightwatch-astro.github.io/astro-up/guide/development) guide for prerequisites and workflow.

## Architecture

| Crate | Purpose |
|-------|---------|
| `astro-up-core` | Shared library — catalog, detection, download, install, config |
| `astro-up-cli` | Terminal interface (clap + ratatui) |
| `astro-up-gui` | Desktop app (Tauri v2 + Vue 3 + PrimeVue) |

All business logic lives in `astro-up-core`. The CLI and GUI are thin adapters.

## License

Apache-2.0
