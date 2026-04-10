# Implementation Plan: User Feedback Survey

**Branch**: `029-feedback-survey` | **Date**: 2026-04-09 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/029-feedback-survey/spec.md`

## Summary

Add an in-app feedback survey that prompts users after 3 successful operations (installs/updates). The dialog offers three actions: leave feedback (opens Tally.so form in browser), snooze for 30 days, or permanently opt out. Survey state is persisted in the existing SQLite config store. No new backend, no new tables — minimal addition to the existing config and Tauri command patterns.

## Technical Context

**Language/Version**: Rust 2024 edition + TypeScript 5 / Vue 3
**Primary Dependencies**: rusqlite (existing), tauri-plugin-shell (existing), PrimeVue Dialog (existing), chrono (existing)
**Storage**: SQLite — existing `config_settings` table + read from existing `operations` table
**Testing**: `cargo test` (Rust unit/integration), Vue component tests
**Target Platform**: Windows (primary), macOS/Linux (CI)
**Project Type**: Desktop app (Tauri v2)
**Performance Goals**: Eligibility check < 50ms (single SQL COUNT + 2 config reads)
**Constraints**: No new dependencies, no new tables, no backend
**Scale/Scope**: Single-user desktop app, ~3 new config fields, 1 new Vue component, 3 new Tauri commands

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | Pass | All logic in `astro-up-core` config + engine modules |
| II. Platform Awareness | Pass | No platform-specific code needed |
| III. Test-First | Pass | Unit tests for eligibility logic, integration test for count query |
| IV. Thin Tauri Boundary | Pass | Commands delegate to core functions |
| V. Spec-Driven | Pass | This plan |
| VI. Simplicity | Pass | No new abstractions — extends existing config + adds one dialog |
| VII. Observability | Pass | `info!` on survey shown/dismissed/completed, `debug!` on eligibility check |

## Project Structure

### Source Code (changes only)

```text
crates/astro-up-core/src/
├── config/
│   ├── model.rs              # Add 3 fields to UiConfig
│   └── mod.rs                # Add set_field/get_field_value cases for new fields
└── engine/
    └── history.rs            # Add count_successful_operations()

crates/astro-up-gui/src/
├── commands.rs               # Add check_survey_eligible, dismiss_survey, complete_survey
└── lib.rs                    # Register new commands in invoke_handler

frontend/src/
├── components/
│   └── shared/
│       └── SurveyDialog.vue  # New component
└── views/
    └── DashboardView.vue     # Mount SurveyDialog, check eligibility on load
```

## Implementation Phases

### Phase 1: Core — Config fields + operation count query

**Files**: `config/model.rs`, `config/mod.rs`, `engine/history.rs`

1. Add `survey_threshold: u32`, `survey_dismissed_at: Option<String>`, `survey_completed_at: Option<String>` to `UiConfig` (stored as ISO 8601 strings, parsed to `DateTime<Utc>` in the eligibility function)
2. Add `set_field` / `get_field_value` match arms for `ui.survey_threshold`, `ui.survey_dismissed_at`, `ui.survey_completed_at`
3. Add `count_successful_operations(conn: &Connection) -> Result<u64, CoreError>` in `history.rs` — single SQL COUNT query
4. Add `check_survey_eligible(conn: &Connection, config: &UiConfig) -> bool` — combines count + config state check
5. Tests: unit test for eligibility logic (threshold, snooze, completed), integration test for count query

### Phase 2: GUI — Tauri commands

**Files**: `commands.rs`, `lib.rs`

1. `check_survey_eligible` command — reads config + counts operations, returns `bool`
2. `dismiss_survey` command — sets `ui.survey_dismissed_at` to now
3. `complete_survey` command — sets `ui.survey_completed_at` to now
4. Register all 3 in `invoke_handler`

### Phase 3: Frontend — Survey dialog component

**Files**: `SurveyDialog.vue`, `DashboardView.vue`

1. Create `SurveyDialog.vue` following `ConfirmDialog.vue` pattern:
   - PrimeVue Dialog, modal, 480px width
   - Icon + "How's Astro-Up working for you?" heading
   - Three buttons: "Leave feedback" (primary), "Not now" (text/secondary), "Don't ask again" (text/secondary)
   - On "Leave feedback": invoke `complete_survey`, then `openUrl('https://tally.so/r/lb7dd5')`
   - On "Not now" or dialog close (Escape/click outside): invoke `dismiss_survey`
   - On "Don't ask again": invoke `complete_survey`
2. In `DashboardView.vue`:
   - `onMounted`: invoke `check_survey_eligible`, set reactive `showSurvey` flag
   - Mount `<SurveyDialog>` with `:visible="showSurvey"`

### Phase 4: Tally.so form [MANUAL]

Form already created at `https://tally.so/r/lb7dd5`. Verify 6 fields are present and form submits correctly.
