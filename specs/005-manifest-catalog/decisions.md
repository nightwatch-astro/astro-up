# Decisions Report: 005-manifest-catalog

**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: SQLite as primary catalog format (JSON during transition)
**Choice**: SQLite with signed JSON as fallback
**Reasoning**: SQLite enables FTS5 search, category indexing, and efficient queries without loading everything into memory. JSON remains for backward compatibility during Go→Rust transition.
**Alternatives**: JSON-only (simpler but poor search), protobuf (overkill)

### D2: Minisign public key baked into binary
**Choice**: Compile-time embedded key, not configurable
**Reasoning**: Key rotation is a rare event that warrants a new release. Runtime-configurable keys are a supply-chain risk — an attacker who can modify config can bypass signature checks.
**Alternatives**: Configurable key (flexible but insecure), multiple keys with rotation (complex)

### D3: Three-tier cache: memory → disk → network
**Choice**: TTL memory cache, disk file cache with ETag, network fetch
**Reasoning**: Memory cache for hot paths (sub-millisecond), disk for cold starts (50ms), network only when stale. ETag avoids re-downloading unchanged catalogs.

### D4: Substring match for fuzzy search, not Levenshtein
**Choice**: Case-insensitive substring matching across name, aliases, and tags
**Reasoning**: For ~100 software entries, substring matching is fast and predictable. Levenshtein adds complexity for minimal UX gain. FTS5 in SQLite provides better search when needed.

## Questions I Would Have Asked

### Q1: Should the catalog support multiple sources (mirrors)?
**My decision**: No. Single source URL. Mirrors add complexity (consistency, freshness). If the primary is down, stale cache suffices.

### Q2: Should slug resolution be exact or fuzzy?
**My decision**: Exact match on slug, fuzzy only on name search. Slugs are stable identifiers — fuzzy matching could return wrong packages.

### Q3: How to handle catalog schema versioning?
**My decision**: Version field in catalog metadata. Client rejects catalogs with unsupported schema versions. Backward-compatible additions don't bump the version.
