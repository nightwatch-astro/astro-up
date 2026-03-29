# Decisions Report: 019-release-pipeline
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: release-plz over release-please
**Choice**: release-plz (Rust-native) instead of release-please (Google, language-agnostic).
**Reasoning**: release-plz understands Cargo.toml natively, handles workspace versioning, and publishes to crates.io. release-please is better for Go/Node but adds TOML parsing overhead for Rust.

### D2: NSIS installer, not MSI
**Choice**: Tauri's NSIS bundler for the Windows installer.
**Reasoning**: NSIS is Tauri v2's default bundler. Produces a familiar Setup.exe with install/uninstall, elevation handling, and WebView2 bootstrapping. MSI is an alternative but NSIS has better Tauri integration.

### D3: SignPath.io deferred
**Choice**: Authenticode code signing via SignPath.io is planned but deferred.
**Reasoning**: SignPath requires account setup and code signing certificate procurement. Ed25519 signing for auto-updater is sufficient for initial releases. Authenticode prevents SmartScreen warnings but isn't blocking for early adopters.

### D4: OIDC trusted publishing for crates.io
**Choice**: Use GitHub OIDC tokens for crates.io publishing, not a stored API token.
**Reasoning**: OIDC tokens are short-lived and don't need secret rotation. Supported by crates.io since 2024. More secure than long-lived API tokens.

## Clarify-Phase Decisions

### C1: Drop portable build — NSIS only
**Decision**: The Go version had both installer and portable builds. Tauri's NSIS installer handles both install and portable use cases (can extract without installing). One artifact simplifies the release pipeline.

### C2: Update endpoint on GitHub Releases, not a separate server
**Decision**: The update JSON file is attached to the GitHub Release as an asset. tauri-plugin-updater fetches it via the Releases API. No need for a separate server or CDN.

### C3: Scoop bucket update via GitHub App token
**Decision**: Use the nightwatch-astro GitHub App for cross-repo dispatch (scoop-bucket update). App tokens are scoped and expire in 1 hour. Safer than a PAT.

### C4: Release builds only on release tags, not on every push
**Decision**: Full Tauri NSIS build takes ~10 minutes. Only run on `v*` tags created by release-plz. PR CI does `cargo check` only (spec 018).

## Questions I Would Have Asked

### Q1: Should we support auto-updating the CLI binary separately from the GUI?
**My decision**: Yes — CLI self-update (spec 015) downloads a standalone binary from the same GitHub Release. Both CLI and GUI artifacts are attached to the same release.

### Q2: Should we support release channels (stable, beta, nightly)?
**My decision**: Not in v1. Single stable channel. Beta/nightly adds release management complexity for a project with a small user base.
