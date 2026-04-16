# FAQ

## General

### Does Astro-Up work on Linux or macOS?

No. Astro-Up is Windows-only because the astrophotography software it manages (ASCOM drivers, N.I.N.A., SharpCap, etc.) is Windows-only. Detection uses Windows APIs (registry, PE headers, WMI, driver store).

### Does it work offline?

Astro-Up requires internet to sync the catalog and download installers. After the first sync, browsing and detection work offline using the cached catalog.

### Can I use it on multiple imaging PCs?

Yes. Each installation is independent with its own config, catalog cache, and backups.

### Is it free?

Yes. Apache 2.0 license.

## Elevation and Permissions

### Why does Astro-Up ask for administrator access?

Many installers (especially ASCOM drivers and device drivers) require admin privileges. Astro-Up requests elevation via Windows UAC only when the manifest declares `elevation = "required"`. User-scoped installs do not need elevation.

### Why not use sudo.exe?

Windows `sudo.exe` (available since Windows 11 24H2) runs the entire process elevated. Astro-Up only elevates the child installer process via `ShellExecuteEx` with `runas`, keeping the main process unprivileged. This is safer and works on all supported Windows versions.

### What does `elevation = "self"` mean?

The installer handles its own elevation internally (triggers its own UAC prompt). Astro-Up launches it without elevation and lets it self-elevate.

## Detection

### Version shows 1.0.0 or 0.0.0 after install

Some installers write a placeholder version to the registry or PE header. Run `astro-up scan` after the first launch of the software -- many applications update their version info on first run.

### Software is installed but not detected

1. Confirm the software is in the catalog: `astro-up show <package-id>`
2. Portable installations may not create registry entries -- the manifest may need a `file_exists` or `pe_file` detection method
3. The package may not have a detection config yet -- check for open [lifecycle tests](https://github.com/nightwatch-astro/astro-up-manifests/actions)

### False "update available"

Version format mismatch between the detection source (registry, PE header) and the catalog version string. For example, registry shows `3.1.0.2008` while the catalog has `3.1.0`. Report these at [astro-up-manifests issues](https://github.com/nightwatch-astro/astro-up-manifests/issues) so the manifest can add a `version_regex` to normalize the format.

## Portable Apps

### What are portable apps?

Packages with install method `portable` or `download_only` are extracted to `paths.portable_apps_dir` instead of running an installer. A Windows shortcut is created automatically. No registry changes or elevation needed.

### Can I force a reinstall?

Yes. Use `--allow-major` with update, or the engine supports `force_reinstall` via the GUI. This re-downloads and re-installs even when the version matches.

## Catalog

### Catalog not refreshing

The catalog has a TTL (default 24h). Force a refresh with `astro-up catalog refresh` or use the re-download button in Settings.

### Multiple download options for a package

Some releases have multiple assets (e.g., Stellarium qt5 vs qt6). Astro-Up shows an asset selection dialog when multiple options exist. In non-interactive mode, the first asset is selected automatically.

## Downloads

### Download fails or stalls

- Downloads support resume -- retry the command and it will continue from where it stopped
- Check proxy settings: `astro-up config show` and look for `network.proxy`
- Verify the download URL is still valid: `astro-up catalog refresh` to get fresh URLs

## Configuration

### Where are settings stored?

SQLite database at `%APPDATA%\nightwatch\astro-up\data\astro-up.db` in the `config_settings` table. See the [full reference](/reference/config).

### Can I use different settings per imaging PC?

Yes. Each PC has its own database. Settings are not synced between machines.
