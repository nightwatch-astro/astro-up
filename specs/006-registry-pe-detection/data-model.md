# Data Model: 006-registry-pe-detection

**Date**: 2026-03-30

## Entities

### DetectionResult

Outcome of detecting a single package.

| Field | Type | Description |
|-------|------|-------------|
| variant | enum | Installed, InstalledUnknownVersion, NotInstalled, Unavailable |
| version | Option\<Version\> | Present only for Installed variant (spec 003 Version type) |
| method | DetectionMethod | Which method produced this result |
| reason | Option\<String\> | Present only for Unavailable variant (diagnostic message) |

**Variants**:
- `Installed(Version)` — package found, version extracted
- `InstalledUnknownVersion` — package found (registry key exists, file present) but version couldn't be parsed
- `NotInstalled` — all chain methods exhausted, nothing found
- `Unavailable(String)` — detection couldn't run (non-Windows, WMI timeout, permission denied)

### DetectionMethod (enum)

| Variant | Description | Platform |
|---------|-------------|----------|
| Registry | Windows uninstall registry keys | Windows |
| PeFile | PE file header version info | Cross-platform |
| Wmi | WMI Win32_PnPSignedDriver query | Windows |
| DriverStore | WMI driver store query | Windows |
| AscomProfile | ASCOM Profile registry keys | Windows |
| FileExists | File presence check (no version) | Cross-platform |
| ConfigFile | Version from config/manifest file | Cross-platform |

### ScanResult

Outcome of a full catalog scan.

| Field | Type | Description |
|-------|------|-------------|
| results | Vec\<PackageDetection\> | Per-package detection outcomes |
| errors | Vec\<ScanError\> | Per-package errors (non-fatal) |
| duration | Duration | Total scan time |
| scanned_at | DateTime\<Utc\> | Scan timestamp |

### PackageDetection

| Field | Type | Description |
|-------|------|-------------|
| package_id | PackageId | From catalog (spec 003/005) |
| result | DetectionResult | Detection outcome |

### ScanError

| Field | Type | Description |
|-------|------|-------------|
| package_id | PackageId | Which package failed |
| method | DetectionMethod | Which method failed |
| error | String | Human-readable error message |

### DetectionCache

In-memory cache with event-driven invalidation.

| Field | Type | Description |
|-------|------|-------------|
| entries | HashMap\<PackageId, CacheEntry\> | Cached results |

### CacheEntry

| Field | Type | Description |
|-------|------|-------------|
| result | DetectionResult | Cached detection result |
| scanned_at | DateTime\<Utc\> | When this entry was created |

**Invalidation rules**:
- Per-package: on install/update of that package
- Full: on explicit `scan` command
- No TTL — entries live until invalidated

### PathToken (enum)

| Variant | Windows expansion | Non-Windows |
|---------|-------------------|-------------|
| ProgramFiles | `%ProgramFiles%` | Unavailable |
| ProgramFilesX86 | `%ProgramFiles(x86)%` | Unavailable |
| AppData | `%APPDATA%` | `$XDG_CONFIG_HOME` or `~/.config` |
| LocalAppData | `%LOCALAPPDATA%` | `$XDG_DATA_HOME` or `~/.local/share` |
| CommonAppData | `%ProgramData%` | `/etc` |
| UserHome | `%USERPROFILE%` | `$HOME` |

### WmiDriverInfo

Result of WMI driver query (Windows-only).

| Field | Type | Description |
|-------|------|-------------|
| driver_provider | String | DriverProviderName from WMI |
| device_class | String | DeviceClass from WMI |
| inf_name | String | InfName from WMI |
| driver_version | String | DriverVersion from WMI |
| device_id | String | DeviceID from WMI |

### VidPid

USB Vendor ID : Product ID pair for hardware matching.

| Field | Type | Description |
|-------|------|-------------|
| vendor_id | u16 | USB Vendor ID (e.g., 0x03C3 for ZWO) |
| product_id | Option\<u16\> | USB Product ID, None = wildcard |

**Matching**: `03C3:*` → `VidPid { vendor_id: 0x03C3, product_id: None }` matches any ZWO device.

### HardwareMatch

| Field | Type | Description |
|-------|------|-------------|
| vid_pid | VidPid | Matched pattern |
| device_name | String | Device name from WMI/system |
| suggested_package | PackageId | Manifest package that declares this hardware |
| already_managed | bool | Whether this package is already detected/in ledger |

## Relationships

```
CatalogReader::list_all() → Vec<PackageSummary>
    ↓ (for each package with detection config)
Software.detection → DetectionConfig
    ↓ (chain: fallback linked list)
DetectionConfig.method → DetectionMethod
DetectionConfig.fallback → Option<DetectionConfig>
    ↓ (execute chain, stop at first success)
DetectionResult
    ↓ (if Installed)
LedgerEntry { source: Acknowledged, version, package_id }
    ↓ (diff against previous scan)
Insert new / Update changed / Remove gone
```

## State Transitions

### Package lifecycle (detection perspective)

```
Unknown → [scan finds it] → Detected (Acknowledged in ledger)
Detected → [next scan still finds it] → Detected (version may update)
Detected → [next scan doesn't find it] → Removed from ledger
Detected → [user installs via astro-up] → Managed (AstroUp in ledger)
Managed → [next scan doesn't find it] → Managed stays (AstroUp entries not auto-removed)
```

Note: Only `Acknowledged` entries are auto-removed on NotInstalled. `AstroUp` entries persist (user explicitly installed — absence is an error condition, not normal).
