# Decisions Report: 015-cli-interface

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: ratatui for progress, not a full TUI app
**Choice**: Progress bars and tables during long operations only.
**Reasoning**: Not a persistent TUI. The app runs a command and exits.

### D2: Auto-detect TTY for output mode
**Choice**: TTY → colors + TUI. Piped → plain text. `--json` overrides both.

### D3: Standalone binary, no Tauri dependency
**Choice**: CLI links against astro-up-core + clap + ratatui only. No WebView.

### D4: Self-update via GitHub Releases
**Choice**: Binary download and swap. Separate from the GUI's tauri-plugin-updater.

## Clarify-Phase Decisions (Interactive)

### C1: install + update as separate commands
**Finding**: User pointed out that `update` for fresh install is confusing.
**Decision**: `install` installs new packages. If already installed, offers to update instead. `update` updates existing packages. If not installed, errors with "use `install`." Clear mental model: install = new, update = existing.

### C2: `show` as unified read command
**Finding**: User suggested `show` instead of overloaded `list`/`scan`/`check`.
**Decision**: `show` is the single read command with subvariants:
- `show` / `show all` — full catalog with install status
- `show installed` — installed only
- `show outdated` — packages with updates (replaces `check`)
- `show <package>` — detailed package info
- `show backups [package]` — backup archives

### C3: `scan` is active detection, `show` reads cache
**Finding**: Need to distinguish "go detect things" from "display what we know."
**Decision**: `scan` runs detection (registry, PE, WMI), updates cache. `show installed` reads the cache. On first run, `show` auto-triggers `scan`. After that, `scan` is explicit.

### C4: `search` for catalog FTS5 search
**Decision**: `search <query>` does full-text search across the catalog. Separate from `show` because it takes a free-form query, not a subvariant.

### C5: Backup under simple commands, backups listing under show
**Decision**: `backup nina` creates. `restore nina` restores (with picker). `show backups nina` lists. No subcommand group — simpler top-level commands.

### C6: JSON output on all commands
**Finding**: User confirmed JSON for all, including write commands.
**Decision**: `--json` works on every command. Write commands output structured results (per-package status, errors). Enables CI scripting: `astro-up update --all --json | jq '.failed'`.

### C7: `check` removed — replaced by `show outdated`
**Finding**: User confirmed `check` is redundant.
**Decision**: Removed. `show outdated` serves the same purpose with a clearer name.

## Questions I Would Have Asked

### Q1: Should `show` auto-refresh the catalog if TTL expired?
**My decision**: Yes — `show` triggers a catalog refresh (ETag conditional fetch) if TTL expired. This is fast (one HTTP request) and keeps data fresh without explicit user action.

### Q2: Should we support shell completion?
**My decision**: Yes — clap generates completions for bash, zsh, fish, PowerShell via `clap_complete`. Low cost, high UX value. Add `astro-up completions <shell>` to output the completion script.
