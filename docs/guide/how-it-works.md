# How It Works

Under the hood — catalog management, version checking, caching, and the update pipeline.

## Manifest Catalog

Software definitions (manifests) are maintained in a [separate repository](https://github.com/nightwatch-astro/astro-up-manifests) and compiled into a SQLite database (`catalog.db`). Your Astro-Up installation fetches this file at runtime — you get the latest catalog without updating the app itself.

### Caching

To keep things fast and minimize bandwidth:

- **Disk cache** — `catalog.db` is persisted locally between sessions
- **TTL validation** — re-syncs only when the cache is stale (configurable, default 12h)
- **ETag support** — conditional requests avoid re-downloading unchanged data
- **Force sync** — `astro-up sync --force` or the Re-download button in Settings

### Signature Verification

The catalog is signed with [minisign](https://jedisct1.github.io/minisign/). Astro-Up verifies signatures before trusting fetched data, protecting against tampered manifests.

## Update Flow

When you update a package:

```
1. Check catalog for latest version
2. Compare against locally detected version
3. If newer: download installer (SHA-256 verified)
4. Back up configuration (profiles, settings, equipment configs)
5. Run installer (silently by default)
6. Re-run detection to verify new version
7. Record install path in ledger for future backups
```

## Detection Pipeline

Detection uses 7 methods (Registry, PE, FileExists, ConfigFile, ASCOM, WMI, DriverStore) with fallback chains. See [Detection](./detection.md) for details.

New detection configs are discovered automatically by the [lifecycle testing workflow](./lifecycle-testing.md), which installs packages on Windows runners and probes for detection signatures.

## Data Flow

```
User Action (GUI or CLI)
  -> Engine (orchestration, dependency resolution)
    -> Catalog (manifest lookup, version comparison)
    -> Detect (scan local system)
    -> Download (fetch installer, verify checksum)
    -> Backup (save config)
    -> Install (run installer, verify detection)
    -> Ledger (record install path)
```

## GUI vs CLI

Both interfaces share the same `astro-up-core` library:

- **No arguments** -> launches the GUI (Tauri v2 + Vue 3)
- **Any subcommand** (e.g., `list`, `check`, `update`) -> runs the CLI (clap)

The engine, catalog, detection, download, and install logic is identical in both paths.
