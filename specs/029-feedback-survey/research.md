# Research: 029 User Feedback Survey

**Date**: 2026-04-09

## Decisions

### R-001: Survey state storage

**Decision**: Store survey state in the existing SQLite-backed `ConfigStore` via new fields on `UiConfig`.

**Rationale**: The config system already supports dot-path key access, `set_field`/`get_field_value`, and SQLite persistence. Adding 3 fields to `UiConfig` is the minimal approach — no new tables, no new modules.

**Alternatives considered**:
- Separate `survey` table in SQLite — overkill for 3 fields, adds migration complexity
- Local file (JSON/TOML) — diverges from existing config pattern, adds a second persistence mechanism

### R-002: Operation counting method

**Decision**: Use a direct SQL `COUNT(*)` query on the `operations` table filtered by `status = 'success'` and `operation_type IN ('install', 'update')`.

**Rationale**: The `operations` table already exists with indexes on `created_at`. A single SQL count query is simpler and more efficient than loading all `OperationRecord`s and filtering in Rust. The `query_history` function exists but returns full records — a dedicated count function avoids unnecessary deserialization.

**Alternatives considered**:
- Increment a counter in config on each operation — risks drift if operations complete while config write fails
- Use `query_history` with filter and count results — works but loads full records into memory unnecessarily

### R-003: Eligibility check trigger point

**Decision**: Check eligibility via a Tauri command invoked by the Dashboard view's `onMounted` hook. The command returns a boolean.

**Rationale**: Dashboard is the landing page and the most natural place for a non-intrusive prompt. Checking on mount (not on every re-render) prevents flicker. The Tauri command encapsulates all logic (count query + config state check) in core, keeping the frontend thin.

**Alternatives considered**:
- Check on app startup in Rust and emit event — adds complexity, dialog may appear before frontend is ready
- Periodic timer — unnecessary overhead for a one-shot check

### R-004: Dialog dismiss behavior

**Decision**: Closing the dialog without clicking any button (Escape, click outside) is treated as "Not now" — snooze for 30 days.

**Rationale**: Clarified with user during `/speckit.clarify`. Prevents the dialog from nagging on every Dashboard visit while respecting that the user didn't explicitly opt out.

### R-005: External form platform

**Decision**: Tally.so hosted form at `https://tally.so/r/lb7dd5`.

**Rationale**: No login required, free tier sufficient, no backend needed. Form already created by user. The app opens the URL in the default browser via `tauri-plugin-shell` (already integrated).

**Alternatives considered**:
- Google Forms — requires Google account styling, less clean
- In-app form with backend — requires infrastructure, overkill for expected volume
- GitHub Discussions — requires GitHub login, most users won't have accounts
