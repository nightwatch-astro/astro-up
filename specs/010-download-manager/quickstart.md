# Quickstart: 010-download-manager

## Module Location

```
crates/astro-up-core/src/download/
├── mod.rs          # DownloadManager, public API, re-exports
├── types.rs        # DownloadRequest, DownloadResult, DownloadProgress, PurgeResult
├── client.rs       # reqwest client builder from AppConfig
├── stream.rs       # Streaming download loop, hash, resume, throttle
└── purge.rs        # Installer retention and auto-purge logic
```

## New Dependencies

Add to `crates/astro-up-core/Cargo.toml`:

```toml
sha2 = "0.10"              # SHA256 streaming hash (blessed.rs, lib.rs)
tokio-util = { version = "0.7", features = ["rt"] }  # CancellationToken
```

Already present: `reqwest` (0.13), `tokio` (1, add `sync` feature), `sysinfo` (0.38), `chrono` (0.4).

## Config Changes (spec 004 iterate)

Add to `NetworkConfig`:
```rust
pub connect_timeout: Duration,          // default: 10s
pub download_speed_limit: u64,          // default: 0 (unlimited), bytes/sec
```

Add to `PathsConfig`:
```rust
pub keep_installers: bool,              // default: true
pub purge_installers_after_days: u32,   // default: 30, 0 = disabled
```

## Event Bus Integration

The download manager receives a `broadcast::Sender<Event>` at construction. During downloads it sends:
- `Event::DownloadStarted { id, url }` — before first byte
- `Event::DownloadProgress { id, progress, bytes_downloaded, total_bytes, speed }` — every 100ms or 64KB
- `Event::DownloadComplete { id }` — after hash verify + rename

No new event variants needed — existing ones cover the contract.

## Error Integration

Add download-specific variants to `CoreError`:
- `DownloadFailed { url, status, reason }` — HTTP errors
- `DiskSpaceInsufficient { required, available }` — pre-download check
- `DownloadInProgress { url }` — sequential lock contention
- `RenameFailed { from, to, cause }` — .part rename failure

Existing variant used: `ChecksumMismatch { expected, actual }`.

## Build & Test

```sh
just check   # clippy + fmt + test
just test    # cargo test -p astro-up-core
```

Integration tests use `httptest` or `wiremock` crate for a local HTTP server.
