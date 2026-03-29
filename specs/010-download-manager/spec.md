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

**Why this priority**: Downloads are the most visible operation — users need feedback on what's happening.

**Independent Test**: Download a known file, verify progress events are emitted and the file is correctly saved.

**Acceptance Scenarios**:

1. **Given** a download URL for NINA installer, **When** download starts, **Then** progress events report bytes/total/speed at regular intervals
2. **Given** a slow connection, **When** downloading, **Then** the progress bar updates smoothly without stalling
3. **Given** the download completes, **When** verifying, **Then** the SHA256 hash matches the expected value from the catalog

---

### User Story 2 - Hash Verification (Priority: P2)

After downloading, the manager verifies the file's SHA256 hash against the expected hash from the version entry. If the hash doesn't match, the file is discarded and the user is informed.

**Why this priority**: Integrity verification prevents corrupted or tampered installers from being executed.

**Independent Test**: Download a file, verify hash matches. Corrupt the file, verify hash check fails.

**Acceptance Scenarios**:

1. **Given** a completed download, **When** the SHA256 matches, **Then** the file is moved to the destination
2. **Given** a completed download, **When** the SHA256 doesn't match, **Then** the file is deleted and an error is reported
3. **Given** no expected hash is available, **When** the download completes, **Then** it proceeds with a warning (no verification)

---

### User Story 3 - Resume Interrupted Downloads (Priority: P3)

If a download is interrupted (network drop, user cancel), the manager can resume from where it left off using HTTP Range headers instead of starting over.

**Why this priority**: Large installers (NINA ~500MB) benefit significantly from resume capability.

**Independent Test**: Start a download, interrupt at 50%, resume, verify the final file is complete and valid.

**Acceptance Scenarios**:

1. **Given** a partial download exists, **When** resuming, **Then** only the remaining bytes are fetched
2. **Given** the server doesn't support Range headers, **When** resuming, **Then** the download restarts from the beginning
3. **Given** the partial file is corrupted, **When** resuming, **Then** the download restarts from the beginning

### Edge Cases

- Server returns a redirect: Follow redirects (up to 10 hops).
- Server returns 403/404: Report a clear download error with the URL.
- Disk full during download: Detect and report before the OS crashes.
- Download via proxy: Use proxy settings from config (spec 004).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST download files via HTTP/HTTPS with streaming (not buffering entire response in memory)
- **FR-002**: System MUST verify downloaded files against expected SHA256 hashes
- **FR-003**: System MUST emit progress events (bytes_downloaded, total_bytes, speed_bytes_per_sec)
- **FR-004**: System MUST support resume via HTTP Range headers for interrupted downloads
- **FR-005**: System MUST download to a temporary file, verify hash, then move to destination (atomic)
- **FR-006**: System MUST use ETag/Last-Modified conditional requests to skip re-downloading unchanged files
- **FR-007**: System MUST support configurable request timeouts and proxy settings from config
- **FR-008**: System MUST follow HTTP redirects (up to 10 hops)
- **FR-009**: System MUST report download errors with the URL and HTTP status code
- **FR-010**: System MUST support cancellation — stop downloading when the user cancels

### Key Entities

- **DownloadRequest**: URL, expected hash, destination path, resume flag
- **DownloadProgress**: bytes_downloaded, total_bytes, speed, elapsed, estimated_remaining
- **DownloadResult**: Success(path, hash_verified) or Error(reason)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Downloads saturate available bandwidth (no artificial throttling)
- **SC-002**: Hash verification adds less than 1 second overhead for files under 500MB
- **SC-003**: Resume saves at least 50% of download time for interrupted large files
- **SC-004**: Progress events are emitted at least once per second during active downloads

## Assumptions

- Download URLs come from the catalog version entries (spec 005) or autoupdate templates (spec 008)
- The download directory is configured in spec 004
- Progress events feed into the CLI progress bar (spec 015) and GUI progress indicator (spec 017)
- Depends on: spec 004 (config for proxy/timeouts/paths), spec 005 (catalog for URLs), spec 008 (autoupdate URL resolution)
