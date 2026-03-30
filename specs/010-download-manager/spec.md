# Feature Specification: Download Manager

**Feature Branch**: `010-download-manager`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 009 — HTTP downloads with hash verification, progress, and resume

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Download with Progress Reporting (Priority: P1)

A user initiates a software update. The download manager fetches the installer from the vendor's URL, reporting progress (bytes downloaded, total size, speed) via events that the CLI shows as a progress bar and the GUI shows as a progress indicator.

**Why this priority**: Downloads are the most visible operation — users need feedback.

**Independent Test**: Download a known file, verify progress events are emitted and the file is correctly saved.

**Acceptance Scenarios**:

1. **Given** a download URL for NINA installer, **When** download starts, **Then** progress events report bytes/total/speed at regular intervals
2. **Given** a slow connection, **When** downloading, **Then** the progress bar updates smoothly without stalling
3. **Given** the download completes, **When** verifying, **Then** the SHA256 hash matches the expected value from the catalog

---

### User Story 2 - Hash Verification (Priority: P2)

After downloading, the manager verifies the file's SHA256 hash against the expected hash from the catalog version entry. If the hash doesn't match, the file is discarded and the user is informed.

**Why this priority**: Integrity verification prevents corrupted or tampered installers from being executed.

**Independent Test**: Download a file, verify hash matches. Corrupt the file, verify hash check fails.

**Acceptance Scenarios**:

1. **Given** a completed download, **When** the SHA256 matches, **Then** the file is moved to the destination
2. **Given** a completed download, **When** the SHA256 doesn't match, **Then** the file is deleted and an error is reported
3. **Given** no expected hash is available, **When** the download completes, **Then** it proceeds with a warning

---

### User Story 3 - Resume Failed Downloads (Priority: P3)

If a download fails mid-stream (network drop, timeout), the manager detects the existing `.part` file on the next attempt and probes the server for resume support via HTTP Range headers. If the server supports it, only the remaining bytes are fetched. If not, the download restarts from scratch.

**Why this priority**: Large installers (NINA ~500MB) benefit significantly from resume. Users on unstable connections (remote observatories) need this.

**Independent Test**: Start a download, simulate a network failure at 50%, retry, verify resume fetches only the remaining bytes.

**Acceptance Scenarios**:

1. **Given** a partial `.part` file exists from a failed download, **When** retrying, **Then** the manager sends a Range header for the remaining bytes
2. **Given** the server supports Range, **When** resuming, **Then** only the remaining bytes are downloaded and appended
3. **Given** the server doesn't support Range (returns 200 instead of 206), **When** resuming, **Then** the download restarts from the beginning
4. **Given** the partial file is older than the server's Last-Modified, **When** resuming, **Then** the download restarts (file may have changed)

---

### User Story 4 - Bandwidth Throttling (Priority: P4)

A user configures a download speed limit in settings. During imaging sessions, large downloads won't saturate the connection and interfere with plate solving, remote desktop, or PHD2 guiding corrections.

**Why this priority**: Astrophotography sessions rely on network stability. Unthrottled downloads can disrupt active imaging.

**Independent Test**: Set a 1MB/s limit, start a download, verify actual throughput stays near the limit.

**Acceptance Scenarios**:

1. **Given** bandwidth limit is set to 1MB/s, **When** downloading, **Then** throughput does not exceed ~1MB/s
2. **Given** no bandwidth limit is configured, **When** downloading, **Then** full available bandwidth is used
3. **Given** bandwidth limit is changed mid-download, **When** the new setting takes effect, **Then** throughput adjusts within a few seconds

---

### User Story 5 - Installer Retention and Auto-Purge (Priority: P5)

Downloaded installers are kept after installation for potential offline re-install. A configurable auto-purge removes installers older than N days when the application runs in the background (system tray).

**Why this priority**: Disk space management without losing the ability to re-install offline.

**Independent Test**: Download and install a package. Verify the installer is retained. Set purge to 1 day, advance time, verify the installer is deleted on next background run.

**Acceptance Scenarios**:

1. **Given** an installer was downloaded and installed, **When** checking the download directory, **Then** the installer file is still present
2. **Given** purge is set to 30 days and an installer is 45 days old, **When** the background purge runs, **Then** the old installer is deleted
3. **Given** purge is set to 0 (disabled), **When** the background purge runs, **Then** no installers are deleted
4. **Given** the app is not running in the background, **When** time passes, **Then** no purging occurs (only runs when app is active)

### Edge Cases

