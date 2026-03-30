# Decisions Report: 019-release-pipeline

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: release-plz over release-please
**Choice**: release-plz (Rust-native) for version management.
**Reasoning**: Understands Cargo.toml natively, handles workspace versioning, publishes to crates.io. release-please is better for Go/Node.

### D2: NSIS over MSI for primary installer
**Choice**: Tauri's NSIS bundler.
**Reasoning**: Tauri v2's default. Familiar Setup.exe, handles elevation, WebView2 bootstrapping.

### D3: OIDC trusted publishing for crates.io
**Choice**: GitHub OIDC tokens, not stored API token.
**Reasoning**: Short-lived, no rotation needed. Supported by crates.io.

### D4: Separate Ed25519 keys for app updates vs catalog
**Choice**: Different keys for different trust boundaries.
**Reasoning**: App signing key in astro-up CI secrets. Catalog signing key in manifests CI secrets.

## Clarify-Phase Decisions (Interactive)

### C1: NSIS installer bundles both GUI and CLI
**Finding**: User asked if NSIS can install both.
**Decision**: Yes — Tauri NSIS can include additional binaries. The installer puts both `astro-up.exe` (GUI) and `astro-up-cli.exe` (CLI) in `Program Files/astro-up/`. One installer, one install path, both binaries.
**Reasoning**: Users get both without managing two downloads. CLI is available on PATH after install.

### C2: Standalone CLI also on GitHub Release
**Decision**: `astro-up-cli.exe` attached separately to the same release for headless/server use. Users who only want CLI download this one file. No installer needed.

### C3: Scoop deferred
**Finding**: User confirmed Scoop is deferred.
**Decision**: No Scoop bucket update in this spec. When implemented, it's a simple JSON update + cross-repo dispatch.

### C4: SignPath.io deferred
**Finding**: User confirmed — waiting until tool is published.
**Decision**: Without Authenticode, SmartScreen will warn on first install. Acceptable for early adopters. SignPath integration is a future task.

### C5: Only astro-up-core published to crates.io
**Decision**: Library crate only. CLI and GUI are application crates — not useful as dependencies. Publishing them would just pollute the crates.io namespace.

## Questions I Would Have Asked

### Q1: Should we add CLI to PATH during install?
**My decision**: Yes — the NSIS installer adds the install directory to the user's PATH. After install, `astro-up` and `astro-up-cli` are available from any terminal. Standard for Windows CLI tools.

### Q2: Should pre-release versions be published?
**My decision**: No — only stable releases. Pre-release testing uses the GitHub Release download directly. crates.io only gets stable versions.
