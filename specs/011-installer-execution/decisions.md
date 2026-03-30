# Decisions Report: 011-installer-execution

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: Per-type default switches, manifest replaces not merges
**Choice**: Each installer type has documented defaults. Manifest `[install.switches]` fully replaces them.
**Reasoning**: Merging is ambiguous. The manifest author knows their installer best.

### D2: Mandatory zip-slip guard
**Choice**: Every ZIP path validated. Paths with `..` rejected. Never optional.
**Reasoning**: Security — malicious ZIPs could write files outside the target directory.

### D3: Smart ZIP nesting detection
**Choice**: Single top-level dir → extract contents. Multiple → extract as-is.
**Reasoning**: Avoids `target/NINA-3.1/NINA-3.1/` double nesting.

### D4: Process tree waiting via Job Objects
**Choice**: Track child processes for bootstrapper-style installers.
**Reasoning**: Burn/NSIS bootstrappers spawn children and exit. Must wait for the whole tree.

## Clarify-Phase Decisions (Interactive)

### C1: Per-installer configurable timeout
**Finding**: User asked about per-installer timeout override.
**Decision**: `[install].timeout` field in manifest (seconds). Default 600 (10 min). Overridable for slow installers (.NET runtime, large driver packs).

### C2: Reboot — warn and offer choice, never auto-reboot
**Finding**: User suggested warning + choice instead of blanket "no reboot."
**Decision**: GUI: dialog "Reboot Now" / "Later." CLI: message + special exit code. Persistent reminder until reboot. Never auto-reboot — users may be mid-imaging session.

### C3: Uninstall for packages that support it
**Finding**: User confirmed uninstall coverage.
**Decision**: (1) Registry uninstall string → execute silently. (2) ZIP/portable → delete directory with confirmation. (3) No uninstaller + no known path → "not supported." `upgrade_behavior = "uninstall_previous"` uses this.

### C4: Installer type always explicit
**Finding**: User confirmed no auto-detection from file.
**Decision**: Manifest MUST specify `[install].method`. Missing = invalid manifest. No PE inspection, no filename heuristics. Deterministic behavior.

### C5: WebView2 is Tauri's job
**Decision**: Not this spec's concern. Handled by Tauri NSIS installer (spec 019).

### C6: Record installs in ledger
**Decision**: Write LedgerEntry after successful install (package_id, version, path, timestamp). Feeds detection (spec 006) for portable apps.

## Questions I Would Have Asked

### Q1: Should we support rollback on install failure?
**My decision**: No for v1. Complex and risky. Backup (spec 013) preserves config. If install fails, user re-runs.

### Q2: Pre/post hooks — PowerShell or cmd?
**My decision**: Both. Detect `.ps1` for PowerShell, otherwise cmd. 60-second timeout. Pre-install failure aborts; post-install failure warns.
