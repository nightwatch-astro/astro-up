# How It Works

## Manifest Pipeline

```
Version checker (CI) scrapes latest versions from GitHub/GitLab/vendor sites
  -> Compiler reads TOML manifests + discovered versions
  -> Builds catalog.db (SQLite with FTS5 search)
  -> Signs with minisign
  -> Published as GitHub Release artifact
```

Astro-Up fetches `catalog.db` at runtime with ETag caching and configurable TTL. You get new packages and versions without updating the app.

## Detection Pipeline

Detection runs in two phases:

1. **WMI enumeration** -- bulk query `Win32_Product` and `Win32_InstalledWin32Program` to find installed software
2. **Per-package chain** -- for each catalog package, try methods in order with fallback:

| Method | Source | Used for |
|--------|--------|----------|
| `registry` | Windows Registry `Uninstall` keys | Most applications |
| `pe_file` | PE header version info from EXE on disk | Portable apps, version validation |
| `wmi` / `wmi_apps` | WMI queries by name or product code | Apps without registry entries |
| `driver_store` | Driver store by INF provider/class | Device drivers |
| `ascom_profile` | ASCOM Profile COM interface | ASCOM drivers |
| `file_exists` | Check if file exists at known path | Simple presence detection |
| `config_file` | Parse version from config/settings file | Apps that store version in config |
| `ledger` | Astro-Up's own install ledger | Download-only packages |

Each detection config can have a `fallback` pointing to another detection config, forming a chain.

## Install Pipeline

```
1. Plan       -- resolve target version, check constraints
2. Process    -- check if software is running (abort if blocking)
3. Disk       -- check available disk space
4. Asset      -- select download asset (prompt if multiple options)
5. Download   -- fetch installer (SHA-256 verified, resumable)
6. Install    -- run installer with elevation if required
7. Verify     -- re-detect to confirm new version
8. Ledger     -- record install path and version
```

Events are emitted at each step, driving progress display in both GUI and CLI.

Elevation is handled per-package: `required` elevates via `ShellExecuteEx` with `runas`, `self` lets the installer handle its own UAC, `prohibited` runs without elevation.

## Data Flow

Both GUI (Tauri v2 + Vue 3) and CLI (clap + ratatui) share `astro-up-core`:

```
User action (GUI command or CLI subcommand)
  -> Orchestrator (pipeline coordinator, operation lock)
    -> Catalog reader (SQLite, FTS5 search)
    -> Scanner (detection chain per package)
    -> Downloader (resumable, SHA-256 verified)
    -> Installer (method-specific, elevation-aware)
    -> History (operations table in SQLite)
```

The orchestrator acquires a global file lock to prevent concurrent operations. Operation history is recorded in the `operations` table for the GUI's operation history view.
