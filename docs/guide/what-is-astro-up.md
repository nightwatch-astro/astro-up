# What is Astro-Up?

Astro-Up is a purpose-built software manager for astrophotography on Windows. It handles the complete lifecycle of your imaging software — detection, installation, updates, and backup — from a single GUI or CLI.

## The Problem

Setting up an astrophotography imaging PC means installing dozens of applications, drivers, and tools from different vendors. Keeping everything updated requires visiting multiple websites, checking version numbers, and running installers manually. Whether your imaging PC is a permanent observatory setup or a portable rig for dark sites, managing software across all those vendors is tedious and error-prone.

## What Astro-Up Does

- **Detect** installed versions of applications, drivers, and tools automatically
- **Browse** a curated catalog of 90+ astrophotography packages
- **Install** new software with verified downloads and silent installers
- **Update** everything with one click or command
- **Back up** configuration before upgrading (profiles, settings, equipment configs)
- **Track** star databases, ASCOM drivers, and resources

## Key Design Principles

- **Windows-optimized** — dark theme, lightweight, fast on imaging PCs
- **Config backup** — automatic backup before every upgrade
- **Major version awareness** — warns before breaking upgrades
- **Extensible** — TOML manifests make adding new software trivial
- **Self-updating** — Astro-Up keeps itself up to date

## Architecture

Astro-Up is built in Rust with a clean separation of concerns:

| Layer | Technology | Purpose |
|-------|-----------|---------|
| `astro-up-core` | Rust library | Catalog, detection, download, install, config |
| `astro-up-cli` | clap + ratatui | Terminal interface for power users |
| `astro-up-gui` | Tauri v2 + Vue 3 | Desktop app with PrimeVue UI |

All business logic lives in `astro-up-core`. The CLI and GUI are thin adapters that share the same engine.

## Status

Astro-Up is in active development. The core engine, CLI, and GUI are functional. The catalog is being populated with detection configs and download manifests via an [automated lifecycle testing pipeline](./lifecycle-testing.md).
