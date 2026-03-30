# Tasks: Download Manager

**Input**: Design documents from `/specs/010-download-manager/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add dependencies and create module skeleton

- [x] T001 Add `sha2 = "0.10"` and `tokio-util = { version = "0.7", features = ["rt"] }` to `crates/astro-up-core/Cargo.toml`, add `sync` feature to existing `tokio` dependency
- [x] T002 Add `wiremock = "0.6"` as dev-dependency to `crates/astro-up-core/Cargo.toml`
- [x] T003 Create `crates/astro-up-core/src/download/mod.rs` with `pub mod types; pub mod client; pub mod stream; pub mod purge;` and add `pub mod download;` to `crates/astro-up-core/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Types, errors, and config changes that all user stories depend on

- [x] T004 [P] Create download types (`DownloadRequest`, `DownloadResult`, `DownloadProgress`, `PurgeResult`) in `crates/astro-up-core/src/download/types.rs` per data-model.md
- [x] T005 [P] Add download error variants to `crates/astro-up-core/src/error.rs`: `DownloadFailed { url, status, reason }`, `DiskSpaceInsufficient { required, available }`, `DownloadInProgress { url }`, `RenameFailed { from, to, cause }`
- [x] T006 [P] Add download config fields to `crates/astro-up-core/src/config/model.rs` and `crates/astro-up-core/src/config/defaults.rs`: `connect_timeout: Duration` (default 10s) and `download_speed_limit: u64` (default 0) on `NetworkConfig`, `keep_installers: bool` (default true) and `purge_installers_after_days: u32` (default 30) on `PathsConfig`
- [x] T007 Implement `build_client(config: &NetworkConfig) -> reqwest::Client` in `crates/astro-up-core/src/download/client.rs` — configure proxy, connect_timeout, read timeout, redirect policy (10 hops), user-agent, rustls
- [x] T008 Implement `DownloadManager` struct with `new(config, event_tx)` constructor and `AtomicBool` sequential download lock in `crates/astro-up-core/src/download/mod.rs`

**Checkpoint**: Foundation ready — all types, errors, config, and manager skeleton in place

---

## Phase 3: User Story 1 — Download with Progress Reporting (Priority: P1) MVP

**Goal**: Download a file via HTTP streaming, emit progress events via broadcast channel, write to `.part` file, rename to final

**Independent Test**: Download a file from wiremock server, verify file contents, verify progress events were emitted with bytes/total/speed

### Implementation for User Story 1

- [x] T009 [US1] Implement core streaming download loop in `crates/astro-up-core/src/download/stream.rs`: `async fn stream_download(client, url, part_path, event_tx, id, throttle_config, cancel_token: CancellationToken) -> Result<(u64, Sha256), CoreError>` — stream chunks via `response.chunk()`, write to `.part` file, hash incrementally with `sha2::Sha256`, emit `Event::DownloadProgress` every 100ms or 64KB (whichever first), compute speed as 5-second rolling average using `VecDeque<(Instant, u64)>`, compute `estimated_remaining` as `remaining_bytes / speed` when `total_bytes > 0` (else `None`). Accept CancellationToken but do not check it yet (T024 adds the check-after-chunk logic)
- [x] T010 [US1] Implement `DownloadManager::download(&self, request: &DownloadRequest, cancel_token: CancellationToken) -> Result<DownloadResult, CoreError>` in `crates/astro-up-core/src/download/mod.rs` — acquire sequential lock via `AtomicBool::compare_exchange` (return `DownloadInProgress` if held), release via drop guard for panic safety, auto-create dest_dir (FR-019), emit `DownloadStarted`, call `stream_download` passing cancel_token, rename `.part` to final, emit `DownloadComplete`. Send User-Agent header `astro-up/{version}` via `env!("CARGO_PKG_VERSION")`
- [x] T011 [US1] Integration test for basic download in `crates/astro-up-core/tests/download_basic.rs`: start wiremock `MockServer`, mount a mock returning 200 with known body bytes and Content-Length header, call `DownloadManager::download()`, assert file exists with correct contents, assert progress events received via broadcast receiver, assert `DownloadResult::Success` with correct `bytes_downloaded`. Also test error reporting (FR-011): mock returns 403, assert error contains URL and HTTP status code. Also test redirects (FR-010): mock with redirect chain, assert follows; mock with >10 redirects, assert error

**Checkpoint**: US1 complete — can download a file, see progress, verify output

---

## Phase 4: User Story 2 — Hash Verification (Priority: P2)

