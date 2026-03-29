# Decisions Report: 018-ci-pipeline
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: Single gate job pattern
**Choice**: One "CI OK" job that depends on all other jobs. Branch protection requires only this job.
**Reasoning**: Adding/removing CI jobs doesn't require updating branch protection rules. The gate job dynamically aggregates results.

### D2: Path filtering via dorny/paths-filter
**Choice**: dorny/paths-filter@v4 (already used in the project).
**Reasoning**: Proven in the existing CI. Avoids running Windows integration tests for frontend-only changes.

### D3: Ubuntu for Rust checks, Windows for integration tests
**Choice**: Rust fmt/clippy/test on Ubuntu (fast, cheap). Integration tests on Windows (real APIs).
**Reasoning**: Most Rust code is cross-platform. Only detection/install logic needs Windows. Ubuntu runners start faster and are free-tier.

## Clarify-Phase Decisions

### C1: Tauri system deps on Ubuntu for GUI crate clippy/check
**Decision**: Install `libwebkit2gtk-4.1-dev` etc. on Ubuntu for `cargo clippy -p astro-up-gui`. This is a check-only build, not a full Tauri build. Full Tauri builds only happen on Windows.

### C2: No Tauri build in PR CI
**Decision**: Tauri NSIS build is slow (~10 min) and only needed for releases. PR CI checks compilation (`cargo check`), not full builds. Release pipeline (spec 019) handles builds.

### C3: cocogitto for conventional commit validation
**Decision**: Use `cocogitto` (already in the project) for PR title and commit message validation. Enforces `feat:`, `fix:`, `chore:` etc.

### C4: Cargo cache keyed by Cargo.lock hash
**Decision**: `Swatinem/rust-cache@v2` with default key strategy (OS + Cargo.lock hash). Invalidates when dependencies change. No manual cache management.

## Questions I Would Have Asked

### Q1: Should CI run on push to main as well as PRs?
**My decision**: Yes — push to main triggers CI for release-please version bumps and direct commits. But skip redundant runs if the PR already passed (concurrency group with cancel-in-progress).

### Q2: Should we run WASM/cross-compilation checks?
**My decision**: No — astro-up targets Windows only. No WASM, no ARM, no Linux distribution.
