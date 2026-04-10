# Data Model: 029 User Feedback Survey

**Date**: 2026-04-09

## Entities

### Survey Config Fields (extends `UiConfig`)

New fields added to the existing `UiConfig` struct in `config/model.rs`:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `survey_threshold` | `u32` | `3` | Number of successful operations before showing survey |
| `survey_dismissed_at` | `Option<DateTime<Utc>>` | `None` | Timestamp of last "Not now" / passive dismiss |
| `survey_completed_at` | `Option<DateTime<Utc>>` | `None` | Timestamp of "Leave feedback" or "Don't ask again" |

**Storage**: Stored in existing `config_settings` SQLite table as dot-path keys:
- `ui.survey_threshold` → integer string
- `ui.survey_dismissed_at` → RFC 3339 string or null
- `ui.survey_completed_at` → RFC 3339 string or null

### Survey Eligibility (derived, not persisted)

Computed by the eligibility check command:

| Input | Source |
|-------|--------|
| Successful operation count | `SELECT COUNT(*) FROM operations WHERE status = 'success' AND operation_type IN ('install', 'update')` |
| Survey threshold | `ui.survey_threshold` config field |
| Dismissed at | `ui.survey_dismissed_at` config field |
| Completed at | `ui.survey_completed_at` config field |

**Eligibility logic**:
```
eligible = count >= threshold
         AND completed_at IS NULL
         AND (dismissed_at IS NULL OR now - dismissed_at > 30 days)
```

## State Transitions

```
[Not Eligible] ---(count reaches threshold)---> [Eligible]
[Eligible] ---(dialog shown)---> [Dialog Visible]
[Dialog Visible] ---(Leave feedback)---> [Completed] (terminal)
[Dialog Visible] ---(Don't ask again)---> [Completed] (terminal)
[Dialog Visible] ---(Not now / Escape / click outside)---> [Snoozed]
[Snoozed] ---(30 days pass)---> [Eligible]
```

## Relationships

- **Operations table** (read-only): counted to determine threshold. No writes from this feature.
- **Config settings table** (read-write): stores survey_threshold, survey_dismissed_at, survey_completed_at.
- No new tables or indexes required.
