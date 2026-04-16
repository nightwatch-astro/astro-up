# CLI Reference

```
astro-up [OPTIONS] <COMMAND>
```

## Global Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--json` | | Output as JSON (for scripting) |
| `--verbose` | `-v` | Show debug output |
| `--quiet` | `-q` | Suppress non-error output |
| `--config <PATH>` | | Path to config database |

## Commands

### `catalog sync`

Sync the catalog if cache is stale.

```sh
astro-up catalog sync
```

### `catalog refresh`

Force re-download the catalog regardless of TTL.

```sh
astro-up catalog refresh
```

### `scan`

Scan the system for installed astrophotography software. Uses WMI enumeration, then per-package detection chains (registry, PE headers, ASCOM profiles, known paths, driver store).

```sh
astro-up scan
```

### `show`

Show software status. Without arguments, lists all catalog packages.

```sh
astro-up show                    # All packages
astro-up show installed          # Installed only
astro-up show outdated           # Packages with available updates
astro-up show backups            # All backups
astro-up show backups nina-app   # Backups for a specific package
astro-up show nina-app           # Detail view for a package
```

### `install`

Download and install a package.

```sh
astro-up install nina-app                # Install (interactive by default)
astro-up install nina-app --dry-run      # Preview only
astro-up install nina-app -y             # Skip confirmation
```

| Flag | Description |
|------|-------------|
| `--dry-run` | Show what would be installed without executing |
| `-y, --yes` | Skip confirmation prompt |

### `update`

Update installed packages.

```sh
astro-up update nina-app               # Update one package
astro-up update --all                  # Update all outdated packages
astro-up update --all --dry-run        # Preview all updates
astro-up update nina-app --allow-major # Allow major version jumps
astro-up update nina-app -y            # Skip confirmation
```

| Flag | Description |
|------|-------------|
| `--all` | Update all outdated packages |
| `--dry-run` | Show plan without executing |
| `--allow-major` | Allow updates across major versions |
| `-y, --yes` | Skip confirmation prompt |

### `search`

Full-text search across package names, descriptions, tags, aliases, and publishers.

```sh
astro-up search "plate solver"
```

### `backup`

Create a configuration backup for a package.

```sh
astro-up backup nina-app
```

### `restore`

Restore a package from a backup.

```sh
astro-up restore nina-app                   # Restore latest backup
astro-up restore nina-app --path backup.zip # Restore specific file
astro-up restore nina-app -y                # Skip confirmation
```

| Flag | Description |
|------|-------------|
| `--path <FILE>` | Restore from a specific backup file |
| `-y, --yes` | Skip confirmation prompt |

### `config`

Manage configuration (SQLite-backed key-value store).

```sh
astro-up config init    # Generate default config
astro-up config show    # Show effective configuration
```

### `self-update`

Update astro-up itself.

```sh
astro-up self-update            # Check and install update
astro-up self-update --dry-run  # Check only
```

### `lifecycle-test`

Run a full lifecycle test for a package (download, install, detect, uninstall). Used by CI to validate manifests.

```sh
astro-up lifecycle-test nina-app --manifest-path ./manifests
astro-up lifecycle-test nina-app --manifest-path ./manifests --dry-run
astro-up lifecycle-test nina-app --manifest-path ./manifests --version 3.1.2
```

| Flag | Description |
|------|-------------|
| `--manifest-path <DIR>` | Path to manifests repo checkout (required) |
| `--version <VER>` | Specific version to test (default: latest) |
| `--install-dir <DIR>` | Install directory for download-only packages |
| `--catalog-path <FILE>` | Path to compiled catalog.db for version resolution |
| `--dry-run` | Download and probe only, skip install/uninstall |
| `--report-file <FILE>` | Write JSON report to file |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Cancelled (Ctrl+C) |
| 3 | Invalid arguments |
| 4 | Network error |
| 5 | No updates available |
