# Decisions Report: 019-release-pipeline
**Created**: 2026-03-29
**Mode**: Unattended

## Scope
Tauri NSIS bundler for Windows installer. Ed25519 signing for auto-updater. SignPath.io Authenticode (deferred). release-plz for versioning/changelogs. Scoop bucket auto-update. Update endpoint JSON for tauri-plugin-updater.

## Dependencies
spec 018 (CI)

## Key Decisions
- Follow migration plan architecture decisions
- Implement in astro-up-core where logic is shared, in crate-specific code where not
- Use types and traits from spec 003 (core domain types)
- Prioritize feature parity with Go implementation, then add Rust-specific improvements

## Questions I Would Have Asked
- Detailed user stories and acceptance scenarios need elaboration during the clarify phase with user input
- Integration points with other specs need validation against actual implementation
- Priority relative to other specs in the same phase needs confirmation
