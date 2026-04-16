# Backup & Restore

Astro-Up backs up application settings before updates, protecting your configuration during upgrades.

## Creating Backups

### GUI

Open a package's detail page and click **Backup**, or use the **Backup** view for batch operations and the Quick Restore panel.

### CLI

```sh
# Back up a specific package
astro-up backup nina-app

# List available backups
astro-up show backups

# Restore from backup
astro-up restore nina-app
```

## What Gets Backed Up

Backups capture application configuration files and settings directories. The catalog defines which paths to include for each package (e.g., `%LOCALAPPDATA%\NINA\` for N.I.N.A.). Backups are stored as compressed archives.

## Restoring

From the CLI, run `astro-up restore <package>`. In the GUI, use the **Quick Restore** panel in the Backup view to select a package and backup version.

## Pre-Update Backups

When updating a package, Astro-Up automatically backs up configuration before running the new installer. The backup path is resolved from the detection result's `install_path`, ensuring it matches the actual install location.

## Backup Policy

Configure in **Settings > Backup** (GUI) or the `backup_policy` config section:

| Setting | Default | Description |
|---------|---------|-------------|
| `scheduled` | Off | Run backups on a schedule |
| `max_per_package` | 5 | Keep at most N backups per package |
| `max_total_size_mb` | 0 (unlimited) | Total backup storage limit in MB |
| `max_age_days` | 0 (never) | Auto-delete backups older than N days |

All backup settings are configurable from the GUI Settings panel, the CLI, or the [configuration file](/reference/config).
