# Decisions Report: 014-custom-tools
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: GitHub-only for v1
**Choice**: Only support GitHub repos, not GitLab or arbitrary URLs.
**Reasoning**: GitHub has a standard Releases API. Supporting arbitrary URLs would require per-site scraping logic. GitLab can be added later using the same pattern.

### D2: Remove deletes manifest, not software
**Choice**: `remove` only removes the tracking manifest, not the installed application.
**Reasoning**: Uninstalling is a separate, risky operation. Users may want to stop tracking a tool without uninstalling it. An explicit `uninstall` command (future) handles actual removal.

### D3: Auto-detect install method from filename heuristics
**Choice**: Infer from filename patterns, not from binary analysis.
**Reasoning**: Filename patterns (Setup.exe → installer, .msi → MSI, .zip → ZIP) are reliable for 90%+ of astrophotography tools. Binary analysis (PE header inspection) is slower and more complex.

### D4: Custom tools stored in data_dir, not alongside official catalog
**Choice**: `{data_dir}/astro-up/custom/` separate from the catalog cache.
**Reasoning**: Custom manifests are user data (persists across catalog updates). Official catalog is cache (replaceable). Different lifecycle = different storage.

## Clarify-Phase Decisions

### C1: Interactive asset selection in CLI, pre-selected in GUI
**Decision**: CLI lists assets and prompts user to pick. GUI shows a dialog with asset list. If only one Windows asset exists, auto-select it.

### C2: Generated manifest includes checkver.github for auto-updates
**Decision**: The generated TOML includes `checkver.github = "owner/repo"` so the custom tool gets update checking automatically. No manual checkver configuration needed.

### C3: No manifest editing UI in this spec
**Decision**: Users who need to customize the generated manifest edit the TOML file directly. A manifest editor UI is deferred to a future spec.

## Questions I Would Have Asked

### Q1: Should custom tools support non-GitHub sources (direct URLs)?
**My decision**: No for v1. Direct URLs lack a standard release/versioning API, making update checking impossible.
**Impact if wrong**: Users with tools only available via direct download can't use custom tools. They'd need to manually create TOML files.

### Q2: Should we validate that the downloaded asset actually installs correctly?
**My decision**: No — that's the install spec's (011) responsibility. Custom tools uses the same install pipeline as official packages.
