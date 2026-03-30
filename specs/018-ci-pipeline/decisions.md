# Decisions Report: 018-ci-pipeline

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: Single gate job pattern
**Choice**: One "CI OK" job depending on all others. Branch protection requires only this.
**Reasoning**: Adding/removing jobs doesn't require branch protection updates.

### D2: dorny/paths-filter@v4 for path filtering
**Choice**: Same pattern as other nightwatch-astro repos (already working).

### D3: Parallel Rust + frontend jobs
**Choice**: Independent jobs run simultaneously. Total CI time = max(Rust, frontend), not sum.

## Clarify-Phase Decisions (Interactive)

### C1: Real installs on every PR touching Rust code
**Finding**: User asked about install test cost. PHD2 download + install + detect + uninstall ≈ 30s. ASCOM ≈ 45s. Total ≈ 1 min on top of a 2 min Windows job.
**Decision**: Run real installs on every PR that touches `crates/`. The 1 min cost is worth it for confidence. No weekly-only schedule — test on every PR.
**Reasoning**: GitHub Free runners have fast internet (~100Mbps). 20MB PHD2 downloads in 3s. The install pipeline is the riskiest code — it should be tested on every change.

### C2: PHD2 (InnoSetup) + ASCOM Platform (MSI) as test subjects
**Finding**: User suggested PHD2. Added ASCOM for MSI coverage.
**Decision**: Two real packages covering the two most common installer types. PHD2 validates InnoSetup silent install + registry detection + PE version. ASCOM validates MSI install + ASCOM Profile registry.
**Reasoning**: Small downloads (~70MB total), fast installs, stable software unlikely to break CI.

### C3: Path filtering matches other nightwatch-astro repos
**Finding**: User said "ensure CI is properly filtered just like other repos."
**Decision**: Same dorny/paths-filter pattern. Path groups: crates, frontend, ci, docs. CI changes trigger all jobs. Docs trigger only the gate job.

### C4: Extend spec 001 CI, not replace
**Finding**: User said "extend it."
**Decision**: This spec adds integration tests and refines path filtering. The existing jobs (check-rust, check-gui, check-frontend, check-windows) are the foundation. No rewrite.

### C5: Portable test executables for unit tests
**Decision**: In addition to real installs (integration), keep small test .exe files with known PE versions checked into the repo for fast unit tests. These test PE parsing without any downloads. The integration tests validate the full pipeline.

## Questions I Would Have Asked

### Q1: Should integration tests run on push to main too?
**My decision**: Yes — catch any issues from squash merges. But use concurrency groups to cancel redundant runs.

### Q2: Should we test the full orchestration pipeline (detect → download → install → verify)?
**My decision**: Yes — at least one end-to-end test that exercises the whole chain. PHD2: detect (not installed) → download → install → detect (installed, version matches) → uninstall → detect (not installed). This is the most valuable integration test.
