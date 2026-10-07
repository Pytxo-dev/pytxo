//! One owned routed worker. The caller must already have admitted the exact
//! attempt, bound its Catalog token, prepared the private launch owner, and
//! materialized the reviewed inputs. This module does not grant those steps.

use std::collections::{BTreeMap, BTreeSet};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use pytxo_core::routing::{
    canonical_digest, AttemptId, AttemptState, BillingSourceMode, Digest, ExecutableIdentity,
    ExecutionProfile, LaunchContract, LaunchTransport, ProfileBinding, StdinDelivery,
};
use pytxo_core::{ExecutionBackend, PytxoError, Result, TaskId};
use pytxo_runner::owned_launch::{
    run_direct_owned_launch, run_owned_launch, DirectOwnedLaunchSpec, LaunchCallbacks, LaunchGuard,
    LaunchIntent, OwnedLaunchReceipt, OwnedLaunchSpec, OwnedProcess, OwnedTransport, PinnedFile,
};
use pytxo_runner::{
    registry_path, verify_materialized_dependency_output, verify_materialized_reviewed_inputs,
    ProcessEntry, ProcessRegistryFile, ReviewedInputManifest, SealedOutputSnapshot,
};
use pytxo_store::capacity::CapacityReservationState;
use pytxo_store::routing::{
    RoutedAttemptRecord, RoutingFacts, RoutingReceipts, RoutingScope, TransitionRoutingAttempt,
};
use pytxo_store::routing_launch::{
    LaunchCreateOutcome, LaunchCreateRequest, LaunchOwnershipPhase, LaunchProcessRegistration,
    LaunchSettlement,
};
use pytxo_store::routing_private::{
    ControllerObservation, ControllerReceiptEnvelope, PrivateArtifactClaim, PrivateArtifactKind,
    ReceiptSource,
};
use pytxo_store::{Catalog, PytxoStore};

use crate::flow::{load_reviewed_staged_mission, FlowPlan};
use crate::routed_claude::{
    claude_edit_arguments, claude_proposal_arguments, reject_reparse_components,
    reject_reparse_descendants, CLAUDE_ADAPTER_ID, CLAUDE_PROPOSAL_ADAPTER_ID,
    CLAUDE_PROPOSAL_TOOL_BUNDLE_ID, CLAUDE_SUBSCRIPTION_ENDPOINT, CLAUDE_TOOL_BUNDLE_ID,
};
use crate::routed_codex::{codex_exec_arguments, CODEX_ADAPTER_ID, CODEX_SUBSCRIPTION_ENDPOINT};
use crate::{ActiveRunGate, ActiveRunState};

pub(crate) struct RoutedOwnedWorker<'a> {
    pub store: &'a PytxoStore,
    pub catalog: &'a Catalog,
    pub plan: &'a FlowPlan,
    pub repo_root: &'a Path,
    pub data_dir: &'a Path,
    pub worktree: &'a Path,
    pub inputs: &'a ReviewedInputManifest,
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub facts: RoutingFacts,
    pub spec: DirectOwnedLaunchSpec,
}

/// Hosted payloads use a separate qualified launch protocol so the private
/// prompt never has to pass through the direct fixture's closed stdin.
#[allow(dead_code)]
pub(crate) struct RoutedHostedWorker<'a> {
    pub store: &'a PytxoStore,
    pub catalog: &'a Catalog,
    pub plan: &'a FlowPlan,
    pub repo_root: &'a Path,
    pub data_dir: &'a Path,
    pub worktree: &'a Path,
    pub inputs: &'a ReviewedInputManifest,
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub facts: RoutingFacts,
    pub spec: OwnedLaunchSpec,
}

/// Resolve the one supported predecessor through Core's registered graph and
/// exact retained winner. Only a trusted local controller may handle these
/// bytes; this read does not authorize a launch.
pub(crate) fn inherited_snapshot_for_attempt(
    store: &PytxoStore,
    attempt: &RoutedAttemptRecord,
) -> Result<Option<SealedOutputSnapshot>> {
    let dependency = match attempt.dependencies.as_slice() {
        [] => return Ok(None),
        [dependency] => dependency,
        _ => return Err(failure("local routed worker supports one predecessor")),
    };
    let retained = store.read_routing_dependency_output(
        &attempt.scope,
        &attempt.task_id,
        &dependency.task_id,
    )?;
    if retained.winner != *dependency || retained.sealed_output.digest != dependency.output_digest {
        return Err(failure("dependency winner changed before native launch"));
    }
    let inherited: SealedOutputSnapshot = serde_json::from_slice(&retained.bytes)
        .map_err(|error| failure(&format!("dependency output is invalid: {error}")))?;
    if serde_json::to_vec(&inherited)
        .map_err(|error| failure(&format!("dependency output is invalid: {error}")))?
        != retained.bytes
    {
        return Err(failure("dependency output serialization changed"));
    }
    Ok(Some(inherited))
}

pub(crate) fn run_prepared_owned_worker(
    worker: &RoutedOwnedWorker<'_>,
) -> Result<OwnedLaunchReceipt> {
    let mut callbacks = WorkerCallbacks {
        worker: WorkerIdentity::from_direct(worker),
        launch: WorkerLaunch::Direct(&worker.spec),
    };
    let receipt = run_direct_owned_launch(&worker.spec, &mut callbacks, || {
        let registry = ProcessRegistryFile::load(&registry_path(worker.data_dir))?;
        Ok(crate::routed_fixture::routed_stop_requested(
            worker.catalog,
            &worker.plan.draft_id,
            &worker.scope.run_id.0,
        )? || registry.cancelled_runs.contains(&worker.scope.run_id.0))
    })?;
    require_job_settlement(worker.store, &worker.attempt_id, &receipt)?;
    Ok(receipt)
}

#[allow(dead_code)]
pub(crate) fn run_prepared_hosted_owned_worker(
    worker: &RoutedHostedWorker<'_>,
) -> Result<OwnedLaunchReceipt> {
    let mut callbacks = WorkerCallbacks {
        worker: WorkerIdentity::from_hosted(worker),
        launch: WorkerLaunch::Hosted(&worker.spec),
    };
    let receipt = run_owned_launch(&worker.spec, &mut callbacks, || {
        let registry = ProcessRegistryFile::load(&registry_path(worker.data_dir))?;
        Ok(crate::routed_fixture::routed_stop_requested(
            worker.catalog,
            &worker.plan.draft_id,
            &worker.scope.run_id.0,
        )? || registry.cancelled_runs.contains(&worker.scope.run_id.0))
    })?;
    require_job_settlement(worker.store, &worker.attempt_id, &receipt)?;
    Ok(receipt)
}

fn require_job_settlement(
    store: &PytxoStore,
    attempt_id: &AttemptId,
    receipt: &OwnedLaunchReceipt,
) -> Result<()> {
    if receipt.process_registered && receipt.active_processes == Some(0) {
        let owner = store
            .launch_ownership(attempt_id)?
            .ok_or_else(|| failure("registered routed worker owner disappeared"))?;
        if owner.phase != LaunchOwnershipPhase::Settled {
            return Err(failure(
                "routed worker Job-zero receipt was not durably settled",
            ));
        }
    }
    Ok(())
}

fn validate_direct_launch_shape(
    launch: &LaunchContract,
    spec: &DirectOwnedLaunchSpec,
    worktree: &Path,
) -> Result<()> {
    if launch.schema_version != 1
        || launch.transport != LaunchTransport::DirectSubprocess
        || launch.stdin_delivery != StdinDelivery::Closed
        || launch.private_stdin_digest.is_some()
        || launch.host.is_some()
        || !launch.dependencies.is_empty()
        || launch.output_protocol != "pytxo-direct-suspended/1"
        || launch.argument_lowering != "windows-createprocess-structured-argv/v1"
        || spec.windows_cmd_verbatim_tail
        || launch.working_directory_policy != "reviewed_attempt_worktree_v1"
        || spec.working_directory.as_path() != worktree
        || launch.barrier_timeout_ms.is_some()
        || u128::from(launch.execution_timeout_ms) != spec.execution_timeout.as_millis()
        || u128::from(launch.settlement_timeout_ms) != spec.settlement_timeout.as_millis()
        || u128::from(launch.output_limit_bytes) != spec.output_limit as u128
        || launch.arguments_digest
            != canonical_digest(&spec.arguments, 1).map_err(|error| failure(&error.to_string()))?
        || launch.environment_policy_digest
            != canonical_digest(&spec.environment, 1)
                .map_err(|error| failure(&error.to_string()))?
    {
        return Err(failure("routed launch shape changed after qualification"));
    }
    Ok(())
}