**Goal**: Verify SHA256 after download, reject mismatches, handle missing hashes

**Independent Test**: Download a file, verify hash matches. Provide wrong expected hash, verify file is deleted and `ChecksumMismatch` error returned

### Implementation for User Story 2

- [x] T012 [US2] Add hash verification to `stream_download` in `crates/astro-up-core/src/download/stream.rs`: after streaming completes, finalize `Sha256` digest, compare hex string against `DownloadRequest.expected_hash`. On mismatch return `CoreError::ChecksumMismatch`. On `None` expected hash, set `hash_verified: false` in result
- [x] T013 [US2] Add conditional request support (ETag/Last-Modified) in `crates/astro-up-core/src/download/stream.rs`: before downloading, if final file already exists, send `If-None-Match`/`If-Modified-Since` headers. On 304 response, return `DownloadResult::Cached`
- [x] T014 [US2] Integration test for hash verification in `crates/astro-up-core/tests/download_hash.rs`: (a) download with correct expected hash — assert success with `hash_verified: true`, (b) download with wrong expected hash — assert `ChecksumMismatch` error and `.part` file deleted, (c) download with no expected hash — assert success with `hash_verified: false` and warning logged

**Checkpoint**: US2 complete — downloads are integrity-verified

---

## Phase 5: User Story 3 — Resume Failed Downloads (Priority: P3)

**Goal**: Resume partial downloads via Range headers, probe server support, validate freshness

**Independent Test**: Start a download, simulate failure at 50% (mock returns partial body then error), retry, verify only remaining bytes are fetched

### Implementation for User Story 3

- [ ] T015 [US3] Implement resume probe in `crates/astro-up-core/src/download/stream.rs`: if `.part` file exists, read its size, send `Range: bytes={size}-` header. If server returns 206: open `.part` in append mode, continue streaming. If server returns 200: delete `.part`, restart from scratch
- [ ] T016 [US3] Add freshness validation in `crates/astro-up-core/src/download/stream.rs`: before resuming, compare `.part` file mtime against server `Last-Modified` header. If server file is newer, delete `.part` and restart. Validate `.part` file size ≤ server Content-Length (corrupt .part detection per CHK011)
- [ ] T017 [US3] Handle hash mismatch after resumed download in `crates/astro-up-core/src/download/mod.rs`: if hash verification fails and download was resumed (`resumed: true`), delete `.part` file and retry once from scratch (CHK012). If second attempt also fails, return `ChecksumMismatch` error
- [ ] T018 [US3] Integration test for resume in `crates/astro-up-core/tests/download_resume.rs`: (a) wiremock serves partial body (e.g., first 512 bytes of 1024), client writes `.part`, second request with Range header — mock uses dynamic response (`|req: &Request|`) to check Range header and return 206 with remaining bytes, assert final file is complete, (b) mock returns 200 on Range request — assert full re-download, (c) mock returns newer Last-Modified — assert `.part` deleted and full download

**Checkpoint**: US3 complete — large downloads survive failures

---

## Phase 6: User Story 4 — Bandwidth Throttling (Priority: P4)

**Goal**: Configurable download speed limit, smooth throttling within 10% accuracy

**Independent Test**: Set 1MB/s limit, download 5MB file, verify elapsed time is ~5 seconds (within 10%)

### Implementation for User Story 4

- [ ] T019 [US4] Implement bandwidth throttling in `crates/astro-up-core/src/download/stream.rs`: after each chunk write, calculate expected elapsed time at configured rate (`bytes_so_far / max_bytes_per_sec`), if ahead of schedule sleep via `tokio::time::sleep()`. Track speed using `VecDeque<(Instant, u64)>` rolling window (5 seconds). Skip throttle when `download_speed_limit == 0`
- [ ] T020 [US4] Integration test for throttle in `crates/astro-up-core/tests/download_throttle.rs`: set `download_speed_limit` to 100KB/s, download a 500KB file from wiremock, assert elapsed time is 4.5–5.5 seconds (10% tolerance per SC-005), assert progress events report speed near 100KB/s

**Checkpoint**: US4 complete — downloads respect bandwidth limits

---

## Phase 7: User Story 5 — Installer Retention and Auto-Purge (Priority: P5)

**Goal**: Keep downloaded installers, auto-purge old ones based on configurable age

**Independent Test**: Create files with backdated mtime in download dir, call purge(30 days), verify old files deleted and new ones kept

### Implementation for User Story 5

