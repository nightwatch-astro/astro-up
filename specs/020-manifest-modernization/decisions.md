# Decisions Report: 020-manifest-modernization
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: SQLite replaces JSON as primary catalog format
**Choice**: catalog.db (SQLite) is the primary artifact. JSON is generated for backward compat during transition only.
**Reasoning**: SQLite enables FTS5 search, indexed queries, and efficient partial reads. JSON requires loading everything into memory. The Rust client (spec 005) benefits from SQLite.

### D2: Per-version files instead of monolithic versions.json
**Choice**: `versions/{id}/{semver}.json` individual files.
**Reasoning**: Cleaner git history (one commit per discovery), easier CI (only process changed files), and natural deduplication (file existence = version known).

### D3: Keep [checkver] in compiled catalog
**Choice**: Don't strip `[checkver]` during compilation (unlike the old `[remote]` which was stripped).
**Reasoning**: Self-describing manifests allow the client to do its own version checking. Stripping checkver would force all checking through the manifest repo CI.

### D4: Rolling GitHub Release tag for distribution
**Choice**: `catalog/latest` tag updated on each pipeline run with new catalog.db.
**Reasoning**: Simple, CDN-cached (GitHub's CDN), no custom infrastructure. Upgrade path to Cloudflare R2 if custom cache headers are needed later.

## Clarify-Phase Decisions

### C1: Single CI job, not multi-stage pipeline
**Decision**: One job does: check versions → compile → sign → publish. Multi-stage adds inter-job artifact passing overhead. The manifest repo is small enough for a single job.

### C2: Backward compat via dual output, not format negotiation
**Decision**: During transition, the pipeline outputs BOTH catalog.db and manifests.json + versions.json. The old Go client reads JSON. The new Rust client reads SQLite. When the Go client is retired, JSON generation is removed.

### C3: Scoop-style template variables adopted wholesale
**Decision**: Use Scoop's exact variable names and semantics ($version, $majorVersion, etc.). No custom variables. This is a proven pattern from Scoop's 18,000+ package ecosystem.

### C4: Hash discovery follows Scoop's tiered pattern
**Decision**: Try in order: (1) URL+regex on a hash page, (2) JSON API endpoint, (3) download the file and compute SHA256. Most packages use (3) as fallback. This matches Scoop's `hash` section.

## Questions I Would Have Asked

### Q1: Should the catalog include historical versions or only latest?
**My decision**: Only latest per package. Historical versions are in the git history and per-version files. The client only needs current versions for update checking.
**Impact if wrong**: Low — adding historical versions later is a catalog schema change, not a pipeline change.

### Q2: Should we support incremental catalog updates (delta)?
**My decision**: No — full catalog rebuild each time. At ~95 packages, the catalog is small (<5MB). Incremental adds complexity for minimal bandwidth savings.