fn pin_matches_identity(pin: &PinnedFile, identity: &ExecutableIdentity) -> bool {
    pin.path.to_string_lossy() == identity.path && Digest(pin.sha256.clone()) == identity.digest
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetWindowsDirectoryW(buffer: *mut u16, capacity: u32) -> u32;
    fn CompareStringOrdinal(
        left: *const u16,
        left_len: i32,
        right: *const u16,
        right_len: i32,
        ignore_case: i32,
    ) -> i32;
}

#[cfg(windows)]
pub(crate) fn observed_windows_root() -> Result<String> {
    let mut buffer = vec![0_u16; 32_768];
    let len = unsafe { GetWindowsDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) } as usize;
    if len == 0 || len >= buffer.len() {
        return Err(failure("cannot resolve operating-system Windows root"));
    }
    String::from_utf16(&buffer[..len])
        .map_err(|_| failure("operating-system Windows root is invalid"))
}

#[cfg(not(windows))]
pub(crate) fn observed_windows_root() -> Result<String> {
    Err(failure("Codex hosted environment requires Windows"))
}

#[cfg(windows)]
pub(crate) fn same_windows_ordinal_case(
    left: &std::ffi::OsStr,
    right: &std::ffi::OsStr,
) -> Result<bool> {
    let left: Vec<u16> = left.encode_wide().collect();
    let right: Vec<u16> = right.encode_wide().collect();
    let left_len = i32::try_from(left.len()).map_err(|_| failure("Windows path is too long"))?;
    let right_len = i32::try_from(right.len()).map_err(|_| failure("Windows path is too long"))?;
    match unsafe { CompareStringOrdinal(left.as_ptr(), left_len, right.as_ptr(), right_len, 1) } {
        2 => Ok(true),
        1 | 3 => Ok(false),
        _ => Err(failure("cannot compare Windows path components")),
    }
}

#[cfg(not(windows))]
pub(crate) fn same_windows_ordinal_case(
    _left: &std::ffi::OsStr,
    _right: &std::ffi::OsStr,
) -> Result<bool> {
    Err(failure("Windows path comparison requires Windows"))
}

#[cfg(windows)]
pub(crate) fn windows_path_starts_with(path: &Path, prefix: &Path) -> Result<bool> {
    let mut path_components = path.components();
    for expected in prefix.components() {
        let Some(actual) = path_components.next() else {
            return Ok(false);
        };
        if !same_windows_ordinal_case(actual.as_os_str(), expected.as_os_str())? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(not(windows))]
pub(crate) fn windows_path_starts_with(path: &Path, prefix: &Path) -> Result<bool> {
    Ok(path.starts_with(prefix))
}

/// The empty environment preserves the existing local hosted fixture. Vendor
/// subscription adapters must supply their exact account environment.
fn validate_hosted_payload_environment(
    profile: &ExecutionProfile,
    binding: &ProfileBinding,
    spec: &OwnedLaunchSpec,
    worktree: &Path,
) -> Result<()> {
    let environment: &BTreeMap<String, String> = &spec.environment;
    if environment.is_empty() {
        if profile.harness_id == "codex"
            || profile.adapter_digest == Digest::of_bytes(CODEX_ADAPTER_ID.as_bytes())
            || binding.auth_owner == "Codex"
            || binding.endpoint_identity == CODEX_SUBSCRIPTION_ENDPOINT
            || profile.harness_id == "claude"
            || profile.adapter_digest == Digest::of_bytes(CLAUDE_ADAPTER_ID.as_bytes())
            || binding.auth_owner == "Claude"
            || binding.endpoint_identity == CLAUDE_SUBSCRIPTION_ENDPOINT
        {
            return Err(failure(
                "subscription account home is missing from the hosted payload",
            ));
        }
        return Ok(());
    }
    if profile.harness_id == "claude"
        || profile.adapter_digest == Digest::of_bytes(CLAUDE_ADAPTER_ID.as_bytes())
        || binding.auth_owner == "Claude"
        || binding.endpoint_identity == CLAUDE_SUBSCRIPTION_ENDPOINT
    {
        return validate_claude_subscription_environment(profile, binding, spec, worktree);
    }
    if environment.len() != 3
        || profile.harness_id != "codex"
        || profile.adapter_contract_version != "1"
        || profile.adapter_digest != Digest::of_bytes(CODEX_ADAPTER_ID.as_bytes())
        || profile.backend != ExecutionBackend::Pty
        || spec.transport != OwnedTransport::Pty
        || binding.profile_digest
            != profile
                .digest()
                .map_err(|error| failure(&error.to_string()))?
        || binding.billing_mode != BillingSourceMode::Subscription
        || binding.credential_reference.is_some()
        || binding.auth_owner != "Codex"
        || binding.endpoint_identity != CODEX_SUBSCRIPTION_ENDPOINT
        || binding.trust_class != "vendor"
    {
        return Err(failure("unsupported hosted payload environment policy"));
    }
    if !spec.dependencies.is_empty()
        || !codex_exec_arguments(&profile.requested_model)
            .is_ok_and(|expected| expected.as_slice() == spec.arguments.as_slice())
    {
        return Err(failure(
            "Codex hosted command differs from the reviewed adapter",
        ));
    }
    let home = environment
        .get("CODEX_HOME")
        .ok_or_else(|| failure("Codex hosted payload environment is incomplete"))?;
    let system_root = environment
        .get("SystemRoot")
        .ok_or_else(|| failure("Codex hosted payload environment is incomplete"))?;
    let windir = environment
        .get("WINDIR")
        .ok_or_else(|| failure("Codex hosted payload environment is incomplete"))?;
    let observed_root = observed_windows_root()?;
    let windows_roots_match = same_windows_ordinal_case(
        std::ffi::OsStr::new(system_root),
        std::ffi::OsStr::new(&observed_root),
    )? && same_windows_ordinal_case(
        std::ffi::OsStr::new(windir),
        std::ffi::OsStr::new(&observed_root),
    )?;
    if !windows_roots_match {
        return Err(failure(
            "hosted payload Windows root differs from the operating system",
        ));
    }
    let home = Path::new(home);
    if !home.is_absolute() || !home.is_dir() || !worktree.is_absolute() || !worktree.is_dir() {
        return Err(failure(
            "Codex account home or attempt worktree is not an absolute directory",
        ));
    }
    let canonical_home = std::fs::canonicalize(home)
        .map_err(|_| failure("Codex account home cannot be resolved"))?;
    let canonical_worktree = std::fs::canonicalize(worktree)
        .map_err(|_| failure("attempt worktree cannot be resolved"))?;
    if windows_path_starts_with(&canonical_home, &canonical_worktree)?
        || windows_path_starts_with(&canonical_worktree, &canonical_home)?
    {
        return Err(failure(
            "Codex account home overlaps the writable attempt worktree",
        ));
    }
    Ok(())
}

fn validate_claude_subscription_environment(
    profile: &ExecutionProfile,
    binding: &ProfileBinding,
    spec: &OwnedLaunchSpec,
    worktree: &Path,
) -> Result<()> {
    let environment = &spec.environment;
    let claims = spec
        .arguments
        .iter()
        .filter_map(|argument| {
            argument
                .strip_prefix("Edit(./")
                .and_then(|argument| argument.strip_suffix(')'))
        })
        .collect::<Vec<_>>();
    let is_proposal = profile.adapter_digest
        == Digest::of_bytes(CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes())
        && profile.skill_tool_bundle_digest
            == Digest::of_bytes(CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes());
    let command_matches_profile = if is_proposal {
        claims.is_empty()
            && claude_proposal_arguments(&profile.requested_model, &["claim.txt".into()])
                .is_ok_and(|expected| expected == spec.arguments)
    } else {
        claims.as_slice().first().is_some_and(|claim| {
            claims.len() == 1
                && claude_edit_arguments(&profile.requested_model, &[(*claim).to_owned()])
                    .is_ok_and(|expected| expected == spec.arguments)
        })
    };
    if environment.len() != 6
        || profile.harness_id != "claude"
        || profile.adapter_contract_version != "1"
        || !(is_proposal
            || (profile.adapter_digest == Digest::of_bytes(CLAUDE_ADAPTER_ID.as_bytes())
                && profile.skill_tool_bundle_digest
                    == Digest::of_bytes(CLAUDE_TOOL_BUNDLE_ID.as_bytes())))
        || profile.capabilities != BTreeSet::from(["read".into(), "edit".into()])
        || profile.backend != ExecutionBackend::Subprocess
        || spec.transport != OwnedTransport::Subprocess
        || binding.profile_digest
            != profile
                .digest()
                .map_err(|error| failure(&error.to_string()))?
        || binding.billing_mode != BillingSourceMode::Subscription
        || binding.credential_reference.is_some()
        || binding.auth_owner != "Claude"
        || binding.endpoint_identity != CLAUDE_SUBSCRIPTION_ENDPOINT
        || binding.trust_class != "vendor"
        || binding.capacity_pool_ids.len() != 1
        || !spec.dependencies.is_empty()
        || spec
            .executable
            .path
            .file_name()
            .is_none_or(|name| name != "claude.exe")
        || !command_matches_profile
    {
        return Err(failure("unsupported Claude subscription launch policy"));
    }
    let home = environment
        .get("HOME")
        .ok_or_else(|| failure("Claude account environment is incomplete"))?;
    let account = Path::new(home);
    if home.is_empty()
        || environment.get("USERPROFILE") != Some(home)
        || environment.get("APPDATA")
            != Some(
                &account
                    .join("AppData/Roaming")
                    .to_string_lossy()
                    .into_owned(),
            )
        || environment.get("LOCALAPPDATA")
            != Some(&account.join("AppData/Local").to_string_lossy().into_owned())
        || !account.is_absolute()
        || !account.is_dir()
        || !worktree.is_absolute()
        || !worktree.is_dir()
    {
        return Err(failure("Claude account home or attempt worktree changed"));
    }
    let observed_root = observed_windows_root()?;
    for key in ["SystemRoot", "WINDIR"] {
        let value = environment
            .get(key)
            .ok_or_else(|| failure("Claude Windows root is absent"))?;
        if !same_windows_ordinal_case(
            std::ffi::OsStr::new(value),
            std::ffi::OsStr::new(&observed_root),
        )? {
            return Err(failure(
                "Claude Windows root differs from the operating system",
            ));
        }
    }
    let canonical_home = std::fs::canonicalize(account)
        .map_err(|_| failure("Claude account home cannot be resolved"))?;
    let canonical_worktree = std::fs::canonicalize(worktree)
        .map_err(|_| failure("Claude worktree cannot be resolved"))?;
    reject_reparse_components(account)
        .map_err(|_| failure("Claude account home contains a reparse point"))?;
    if is_proposal
        && crate::routed_claude::claude_subscription_account_source_id(account)
            .map_err(|_| failure("Claude account source cannot be observed"))?
            != binding.billing_source_id
    {
        return Err(failure(
            "Claude proposal account home differs from reviewed subscription source",
        ));
    }
    reject_reparse_descendants(worktree)
        .map_err(|_| failure("Claude worktree contains a reparse point"))?;
    if windows_path_starts_with(&canonical_home, &canonical_worktree)?
        || windows_path_starts_with(&canonical_worktree, &canonical_home)?
    {
        return Err(failure(
            "Claude account home overlaps the writable attempt worktree",
        ));
    }
    Ok(())
}

fn validate_hosted_launch_shape(
    launch: &LaunchContract,
    executable: &ExecutableIdentity,
    spec: &OwnedLaunchSpec,
    worktree: &Path,
    profile: &ExecutionProfile,
    binding: &ProfileBinding,
) -> Result<()> {
    validate_hosted_payload_environment(profile, binding, spec, worktree)?;
    let transport = match spec.transport {
        OwnedTransport::Subprocess => LaunchTransport::HostSubprocess,
        OwnedTransport::Pty => LaunchTransport::HostPty,
    };
    let host = launch
        .host
        .as_ref()
        .ok_or_else(|| failure("qualified hosted launch has no pinned host"))?;
    if launch.schema_version != 1
        || launch.transport != transport
        || launch.stdin_delivery != StdinDelivery::PrivateHostPipe
        || launch.private_stdin_digest.as_ref() != Some(&Digest::of_bytes(&spec.stdin))
        || launch.output_protocol != "pytxo-attempt-host/1"
        || launch.argument_lowering != "rust-std-command-windows-structured-argv/v1"
        || launch.working_directory_policy != "reviewed_attempt_worktree_v1"
        || !pin_matches_identity(&spec.bootstrap_host, host)
        || !pin_matches_identity(&spec.executable, executable)
        || launch.dependencies.len() != spec.dependencies.len()
        || !launch
            .dependencies
            .iter()
            .zip(&spec.dependencies)
            .all(|(identity, pin)| pin_matches_identity(pin, identity))
        || spec.working_directory.as_path() != worktree
        || !launch.barrier_timeout_ms.is_some_and(|timeout| {
            timeout > 0 && u128::from(timeout) == spec.barrier_timeout.as_millis()
        })
        || u128::from(launch.execution_timeout_ms) != spec.execution_timeout.as_millis()
        || u128::from(launch.settlement_timeout_ms) != spec.settlement_timeout.as_millis()
        || u128::from(launch.output_limit_bytes) != spec.output_limit as u128
        || launch.arguments_digest
            != canonical_digest(&spec.arguments, 1).map_err(|error| failure(&error.to_string()))?
        || launch.environment_policy_digest
            != canonical_digest(&spec.environment, 1)
                .map_err(|error| failure(&error.to_string()))?
    {
        return Err(failure(
            "routed hosted launch shape changed after qualification",
        ));
    }
    for pin in std::iter::once(&spec.bootstrap_host)
        .chain(std::iter::once(&spec.executable))
        .chain(&spec.dependencies)
    {
        if PinnedFile::observe(pin.path.clone())? != *pin {
            return Err(failure(
                "routed hosted launch pin changed after qualification",
            ));
        }
    }
    Ok(())
}

struct WorkerIdentity<'a> {
    store: &'a PytxoStore,
    catalog: &'a Catalog,
    plan: &'a FlowPlan,
    repo_root: &'a Path,
    data_dir: &'a Path,
    worktree: &'a Path,
    inputs: &'a ReviewedInputManifest,
    scope: RoutingScope,
    task_id: TaskId,
    attempt_id: AttemptId,
    facts: RoutingFacts,
}

