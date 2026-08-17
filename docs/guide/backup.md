# Backup & Restore

Astro-Up backs up application settings before updates, protecting your configuration during upgrades.

## Creating Backups

### GUI

Open a package's detail page and use the **Backup** tab, or use the **Backup** view for batch operations and the Quick Restore panel.

### CLI

```sh
# Back up a specific package
astro-up backup nina-app
```

## What Gets Backed Up

Backups capture application configuration files and settings directories. The catalog defines which paths to include for each package (e.g., `%LOCALAPPDATA%\NINA\` for N.I.N.A.).

Backups are stored as compressed archives in the backup directory.

## Restoring

```sh
# Restore the most recent backup
astro-up restore nina-app

# Restore a specific archive, skipping the confirmation prompt
astro-up restore nina-app --path <archive> --yes
```

In the GUI, use the **Quick Restore** panel in the Backup view to select an application and backup version.

## Backup Policy

Configure automatic backup behavior in **Settings > Backup** (GUI) or the `[backup_policy]` section of `config.toml`:

| Setting | Default | Description |
|---------|---------|-------------|
| Scheduled backups | Off | Run backups on a schedule (daily/weekly/monthly) |
| Max per package | 5 | Keep at most N backups per package |
| Max total size | 0 (unlimited) | Total backup storage limit in MB |
| Max age | 0 (never) | Auto-delete backups older than N days |

All backup settings are configurable from the GUI Settings panel, the CLI, or the [configuration file](/reference/config).

## Pre-Update Backups

When updating a package, Astro-Up automatically backs up configuration before running the new installer. The install path is resolved from the ledger's recorded `install_path`, ensuring backup paths match the actual install location.
