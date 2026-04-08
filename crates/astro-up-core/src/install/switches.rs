use std::path::Path;

use crate::types::{InstallConfig, InstallMethod};

/// Returns the default silent switches for an installer type.
fn default_silent_switches(method: &InstallMethod) -> Vec<String> {
    match method {
        InstallMethod::InnoSetup => vec![
            "/VERYSILENT".into(),
            "/NORESTART".into(),
            "/SUPPRESSMSGBOXES".into(),
        ],
        InstallMethod::Msi => vec!["/qn".into(), "/norestart".into()],
        InstallMethod::Nsis => vec!["/S".into()],
        InstallMethod::Wix | InstallMethod::Burn => {
            vec!["/quiet".into(), "/norestart".into()]
        }
        // Exe, Zip, Portable, DownloadOnly have no default switches
        InstallMethod::Exe
        | InstallMethod::Zip
        | InstallMethod::Portable
        | InstallMethod::DownloadOnly => vec![],
    }
}

/// Resolves the effective silent switches for an install operation.
///
/// - If `config.switches` is `Some` with a `silent` list: use those (even if empty).
///   An empty list means "no switches" — run installer with no arguments.
/// - If `config.switches` is `None`: use type defaults.
pub fn resolve_switches(config: &InstallConfig) -> Vec<String> {
    match &config.switches {
        Some(switches) => switches.silent.clone(),
        None => default_silent_switches(&config.method),
    }
}

/// Returns the install directory switch for a given method, if applicable.
fn install_dir_switch(method: &InstallMethod, dir: &Path) -> Option<String> {
    let dir_str = dir.display();
    match method {
        InstallMethod::InnoSetup => Some(format!("/DIR={dir_str}")),
        InstallMethod::Msi => Some(format!("INSTALLDIR={dir_str}")),
        InstallMethod::Nsis => Some(format!("/D={dir_str}")),
        InstallMethod::Wix | InstallMethod::Burn => Some(format!("INSTALLDIR={dir_str}")),
        // Zip/Portable use the dir directly (not as a switch)
        // Exe, DownloadOnly: no standard directory switch
        _ => None,
    }
}

