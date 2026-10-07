//! Opt-in cooperative local ownership. No legacy shell, routing or retry fallback.
//! The caller supplies Orbit/Galaxy authority for one repository execution domain.
use pytxo_core::{PytxoError, Result};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PinnedFile {
    pub path: PathBuf,
    pub sha256: String,
}
impl PinnedFile {
    pub fn observe(path: PathBuf) -> Result<Self> {
        if !path.is_absolute() {
            return Err(PytxoError::Runner(
                "launch identity requires an absolute path".into(),
            ));
        }
        let sha256 = format!("{:x}", Sha256::digest(std::fs::read(&path)?));
        Ok(Self { path, sha256 })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnedTransport {
    Subprocess,
    Pty,
}
#[derive(Clone)]
pub struct OwnedLaunchSpec {
    pub bootstrap_host: PinnedFile,
    pub executable: PinnedFile,
    /// Complete adapter-resolved script/runtime chain, individually pinned.
    pub dependencies: Vec<PinnedFile>,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub working_directory: PathBuf,
    pub stdin: Vec<u8>,
    pub transport: OwnedTransport,
    pub barrier_timeout: Duration,
    pub execution_timeout: Duration,
    pub settlement_timeout: Duration,
    pub output_limit: usize,
}
impl std::fmt::Debug for OwnedLaunchSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedLaunchSpec")
            .field("dependency_count", &self.dependencies.len())
            .field("argument_count", &self.arguments.len())
            .field("environment_count", &self.environment.len())
            .field("stdin", &"[redacted]")
            .field("transport", &self.transport)
            .field("barrier_timeout", &self.barrier_timeout)
            .field("execution_timeout", &self.execution_timeout)
            .field("settlement_timeout", &self.settlement_timeout)
            .field("output_limit", &self.output_limit)
            .finish()
    }
}
/// A separately qualified subprocess backend. The payload is created suspended,
/// joined to its owned Job, durably registered, then resumed under the Stop gate.
/// This backend has no bootstrap executable, PTY, or private stdin channel.
#[derive(Clone)]
pub struct DirectOwnedLaunchSpec {
    pub executable: PinnedFile,
    pub arguments: Vec<String>,
    /// Only for a pinned Windows cmd.exe with exactly /D /C and one reviewed
    /// command tail. cmd.exe parses quotes as shell syntax, not CRT arguments.
    pub windows_cmd_verbatim_tail: bool,
    pub environment: BTreeMap<String, String>,
    pub working_directory: PathBuf,
    pub execution_timeout: Duration,
    pub settlement_timeout: Duration,
    pub output_limit: usize,
}
impl std::fmt::Debug for DirectOwnedLaunchSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectOwnedLaunchSpec")
            .field("argument_count", &self.arguments.len())
            .field("windows_cmd_verbatim_tail", &self.windows_cmd_verbatim_tail)
            .field("environment_count", &self.environment.len())
            .field("execution_timeout", &self.execution_timeout)
            .field("settlement_timeout", &self.settlement_timeout)
            .field("output_limit", &self.output_limit)
            .finish()
    }
}
#[derive(Clone, Debug)]
pub struct LaunchIntent {
    pub job_name: String,
    pub launch_nonce: String,
}
#[derive(Clone, Debug)]
pub struct OwnedProcess {
    pub pid: u32,
    pub start_identity: Option<String>,
    pub job_name: String,
}
/// Returned guard must hold the same short, cross-process gate used by Stop.
/// `authorize` durably records launching before returning it. No callback may
/// release that gate before this guard is dropped by the launcher.
pub trait LaunchGuard {
    /// Called under the launch gate after the final validation, cancellation,
    /// and deadline checks, immediately before native process creation. A
    /// durable create token must not be consumed during `authorize`.
    fn permit_create(&mut self, intent: &LaunchIntent) -> Result<()>;
    fn register(&mut self, process: &OwnedProcess) -> Result<()>;
    fn cancelled(&mut self) -> Result<bool>;
}
pub trait LaunchCallbacks {
    fn authorize(&mut self, intent: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>>;
    fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()>;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnedOutcome {
    Succeeded,
    Failed,
    Cancelled,
    RecoveryRequired,
}
#[derive(Clone)]
pub struct OwnedLaunchReceipt {
    pub outcome: OwnedOutcome,
    pub intent: LaunchIntent,
    /// Owned root: helper for `pytxo-attempt-host/1`, payload for direct mode.
    pub bootstrap: Option<OwnedProcess>,
    pub process_registered: bool,
    pub barrier_released: bool,
    /// Helper acknowledgement for the helper protocol; the direct protocol
    /// reports the OS-created root PID here. Neither attests provider identity.
    pub payload_pid_reported: Option<u32>,
    pub payload_exit_code: Option<u32>,
    /// Best-effort PID observations from live Job snapshots, not an exhaustive
    /// membership proof. Root membership and Job-zero settlement are authoritative.
    pub observed_job_pids: Vec<u32>,
    pub active_processes: Option<u32>,
    pub terminated_job: bool,
    pub exit_code: Option<u32>,
    pub stdout: String,
    pub stderr: String,
    /// False when stdout contained invalid UTF-8 before display decoding.
    /// Candidate protocols must reject such output; `stdout` is lossy then.
    pub stdout_utf8_valid: bool,
    pub output_complete: bool,
    pub output_truncated: bool,
    pub observed_model: Option<String>,
    pub observed_usage: Option<u64>,
    pub chain: Vec<PinnedFile>,
    pub bootstrap_sha256: String,
    pub protocol: String,
    /// Hash of the supplied adapter chain, helper protocol and argument lowering.
    /// It does not independently establish that the supplied dependency chain is complete.
    pub bundle_sha256: String,
    pub argument_lowering: String,
    pub elapsed_ms: u128,
    pub error: Option<String>,
}
impl std::fmt::Debug for OwnedLaunchReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedLaunchReceipt")
            .field("outcome", &self.outcome)
            .field("process_registered", &self.process_registered)
            .field("barrier_released", &self.barrier_released)
            .field("payload_exit_code", &self.payload_exit_code)
            .field("active_processes", &self.active_processes)
            .field("terminated_job", &self.terminated_job)
            .field("exit_code", &self.exit_code)
            .field("output_complete", &self.output_complete)
            .field("output_truncated", &self.output_truncated)
            .field("stdout_utf8_valid", &self.stdout_utf8_valid)
            .field("elapsed_ms", &self.elapsed_ms)
            .field("error_present", &self.error.is_some())
            .finish()
    }
}
pub fn run_owned_launch(
    spec: &OwnedLaunchSpec,
    callbacks: &mut dyn LaunchCallbacks,
    cancelled: impl Fn() -> Result<bool>,
) -> Result<OwnedLaunchReceipt> {
    #[cfg(windows)]
    {
        windows::run(spec, callbacks, cancelled)
    }
    #[cfg(not(windows))]
    {
        let _ = (spec, callbacks, cancelled);
        Err(PytxoError::Runner(
            "owned launch is unqualified on this platform".into(),
        ))
    }
}

