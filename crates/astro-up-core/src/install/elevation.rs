use std::time::Duration;

use crate::error::CoreError;

/// Checks if the current process has an elevated (high-integrity) token.
///
/// Uses the process token's integrity level rather than `IsUserAnAdmin()`,
/// which can return `true` for admin-group users even when running with
/// the limited (non-elevated) UAC token.
#[cfg(windows)]
pub fn is_elevated() -> bool {
    use windows::Win32::Security::TOKEN_QUERY;
    use windows::Win32::Security::{
        GetTokenInformation, TOKEN_MANDATORY_LABEL, TokenIntegrityLevel,
    };
    // SECURITY_MANDATORY_HIGH_RID = 0x2000 (not always exported by windows crate features)
    const HIGH_INTEGRITY_RID: u32 = 0x2000;
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = windows::Win32::Foundation::HANDLE::default();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY,
            std::ptr::from_mut(&mut token),
        )
        .is_err()
        {
            return windows::Win32::UI::Shell::IsUserAnAdmin().as_bool();
        }

        // Query token integrity level — first call gets required buffer size
        let mut size: u32 = 0;
        let _ = GetTokenInformation(
            token,
            TokenIntegrityLevel,
            None,
            0,
            std::ptr::from_mut(&mut size),
        );
        if size == 0 {
            windows::Win32::Foundation::CloseHandle(token).ok();
            return windows::Win32::UI::Shell::IsUserAnAdmin().as_bool();
        }

        let mut buffer = vec![0u8; size as usize];
        let ok = GetTokenInformation(
            token,
            TokenIntegrityLevel,
            Some(buffer.as_mut_ptr().cast()),
            size,
            std::ptr::from_mut(&mut size),
        );
        windows::Win32::Foundation::CloseHandle(token).ok();

        if ok.is_err() {
            return windows::Win32::UI::Shell::IsUserAnAdmin().as_bool();
        }

        let label: &TOKEN_MANDATORY_LABEL = &*(buffer.as_ptr().cast());
        let sid = label.Label.Sid;
        let sub_authority_count = *windows::Win32::Security::GetSidSubAuthorityCount(sid);
        if sub_authority_count == 0 {
            return false;
        }
        let rid =
            *windows::Win32::Security::GetSidSubAuthority(sid, u32::from(sub_authority_count - 1));
        rid >= HIGH_INTEGRITY_RID
    }
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}

/// Spawns an installer process with elevated (admin) privileges.
///
/// Elevates only the installer process — the calling application continues
/// running unprivileged. This avoids re-executing the entire app, which would
/// restart the GUI in Tauri.
///
/// Strategy:
/// Uses `ShellExecuteExW` with the `runas` verb to trigger a UAC prompt.
///
/// Note: `sudo.exe` (Windows 11 24H2+) was previously preferred but its inline
/// mode does not grant full admin rights to GUI installers with embedded admin
/// manifests — both .NET and ZWO installers returned E_ACCESSDENIED via sudo.
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
    spawn_elevated_runas(exe, args, timeout).await
}

/// Elevation via `ShellExecuteExW` with `runas` verb.
/// Shows a UAC prompt and waits for the elevated process to complete.
///
/// Uses `SW_SHOWNORMAL` instead of `SW_HIDE` because some installers
/// (notably WiX Burn bootstrappers) rely on window messaging internally
/// and fail when started hidden.
#[cfg(windows)]
async fn spawn_elevated_runas(
    exe: &str,
    args: &[String],
    timeout: Duration,
) -> Result<i32, CoreError> {
    spawn_elevated_runas_inner(exe, args, timeout, false).await
}

/// Inner implementation shared between simple elevation and job-object elevation.
/// When `with_job` is true, wraps the elevated process in a Windows Job Object
/// for process tree tracking (needed for Burn bootstrappers).
#[cfg(windows)]
async fn spawn_elevated_runas_inner(
    exe: &str,
    args: &[String],
    timeout: Duration,
    with_job: bool,
) -> Result<i32, CoreError> {
    tracing::info!(with_job, "using ShellExecuteExW runas for UAC elevation");

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
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
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
            nShow: SW_SHOWNORMAL.0,
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

        // Optionally wrap in a Job Object for process tree tracking.
        // The process is already running, but child processes created after
        // assignment will be contained. This is critical for Burn bootstrappers
        // that spawn MSI child processes.
        let job_handle = if with_job {
            match create_and_assign_job(sei.hProcess) {
                Ok(job) => Some(job),
                Err(e) => {
                    tracing::warn!(error = %e, "failed to create job object for elevated process, continuing without");
                    None
                }
            }
        } else {
            None
        };

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
            if let Some(job) = job_handle {
                CloseHandle(job).ok();
            }
        }

        code
    })
    .await
    .map_err(|e| CoreError::Io(std::io::Error::other(e)))?
}

/// Create a Job Object and assign the given process to it.
/// Returns the job handle on success.
#[cfg(windows)]
fn create_and_assign_job(
    process: windows::Win32::Foundation::HANDLE,
) -> Result<windows::Win32::Foundation::HANDLE, CoreError> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    };

    let job = unsafe { CreateJobObjectW(None, None) }
        .map_err(|e| CoreError::Io(std::io::Error::other(e)))?;

    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            (&raw const info).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    }
    .map_err(|e| {
        unsafe {
            CloseHandle(job).ok();
        }
        CoreError::Io(std::io::Error::other(e))
    })?;

    unsafe { AssignProcessToJobObject(job, process) }.map_err(|e| {
        unsafe {
            CloseHandle(job).ok();
        }
        CoreError::Io(std::io::Error::other(e))
    })?;

    tracing::debug!("elevated process assigned to Job Object");
    Ok(job)
}

/// Spawns an elevated process with Job Object tracking.
///
/// Combines UAC elevation with process tree management — needed for
/// Burn bootstrappers and other installers that spawn child processes.
#[cfg(windows)]
#[tracing::instrument(skip_all, fields(exe = %exe, timeout_secs = timeout.as_secs()))]
pub async fn spawn_elevated_with_job(
    exe: &str,
    args: &[String],
    timeout: Duration,
) -> Result<i32, CoreError> {
    tracing::info!(args = ?args, "spawning elevated installer with job object");
    spawn_elevated_runas_inner(exe, args, timeout, true).await
}

#[cfg(not(windows))]
#[tracing::instrument(skip_all, fields(exe = %_exe, timeout_secs = _timeout.as_secs()))]
pub async fn spawn_elevated_with_job(
    _exe: &str,
    _args: &[String],
    _timeout: Duration,
) -> Result<i32, CoreError> {
    tracing::info!("elevated job object execution not supported on this platform");
    Err(CoreError::ElevationRequired)
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
