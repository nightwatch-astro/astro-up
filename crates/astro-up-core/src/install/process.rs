use std::time::Duration;
#[cfg(windows)]
use std::time::Instant;

use tokio_util::sync::CancellationToken;
use tracing::error;
#[cfg(windows)]
use tracing::{debug, info, warn};

use crate::error::CoreError;

/// Spawns an installer process and waits for it to complete.
///
/// Uses `tokio::process::Command` with timeout and cancellation support.
/// On drop, the child process is killed (`kill_on_drop(true)`).
///
/// Returns the process exit code.
#[cfg(windows)]
#[tracing::instrument(skip_all, fields(exe = %exe, timeout_secs = timeout.as_secs()))]
pub async fn spawn_simple(
    exe: &str,
    args: &[String],
    timeout: Duration,
    cancel_token: CancellationToken,
) -> Result<i32, CoreError> {
    use tokio::process::Command;

    info!(args = ?args, "spawning installer process");
    let start = Instant::now();

    let mut child = match Command::new(exe).args(args).kill_on_drop(true).spawn() {
        Ok(child) => child,
        Err(e) => {
            error!(error = %e, "failed to spawn installer process");
            return Err(CoreError::Io(e));
        }
    };

    let result = tokio::select! {
        result = child.wait() => {
            let status = result?;
            let code = status.code().unwrap_or(-1);
            info!(exit_code = code, duration_ms = start.elapsed().as_millis() as u64, "installer process exited");
            Ok(code)
        }
        () = cancel_token.cancelled() => {
            if let Err(e) = child.kill().await {
                tracing::trace!(error = %e, "failed to kill child process during cancellation");
            }
            warn!(duration_ms = start.elapsed().as_millis() as u64, "installer process cancelled");
            Err(CoreError::Cancelled)
        }
        () = tokio::time::sleep(timeout) => {
            if let Err(e) = child.kill().await {
                tracing::trace!(error = %e, "failed to kill child process during timeout");
            }
            warn!(timeout_secs = timeout.as_secs(), duration_ms = start.elapsed().as_millis() as u64, "installer process timed out");
            Err(CoreError::InstallerTimeout { timeout_secs: timeout.as_secs() })
        }
    };

    result
}

#[cfg(not(windows))]
#[tracing::instrument(skip_all, fields(exe = %_exe, timeout_secs = _timeout.as_secs()))]
pub async fn spawn_simple(
    _exe: &str,
    _args: &[String],
    _timeout: Duration,
    _cancel_token: CancellationToken,
) -> Result<i32, CoreError> {
    error!("installer execution is only supported on Windows");
    Err(CoreError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "installer execution is only supported on Windows",
    )))
}

