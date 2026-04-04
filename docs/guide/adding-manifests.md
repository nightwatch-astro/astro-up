# Adding Manifests

Software packages are defined as TOML manifests in the [astro-up-manifests](https://github.com/nightwatch-astro/astro-up-manifests) repository. Here's how to add a new package.

## Steps

1. Fork the [manifests repo](https://github.com/nightwatch-astro/astro-up-manifests)
2. Create a new file in `manifests/` named `<package-id>.toml` (e.g., `nina-app.toml`)
3. Fill in the manifest fields (see template below)
4. Add a package icon to `assets/icons/<package-id>.png` (128x128 PNG, transparent background). If no official icon is available, use a placeholder — but the file must exist.
5. Submit a pull request

## Manifest Template

```toml
[package]
id = "my-package"
name = "My Package"
category = "capture"
publisher = "Publisher Name"
website = "https://example.com"
description = "Short description of the software."
license = "MIT"

[checkver]
# How to discover the latest version
github = "owner/repo"
# Or: url = "https://example.com/download"
#     regex = "Version ([\\d.]+)"

[autoupdate]
url = "https://example.com/downloads/my-package-$version-setup.exe"

[install]
method = "inno_setup"
quiet_args = ["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"]
interactive_args = ["/NORESTART"]

[detection]
method = "Registry"
registry_key = "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\My Package_is1"
registry_value = "DisplayVersion"

[backup]
config_paths = [
  "$LOCALAPPDATA/MyPackage/settings.json",
  "$LOCALAPPDATA/MyPackage/profiles/",
]
```

## Finding Information

- **Install method**: Run the installer with `/?` to see flags. See the [Silent Installers Guide](./silent-installers.md).
- **Registry key**: Install the software, then check `regedit` under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`.
- **Detection config**: You can skip this — the [lifecycle testing workflow](./lifecycle-testing.md) discovers detection configs automatically by installing on a Windows runner.

## Tips

- Use the `vendor-product` naming convention for IDs (e.g., `zwo-asi-camera-driver`)
- Every package must have an icon in `assets/icons/<package-id>.png` — the GUI uses it in the catalog and detail views
- Test your manifest locally before submitting: `astro-up install my-package --dry-run`
- Detection configs are validated against the actual install by CI
