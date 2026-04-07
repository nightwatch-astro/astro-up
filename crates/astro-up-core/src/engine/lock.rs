//! Global lock file — PID-based single-instance enforcement with stale detection.
//!
//! Uses OS-level advisory file locking (`fd_lock`) combined with PID tracking
//! to ensure only one orchestration engine runs at a time.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::CoreError;

/// Global orchestration lock — ensures single-instance execution.
///
/// Combines `fd_lock::RwLock<File>` for OS-level advisory locking with PID
/// tracking for stale lock detection. The lock is automatically released when
/// the struct is dropped (the write guard's destructor calls `flock(UNLOCK)`).
pub struct OrchestrationLock {
    /// The write guard whose lifetime maintains the OS-level advisory lock.
    /// Dropping this guard releases the flock.
    _guard: fd_lock::RwLockWriteGuard<'static, File>,
    path: PathBuf,
}

// SAFETY: The guard holds an OS-level flock on a file descriptor. File
// descriptors are process-global resources not tied to any particular thread.
// The guard only calls flock(UNLOCK) on drop, which is thread-safe.
unsafe impl Send for OrchestrationLock {}

impl std::fmt::Debug for OrchestrationLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrchestrationLock")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl OrchestrationLock {
    /// Acquire the orchestration lock at the given path.
    ///
    /// If another process holds the lock, its PID is checked via `sysinfo`.
    /// Stale locks (dead PIDs) are reclaimed automatically.
    pub fn acquire(lock_path: &Path) -> Result<Self, CoreError> {
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Probe for contention with a temporary file handle. This handle is
        // dropped immediately — it only checks whether the lock is available.
        Self::probe_lock(lock_path)?;

        // We know the lock is available. Open a fresh handle and acquire the
        // exclusive flock that we will hold for the struct's lifetime.
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;

        // Heap-allocate the RwLock and leak it so the guard can borrow it
        // with a 'static lifetime. The leaked allocation is small (just a
        // File wrapper) and only happens once per orchestration session. The
        // guard's Drop releases the flock; the File fd is closed when the
        // guard drops (it is the only remaining reference).
        let rw_lock = Box::leak(Box::new(fd_lock::RwLock::new(file)));

        let mut guard = rw_lock.try_write().map_err(|err| {
            // Race: another process grabbed the lock between probe and acquire.
            if err.kind() == std::io::ErrorKind::WouldBlock {
                let pid = read_pid_from_file(lock_path).unwrap_or(0);
                CoreError::OrchestrationLocked { pid }
            } else {
                CoreError::Io(err)
            }
        })?;

        // We hold the exclusive OS lock — write our PID.
        guard.set_len(0)?;
        write!(guard, "{}", std::process::id())?;
        guard.sync_all()?;

        Ok(Self {
            _guard: guard,
            path: lock_path.to_owned(),
        })
    }

    /// Probe whether the lock file is available, handling stale locks.
    ///
    /// Opens a temporary file handle, tries the lock, and drops everything.
    /// Returns `Ok(())` if the lock is available, `Err` if contention or I/O.
    fn probe_lock(lock_path: &Path) -> Result<(), CoreError> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;

        let mut rw_lock = fd_lock::RwLock::new(file);

        match rw_lock.try_write() {
            Ok(_guard) => {
                // Lock is available. Guard drops here, releasing the probe lock.
                Ok(())
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                // Another process holds the lock — check if it is still alive.
                let pid = read_pid_from_file(lock_path);
                if let Some(pid) = pid {
                    if process_exists(pid) {
                        return Err(CoreError::OrchestrationLocked { pid });
                    }
                    // Stale lock — process is dead. The OS releases advisory
                    // locks when the holding process exits, so the lock should
                    // now be available.
                    tracing::warn!(pid, "detected stale orchestration lock");
                    Ok(())
                } else {
                    Err(CoreError::OrchestrationLocked { pid: 0 })
                }
            }
            Err(err) => Err(CoreError::Io(err)),
        }
    }

    /// Path to the lock file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Read PID from a lock file, returning `None` on any failure.
fn read_pid_from_file(path: &Path) -> Option<u32> {
    let mut file = File::open(path)
        .inspect_err(|e| {
            tracing::debug!(error = %e, path = %path.display(), "failed to open lock file for PID read");
        })
        .ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .inspect_err(|e| {
            tracing::debug!(error = %e, path = %path.display(), "failed to read lock file contents");
        })
        .ok()?;
    let trimmed = contents.trim();
    match trimmed.parse::<u32>() {
        Ok(pid) => Some(pid),
        Err(_) => {
            tracing::debug!(
                raw_value = trimmed,
                path = %path.display(),
                "failed to parse PID from lock file"
            );
            None
        }
    }
}

/// Check if a process with the given PID is currently running.
fn process_exists(pid: u32) -> bool {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};

    let s = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );
    s.process(sysinfo::Pid::from_u32(pid)).is_some()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn acquire_and_release() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");

        let lock = OrchestrationLock::acquire(&lock_path).unwrap();
        assert!(lock_path.exists());
        assert_eq!(lock.path(), lock_path);

        // On non-Windows, verify PID was written. On Windows, the exclusive
        // fd_lock prevents other handles from reading the file.
        #[cfg(not(windows))]
        {
            let contents = std::fs::read_to_string(&lock_path).unwrap();
            assert_eq!(contents, std::process::id().to_string());
        }
    }

    #[test]
    fn double_acquire_fails() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");

        let _lock = OrchestrationLock::acquire(&lock_path).unwrap();
        let result = OrchestrationLock::acquire(&lock_path);
        assert!(
            result.is_err(),
            "second acquire should fail while first is held"
        );
    }

    #[test]
    fn creates_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("nested").join("dir").join("test.lock");

        let _lock = OrchestrationLock::acquire(&lock_path).unwrap();
        assert!(lock_path.exists());
    }

    #[test]
    fn path_accessor() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("orchestration.lock");

        let lock = OrchestrationLock::acquire(&lock_path).unwrap();
        assert_eq!(lock.path(), lock_path);
    }
}
