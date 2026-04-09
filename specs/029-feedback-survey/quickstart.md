# Quickstart: 029 User Feedback Survey

## Prerequisites

- Rust toolchain (see `rust-toolchain.toml`)
- Node.js + pnpm
- `just setup` completed

## Development

```sh
just dev    # Start Tauri dev server with hot-reload
```

## Testing the survey dialog

The dialog appears when:
1. Successful operation count >= threshold (default 3)
2. Survey not previously completed
3. Not within 30-day snooze period

To trigger in dev without performing real installs:

```sql
-- Insert test operation records into the SQLite database
-- Database location: ~/AppData/Local/com.nightwatch.astro-up/astro-up.db (Windows)
-- or the data_dir resolved by directories crate

INSERT INTO operations (package_id, operation_type, from_version, to_version, status, duration_ms, error_message, created_at)
VALUES
  ('nina', 'install', NULL, '3.1.2', 'success', 5000, NULL, '2026-04-01T00:00:00Z'),
  ('phd2', 'install', NULL, '2.6.13', 'success', 3000, NULL, '2026-04-02T00:00:00Z'),
  ('sharpcap', 'update', '4.0', '4.1', 'success', 4000, NULL, '2026-04-03T00:00:00Z');
```

Then navigate to the Dashboard — the survey dialog should appear.

## Resetting survey state

```sql
-- Clear dismissal/completion to re-trigger
DELETE FROM config_settings WHERE key IN ('ui.survey_dismissed_at', 'ui.survey_completed_at');
```

## Verification

```sh
just check  # All quality checks (clippy, fmt, eslint, tests)
just test   # Rust + Vue tests
```
