# Detection

Astro-Up automatically detects installed astrophotography software using multiple strategies, ordered by reliability.

## Detection Methods

| Method | Use case | Fields used |
|--------|----------|-------------|
| **Registry** | Most installed apps | `registry_key`, `registry_value` |
| **PeFile** | Apps with versioned executables | `file_path` |
| **FileExists** | Portable apps, data files | `file_path` |
| **ConfigFile** | Apps with version in config | `file_path`, `version_regex` |
| **AscomProfile** | ASCOM drivers | Device type + ProgID |
| **Wmi** | USB/hardware drivers | `driver_provider`, `device_class`, `inf_name` |
| **DriverStore** | Driver packages | INF name |
| **MSI** | MSI-installed packages | `product_code`, `upgrade_code` |

### Windows Registry

The primary detection method. Astro-Up scans:

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`
- `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`
- Application-specific keys defined in the catalog

Registry entries provide the installed version, install path, and uninstall command.

### PE Header Analysis

For software without clean registry entries, Astro-Up reads PE (Portable Executable) headers from known install locations to extract `FileVersion` and `ProductVersion` from the `VS_FIXEDFILEINFO` resource.

### Fallback Chains

Detection configs support a `fallback` field — if the primary method doesn't find the package, the fallback method is tried. This handles cases where the same software installs differently depending on method (MSI vs EXE vs ZIP).

## How Scanning Works

1. For each package with a detection config, gather detection rules
2. Run the primary detection method
3. If not found, try the fallback chain
4. Parse version strings and normalize to semver
5. Compare detected version against latest catalog version

## Scan Triggers

- **GUI**: automatic on launch (if enabled), or via the Installed view's refresh button
- **CLI**: `astro-up scan`

## Detection Status

| Status | Meaning |
|--------|---------|
| **Installed** | Found and up to date |
| **Update Available** | Installed but newer version exists |
| **Not Installed** | Not found on system |
| **Acknowledged** | Manually marked as known |

## Populating Detection Configs

Detection configs are discovered automatically by the [lifecycle testing workflow](./lifecycle-testing.md), which installs each package on a Windows runner and probes for detection signatures.