impl<'a> WorkerIdentity<'a> {
    fn from_direct(worker: &'a RoutedOwnedWorker<'_>) -> Self {
        Self {
            store: worker.store,
            catalog: worker.catalog,
            plan: worker.plan,
            repo_root: worker.repo_root,
            data_dir: worker.data_dir,
            worktree: worker.worktree,
            inputs: worker.inputs,
            scope: worker.scope.clone(),
            task_id: worker.task_id.clone(),
            attempt_id: worker.attempt_id.clone(),
            facts: worker.facts.clone(),
        }
    }

    fn from_hosted(worker: &'a RoutedHostedWorker<'_>) -> Self {
        Self {
            store: worker.store,
            catalog: worker.catalog,
            plan: worker.plan,
            repo_root: worker.repo_root,
            data_dir: worker.data_dir,
            worktree: worker.worktree,
            inputs: worker.inputs,
            scope: worker.scope.clone(),
            task_id: worker.task_id.clone(),
            attempt_id: worker.attempt_id.clone(),
            facts: worker.facts.clone(),
        }
    }
}

#[derive(Clone, Copy)]
enum WorkerLaunch<'a> {
    Direct(&'a DirectOwnedLaunchSpec),
    Hosted(&'a OwnedLaunchSpec),
}

struct WorkerCallbacks<'a, 'b> {
    worker: WorkerIdentity<'a>,
    launch: WorkerLaunch<'b>,
}

struct WorkerGuard<'a, 'b, 'c> {
    callbacks: &'a mut WorkerCallbacks<'b, 'c>,
    _gate: ActiveRunGate,
    registered_process: Option<OwnedProcess>,
}

impl WorkerCallbacks<'_, '_> {
    fn catalog_stop_requested(&self) -> Result<bool> {
        crate::routed_fixture::routed_stop_requested(
            self.worker.catalog,
            &self.worker.plan.draft_id,
            &self.worker.scope.run_id.0,
        )
    }

