# Decisions Report: 016-tauri-app-shell
**Created**: 2026-03-29
**Mode**: Unattended

## Scope
Tauri v2 application. Plugin config, window management, system tray. Commands wrapping core crate. Event bridge Rust→frontend. System tray with update badge. Self-update via tauri-plugin-updater. Ed25519 signatures.

## Dependencies
spec 004 (config), spec 012 (engine)

## Key Decisions
- Follow migration plan architecture decisions
- Implement in astro-up-core where logic is shared, in crate-specific code where not
- Use types and traits from spec 003 (core domain types)
- Prioritize feature parity with Go implementation, then add Rust-specific improvements

## Questions I Would Have Asked
- Detailed user stories and acceptance scenarios need elaboration during the clarify phase with user input
- Integration points with other specs need validation against actual implementation
- Priority relative to other specs in the same phase needs confirmation
