# Decisions Report: 005-manifest-catalog

**Created**: 2026-03-29
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: SQLite only — no JSON format
**Choice**: SQLite is the only catalog format. No JSON, no transition period.
**Reasoning**: This is a greenfield Rust project. The Go client is archived. No backward compatibility needed. SQLite gives us FTS5 search, indexed queries, and single-file distribution.
**Alternatives rejected**: JSON (no search capability, must load all into memory), dual format (unnecessary complexity)

### D2: Minisign public key baked into binary
**Choice**: Compile-time embedded key, not configurable.
**Reasoning**: Key rotation warrants a new release. Configurable keys are a supply-chain risk.

### D3: Disk file + ETag — no memory cache layer
**Choice**: The SQLite .db file on disk IS the cache. ETag conditional fetch checks for updates. No in-memory cache tier.
**Reasoning**: SQLite queries are fast enough (<10ms) that a memory cache adds complexity for no measurable gain. The file on disk is the single source of truth.
**Changed from**: Original spec had three-tier cache (memory → disk → network). Simplified per user feedback.

### D4: FTS5 full-text search
**Choice**: SQLite FTS5 for search across name, aliases, tags, description.
**Reasoning**: FTS5 is built into SQLite (bundled via rusqlite), provides ranked results, handles word boundaries ("plate" matches "PlateSolve"), and scales beyond substring matching.
**Changed from**: Original spec had substring matching. Upgraded per user feedback.

### D5: Package ID = short name (merged with slug)
**Choice**: A single `id` field that IS the short, human-friendly identifier. No separate slug. Manifest filename = ID + `.toml`.
**Reasoning**: For a curated catalog of ~100 niche astrophotography packages, name collisions are extremely unlikely. The `{vendor}-{product}` convention was defensive overkill. `nina` is better UX than `nina-app`.
**ID regex**: `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` (lowercase, hyphen-separated, 2-50 chars)
**Migration**: One-time rename of existing 96 manifests in astro-up-manifests repo (e.g., `nina-app.toml` → `nina.toml`).
**Aliases**: The `aliases` field provides search terms (e.g., `["n.i.n.a.", "nighttime-imaging"]`) but NOT resolution identifiers. `astro-up update nina` works, `astro-up update n.i.n.a.` does not (use search instead).

### D6: apt-style PID lockfile for write operations
**Choice**: `{data_dir}/astro-up/astro-up.lock` with PID. Write operations (update, install) acquire the lock. Read operations (list, check, search) proceed without locking.
**Reasoning**: Simpler than file-level locking on the SQLite file. Matches user expectation from apt/dpkg. Second instance gets "another instance is running" and exits.
**Changed from**: Original spec had file-level advisory locking on the cache file.

### D7: No explicit offline mode
**Choice**: No `--offline` flag. If the local catalog exists, it's used. If network is down and TTL expired, the local catalog is still used (it's valid data, just stale). Only error if no local catalog AND no network.
**Reasoning**: Offline mode is implicit — the local SQLite file IS offline capability. Network is only needed for initial fetch and periodic refresh. Users can't download installers offline anyway.

### D8: meta.json dropped entirely
**Choice**: No meta.json for lightweight change detection.
**Reasoning**: ETag on the catalog URL achieves the same goal with one fewer file. Dropped, not deferred.

## Questions I Would Have Asked

### Q1: Should the catalog support multiple sources (mirrors)?
**My decision**: No. Single source URL. If GitHub is down, local catalog suffices.

### Q2: How to handle catalog schema versioning?
**My decision**: Version field in SQLite metadata table. Client rejects catalogs with unsupported schema versions. Backward-compatible additions don't bump the version.

### Q3: Should the manifest rename be automated or manual?
**My decision**: Automated script in the manifests repo. One PR renaming all 96 files, updating cross-references. Old filenames don't persist as aliases — clean break.
