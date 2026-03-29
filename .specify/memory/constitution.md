<!--
Sync Impact Report
- Version change: 0.0.0 → 1.0.0 (initial ratification)
- Added principles: I–VI (all new)
- Added sections: Technical Stack, Development Workflow, Governance
- Templates requiring updates:
  - .specify/templates/plan-template.md — ⚠ pending (Technical Context defaults need updating for Rust/Tauri/Vue)
  - .specify/templates/spec-template.md — ✅ no changes needed (generic enough)
  - .specify/templates/tasks-template.md — ⚠ pending (path conventions need Cargo workspace layout)
- Follow-up TODOs: none
-->

# Astro-Up Constitution

## Core Principles

### I. Modules-First Crate Layout

All domain logic lives in `astro-up-core` as modules (`types/`, `detect/`, `download/`,
`install/`, `engine/`, `catalog/`, `providers/`, `config/`, `backup/`). Modules share one
`Cargo.toml` and one compilation unit. A module MUST only be extracted into its own crate
when compile times justify the split. Public API boundaries MUST be clean enough that
extraction requires no interface changes — just move the directory, add `Cargo.toml`,
update imports.

### II. Platform Awareness

Windows is the primary target. All Windows-specific code MUST be gated behind
`#[cfg(windows)]` or Cargo features. Cross-platform code MUST compile and pass tests
on macOS and Linux (CI runs on all three). Platform-specific crates (`winreg`, `wmi`)
MUST be optional dependencies. The `pelite` crate provides cross-platform PE file
parsing — prefer it over Windows-only APIs.

### III. Test-First with Integration Tests

Prefer integration tests over mocks. Unit tests for pure logic (version parsing,
URL template expansion, dependency graph resolution). Integration tests for anything
touching I/O, system state, or cross-module boundaries. Use `insta` for snapshot
testing of CLI output, JSON responses, and serialized structures. Use `tempfile` for
filesystem tests. Mock only external services (GitHub API, vendor websites) — never
mock internal interfaces.

### IV. Thin Tauri Boundary

The `astro-up-gui` crate MUST be a thin adapter layer. All business logic lives in
`astro-up-core`. Tauri commands (`#[tauri::command]`) MUST delegate to core functions
with minimal transformation. The CLI (`astro-up-cli`) and GUI MUST produce identical
results for the same operations. This ensures core logic is testable without a window
or WebView.

### V. Spec-Driven Development

Every feature MUST have a speckit specification before implementation begins. The spec
defines acceptance criteria, the plan defines technical approach, tasks define execution
order. Deviations from spec MUST be flagged and routed through the iterate workflow.
No code without a spec. No spec changes without user approval.

### VI. Simplicity

Start with the simplest approach that meets the spec. No speculative abstractions,
no premature optimization, no features beyond what was specified. Three similar lines
of code is better than a premature abstraction. Add complexity only when the current
approach demonstrably fails. YAGNI applies to error handling, configuration options,
and extensibility points equally.

## Technical Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Language | Rust 2024 edition | Tauri, minisign, pelite; no runtime dependency |
| Async | Tokio | HTTP, WMI, concurrent downloads |
| GUI | Tauri v2 | Plugins, testing, community (104K stars) |
| Frontend | Vue 3 + PrimeVue + VueQuery | DataTable, 80+ components, dark mode |
| CLI | clap + ratatui | Separate binary, no WebView dependency |
| Storage | SQLite (rusqlite, bundled) | Catalog, versions, config, ledger |
| Logging | tracing | Tauri uses tracing internally |
| Errors | thiserror (library) + anyhow (app) | Typed enums in core, context in binaries |
| Testing | insta + pretty_assertions + tempfile | Snapshot testing, clear diffs |
| CI | GitHub Actions | Rust + Vue + integration tests |
| Distribution | Tauri NSIS bundler | Single installer, auto-updater |
| Manifests | Separate repo, TOML → SQLite | GitHub Releases distribution |

## Development Workflow

- **Branching**: Feature branches off `main`, merged with `--no-ff`
- **Commits**: Conventional commits (cocogitto), reference issue numbers
- **Reviews**: CI MUST pass before merge; PR required for `main`
- **Dependencies**: Vet against [blessed.rs](https://blessed.rs); use `cargo-deny` for audits
- **Formatting**: `cargo fmt` + `cargo clippy` enforced in CI; `pnpm lint` for frontend
- **Releases**: release-please for version management; Ed25519 signing for auto-updater

## Governance

This constitution supersedes all other development practices for the astro-up project.
Amendments require: (1) documented rationale, (2) user approval, (3) version bump per
semver (MAJOR for principle removal/redefinition, MINOR for additions, PATCH for
clarifications). All specs and plans MUST verify compliance with these principles via
the Constitution Check in plan-template.md.

**Version**: 1.0.0 | **Ratified**: 2026-03-29 | **Last Amended**: 2026-03-29
