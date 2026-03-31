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

/// Re-executes the current process with elevated privileges.
///
/// Strategy:
/// 1. If `sudo.exe` is on PATH (Windows 11 24H2+), use it for inline elevation.
/// 2. Otherwise, use `ShellExecuteExW` with the `runas` verb (opens new window).
#[cfg(windows)]
pub async fn elevate_and_reexec(args: &[String]) -> Result<(), CoreError> {
    let current_exe = std::env::current_exe()?;

    if detect_sudo() {
        // Sudo path: inline elevation in the same terminal
        let status = tokio::process::Command::new("sudo")
            .arg(&current_exe)
            .args(args)
            .status()
            .await?;

        if status.success() {
            Ok(())
        } else {
            Err(CoreError::ElevationRequired)
        }
    } else {
        use super::wide::to_wide_null;

        let args_str = args.join(" ");
        let exe_wide = to_wide_null(&current_exe.to_string_lossy());
        let args_wide = to_wide_null(&args_str);
        let verb_wide = to_wide_null("runas");

        use windows::Win32::UI::Shell::*;
        use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
        use windows::core::PCWSTR;

        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            lpVerb: PCWSTR(verb_wide.as_ptr()),
            lpFile: PCWSTR(exe_wide.as_ptr()),
            lpParameters: PCWSTR(args_wide.as_ptr()),
            nShow: SW_HIDE.0,
            ..Default::default()
        };

        let success = unsafe { ShellExecuteExW(&mut sei) };
        if success.is_ok() {
            if !sei.hProcess.is_invalid() {
                // Extract raw handle as isize so it's Send-safe across spawn_blocking
                let raw_handle = sei.hProcess.0 as isize;
                tokio::task::spawn_blocking(move || unsafe {
                    use windows::Win32::Foundation::HANDLE;
                    let handle = HANDLE(raw_handle as *mut std::ffi::c_void);
                    windows::Win32::System::Threading::WaitForSingleObject(
                        handle,
                        windows::Win32::System::Threading::INFINITE,
                    );
                    windows::Win32::Foundation::CloseHandle(handle).ok();
                })
                .await
                .map_err(|e| CoreError::Io(std::io::Error::other(e)))?;
            }
            Ok(())
        } else {
            Err(CoreError::ElevationRequired)
        }
    }
}

#[cfg(not(windows))]
pub async fn elevate_and_reexec(_args: &[String]) -> Result<(), CoreError> {
    Err(CoreError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "elevation is only supported on Windows",
    )))
}