    /// Authorize and permit_create check the full launch and filesystem;
    /// Runner also validates the spec between them. Its cancellation polls
    /// need the current Stop/owner fence, but must not repeatedly hash the
    /// vendor CLI or outlast the suspended host's short release barrier.
    fn cancelled_at_barrier(&self, process: Option<&OwnedProcess>) -> Result<bool> {
        let worker = &self.worker;
        let active: ActiveRunState =
            serde_json::from_slice(&std::fs::read(worker.data_dir.join("active_run.json"))?)
                .map_err(|_| failure("routed active marker is invalid after registration"))?;
        let start = pytxo_runner::process_start_identity(std::process::id())?
            .ok_or_else(|| failure("routed supervisor identity disappeared"))?;
        if active.run_id != worker.scope.run_id.0
            || active.repo_root != worker.repo_root.to_string_lossy()
            || active.supervisor_pid != std::process::id()
            || active.supervisor_start_identity.as_deref() != Some(&start)
        {
            return Err(failure(
                "routed active marker owner changed after registration",
            ));
        }
        if self.catalog_stop_requested()?
            || ProcessRegistryFile::load(&registry_path(worker.data_dir))?
                .cancelled_runs
                .contains(&worker.scope.run_id.0)
        {
            return Ok(true);
        }
        let history = worker
            .store
            .routing_history(&worker.scope)?
            .ok_or_else(|| failure("routed mission disappeared after registration"))?;
        if history.cancelled || history.cancel_epoch != history.mission.authorization.cancel_epoch {
            return Ok(true);
        }
        let attempt = history
            .attempts
            .iter()
            .find(|attempt| attempt.attempt_id == worker.attempt_id)
            .ok_or_else(|| failure("routed attempt disappeared after registration"))?;
        let owner = worker
            .store
            .launch_ownership(&worker.attempt_id)?
            .ok_or_else(|| failure("routed launch owner disappeared after registration"))?;
        let reservation = worker
            .catalog
            .capacity_reservation(&attempt.capacity_reservation)?
            .ok_or_else(|| failure("routed capacity disappeared after registration"))?;
        let phase_matches = match process {
            Some(process) => {
                attempt.state == AttemptState::Running
                    && owner.phase == LaunchOwnershipPhase::Registered
                    && owner.job_name.as_deref() == Some(process.job_name.as_str())
                    && owner.pid == Some(process.pid)
                    && owner.start_identity == process.start_identity
            }
            None => {
                attempt.state == AttemptState::Launching
                    && owner.phase == LaunchOwnershipPhase::Prepared
            }
        };
        if attempt.scope != worker.scope
            || attempt.task_id != worker.task_id
            || !phase_matches
            || owner.request.scope != worker.scope
            || owner.request.task_id != worker.task_id
            || owner.request.attempt_id != worker.attempt_id
            || owner.request.reservation_id != attempt.capacity_reservation
            || reservation.state != CapacityReservationState::Bound
            || reservation.domain_id != worker.scope.domain_id.0
            || reservation.run_id != worker.scope.run_id.0
            || reservation.attempt_id != worker.attempt_id.0
            || reservation.launch_token.as_deref() != Some(owner.request.launch_token.as_str())
        {
            return Err(failure("routed attempt owner changed at launch barrier"));
        }
        Ok(false)
    }

    fn assert_authority(&self) -> Result<RoutedAttemptRecord> {
        let worker = &self.worker;
        let active_path = worker.data_dir.join("active_run.json");
        let active: ActiveRunState = serde_json::from_slice(&std::fs::read(&active_path)?)
            .map_err(|error| failure(&format!("routed active marker is invalid: {error}")))?;
        let current_start = pytxo_runner::process_start_identity(std::process::id())?
            .ok_or_else(|| failure("routed supervisor process has no start identity"))?;
        if active.run_id != worker.scope.run_id.0
            || active.repo_root != worker.repo_root.to_string_lossy()
            || active.supervisor_pid != std::process::id()
            || active.supervisor_start_identity.as_deref() != Some(&current_start)
        {
            return Err(failure("routed active marker owner changed"));
        }
        let registry = ProcessRegistryFile::load(&registry_path(worker.data_dir))?;
        if self.catalog_stop_requested()?
            || registry.cancelled_runs.contains(&worker.scope.run_id.0)
        {
            return Err(PytxoError::Cancelled(worker.scope.run_id.0.clone()));
        }
        let reviewed = load_reviewed_staged_mission(worker.store, worker.plan)
            .map_err(|error| failure(&error.to_string()))?;
        let registered = worker
            .store
            .load_registered_mission_with_recipe_integrity(&worker.scope)?;
        if reviewed != registered {
            return Err(failure(
                "routed reviewed mission changed before native launch",
            ));
        }
        let history = worker
            .store
            .routing_history(&worker.scope)?
            .ok_or_else(|| failure("routed attempt history disappeared"))?;
        if history.cancelled || history.cancel_epoch != reviewed.authorization.cancel_epoch {
            return Err(PytxoError::Cancelled(worker.scope.run_id.0.clone()));
        }
        let attempt = history
            .attempts
            .into_iter()
            .find(|attempt| attempt.attempt_id == worker.attempt_id)
            .ok_or_else(|| failure("routed attempt disappeared"))?;
        if attempt.scope != worker.scope || attempt.task_id != worker.task_id {
            return Err(failure("routed attempt scope changed"));
        }
        let reservation = worker
            .catalog
            .capacity_reservation(&attempt.capacity_reservation)?
            .ok_or_else(|| failure("routed host reservation disappeared"))?;
        let owner = worker
            .store
            .launch_ownership(&worker.attempt_id)?
            .ok_or_else(|| failure("routed launch owner disappeared"))?;
        if reservation.state != CapacityReservationState::Bound
            || reservation.domain_id != worker.scope.domain_id.0
            || reservation.run_id != worker.scope.run_id.0
            || reservation.attempt_id != worker.attempt_id.0
            || reservation.launch_token.as_deref() != Some(owner.request.launch_token.as_str())
            || owner.request.reservation_id != attempt.capacity_reservation
        {
            return Err(failure("routed host reservation or launch token changed"));
        }
        let launch = attempt
            .selected
            .observation
            .launch
            .as_ref()
            .ok_or_else(|| failure("routed launch contract absent"))?;
        let observed = &attempt.selected.observation.executable;
        match self.launch {
            WorkerLaunch::Direct(spec) => {
                if !pin_matches_identity(&spec.executable, observed)
                    || PinnedFile::observe(spec.executable.path.clone())? != spec.executable
                {
                    return Err(failure("routed selected executable pin changed"));
                }
                validate_direct_launch_shape(launch, spec, worker.worktree)?;
            }
            WorkerLaunch::Hosted(spec) => {
                if attempt.selected.profile.harness_id == "claude" {
                    let reviewed_task = reviewed
                        .tasks
                        .iter()
                        .find(|task| task.contract.task_id == attempt.task_id)
                        .ok_or_else(|| failure("reviewed Claude task disappeared"))?;
                    let expected = if attempt.selected.profile.adapter_digest
                        == Digest::of_bytes(CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes())
                    {
                        claude_proposal_arguments(
                            &attempt.selected.profile.requested_model,
                            &reviewed_task.contract.claim_roots,
                        )
                    } else {
                        claude_edit_arguments(
                            &attempt.selected.profile.requested_model,
                            &reviewed_task.contract.claim_roots,
                        )
                    };
                    if !expected.is_ok_and(|arguments| arguments == spec.arguments) {
                        return Err(failure("Claude command differs from reviewed claim roots"));
                    }
                }
                validate_hosted_launch_shape(
                    launch,
                    observed,
                    spec,
                    worker.worktree,
                    &attempt.selected.profile,
                    &attempt.selected.binding,
                )?;
            }
        }
        worker.store.read_routing_attempt_handoff(&attempt)?;
        match inherited_snapshot_for_attempt(worker.store, &attempt)? {
            Some(inherited) => verify_materialized_dependency_output(
                worker.repo_root,
                worker.worktree,
                worker.inputs,
                &inherited,
            )?,
            None => verify_materialized_reviewed_inputs(
                worker.repo_root,
                worker.worktree,
                worker.inputs,
            )?,
        }
        Ok(attempt)
    }

    fn transition(
        &self,
        current: &RoutedAttemptRecord,
        to: AttemptState,
        receipts: RoutingReceipts,
    ) -> Result<RoutedAttemptRecord> {
        let worker = &self.worker;
        let history = worker
            .store
            .routing_history(&worker.scope)?
            .ok_or_else(|| failure("routed task history disappeared"))?;
        let task = history
            .tasks
            .iter()
            .find(|task| task.registration.contract.task_id == worker.task_id)
            .ok_or_else(|| failure("routed task disappeared"))?;
        let mut facts = worker.facts.clone();
        facts.now_ms = now_ms()?;
        worker
            .store
            .transition_routing_attempt(&TransitionRoutingAttempt {
                scope: worker.scope.clone(),
                event_id: format!(
                    "{}:{}:{}",
                    worker.attempt_id.0,
                    current.revision,
                    to.as_str()
                ),
                attempt_id: worker.attempt_id.clone(),
                expected_attempt_revision: current.revision,
                expected_task_revision: task.revision,
                to,
                facts,
                receipts,
                failure: None,
            })
    }

    fn claim(&self, name: &str) -> Result<PrivateArtifactClaim> {
        let worker = &self.worker;
        let owner = worker
            .store
            .launch_ownership(&worker.attempt_id)?
            .ok_or_else(|| failure("owned worker launch row is absent"))?;
        Ok(PrivateArtifactClaim {
            scope: worker.scope.clone(),
            task_id: worker.task_id.clone(),
            attempt_id: worker.attempt_id.clone(),
            reservation_id: owner.request.reservation_id,
            kind: PrivateArtifactKind::ControllerReceipt,
            artifact_id: format!("{}:{name}", worker.attempt_id.0),
            event_id: format!("{}:{name}:observed", worker.attempt_id.0),
        })
    }
}

