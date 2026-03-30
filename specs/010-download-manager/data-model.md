# Data Model: 010-download-manager

**Date**: 2026-03-30

## Entities

### DownloadRequest

Input to the download manager. Constructed by the orchestration engine from catalog data.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `url` | `String` | yes | Download URL from `VersionEntry.url` |
| `expected_hash` | `Option<String>` | no | SHA256 hex string from `VersionEntry.sha256` |
| `dest_dir` | `PathBuf` | yes | Target directory (from `AppConfig.paths.download_dir`) |
| `filename` | `String` | yes | Final filename (extracted from URL or provided) |
| `resume` | `bool` | yes | Whether to attempt resume (default: true) |

**Derived**: `dest_path()` → `dest_dir.join(filename)`, `part_path()` → `dest_dir.join(format!("{filename}.part"))`

### DownloadProgress

Emitted via broadcast channel during active download. Maps to existing `Event::DownloadProgress`.

| Field | Type | Description |
|-------|------|-------------|
| `bytes_downloaded` | `u64` | Total bytes written to disk so far |
| `total_bytes` | `u64` | Content-Length from server (0 = unknown) |
| `speed_bytes_per_sec` | `f64` | Current speed (5-second rolling average) |
| `elapsed` | `Duration` | Time since download started |
| `estimated_remaining` | `Option<Duration>` | `None` if total_bytes is 0 |

### DownloadResult

Return value from the download operation.

```rust
pub enum DownloadResult {
    /// File downloaded (or already cached) and verified.
    Success {
        path: PathBuf,
        hash_verified: bool,
        bytes_downloaded: u64,
        resumed: bool,
    },
    /// File already exists and matches expected hash — skipped download.
    Cached { path: PathBuf },
}
```

### DownloadError

Download-specific errors (extends `CoreError`).

| Variant | Fields | Description |
|---------|--------|-------------|
| `DownloadFailed` | `url, status: u16, reason` | Non-2xx response, network error, or redirect overflow |
| `ChecksumMismatch` | `expected, actual` | SHA256 verification failed (existing CoreError variant) |
| `DiskSpaceInsufficient` | `required, available` | Less than 2x file size available |
| `RenameFailed` | `from, to, cause: Box<dyn Error>` | `.part` → final rename failed after retries |
| `DownloadInProgress` | `url` | Sequential download lock contention |
| `Cancelled` | — | User cancelled via CancellationToken |

### ThrottleState (internal)

Not exposed publicly. Used by the download loop for bandwidth control.

| Field | Type | Description |
|-------|------|-------------|
| `max_bytes_per_sec` | `u64` | From config (0 = unlimited) |
| `window` | `VecDeque<(Instant, u64)>` | Rolling 5-second window of (timestamp, bytes) pairs |

### PurgeConfig (from AppConfig)

Read from existing config system. Not a new struct — values come from `AppConfig.paths`.

| Config Key | Type | Default |
|------------|------|---------|
| `paths.keep_installers` | `bool` | `true` |
| `paths.purge_installers_after_days` | `u32` | `30` (0 = disabled) |

## Relationships

```
VersionEntry (spec 005)
  └─ url, sha256 ──→ DownloadRequest
                        │
                        ▼
AppConfig (spec 004)
  ├─ paths.download_dir ──→ DownloadRequest.dest_dir
  ├─ network.* ──→ reqwest::Client builder
  └─ paths.purge_* ──→ purge() method
                        │
                        ▼
Event (spec 003)
  ├─ DownloadStarted { id, url }
  ├─ DownloadProgress { id, progress, bytes_downloaded, total_bytes, speed, elapsed, estimated_remaining }
  └─ DownloadComplete { id }

Filesystem artifacts:
  download_dir/
  ├─ {filename}.part    ── in-progress download
  ├─ {filename}         ── completed download
  └─ {filename}.etag    ── cached ETag for conditional requests (FR-008)
```

## State Transitions

```
[No file] ──download()──→ [.part file writing]
                              │
                    ┌─────────┼──────────┐
                    ▼         ▼          ▼
               [Cancelled] [Error]  [.part complete]
               (keep .part) (keep    │
                            .part)   ▼
                                  [Hash check]
                                  │         │
                                  ▼         ▼
                              [Mismatch] [Match]
                              │              │
                              ▼              ▼
                          [Delete .part] [Rename to final]
                          [Retry once]       │
                                             ▼
                                         [Success]

[.part exists] ──resume()──→ [Probe server]
                                │         │
                                ▼         ▼
                           [206 Partial] [200 Full]
                           [Append]      [Restart]
```