- [ ] T021 [US5] Implement `DownloadManager::purge()` in `crates/astro-up-core/src/download/purge.rs`: scan `download_dir` for files (not `.part` files), check each file's mtime against `max_age_days`, delete files older than threshold, return `PurgeResult { files_deleted, bytes_reclaimed }`. Skip when `max_age_days == 0` (disabled)
- [ ] T022 [US5] Integration test for purge in `crates/astro-up-core/tests/download_purge.rs`: (a) create files in tempdir with varied mtimes (using `filetime` crate or `std::fs::File::set_times`), call purge with 30 days, assert old files deleted and recent files kept, (b) call purge with 0 days — assert no files deleted, (c) verify `.part` files are never purged

**Checkpoint**: US5 complete — disk space managed automatically

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, robustness, and cleanup

- [ ] T023 [P] Implement disk space pre-check in `crates/astro-up-core/src/download/stream.rs`: before downloading, use `sysinfo::Disks` to check available space ≥ 2x Content-Length (FR-017). Skip if Content-Length unknown. Return `DiskSpaceInsufficient` error if insufficient
- [ ] T024 [P] Implement cancellation support in `crates/astro-up-core/src/download/stream.rs`: accept `CancellationToken` parameter, check `token.is_cancelled()` after each chunk. On cancellation, leave `.part` file on disk, return `CoreError::Cancelled` (add variant to error.rs)
- [ ] T025 [P] Implement rename retry logic in `crates/astro-up-core/src/download/mod.rs`: after hash verification, `fs::rename(.part, final)` with up to 3 retries and 1-second `tokio::time::sleep()` delay. On final failure return `CoreError::RenameFailed`
- [ ] T026 [P] Handle indeterminate Content-Length in `crates/astro-up-core/src/download/stream.rs`: when `response.content_length()` is `None`, set `total_bytes = 0` in progress events, skip disk space check, still run hash verification after completion (CHK015)
- [ ] T027 Update `Downloader` trait in `crates/astro-up-core/src/traits.rs` to align signature with `DownloadManager::download()` — accept `DownloadRequest` + `CancellationToken`, return `Result<DownloadResult, CoreError>`
- [ ] T028 Update flume channel comment in `crates/astro-up-core/src/events.rs` line 65 — change "flume channels" to "broadcast channels" to match actual implementation
- [ ] T029 Run `just check` — ensure clippy, fmt, and all tests pass across core, cli, gui crates

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 — MVP
- **Phase 4 (US2)**: Depends on Phase 3 (hash integrates into stream loop)
- **Phase 5 (US3)**: Depends on Phase 3 (resume extends stream loop)
- **Phase 6 (US4)**: Depends on Phase 3 (throttle integrates into stream loop)
- **Phase 7 (US5)**: Depends on Phase 2 only (purge is independent of download flow)
- **Phase 8 (Polish)**: T023–T026 depend on Phase 3, T027–T029 depend on all phases

### User Story Dependencies

- **US1 (P1)**: Foundation only — no story dependencies
- **US2 (P2)**: Depends on US1 (hash verification wraps around the stream loop)
- **US3 (P3)**: Depends on US1 (resume extends the stream loop with Range headers)
- **US4 (P4)**: Depends on US1 (throttle adds sleep to the stream loop)
- **US5 (P5)**: Independent of other stories (purge scans files, doesn't touch downloads)

### Parallel Opportunities

- T004, T005, T006 can run in parallel (different files)
- T023, T024, T025, T026 can run in parallel (different concerns, clearly separated code)
- US5 (Phase 7) can run in parallel with US2/US3/US4 (no shared code)

---

## Parallel Example: Foundational Phase

```
# These three tasks modify different files — run in parallel:
Task T004: download/types.rs
Task T005: error.rs
Task T006: config/model.rs + config/defaults.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T009)
3. Complete Phase 3: US1 — Download with Progress (T009–T011)
4. **STOP and VALIDATE**: Download a file, see progress, verify output
5. Commit and checkpoint

### Incremental Delivery

1. Setup + Foundational → skeleton ready
2. US1 → basic downloads work (MVP)
3. US2 → integrity verification
4. US3 → resume support
5. US4 → bandwidth control
6. US5 → disk management (can run earlier, independent)
7. Polish → edge cases and cleanup

---

## Notes

- All download logic in `crates/astro-up-core/src/download/` — no GUI or CLI code
- Integration tests use `wiremock::MockServer` for HTTP mocking with dynamic responses
- Config changes (T006) technically belong to spec 004 iterate — stub defaults here, formalize via iterate later
- The `Downloader` trait update (T028) aligns the existing trait with the new concrete implementation
