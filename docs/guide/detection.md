# Detection

Astro-Up automatically detects installed astrophotography software using 9 detection methods, ordered by reliability.

## Detection Methods

| Method | Use case | Key fields |
|--------|----------|------------|
| **Registry** | Most installed apps | `registry_key`, `registry_value` |
| **PeFile** | Apps with versioned executables | `file_path`; optional `version_regex` for binary string extraction |
| **FileExists** | Portable apps, data files | `file_path` |
| **ConfigFile** | Apps with version in config files | `file_path`, `version_regex` |
| **AscomProfile** | ASCOM drivers | Device type + ProgID |
| **Wmi** | USB/hardware drivers | `driver_provider`, `device_class`, `inf_name` |
| **DriverStore** | Driver packages | INF name |
| **WmiApps** | Win32_InstalledWin32Program matching | Name pattern matching |
| **Ledger** | Manual tracking only | Recorded from previous installs |

### Registry

The primary detection method. Astro-Up scans the standard uninstall keys (`HKLM` and `HKCU`) plus application-specific registry keys defined in the catalog. Registry entries provide the installed version, install path, and uninstall command.

### PE Header Analysis

For software without clean registry entries, Astro-Up reads PE (Portable Executable) headers from known install locations to extract `FileVersion` and `ProductVersion`. Some applications (e.g., ASTAP) embed placeholder PE versions -- for these, Astro-Up falls back to `version_regex` to extract the real version from binary strings in the executable.

### Fallback Chains

Detection configs support a `fallback` field. If the primary method does not find the package, the fallback method is tried. This handles cases where the same software installs differently depending on method (MSI vs EXE vs ZIP).

## How Scanning Works

1. For each package with a detection config, gather detection rules
2. Run the primary detection method
3. If not found, try the fallback chain
4. Parse version strings and normalize to semver
5. Compare detected version against latest catalog version

## Scan Triggers

- **GUI**: automatic on launch (configurable), or via the **Installed** view refresh button
- **CLI**: `astro-up scan`

## Detection Status

| Status | Meaning |
|--------|---------|
| **Installed** | Found and up to date |
| **Update Available** | Installed but newer version exists |
| **Not Installed** | Not found on system |
| **Acknowledged** | Manually marked as known (via Ledger) |
