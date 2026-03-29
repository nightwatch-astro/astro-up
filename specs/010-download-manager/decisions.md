# Decisions Report: 010-download-manager
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions
- **Streaming downloads**: reqwest streaming response, not buffering in memory. Critical for large installers (500MB+).
- **Temp file then atomic move**: Download to `.part` file, verify hash, rename to final. Prevents partial files from being used.
- **SHA256 only**: No MD5 or SHA1. SHA256 is the standard in the manifest version entries.
- **Progress via channel/callback**: Emit DownloadProgress events that consumers (CLI/GUI) subscribe to. Decouples download from display.

## Questions I Would Have Asked
- Q1: Should we support parallel chunk downloads? Decision: No — adds complexity, most servers don't support it well. Single stream is sufficient.
- Q2: Should downloads be queued or parallel? Decision: Sequential by default, configurable concurrency. Avoids bandwidth contention.

## Clarify-Phase Decisions

### C1: .part file pattern for atomicity
**Decision**: Download to `.part`, verify, rename. This prevents partially downloaded files from being mistaken for complete downloads. On crash recovery, any `.part` file is a known incomplete download.

### C2: Progress events every 100ms or 64KB
**Decision**: 100ms ensures smooth UI updates. 64KB ensures progress on slow connections even within the time window. Both thresholds are checked — whichever triggers first emits an event.

### C3: Proceed without hash if unavailable
**Decision**: Some versions are too new for the manifest CI to have computed a hash. Blocking downloads on missing hashes would break the update flow. Warn instead. Users who care about integrity can wait for the hash to be published.

### C4: 2x disk space margin
**Decision**: Need space for both the .part temp file and the final file simultaneously (during rename). 2x Content-Length is the minimum. Warn early to avoid wasted bandwidth.

### C5: Sequential downloads by default
**Decision**: One download at a time. Parallel downloads add complexity (bandwidth contention, multiple progress bars) for marginal speed gain on residential connections. The orchestration engine (spec 012) controls sequencing.
