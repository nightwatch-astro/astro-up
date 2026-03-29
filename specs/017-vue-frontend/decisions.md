# Decisions Report: 017-vue-frontend
**Created**: 2026-03-29
**Mode**: Unattended

## Scope
Vue 3 + PrimeVue 4 + VueQuery 5. DataTable for software list. Badge for status. ProgressBar for downloads. Toast for notifications. Settings form. VueQuery composables wrapping Tauri invoke. Dark mode Aura. Dashboard/Settings/Custom views.

## Dependencies
spec 016 (Tauri shell)

## Key Decisions
- Follow migration plan architecture decisions
- Implement in astro-up-core where logic is shared, in crate-specific code where not
- Use types and traits from spec 003 (core domain types)
- Prioritize feature parity with Go implementation, then add Rust-specific improvements

## Questions I Would Have Asked
- Detailed user stories and acceptance scenarios need elaboration during the clarify phase with user input
- Integration points with other specs need validation against actual implementation
- Priority relative to other specs in the same phase needs confirmation
