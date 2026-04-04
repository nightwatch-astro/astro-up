# Implementation Plan: GUI File/Directory Browser Picker

**Branch**: `025-file-picker` | **Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/025-file-picker/spec.md`

## Summary

Add native OS file/directory browse buttons to all path fields in Settings. Uses the Tauri dialog plugin (already installed on backend) with its frontend JS API (`open()` for directories, `save()` for log file). Affects PathsSection (3 directory fields) and LoggingSection (1 file field).

## Technical Context

**Language/Version**: Rust 2024 edition (backend, no changes needed) + TypeScript 5 / Vue 3 (frontend)
**Primary Dependencies**: `@tauri-apps/plugin-dialog` (new frontend dep), PrimeVue 4 (InputGroup pattern)
**Storage**: N/A (uses existing config save flow)
**Testing**: Manual testing (native OS dialogs cannot be automated in unit tests)
**Target Platform**: Windows (primary), macOS/Linux (cross-platform via Tauri)
**Project Type**: Desktop app (Tauri v2)
**Performance Goals**: N/A (single dialog invocation, no performance concerns)
**Constraints**: None beyond existing project constraints
**Scale/Scope**: 4 path fields across 2 Vue components

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | No Rust changes needed — dialog plugin already registered |
| II. Platform Awareness | PASS | Tauri dialog API is cross-platform; no `cfg(windows)` needed |
| III. Test-First | PASS | Native OS dialogs require manual testing; no mock-worthy interfaces |
| IV. Thin Tauri Boundary | PASS | No new Tauri commands — frontend calls dialog plugin JS API directly |
| V. Spec-Driven | PASS | Spec written and clarified |
| VI. Simplicity | PASS | Minimal changes: add npm package, add browse buttons to 2 components |

No violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/025-file-picker/
├── spec.md
├── plan.md              # This file
├── research.md          # Phase 0 output
└── checklists/
    └── requirements.md
```

### Source Code (repository root)

```text
frontend/
├── src/
│   ├── components/
│   │   └── settings/
│   │       ├── PathsSection.vue   # Add browse buttons for download_dir, cache_dir, data_dir
│   │       └── LoggingSection.vue # Add browse button for log_file, make path editable
│   └── composables/
│       └── useFilePicker.ts       # New: shared composable wrapping dialog API
└── package.json                   # Add @tauri-apps/plugin-dialog
```

**Structure Decision**: All changes in `frontend/` — no Rust/backend modifications. New composable `useFilePicker.ts` encapsulates the dialog API calls to avoid duplicating `open()`/`save()` setup in each component.

## Design

### Approach

1. **New composable `useFilePicker.ts`**: Wraps `@tauri-apps/plugin-dialog` `open()` and `save()` calls. Provides two functions:
   - `pickDirectory(defaultPath?: string): Promise<string | null>` — calls `open({ directory: true, defaultPath, title: "Select Directory" })`
   - `pickLogFile(defaultPath?: string): Promise<string | null>` — calls `save({ defaultPath, filters: [{ name: "Log files", extensions: ["log"] }], title: "Select Log File" })`

2. **PathsSection.vue changes**:
   - Import `InputGroup` and `InputGroupAddon` from PrimeVue
   - Wrap each path InputText in an InputGroup with a Button (folder icon)
   - Add `data_dir` field (same pattern as `download_dir`/`cache_dir`)
   - Button click calls `pickDirectory(config.fieldName)`, updates model on non-null result

3. **LoggingSection.vue changes**:
   - Replace `<code>` display with InputGroup (InputText + Button)
   - Button click calls `pickLogFile(config.log_file)`, updates model on non-null result
   - Keep conditional visibility (`v-if="config.log_to_file"`)

### Key Decision: Composable vs Inline

Using a composable rather than inlining dialog calls because:
- 4 fields need the same pattern (DRY)
- Centralizes the `@tauri-apps/plugin-dialog` import
- Easy to add new path fields in future specs
- Testable in isolation (mock the import)

Alternative rejected: Creating a reusable `PathInput.vue` component. While cleaner, it would require refactoring the existing InputText bindings and event flow in both sections. The composable approach is simpler and lower risk.
