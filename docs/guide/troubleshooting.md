# Troubleshooting

## Elevation Failures

**UAC prompt not appearing**: Astro-Up elevates the installer process via `ShellExecuteEx` with `runas`. If UAC is disabled via Group Policy, installers requiring elevation will fail. Either enable UAC or run Astro-Up itself as administrator.

**Elevation denied**: When you click "No" on the UAC prompt, the installer returns a cancellation exit code. Astro-Up reports this as a failed operation. Re-run and accept the prompt.

**`elevation = "self"` packages**: These installers trigger their own UAC. If they fail silently, the installer may not support the silent switches in the manifest. Try interactive mode: `astro-up install <package> --yes` (uses interactive switches).

## Version Detection

**Version shows 1.0.0 or 0.0.0**: The installer wrote a placeholder version to the registry or PE header. Run `astro-up scan` after the software has been launched once -- many applications update their version info on first run.

**Software installed but not detected**: Check `astro-up show <package-id>` to confirm it is in the catalog. Portable installations without registry entries need a `pe_file` or `file_exists` detection method in the manifest. If the manifest lacks a `[detection]` section, the lifecycle test workflow discovers one automatically.

**False "update available"**: Version format mismatch between detected version and catalog version. For example, registry reports `3.1.0.2008` but the catalog has `3.1.0`. Report at [astro-up-manifests issues](https://github.com/nightwatch-astro/astro-up-manifests/issues) -- the fix is adding `version_regex` to normalize the format.

## Asset Selection

**"Asset selection cancelled"**: The package has multiple download assets (e.g., 32-bit vs 64-bit, qt5 vs qt6). In non-interactive mode or when no selector is provided, the first asset is picked automatically. In the GUI, a dialog prompts you to choose.

## Catalog

**Catalog not refreshing**: The catalog has a TTL (default 24h). Force with `astro-up catalog refresh`. If the catalog file is corrupted (integrity check fails), delete it from `%APPDATA%\nightwatch\astro-up\data\` and re-sync.

**Catalog schema unsupported**: Your astro-up version is older than the catalog. Update astro-up: `astro-up self-update`.

## Downloads

**Download fails**: Downloads are resumable -- re-run the command and it continues from where it stopped. Check proxy settings (`network.proxy`) if behind a firewall.

**Slow downloads**: Set a speed limit with `network.download_speed_limit` if you need to cap bandwidth during imaging sessions.

## General

### Verbose output

```sh
astro-up --verbose scan
```

### Log file

Enable file logging: set `logging.log_to_file` to `true` via `astro-up config`. Logs are written to `%APPDATA%\nightwatch\astro-up\data\logs\`. Old logs are pruned after `logging.max_age_days` (default 365).

### Reporting issues

Include with your bug report:
- The command you ran with `--verbose` output
- Your Windows version
- Which package was affected

File issues at [github.com/nightwatch-astro/astro-up/issues](https://github.com/nightwatch-astro/astro-up/issues).
