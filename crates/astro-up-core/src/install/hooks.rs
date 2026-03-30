use std::time::Duration;

use crate::error::CoreError;

const HOOK_TIMEOUT: Duration = Duration::from_secs(60);

/// Executes a pre/post install hook command.
///
/// - `.ps1` files are executed via PowerShell.
/// - All other commands are executed via `cmd /c` on Windows, `sh -c` elsewhere.
/// - Timeout: 60 seconds.
///
/// On Windows with `elevated = true`, hooks inherit the current process elevation.
/// Hooks from manifests are trusted (signed manifests authored by maintainer).
#[cfg(windows)]
pub async fn run_hook(command: &str, _elevated: bool) -> Result<(), CoreError> {
    use tokio::process::Command;

    let (program, args) = if command.ends_with(".ps1") {
        (
            "powershell.exe",
            vec![
                "-ExecutionPolicy",
                "Bypass",
                "-NoProfile",
                "-File",
                command,
            ],
        )
    } else {
        ("cmd", vec!["/c", command])
    };

    let child = Command::new(program)
        .args(&args)
        .kill_on_drop(true)
        .spawn()?;

    let result = tokio::time::timeout(HOOK_TIMEOUT, child.wait_with_output()).await;

    match result {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(())
            } else {
                let code = output.status.code().unwrap_or(-1);
                Err(CoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("hook {command:?} failed with exit code {code}"),
                )))
            }
        }
        Ok(Err(e)) => Err(CoreError::Io(e)),
        Err(_) => Err(CoreError::InstallerTimeout {
            timeout_secs: HOOK_TIMEOUT.as_secs(),
        }),
    }
}

#[cfg(not(windows))]
pub async fn run_hook(command: &str, _elevated: bool) -> Result<(), CoreError> {
    use tokio::process::Command;

    let child = Command::new("sh")
        .args(["-c", command])
        .kill_on_drop(true)
        .spawn()?;

    let result = tokio::time::timeout(HOOK_TIMEOUT, child.wait_with_output()).await;

    match result {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(())
            } else {
                let code = output.status.code().unwrap_or(-1);
                Err(CoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("hook {command:?} failed with exit code {code}"),
                )))
            }
        }
        Ok(Err(e)) => Err(CoreError::Io(e)),
        Err(_) => Err(CoreError::InstallerTimeout {
            timeout_secs: HOOK_TIMEOUT.as_secs(),
        }),
    }
}
