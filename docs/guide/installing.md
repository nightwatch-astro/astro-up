# Installing & Updating

## Installing Software

### From the GUI

1. Navigate to **Catalog** and find the package
2. Click the package to open its detail page
3. Click **Install** -- Astro-Up downloads, verifies, and runs the installer
4. For packages with multiple assets (e.g., 32-bit vs 64-bit), an asset selection dialog appears

### From the CLI

```sh
# Install a package
astro-up install nina-app

# Force reinstall an already-installed package
astro-up install nina-app --force
```

## Install Pipeline

Each install goes through these steps:

1. **Download** -- fetch the installer with progress tracking and SHA-256 verification
2. **Elevate** -- request administrator privileges via UAC if needed
3. **Execute** -- run the installer silently by default
4. **Verify** -- run detection to confirm the package is installed
5. **Record** -- store install path in the ledger for future updates

## Updating

```sh
# Update a specific package
astro-up update nina-app

# Update all outdated packages (queued sequentially)
astro-up update --all
```

In the GUI, the **Installed** view shows an update badge next to outdated packages. Click **Update** on the detail page, or update multiple packages -- they queue and run sequentially.

## Supported Install Methods

| Method | Description |
|--------|-------------|
| `exe` | Generic EXE with custom silent args |
| `msi` | Windows Installer (`/qn /norestart`) |
| `inno_setup` | InnoSetup installers (`/VERYSILENT /NORESTART`) |
| `nsis` | NSIS installers (`/S`) |
| `wix` | WiX-based installers |
| `burn` | WiX Burn bootstrappers (with Job Object process tree tracking) |
| `zip` | Extract to target directory (supports `zip_wrapped` inner installers) |
| `portable` | Extract and create a Windows shortcut in the portable apps folder |
| `download_only` | Download to the portable apps directory without running an installer |

## Elevation (UAC)

Astro-Up handles Windows administrator privileges automatically:

- **Proactive** -- if the manifest declares elevation is required, UAC is requested upfront
- **Reactive** -- if an installer returns exit code 740 or `E_ACCESSDENIED`, Astro-Up retries with elevation

Elevation uses `ShellExecuteExW runas` for standard UAC prompts.

## Download Management

- Downloads are saved to a configurable directory
- Partial downloads resume automatically via HTTP Range requests
- SHA-256 checksums are verified after download
- Speed throttling is configurable in **Settings > Network**
