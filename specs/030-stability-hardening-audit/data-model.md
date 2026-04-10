# Data Model: Stability and Hardening Audit

No new entities introduced. This audit modifies behavior of existing entities without schema changes.

## Modified Entities

### Backup Archive (existing)

**Validation additions**:
- ZIP entry paths: normalized, validated against restore root directory
- Entry types: symlinks, reparse points, junctions rejected
- Restore conflict policy: overwrite (full replacement)

### Application State (existing, `gui/src/state.rs`)

**Lock type change**:
- Before: `std::sync::Mutex<T>` (poisoning on panic)
- After: `parking_lot::Mutex<T>` (no poisoning, direct `MutexGuard` return)
- No schema change — same wrapped types, different synchronization primitive

### Task Supervisor (new runtime concept, not persisted)

**Fields**:
- `task_name: String` — identifier for the spawned task
- `restart_count: u32` — panics in current window
- `window_start: Instant` — start of current 10-minute window
- `is_critical: bool` — determines restart vs log-and-die behavior

**Behavior**:
- Window resets after 10 minutes of no panics
- Budget: 3 restarts per window
- On exhaustion: emit Tauri event `task-budget-exhausted` with task name

### Path Allowlist (new runtime concept, not persisted)

**Derived from**:
- `AppConfig.paths.backup_dir` — backup archive directory
- `AppConfig.paths.cache_dir` — application cache
- Per-package `config_paths` from catalog — package configuration directories

**Validation rules**:
- Path must resolve to a subdirectory of one allowlist entry
- Component-based matching via `Path::starts_with()` (not string prefix)
- Symlinks and mount points rejected at validation time
