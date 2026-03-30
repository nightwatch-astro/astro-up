# Decisions Report: 010-download-manager

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: Streaming downloads via reqwest
**Choice**: reqwest streaming response, not buffering in memory.
**Reasoning**: Critical for large installers (500MB+). Stream to disk, hash incrementally.

### D2: .part file pattern for atomicity
**Choice**: Download to `.part`, verify hash, rename to final.
**Reasoning**: Prevents partial files from being mistaken for complete downloads. On crash, `.part` = known incomplete.

### D3: Sequential downloads, not parallel
**Choice**: One download at a time.
**Reasoning**: Parallel downloads add complexity (bandwidth contention, multiple progress bars). Orchestration engine controls sequencing.

### D4: SHA256 only
**Choice**: No MD5 or SHA1.
**Reasoning**: SHA256 is the standard in catalog version entries. One algorithm, no negotiation.

## Clarify-Phase Decisions (Interactive)

### C1: HTTP resume with server probe
**Finding**: User asked "can we check whether we can add http resume if a download fails?"
**Decision**: Yes — on retry, check for existing `.part` file. Send Range header. If server returns 206, append. If 200, restart. Also check Last-Modified to ensure the file hasn't changed since the partial download.
**Reasoning**: Large files (NINA ~500MB) on unstable connections (remote observatories) benefit significantly. The probe is one extra HEAD request — negligible overhead.

### C2: Installer retention with configurable auto-purge
**Finding**: User wanted cleanup as a configurable option with auto-purge on schedule.
**Decision**: Keep installers after install by default. Auto-purge removes files older than N days (default: 30). Purge only runs when the app is active in background/tray mode. Setting 0 disables purge.
**Reasoning**: Keeps the ability to re-install offline while preventing unbounded disk usage. Background-only purge means no surprises when the user launches the app to do work.

### C3: Bandwidth throttling
**Finding**: User confirmed throttling is needed for imaging sessions.
**Decision**: Configurable `network.download_speed_limit` in bytes/sec (0 = unlimited). Applied per-download via a token bucket or sleep-between-chunks pattern.
**Reasoning**: Active imaging sessions depend on network stability for plate solving, PHD2 corrections, and remote desktop. Unthrottled downloads can disrupt all of these.

### C4: New config keys needed in spec 004
**Finding**: This spec introduces three new config keys not in spec 004's defaults table.
**Decision**: Add to spec 004 during iterate: `network.download_speed_limit`, `paths.keep_installers`, `paths.purge_installers_after_days`.
**Action**: Update spec 004 via iterate when implementing this spec.

### C5: Progress events every 100ms or 64KB
**Decision**: Whichever triggers first. 100ms ensures smooth UI. 64KB ensures progress on slow connections within the time window.

## Questions I Would Have Asked

### Q1: Should we support download mirrors / fallback URLs?
**My decision**: No — single URL from catalog. If it fails, retry the same URL. Mirrors add complexity for ~100 packages.

### Q2: Should partial downloads survive app restarts?
**My decision**: Yes — `.part` files persist on disk. On restart, the resume logic picks them up. No in-memory state needed.
