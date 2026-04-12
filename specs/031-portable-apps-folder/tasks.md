# Tasks: Portable Apps Folder

**Input**: Design documents from `/specs/031-portable-apps-folder/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US5)

## Phase 1: Setup (Config Field)

**Purpose**: Add the `portable_apps_dir` config field across backend and frontend

- [ ] T001 [P] Add `portable_apps_dir: PathBuf` field to `PathsConfig` in `crates/astro-up-core/src/config/model.rs`
- [ ] T002 [P] Add default value for `portable_apps_dir` in `crates/astro-up-core/src/config/defaults.rs` — default to `{data_dir}/../apps/` resolving to `{AppData}/nightwatch/astro-up/apps/`
- [ ] T003 Add `config_set`/`config_get` match arms for `paths.portable_apps_dir` in `crates/astro-up-core/src/config/mod.rs`
- [ ] T004 Update config snapshot test in `crates/astro-up-core/tests/config/defaults_test.rs` — add `portable_apps_dir` to expected output and update key count

---

## Phase 2: Foundational (Install Handler Changes)

**Purpose**: Modify `handle_download_only` to copy/extract instead of opening Explorer. Wire the portable dir into the install pipeline.

- [ ] T005 Rewrite `handle_download_only` in `crates/astro-up-core/src/install/mod.rs` — replace Explorer open with: detect if file is zip (check magic bytes `PK\x03\x04`), extract if zip using existing `zip::extract_zip`, copy if single file. Create `{dest}/{package-id}/` dir. Return `InstallResult::Success { path: Some(dest) }`
- [ ] T006 In `crates/astro-up-gui/src/commands.rs` `run_orchestrated_operation_inner`, when install method is `DownloadOnly` or `Portable`, set `InstallRequest.install_dir = Some(config.paths.portable_apps_dir.join(package_id))`. Read `portable_apps_dir` from `config.paths`

**Checkpoint**: Download-only packages now land in the portable apps directory

---

## Phase 3: User Story 1 — Install download-only app to portable folder (Priority: P1)

**Goal**: `download_only` packages copy/extract to `{portable_apps_dir}/{package-id}/` with ledger recording the install path.

**Independent Test**: Install `ioptron-upgrade-utility-v3` on .111, verify file appears in `apps/` dir, verify ledger has install_path.

- [ ] T007 [US1] Verify the orchestrator passes `install_dir` through to `InstallRequest` for download-only packages — trace the flow in `crates/astro-up-core/src/engine/orchestrator.rs` from `plan` through `execute_single`. Fix if not threaded through. Also verify the CLI install command in `crates/astro-up-cli/src/commands/install.rs` passes `portable_apps_dir` from config.
- [ ] T008 [US1] Verify the `PackageComplete` event emits the portable dir path (not the temp download dir) via `download_path` field — check event emission in `crates/astro-up-core/src/engine/orchestrator.rs`
- [ ] T009 [US1] Verify the ledger `upsert_acknowledged` records `install_path` from `InstallResult::Success { path }` for download-only packages

**Checkpoint**: US1 complete — download-only packages install to portable dir with correct ledger and events

---

## Phase 4: User Story 2 — Configure the portable apps directory (Priority: P2)

**Goal**: Settings > Paths has a field for the portable apps directory.

**Independent Test**: Change path in Settings, install a download-only package, verify it uses the new path.

- [ ] T010 [P] [US2] Add `portable_apps_dir: string` to `PathsConfig` interface in `frontend/src/types/config.ts`
- [ ] T011 [P] [US2] Add `portable_apps_dir` validation in `frontend/src/validation/config.ts`
- [ ] T012 [P] [US2] Add default `portable_apps_dir: ""` to `defaultConfig.paths` in `frontend/src/views/SettingsView.vue`
- [ ] T013 [US2] Add "Portable Apps Directory" input with browse button to `frontend/src/components/settings/PathsSection.vue` — reuse the pattern from the `download_dir` field

**Checkpoint**: US2 complete — settings UI shows and persists the portable apps dir

---

## Phase 5: User Story 3 — View and open portable app location (Priority: P2)

**Goal**: Package detail shows install path with clickable "Open folder" action.

**Independent Test**: Install a portable app, navigate to detail, see path, click to open.

- [ ] T014 [US3] Display `install_path` in `frontend/src/components/detail/OverviewTab.vue` when present — show path text with a clickable folder icon that calls `open()` from `@tauri-apps/plugin-shell`
- [ ] T015 [US3] Verify `list_software` in `crates/astro-up-gui/src/commands.rs` includes `install_path` from the ledger in the enriched package JSON — check if already present from detection cache or ledger enrichment

**Checkpoint**: US3 complete — install path visible and openable

---

## Phase 6: User Story 4 — Update replaces in-place (Priority: P3)

**Goal**: Updating a portable app replaces old version in same location.

**Independent Test**: Install, then update — verify old files replaced, ledger updated.

- [ ] T016 [US4] In `handle_download_only` (`crates/astro-up-core/src/install/mod.rs`), when target dir already exists, delete its contents before copying/extracting the new version (clean replace)
- [ ] T017 [US4] Verify the orchestrator passes the existing ledger `install_path` as `install_dir` on the `InstallRequest` when updating a package that already has a recorded path — check `crates/astro-up-core/src/engine/orchestrator.rs`

**Checkpoint**: US4 complete — updates replace in-place

---

## Phase 7: User Story 5 — Portable method alignment (Priority: P3)

**Goal**: `handle_portable_install` uses the portable apps dir (same as download-only).

**Independent Test**: Test with `method = "portable"`, verify it lands in the portable apps folder.

- [ ] T018 [US5] Verify `handle_portable_install` in `crates/astro-up-core/src/install/mod.rs` uses `resolve_install_dir` which reads `install_dir` from request — T006 already sets this for Portable method. Document verification or fix if needed.

**Checkpoint**: US5 complete — portable method aligned

---

## Phase 8: Polish & Cross-Cutting

- [ ] T019 [P] Add integration test for download-only portable install in `crates/astro-up-core/tests/` — create a test zip, run through `handle_download_only`, verify extraction to target dir
- [ ] T020 [P] Update `docs/reference/config.md` with `portable_apps_dir` field
- [ ] T021 [MANUAL] Run quickstart.md validation on .111 — install a download-only package end-to-end, verify portable dir, ledger, UI path display

---

## Task Dependencies

<!-- Machine-readable. Generated by /speckit.tasks, updated by /speckit.iterate.apply -->
<!-- Do not edit manually unless you also update GitHub issue dependencies -->

```toml
[graph]
T001.blocked_by = []
T002.blocked_by = []
T003.blocked_by = ["T001", "T002"]
T004.blocked_by = ["T003"]
T005.blocked_by = []
T006.blocked_by = ["T003", "T005"]
T007.blocked_by = ["T006"]
T008.blocked_by = ["T007"]
T009.blocked_by = ["T007"]
T010.blocked_by = []
T011.blocked_by = []
T012.blocked_by = []
T013.blocked_by = ["T010", "T011", "T012"]
T014.blocked_by = ["T009"]
T015.blocked_by = ["T009"]
T016.blocked_by = ["T005"]
T017.blocked_by = ["T009"]
T018.blocked_by = ["T006"]
T019.blocked_by = ["T005"]
T020.blocked_by = ["T003"]
T021.blocked_by = ["T009", "T013"]
```

## Parallel Opportunities

**Phase 1** (config, all parallel):
```
T001 (model) | T002 (defaults) — different files
```

**Phase 2** (T005 independent of T001/T002):
```
T005 (handler rewrite) can start immediately — no config dependency
T006 waits for both T003 and T005
```

**Phase 4** (frontend, all parallel):
```
T010 (types) | T011 (validation) | T012 (defaults) — different files
```

**Phase 8** (all parallel):
```
T019 (test) | T020 (docs) — independent
```

## Implementation Strategy

### MVP (User Story 1 Only)

1. T001-T004: Config field setup
2. T005-T006: Handler rewrite + wiring
3. T007-T009: Verify US1 end-to-end
4. **STOP and VALIDATE** on .111

### Full Delivery

5. T010-T013: Settings UI (US2)
6. T014-T015: Detail view path display (US3)
7. T016-T017: Update in-place (US4)
8. T018: Portable alignment (US5)
9. T019-T021: Polish

## Notes

- T005 is the largest task — rewriting `handle_download_only` from "open Explorer" to "copy/extract"
- T006 is the critical wiring — injecting portable dir path into the install pipeline
- T018 may be a no-op if `resolve_install_dir` already works correctly with the `install_dir` set by T006
- [MANUAL] T021 requires .111 Windows machine access
- 21 tasks total across 8 phases
