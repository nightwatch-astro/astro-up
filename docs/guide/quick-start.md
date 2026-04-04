# Quick Start

## GUI

Launch Astro-Up from the Start Menu or system tray. On first run:

1. The catalog syncs automatically
2. Navigate to **Catalog** to browse available software
3. Switch to **Installed** to see what's already on your system
4. Click any package for details, version history, and install options

## CLI

```sh
# Sync the catalog
astro-up sync

# Scan for installed software
astro-up scan

# List all available packages
astro-up list

# Check for updates
astro-up check

# Update everything
astro-up update --all

# Update a specific package
astro-up update nina-app
```

All CLI commands support `--json` for machine-readable output.

## What Happens During an Update

1. Astro-Up checks the catalog for the latest version
2. If an update is available, it downloads the installer with SHA-256 verification
3. Your configuration is backed up automatically
4. The installer runs silently (or interactively, if configured)
5. Detection re-runs to verify the new version is installed
