use std::path::Path;

use crate::error::CoreError;

/// Searches the Windows registry for the uninstall command of a package.
///
/// Checks both per-machine and per-user uninstall keys. Prefers
/// `QuietUninstallString` over `UninstallString`.
#[cfg(windows)]
pub fn find_uninstall_command(package_id: &str) -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let search_paths = [
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    let package_id_lower = package_id.to_lowercase();

    for (hive, path) in &search_paths {
        let Ok(key) = RegKey::predef(*hive).open_subkey(path) else {
            continue;
        };
        for name in key.enum_keys().flatten() {
            let Ok(subkey) = key.open_subkey(&name) else {
                continue;
            };
            let display_name: String = subkey.get_value("DisplayName").unwrap_or_default();
            if !display_name.to_lowercase().contains(&package_id_lower) {
                continue;
            }
            // Prefer QuietUninstallString
            if let Ok(quiet) = subkey.get_value::<String, _>("QuietUninstallString") {
                if !quiet.is_empty() {
                    return Some(quiet);
                }
            }
            if let Ok(uninstall) = subkey.get_value::<String, _>("UninstallString") {
                if !uninstall.is_empty() {
                    return Some(uninstall);
                }
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub fn find_uninstall_command(_package_id: &str) -> Option<String> {
    None
}

/// Executes an uninstall command silently.
///
/// Appends common silent switches if not already present in the command.
#[cfg(windows)]
pub async fn run_uninstall(command: &str, quiet: bool) -> Result<(), CoreError> {
    use tokio::process::Command;

    let mut parts: Vec<String> = shell_words_split(command);
    if parts.is_empty() {
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty uninstall command",
        )));
    }

    let exe = parts.remove(0);
    if quiet {
        // Add common silent switches if not already present
        let has_silent = parts.iter().any(|a| {
            let lower = a.to_lowercase();
            lower.contains("/silent")
                || lower.contains("/quiet")
                || lower.contains("/qn")
                || lower == "/s"
                || lower.contains("/verysilent")
        });
        if !has_silent {
            parts.push("/S".into());
        }
    }

    let status = Command::new(&exe)
        .args(&parts)
        .kill_on_drop(true)
        .status()
        .await?;

    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        Err(CoreError::Io(std::io::Error::other(format!(
            "uninstall command failed with exit code {code}"
        ))))
    }
}

#[cfg(not(windows))]
pub async fn run_uninstall(_command: &str, _quiet: bool) -> Result<(), CoreError> {
    Err(CoreError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "uninstall is only supported on Windows",
    )))
}

/// Removes a ZIP/portable install by deleting the directory.
///
/// Requires `confirm = true` — the caller (CLI/GUI) must prompt the user first.
pub async fn remove_directory(install_dir: &Path, confirm: bool) -> Result<(), CoreError> {
    if !confirm {
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "uninstall requires confirmation — set confirm=true after prompting user",
        )));
    }

    match tokio::fs::remove_dir_all(install_dir).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(CoreError::NotFound {
            input: install_dir.display().to_string(),
        }),
        Err(e) => Err(e.into()),
    }
}

/// Simple command-line splitting that handles quoted strings.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn shell_words_split(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for ch in input.chars() {
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
            }
            c if c == quote_char && in_quote => {
                in_quote = false;
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_words_simple() {
        let parts = shell_words_split("program /S /norestart");
        assert_eq!(parts, vec!["program", "/S", "/norestart"]);
    }

    #[test]
    fn shell_words_quoted_path() {
        let parts = shell_words_split(r#""C:\Program Files\app\uninstall.exe" /S"#);
        assert_eq!(parts, vec![r"C:\Program Files\app\uninstall.exe", "/S"]);
    }

    #[tokio::test]
    async fn remove_directory_requires_confirm() {
        let dir = tempfile::tempdir().unwrap();
        let result = remove_directory(dir.path(), false).await;
        assert!(result.is_err());
        assert!(dir.path().exists()); // Not deleted
    }

    #[tokio::test]
    async fn remove_directory_with_confirm() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();
        let path = dir.path().to_path_buf();
        let result = remove_directory(&path, true).await;
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn remove_directory_not_found() {
        let path = std::path::PathBuf::from("/nonexistent/path/12345");
        let result = remove_directory(&path, true).await;
        assert!(result.is_err());
    }
}
