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
