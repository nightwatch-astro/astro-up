# Regression Test Packages

Packages that should always pass the lifecycle test workflow. Use these for
smoke-testing changes to the detection pipeline or installer integration.

## Always-Pass Packages

| Package ID | Detection Method | Expected Install Path | Notes |
|------------|-----------------|----------------------|-------|
| `nina` | Registry (`NINA 2` / `DisplayVersion`) | `C:\Program Files\NINA\` | Fallback: PE file version from `NINA.exe` |
| `phd2` | Registry (`PHD2` / `DisplayVersion`) | `C:\Program Files (x86)\PHDGuiding2\` | Stable registry layout across versions |
| `ascom-platform` | Registry (`ASCOM Platform 6` / `DisplayVersion`) | `C:\Program Files\ASCOM\` | Has `product_code` GUID; version_regex `^(\d+\.\d+)` strips patch |
| `sharpcap` | Registry | `C:\Program Files\SharpCap\` | Depends on `ascom-platform` |
| `astap` | PE file / file_exists | `C:\Program Files\astap\` | Portable-style install; detection via executable presence |

## Expected Detection Methods

### Registry-based (majority of packages)

Most Windows desktop applications register in `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{DisplayName}`. The detection chain reads `DisplayVersion` and optionally applies `version_regex` to normalize the version string.

### PE file-based (fallback)

When registry detection fails or is unavailable, the PE file header of the main executable provides `FileVersion` or `ProductVersion`. Used as a fallback for NINA and similar packages.

### Driver store (hardware packages)

ASCOM drivers and camera SDKs install via `.inf` files into the Windows driver store. Detection queries WMI `Win32_PnPSignedDriver` filtered by `InfProvider` and `DeviceClass`.

## Running Regression Tests

```bash
# Test a single regression package
gh workflow run lifecycle-test.yml -f package_id=nina

# Test all regression packages (manually, one at a time)
for pkg in nina phd2 ascom-platform; do
  gh workflow run lifecycle-test.yml -f package_id=$pkg
done
```

## When to Update This List

- After adding a new detection method
- After changing the catalog schema (`detection` table columns)
- After modifying `InstallerService` post-install path resolution
- When a previously-passing package starts failing (investigate before removing)
