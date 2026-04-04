# CLI Commands

The `astro-up` CLI provides all functionality available in the GUI, plus automation-friendly features like JSON output.

Full documentation: [nightwatch-astro.github.io/astro-up](https://nightwatch-astro.github.io/astro-up/)

## Global Options

```
astro-up [OPTIONS] <COMMAND>

Options:
  --config <PATH>       Path to config file
  --log-level <LEVEL>   Override log level (error, warn, info, debug, trace)
  --json                Output in JSON format
  -v, --verbose         Increase verbosity
  -q, --quiet           Suppress non-essential output
  -h, --help            Print help
  -V, --version         Print version
```

## Commands

### `sync`

Synchronize the software catalog.

```sh
astro-up sync           # Sync if cache is stale
astro-up sync --force   # Force re-download
```

### `scan`

Scan the system for installed astrophotography software.

```sh
astro-up scan           # Scan and display results
astro-up scan --json    # Output as JSON
```

### `list` / `show`

List packages from the catalog.

```sh
astro-up list                    # All packages
astro-up show installed          # Installed only
astro-up show outdated           # Packages with updates
astro-up show backups nina-app   # Backups for a package
astro-up list --json             # JSON output
```

### `check`

Check installed packages for available updates.

```sh
astro-up check          # Check all
astro-up check nina-app # Check specific
```

### `install`

Download and install a package.

```sh
astro-up install nina-app                # Silent install
astro-up install nina-app --interactive  # Show installer UI
```

### `update`

Update installed packages.

```sh
astro-up update nina-app   # Update specific
astro-up update --all      # Update all
```

### `backup`

Manage backups.

```sh
astro-up backup nina-app   # Back up a package
astro-up backup --list     # List backups
```

### `restore`

Restore from a backup.

```sh
astro-up restore nina-app  # Restore latest
```

### `config`

View or modify configuration.

```sh
astro-up config            # Show current config
astro-up config --edit     # Open in editor
```

### `clear`

Clear cached data.

```sh
astro-up clear --cache      # Clear catalog cache
astro-up clear --downloads  # Clear downloaded installers
astro-up clear --all        # Clear everything
```

### `self-update`

Update Astro-Up itself.

```sh
astro-up self-update           # Check and install
astro-up self-update --dry-run # Check only
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Cancelled (Ctrl+C) |
| 3 | Invalid arguments |
| 4 | Network error |
| 5 | No updates available |
