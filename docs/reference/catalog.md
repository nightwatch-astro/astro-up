# Catalog Format

The catalog is a pre-compiled SQLite database containing all package definitions, detection configs, install configs, and version history. Distributed via GitHub Releases and signed with [minisign](https://jedisct1.github.io/minisign/).

## Pipeline

```
TOML manifests (astro-up-manifests repo)
  -> Version checker scrapes latest versions from GitHub/websites
  -> Compiler builds catalog.db from manifests + discovered versions
  -> Signed with minisign
  -> Published as GitHub Release artifact
  -> Fetched by astro-up at runtime (ETag caching, TTL-based refresh)
```

## Database Tables

### `meta`

| Column | Type | Description |
|--------|------|-------------|
| `key` | TEXT PK | Metadata key (`schema_version`, `compiled_at`) |
| `value` | TEXT | Metadata value |

### `packages`

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | Package ID (e.g., `nina-app`) |
| `manifest_version` | INTEGER | Manifest schema version |
| `name` | TEXT | Display name |
| `description` | TEXT | Short description |
| `publisher` | TEXT | Developer or organization |
| `homepage` | TEXT | Official project URL |
| `category` | TEXT | `capture`, `guiding`, `platesolving`, `equipment`, `focusing`, `planetarium`, `viewers`, `prerequisites`, `usb`, `driver` |
| `type` | TEXT | `application`, `driver`, `runtime`, `database`, `usb_driver`, `resource` |
| `slug` | TEXT | URL-friendly short name |
| `license` | TEXT | License identifier |
| `tags` | TEXT (JSON) | Searchable tags array |
| `aliases` | TEXT (JSON) | Alternative names array |
| `dependencies` | TEXT (JSON) | Required package IDs array |
| `icon_base64` | TEXT | Base64-encoded package icon |

### `packages_fts`

FTS5 virtual table for full-text search across name, description, tags, aliases, publisher.

### `versions`

| Column | Type | Description |
|--------|------|-------------|
| `package_id` | TEXT FK | References `packages.id` |
| `version` | TEXT | Version string |
| `url` | TEXT | Primary download URL |
| `sha256` | TEXT | SHA-256 checksum |
| `discovered_at` | TEXT | ISO 8601 discovery timestamp |
| `release_notes_url` | TEXT | Changelog URL |
| `pre_release` | INTEGER | 1 if pre-release |
| `assets` | TEXT (JSON) | Array of `{name, url, size}` for multi-asset releases |

### `detection`

| Column | Type | Description |
|--------|------|-------------|
| `package_id` | TEXT FK | References `packages.id` |
| `method` | TEXT | `registry`, `pe_file`, `wmi`, `wmi_apps`, `driver_store`, `ascom_profile`, `file_exists`, `config_file`, `ledger` |
| `file_path` | TEXT | PE/file path for file-based detection |
| `registry_key` | TEXT | Registry key path |
| `registry_value` | TEXT | Registry value name |
| `version_regex` | TEXT | Regex for extracting version from text |
| `product_code` | TEXT | MSI product GUID |
| `upgrade_code` | TEXT | MSI upgrade GUID |
| `inf_provider` | TEXT | Driver INF provider name |
| `device_class` | TEXT | Driver device class |
| `inf_name` | TEXT | Driver INF filename |
| `fallback_config` | TEXT (JSON) | Recursive `DetectionConfig` for fallback chain |

### `install`

| Column | Type | Description |
|--------|------|-------------|
| `package_id` | TEXT FK | References `packages.id` |
| `method` | TEXT | `exe`, `msi`, `inno_setup`, `nsis`, `wix`, `burn`, `zip`, `portable`, `download_only` |
| `scope` | TEXT | `machine`, `user`, `either` |
| `elevation` | TEXT | `required`, `prohibited`, `self` |
| `switches` | TEXT (JSON) | `{silent, interactive, log, install_dir}` |
| `exit_codes` | TEXT (JSON) | Map of exit code to known meaning |
| `success_codes` | TEXT (JSON) | Array of non-zero success codes |
| `zip_wrapped` | INTEGER | 1 if download is a zip containing the installer |
| `zip_inner_path` | TEXT | Subfolder inside zip to find installer |

## Source Repository

TOML manifests are maintained in [nightwatch-astro/astro-up-manifests](https://github.com/nightwatch-astro/astro-up-manifests). See [Adding Manifests](/guide/adding-manifests) for the manifest format.