pub fn run_direct_owned_launch(
    spec: &DirectOwnedLaunchSpec,
    callbacks: &mut dyn LaunchCallbacks,
    cancelled: impl Fn() -> Result<bool>,
) -> Result<OwnedLaunchReceipt> {
    #[cfg(windows)]
    {
        windows::run_direct(spec, callbacks, cancelled)
    }
    #[cfg(not(windows))]
    {
        let _ = (spec, callbacks, cancelled);
        Err(PytxoError::Runner(
            "direct owned launch is unqualified on this platform".into(),
        ))
    }
}

#[cfg(windows)]
#[path = "owned_launch_windows.rs"]
mod windows;

#[cfg(windows)]
pub(crate) fn system_cmd_path() -> Result<PathBuf> {
    Ok(windows::system_root()?.join("System32").join("cmd.exe"))
}

/// Resolve an adjacent packaged helper or a caller-qualified absolute pin. Never PATH-search.
/// Observing a new adjacent pin is not adapter qualification or launch authority.
pub fn resolve_attempt_host(qualified: Option<&PinnedFile>) -> Result<PinnedFile> {
    let pin = match qualified {
        Some(pin) => pin.clone(),
        None => PinnedFile::observe(std::env::current_exe()?.with_file_name(if cfg!(windows) {
            "pytxo-attempt-host.exe"
        } else {
            "pytxo-attempt-host"
        }))?,
    };
    if PinnedFile::observe(pin.path.clone())? != pin {
        return Err(PytxoError::Runner("attempt host identity changed".into()));
    }
    Ok(pin)
}

/// Narrow companion entry point. No configuration, project hooks, routing or provider work.
pub fn attempt_host_main() -> i32 {
    #[cfg(windows)]
    {
        windows::host_main().unwrap_or(125)
    }
    #[cfg(not(windows))]
    {
        125
    }
}

/// Read-only recovery observation. Missing/inaccessible/stale ownership never means zero.
/// Caller retains Core recovery authority; this function cannot release capacity or settle.
pub fn observe_owned_job(process: &OwnedProcess) -> Result<Option<u32>> {
    #[cfg(windows)]
    {
        windows::observe(process)
    }
    #[cfg(not(windows))]
    {
        let _ = process;
        Ok(None)
    }
}

/// Stop one Job named by an immutable, trusted owner row. Only a still-live
/// bootstrap with matching PID/start identity and Job membership can produce
/// release-grade `VerifiedZero`. A vanished root may leave children in its
/// exact named Job: those can be stopped, but the result is not a release proof.
/// This does not consult the legacy process registry or authorize Store state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnedJobStopResult {
    VerifiedZero,
    NamedJobTerminatedUnverified,
    Unknown,
}

pub fn terminate_owned_job(
    process: &OwnedProcess,
    launch_nonce: &str,
    timeout: Duration,
) -> Result<OwnedJobStopResult> {
    if timeout.is_zero() || timeout > Duration::from_secs(60) {
        return Err(PytxoError::Runner(
            "owned Job stop timeout is invalid".into(),
        ));
    }
    let canonical = uuid::Uuid::parse_str(launch_nonce).is_ok_and(|value| {
        value.get_version_num() == 4 && value.hyphenated().to_string() == launch_nonce
    });
    if !canonical || process.job_name != format!("Local\\PytxoAttempt-{launch_nonce}") {
        return Err(PytxoError::Runner(
            "owned Job name and launch nonce do not match".into(),
        ));
    }
    #[cfg(windows)]
    {
        windows::terminate_registered(process, timeout)
    }
    #[cfg(not(windows))]
    {
        let _ = process;
        Ok(OwnedJobStopResult::Unknown)
    }
}