/// Builds the complete argument list for an installer invocation.
///
/// For MSI: returns args for msiexec (the installer path is passed as `/i <path>`).
/// For other types: returns args to pass to the installer executable directly.
pub fn build_args(
    config: &InstallConfig,
    installer_path: &Path,
    install_dir: Option<&Path>,
) -> (String, Vec<String>) {
    let mut args = Vec::new();

    match config.method {
        InstallMethod::Msi => {
            // MSI uses msiexec as the executable
            args.push("/i".into());
            args.push(installer_path.display().to_string());
            args.extend(resolve_switches(config));
            if let Some(dir) = install_dir {
                if let Some(switch) = install_dir_switch(&config.method, dir) {
                    args.push(switch);
                }
            }
            ("msiexec".into(), args)
        }
        _ => {
            // All other types: run the installer directly
            args.extend(resolve_switches(config));
            if let Some(dir) = install_dir {
                if let Some(switch) = install_dir_switch(&config.method, dir) {
                    args.push(switch);
                }
            }
            (installer_path.display().to_string(), args)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::{InstallConfig, InstallMethod, InstallerSwitches};

    fn config_for(method: InstallMethod) -> InstallConfig {
        InstallConfig {
            method,
            ..Default::default()
        }
    }

    #[test]
    fn innosetup_defaults() {
        let config = config_for(InstallMethod::InnoSetup);
        let switches = resolve_switches(&config);
        assert_eq!(
            switches,
            vec!["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"]
        );
    }

    #[test]
    fn msi_defaults() {
        let config = config_for(InstallMethod::Msi);
        let switches = resolve_switches(&config);
        assert_eq!(switches, vec!["/qn", "/norestart"]);
    }

    #[test]
    fn nsis_defaults() {
        let config = config_for(InstallMethod::Nsis);
        let switches = resolve_switches(&config);
        assert_eq!(switches, vec!["/S"]);
    }

    #[test]
    fn wix_defaults() {
        let config = config_for(InstallMethod::Wix);
        let switches = resolve_switches(&config);
        assert_eq!(switches, vec!["/quiet", "/norestart"]);
    }

    #[test]
    fn burn_defaults() {
        let config = config_for(InstallMethod::Burn);
        let switches = resolve_switches(&config);
        assert_eq!(switches, vec!["/quiet", "/norestart"]);
    }

    #[test]
    fn exe_no_defaults() {
        let config = config_for(InstallMethod::Exe);
        let switches = resolve_switches(&config);
        assert!(switches.is_empty());
    }

    #[test]
    fn zip_no_defaults() {
        let config = config_for(InstallMethod::Zip);
        let switches = resolve_switches(&config);
        assert!(switches.is_empty());
    }

    #[test]
    fn portable_no_defaults() {
        let config = config_for(InstallMethod::Portable);
        let switches = resolve_switches(&config);
        assert!(switches.is_empty());
    }

    #[test]
    fn download_only_no_defaults() {
        let config = config_for(InstallMethod::DownloadOnly);
        let switches = resolve_switches(&config);
        assert!(switches.is_empty());
    }

    #[test]
    fn custom_switches_replace_defaults() {
        let mut config = config_for(InstallMethod::InnoSetup);
        config.switches = Some(InstallerSwitches {
            silent: vec!["/SILENT".into(), "/CUSTOM".into()],
            interactive: vec![],
            upgrade: vec![],
            install_location: None,
            log: None,
            custom: vec![],
        });
        let switches = resolve_switches(&config);
        assert_eq!(switches, vec!["/SILENT", "/CUSTOM"]);
    }

    #[test]
    fn empty_switches_means_no_switches() {
        let mut config = config_for(InstallMethod::InnoSetup);
        config.switches = Some(InstallerSwitches {
            silent: vec![],
            interactive: vec![],
            upgrade: vec![],
            install_location: None,
            log: None,
            custom: vec![],
        });
        let switches = resolve_switches(&config);
        assert!(switches.is_empty());
    }

    #[test]
    fn msi_build_args() {
        let config = config_for(InstallMethod::Msi);
        let (exe, args) = build_args(&config, Path::new("installer.msi"), None);
        assert_eq!(exe, "msiexec");
        assert_eq!(args, vec!["/i", "installer.msi", "/qn", "/norestart"]);
    }

    #[test]
    fn innosetup_with_dir_override() {
        let config = config_for(InstallMethod::InnoSetup);
        let dir = Path::new("C:\\Programs\\NINA");
        let (exe, args) = build_args(&config, Path::new("setup.exe"), Some(dir));
        assert_eq!(exe, "setup.exe");
        assert!(args.contains(&"/DIR=C:\\Programs\\NINA".to_string()));
    }

    #[test]
    fn nsis_with_dir_override() {
        let config = config_for(InstallMethod::Nsis);
        let dir = Path::new("C:\\Programs\\PHD2");
        let (_, args) = build_args(&config, Path::new("setup.exe"), Some(dir));
        assert!(args.contains(&"/D=C:\\Programs\\PHD2".to_string()));
    }

    #[test]
    fn msi_with_dir_override() {
        let config = config_for(InstallMethod::Msi);
        let dir = Path::new("C:\\Programs\\ASCOM");
        let (exe, args) = build_args(&config, Path::new("driver.msi"), Some(dir));
        assert_eq!(exe, "msiexec");
        assert!(args.contains(&"INSTALLDIR=C:\\Programs\\ASCOM".to_string()));
    }

    #[test]
    fn wix_with_dir_override() {
        let config = config_for(InstallMethod::Wix);
        let dir = Path::new("C:\\Programs\\SharpCap");
        let (_, args) = build_args(&config, Path::new("setup.exe"), Some(dir));
        assert!(args.contains(&"INSTALLDIR=C:\\Programs\\SharpCap".to_string()));
    }
}
