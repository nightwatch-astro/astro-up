use std::time::Duration;

use crate::error::CoreError;

/// Checks if the current process is running with admin privileges.
#[cfg(windows)]
pub fn is_elevated() -> bool {
    unsafe { windows::Win32::UI::Shell::IsUserAnAdmin().as_bool() }
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}

/// Checks if `sudo.exe` is available on PATH (Windows 11 24H2+).
#[cfg(windows)]
pub fn detect_sudo() -> bool {
    which_sudo().is_some()
}

#[cfg(not(windows))]
pub fn detect_sudo() -> bool {
    false
}

#[cfg(windows)]
fn which_sudo() -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join("sudo.exe");
            candidate.is_file().then_some(candidate)
        })
    })
}

/// Spawns an installer process with elevated (admin) privileges.
///
/// Elevates only the installer process — the calling application continues
/// running unprivileged. This avoids re-executing the entire app, which would
/// restart the GUI in Tauri.
///
/// Strategy:
/// 1. If `sudo.exe` is on PATH (Windows 11 24H2+), prefix the command with `sudo`.
/// 2. Otherwise, use `ShellExecuteExW` with the `runas` verb to trigger a UAC prompt.
///
/// Returns the installer process exit code.
#[cfg(windows)]
#[tracing::instrument(skip_all, fields(exe = %exe, timeout_secs = timeout.as_secs()))]
pub async fn spawn_elevated(
    exe: &str,
    args: &[String],
    timeout: Duration,
) -> Result<i32, CoreError> {
    tracing::info!(args = ?args, "spawning installer with elevation");

    if detect_sudo() {
        spawn_elevated_sudo(exe, args, timeout).await
    } else {
        spawn_elevated_runas(exe, args, timeout).await
    }
}

/// Elevation via `sudo.exe` (Windows 11 24H2+). Inline elevation — no new window.
#[cfg(windows)]
async fn spawn_elevated_sudo(
    exe: &str,
    args: &[String],
    timeout: Duration,
) -> Result<i32, CoreError> {
    use std::time::Instant;

    tracing::info!("using sudo.exe for inline elevation");
    let start = Instant::now();

    let mut child = tokio::process::Command::new("sudo")
        .arg(exe)
        .args(args)
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            tracing::error!(error = %e, "failed to spawn sudo process");
            CoreError::Io(e)
        })?;

    tokio::select! {
        result = child.wait() => {
            let status = result?;
            let code = status.code().unwrap_or(-1);
            tracing::info!(
                exit_code = code,
                duration_ms = start.elapsed().as_millis() as u64,
                "elevated installer process exited"
            );
            Ok(code)
        }
        () = tokio::time::sleep(timeout) => {
            if let Err(e) = child.kill().await {
                tracing::trace!(error = %e, "failed to kill elevated process during timeout");
            }
            tracing::warn!(
                timeout_secs = timeout.as_secs(),
                duration_ms = start.elapsed().as_millis() as u64,
                "elevated installer process timed out"
            );
            Err(CoreError::InstallerTimeout { timeout_secs: timeout.as_secs() })
        }
    }
}

/// Elevation via `ShellExecuteExW` with `runas` verb (pre-Win11 24H2).
/// Shows a UAC prompt and waits for the elevated process to complete.
#[cfg(windows)]
async fn spawn_elevated_runas(
    exe: &str,
    args: &[String],
    timeout: Duration,
) -> Result<i32, CoreError> {
    tracing::info!("using ShellExecuteExW runas for UAC elevation");

    let exe_owned = exe.to_owned();
    let args_str = args.join(" ");
    let timeout_ms = timeout.as_millis() as u32;

    tokio::task::spawn_blocking(move || {
        use super::wide::to_wide_null;
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};
        use windows::Win32::UI::Shell::{
            SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
        };
        use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
        use windows::core::PCWSTR;

        let exe_wide = to_wide_null(&exe_owned);
        let args_wide = to_wide_null(&args_str);
        let verb_wide = to_wide_null("runas");

        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            lpVerb: PCWSTR(verb_wide.as_ptr()),
            lpFile: PCWSTR(exe_wide.as_ptr()),
            lpParameters: PCWSTR(args_wide.as_ptr()),
            nShow: SW_HIDE.0,
            ..Default::default()
        };

        let result = unsafe { ShellExecuteExW(std::ptr::from_mut(&mut sei)) };
        if result.is_err() {
            tracing::warn!("ShellExecuteExW runas failed — user may have declined UAC");
            return Err(CoreError::ElevationRequired);
        }

        if sei.hProcess.is_invalid() {
            tracing::warn!("ShellExecuteExW returned no process handle");
            return Err(CoreError::ElevationRequired);
        }

        let wait = unsafe { WaitForSingleObject(sei.hProcess, timeout_ms) };
        let code = if wait.0 == 0 {
            // WAIT_OBJECT_0 — process exited
            let mut exit_code: u32 = 0;
            unsafe {
                if let Err(e) = GetExitCodeProcess(sei.hProcess, &raw mut exit_code) {
                    tracing::trace!(error = %e, "failed to get exit code from elevated process");
                }
            }
            tracing::info!(
                exit_code = exit_code as i32,
                "elevated installer process exited"
            );
            Ok(exit_code as i32)
        } else {
            // WAIT_TIMEOUT or error
            tracing::warn!(
                timeout_secs = timeout.as_secs(),
                "elevated installer process timed out"
            );
            Err(CoreError::InstallerTimeout {
                timeout_secs: timeout.as_secs(),
            })
        };

        unsafe {
            CloseHandle(sei.hProcess).ok();
        }

        code
    })
    .await
    .map_err(|e| CoreError::Io(std::io::Error::other(e)))?
}

#[cfg(not(windows))]
#[tracing::instrument(skip_all, fields(exe = %_exe, timeout_secs = _timeout.as_secs()))]
pub async fn spawn_elevated(
    _exe: &str,
    _args: &[String],
    _timeout: Duration,
) -> Result<i32, CoreError> {
    tracing::info!("elevation requested but not supported on this platform");
    Err(CoreError::ElevationRequired)
}
