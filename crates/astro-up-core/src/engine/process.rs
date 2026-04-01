//! Running-process detection via sysinfo.

use std::path::PathBuf;

use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};

/// Information about a running process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    /// Process name (e.g. `NINA.exe`).
    pub name: String,
    /// Operating-system process identifier.
    pub pid: u32,
    /// Full path to the executable, if available.
    pub exe_path: Option<PathBuf>,
}

/// Check whether a process with the given name is currently running.
///
/// Returns the first match using **case-insensitive** comparison.
/// Only refreshes the process list — no CPU/memory/disk overhead.
pub fn check_process_running(process_name: &str) -> Option<ProcessInfo> {
    let sys = refreshed_system();
    find_processes(&sys, process_name).into_iter().next()
}

/// Return *all* running processes whose name matches (case-insensitive).
pub fn check_processes_running(process_name: &str) -> Vec<ProcessInfo> {
    let sys = refreshed_system();
    find_processes(&sys, process_name)
}

/// Build a [`System`] with only the process list refreshed (minimal overhead).
fn refreshed_system() -> System {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        ),
    );
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys
}

/// Scan all processes for case-insensitive name matches.
fn find_processes(sys: &System, name: &str) -> Vec<ProcessInfo> {
    sys.processes()
        .values()
        .filter(|p| p.name().to_string_lossy().eq_ignore_ascii_case(name))
        .map(|p| ProcessInfo {
            name: p.name().to_string_lossy().into_owned(),
            pid: p.pid().as_u32(),
            exe_path: p.exe().map(PathBuf::from),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_info_fields() {
        let info = ProcessInfo {
            name: "test.exe".to_string(),
            pid: 42,
            exe_path: Some(PathBuf::from("/usr/bin/test")),
        };
        assert_eq!(info.name, "test.exe");
        assert_eq!(info.pid, 42);
        assert_eq!(info.exe_path, Some(PathBuf::from("/usr/bin/test")));
    }

    #[test]
    fn process_info_without_exe_path() {
        let info = ProcessInfo {
            name: "daemon".to_string(),
            pid: 1,
            exe_path: None,
        };
        assert!(info.exe_path.is_none());
    }

    #[test]
    fn nonexistent_process_returns_none() {
        // A process name that will never exist.
        let result = check_process_running("__astro_up_nonexistent_process_42__");
        assert!(result.is_none());
    }

    #[test]
    fn nonexistent_process_returns_empty_vec() {
        let result = check_processes_running("__astro_up_nonexistent_process_42__");
        assert!(result.is_empty());
    }

    #[test]
    fn finds_current_process() {
        // The test runner itself should be discoverable.
        // The binary is named something like `astro_up_core-<hash>` but we
        // cannot predict the exact name.  Instead, look up by our own PID
        // to verify the machinery works end-to-end.
        let own_pid = std::process::id();
        let sys = refreshed_system();
        let proc = sys.process(sysinfo::Pid::from_u32(own_pid));
        assert!(proc.is_some(), "own process should be visible via sysinfo");

        // Now use the name-based API — case-insensitive.
        let own_name = proc.unwrap().name().to_string_lossy().to_string();
        let matches = check_processes_running(&own_name);
        assert!(
            matches.iter().any(|p| p.pid == own_pid),
            "should find own process by name"
        );

        // Verify case-insensitivity by uppercasing the name.
        let upper = own_name.to_ascii_uppercase();
        let matches_upper = check_processes_running(&upper);
        assert!(
            matches_upper.iter().any(|p| p.pid == own_pid),
            "should find own process with uppercased name"
        );
    }
}
