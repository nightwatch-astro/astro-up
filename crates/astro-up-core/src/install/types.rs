use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::events::Event;
use crate::types::{InstallConfig, InstallMethod, KnownExitCode};

/// Success outcomes from an install operation. Failures are `CoreError`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallResult {
    Success { path: Option<PathBuf> },
    SuccessRebootRequired { path: Option<PathBuf> },
    Cancelled,
}

/// Result of interpreting an installer's exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitCodeOutcome {
    Success,
    SuccessRebootRequired,
    ElevationRequired,
    Failed {
        code: i32,
        semantic: Option<KnownExitCode>,
    },
}

/// Input to the installer service.
#[derive(Debug)]
pub struct InstallRequest {
    pub package_id: String,
    pub package_name: String,
    pub version: crate::types::Version,
    pub installer_path: PathBuf,
    pub install_dir: Option<PathBuf>,
    pub install_config: InstallConfig,
    /// Detection config for post-install path resolution. When the installer does not
    /// return an explicit path (exe/msi installs), the service runs the detection chain
    /// to extract `install_path` for the ledger entry.
    pub detection_config: Option<crate::types::DetectionConfig>,
    pub timeout: Duration,
    pub quiet: bool,
    pub install_scope: crate::config::InstallScope,
    pub cancel_token: CancellationToken,
    pub event_tx: broadcast::Sender<Event>,
}

/// Input for uninstall operations.
#[derive(Debug)]
pub struct UninstallRequest {
    pub package_id: String,
    pub uninstall_command: Option<String>,
    pub install_dir: Option<PathBuf>,
    pub method: InstallMethod,
    pub quiet: bool,
    pub confirm: bool,
    pub cancel_token: CancellationToken,
}
