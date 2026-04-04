# Tasks: GUI File/Directory Browser Picker

**Input**: Design documents from `/specs/025-file-picker/`
**Prerequisites**: plan.md, spec.md, research.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Install frontend dependency and create shared composable

- [x] T001 Install `@tauri-apps/plugin-dialog` npm package in `frontend/package.json`
- [x] T002 Create `useFilePicker` composable with `pickDirectory()` and `pickLogFile()` functions in `frontend/src/composables/useFilePicker.ts`

**Checkpoint**: Dialog API available to components

---

## Phase 2: User Story 1 - Browse for a Directory (Priority: P1)

**Goal**: Add browse buttons to all directory path fields in PathsSection

**Independent Test**: Click browse button next to any directory field, select a folder, verify path populates and auto-saves

- [x] T003 [US1] Add `data_dir` field as InputGroup (InputText + browse Button) to `frontend/src/components/settings/PathsSection.vue`
- [x] T004 [US1] Convert existing `download_dir` and `cache_dir` InputText fields to InputGroup pattern with browse Button in `frontend/src/components/settings/PathsSection.vue`
- [x] T005 [US1] Wire browse buttons to `pickDirectory()` from `useFilePicker` composable, passing current path as defaultPath, in `frontend/src/components/settings/PathsSection.vue`

**Checkpoint**: All 3 directory fields have working browse buttons

---

## Phase 3: User Story 2 - Browse for a Log File (Priority: P2)

**Goal**: Add browse button to log file path in LoggingSection

**Independent Test**: Enable "Log to file", click browse button, select/name a `.log` file, verify path populates and auto-saves

- [x] T006 [US2] Replace read-only `<code>` display with InputGroup (InputText + browse Button) for `log_file` in `frontend/src/components/settings/LoggingSection.vue`
- [x] T007 [US2] Wire browse button to `pickLogFile()` from `useFilePicker` composable, passing current log_file as defaultPath, in `frontend/src/components/settings/LoggingSection.vue`

**Checkpoint**: Log file field has working browse button with `.log` filter

---

## Phase 4: Polish & Cross-Cutting Concerns

- [x] T008 Run `pnpm lint` and `pnpm vue-tsc --noEmit` to verify no type or lint errors in `frontend/`
- [ ] T009 Verify all 4 browse buttons work with cancel (no path change) and selection (path updates + auto-saves) via manual testing [MANUAL]

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **US1 (Phase 2)**: Depends on T001 + T002 (composable must exist)
- **US2 (Phase 3)**: Depends on T001 + T002 (composable must exist). Independent of US1.
- **Polish (Phase 4)**: Depends on all phases complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Setup only. No dependency on US2.
- **User Story 2 (P2)**: Depends on Setup only. No dependency on US1.

### Within Each User Story

- T003 before T004 (add new field before refactoring existing fields — cleaner diff)
- T004 before T005 (InputGroup structure must exist before wiring handlers)
- T006 before T007 (InputGroup structure must exist before wiring handler)

### Parallel Opportunities

- US1 and US2 can run in parallel after Setup completes (different components)
- T003 and T006 can run in parallel [P] (different files)

## Task Dependencies

```toml
[graph]
T001 = { blocked_by = [] }
T002 = { blocked_by = ["T001"] }
T003 = { blocked_by = ["T002"] }
T004 = { blocked_by = ["T003"] }
T005 = { blocked_by = ["T004"] }
T006 = { blocked_by = ["T002"] }
T007 = { blocked_by = ["T006"] }
T008 = { blocked_by = ["T005", "T007"] }
T009 = { blocked_by = ["T008"] }
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: US1 (T003-T005)
3. **STOP and VALIDATE**: Test directory browse buttons independently

### Incremental Delivery

1. Setup → composable ready
2. Add US1 → 3 directory browse buttons functional
3. Add US2 → log file browse button functional
4. Polish → lint clean, manual verification
