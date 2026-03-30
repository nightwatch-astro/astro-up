# Decisions Report: 020-manifest-modernization

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: SQLite replaces JSON as the distribution format
**Choice**: catalog.db (SQLite) is the only published artifact. No manifests.json or versions.json.
**Reasoning**: SQLite enables FTS5 search, indexed queries, single-file distribution. JSON requires loading everything into memory.

### D2: Per-version files instead of monolithic versions.json
**Choice**: `versions/{id}/{semver}.json` individual files.
**Reasoning**: Cleaner git history, natural deduplication (file existence = version known), simpler CI.

### D3: Rolling GitHub Release tag for distribution
**Choice**: `catalog/latest` tag updated on each pipeline run.
**Reasoning**: Simple, CDN-cached via GitHub, no custom infrastructure.

### D4: Single CI job for full pipeline
**Choice**: One job: check → compile → sign → publish. No multi-stage.
**Reasoning**: Manifest repo is small enough. Multi-stage adds inter-job artifact passing overhead.

## Clarify-Phase Decisions (Interactive)

### C1: Remove committed build artifacts from git
**Finding**: User asked if we still need versions.json/manifests.json.
**Decision**: No — remove them from git entirely. They're build artifacts, not source. Add to .gitignore. The repo contains only:
- Source: `manifests/*.toml` + `versions/{id}/{semver}.json`
- Build artifacts go to GitHub Releases only: `catalog.db`, `catalog.db.minisig`, `stats.json`
**Reasoning**: Build artifacts in git cause merge conflicts, bloat the repo, and confuse source-of-truth. The Go client that consumed them is archived.

### C2: Manifest ID rename is part of this spec
**Finding**: User confirmed rename should be in this spec, not separate.
**Decision**: One-time rename of all 96 manifests from `{vendor}-{product}` to short IDs. Per-version directories also renamed. No old IDs in aliases (clean break). CI validates uniqueness.

### C3: stats.json published to GitHub Releases for docs site
**Finding**: The docs site (astro-up.github.io) currently fetches stats.json from raw.githubusercontent.com.
**Decision**: stats.json moves to GitHub Releases alongside catalog.db. Docs site updated to fetch from there. Same rolling `catalog/latest` tag.

### C4: No backward compatibility — clean break
**Decision**: Go client is archived. No dual JSON+SQLite output. No old manifest IDs. One format, one set of IDs.

### C5: version_format field in catalog for client-side comparison
**Decision**: The packages table includes `version_format` (from the manifest TOML). The client (spec 012) uses this to select the correct version parser (semver, date, or custom regex).

### C6: Catalog includes full manifest as JSON column
**Decision**: The `packages.manifest` column stores the full manifest as JSON. This lets the client access detection config, install config, checkver config, etc. without a separate fetch. One table, one query.

## Questions I Would Have Asked

### Q1: Should per-version files include changelog/release notes URL?
**My decision**: Yes — `{url, sha256, discovered_at, release_notes_url}`. The release notes URL feeds into the client's "what's new" display. Optional field.

### Q2: Should the catalog include ALL historical versions or only latest?
**My decision**: Latest only in catalog.db (one row per package in versions table with the newest). Historical versions remain as per-version files in git. Keeps the catalog small. Client only needs latest for update checking.

### Q3: Should the rename be automated or manual?
**My decision**: Automated script. Generate the rename mapping from a CSV (old_id → new_id), rename files + directories, update any cross-references, validate with the compiler. One PR.
