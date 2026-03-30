# Decisions Report: 021-feature-parity
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: Parity is one-directional (Go→Rust)
**Choice**: Every Go feature must exist in Rust. Rust-only features are fine.
**Reasoning**: The goal is to replace Go without regression. New Rust features are a bonus, not a parity requirement.

### D2: Feature matrix is a markdown document, not automated CI
**Choice**: A manually maintained markdown table, not an automated comparison tool.
**Reasoning**: Feature parity is a one-time migration gate, not an ongoing check. A markdown matrix is reviewed by humans and signed off. Automation is overkill for a one-time event.

### D3: Detection parity tested on real hardware
**Choice**: Run both Go and Rust on a Windows machine with actual astrophotography software. Not mocked.
**Reasoning**: Detection tests against real registry keys, real PE files, real WMI queries. Mocks can't validate this.

### D4: Performance benchmarks via hyperfine
**Choice**: Use hyperfine for CLI benchmarks (startup, scan, check). Manual timing for GUI operations.
**Reasoning**: hyperfine handles warm-up, statistical analysis, and comparison. Already available in the project's tool set.

## Clarify-Phase Decisions

### C1: Go archive stays available for comparison
**Decision**: The old astro-up/astro-up repo is archived but not deleted. It can be cloned for parity testing. Once parity is confirmed, it remains as a historical reference.

### C2: Parity report is the go/no-go gate
**Decision**: The parity matrix must be fully green (all implemented or N/A) before the project is announced as ready. Partial migration is not acceptable — users get either Go or Rust, not both.

### C3: N/A is valid for replaced features
**Decision**: Bubble Tea TUI → ratatui is N/A (different but equivalent). Wails → Tauri is N/A. The parity is functional, not implementation-identical.

### C4: Manifest compatibility tested against all 95+ manifests
**Decision**: Parse every manifest through both Go and Rust, diff the outputs. Any field-level difference is a bug. Template variable resolution is the highest-risk area.

## Questions I Would Have Asked

### Q1: When should parity verification run — after all specs or incrementally?
**My decision**: Incrementally after each phase. Don't wait until the end. Phase 1 (core types) → verify types match. Phase 2 (detection) → verify detection matches. Etc.
**Impact if wrong**: Low — incremental is strictly better. Catches divergence early.

### Q2: Should we port Go integration tests to Rust?
**My decision**: Yes — port the most valuable integration tests. Don't port tests that test Go-specific patterns (goroutine behavior, channel mechanics). Focus on behavioral tests (given input X, expect output Y).