- Server returns a redirect: Follow redirects (up to 10 hops). If exceeded, report error with the redirect chain. GitHub Releases uses 302 to CDN. (CHK001)
- Server returns 403/404: Report clear download error with URL and HTTP status.
- Disk full during download: Detect via write error, report required vs available space.
- Download via proxy: Use proxy settings from config (spec 004).
- Resume probe fails (server error on Range request): Fall back to full re-download.
- Multiple downloads for the same package (e.g., retry after hash mismatch): Delete the old .part file first.
- No Content-Length header (chunked transfer): Progress reports bytes_downloaded with total_bytes=0 (indeterminate). Disk space check is skipped (cannot estimate). Hash verification still runs after completion. (CHK015)
- Corrupt `.part` file (e.g., disk error during previous write): Validate `.part` file size against Range offset before resuming. If the local file is larger than the server reports remaining, delete and restart. (CHK011)
- Hash mismatch after resumed download: Delete the file and retry once from scratch (the `.part` may have been corrupted). If the second attempt also fails, report error. (CHK012)
- Server file changed between attempts (same size, different Last-Modified): Restart from scratch — freshness check catches this. (CHK013)
- Rename from `.part` to final fails (target locked by another process): Retry rename up to 3 times with 1-second delay. If still locked, report error with the blocking path. (CHK016)
- URL with special characters or spaces: URLs are used as-is from the catalog (already URL-encoded by the manifest pipeline). No additional encoding applied. (CHK017)
- Download directory doesn't exist: Auto-create the directory tree before downloading. (CHK018)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST download files via HTTP/HTTPS with streaming using a fixed chunk size (default 64KB buffer — not buffering entire response in memory)
- **FR-002**: System MUST verify downloaded files against expected SHA256 hashes. On hash mismatch after resume, retry once from scratch before reporting failure.
- **FR-003**: System MUST emit progress events (bytes_downloaded, total_bytes, speed_bytes_per_sec) via the project event bus (existing `Event` enum + `tokio::sync::broadcast` channel, capacity 64). When total_bytes is unknown (no Content-Length), report 0 for total_bytes (indeterminate progress).
- **FR-004**: System MUST support resume via HTTP Range headers for failed downloads. Before resuming, validate the `.part` file size is consistent with the expected offset.
- **FR-005**: System MUST probe server resume support (send Range, check for 206 vs 200 response)
- **FR-006**: System MUST validate partial file freshness against server Last-Modified before resuming
- **FR-007**: System MUST download to a `.part` temp file (named `{final_filename}.part`), verify hash, then rename to final. If rename fails (file locked), retry up to 3 times with 1-second delay.
- **FR-008**: System MUST use ETag/Last-Modified conditional requests to skip re-downloading unchanged files on fresh downloads (not resume — resume uses Range)
- **FR-009**: System MUST support separate connect timeout (default 10s) and read timeout (default 30s), plus proxy settings, all from config (spec 004)
- **FR-010**: System MUST follow HTTP redirects (up to 10 hops). If exceeded, report error.
- **FR-011**: System MUST report download errors with the URL and HTTP status code
- **FR-012**: System MUST support cancellation — stop downloading when the user cancels. The download manager itself enforces sequential downloads (one at a time); concurrent requests are queued or rejected.
- **FR-013**: System MUST support configurable bandwidth throttling (bytes/sec, 0 = unlimited). Measured as a 5-second rolling average.
- **FR-014**: System MUST retain downloaded installers after installation by default. Download directory is flat: `{download_dir}/{filename}`.
- **FR-015**: System MUST support configurable auto-purge of installers older than N days (default: 30, 0 = disabled). Purge logic lives in the download manager module but is only invoked by the background service (spec 016).
- **FR-016**: System MUST only run auto-purge when the application is active (background/tray mode). The download manager exposes a `purge()` method; the caller decides when to invoke it.
- **FR-017**: System MUST check available disk space before starting a download (warn if < 2x file size). If Content-Length is unknown, skip the disk space check.
- **FR-018**: System MUST send `astro-up/{version}` User-Agent header using the crate version at build time (compile-time `env!("CARGO_PKG_VERSION")`)
- **FR-019**: System MUST auto-create the download directory if it doesn't exist before starting a download

### Key Entities

- **DownloadRequest**: URL, expected hash, destination path, resume flag
- **DownloadProgress**: bytes_downloaded, total_bytes, speed, elapsed, estimated_remaining
- **DownloadResult**: Success(path, hash_verified) or Error(reason)
- **ThrottleConfig**: max_bytes_per_sec (0 = unlimited), from AppConfig
- **PurgeConfig**: max_age_days (0 = disabled), from AppConfig

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Downloads achieve at least 80% of available bandwidth when no throttle is set
- **SC-002**: Hash verification adds less than 1 second overhead for files under 500MB
- **SC-003**: Resume saves download time proportional to the already-downloaded portion
- **SC-004**: Progress events are emitted at least once per second during active downloads
- **SC-005**: Bandwidth throttling stays within 10% of the configured limit (measured as 5-second rolling average)

## Default Values

| Setting | Default | Config Key |
|---------|---------|------------|
| Bandwidth limit | 0 (unlimited) | `network.download_speed_limit` |
| Installer retention | enabled | `paths.keep_installers` |
| Auto-purge age | 30 days | `paths.purge_installers_after_days` |
| Connect timeout | 10 seconds | `network.connect_timeout` |
| Read timeout | 30 seconds | `network.timeout` |
| Streaming chunk size | 64 KB | (not configurable) |
| Broadcast channel capacity | 64 | (not configurable) |

## Clarifications

### Session 2026-03-30

- Q: How do consumers (CLI, GUI) receive progress events? → A: Via the existing project event bus — `Event` enum (spec 003) with `DownloadStarted`/`DownloadProgress`/`DownloadComplete` variants, delivered through `tokio::sync::broadcast` channels (multi-producer, multi-consumer). Replaces the originally-planned flume dependency — all consumers already run tokio. Lagged receivers skip stale progress events gracefully.

## Assumptions

- Download URLs come from the catalog version entries (spec 005)
- The download directory path is configured in spec 004 (`paths.download_dir`), with config key names consistent with spec 004's naming convention (`network.*` for network settings, `paths.*` for directory settings)
- Progress events feed into the CLI progress bar (spec 015) and GUI progress indicator (spec 017)
- Sequential downloads enforced by the download manager itself (one active download at a time). Concurrent callers receive a "download in progress" error. The orchestration engine (spec 012) is the primary caller but other callers must handle rejection.
- Auto-purge logic lives in the download manager module as a `purge()` method. The background service (spec 016) calls it on a schedule. The download manager does not own the scheduling.
- Depends on: spec 004 (config for proxy/timeouts/paths/throttle/purge), spec 005 (catalog for download URLs)
