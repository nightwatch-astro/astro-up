# What is Astro-Up?

Astro-Up is a software manager for astrophotography on Windows. It detects, installs, and updates your imaging software from a single app.

## The Problem

Setting up an astrophotography PC means installing dozens of applications, drivers, and tools from different vendors. Keeping everything updated requires visiting multiple websites, checking version numbers, and running installers manually. Whether your PC lives in an observatory or travels to dark sites, managing all that software is tedious and error-prone.

## What Astro-Up Does

- **Detect** installed software using 9 methods (registry, PE headers, ASCOM, WMI, driver store, and more)
- **Browse** a curated catalog of astrophotography packages
- **Install** new software with verified downloads and silent or interactive installers
- **Update** everything in one click, or queue multiple updates to run sequentially
- **Force reinstall** packages when something goes wrong
- **Manage portable apps** with automatic Windows shortcut creation

## Key Principles

| Principle | Detail |
|-----------|--------|
| Windows-optimized | Dark theme, lightweight, fast on imaging PCs |
| Major version awareness | Warns before breaking upgrades |
| Extensible catalog | TOML manifests make adding new software straightforward |
| Self-updating | Astro-Up keeps itself current with markdown release notes |

## Architecture

| Layer | Technology | Purpose |
|-------|-----------|---------|
| `astro-up-core` | Rust library | Catalog, detection, download, install, config |
| `astro-up-cli` | clap + ratatui | Terminal interface for power users |
| `astro-up-gui` | Tauri v2 + Vue 3 | Desktop app with PrimeVue dark theme |

All business logic lives in `astro-up-core`. The CLI and GUI are thin adapters sharing the same engine.

## Status

Astro-Up is in active development (v0.1.x). The core engine, CLI, and GUI are functional. The catalog is being populated with detection configs and download manifests via an [automated lifecycle testing pipeline](./lifecycle-testing.md).
