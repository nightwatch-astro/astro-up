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

### Verifying a download

Every released binary and installer carries a GitHub build provenance attestation — a
signed statement of which workflow run, at which commit, produced those exact bytes.
Check one with the [GitHub CLI](https://cli.github.com/):

```sh
gh attestation verify Astro-Up_x64-setup.exe -R nightwatch-astro/astro-up
gh attestation verify astro-up-cli.exe       -R nightwatch-astro/astro-up
```

Verification contacts GitHub, since the attestation is stored with the repository rather
than alongside the download. Add `--format json` for the full predicate.

This is separate from the `.sig` files in each release: those are Tauri updater
signatures, checked by the app's own auto-updater against its embedded public key, and
they work offline.

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

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

This project is licensed under the GNU Affero General Public License v3.0 — see [LICENSE](LICENSE) for details.

If you modify this software and make it available over a network, you must make your modified source code available under the same license.
