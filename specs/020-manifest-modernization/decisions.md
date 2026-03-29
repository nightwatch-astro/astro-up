# Decisions Report: 020-manifest-modernization
**Created**: 2026-03-29
**Mode**: Unattended

## Scope
For nightwatch-astro/astro-up-manifests repo. Rename remote→checkver in TOMLs. Scoop-style version variables. TOML→SQLite compilation. Per-version files. Single CI job. GitHub Releases distribution with catalog.db.

## Dependencies
N/A (separate repo)

## Key Decisions
- Follow migration plan architecture decisions
- Implement in astro-up-core where logic is shared, in crate-specific code where not
- Use types and traits from spec 003 (core domain types)
- Prioritize feature parity with Go implementation, then add Rust-specific improvements

## Questions I Would Have Asked
- Detailed user stories and acceptance scenarios need elaboration during the clarify phase with user input
- Integration points with other specs need validation against actual implementation
- Priority relative to other specs in the same phase needs confirmation
