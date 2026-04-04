pub mod elevation;
pub mod exit_codes;
pub mod hooks;
pub mod ledger;
pub mod process;
pub mod switches;
pub mod types;
pub mod uninstall;
pub(crate) mod wide;
pub mod zip;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use tracing::{info, instrument, warn};

use crate::error::CoreError;
use crate::events::Event;
use crate::traits::Installer;
use crate::types::{Elevation, InstallMethod, UpgradeBehavior};

use self::exit_codes::interpret_exit_code;
use self::switches::build_args;
use self::types::{ExitCodeOutcome, InstallRequest, InstallResult, UninstallRequest};

/// Default installer timeout: 10 minutes.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(600);
const DEFAULT_INSTALL_SUBDIR: &str = "packages";

/// Facade for installer execution. Handles the full lifecycle:
/// pre-hooks -> elevation -> spawn -> exit code interpretation -> post-hooks -> ledger.
pub struct InstallerService {
    default_timeout: Duration,
    default_install_dir: PathBuf,
}

impl InstallerService {
    pub fn new(default_timeout: Duration, default_install_dir: PathBuf) -> Self {
        Self {
            default_timeout,
            default_install_dir,
        }
    }

    /// Creates a service with default timeout (600s) and the given data directory.
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self::new(DEFAULT_TIMEOUT, data_dir)
    }

    /// Returns the default timeout for installers without a manifest override.
    pub fn default_timeout(&self) -> Duration {
        self.default_timeout
    }

    fn install_dir_for(&self, package_id: &str) -> PathBuf {
        self.default_install_dir
            .join(DEFAULT_INSTALL_SUBDIR)
            .join(package_id)
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
        for hook_cmd in &config.pre_install {
            info!(hook = %hook_cmd, "running pre-install hook");
            if let Err(e) = hooks::run_hook(hook_cmd).await {
                let _ = request.event_tx.send(Event::InstallFailed {
                    id: request.package_id.clone(),
                    error: format!("pre-install hook failed: {e}"),
                });
                return Err(e);
            }
        }

        // Proactive elevation check
        #[cfg(windows)]
        if matches!(config.elevation, Some(Elevation::Required)) && !elevation::is_elevated() {
            info!("proactive elevation required, re-executing");
            let args: Vec<String> = std::env::args().collect();
            elevation::elevate_and_reexec(&args).await?;
            return Ok(InstallResult::Success { path: None });
        }
        #[cfg(not(windows))]
        if matches!(config.elevation, Some(Elevation::Required)) && !elevation::is_elevated() {
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

        if matches!(config.upgrade_behavior, Some(UpgradeBehavior::Deny)) {
            return Err(CoreError::UpgradeDenied {
                package_id: request.package_id.clone(),
            });
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
                if let Err(e) = hooks::run_hook(hook_cmd).await {
                    warn!(hook = %hook_cmd, error = %e, "post-install hook failed");
                }
            }

            // Resolve install_path: installer result -> detection chain -> None
            let result_path = match &result {
                Ok(InstallResult::Success { path })
                | Ok(InstallResult::SuccessRebootRequired { path }) => path.clone(),
                _ => None,
            };
            let install_path = if result_path.is_some() {
                result_path
            } else {
                self.resolve_install_path_via_detection(&request.detection_config)
                    .await
            };
            let entry = ledger::record_install(
                &request.package_id,
                &request.version,
                install_path.as_deref(),
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

    fn resolve_install_dir(&self, request: &InstallRequest) -> PathBuf {
        request
            .install_dir
            .clone()
            .unwrap_or_else(|| self.install_dir_for(&request.package_id))
    }

    async fn handle_zip_install(
        &self,
        request: &InstallRequest,
    ) -> Result<InstallResult, CoreError> {
        let dest = self.resolve_install_dir(request);
        let path = zip::extract_zip(&request.installer_path, &dest).await?;
        Ok(InstallResult::Success { path: Some(path) })
    }

    async fn handle_portable_install(
        &self,
        request: &InstallRequest,
    ) -> Result<InstallResult, CoreError> {
        let dest = self.resolve_install_dir(request);
        tokio::fs::create_dir_all(&dest).await?;
        let filename = request.installer_path.file_name().unwrap_or_default();
        tokio::fs::copy(&request.installer_path, dest.join(filename)).await?;
        Ok(InstallResult::Success { path: Some(dest) })
    }

    /// Handles DownloadOnly packages: opens the containing folder on Windows.
    /// On non-Windows, returns `Success` without opening a folder (no desktop
    /// environment assumed in CI/cross-compile targets).
    #[allow(unused_variables)]
    async fn handle_download_only(
        &self,
        installer_path: &std::path::Path,
    ) -> Result<InstallResult, CoreError> {
        #[cfg(windows)]
        if let Some(parent) = installer_path.parent() {
            std::process::Command::new("explorer").arg(parent).spawn()?;
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

    /// Attempt to resolve the install path by running the detection chain.
    ///
    /// Returns the `install_path` from `DetectionResult::Installed` if detection
    /// succeeds, or `None` if no config is provided or detection does not find
    /// an installed path.
    async fn resolve_install_path_via_detection(
        &self,
        detection_config: &Option<crate::types::DetectionConfig>,
    ) -> Option<PathBuf> {
        let Some(config) = detection_config else {
            return None;
        };
        let resolver = crate::detect::PathResolver::new();
        let result = crate::detect::run_chain(config, &resolver, None).await;
        match result {
            crate::detect::DetectionResult::Installed { install_path, .. }
            | crate::detect::DetectionResult::InstalledUnknownVersion { install_path, .. } => {
                install_path.map(PathBuf::from)
            }
            _ => None,
        }
    }

    fn record_metrics(&self, package_id: &str, start: Instant) {
        let duration = start.elapsed();
        info!(
            package = %package_id,
            metric = crate::metrics::INSTALL_DURATION_SECONDS,
            duration_secs = duration.as_secs_f64(),
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
