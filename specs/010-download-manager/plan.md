# Implementation Plan: Download Manager

**Branch**: `010-download-manager` | **Date**: 2026-03-30 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/010-download-manager/spec.md`

## Summary

HTTP download manager for astro-up-core: streaming downloads with SHA256 verification, resume via Range headers, progress events via tokio broadcast, bandwidth throttling, and installer auto-purge. All logic in a new `download` module within astro-up-core.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: reqwest 0.13 (existing), sha2 0.10 (new), tokio-util 0.7 (new — CancellationToken)
**Storage**: N/A (file-based — `.part` files and final installers on disk)
**Testing**: cargo test + `wiremock` for HTTP server simulation, `tempfile` for filesystem tests, `insta` for snapshots
**Target Platform**: Windows primary, cross-platform CI (macOS, Linux)
**Project Type**: Library (astro-up-core crate)
**Performance Goals**: 80% bandwidth utilization (SC-001), <1s hash overhead for 500MB (SC-002), progress events ≥1/sec (SC-004), throttle within 10% (SC-005)
**Constraints**: Sequential downloads (one at a time), streaming (no full-body buffering), 64KB chunk size
**Scale/Scope**: ~100 packages, largest installer ~500MB (NINA)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | New `download/` module in astro-up-core, not a new crate |
| II. Platform Awareness | PASS | No platform-specific code — reqwest + tokio are cross-platform. Rename retry logic uses `std::fs::rename` (works everywhere, Windows lock retry is a loop) |
| III. Test-First | PASS | Integration tests with wiremock HTTP server, tempfile for disk ops. No internal mocks. |
| IV. Thin Tauri Boundary | PASS | All download logic in core. GUI/CLI consume events via broadcast channel. |
| V. Spec-Driven | PASS | Full spec exists with 19 FRs, 5 SCs, 5 user stories |
| VI. Simplicity | PASS | Sleep-between-chunks throttle (not token bucket crate), VecDeque rolling window (not stats library), sequential downloads (not concurrent) |

**Post-Phase 1 re-check**: All principles still pass. No new abstractions, no new crates, no platform-specific code.

## Project Structure

### Documentation (this feature)

```text
specs/010-download-manager/
├── spec.md
├── decisions.md
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── download-manager.rs
├── checklists/
│   ├── requirements.md
│   └── download.md
└── tasks.md             # Phase 2 output (next step)
```

### Source Code (repository root)

```text
crates/astro-up-core/src/
├── download/
│   ├── mod.rs           # DownloadManager struct, public API, re-exports
│   ├── types.rs         # DownloadRequest, DownloadResult, DownloadProgress, PurgeResult
│   ├── client.rs        # reqwest client builder from AppConfig
│   ├── stream.rs        # Download loop: streaming, hashing, resume, throttle
│   └── purge.rs         # Installer retention and auto-purge
├── config/
│   └── model.rs         # (modified) Add connect_timeout, download_speed_limit, keep_installers, purge_days
├── error.rs             # (modified) Add download-specific error variants
├── events.rs            # (unchanged) Existing DownloadStarted/Progress/Complete variants
├── traits.rs            # (modified) Update Downloader trait to use new types
└── lib.rs               # (modified) Add `pub mod download;`

crates/astro-up-core/
└── Cargo.toml           # (modified) Add sha2, tokio-util deps

tests/                   # Integration tests
└── download/
    ├── basic_download.rs
    ├── resume.rs
    ├── hash_verification.rs
    ├── throttle.rs
    └── purge.rs
```

**Structure Decision**: New `download/` module within existing astro-up-core crate. Five files split by responsibility: types, client config, streaming loop, and purge. Integration tests in `tests/` directory with wiremock HTTP server.

## Complexity Tracking

No violations — all complexity gates pass.

## Implementation Phases

### Phase A: Foundation (types + client + basic download)

**Goal**: Download a file to disk with progress events.

1. Add `sha2` and `tokio-util` dependencies to Cargo.toml
2. Create `download/types.rs` — DownloadRequest, DownloadResult, DownloadProgress, PurgeResult
3. Add download error variants to `error.rs`
4. Create `download/client.rs` — build reqwest::Client from AppConfig (proxy, timeouts, redirects, user-agent)
5. Create `download/mod.rs` — DownloadManager struct with `new()` and sequential download lock
6. Create `download/stream.rs` — basic streaming download loop (chunk → hash → write → progress event)
7. Wire `pub mod download` in lib.rs
8. Add config fields: `network.connect_timeout`, `network.download_speed_limit` to NetworkConfig
9. Integration test: download a file from wiremock server, verify contents and hash

### Phase B: Verification + Resume

**Goal**: SHA256 verification, conditional requests, and resume.

1. Hash verification — compare final SHA256 against expected, delete on mismatch, retry once from scratch
2. Conditional requests — ETag/Last-Modified to skip unchanged files (304 → Cached result)
3. Resume probe — check `.part` file existence, send Range header, handle 206 vs 200
4. Freshness check — compare `.part` file age against server Last-Modified
5. `.part` file size validation before resume (corrupt .part handling)
6. Hash mismatch after resume → delete and retry from scratch
7. Integration tests: resume, hash mismatch, conditional request, corrupt .part

### Phase C: Throttle + Cancellation + Edge Cases

**Goal**: Bandwidth control, user cancellation, and robustness.

1. Bandwidth throttling — sleep-between-chunks with 5-second rolling window
2. Cancellation — CancellationToken check after each chunk, leave .part on cancel
3. Disk space check — sysinfo available space before download
4. Rename retry — 3 attempts with 1-second delay for Windows file locks
5. Download directory auto-creation (FR-019)
6. Indeterminate progress (no Content-Length) — total_bytes=0, skip disk check
7. Sequential download enforcement — AtomicBool guard, return DownloadInProgress error
8. Integration tests: throttle accuracy, cancellation, disk space, rename retry

### Phase D: Purge + Cleanup

**Goal**: Installer retention and auto-purge.

1. Create `download/purge.rs` — scan download_dir, delete files older than max_age_days
2. Add config fields: `paths.keep_installers`, `paths.purge_installers_after_days` to PathsConfig
3. Update Downloader trait in traits.rs to align with new DownloadManager API
4. Integration test: purge with various ages, disabled purge (0 days)

## Dependencies

- **Spec 004** (config): Need iterate to add 4 new config keys. Can stub defaults initially.
- **Spec 005** (catalog): VersionEntry provides download URLs and SHA256 hashes. Read-only dependency — no changes needed.
- **Spec 003** (types): Event enum already has download variants. No changes needed.

## New Crate Dependencies

| Crate | Version | Reason | blessed.rs | lib.rs |
|-------|---------|--------|------------|--------|
| `sha2` | 0.10 | SHA256 streaming hash | yes | yes |
| `tokio-util` | 0.7 | CancellationToken | yes | yes |
| `wiremock` | 0.6 | HTTP mock server (dev-dependency) | no | yes |
