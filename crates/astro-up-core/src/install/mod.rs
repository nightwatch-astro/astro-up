pub mod elevation;
pub mod exit_codes;
pub mod hooks;
pub mod ledger;
pub mod process;
pub mod switches;
pub mod types;
pub mod uninstall;
pub mod zip;

use std::path::PathBuf;
use std::time::Instant;

use tracing::{info, instrument, warn};

use crate::error::CoreError;
use crate::events::Event;
use crate::traits::Installer;
use crate::types::{Elevation, InstallMethod, UpgradeBehavior};

use self::exit_codes::interpret_exit_code;
use self::switches::build_args;
use self::types::{ExitCodeOutcome, InstallRequest, InstallResult, UninstallRequest};

const DEFAULT_INSTALL_SUBDIR: &str = "packages";

/// Facade for installer execution. Handles the full lifecycle:
/// pre-hooks -> elevation -> spawn -> exit code interpretation -> post-hooks -> ledger.
pub struct InstallerService {
    data_dir: PathBuf,
}

impl InstallerService {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn default_install_dir(&self, package_id: &str) -> PathBuf {
        self.data_dir.join(DEFAULT_INSTALL_SUBDIR).join(package_id)
    }

    #[instrument(skip_all, fields(package = %request.package_id))]
    pub async fn install(&self, request: &InstallRequest) -> Result<InstallResult, CoreError> {
        let start = Instant::now();
        let config = &request.install_config;

        let _ = request.event_tx.send(Event::InstallStarted {
            id: request.package_id.clone(),
        });

        // DownloadOnly: open folder, no execution
        if config.method == InstallMethod::DownloadOnly {
            let result = self.handle_download_only(&request.installer_path).await;
            self.record_metrics(&request.package_id, start);
            return result;
        }

        // Pre-install hooks (abort on failure)
        let is_elevated = elevation::is_elevated();
        for hook_cmd in &config.pre_install {
            info!(hook = %hook_cmd, "running pre-install hook");
            if let Err(e) = hooks::run_hook(hook_cmd, is_elevated).await {
                let _ = request.event_tx.send(Event::InstallFailed {
                    id: request.package_id.clone(),
                    error: format!("pre-install hook failed: {e}"),
                });
                return Err(e);
            }
        }

        // Proactive elevation check
        #[cfg(windows)]
        if matches!(config.elevation, Some(Elevation::Required)) && !is_elevated {
            info!("proactive elevation required, re-executing");
            let args: Vec<String> = std::env::args().collect();
            elevation::elevate_and_reexec(&args).await?;
            return Ok(InstallResult::Success { path: None });
        }
        #[cfg(not(windows))]
        if matches!(config.elevation, Some(Elevation::Required)) && !is_elevated {
            return Err(CoreError::ElevationRequired);
        }

        // upgrade_behavior = uninstall_previous
        if matches!(
            config.upgrade_behavior,
            Some(UpgradeBehavior::UninstallPrevious)
        ) {
            info!("upgrade_behavior=uninstall_previous, uninstalling current version");
            let uninstall_cmd = uninstall::find_uninstall_command(&request.package_id);
            if let Some(cmd) = uninstall_cmd {
                uninstall::run_uninstall(&cmd, request.quiet).await?;
            }
        }

        // Deny upgrade if configured
        if matches!(config.upgrade_behavior, Some(UpgradeBehavior::Deny)) {
            return Err(CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("upgrade denied for {}", request.package_id),
            )));
        }

        // Execute based on method
        let result = match config.method {
            InstallMethod::Zip | InstallMethod::ZipWrap => self.handle_zip_install(request).await,
            InstallMethod::Portable => self.handle_portable_install(request).await,
            _ => self.handle_exe_install(request).await,
        };

        // Post-install hooks (warn on failure, don't abort)
        if result.is_ok() {
            for hook_cmd in &config.post_install {
                info!(hook = %hook_cmd, "running post-install hook");
                if let Err(e) = hooks::run_hook(hook_cmd, is_elevated).await {
                    warn!(hook = %hook_cmd, error = %e, "post-install hook failed");
                }
            }

            // Record ledger entry
            let install_path = match &result {
                Ok(InstallResult::Success { path })
                | Ok(InstallResult::SuccessRebootRequired { path }) => path.as_ref(),
                _ => None,
            };
            let entry = ledger::record_install(
                &request.package_id,
                &request.version,
                install_path,
            );
            info!(package = %entry.package_id, "recorded install in ledger");
        }

        // Emit completion events
        match &result {
            Ok(InstallResult::Success { .. } | InstallResult::Cancelled) => {
                let _ = request.event_tx.send(Event::InstallComplete {
                    id: request.package_id.clone(),
                });
            }
            Ok(InstallResult::SuccessRebootRequired { .. }) => {
                let _ = request.event_tx.send(Event::InstallRebootRequired {
                    id: request.package_id.clone(),
                });
            }
            Err(e) => {
                let _ = request.event_tx.send(Event::InstallFailed {
                    id: request.package_id.clone(),
                    error: e.to_string(),
                });
            }
        }

        self.record_metrics(&request.package_id, start);
        result
    }

    async fn handle_exe_install(
        &self,
        request: &InstallRequest,
    ) -> Result<InstallResult, CoreError> {
        let config = &request.install_config;
        let (exe, args) = build_args(
            config,
            &request.installer_path,
            request.install_dir.as_deref(),
        );

        let exit_code = if matches!(config.method, InstallMethod::Burn) {
            process::spawn_with_job_object(
                &exe,
                &args,
                request.timeout,
                request.cancel_token.clone(),
            )
            .await?
        } else {
            process::spawn_simple(&exe, &args, request.timeout, request.cancel_token.clone())
                .await?
        };

        let outcome = interpret_exit_code(exit_code, config);

        match outcome {
            ExitCodeOutcome::Success => Ok(InstallResult::Success { path: None }),
            ExitCodeOutcome::SuccessRebootRequired => {
                Ok(InstallResult::SuccessRebootRequired { path: None })
            }
            ExitCodeOutcome::ElevationRequired => {
                #[cfg(windows)]
                {
                    info!("reactive elevation (exit code 740), re-executing");
                    let args_vec: Vec<String> = std::env::args().collect();
                    elevation::elevate_and_reexec(&args_vec).await?;
                    Ok(InstallResult::Success { path: None })
                }
                #[cfg(not(windows))]
                Err(CoreError::ElevationRequired)
            }
            ExitCodeOutcome::Failed { code, semantic } => {
                if let Some(known) = semantic {
                    Err(CoreError::InstallerFailed {
                        exit_code: code,
                        response: known,
                    })
                } else {
                    Err(CoreError::Io(std::io::Error::other(format!(
                        "installer failed with exit code {code}"
                    ))))
                }
            }
        }
    }

    async fn handle_zip_install(
        &self,
        request: &InstallRequest,
    ) -> Result<InstallResult, CoreError> {
        let dest = request
            .install_dir
            .clone()
            .unwrap_or_else(|| self.default_install_dir(&request.package_id));
        let path = zip::extract_zip(&request.installer_path, &dest).await?;
        Ok(InstallResult::Success { path: Some(path) })
    }

    async fn handle_portable_install(
        &self,
        request: &InstallRequest,
    ) -> Result<InstallResult, CoreError> {
        let dest = request
            .install_dir
            .clone()
            .unwrap_or_else(|| self.default_install_dir(&request.package_id));
        tokio::fs::create_dir_all(&dest).await?;
        let filename = request.installer_path.file_name().unwrap_or_default();
        tokio::fs::copy(&request.installer_path, dest.join(filename)).await?;
        Ok(InstallResult::Success { path: Some(dest) })
    }

    #[allow(unused_variables)]
    async fn handle_download_only(
        &self,
        installer_path: &std::path::Path,
    ) -> Result<InstallResult, CoreError> {
        #[cfg(windows)]
        if let Some(parent) = installer_path.parent() {
            let parent = parent.to_path_buf();
            tokio::task::spawn_blocking(move || {
                use std::ffi::OsStr;
                use std::os::windows::ffi::OsStrExt;

                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::Shell::ShellExecuteW;
                use windows::core::PCWSTR;

                let verb: Vec<u16> = OsStr::new("open")
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                let dir: Vec<u16> = OsStr::new(&parent)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                unsafe {
                    ShellExecuteW(
                        HWND::default(),
                        PCWSTR(verb.as_ptr()),
                        PCWSTR(dir.as_ptr()),
                        None,
                        None,
                        windows::Win32::UI::WindowsAndMessaging::SW_SHOW.0,
                    );
                }
            })
            .await
            .map_err(|e| CoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        }
        Ok(InstallResult::Success { path: None })
    }

    #[instrument(skip_all, fields(package = %request.package_id))]
    pub async fn uninstall(&self, request: &UninstallRequest) -> Result<(), CoreError> {
        match request.method {
            InstallMethod::Zip | InstallMethod::ZipWrap | InstallMethod::Portable => {
                let dir = request
                    .install_dir
                    .as_ref()
                    .ok_or_else(|| CoreError::NotFound {
                        input: format!(
                            "no install directory known for {} - uninstall not supported",
                            request.package_id
                        ),
                    })?;
                uninstall::remove_directory(dir, request.confirm).await
            }
            _ => {
                let cmd =
                    request
                        .uninstall_command
                        .as_deref()
                        .ok_or_else(|| CoreError::NotFound {
                            input: format!("no uninstall command found for {}", request.package_id),
                        })?;
                uninstall::run_uninstall(cmd, request.quiet).await
            }
        }
    }

    fn record_metrics(&self, package_id: &str, start: Instant) {
        let duration = start.elapsed();
        info!(
            package = %package_id,
            metric = crate::metrics::INSTALL_DURATION_SECONDS,
            duration_secs = duration.as_secs_f64(),
            duration_ms = duration.as_millis() as u64,
            "install completed"
        );
    }
}

impl Installer for InstallerService {
    async fn install(&self, request: &InstallRequest) -> Result<InstallResult, CoreError> {
        self.install(request).await
    }

    async fn uninstall(&self, request: &UninstallRequest) -> Result<(), CoreError> {
        self.uninstall(request).await
    }

    fn supports(&self, method: &InstallMethod) -> bool {
        // All 10 installer types are supported
        matches!(
            method,
            InstallMethod::Exe
                | InstallMethod::Msi
                | InstallMethod::InnoSetup
                | InstallMethod::Nullsoft
                | InstallMethod::Wix
                | InstallMethod::Burn
                | InstallMethod::Zip
                | InstallMethod::ZipWrap
                | InstallMethod::Portable
                | InstallMethod::DownloadOnly
        )
    }
}
