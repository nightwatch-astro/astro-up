# Catalog Format

The Astro-Up catalog is a pre-built SQLite database distributed via GitHub Releases. This page documents the catalog structure for contributors and advanced users.

## Distribution

The catalog is maintained in the [astro-up-manifests](https://github.com/nightwatch-astro/astro-up-manifests) repository and published as `catalog.db` in GitHub Releases.

## Database Schema

### `packages`

The main package table.

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | Unique package identifier (e.g., `nina-app`) |
| `name` | TEXT | Display name |
| `category` | TEXT | Package category |
| `publisher` | TEXT | Developer or organization |
| `website` | TEXT | Official project URL |
| `description` | TEXT | Short description |
| `license` | TEXT | License identifier |

### `detection_configs`

How to detect if a package is installed.

| Column | Type | Description |
|--------|------|-------------|
| `package_id` | TEXT FK | References `packages.id` |
| `method` | TEXT | Detection method: `registry`, `pe_header`, `known_path` |
| `config` | TEXT (JSON) | Method-specific detection parameters |

### `versions`

Known versions with download information.

| Column | Type | Description |
|--------|------|-------------|
| `package_id` | TEXT FK | References `packages.id` |
| `version` | TEXT | Version string (semver when possible) |
| `download_url` | TEXT | Installer download URL |
| `sha256` | TEXT | SHA-256 checksum of the installer |
| `release_date` | TEXT | ISO 8601 release date |
| `installer_type` | TEXT | `exe`, `msi`, `zip` |
| `silent_args` | TEXT | Arguments for silent installation |

## Contributing

To add a new package to the catalog, see the [astro-up-manifests](https://github.com/nightwatch-astro/astro-up-manifests) repository for contribution guidelines.
