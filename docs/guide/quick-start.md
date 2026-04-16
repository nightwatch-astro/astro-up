# Quick Start

## GUI

Launch Astro-Up from the Start Menu or system tray. On first run:

1. The catalog syncs automatically
2. Navigate to **Catalog** to browse available software
3. Switch to **Installed** to see what is already on your system
4. Click any package for details, version history, and install options

The **Dashboard** gives an overview of installed software, available updates, and recent activity. The **Settings** view lets you configure all options.

## CLI

```sh
# Sync the catalog
astro-up sync

# Scan for installed software
astro-up scan

# Show all packages, installed, or outdated
astro-up show all
astro-up show installed
astro-up show outdated

# Search the catalog
astro-up search "plate solver"

# Install a package
astro-up install nina-app

# Update a specific package or all at once
astro-up update nina-app
astro-up update --all

# Show available backups
astro-up show backups

# Check or initialize config
astro-up config show
astro-up config init

# Self-update Astro-Up itself
astro-up self-update
```

All CLI commands support `--json` for machine-readable output.

## What Happens During an Update

1. Astro-Up checks the catalog for the latest version
2. Downloads the installer with SHA-256 verification
3. Backs up your configuration automatically
4. Runs the installer silently (or interactively if configured)
5. Re-runs detection to verify the new version is installed
6. Records the result in operation history
