# Silent Installer Guide

When adding a manifest, you need to specify how the installer runs silently. This guide covers common patterns.

## Identifying the Installer Type

Right-click the installer > Properties > Details. Look for:
- **InnoSetup**: Mentions "Inno Setup" in the description, or leaves `unins000.exe` after install
- **NSIS**: Has an NSIS icon or mentions "Nullsoft" in properties
- **MSI**: File extension is `.msi`
- **ZIP**: Just an archive to extract

You can also run the installer with `/?` or `/help` to see available flags.

## Common Silent Arguments

### InnoSetup (most astrophotography software)

```toml
[install]
method = "inno_setup"
quiet_args = ["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"]
interactive_args = ["/NORESTART"]
```

`/VERYSILENT` hides the installer completely. `/SUPPRESSMSGBOXES` prevents dialog popups.

### NSIS

```toml
[install]
method = "nsis"
quiet_args = ["/S"]
```

NSIS uses `/S` (capital S) for silent mode.

### MSI

```toml
[install]
method = "msi"
quiet_args = ["/qn", "/norestart"]
```

Standard Windows Installer flags. `/qn` means no UI, `/norestart` prevents automatic reboot.

### ZIP (extract only)

```toml
[install]
method = "zip"
install_dir = "{program_files}/SoftwareName"
```

No silent args needed — Astro-Up extracts to the target directory.

## Testing Silent Install

Before submitting a manifest, test on a Windows machine:

```powershell
# InnoSetup
.\setup.exe /VERYSILENT /NORESTART /SUPPRESSMSGBOXES

# NSIS
.\setup.exe /S

# MSI
msiexup /i installer.msi /qn /norestart
```

Verify the software installed correctly and check the registry for version detection.

## When Silent Install Isn't Possible

Some vendors don't support silent installation. In these cases, set `manual_download = true` in the manifest and provide a `download_page` URL.
