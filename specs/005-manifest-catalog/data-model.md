# Data Model: 005-manifest-catalog

**Date**: 2026-03-30

## Entities

### PackageId (newtype)

Validated string wrapping the package identifier.

| Field | Type | Constraints |
|-------|------|-------------|
| inner | `String` | Regex: `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`, length 2-50 |

Implements: `Display`, `FromStr`, `Serialize`, `Deserialize`, `Hash`, `Eq`, `Ord`, `AsRef<str>`

### PackageSummary (read from `packages` table)

Lightweight struct for catalog queries — does NOT include operational fields (detection, install, etc.).

| Field | Type | Source Column | Notes |
|-------|------|---------------|-------|
| id | `PackageId` | `id` | Primary key |
| name | `String` | `name` | Display name |
| slug | `String` | `slug` | Display-friendly label (e.g., "N.I.N.A.") |
| description | `Option<String>` | `description` | |
| publisher | `Option<String>` | `publisher` | |
| homepage | `Option<String>` | `homepage` | |
| category | `Category` | `category` | Enum from spec 003 |
| software_type | `SoftwareType` | `type` | Enum from spec 003 |
| license | `Option<String>` | `license` | |
| aliases | `Vec<String>` | `aliases` | JSON-decoded from TEXT |
| tags | `Vec<String>` | `tags` | JSON-decoded from TEXT |
| dependencies | `Vec<String>` | `dependencies` | JSON-decoded from TEXT |
| manifest_version | `u32` | `manifest_version` | |

### VersionEntry (read from `versions` table)

| Field | Type | Source Column | Notes |
|-------|------|---------------|-------|
| package_id | `PackageId` | `package_id` | FK to packages |
| version | `String` | `version` | Version string (not necessarily semver) |
| url | `String` | `url` | Download URL |
| sha256 | `Option<String>` | `sha256` | Hash for integrity check |
| discovered_at | `DateTime<Utc>` | `discovered_at` | RFC 3339 string in SQLite |
| release_notes_url | `Option<String>` | `release_notes_url` | |
| pre_release | `bool` | `pre_release` | 0/1 integer in SQLite |

### CatalogMeta (read from `meta` table)

| Field | Type | Source Key | Notes |
|-------|------|-----------|-------|
| schema_version | `String` | `schema_version` | Must match `SUPPORTED_SCHEMA` |
| compiled_at | `DateTime<Utc>` | `compiled_at` | RFC 3339 |

### CatalogSidecar (JSON file: `catalog.db.meta`)

| Field | Type | Notes |
|-------|------|-------|
| etag | `Option<String>` | From HTTP `ETag` header |
| fetched_at | `DateTime<Utc>` | When catalog was last fetched |

### PidLock

| Field | Type | Notes |
|-------|------|-------|
| path | `PathBuf` | `{data_dir}/astro-up/astro-up.lock` |
| pid | `u32` | Current process ID |

Acquires on creation, releases on `Drop`. Checks for stale locks (dead PIDs).

## Relationships

```
PackageSummary 1──* VersionEntry  (package_id FK)
CatalogMeta    1──1 Catalog       (singleton in meta table)
CatalogSidecar 1──1 Catalog       (companion file on disk)
PidLock        1──1 Catalog       (guards write operations)
```

## State Transitions

### Catalog Lifecycle

```
NoLocalCatalog ──[fetch+verify]──> Valid
Valid ──[TTL expired]──> Stale
Stale ──[fetch 304]──> Valid (reset fetched_at)
Stale ──[fetch 200+verify]──> Valid (new file)
Stale ──[fetch fail]──> Stale (use as-is)
Valid ──[schema mismatch]──> Rejected (prompt update)
Valid ──[sig invalid]──> Rejected (keep previous)
NoLocalCatalog ──[fetch fail]──> Error (no catalog available)
```
