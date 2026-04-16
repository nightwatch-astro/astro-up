# Adding Manifests

Software packages are defined as TOML manifests in the [astro-up-manifests](https://github.com/nightwatch-astro/astro-up-manifests) repository.

## Steps

1. Fork the [manifests repo](https://github.com/nightwatch-astro/astro-up-manifests)
2. Create `manifests/<package-id>.toml` (e.g., `nina-app.toml`)
3. Add a package icon to `assets/icons/<package-id>.png` (128x128 PNG, transparent background)
4. Submit a pull request

## Complete Example

```toml
id = "nina-app"
name = "N.I.N.A."
slug = "nina"
type = "application"
category = "capture"
publisher = "N.I.N.A. Team"
homepage = "https://nighttime-imaging.eu"
description = "Nighttime Imaging 'N' Astronomy — advanced capture sequencer."
license = "MPL-2.0"
aliases = ["NINA", "Nighttime Imaging"]
tags = ["capture", "sequencer", "imaging"]

[install]
method = "inno_setup"
scope = "user"
elevation = "required"
zip_wrapped = false
upgrade_behavior = "install"
timeout = "10m"

[install.switches]
silent = ["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"]
interactive = ["/NORESTART"]
log = "/LOG=$TEMP\\nina-install.log"

[install.known_exit_codes]
"1" = "package_in_use"
"5" = "cancelled_by_user"
"3010" = "reboot_required"

[detection]
method = "registry"
registry_key = "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\NINA 2_is1"
registry_value = "DisplayVersion"

[detection.fallback]
method = "pe_file"
file_path = "$PROGRAMFILES\\NINA\\NINA.exe"
version_regex = "^(\\d+\\.\\d+\\.\\d+)"

[checkver]
provider = "github"
github = "nightwatch-astro/nina"
asset_pattern = "NINA-.*-setup\\.exe$"

[checkver.autoupdate]
url = "https://github.com/nightwatch-astro/nina/releases/download/v$version/NINA-$version-setup.exe"

[dependencies]
requires = [{ id = "dotnet-desktop-runtime", min_version = "8.0" }]
optional = ["ascom-platform"]

[hardware]
vid_pid = []
device_class = "Camera"
```

## Field Reference

### Top-level

| Field | Required | Description |
|-------|----------|-------------|
| `id` | yes | Package ID (`vendor-product` convention, lowercase, hyphens only) |
| `name` | yes | Display name |
| `slug` | yes | URL-friendly short name |
| `type` | yes | `application`, `driver`, `runtime`, `database`, `usb_driver`, `resource` |
| `category` | yes | `capture`, `guiding`, `platesolving`, `equipment`, `focusing`, `planetarium`, `viewers`, `prerequisites`, `usb`, `driver` |
| `publisher` | no | Developer or organization |
| `homepage` | no | Official project URL |
| `description` | no | Short description |
| `license` | no | SPDX license identifier |
| `aliases` | no | Alternative names (used in search) |
| `tags` | no | Searchable tags |

### `[install]`

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | `exe`, `msi`, `inno_setup`, `nsis`, `wix`, `burn`, `zip`, `portable`, `download_only` |
| `scope` | string | `machine`, `user`, `either` |
| `elevation` | string | `required` (UAC via astro-up), `prohibited` (no elevation), `self` (installer handles UAC) |
| `zip_wrapped` | bool | Download is a zip containing the actual installer |
| `zip_inner_path` | string | Subfolder inside zip where installer lives (e.g., `"x64"`) |
| `upgrade_behavior` | string | `install` (over-install), `uninstall_previous`, `deny` |
| `timeout` | duration | Installer timeout override (default 10m, range 10s--3600s) |
| `switches.silent` | string[] | Silent install arguments |
| `switches.interactive` | string[] | Interactive install arguments |
| `switches.log` | string | Log file path template |
| `switches.install_dir` | string | Install location argument |
| `known_exit_codes` | map | Exit code to meaning: `package_in_use`, `reboot_required`, `cancelled_by_user`, `already_installed`, `missing_dependency`, `disk_full` |
| `success_codes` | int[] | Non-zero exit codes that mean success |

### `[detection]`

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | `registry`, `pe_file`, `wmi`, `wmi_apps`, `driver_store`, `ascom_profile`, `file_exists`, `config_file`, `ledger` |
| `registry_key` | string | Full registry key path |
| `registry_value` | string | Registry value name (e.g., `DisplayVersion`) |
| `file_path` | string | Path to executable or config file |
| `version_regex` | string | Regex to extract version from detected string |
| `product_code` | string | MSI product GUID |
| `upgrade_code` | string | MSI upgrade GUID |
| `inf_provider` | string | Driver INF provider name (for driver_store) |
| `device_class` | string | Driver device class |
| `inf_name` | string | Driver INF filename |
| `fallback` | object | Another `[detection]` config tried if primary fails |

### `[checkver]` / `[dependencies]` / `[hardware]`

See the [complete example](#complete-example) above. Detection configs are optional -- the [lifecycle testing workflow](./lifecycle-testing.md) discovers them automatically by installing on a Windows runner and probing for detection signatures.

## Tips

- Use `vendor-product` naming: `zwo-asi-camera-driver`, `ascom-platform`
- Every package needs an icon at `assets/icons/<package-id>.png`
- Test locally: `astro-up install my-package --dry-run`
- Detection configs are validated by CI lifecycle tests
