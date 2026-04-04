# Installing & Updating

## Installing Software

### From the GUI

1. Navigate to **Catalog** and find the package
2. Click the package to open its detail page
3. Click **Install** — Astro-Up downloads, verifies, and runs the installer

### From the CLI

```sh
# Install a specific package
astro-up install nina-app

# Install with interactive installer (shows installer UI)
astro-up install nina-app --interactive
```

## Install Pipeline

Each install goes through these steps:

1. **Plan** — resolve dependencies and determine install order
2. **Download** — fetch the installer with progress tracking and SHA-256 verification
3. **Backup** — optionally back up existing settings (if already installed)
4. **Execute** — run the installer (silent by default)
5. **Verify** — run detection to confirm the package is installed
6. **Record** — store install path in the ledger for future backup/uninstall

## Updating

### Check for Updates

```sh
# Check all installed packages
astro-up check

# Update a specific package
astro-up update nina-app

# Update all
astro-up update --all
```

### GUI

The **Installed** view shows an update badge next to packages with newer versions. Click **Update** on the detail page to start the update pipeline.

## Supported Install Methods

| Method | Description |
|--------|-------------|
| `inno_setup` | InnoSetup installers (`/VERYSILENT /NORESTART`) |
| `nsis` | NSIS installers (`/S`) |
| `msi` | Windows Installer (`/qn /norestart`) |
| `exe` | Generic EXE with custom silent args |
| `zip` | Extract to target directory |
| `zip_wrap` | ZIP containing an installer |
| `download_only` | Download without install (manual) |
| `portable` | Single binary, no install needed |

## Silent vs Interactive

By default, installers run silently (no UI). Change the default in **Settings > General > Install Method**, or use `--interactive` on the CLI.

## Download Management

- Downloads are saved to a configurable directory
- Partial downloads resume automatically via HTTP Range requests
- SHA-256 checksums are verified after download
- Speed throttling is configurable in **Settings > Network**
