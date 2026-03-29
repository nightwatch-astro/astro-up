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
