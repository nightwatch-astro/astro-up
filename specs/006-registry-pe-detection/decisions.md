# Decisions Report: 006-registry-pe-detection (Software and Driver Detection)

**Created**: 2026-03-29
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: pelite for cross-platform PE detection
**Choice**: pelite crate. Works on Linux/macOS CI without Windows.

### D2: Check both registry views unconditionally
**Choice**: Always check native and WOW6432Node. Many astro apps are 32-bit.

### D3: Version coercion via spec 003 Version::parse()
**Choice**: Reuse lenient parser. Strip 4th component, pad missing.

### D4: ASCOM Platform 7 minimum
**Choice**: Require ASCOM Platform 7+. Current version, maintains profile registry. Older EOL.

## Clarify-Phase Decisions (Interactive)

### C1: Event-driven detection cache
**Decision**: In-memory cache, invalidated per-package on install/update, fully on explicit `scan`. External changes caught on next scan.

### C2: Install ledger is a path resolver, not a detection method
**Decision**: Detection chain stays: registry → pe_file → file_exists. PE resolves paths from manifest config OR ledger. Ledger provides path, PE provides version. Catches self-updates.

### C3: Default chain with manifest override
**Decision**: Fixed default: registry → pe_file → file_exists. Manifest overrides for edge cases (ASCOM Profile, WMI, config_file).

### C4: Alpaca dropped — ASCOM detection is registry-only
**Decision**: ASCOM Alpaca is a device control protocol, not driver management. Spec 009 (Alpaca) dropped entirely. ASCOM drivers detected via Profile registry keys only.

### C5: WMI folded into this spec (was spec 007)
**Decision**: WMI driver detection is another detection method, not a separate spec. Added `method = "wmi"` / `method = "driver_store"` to the detection chain. Used for USB serial drivers (FTDI, CP210x) and driver packs (ZWO, QHY) that may not have uninstall registry keys.
**Reasoning**: Same consumer (detection chain), same purpose (detect installed version). Different API but same abstraction level. Keeps detection in one spec.

### C6: VID:PID matching for brownfield hardware discovery
**Decision**: Keep VID:PID matching in this spec. Primary use case: brownfield onboarding — user has hardware connected, astro-up suggests relevant driver packages. Wildcard matching (`03C3:*` for all ZWO products). Also used by manifest repo CI to validate `[hardware]` sections. Device connection notifications (hotplug) deferred.
**Reasoning**: Brownfield is the key use case. Without VID:PID, users must manually associate hardware with driver packages. CI validation is a bonus.

### C7: Package version vs individual driver version
**Decision**: Use whatever version the detection method returns. For WMI, that's the individual driver version Windows loaded. For registry, that's the package installer version. Don't reconcile them — the version comparison against the catalog still works either way.
**Reasoning**: The catalog tracks the package version from the vendor's download page. If the individual driver version is "close enough" to identify whether an update is needed, that's sufficient.

## Dropped Specs

- **Spec 007 (WMI Driver Detection)**: Folded into this spec. Branch `007-wmi-driver-detection` can be deleted.
- **Spec 009 (ASCOM Alpaca Client)**: Dropped entirely. Alpaca is for device control, not driver management. Branch `009-alpaca-client` can be deleted.

## Questions I Would Have Asked

### Q1: Should we detect apps installed by other package managers?
**My decision**: No — we detect via registry/PE/WMI regardless of install source.

### Q2: Should detection report the install method?
**My decision**: No — detection reports presence and version only. Install method comes from the manifest.

### Q3: Should VID:PID matching auto-install suggested driver packages?
**My decision**: No — discovery suggests packages, user confirms. No auto-install from hardware detection. That would be surprising and potentially dangerous.
