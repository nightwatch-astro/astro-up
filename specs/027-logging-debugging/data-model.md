# Data Model: Logging and Debugging

**Date**: 2026-04-07

## Summary

No new data entities. This spec adds observability to existing operations without changing data structures.

## Structured Log Fields

While not a data model in the traditional sense, the spec defines required structured fields for log entries:

| Field | Type | Required For | Example |
|-------|------|-------------|---------|
| `operation_id` | String (UUID) | All top-level operations | `"3f2504e0-4f89-11d3-9a0c-0305e82c3301"` |
| `package` | String | Package operations | `"PHD2"` |
| `duration_ms` | u64 | Operation boundaries | `1523` |
| `url` | String | Network calls | `"https://github.com/..."` |
| `path` | String (PathBuf display) | File I/O | `"C:\\Users\\...\\catalog.db"` |
| `exit_code` | i32 | Process spawning | `0` |
| `bytes` | u64 | Downloads | `15728640` |
| `count` | usize | Collection operations | `42` |

## Frontend Logger Interface

```typescript
// frontend/src/utils/logger.ts
interface Logger {
  debug(context: string, message: string): void
  info(context: string, message: string): void
  warn(context: string, message: string): void
  error(context: string, message: string): void
}
```

Output target: LogPanel store (`useLogPanel().addLog()`)
