# Feature Specification: User Feedback Survey

**Feature Branch**: `029-feedback-survey`
**Created**: 2026-04-09
**Status**: Draft
**Type**: implementation
**Input**: User description: "In-app feedback survey triggered after users complete operations, with Tally.so form for response collection"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Prompted for Feedback After Using the App (Priority: P1)

A user who has successfully completed several installs and updates sees a friendly dialog asking how Astro-Up is working for them. They can choose to leave feedback, snooze the prompt, or opt out permanently.

**Why this priority**: Core feature — without the trigger and dialog, nothing else works.

**Independent Test**: Can be tested by ensuring the operation count threshold is met and verifying the dialog appears on the next Dashboard load.

**Acceptance Scenarios**:

1. **Given** a user has completed 3 or more successful operations (installs or updates), **When** they navigate to the Dashboard, **Then** a feedback dialog appears asking "How's Astro-Up working for you?"
2. **Given** a user has completed fewer than 3 successful operations, **When** they navigate to the Dashboard, **Then** no feedback dialog appears.
3. **Given** the feedback dialog is visible, **When** the user clicks "Love it! Leave feedback", **Then** the Tally.so form opens in their default browser and the dialog is permanently dismissed.
4. **Given** the feedback dialog is visible, **When** the user clicks "Not now", **Then** the dialog closes and will not reappear for 30 days.
5. **Given** the feedback dialog is visible, **When** the user clicks "Don't ask again", **Then** the dialog closes and never appears again.

---

### User Story 2 - Completing the Tally.so Feedback Form (Priority: P1)

A user who clicked "Leave feedback" lands on a short Tally.so form. The form takes under 60 seconds to complete and covers satisfaction, recommendations, missing software, improvement suggestions, discovery channel, and optional contact info.

**Why this priority**: Equal to the dialog — the form is the actual feedback collection mechanism. Without a well-designed form, the dialog is pointless.

**Independent Test**: Can be tested by visiting the Tally.so form URL directly and completing it in under 60 seconds.

**Acceptance Scenarios**:

1. **Given** a user opens the feedback form, **When** they view the form, **Then** they see 6 fields: satisfaction rating, recommendation score, missing software, improvement suggestions, how they found Astro-Up, and optional email.
2. **Given** a user is filling out the form, **When** they submit with only the required fields completed, **Then** the form submits successfully (email is optional).
3. **Given** a user completes the form, **When** they submit, **Then** responses are recorded in the Tally.so dashboard for the project owner to review.

---

### User Story 3 - Re-prompted After Snooze Period (Priority: P2)

A user who previously dismissed the survey with "Not now" is prompted again after 30 days, provided they are still actively using the app.

**Why this priority**: Increases response rate over time without being annoying.

**Independent Test**: Can be tested by setting `survey_dismissed_at` to >30 days ago and verifying the dialog reappears.

**Acceptance Scenarios**:

1. **Given** a user dismissed the survey 31 days ago and has continued using the app, **When** they navigate to the Dashboard, **Then** the feedback dialog appears again.
2. **Given** a user dismissed the survey 29 days ago, **When** they navigate to the Dashboard, **Then** no feedback dialog appears.

---

### Edge Cases

- What happens if the user has exactly 3 operations but all are failed/cancelled? Only successful operations (`OperationStatus::Success`) count toward the threshold.
- What happens if the user completes the survey but the browser fails to open? The dialog marks the survey as completed regardless — the intent was expressed.
- What happens if the config store is corrupted or missing survey fields? Fall back to default values (threshold = 3, no dismissal/completion timestamps), which means the dialog may appear.
- What happens if the user is offline? The dialog still appears — it just opens a browser URL. If the browser can't load Tally.so, that's outside the app's control.

## Clarifications

### Session 2026-04-09

- Q: What happens if the user closes the dialog without clicking any button (Escape, click outside)? → A: Treat as "Not now" — snooze for 30 days.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST count successful operations (installs and updates) to determine survey eligibility.
- **FR-002**: System MUST show the feedback dialog when the user has completed at least the configured threshold of successful operations AND has not completed or recently dismissed the survey.
- **FR-003**: The feedback dialog MUST offer three actions: leave feedback (opens external form), snooze for 30 days, and permanently opt out.
- **FR-004**: "Leave feedback" MUST open the Tally.so form URL in the user's default browser.
- **FR-005**: "Not now" MUST record a dismissal timestamp and suppress the dialog for 30 days.
- **FR-006**: "Don't ask again" MUST permanently suppress the dialog.
- **FR-007**: The survey eligibility check MUST run when the Dashboard view loads.
- **FR-008**: The operation threshold MUST be configurable (default: 3).
- **FR-009**: The Tally.so form MUST collect: satisfaction rating, recommendation score (NPS), missing software requests, improvement suggestions, discovery channel, and optional contact email.
- **FR-010**: The Tally.so form MUST be completable without any login or account.
- **FR-011**: Only operations with `Success` status MUST count toward the threshold. Failed, cancelled, and reboot-pending operations MUST NOT count.

### Key Entities

- **Survey State**: Tracks eligibility — operation count, dismissal timestamp, completion timestamp, configured threshold.
- **Feedback Form**: External Tally.so form with 6 fields collecting user sentiment and suggestions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users who meet the threshold see the feedback dialog within 1 second of Dashboard load.
- **SC-002**: The feedback form can be completed in under 60 seconds.
- **SC-003**: Users who click "Not now" are not prompted again for 30 days.
- **SC-004**: Users who click "Don't ask again" or "Leave feedback" are never prompted again.
- **SC-005**: The dialog does not appear for users below the operation threshold.

## Assumptions

- The Tally.so form will be created and hosted on tally.so (free tier is sufficient for expected volume).
- The form URL is `https://tally.so/r/lb7dd5` and will be hardcoded (not user-configurable).
- The 30-day snooze period is fixed, not configurable.
- Only the GUI app shows the survey dialog — the CLI does not.
- The operation history table (`OperationRecord` with `OperationType` and `OperationStatus`) already exists and tracks installs/updates with success/failure status.
- `tauri-plugin-shell` (already integrated) handles opening external URLs in the default browser.
