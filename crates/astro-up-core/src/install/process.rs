use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::error::CoreError;

/// Spawns an installer process and waits for it to complete.
///
/// Uses `tokio::process::Command` with timeout and cancellation support.
/// On drop, the child process is killed (`kill_on_drop(true)`).
///
/// Returns the process exit code.
#[cfg(windows)]
pub async fn spawn_simple(
    exe: &str,
    args: &[String],
    timeout: Duration,
    cancel_token: CancellationToken,
) -> Result<i32, CoreError> {
    use tokio::process::Command;

    let mut child = Command::new(exe).args(args).kill_on_drop(true).spawn()?;

    tokio::select! {
        result = child.wait() => {
            let status = result?;
            Ok(status.code().unwrap_or(-1))
        }
        () = cancel_token.cancelled() => {
            child.kill().await.ok();
            Err(CoreError::Cancelled)
        }
        () = tokio::time::sleep(timeout) => {
            child.kill().await.ok();
            Err(CoreError::InstallerTimeout { timeout_secs: timeout.as_secs() })
        }
    }
}

#[cfg(not(windows))]
pub async fn spawn_simple(
    _exe: &str,
    _args: &[String],
    _timeout: Duration,
    _cancel_token: CancellationToken,
) -> Result<i32, CoreError> {
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
pub async fn spawn_with_job_object(
    exe: &str,
    args: &[String],
    timeout: Duration,
    cancel_token: CancellationToken,
) -> Result<i32, CoreError> {
    use std::ffi::OsStr;
    use std::mem;
    use std::os::windows::ffi::OsStrExt;

    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::JobObjects::*;
    use windows::Win32::System::Threading::*;
    use windows::core::PWSTR;

    // Build command line: "exe" arg1 arg2 ...
    let cmd_line = if args.is_empty() {
        format!("\"{exe}\"")
    } else {
        format!("\"{exe}\" {}", args.join(" "))
    };
    let mut cmd_wide: Vec<u16> = OsStr::new(&cmd_line)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Create job object
    let job = unsafe { CreateJobObjectW(None, None) }
        .map_err(|e| CoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // Configure: kill all processes when job handle closes
    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            mem::size_of_val(&info) as u32,
        )
    }
    .map_err(|e| CoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // Create process suspended
    let mut si = STARTUPINFOW::default();
    si.cb = mem::size_of::<STARTUPINFOW>() as u32;
    let mut pi = PROCESS_INFORMATION::default();

    unsafe {
        CreateProcessW(
            None,
            Some(PWSTR(cmd_wide.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            None,
            &si,
            &mut pi,
        )
    }
    .map_err(|e| CoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let process_handle = pi.hProcess;
    let thread_handle = pi.hThread;

    // Assign to job, then resume
    let assign_result = unsafe { AssignProcessToJobObject(job, process_handle) };
    if assign_result.is_err() {
        // Cleanup on failure
        unsafe {
            windows::Win32::System::Threading::TerminateProcess(process_handle, 1).ok();
            CloseHandle(thread_handle).ok();
            CloseHandle(process_handle).ok();
            CloseHandle(job).ok();
        }
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "failed to assign process to job object",
        )));
    }

    unsafe {
        ResumeThread(thread_handle);
        CloseHandle(thread_handle).ok();
    }

    // Extract raw handles as isize for Send safety across spawn_blocking
    let raw_process = process_handle.0 as isize;
    let raw_job = job.0 as isize;
    let timeout_ms = timeout.as_millis() as u32;
    let timeout_secs = timeout.as_secs();

    let result = tokio::select! {
        exit_code = async {
            tokio::task::spawn_blocking(move || {
                use windows::Win32::Foundation::HANDLE;

                let proc_h = HANDLE(raw_process as *mut std::ffi::c_void);
                let job_h = HANDLE(raw_job as *mut std::ffi::c_void);

                let wait = unsafe { WaitForSingleObject(proc_h, timeout_ms) };
                let code: Result<i32, u64> = if wait.0 == 0 {
                    // WAIT_OBJECT_0 = 0
                    let mut exit_code: u32 = 0;
                    unsafe {
                        GetExitCodeProcess(proc_h, &mut exit_code).ok();
                    }
                    Ok(exit_code as i32)
                } else {
                    Err(timeout_secs)
                };
                unsafe {
                    CloseHandle(proc_h).ok();
                    CloseHandle(job_h).ok();
                }
                code
            }).await.map_err(|e| CoreError::Io(std::io::Error::other(e)))?
        } => {
            match exit_code {
                Ok(code) => Ok(code),
                Err(secs) => Err(CoreError::InstallerTimeout { timeout_secs: secs }),
            }
        }
        () = cancel_token.cancelled() => {
            Err(CoreError::Cancelled)
        }
    };

    result
}

#[cfg(not(windows))]
pub async fn spawn_with_job_object(
    _exe: &str,
    _args: &[String],
    _timeout: Duration,
    _cancel_token: CancellationToken,
) -> Result<i32, CoreError> {
    Err(CoreError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "job object execution is only supported on Windows",
    )))
}