/// Spawns an installer with a Windows Job Object for process tree management.
///
/// Used for bootstrapper-style installers (e.g., Burn) that spawn child processes.
/// The Job Object ensures the entire process tree is killed on timeout/cancel.
///
/// Flow:
/// 1. Create Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`
/// 2. Create process suspended via `CreateProcessW`
/// 3. Assign process to job
/// 4. Resume thread
/// 5. Wait for process with timeout/cancellation
#[cfg(windows)]
#[tracing::instrument(skip_all, fields(exe = %exe, timeout_secs = timeout.as_secs()))]
pub async fn spawn_with_job_object(
    exe: &str,
    args: &[String],
    timeout: Duration,
    cancel_token: CancellationToken,
) -> Result<i32, CoreError> {
    use std::mem;

    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    };
    use windows::Win32::System::Threading::{
        CREATE_SUSPENDED, CreateProcessW, GetExitCodeProcess, PROCESS_INFORMATION, ResumeThread,
        STARTUPINFOW, WaitForSingleObject,
    };
    use windows::core::PWSTR;

    use super::wide::to_wide_null;

    info!(args = ?args, "spawning installer process with job object");
    let start = Instant::now();

    let cmd_line = if args.is_empty() {
        format!("\"{exe}\"")
    } else {
        format!("\"{exe}\" {}", args.join(" "))
    };
    let mut cmd_wide = to_wide_null(&cmd_line);

    // Create job object
    debug!("creating Windows Job Object");
    let job = unsafe { CreateJobObjectW(None, None) }
        .map_err(|e| CoreError::Io(std::io::Error::other(e)))?;

    // Configure: kill all processes when job handle closes
    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            (&raw const info).cast(),
            mem::size_of_val(&info) as u32,
        )
    }
    .map_err(|e| CoreError::Io(std::io::Error::other(e)))?;
    debug!("Job Object configured with KILL_ON_JOB_CLOSE");

    // Create process suspended
    let si = STARTUPINFOW {
        cb: mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();

    if let Err(e) = unsafe {
        CreateProcessW(
            None,
            Some(PWSTR(cmd_wide.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            None,
            &raw const si,
            &raw mut pi,
        )
    } {
        error!(error = %e, "failed to create suspended process");
        return Err(CoreError::Io(std::io::Error::other(e)));
    }

    let process_handle = pi.hProcess;
    let thread_handle = pi.hThread;

    // Assign to job, then resume
    debug!("assigning process to Job Object");
    let assign_result = unsafe { AssignProcessToJobObject(job, process_handle) };
    if assign_result.is_err() {
        error!("failed to assign process to Job Object");
        // Cleanup on failure
        unsafe {
            if let Err(e) = windows::Win32::System::Threading::TerminateProcess(process_handle, 1) {
                tracing::trace!(error = %e, "failed to terminate process during job assignment cleanup");
            }
            if let Err(e) = CloseHandle(thread_handle) {
                tracing::trace!(error = %e, "failed to close thread handle during job assignment cleanup");
            }
            if let Err(e) = CloseHandle(process_handle) {
                tracing::trace!(error = %e, "failed to close process handle during job assignment cleanup");
            }
            if let Err(e) = CloseHandle(job) {
                tracing::trace!(error = %e, "failed to close job handle during job assignment cleanup");
            }
        }
        return Err(CoreError::Io(std::io::Error::other(
            "failed to assign process to job object",
        )));
    }

    unsafe {
        ResumeThread(thread_handle);
        if let Err(e) = CloseHandle(thread_handle) {
            tracing::trace!(error = %e, "failed to close thread handle after resume");
        }
    }
    debug!("process resumed within Job Object");

    // Extract raw handles as isize for Send safety across spawn_blocking
    let raw_process = process_handle.0 as isize;
    let raw_job = job.0 as isize;
    let timeout_ms = timeout.as_millis() as u32;
    let timeout_secs = timeout.as_secs();

    let result = tokio::select! {
        join_result = tokio::task::spawn_blocking(move || {
            use windows::Win32::Foundation::HANDLE;

            let proc_h = HANDLE(raw_process as *mut std::ffi::c_void);
            let job_h = HANDLE(raw_job as *mut std::ffi::c_void);

            let wait = unsafe { WaitForSingleObject(proc_h, timeout_ms) };
            let code = if wait.0 == 0 {
                let mut exit_code: u32 = 0;
                unsafe {
                    if let Err(e) = GetExitCodeProcess(proc_h, &raw mut exit_code) {
                        tracing::trace!(error = %e, "failed to get exit code from process");
                    }
                }
                Ok(exit_code as i32)
            } else {
                Err(timeout_secs)
            };
            unsafe {
                if let Err(e) = CloseHandle(proc_h) {
                    tracing::trace!(error = %e, "failed to close process handle after wait");
                }
                if let Err(e) = CloseHandle(job_h) {
                    tracing::trace!(error = %e, "failed to close job handle after wait");
                }
            }
            code
        }) => {
            match join_result {
                Ok(Ok(code)) => {
                    info!(exit_code = code, duration_ms = start.elapsed().as_millis() as u64, "job object process exited");
                    Ok(code)
                }
                Ok(Err(secs)) => {
                    warn!(timeout_secs = secs, duration_ms = start.elapsed().as_millis() as u64, "job object process timed out");
                    Err(CoreError::InstallerTimeout { timeout_secs: secs })
                }
                Err(e) => {
                    error!(error = %e, "job object spawn_blocking task failed");
                    Err(CoreError::Io(std::io::Error::other(e)))
                }
            }
        }
        () = cancel_token.cancelled() => {
            warn!(duration_ms = start.elapsed().as_millis() as u64, "job object process cancelled");
            Err(CoreError::Cancelled)
        }
    };

    result
}

#[cfg(not(windows))]
#[tracing::instrument(skip_all, fields(exe = %_exe, timeout_secs = _timeout.as_secs()))]
pub async fn spawn_with_job_object(
    _exe: &str,
    _args: &[String],
    _timeout: Duration,
    _cancel_token: CancellationToken,
) -> Result<i32, CoreError> {
    error!("job object execution is only supported on Windows");
    Err(CoreError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "job object execution is only supported on Windows",
    )))
}
