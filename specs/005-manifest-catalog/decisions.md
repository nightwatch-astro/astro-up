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

## Clarify-Phase Decisions

### C1: Format detection by URL extension, not content sniffing
**Finding**: Spec was ambiguous about how the client knows whether it received SQLite or JSON.
**Decision**: URL extension dictates format. Default URL ends in `.db` (SQLite). JSON URL would end in `.json`. No magic bytes detection.

### C2: File locking for concurrent cache access
**Finding**: CLI and GUI could run simultaneously, both accessing the cache.
**Decision**: Use file-level advisory locking. Second reader gets stale cache if lock is held. Simple, no shared-memory complexity.

### C3: meta.json deferred — ETag is sufficient
**Finding**: Migration plan mentions a 100-byte `meta.json` for lightweight change detection.
**Decision**: Defer. HTTP ETag on the catalog URL achieves the same goal (conditional GET) without an extra file to maintain.

### C4: Catalog bundles version info
**Finding**: In the old Go system, the client fetches both `manifests.json` and `versions.json` separately.
**Decision**: The SQLite catalog bundles everything — manifest metadata + latest version per package. One fetch, not two. The manifest repo's CI compiles both into the catalog.
