# Troubleshooting

## Version Detection

### Software is installed but Astro-Up doesn't detect it

Astro-Up detects software through Windows Registry, PE file headers, ASCOM profiles, and known paths. If detection fails:

1. Confirm the software is in the catalog (`astro-up list`)
2. Some software registers differently depending on install method (MSI vs EXE vs ZIP)
3. Portable installations may not create registry entries
4. The package may not have a detection config yet — check if there's an open [lifecycle test](https://github.com/nightwatch-astro/astro-up-manifests/actions) for it

### Wrong version detected

Some vendors store version numbers inconsistently between the registry and the executable. Please [report it](https://github.com/nightwatch-astro/astro-up-manifests/issues) if you see the wrong version.

## Updates

### Installer fails silently

Silent installation arguments vary by vendor. If an update fails:

1. Try running the update with `--interactive` to see the installer UI
2. Check if the software requires admin privileges
3. Check if another instance of the software is running (Astro-Up warns about blocking processes)

### Download fails

- Verify your internet connection
- Some vendor download links expire or change — try syncing the catalog: `astro-up sync --force`
- Check **Settings > Network** for proxy or timeout issues

## Backups

### Restore doesn't fix the issue

Backups cover configuration files (profiles, settings, equipment configs) but not the application itself. If an update broke the software:

1. Restore the backup from the Backup view or `astro-up restore <package>`
2. Manually reinstall the previous version from the vendor's website

### Backup paths are empty

Some software doesn't create config files until first run. If you've never launched the software, there's nothing to back up.

## General

### Verbose output

```sh
astro-up --verbose check
```

### Log file

Enable file logging in **Settings > Logging** or set `log_to_file = true` in `config.toml`. Logs are written to `%APPDATA%\nightwatch\astro-up\logs\`.

### Reporting issues

When reporting a bug, include:
- The command you ran (with `--verbose` output)
- Your Windows version
- Which software was affected

File issues at [github.com/nightwatch-astro/astro-up/issues](https://github.com/nightwatch-astro/astro-up/issues).
