// Contract: InstallerService — spec 011 installer execution
// This is a design contract, not compilable code.

use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

// --- New types (install/types.rs) ---

/// Input to the installer service.
pub struct InstallRequest {
    pub package_id: String,
    pub package_name: String,
    pub version: Version,
    pub installer_path: PathBuf,
    pub install_dir: Option<PathBuf>,
    pub install_config: InstallConfig, // from types/install.rs (spec 003)
    pub timeout: Duration,             // effective timeout (manifest or default 600s)
    pub quiet: bool,
    pub cancel_token: CancellationToken,
    pub event_tx: broadcast::Sender<Event>,
}

/// Success outcomes. Failures are CoreError.
pub enum InstallResult {
    Success { path: Option<PathBuf> },
    SuccessRebootRequired { path: Option<PathBuf> },
    Cancelled,
}

/// Input for uninstall operations.
pub struct UninstallRequest {
    pub package_id: String,
    pub uninstall_command: Option<String>, // from registry
    pub install_dir: Option<PathBuf>,      // for ZIP/portable deletion
    pub method: InstallMethod,
    pub quiet: bool,
    pub confirm: bool,                     // required for directory deletion
    pub cancel_token: CancellationToken,
}

// --- Modified trait (traits.rs) ---

/// Updated Installer trait — returns InstallResult instead of ().
#[trait_variant::make(InstallerDyn: Send)]
pub trait Installer: Send {
    async fn install(&self, request: &InstallRequest) -> Result<InstallResult, CoreError>;
    async fn uninstall(&self, request: &UninstallRequest) -> Result<(), CoreError>;
    fn supports(&self, method: &InstallMethod) -> bool;
}

// --- Service (install/mod.rs) ---

/// Facade for installer execution. Implements the Installer trait.
pub struct InstallerService {
    default_timeout: Duration,
    default_install_dir: PathBuf, // paths.data_dir
}

impl InstallerService {
    pub fn new(default_timeout: Duration, default_install_dir: PathBuf) -> Self;

    // Internal methods (not part of trait):

    /// Resolve effective switches: manifest overrides or type defaults.
    fn resolve_switches(config: &InstallConfig) -> Vec<String>;

    /// Interpret exit code using success_codes -> known_exit_codes -> defaults.
    fn interpret_exit_code(code: i32, config: &InstallConfig) -> ExitCodeOutcome;

    /// Run pre/post hooks (.ps1 via PowerShell, else cmd /c). 60s timeout.
    /// Hooks inherit the current process elevation implicitly.
    async fn run_hook(command: &str) -> Result<(), CoreError>;

    /// Record successful install in ledger.
    async fn record_ledger(
        package_id: &str,
        version: &Version,
        install_path: Option<&Path>,
    ) -> Result<(), CoreError>;
}

// --- Platform-specific (cfg(windows)) ---

#[cfg(windows)]
impl InstallerService {
    /// Check if current process has admin privileges.
    fn is_elevated() -> bool;

    /// Re-exec via sudo (if available) or ShellExecuteExW runas.
    async fn elevate_and_reexec(args: &[String]) -> Result<(), CoreError>;

    /// Spawn process with Job Object for tree waiting.
    async fn spawn_with_job_object(
        exe: &Path,
        args: &[String],
        timeout: Duration,
        cancel_token: CancellationToken,
    ) -> Result<i32, CoreError>;

    /// Spawn simple process (no Job Object).
    async fn spawn_simple(
        exe: &Path,
        args: &[String],
        timeout: Duration,
        cancel_token: CancellationToken,
    ) -> Result<i32, CoreError>;

    /// Extract uninstall string from registry.
    fn find_uninstall_command(package_id: &str) -> Option<String>;
}

// --- ZIP extraction (install/zip.rs, cross-platform) ---

/// Extract ZIP with zip-slip protection and single-root flattening.
pub async fn extract_zip(
    archive_path: &Path,
    dest_dir: &Path,
) -> Result<PathBuf, CoreError>;

// --- Exit code interpretation (install/exit_codes.rs, cross-platform) ---

pub enum ExitCodeOutcome {
    Success,
    SuccessRebootRequired,
    ElevationRequired,
    Failed { code: i32, semantic: Option<KnownExitCode> },
}