impl LaunchCallbacks for WorkerCallbacks<'_, '_> {
    fn authorize(&mut self, _intent: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
        let active_path = self.worker.data_dir.join("active_run.json");
        let gate =
            ActiveRunGate::acquire(&active_path).map_err(|error| failure(&error.to_string()))?;
        let attempt = self.assert_authority()?;
        if attempt.state != AttemptState::Preparing {
            return Err(failure("owned worker is not in Preparing state"));
        }
        if let WorkerLaunch::Hosted(spec) = self.launch {
            if attempt.selected.profile.adapter_digest
                == Digest::of_bytes(CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes())
            {
                let expected = crate::routed_prompt::render_admitted_claude_proposal_prompt(
                    self.worker.store,
                    &self.worker.scope,
                    &self.worker.attempt_id,
                    self.worker.repo_root,
                    self.worker.inputs,
                )
                .map_err(|_| failure("Claude proposal source differs from reviewed input"))?;
                if spec.stdin != expected {
                    return Err(failure(
                        "Claude proposal stdin differs from reviewed source",
                    ));
                }
            } else {
                crate::routed_prompt::verify_admitted_attempt_stdin(
                    self.worker.store,
                    &self.worker.scope,
                    &self.worker.attempt_id,
                    &spec.stdin,
                )
                .map_err(|_| failure("routed hosted stdin differs from the reviewed attempt"))?;
            }
        }
        let owner = self
            .worker
            .store
            .launch_ownership(&self.worker.attempt_id)?
            .ok_or_else(|| failure("owned worker launch row is absent"))?;
        if owner.phase != LaunchOwnershipPhase::Prepared {
            return Err(failure("owned worker launch token is not fresh"));
        }
        let history = self
            .worker
            .store
            .routing_history(&self.worker.scope)?
            .ok_or_else(|| failure("routed mission disappeared"))?;
        let qualification = attempt
            .selected
            .observation
            .qualification
            .as_ref()
            .ok_or_else(|| failure("selected routed adapter lost qualification"))?;
        let claim = self.claim("launch-checks")?;
        let receipt = ControllerReceiptEnvelope {
            schema_version: 1,
            scope: self.worker.scope.clone(),
            task_id: self.worker.task_id.clone(),
            attempt_id: self.worker.attempt_id.clone(),
            reservation_id: owner.request.reservation_id.clone(),
            observed_at_ms: now_ms()?,
            evidence_id: claim.event_id.clone(),
            source: ReceiptSource::TrustedController,
            observation: ControllerObservation::LaunchChecks {
                mission_digest: canonical_digest(&history.mission, 1)
                    .map_err(|error| failure(&error.to_string()))?,
                qualification_digest: qualification
                    .digest()
                    .map_err(|error| failure(&error.to_string()))?,
                launch_fingerprint: attempt.launch_fingerprint.clone(),
                launch_token: owner.request.launch_token.clone(),
            },
        };
        let reference = self
            .worker
            .store
            .put_controller_receipt(&claim, &receipt)
            .map_err(|error| failure(&error.to_string()))?;
        self.transition(
            &attempt,
            AttemptState::Launching,
            RoutingReceipts {
                launch_checks: Some(reference.digest),
                ..Default::default()
            },
        )?;
        Ok(Box::new(WorkerGuard {
            callbacks: self,
            _gate: gate,
            registered_process: None,
        }))
    }

    fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()> {
        if receipt.active_processes != Some(0) {
            return Ok(());
        }
        let owner = self
            .worker
            .store
            .launch_ownership(&self.worker.attempt_id)?
            .ok_or_else(|| failure("owned worker disappeared before settlement"))?;
        if owner.phase == LaunchOwnershipPhase::Prepared
            && !receipt.process_registered
            && receipt.bootstrap.is_none()
        {
            // The runner returned before the create hook. The supervisor's
            // existing Prepared no-launch path closes this owner durably.
            return Ok(());
        }
        if owner.phase != LaunchOwnershipPhase::Registered {
            return Err(failure("owned worker is not registered for Job settlement"));
        }
        let root = receipt
            .bootstrap
            .as_ref()
            .ok_or_else(|| failure("owned worker root identity is absent"))?;
        if owner.pid != Some(root.pid)
            || owner.start_identity != root.start_identity
            || owner.job_name.as_deref() != Some(root.job_name.as_str())
            || owner.launch_nonce.as_deref() != Some(receipt.intent.launch_nonce.as_str())
        {
            return Err(failure(
                "owned worker native root differs from durable registration",
            ));
        }
        let claim = self.claim("native-job-zero")?;
        let envelope = ControllerReceiptEnvelope {
            schema_version: 1,
            scope: self.worker.scope.clone(),
            task_id: self.worker.task_id.clone(),
            attempt_id: self.worker.attempt_id.clone(),
            reservation_id: owner.request.reservation_id,
            observed_at_ms: now_ms()?,
            evidence_id: claim.event_id.clone(),
            source: ReceiptSource::OwnedJobObservation,
            observation: ControllerObservation::NativeJobZero {
                job_name: receipt.intent.job_name.clone(),
                launch_nonce: receipt.intent.launch_nonce.clone(),
                active_processes: 0,
            },
        };
        let reference = self
            .worker
            .store
            .put_controller_receipt(&claim, &envelope)
            .map_err(|error| failure(&error.to_string()))?;
        self.worker
            .store
            .settle_launch_ownership(&LaunchSettlement {
                scope: self.worker.scope.clone(),
                attempt_id: self.worker.attempt_id.clone(),
                event_id: format!("{}:native-job-settled", self.worker.attempt_id.0),
                receipt_claim: claim,
                receipt_ref: reference,
            })?;
        Ok(())
    }
}

impl LaunchGuard for WorkerGuard<'_, '_, '_> {
    fn permit_create(&mut self, intent: &LaunchIntent) -> Result<()> {
        let callbacks = &mut self.callbacks;
        // Authorize compared exact reviewed stdin. The owned host may prepare
        // its payload afterward, so recheck hosted account, worktree links and
        // launch authority at native create under the same Stop gate.
        if matches!(callbacks.launch, WorkerLaunch::Hosted(_)) {
            callbacks.assert_authority()?;
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_STRONG_CREATE").is_some() {
            let history = callbacks
                .worker
                .store
                .routing_history(&callbacks.worker.scope)?;
            if history.as_ref().is_some_and(|history| {
                history.attempts.iter().any(|attempt| {
                    attempt.attempt_id == callbacks.worker.attempt_id && attempt.ordinal == 2
                })
            }) {
                let saved = callbacks
                    .worker
                    .catalog
                    .get_flow_draft(&callbacks.worker.plan.draft_id)?
                    .ok_or_else(|| failure("routed Stop fixture lost reviewed Flow"))?;
                let exact_plan = saved
                    .plan_json
                    .ok_or_else(|| failure("routed Stop fixture lost saved plan"))?;
                if !callbacks.worker.catalog.request_routed_flow_stop(
                    &callbacks.worker.plan.draft_id,
                    &exact_plan,
                    &callbacks.worker.scope.run_id.0,
                )? {
                    return Err(failure("routed Stop fixture could not stage exact Stop"));
                }
            }
        }
        if callbacks.catalog_stop_requested()? {
            return Err(PytxoError::Cancelled(
                callbacks.worker.scope.run_id.0.clone(),
            ));
        }
        let create = callbacks
            .worker
            .store
            .authorize_launch_create(&LaunchCreateRequest {
                scope: callbacks.worker.scope.clone(),
                attempt_id: callbacks.worker.attempt_id.clone(),
                event_id: format!("{}:native-create", callbacks.worker.attempt_id.0),
                job_name: intent.job_name.clone(),
                launch_nonce: intent.launch_nonce.clone(),
            })?;
        if !matches!(create, LaunchCreateOutcome::Fresh(_)) {
            return Err(failure("owned worker native-create token was already used"));
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_EXPIRE_AFTER_CREATE").is_some() {
            std::thread::sleep(std::time::Duration::from_secs(31));
        }
        Ok(())
    }

    fn register(&mut self, process: &OwnedProcess) -> Result<()> {
        let callbacks = &mut self.callbacks;
        if process.job_name.is_empty() || process.pid == 0 {
            return Err(failure("native worker process identity is invalid"));
        }
        let start = process
            .start_identity
            .as_ref()
            .ok_or_else(|| failure("native worker process start identity is absent"))?;
        callbacks
            .worker
            .store
            .register_launch_process(&LaunchProcessRegistration {
                scope: callbacks.worker.scope.clone(),
                attempt_id: callbacks.worker.attempt_id.clone(),
                event_id: format!("{}:native-register", callbacks.worker.attempt_id.0),
                observed_job_name: process.job_name.clone(),
                pid: process.pid,
                start_identity: start.clone(),
            })?;
        ProcessRegistryFile::update(&registry_path(callbacks.worker.data_dir), |registry| {
            registry.push(ProcessEntry {
                run_id: callbacks.worker.scope.run_id.0.clone(),
                repo_root: callbacks.worker.repo_root.to_string_lossy().into_owned(),
                agent_key: format!(
                    "{}:{}",
                    callbacks.worker.scope.run_id.0, callbacks.worker.attempt_id.0
                ),
                pid: process.pid,
                start_identity: Some(start.clone()),
                worktree_path: callbacks.worker.worktree.to_string_lossy().into_owned(),
                branch: String::new(),
            });
            Ok(())
        })?;
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_WORKER_REGISTERED").is_some() {
            return Err(failure("injected post-registration transition failure"));
        }
        let history = callbacks
            .worker
            .store
            .routing_history(&callbacks.worker.scope)?
            .ok_or_else(|| failure("routed history disappeared after native register"))?;
        let attempt = history
            .attempts
            .into_iter()
            .find(|attempt| attempt.attempt_id == callbacks.worker.attempt_id)
            .ok_or_else(|| failure("routed attempt disappeared after native register"))?;
        let identity = canonical_digest(&(1_u32, &process.job_name, process.pid, start), 1)
            .map_err(|error| failure(&error.to_string()))?;
        callbacks.transition(
            &attempt,
            AttemptState::Running,
            RoutingReceipts {
                process_identity: Some(identity),
                ..Default::default()
            },
        )?;
        self.registered_process = Some(process.clone());
        Ok(())
    }

    fn cancelled(&mut self) -> Result<bool> {
        if matches!(self.callbacks.launch, WorkerLaunch::Hosted(_)) {
            return self
                .callbacks
                .cancelled_at_barrier(self.registered_process.as_ref());
        }
        match self.callbacks.assert_authority() {
            Ok(_) => Ok(false),
            Err(PytxoError::Cancelled(_)) => Ok(true),
            Err(error) => Err(error),
        }
    }
}

