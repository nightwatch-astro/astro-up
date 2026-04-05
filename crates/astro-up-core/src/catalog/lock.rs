//! PID lockfile — acquire, release, stale detection.

use std::path::{Path, PathBuf};

use crate::error::CoreError;

/// An apt-style PID lockfile. Acquired on creation, released on `Drop`.
#[derive(Debug)]
pub struct PidLock {
    path: PathBuf,
}

impl PidLock {
    /// Acquire the lockfile. Checks for stale locks (dead PIDs).
    pub fn acquire(path: &Path) -> Result<Self, CoreError> {
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            if let Ok(pid) = contents.trim().parse::<u32>() {
                if process_exists(pid) {
                    return Err(CoreError::CatalogLocked { pid });
                }
                tracing::warn!(pid, "removing stale lockfile");
            } else {
                tracing::warn!("removing corrupt lockfile");
            }
            std::fs::remove_file(path)?;
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, std::process::id().to_string())?;
        Ok(Self {
            path: path.to_owned(),
        })
    }
}

impl Drop for PidLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
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
        let lock_path = dir.path().join("test.lock");

        {
            let _lock = PidLock::acquire(&lock_path).unwrap();
            assert!(lock_path.exists());
            let contents = std::fs::read_to_string(&lock_path).unwrap();
            assert_eq!(contents, std::process::id().to_string());
        }
        // Dropped — file should be gone
        assert!(!lock_path.exists());
    }

    #[test]
    fn double_acquire_fails() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");

        let _lock = PidLock::acquire(&lock_path).unwrap();
        let result = PidLock::acquire(&lock_path);
        assert!(result.is_err());
    }

    #[test]
    fn stale_lock_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");

        // Write a PID that doesn't exist (99999999 is unlikely to be running)
        std::fs::write(&lock_path, "99999999").unwrap();

        // Should succeed — stale lock detected and removed
        let lock = PidLock::acquire(&lock_path);
        assert!(lock.is_ok());
    }
}
