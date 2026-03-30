# Research: 010-download-manager

**Date**: 2026-03-30

## R1: HTTP Streaming with reqwest

**Decision**: Use `reqwest` 0.13 (already a dependency) with `chunk()` streaming API.

**Rationale**: reqwest is already integrated for catalog fetching. The `response.chunk().await?` API streams body data in fixed-size chunks without buffering the entire response. Supports:
- `bytes_stream()` via `futures_util::StreamExt` (not needed — `chunk()` is simpler and avoids the extra dep)
- `connect_timeout()`, `read_timeout()`, `timeout()` per-request and per-client
- `redirect::Policy::limited(10)` — matches FR-010
- Proxy via `Proxy::http()` / `Proxy::all()` — matches FR-009
- Custom `User-Agent` header via `user_agent()` builder method
- rustls TLS (already configured, no OpenSSL)

**Alternatives considered**:
- `ureq`: Blocking, no streaming. Rejected.
- `hyper` directly: Lower-level, reqwest is the right abstraction. Rejected.

## R2: SHA256 Hashing

**Decision**: Use `sha2` crate (blessed.rs, lib.rs) with incremental `Sha256::update()`.

**Rationale**: Hash each chunk as it streams to disk — zero extra memory and no second pass over the file. The `Digest` trait provides `update(data)` for streaming and `finalize()` for the final hash. Hex encoding via `hex` crate or `format!("{:x}", hash)`.

**Alternatives considered**:
- `ring`: Heavier, C dependencies. SHA256-only use doesn't justify it.
- `openssl`: Would reintroduce system dependency. Rejected.

## R3: Progress Events via tokio::sync::broadcast

**Decision**: Use existing `tokio::sync::broadcast` channel (tokio already a dependency) with `Event` enum.

**Rationale**: Broadcast channels are multi-producer, multi-consumer — CLI and GUI can both subscribe. The existing `Event` enum already has `DownloadStarted`, `DownloadProgress`, `DownloadComplete` variants. Key behaviors:
- `broadcast::channel(64)` capacity — lagged receivers get `RecvError::Lagged` and skip stale progress events (acceptable: progress is transient)
- `Sender::subscribe()` creates new receivers after channel creation
- Both `Sender` and `Receiver` are `Send + Sync` (required for cross-task use)
- No new dependency needed

**Note**: The events.rs comment mentions "flume channels" but the spec clarification settled on `tokio::sync::broadcast`. The compile-time assertion comment should be updated.

## R4: Bandwidth Throttling Approach

**Decision**: Sleep-between-chunks with a rolling window, not a token bucket crate.

**Rationale**: For sequential downloads with a 64KB chunk size, the simplest approach is:
1. After each chunk write, check elapsed time vs. expected time at the throttle rate
2. If ahead of schedule, `tokio::time::sleep()` for the difference
3. Measure speed as a 5-second rolling average (ring buffer of timestamps+bytes)

This avoids adding a `governor` or `leaky-bucket` dependency for a single-consumer, single-producer scenario. The 10% accuracy target (SC-005) is achievable with millisecond-resolution sleeps.

**Alternatives considered**:
- `governor` crate (token bucket): Designed for multi-consumer rate limiting. Overkill for sequential file downloads.
- `leaky-bucket`: Same — designed for request rate limiting, not byte-rate throttling.

## R5: Disk Space Check

**Decision**: Use `sysinfo` crate (already a dependency) for available disk space.

**Rationale**: `sysinfo::Disks` provides available space per mount point. Check `available_space >= 2 * content_length` before downloading (FR-017). Skip if Content-Length is unknown.

**Alternative**: `fs2::available_space()` — simpler but adds a dependency. `sysinfo` is already present.

## R6: Config Keys to Add (spec 004 iterate)

**Decision**: Three new config keys needed, to be added via spec 004 iterate:

| Key | Type | Default | Location |
|-----|------|---------|----------|
| `network.connect_timeout` | Duration | 10s | NetworkConfig |
| `network.download_speed_limit` | u64 | 0 (unlimited) | NetworkConfig |
| `paths.keep_installers` | bool | true | PathsConfig |
| `paths.purge_installers_after_days` | u32 | 30 (0 = disabled) | PathsConfig |

**Note**: `network.timeout` (30s read timeout) already exists. `network.connect_timeout` is new — reqwest supports it natively via `connect_timeout()`.

## R7: Cancellation

**Decision**: Use `tokio_util::sync::CancellationToken` (from `tokio-util`, lib.rs).

**Rationale**: The download loop checks `token.is_cancelled()` after each chunk. On cancellation, the `.part` file is left on disk for resume. `CancellationToken` is the standard tokio pattern — cooperative, no unsafe, cloneable.

**Alternative**: `tokio::select!` with a oneshot channel. Works but less ergonomic for the caller. CancellationToken is idiomatic.

## R8: Rename Retry on Windows

**Decision**: Retry `fs::rename()` up to 3 times with 1-second `tokio::time::sleep()` delay.

**Rationale**: Windows file locks (antivirus, indexer) can temporarily block renames. Three retries with 1-second delay covers transient locks. If still locked, report the error with the blocking path. No crate needed — just a loop.