fn now_ms() -> Result<u64> {
    u64::try_from(chrono::Utc::now().timestamp_millis())
        .map_err(|error| failure(&format!("routed clock is invalid: {error}")))
}

fn failure(message: &str) -> PytxoError {
    PytxoError::Runner(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pytxo_core::routing::{BillingSourceId, BindingId, ModelIdentity, ProfileId};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;
    use std::time::Duration;

    fn codex_subscription() -> (ExecutionProfile, ProfileBinding) {
        let profile = ExecutionProfile {
            schema_version: 1,
            canonicalization_version: 1,
            id: ProfileId("codex-test".into()),
            revision: 1,
            harness_id: "codex".into(),
            adapter_contract_version: "1".into(),
            adapter_digest: Digest::of_bytes(CODEX_ADAPTER_ID.as_bytes()),
            requested_model: ModelIdentity {
                provider: "openai".into(),
                model: "test-model".into(),
                reasoning: None,
                revision: None,
            },
            skill_tool_bundle_digest: Digest::of_bytes(b"approved tools"),
            backend: ExecutionBackend::Pty,
            capabilities: BTreeSet::new(),
        };
        let binding = ProfileBinding {
            schema_version: 1,
            canonicalization_version: 1,
            id: BindingId("codex-subscription-test".into()),
            revision: 1,
            profile_digest: profile.digest().unwrap(),
            credential_reference: None,
            auth_owner: "Codex".into(),
            billing_source_id: BillingSourceId("account-test".into()),
            billing_mode: BillingSourceMode::Subscription,
            endpoint_identity: CODEX_SUBSCRIPTION_ENDPOINT.into(),
            trust_class: "vendor".into(),
            capacity_pool_ids: BTreeSet::new(),
        };
        (profile, binding)
    }

    fn generic_hosted_fixture() -> (ExecutionProfile, ProfileBinding) {
        let (mut profile, mut binding) = codex_subscription();
        profile.harness_id = "fixture-host".into();
        profile.adapter_digest = Digest::of_bytes(b"fixture-host");
        binding.profile_digest = profile.digest().unwrap();
        binding.auth_owner = "fixture".into();
        binding.endpoint_identity = "fixture:local".into();
        binding.billing_mode = BillingSourceMode::Local;
        (profile, binding)
    }

    #[test]
    fn direct_launch_rejects_changed_cwd_settings_argv_and_environment() {
        let worktree = PathBuf::from(r"C:\reviewed-attempt");
        let spec = DirectOwnedLaunchSpec {
            executable: pytxo_runner::owned_launch::PinnedFile {
                path: PathBuf::from(r"C:\Windows\System32\cmd.exe"),
                sha256: "pinned".into(),
            },
            arguments: vec!["/D".into(), "/C".into(), "echo ok".into()],
            windows_cmd_verbatim_tail: false,
            environment: BTreeMap::new(),
            working_directory: worktree.clone(),
            execution_timeout: Duration::from_secs(30),
            settlement_timeout: Duration::from_secs(5),
            output_limit: 4096,
        };
        let launch = LaunchContract {
            schema_version: 1,
            transport: LaunchTransport::DirectSubprocess,
            host: None,
            dependencies: vec![],
            arguments_digest: canonical_digest(&spec.arguments, 1).unwrap(),
            environment_policy_digest: canonical_digest(&spec.environment, 1).unwrap(),
            working_directory_policy: "reviewed_attempt_worktree_v1".into(),
            stdin_delivery: StdinDelivery::Closed,
            private_stdin_digest: None,
            output_protocol: "pytxo-direct-suspended/1".into(),
            argument_lowering: "windows-createprocess-structured-argv/v1".into(),
            barrier_timeout_ms: None,
            execution_timeout_ms: 30_000,
            settlement_timeout_ms: 5_000,
            output_limit_bytes: 4096,
        };
        validate_direct_launch_shape(&launch, &spec, &worktree).unwrap();
        let mut changed = spec.clone();
        changed.working_directory = PathBuf::from(r"C:\other");
        assert!(validate_direct_launch_shape(&launch, &changed, &worktree).is_err());
        changed = spec.clone();
        changed.execution_timeout += Duration::from_millis(1);
        assert!(validate_direct_launch_shape(&launch, &changed, &worktree).is_err());
        changed = spec.clone();
        changed.arguments.push("unreviewed".into());
        assert!(validate_direct_launch_shape(&launch, &changed, &worktree).is_err());
        changed = spec.clone();
        changed.windows_cmd_verbatim_tail = true;
        assert!(validate_direct_launch_shape(&launch, &changed, &worktree).is_err());
        changed = spec.clone();
        changed
            .environment
            .insert("UNREVIEWED".into(), "value".into());
        assert!(validate_direct_launch_shape(&launch, &changed, &worktree).is_err());
    }

    #[test]
    fn hosted_launch_binds_private_stdin_host_chain_and_exact_shape() {
        let (profile, binding) = generic_hosted_fixture();
        let temp = tempfile::tempdir().unwrap();
        let host_path = temp.path().join("pytxo-attempt-host.exe");
        let executable_path = temp.path().join("agent.exe");
        let dependency_path = temp.path().join("agent-runtime.exe");
        std::fs::write(&host_path, b"host-v1").unwrap();
        std::fs::write(&executable_path, b"agent-v1").unwrap();
        std::fs::write(&dependency_path, b"dependency-v1").unwrap();
        let host = PinnedFile::observe(host_path.clone()).unwrap();
        let executable = PinnedFile::observe(executable_path.clone()).unwrap();
        let dependency = PinnedFile::observe(dependency_path.clone()).unwrap();
        let identity = |pin: &PinnedFile| ExecutableIdentity {
            path: pin.path.to_string_lossy().into_owned(),
            version: "test-v1".into(),
            digest: Digest(pin.sha256.clone()),
        };
        let spec = OwnedLaunchSpec {
            bootstrap_host: host.clone(),
            executable: executable.clone(),
            dependencies: vec![dependency.clone()],
            arguments: vec!["--headless".into()],
            environment: BTreeMap::new(),
            working_directory: temp.path().to_path_buf(),
            stdin: b"private reviewed task\n".to_vec(),
            transport: OwnedTransport::Subprocess,
            barrier_timeout: Duration::from_secs(5),
            execution_timeout: Duration::from_secs(30),
            settlement_timeout: Duration::from_secs(5),
            output_limit: 4096,
        };
        let launch = LaunchContract {
            schema_version: 1,
            transport: LaunchTransport::HostSubprocess,
            host: Some(identity(&host)),
            dependencies: vec![identity(&dependency)],
            arguments_digest: canonical_digest(&spec.arguments, 1).unwrap(),
            environment_policy_digest: canonical_digest(&spec.environment, 1).unwrap(),
            working_directory_policy: "reviewed_attempt_worktree_v1".into(),
            stdin_delivery: StdinDelivery::PrivateHostPipe,
            private_stdin_digest: Some(Digest::of_bytes(&spec.stdin)),
            output_protocol: "pytxo-attempt-host/1".into(),
            argument_lowering: "rust-std-command-windows-structured-argv/v1".into(),
            barrier_timeout_ms: Some(5_000),
            execution_timeout_ms: 30_000,
            settlement_timeout_ms: 5_000,
            output_limit_bytes: 4096,
        };
        let check = |launch: &LaunchContract, spec: &OwnedLaunchSpec| {
            validate_hosted_launch_shape(
                launch,
                &identity(&executable),
                spec,
                temp.path(),
                &profile,
                &binding,
            )
        };
        check(&launch, &spec).unwrap();
        let mut pty_spec = spec.clone();
        pty_spec.transport = OwnedTransport::Pty;
        let mut pty_launch = launch.clone();
        pty_launch.transport = LaunchTransport::HostPty;
        check(&pty_launch, &pty_spec).unwrap();

        #[cfg(windows)]
        {
            let (profile, binding) = codex_subscription();
            let check = |launch: &LaunchContract, spec: &OwnedLaunchSpec| {
                validate_hosted_launch_shape(
                    launch,
                    &identity(&executable),
                    spec,
                    temp.path(),
                    &profile,
                    &binding,
                )
            };
            assert!(check(&pty_launch, &pty_spec).is_err());
            let account_home = tempfile::tempdir().unwrap();
            let windows_root = observed_windows_root().unwrap();
            let mut codex_spec = pty_spec.clone();
            codex_spec.dependencies.clear();
            codex_spec.arguments = vec![
                "exec".into(),
                "--ignore-user-config".into(),
                "--ephemeral".into(),
                "--sandbox".into(),
                "workspace-write".into(),
                "--model".into(),
                profile.requested_model.model.clone(),
                "-".into(),
            ];
            codex_spec.environment = BTreeMap::from([
                (
                    "CODEX_HOME".into(),
                    account_home.path().to_string_lossy().into_owned(),
                ),
                ("SystemRoot".into(), windows_root.clone()),
                ("WINDIR".into(), windows_root.clone()),
            ]);
            let with_digest = |spec: &OwnedLaunchSpec| {
                let mut qualified = pty_launch.clone();
                qualified.arguments_digest = canonical_digest(&spec.arguments, 1).unwrap();
                qualified.environment_policy_digest =
                    canonical_digest(&spec.environment, 1).unwrap();
                qualified.dependencies = spec.dependencies.iter().map(&identity).collect();
                qualified
            };
            check(&with_digest(&codex_spec), &codex_spec).unwrap();
            for arguments in [
                vec!["login".into(), "status".into()],
                vec![
                    "exec".into(),
                    "--ignore-user-config".into(),
                    "--ephemeral".into(),
                    "--sandbox".into(),
                    "danger-full-access".into(),
                    "--model".into(),
                    profile.requested_model.model.clone(),
                    "-".into(),
                ],
                vec![
                    "exec".into(),
                    "--ignore-user-config".into(),
                    "--ephemeral".into(),
                    "--sandbox".into(),
                    "workspace-write".into(),
                    "--model".into(),
                    "other-model".into(),
                    "-".into(),
                ],
                vec![
                    "exec".into(),
                    "--ignore-user-config".into(),
                    "--ephemeral".into(),
                    "--sandbox".into(),
                    "workspace-write".into(),
                    "--model".into(),
                    profile.requested_model.model.clone(),
                ],
            ] {
                let mut changed = codex_spec.clone();
                changed.arguments = arguments;
                assert!(check(&with_digest(&changed), &changed).is_err());
            }
            let mut changed = codex_spec.clone();
            changed.dependencies.push(dependency.clone());
            assert!(check(&with_digest(&changed), &changed).is_err());
            let (generic_profile, generic_binding) = generic_hosted_fixture();
            assert!(validate_hosted_launch_shape(
                &with_digest(&codex_spec),
                &identity(&executable),
                &codex_spec,
                temp.path(),
                &generic_profile,
                &generic_binding,
            )
            .is_err());

            let canonical_worktree = std::fs::canonicalize(temp.path()).unwrap();
            let original_name = canonical_worktree.file_name().unwrap().to_string_lossy();
            let alternate_name: String = original_name
                .chars()
                .map(|character| {
                    if character.is_ascii_lowercase() {
                        character.to_ascii_uppercase()
                    } else if character.is_ascii_uppercase() {
                        character.to_ascii_lowercase()
                    } else {
                        character
                    }
                })
                .collect();
            assert_ne!(original_name, alternate_name);
            let alternate_worktree = canonical_worktree.parent().unwrap().join(alternate_name);
            let recanonicalized = std::fs::canonicalize(&alternate_worktree).unwrap();
            println!(
                "Windows canonicalize normalizes path casing: {}",
                canonical_worktree == recanonicalized
            );
            assert!(windows_path_starts_with(&canonical_worktree, &alternate_worktree).unwrap());
            assert!(windows_path_starts_with(&canonical_worktree, &recanonicalized).unwrap());
            let mut changed = codex_spec.clone();
            changed.environment.insert(
                "CODEX_HOME".into(),
                alternate_worktree.to_string_lossy().into_owned(),
            );
            assert!(check(&with_digest(&changed), &changed).is_err());

            let mut changed = codex_spec.clone();
            changed
                .environment
                .insert("OPENAI_API_KEY".into(), "not-a-key".into());
            assert!(check(&with_digest(&changed), &changed).is_err());
            let mut changed = codex_spec.clone();
            changed.environment.remove("WINDIR");
            assert!(check(&with_digest(&changed), &changed).is_err());
            let mut changed = codex_spec.clone();
            changed.environment.remove("CODEX_HOME");
            changed
                .environment
                .insert("Codex_Home".into(), "other".into());
            assert!(check(&with_digest(&changed), &changed).is_err());
            for key in ["SystemRoot", "WINDIR"] {
                let mut changed = codex_spec.clone();
                changed
                    .environment
                    .insert(key.into(), r"C:\WrongWindows".into());
                assert!(check(&with_digest(&changed), &changed).is_err());
            }
            let mut changed = codex_spec.clone();
            changed.environment.insert(
                "CODEX_HOME".into(),
                temp.path().to_string_lossy().into_owned(),
            );
            assert!(check(&with_digest(&changed), &changed).is_err());
            let mut changed = codex_spec.clone();
            changed.environment.insert(
                "CODEX_HOME".into(),
                temp.path().parent().unwrap().to_string_lossy().into_owned(),
            );
            assert!(check(&with_digest(&changed), &changed).is_err());
            let nested_home = temp.path().join("account");
            std::fs::create_dir(&nested_home).unwrap();
            let mut changed = codex_spec.clone();
            changed.environment.insert(
                "CODEX_HOME".into(),
                nested_home.to_string_lossy().into_owned(),
            );
            assert!(check(&with_digest(&changed), &changed).is_err());
            let mut changed = binding.clone();
            changed.billing_mode = BillingSourceMode::Api;
            assert!(validate_hosted_launch_shape(
                &with_digest(&codex_spec),
                &identity(&executable),
                &codex_spec,
                temp.path(),
                &profile,
                &changed,
            )
            .is_err());
            let mut changed = binding.clone();
            changed.credential_reference = Some("unexpected-key".into());
            assert!(validate_hosted_launch_shape(
                &with_digest(&codex_spec),
                &identity(&executable),
                &codex_spec,
                temp.path(),
                &profile,
                &changed,
            )
            .is_err());
            let mut changed = profile.clone();
            changed.harness_id = "other".into();
            assert!(validate_hosted_launch_shape(
                &with_digest(&codex_spec),
                &identity(&executable),
                &codex_spec,
                temp.path(),
                &changed,
                &binding,
            )
            .is_err());
            let mut changed = codex_spec.clone();
            changed.environment.insert("WINDIR".into(), windows_root);
            changed.environment.insert("CODEX_HOME".into(), "".into());
            assert!(check(&with_digest(&changed), &changed).is_err());
            assert!(check(&pty_launch, &codex_spec).is_err());
        }

        let mut changed = spec.clone();
        changed.stdin.push(b'!');
        assert!(check(&launch, &changed).is_err());
        let mut changed = launch.clone();
        changed.private_stdin_digest = None;
        assert!(check(&changed, &spec).is_err());
        let mut changed = spec.clone();
        changed.bootstrap_host = executable.clone();
        assert!(check(&launch, &changed).is_err());
        let mut changed = spec.clone();
        changed.executable = host.clone();
        assert!(check(&launch, &changed).is_err());
        let mut changed = spec.clone();
        changed.dependencies[0] = host.clone();
        assert!(check(&launch, &changed).is_err());
        let mut changed = launch.clone();
        changed.dependencies.clear();
        assert!(check(&changed, &spec).is_err());
        let mut changed = spec.clone();
        changed.arguments.push("--unreviewed".into());
        assert!(check(&launch, &changed).is_err());
        let mut changed = spec.clone();
        changed.environment.insert("TOKEN".into(), "private".into());
        assert!(check(&launch, &changed).is_err());
        let mut changed = spec.clone();
        changed.working_directory = host_path;
        assert!(check(&launch, &changed).is_err());
        let mut changed = spec.clone();
        changed.execution_timeout += Duration::from_millis(1);
        assert!(check(&launch, &changed).is_err());
        let mut changed = spec.clone();
        changed.barrier_timeout += Duration::from_millis(1);
        assert!(check(&launch, &changed).is_err());
        let mut changed = spec.clone();
        changed.settlement_timeout += Duration::from_millis(1);
        assert!(check(&launch, &changed).is_err());
        let mut changed = spec.clone();
        changed.output_limit += 1;
        assert!(check(&launch, &changed).is_err());
        let mut changed = launch.clone();
        changed.barrier_timeout_ms = None;
        assert!(check(&changed, &spec).is_err());
        let mut changed = launch.clone();
        changed.output_protocol = "other".into();
        assert!(check(&changed, &spec).is_err());
        let mut changed = launch.clone();
        changed.argument_lowering = "other".into();
        assert!(check(&changed, &spec).is_err());
        let mut changed = launch.clone();
        changed.environment_policy_digest = Digest::of_bytes(b"changed policy");
        assert!(check(&changed, &spec).is_err());
        assert!(check(&launch, &pty_spec).is_err());
        std::fs::write(dependency_path, b"changed dependency").unwrap();
        assert!(check(&launch, &spec).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn claude_subscription_native_gate_reconstructs_command_and_account_policy() {
        use crate::routed_claude::{
            claude_edit_arguments, CLAUDE_ADAPTER_ID, CLAUDE_SUBSCRIPTION_ENDPOINT,
            CLAUDE_TOOL_BUNDLE_ID,
        };

        let (mut profile, mut binding) = codex_subscription();
        profile.harness_id = "claude".into();
        profile.adapter_digest = Digest::of_bytes(CLAUDE_ADAPTER_ID.as_bytes());
        profile.skill_tool_bundle_digest = Digest::of_bytes(CLAUDE_TOOL_BUNDLE_ID.as_bytes());
        profile.backend = ExecutionBackend::Subprocess;
        profile.capabilities = BTreeSet::from(["read".into(), "edit".into()]);
        profile.requested_model = ModelIdentity {
            provider: "anthropic".into(),
            model: "haiku".into(),
            reasoning: None,
            revision: None,
        };
        binding.profile_digest = profile.digest().unwrap();
        binding.auth_owner = "Claude".into();
        binding.endpoint_identity = CLAUDE_SUBSCRIPTION_ENDPOINT.into();
        binding.capacity_pool_ids = BTreeSet::from(["claude-account".into()]);
        let temp = tempfile::tempdir().unwrap();
        let account = temp.path().join("account");
        let worktree = temp.path().join("worktree");
        std::fs::create_dir(&account).unwrap();
        std::fs::create_dir(&worktree).unwrap();
        let root = observed_windows_root().unwrap();
        let account_text = account.to_string_lossy().into_owned();
        let mut spec = OwnedLaunchSpec {
            bootstrap_host: PinnedFile {
                path: temp.path().join("host.exe"),
                sha256: "test".into(),
            },
            executable: PinnedFile {
                path: temp.path().join("claude.exe"),
                sha256: "test".into(),
            },
            dependencies: vec![],
            arguments: claude_edit_arguments(&profile.requested_model, &["result.txt".into()])
                .unwrap(),
            environment: BTreeMap::from([
                ("HOME".into(), account_text.clone()),
                ("USERPROFILE".into(), account_text.clone()),
                (
                    "APPDATA".into(),
                    account
                        .join("AppData/Roaming")
                        .to_string_lossy()
                        .into_owned(),
                ),
                (
                    "LOCALAPPDATA".into(),
                    account.join("AppData/Local").to_string_lossy().into_owned(),
                ),
                ("SystemRoot".into(), root.clone()),
                ("WINDIR".into(), root.clone()),
            ]),
            working_directory: worktree.clone(),
            stdin: b"private".to_vec(),
            transport: OwnedTransport::Subprocess,
            barrier_timeout: Duration::from_secs(10),
            execution_timeout: Duration::from_secs(180),
            settlement_timeout: Duration::from_secs(10),
            output_limit: 32 * 1024,
        };
        let check =
            |profile: &ExecutionProfile, binding: &ProfileBinding, spec: &OwnedLaunchSpec| {
                validate_hosted_payload_environment(profile, binding, spec, &worktree)
            };
        check(&profile, &binding, &spec).unwrap();
        std::fs::write(&spec.bootstrap_host.path, b"embedded host pin").unwrap();
        std::fs::write(&spec.executable.path, b"claude CLI pin").unwrap();
        let host_pin = PinnedFile::observe(spec.bootstrap_host.path.clone()).unwrap();
        let cli_pin = PinnedFile::observe(spec.executable.path.clone()).unwrap();
        let selected = pytxo_core::routing::RouteTarget {
            profile_id: profile.id.clone(),
            binding_id: binding.id.clone(),
        };
        let template = crate::routed_claude::build_claude_launch_template(
            crate::routed_claude::ClaudeTemplateInput {
                profile: &profile,
                binding: &binding,
                selected_target: &selected,
                account: crate::routed_claude::ClaudeAccountSource {
                    binding_id: &binding.id,
                    billing_source_id: &binding.billing_source_id,
                    auth_owner: &binding.auth_owner,
                    endpoint_identity: &binding.endpoint_identity,
                    capacity_pool_ids: &binding.capacity_pool_ids,
                    account_home: &account,
                },
                claude: &cli_pin,
                claude_version: "test-cli",
                embedded_host: &host_pin,
                embedded_host_version: "test-host",
                worktree: &worktree,
                claim_roots: &["result.txt".into()],
                system_root: Path::new(&root),
                private_prompt: b"private",
                permission_profile: pytxo_core::PermissionProfile::Orbit,
                barrier_timeout: Duration::from_secs(10),
                execution_timeout: Duration::from_secs(180),
                settlement_timeout: Duration::from_secs(10),
                output_limit: 32 * 1024,
            },
        )
        .unwrap();
        validate_hosted_launch_shape(
            &template.launch,
            &template.executable,
            &template.spec,
            &worktree,
            &profile,
            &binding,
        )
        .unwrap();
        let outside = temp.path().join("outside.txt");
        std::fs::write(&outside, b"outside").unwrap();
        let linked = worktree.join("linked.txt");
        if let Err(error) = std::os::windows::fs::symlink_file(&outside, &linked) {
            if error.kind() != std::io::ErrorKind::PermissionDenied {
                panic!("cannot create disposable worktree link: {error}");
            }
        } else {
            assert!(check(&profile, &binding, &spec).is_err());
            std::fs::remove_file(&linked).unwrap();
            check(&profile, &binding, &spec).unwrap();
        }
        let original = spec.clone();
        for arguments in [
            vec!["auth".into(), "status".into()],
            vec!["-p".into(), "--model".into(), "opus".into()],
            vec![
                "-p".into(),
                "--model".into(),
                "haiku".into(),
                "--permission-mode".into(),
                "acceptEdits".into(),
                "--allowedTools".into(),
                "Edit".into(),
                "Read".into(),
            ],
        ] {
            spec.arguments = arguments;
            assert!(check(&profile, &binding, &spec).is_err());
        }
        spec = original.clone();
        spec.arguments
            .retain(|argument| argument != "--disallowedTools" && argument != "mcp__*");
        assert!(check(&profile, &binding, &spec).is_err());
        spec = original.clone();
        spec.environment
            .insert("ANTHROPIC_API_KEY".into(), "secret".into());
        assert!(check(&profile, &binding, &spec).is_err());
        spec = original.clone();
        spec.environment
            .insert("HOME".into(), worktree.to_string_lossy().into_owned());
        assert!(check(&profile, &binding, &spec).is_err());
        spec = original.clone();
        spec.environment.remove("USERPROFILE");
        assert!(check(&profile, &binding, &spec).is_err());
        let mut api_binding = binding.clone();
        api_binding.billing_mode = BillingSourceMode::Api;
        assert!(check(&profile, &api_binding, &original).is_err());
    }
}
