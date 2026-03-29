# Decisions Report: 014-custom-tools
**Created**: 2026-03-29
**Mode**: Unattended

## Scope
Add/remove user-defined tools from GitHub URLs. Parse repo URL, fetch latest release, filter Windows assets, auto-detect install method, generate manifest TOML.

## Dependencies
spec 003 (types), spec 005 (catalog), spec 011 (installer)

## Key Decisions
- Follow migration plan architecture decisions
- Implement in astro-up-core where logic is shared, in crate-specific code where not
- Use types and traits from spec 003 (core domain types)
- Prioritize feature parity with Go implementation, then add Rust-specific improvements

## Questions I Would Have Asked
- Detailed user stories and acceptance scenarios need elaboration during the clarify phase with user input
- Integration points with other specs need validation against actual implementation
- Priority relative to other specs in the same phase needs confirmation
