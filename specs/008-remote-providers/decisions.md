# Decisions Report: 008-remote-providers
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions
- **No headless browser in client**: Browser scraping stays in the manifest repo CI checker. Client only uses HTTP-based providers. This keeps the client lightweight.
- **Parallel version checking**: Use tokio tasks with configurable concurrency (default: 10). Rate limiting per host, not global.
- **GitHub provider via octocrab**: Well-maintained, typed API client. Simpler than raw reqwest for GitHub-specific patterns.
- **Autoupdate URL templates**: Reuse Scoop's `$version`, `$majorVersion` etc. template variables for download URL construction.

## Questions I Would Have Asked
- Q1: Should we support custom HTTP headers for vendor sites? Decision: Yes — some vendors require specific User-Agent or Accept headers. Configurable per manifest.
- Q2: Should failed checks block the scan? Decision: No — report failures per-package but continue checking others. Use `continue-on-error` pattern.

## Clarify-Phase Decisions

### C1: Token bucket rate limiting, not sliding window
**Decision**: Token bucket is simpler to implement (atomic counter with refill timer) and works well for bursty patterns (bulk check at startup, then idle). Sliding window is overkill.

### C2: Default 10 concurrent checks
**Decision**: 10 balances speed vs resource usage. Most astro users are on residential internet. Higher concurrency risks rate limiting and DNS exhaustion.

### C3: Autoupdate URL template expansion is part of this spec
**Decision**: Version checking and URL construction are tightly coupled — the same `[checkver]` section defines both the version source and the download URL template. Splitting them would create unnecessary indirection.

### C4: HTML scraping tolerates selector failures gracefully
**Decision**: If a CSS selector returns no matches, report "version not found" for that package. Don't crash. Selector patterns are validated by manifest repo CI — client-side failures mean the page changed.

### C5: GitHub token from config (spec 004) via NetworkConfig.github_token
**Decision**: The provider reads the token from the loaded AppConfig. No separate token storage. This keeps all credentials in one place.
