# Lifecycle Testing

Astro-Up includes an automated workflow that downloads, installs, probes, and uninstalls packages on a Windows runner. It discovers detection signatures and install metadata, producing a `[detection]` TOML config that gets added to the manifest via PR.

## Why

The version checker pipeline discovers versions by scraping web pages or following redirects — it never downloads or installs the actual software. This means it cannot discover install locations, registry keys, or file paths needed for detection. The lifecycle testing workflow fills that gap by performing actual installs on Windows.

It also serves as an install/uninstall regression test for every package in the catalog.

## How It Works

The workflow runs on `windows-latest` GitHub Actions runners and executes 7 phases per package:

### 1. Download
Fetch the installer using the manifest's `[checkver.autoupdate]` URL with `$version` substituted. Verify the file exists and record SHA-256.

### 2. Install
Run the installer using the manifest's `[install]` method and silent switches. Handle elevation (GitHub Actions Windows runners run as admin by default).

### 3. Detect
Probe the system for detection signatures using all 8 strategies in order of reliability:

1. **Registry scan** — Search `HKLM\SOFTWARE`, `HKCU\SOFTWARE`, and WOW6432Node for matching entries
2. **PE file version** — Find installed `.exe` files and read `VS_FIXEDFILEINFO`
3. **File existence** — Check known install locations and PATH
4. **Config file** — Find version in config/manifest files via regex
5. **ASCOM registration** — Check `HKLM\SOFTWARE\ASCOM\{DeviceType}` for driver entries
6. **WMI driver query** — Query `Win32_PnPSignedDriver` by provider, class, INF name
7. **Driver store** — Check the Windows driver store
8. **MSI product code** — Query Uninstall keys for MSI-registered packages

### 4. Verify Install
Confirm the best detection method correctly reports "installed" with the expected version.

### 5. Uninstall
Run the uninstaller (discovered from registry `UninstallString` or `QuietUninstallString`).

### 6. Verify Removal
Re-run detection to confirm the package is no longer found. Flag leftover files or registry keys.

### 7. Report
Output pass/fail for each phase, the discovered TOML config, and all probed locations for debugging.

## Output

For each package, the workflow produces:

- A `[detection]` TOML section with the best-fit method and fallback chain
- Phase-by-phase pass/fail report with logs
- All probed locations and results for debugging
- An auto-created PR adding the detection config to the manifest

## Detection Config Fields

The discovered config maps to the Rust app's `DetectionConfig`:

| Field | Purpose | Used by |
|-------|---------|---------|
| `file_path` | Path template with `{program_files}` tokens | PeFile, FileExists, ConfigFile |
| `version_regex` | Regex to extract version from config files | ConfigFile |
| `product_code` | MSI product GUID | MSI detection |
| `upgrade_code` | MSI upgrade GUID | MSI detection |
| `driver_provider` | WMI DriverProviderName filter | Wmi |
| `device_class` | WMI DeviceClass filter | Wmi |
| `inf_name` | WMI InfName filter | Wmi |
| `fallback` | Recursive DetectionConfig | All methods (chain pattern) |

## Workflow Modes

- **Single package**: `workflow_dispatch` with package ID — runs the full lifecycle for one package
- **Matrix sweep**: Scheduled or manual trigger for all packages missing detection configs. Uses concurrency limits to avoid overwhelming the runner.
- **Dry run**: Download and probe only, skip install/uninstall — useful for portable apps and data files

## Non-Installed Packages

For portable apps, firmware files, and data (star databases, framing caches):

- Single binaries: detection method is `FileExists` with the expected path
- ZIP-extracted tools: check common extract paths and PATH
- Firmware/data: marked as `not_applicable` with a note — skip uninstall phase

## Constraints

- Must clean up after itself (uninstall even if probing fails)
- Must handle elevation (some packages need admin)
- Works for all install methods: `inno_setup`, `nsis`, `msi`, `exe`, `zip`, `zip_wrap`, `download_only`, `portable`
- Handles install failures gracefully (reports "detection unknown", doesn't fail the workflow)
- Idempotent — running twice produces the same result
- 10 minute timeout per package
- No shared state between matrix jobs
