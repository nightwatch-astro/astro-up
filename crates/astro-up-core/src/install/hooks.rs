use std::time::Duration;

use tokio::process::Child;

use crate::error::CoreError;

const HOOK_TIMEOUT: Duration = Duration::from_secs(60);

/// Executes a pre/post install hook command.
///
/// - `.ps1` files are executed via PowerShell (Windows only).
/// - All other commands are executed via `cmd /c` on Windows, `sh -c` elsewhere.
/// - Timeout: 60 seconds.
///
/// Hooks from manifests are trusted (signed manifests authored by maintainer).
#[cfg(windows)]
pub async fn run_hook(command: &str) -> Result<(), CoreError> {
    use tokio::process::Command;

    let child = if std::path::Path::new(command)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ps1"))
    {
        Command::new("powershell.exe")
            .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-File", command])
            .kill_on_drop(true)
            .spawn()?
    } else {
        Command::new("cmd")
            .args(["/c", command])
            .kill_on_drop(true)
            .spawn()?
    };

    await_with_timeout(child, command).await
}

#[cfg(not(windows))]
pub async fn run_hook(command: &str) -> Result<(), CoreError> {
    use tokio::process::Command;

    let child = Command::new("sh")
        .args(["-c", command])
        .kill_on_drop(true)
        .spawn()?;

    await_with_timeout(child, command).await
}

async fn await_with_timeout(child: Child, command: &str) -> Result<(), CoreError> {
    let result = tokio::time::timeout(HOOK_TIMEOUT, child.wait_with_output()).await;

    match result {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(())
            } else {
                let code = output.status.code().unwrap_or(-1);
                Err(CoreError::Io(std::io::Error::other(format!(
                    "hook {command:?} failed with exit code {code}"
                ))))
            }
        }
        Ok(Err(e)) => Err(CoreError::Io(e)),
        Err(_) => Err(CoreError::InstallerTimeout {
            timeout_secs: HOOK_TIMEOUT.as_secs(),
        }),
    }
}
