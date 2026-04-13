use std::path::Path;

use tracing::{info, instrument, warn};

use crate::error::CoreError;

/// Creates a Windows shortcut (.lnk) pointing to the target executable.
///
/// On non-Windows platforms this is a no-op.
///
/// # Arguments
/// * `target_exe` - Path to the executable the shortcut should launch.
/// * `shortcut_dir` - Directory where the `.lnk` file will be created.
/// * `name` - Display name for the shortcut (without `.lnk` extension).
#[instrument(skip_all, fields(target = %target_exe.display(), shortcut_dir = %shortcut_dir.display(), name))]
pub fn create_shortcut(
    target_exe: &Path,
    shortcut_dir: &Path,
    name: &str,
) -> Result<(), CoreError> {
    #[cfg(windows)]
    {
        create_shortcut_windows(target_exe, shortcut_dir, name)
    }
    #[cfg(not(windows))]
    {
        let _ = (target_exe, shortcut_dir, name);
        Ok(())
    }
}

#[cfg(windows)]
fn create_shortcut_windows(
    target_exe: &Path,
    shortcut_dir: &Path,
    name: &str,
) -> Result<(), CoreError> {
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        CoUninitialize,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::PCWSTR;

    use super::wide::to_wide_null;

    let target_str = target_exe.to_string_lossy();
    let working_dir = target_exe
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let lnk_path = shortcut_dir.join(format!("{name}.lnk"));
    let lnk_str = lnk_path.to_string_lossy();

    let target_wide = to_wide_null(&target_str);
    let working_dir_wide = to_wide_null(&working_dir);
    let lnk_wide = to_wide_null(&lnk_str);

    unsafe {
        // Initialize COM (apartment-threaded). If COM is already initialized on
        // this thread we still proceed — the subsequent calls will work.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| {
                CoreError::Io(std::io::Error::other(format!(
                    "failed to create ShellLink COM object: {e}"
                )))
            })?;

        shell_link
            .SetPath(PCWSTR(target_wide.as_ptr()))
            .map_err(|e| {
                CoreError::Io(std::io::Error::other(format!(
                    "IShellLinkW::SetPath failed: {e}"
                )))
            })?;

        shell_link
            .SetWorkingDirectory(PCWSTR(working_dir_wide.as_ptr()))
            .map_err(|e| {
                CoreError::Io(std::io::Error::other(format!(
                    "IShellLinkW::SetWorkingDirectory failed: {e}"
                )))
            })?;

        // QueryInterface for IPersistFile and save the .lnk
        let persist_file: windows::Win32::System::Com::IPersistFile =
            windows::core::Interface::cast(&shell_link).map_err(|e| {
                CoreError::Io(std::io::Error::other(format!(
                    "QueryInterface for IPersistFile failed: {e}"
                )))
            })?;

        persist_file
            .Save(PCWSTR(lnk_wide.as_ptr()), true)
            .map_err(|e| {
                CoreError::Io(std::io::Error::other(format!(
                    "IPersistFile::Save failed: {e}"
                )))
            })?;

        CoUninitialize();
    }

    info!(shortcut = %lnk_path.display(), target = %target_exe.display(), "created shortcut");
    Ok(())
}

/// Scans a directory for `.exe` files and returns the best candidate.
///
/// If exactly one `.exe` is found, it is returned. If multiple are found,
/// the largest one is picked (likely the main application). Returns `None`
/// if no executables are found.
#[instrument(skip_all, fields(dir = %dir.display()))]
pub fn find_main_executable(dir: &Path) -> Option<std::path::PathBuf> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(error = %e, "failed to read directory for exe scan");
            return None;
        }
    };

    let mut exes: Vec<(std::path::PathBuf, u64)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("exe") {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    exes.push((path, size));
                }
            }
        }
    }

    match exes.len() {
        0 => {
            info!("no exe files found in portable dir, skipping shortcut");
            None
        }
        1 => exes.into_iter().next().map(|(path, _)| path),
        n => {
            info!(count = n, "multiple exe files found, picking largest");
            exes.into_iter()
                .max_by_key(|(_, size)| *size)
                .map(|(p, _)| p)
        }
    }
}

/// Creates a shortcut for a portable app after install/extraction.
///
/// Scans the package directory for the main executable and creates a `.lnk`
/// shortcut in the specified shortcut directory. Failures are logged as
/// warnings but do not propagate — shortcut creation must never fail an
/// install.
#[instrument(skip_all, fields(package_dir = %package_dir.display(), package_name))]
pub fn create_portable_shortcut(package_dir: &Path, shortcut_dir: &Path, package_name: &str) {
    let Some(exe) = find_main_executable(package_dir) else {
        return;
    };

    if let Err(e) = create_shortcut(&exe, shortcut_dir, package_name) {
        warn!(
            error = %e,
            package_name,
            "failed to create shortcut for portable app"
        );
    }
}
