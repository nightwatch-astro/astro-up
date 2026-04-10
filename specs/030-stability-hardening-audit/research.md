# Research: Stability and Hardening Audit

## R1: parking_lot Compatibility with Tauri v2 State

**Decision**: Use `parking_lot::Mutex` as drop-in replacement for `std::sync::Mutex` in all Tauri state.

**Rationale**:
- `parking_lot::Mutex::lock()` returns `MutexGuard` directly (no `Result`) — eliminates 33 `.unwrap()` calls
- No poisoning: lock released normally on panic, preventing cascading failures
- 1 byte vs boxed std Mutex; adaptive spinning for micro-contention
- Tauri's `State<'_, T>` requires `T: Send + Sync + 'static`. `parking_lot::Mutex<T>` satisfies this when `T: Send`.
- `app.manage()` accepts any `T: Send + Sync + 'static` — no Tauri-specific adapter needed

**Alternatives considered**:
- `std::sync::Mutex` with error handling on poisoned locks: More code, same behavior on panic (lock released), but requires handling `PoisonError` at every call site
- `tokio::sync::Mutex`: Async-aware but unnecessary overhead for short critical sections in Tauri commands; also `.lock().await` changes the API surface
- Custom wrapper around `std::sync::Mutex` that logs and clears poison: More boilerplate than parking_lot with no additional benefit

**Source**: [parking_lot docs](https://docs.rs/parking_lot/latest/parking_lot/type.Mutex.html), [Tauri v2 state management](https://v2.tauri.app/develop/state-management)

## R2: ZIP Path Traversal Mitigation

**Decision**: Validate every ZIP entry path before extraction using path canonicalization and component checking.

**Rationale**:
- The `zip` crate (used by astro-up) does NOT automatically sanitize entry names
- Entries can contain `..`, absolute paths, symlinks, and Windows-specific reparse points
- Standard mitigation: strip leading `/` and `\`, reject entries containing `..` after normalization, verify resolved path starts with intended root using `Path::starts_with()` (component-based, not string-based)

**Alternatives considered**:
- `zip::read::ZipFile::enclosed_name()`: Returns `None` for unsafe paths but only handles `..` and absolute paths, not symlinks/reparse points. Useful as a first-pass filter but insufficient alone.
- Custom path sanitizer: Required for Windows reparse point detection (check external attributes in ZIP entry for `FILE_ATTRIBUTE_REPARSE_POINT`)

## R3: Panic Boundaries in Async Tasks

**Decision**: Use `std::panic::catch_unwind` with `AssertUnwindSafe` wrapper for spawned async tasks.

**Rationale**:
- `tokio::spawn` propagates panics to the JoinHandle — if not awaited, panic is silently lost
- `catch_unwind` catches panic at unwind boundary, allowing logging and optional restart
- `AssertUnwindSafe` is needed because most futures aren't `UnwindSafe` by default
- Critical task restart: simple counter + timestamp window (not a full supervisor framework per Constitution VI: Simplicity)

**Alternatives considered**:
- `tokio::task::Builder::spawn` with panic hook: More complex, global panic hook affects all tasks
- Custom supervisor crate (e.g., `bastion`): Overkill for 2-3 critical tasks; adds significant dependency

## R4: Mounted-Flag Pattern for Tauri Invoke

**Decision**: Use a `ref(true)` flag set to `false` on `onUnmounted`, checked before applying invoke results.

**Rationale**:
- Tauri's IPC (`invoke()`) does not support `AbortController` or cancellation
- The Promise returned by `invoke()` will always resolve/reject regardless of component lifecycle
- Checking `isMounted.value` before state updates prevents "state update on unmounted component" issues
- VueQuery handles this internally for queries/mutations via its own lifecycle management

**Alternatives considered**:
- AbortController: Not supported by Tauri IPC
- VueQuery for all calls: Would require converting one-off invokes (survey submit, asset selection confirm) into mutations — over-engineering for fire-and-forget calls
- `@vueuse/core` `useIsMounted`: External utility; trivial to implement inline (3 lines)

## R5: Windows Reparse Point Detection in ZIP

**Decision**: Check ZIP entry external attributes for `FILE_ATTRIBUTE_REPARSE_POINT` (0x400) on Windows.

**Rationale**:
- ZIP external attributes (when created on Windows) contain NTFS file attributes in the high 16 bits
- `FILE_ATTRIBUTE_REPARSE_POINT` indicates symlinks, junctions, and other reparse points
- The `zip` crate exposes `ZipFile::unix_mode()` but not Windows attributes directly — need to read `external_attributes` from the central directory entry
- On non-Windows hosts: check Unix mode for symlink bit (0xA000 in upper 16 bits)

**Alternatives considered**:
- Only check Unix symlink mode: Misses Windows-specific reparse points
- Post-extraction check with `fs::symlink_metadata()`: Too late — file already written to disk
