# Decisions Report: 018-ci-pipeline
**Created**: 2026-03-29
**Mode**: Unattended

## Scope
GitHub Actions CI. cargo check/test/clippy/fmt. pnpm lint/test/build. Tauri build on Windows runner. Integration tests with real detection. Conventional commits via cocogitto. Reuse nightwatch-astro/.github shared workflows.

## Dependencies
spec 001 (scaffolding)

## Key Decisions
- Follow migration plan architecture decisions
- Implement in astro-up-core where logic is shared, in crate-specific code where not
- Use types and traits from spec 003 (core domain types)
- Prioritize feature parity with Go implementation, then add Rust-specific improvements

## Questions I Would Have Asked
- Detailed user stories and acceptance scenarios need elaboration during the clarify phase with user input
- Integration points with other specs need validation against actual implementation
- Priority relative to other specs in the same phase needs confirmation
