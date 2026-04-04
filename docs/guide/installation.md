# Installation

## Windows Installer

Download the latest release from [GitHub Releases](https://github.com/nightwatch-astro/astro-up/releases/latest):

- **`Astro-Up_x.y.z_x64-setup.exe`** — for Intel/AMD (most PCs)

The installer:
- Installs to `%LOCALAPPDATA%\Programs\astro-up\`
- Adds Astro-Up to your PATH
- Creates a Start Menu shortcut
- Sets up system tray integration

## CLI Only

If you prefer just the command line:

```sh
cargo install astro-up-cli
```

## System Requirements

- **OS**: Windows 10 or Windows 11
- **Architecture**: x64 (Intel/AMD)
- **Disk**: ~20 MB for the app, additional space for downloaded installers

## Verify the Download

Each release includes SHA-256 checksums. Verify your download:

```powershell
(Get-FileHash .\Astro-Up_1.0.0_x64-setup.exe).Hash
```

## Updating

Astro-Up can update itself:

- **GUI**: You'll see a banner when an update is available — click Install
- **CLI**: `astro-up self-update`

## Uninstall

Settings > Apps > Astro-Up > Uninstall, or run the uninstaller from the Start Menu.
