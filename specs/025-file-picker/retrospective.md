# Retrospective: 025-file-picker

**Date**: 2026-04-04
**Spec Adherence**: 100% (14/14 requirements)
**Task Completion**: 8/8 code tasks + 1 manual (T009 pending)
**Drift**: 0%
**Conflicts**: 0

## Outcome

Clean implementation of native OS file/directory browse buttons for all 4 path fields in Settings. Minimal footprint: 1 new composable (23 lines), modifications to 2 existing components, 1 new npm dependency.

## What Went Well

1. **Spec was well-scoped**: Small, self-contained feature with clear boundaries. No Rust changes needed — leveraged existing Tauri dialog plugin registration from spec 016.
2. **Composable pattern**: `useFilePicker.ts` cleanly wraps the dialog API, avoiding duplication across components. Good balance — not over-abstracted (no unnecessary `PathInput.vue` component).
3. **Zero drift**: Implementation matched spec exactly. All 9 FRs and 5 SCs verified.
4. **Existing infrastructure reuse**: Auto-save via deep watch, config types, InputGroup pattern — all from prior specs. No new plumbing needed.

## What Could Be Improved

1. **Process gap: GitHub issues not closed during implementation**. The implementing agent marked `[X]` in tasks.md (violating `HAS_PROJECT` rules) but didn't close GitHub issues #717-#724. Issues remain open despite code being complete.
   - **Impact**: Project board shows incorrect status. Verify-tasks querying closed issues would find zero completed tasks.
   - **Root cause**: Implement step agent wasn't properly following `HAS_PROJECT` workflow for issue closure.
   - **Action**: Close issues via PR body (`fixes #717, fixes #718, ...`) when merging.

2. **Package install in wrong CWD**: Initial commit missed `@tauri-apps/plugin-dialog` in `package.json`, requiring a follow-up fix commit (`14969ca`). The `pnpm add` ran in the repo root instead of `frontend/`.
   - **Impact**: Minor — caught immediately, one extra commit.
   - **Root cause**: Agent didn't `cd frontend/` before running `pnpm add`.

## Findings

### Cross-cutting (applicable to future specs)

1. **`HAS_PROJECT` issue closure must happen during implementation, not deferred to PR merge.** When implementing with `HAS_PROJECT = true`, each task completion should close the corresponding issue immediately via `gh issue close`. Relying on PR body `fixes #N` delays status visibility on the project board. This is especially important for multi-task specs where the board should reflect incremental progress.

2. **Frontend `pnpm` commands must run in `frontend/` directory.** The project has a nested `frontend/` workspace — `pnpm add` from repo root fails silently or installs in the wrong location.

### Spec-specific (no cross-cutting action)

3. **Type assertion for `keyof PathsConfig`**: The `browseDirectory` function uses `keyof PathsConfig` which includes non-string fields, requiring a runtime `typeof` guard and type assertion. Safe but could be made stricter with a string-key union type. Not worth changing now — the function is only called with 3 known string fields.

## Metrics

| Metric | Value |
|--------|-------|
| Commits | 4 (spec + implement + fix + docs) |
| Files changed | 5 (composable, 2 components, package.json, tasks.md) |
| Lines added | ~60 |
| Lines removed | ~15 |
| New dependencies | 1 (@tauri-apps/plugin-dialog) |
| Quality gate issues | 0 |
