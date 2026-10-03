//! Explicitly opted-in, local-only adapter qualification for the first routed
//! runtime slice. A probe result is observed through the owned Windows Job;
//! no caller-supplied readiness booleans or provider billing mode are accepted.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{bail, Context};
use pytxo_core::routing::{
    canonical_digest, qualification_fingerprint, AdapterQualification, AttemptFailureClass,
    AttemptId, AttemptState, BillingSourceMode, Digest, ExecutableIdentity, HandoffOrigin,
    HandoffRequirements, LaunchContract, LaunchTransport, MeteringSupport, ModelIdentityLevel,
    PortableHandoffManifest, ProfileObservation, Readiness, RouteSelection, RoutingMode,
    StdinDelivery,
};
#[cfg(feature = "routed-test-faults")]
use pytxo_core::routing::{
    AdviceChoice, AdviceDistribution, AdviceEnvelope, AdviceRequestId, AdviceUsageStatus,
};
use pytxo_core::{ExecutionBackend, PermissionProfile, PytxoConfig, TaskId};
use pytxo_planner::advisor::Advisor;
#[cfg(any(windows, feature = "routed-test-faults"))]
use pytxo_runner::owned_launch::run_owned_launch;
use pytxo_runner::owned_launch::{
    run_direct_owned_launch, DirectOwnedLaunchSpec, LaunchCallbacks, LaunchGuard, LaunchIntent,
    OwnedLaunchReceipt, OwnedOutcome, OwnedProcess, PinnedFile,
};
#[cfg(feature = "routed-test-faults")]
use pytxo_runner::owned_launch::{OwnedLaunchSpec, OwnedTransport};
use pytxo_runner::{
    capture_reviewed_inputs, materialize_dependency_output, materialize_reviewed_inputs,
    prepare_reviewed_worktree, registry_path, ProcessRegistryFile, ReviewedInputManifest,
};
use pytxo_store::capacity::{
    CapacityBindRequest, CapacityOwner, CapacityReservationRequest, CapacityResourceRequest,
};
use pytxo_store::routing::{
    AdmitRoutingAttempt, ObserveRoutingDecision, RegisteredProfile, RoutingFacts, RoutingMission,
    RoutingReceipts, RoutingScope, TransitionRoutingAttempt,
};
use pytxo_store::routing_capacity_intent::RoutingCapacityIntentRequest;
use pytxo_store::routing_launch::LaunchOwnershipRequest;
use pytxo_store::routing_private::{
    handoff_manifest_claim, PrivateArtifactClaim, PrivateArtifactKind,
};
use pytxo_store::{Catalog, PytxoStore, RoutedWorktreeInstance};

use crate::flow::FlowPlan;
use crate::routed_claude::ClaudeLaunchTemplate;
use crate::ActiveRunGate;

const FIXTURE_ADAPTER: &str = "pytxo-local-fixture-v1";
#[cfg(feature = "routed-test-faults")]
const POWERSHELL_FIXTURE_ADAPTER: &str = "pytxo-local-fixture-powershell-v1";
const PROBE_TIMEOUT: Duration = Duration::from_secs(8);

fn powershell_handoff_fixture_opted_in() -> bool {
    #[cfg(feature = "routed-test-faults")]
    {
        std::env::var("PYTXO_TEST_ROUTED_POWERSHELL_HANDOFF")
            .ok()
            .as_deref()
            == Some("1")
    }
    #[cfg(not(feature = "routed-test-faults"))]
    {
        false
    }
}

#[derive(Debug)]
pub(crate) struct RoutedPreflightStopped;

impl std::fmt::Display for RoutedPreflightStopped {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "reviewed routed startup was stopped before worker admission"
        )
    }
}

impl std::error::Error for RoutedPreflightStopped {}

pub(crate) fn routed_stop_requested(
    catalog: &Catalog,
    draft_id: &str,
    run_id: &str,
) -> pytxo_core::Result<bool> {
    catalog
        .routed_flow_stop_requested(draft_id, run_id)?
        .ok_or_else(|| pytxo_core::PytxoError::Store("reviewed routed Stop claim is absent".into()))
}

fn ensure_routed_probe_not_stopped(
    catalog: &Catalog,
    draft_id: &str,
    run_id: &str,
) -> anyhow::Result<()> {
    if routed_stop_requested(catalog, draft_id, run_id)? {
        return Err(RoutedPreflightStopped.into());
    }
    Ok(())
}

fn owned_probe_was_stopped(receipt: &OwnedLaunchReceipt) -> bool {
    receipt.outcome == OwnedOutcome::Cancelled
        && receipt.active_processes == Some(0)
        && receipt.error.is_none()
}

pub(crate) fn fixture_worker_timeout() -> Duration {
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_EXPIRE_BEFORE_CREATE").is_some() {
        return Duration::from_millis(1);
    }
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_SLOW_SIBLINGS").is_some() {
        return Duration::from_secs(60);
    }
    Duration::from_secs(30)
}

pub(crate) fn fixture_worker_arguments(
    goal: &str,
    profile_id: &str,
) -> anyhow::Result<Vec<String>> {
    #[cfg(feature = "routed-test-faults")]
    if hosted_fixture_opted_in() {
        return if goal == "pytxo-local-fixture-v1:write-result"
            && matches!(profile_id, "everyday" | "strong")
        {
            Ok(vec!["run".into()])
        } else {
            bail!("synthetic hosted fixture supports only one reviewed result task")
        };
    }
    #[cfg(feature = "routed-test-faults")]
    if powershell_handoff_fixture_opted_in() && profile_id == "strong" {
        return powershell_worker_arguments(goal);
    }
    match goal {
        "pytxo-local-fixture-v1:write-seed" | "pytxo-local-fixture-v1:write-result" => {
            let output = if goal.ends_with("write-seed") {
                "seed.txt"
            } else {
                "result.txt"
            };
            #[cfg(feature = "routed-test-faults")]
            if std::env::var_os("PYTXO_TEST_ROUTED_SLOW_SIBLINGS").is_some() {
                let ping =
                    std::path::PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(|| {
                        anyhow::anyhow!("native overlap fixture has no SystemRoot")
                    })?)
                    .join("System32")
                    .join("PING.EXE");
                if !ping.is_file() {
                    bail!("native overlap fixture has no system ping executable");
                }
                return Ok(vec![
                    "/D".into(),
                    "/Q".into(),
                    "/C".into(),
                    format!(
                        "{} -n 15 127.0.0.1 >NUL && echo pytxo-routed>{output}",
                        ping.display()
                    ),
                ]);
            }
            Ok(vec![
                "/D".into(),
                "/Q".into(),
                "/C".into(),
                "echo".into(),
                format!("pytxo-routed>{output}"),
            ])
        }
        "pytxo-local-fixture-v1:write-result-from-seed" => Ok(vec![
            "/D".into(),
            "/V:ON".into(),
            "/Q".into(),
            "/C".into(),
            "set /p seed=<seed.txt && if !seed!==pytxo-routed echo pytxo-routed>result.txt".into(),
        ]),
        "pytxo-local-fixture-v1:write-result-with-repair" => match profile_id {
            "everyday" => Ok(vec![
                "/D".into(),
                "/Q".into(),
                "/C".into(),
                "exit".into(),
                "/b".into(),
                "7".into(),
            ]),
            "strong" => fixture_worker_arguments("pytxo-local-fixture-v1:write-result", profile_id),
            _ => bail!("repair fixture requires an exact reviewed profile"),
        },
        _ => bail!("reviewed task is not a bounded local fixture action"),
    }
}

#[cfg(feature = "routed-test-faults")]
fn powershell_worker_arguments(goal: &str) -> anyhow::Result<Vec<String>> {
    let write = |output: &str| {
        format!(
            "[System.IO.File]::WriteAllText('{output}', 'pytxo-routed' + [Environment]::NewLine, [System.Text.Encoding]::ASCII)"
        )
    };
    let script = match goal {
        "pytxo-local-fixture-v1:write-seed" => write("seed.txt"),
        "pytxo-local-fixture-v1:write-result"
        | "pytxo-local-fixture-v1:write-result-with-repair" => write("result.txt"),
        "pytxo-local-fixture-v1:write-result-from-seed" => format!(
            "try {{ $seed = [System.Convert]::ToBase64String([System.IO.File]::ReadAllBytes('seed.txt')) }} catch {{ exit 7 }}; if ($seed -cne 'cHl0eG8tcm91dGVkDQo=') {{ exit 7 }}; {}",
            write("result.txt")
        ),
        _ => bail!("reviewed task is not a bounded PowerShell fixture action"),
    };
    Ok(vec![
        "-NoProfile".into(),
        "-NonInteractive".into(),
        "-Command".into(),
        script,
    ])
}

#[derive(Clone)]
pub(crate) struct QualifiedFixture {
    pub observation: ProfileObservation,
    pub executable: PinnedFile,
    pub hosted: Option<HostedFixture>,
}

#[derive(Clone)]
pub(crate) struct HostedFixture {
    pub bootstrap_host: PinnedFile,
    pub prompt: Vec<u8>,
    pub proposal_template: Option<ClaudeLaunchTemplate>,
}

#[derive(Clone)]
pub(crate) struct AdmittedFixture {
    pub draft_id: String,
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub agent_id: String,
    pub reservation_id: String,
    pub launch_token: String,
    pub worktree: PathBuf,
    pub inputs: ReviewedInputManifest,
    pub facts: RoutingFacts,
    pub selected: QualifiedFixture,
    pub worker_arguments: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoundedRouteKind {
    LocalFixture,
    ClaudeProposal,
}

/// One selected local fixture attempt. Every cross-database boundary keeps
/// the same IDs; any error after the intent is durable remains recovery-owned.
#[expect(
    clippy::too_many_arguments,
    reason = "keep reviewed authority and store boundaries explicit"
)]
pub(crate) fn admit_one_local_fixture(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    task_index: usize,
    observed_fixtures: Vec<QualifiedFixture>,
    advisor: Option<Arc<dyn Advisor>>,
    hosted_client: Option<Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>>,
) -> anyhow::Result<AdmittedFixture> {
    admit_one_bounded_route(
        store,
        catalog,
        mission,
        plan,
        cfg,
        repo_root,
        data_dir,
        task_index,
        observed_fixtures,
        advisor,
        hosted_client,
        BoundedRouteKind::LocalFixture,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep reviewed admission and native route kind explicit"
)]
pub(crate) fn admit_one_claude_proposal(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    observed: Vec<QualifiedFixture>,
    hosted_client: Option<Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>>,
) -> anyhow::Result<AdmittedFixture> {
    admit_one_bounded_route(
        store,
        catalog,
        mission,
        plan,
        cfg,
        repo_root,
        data_dir,
        0,
        observed,
        None,
        hosted_client,
        BoundedRouteKind::ClaudeProposal,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep reviewed admission and native route kind explicit"
)]
fn admit_one_bounded_route(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    task_index: usize,
    observed_fixtures: Vec<QualifiedFixture>,
    advisor: Option<Arc<dyn Advisor>>,
    hosted_client: Option<Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>>,
    kind: BoundedRouteKind,
) -> anyhow::Result<AdmittedFixture> {
    if (kind == BoundedRouteKind::LocalFixture && !fixture_opted_in())
        || (kind == BoundedRouteKind::ClaudeProposal && !claude_proposal_opted_in())
        || !(1..=2).contains(&mission.tasks.len())
        || mission.profiles.len() != 2
        || mission.authorization.permission_profile != PermissionProfile::Orbit
        || mission.authorization.minimum_model_identity != ModelIdentityLevel::Requested
        || mission.authorization.live_advice_authorized
        || !matches!(
            mission.policy.mode,
            RoutingMode::Rules | RoutingMode::Shadow
        )
    {
        bail!("reviewed mission is not eligible for the local fixture supervisor");
    }
    let task = mission
        .tasks
        .get(task_index)
        .context("local fixture task wave is outside the reviewed mission")?;
    let sibling_pair = crate::flow::routed_parallel_siblings(plan);
    let (goal, output, dependencies): (&str, &str, &[TaskId]) =
        if kind == BoundedRouteKind::ClaudeProposal {
            crate::flow::validate_one_task_claude_route_shape(plan, mission)?;
            let [claim] = task.contract.claim_roots.as_slice() else {
                bail!("Claude proposal needs one existing reviewed text claim");
            };
            (task.contract.goal.as_str(), claim.as_str(), &[])
        } else {
            match (mission.tasks.len(), task_index, sibling_pair) {
                (1, 0, _)
                    if task.contract.goal == "pytxo-local-fixture-v1:write-result-with-repair" =>
                {
                    (
                        "pytxo-local-fixture-v1:write-result-with-repair",
                        "result.txt",
                        &[],
                    )
                }
                (1, 0, _) => ("pytxo-local-fixture-v1:write-result", "result.txt", &[]),
                (2, 0, _) => ("pytxo-local-fixture-v1:write-seed", "seed.txt", &[]),
                (2, 1, true) => ("pytxo-local-fixture-v1:write-result", "result.txt", &[]),
                (2, 1, false) => (
                    "pytxo-local-fixture-v1:write-result-from-seed",
                    "result.txt",
                    std::slice::from_ref(&mission.tasks[0].contract.task_id),
                ),
                _ => bail!("local fixture has no reviewed wave at this index"),
            }
        };
    if kind == BoundedRouteKind::LocalFixture {
        let expected_check = format!("if exist {output} (exit /b 0) else (exit /b 1)");
        if task.contract.goal != goal
            || task.contract.claim_roots != [output]
            || task.contract.dependencies != dependencies
            || task.check_recipes.len() != 1
            || task.check_recipes[0].command != expected_check
        {
            bail!("reviewed task differs from the bounded local fixture action and check");
        }
    } else if task.check_recipes.len() != 1 || task.contract.dependencies != dependencies {
        bail!("Claude proposal experiment requires one frozen checker and no dependencies");
    }
    let scope = RoutingScope {
        domain_id: mission.authorization.domain_id.clone(),
        run_id: mission.authorization.run_id.clone(),
    };
    let gate = ActiveRunGate::acquire(&data_dir.join("active_run.json"))?;
    let registry_cancelled = ProcessRegistryFile::load(&registry_path(data_dir))?
        .cancelled_runs
        .contains(&scope.run_id.0);
    let mission_cancelled = store
        .routing_history(&scope)?
        .is_some_and(|history| history.cancelled);
    let catalog_stop_requested = routed_stop_requested(catalog, &plan.draft_id, &scope.run_id.0)?;
    if registry_cancelled || mission_cancelled || catalog_stop_requested {
        bail!(
            "routed local fixture was cancelled before admission (registry={registry_cancelled}, mission={mission_cancelled}, catalog={catalog_stop_requested})"
        );
    }
    let task_id = task.contract.task_id.clone();
    let history = store
        .routing_history(&scope)?
        .context("routed mission disappeared before admission")?;
    let current_task = history
        .tasks
        .iter()
        .find(|row| row.registration.contract.task_id == task_id)
        .context("reviewed routing task disappeared before admission")?;
    let ordinal = current_task.next_ordinal;
    let previous = if ordinal == 2 {
        let predecessor = history
            .attempts
            .iter()
            .find(|attempt| Some(&attempt.attempt_id) == current_task.current_attempt.as_ref())
            .context("routed repair predecessor disappeared")?;
        let failure = predecessor
            .failure
            .as_ref()
            .context("routed repair predecessor has no failure evidence")?;
        if mission.authorization.limits.max_attempts != 2
            || predecessor.ordinal != 1
            || predecessor.task_id != task_id
            || predecessor.state != AttemptState::Failed
            || !predecessor.ownership_released
            || failure.attempt_id != predecessor.attempt_id
            || failure.actionable_evidence_digest.is_none()
            || !matches!(
                failure.failure_class,
                AttemptFailureClass::Implementation | AttemptFailureClass::Check
            )
        {
            bail!("routed repair predecessor is not a settled actionable failure");
        }
        Some(failure.clone())
    } else if ordinal == 1 {
        None
    } else {
        bail!("routed task exceeded the reviewed attempt count");
    };
    if task.contract.permission_profile != PermissionProfile::Orbit {
        bail!("local fixture task is outside Orbit");
    }
    if plan.execution_backend != "subprocess" {
        bail!("direct local fixture requires reviewed subprocess backend");
    }
    crate::hypervisor::require_routed_worktree_isolation(cfg)?;
    if observed_fixtures.len() != mission.profiles.len() {
        bail!("local fixture pre-run qualification count changed");
    }
    let hosted = kind == BoundedRouteKind::LocalFixture && hosted_fixture_opted_in();
    if hosted && (mission.tasks.len() != 1 || task_index != 0 || !dependencies.is_empty()) {
        bail!("synthetic hosted fixture supports one first attempt without dependencies");
    }
    let attempt_id = AttemptId(format!("routed-{}", uuid::Uuid::new_v4()));
    let worktree = repo_root
        .join(&cfg.worktree_dir)
        .join(&scope.run_id.0)
        .join(&attempt_id.0);
    let inputs = capture_reviewed_inputs(repo_root, &task.contract.base)?;
    let observed_check = if let Some(predecessor_id) = current_task.current_attempt.as_ref() {
        let predecessor = history
            .attempts
            .iter()
            .find(|candidate| candidate.attempt_id == *predecessor_id)
            .context("routed repair predecessor disappeared before prompt binding")?;
        crate::routed_prompt::observed_failed_check(store, &scope, predecessor, &task.contract)?
    } else {
        None
    };
    let hosted_prompt = if hosted {
        Some(crate::routed_prompt::render_private_attempt_prompt(
            crate::routed_prompt::RoutedPromptInput {
                task: &task.contract,
                attempt_id: &attempt_id,
                ordinal,
                dependencies: &[],
                previous: previous.as_ref(),
                observed_check: observed_check.as_ref(),
            },
        )?)
    } else {
        None
    };
    let claude_prompt = if kind == BoundedRouteKind::ClaudeProposal {
        let source = pytxo_runner::read_reviewed_claim_text(repo_root, &inputs, output)?;
        Some(crate::routed_prompt::render_claude_proposal_payload(
            crate::routed_prompt::render_private_attempt_prompt(
                crate::routed_prompt::RoutedPromptInput {
                    task: &task.contract,
                    attempt_id: &attempt_id,
                    ordinal,
                    dependencies: &[],
                    previous: previous.as_ref(),
                    observed_check: observed_check.as_ref(),
                },
            )?,
            output,
            &source,
        )?)
    } else {
        None
    };
    let mut qualified = Vec::new();
    for (ordinal, (profile, mut fixture)) in
        mission.profiles.iter().zip(observed_fixtures).enumerate()
    {
        if fixture.observation.profile_digest != profile.profile.digest()?
            || fixture.observation.binding_digest != profile.binding.digest()?
            || fixture.observation.expires_at_ms
                <= u64::try_from(chrono::Utc::now().timestamp_millis())?
        {
            bail!("local fixture pre-run qualification identity or freshness changed");
        }
        if let Some(prompt) = claude_prompt.as_ref() {
            use crate::routed_claude::{
                build_claude_proposal_template, ClaudeAccountSource, ClaudeTemplateInput,
            };
            let prepared = fixture
                .hosted
                .as_mut()
                .context("Claude proposal has no hosted probe")?;
            let preflight = prepared
                .proposal_template
                .as_ref()
                .context("Claude proposal has no native preflight template")?;
            let account_home = preflight
                .spec
                .environment
                .get("HOME")
                .map(Path::new)
                .context("Claude preflight lost its account home")?;
            let system_root = preflight
                .spec
                .environment
                .get("SystemRoot")
                .map(Path::new)
                .context("Claude preflight lost its Windows root")?;
            let target = if profile.profile.id == mission.policy.everyday.profile_id {
                &mission.policy.everyday
            } else if profile.profile.id == mission.policy.strong.profile_id {
                &mission.policy.strong
            } else {
                bail!("Claude preflight profile is outside the reviewed route");
            };
            let actual = build_claude_proposal_template(ClaudeTemplateInput {
                profile: &profile.profile,
                binding: &profile.binding,
                selected_target: target,
                account: ClaudeAccountSource {
                    binding_id: &profile.binding.id,
                    billing_source_id: &profile.binding.billing_source_id,
                    auth_owner: &profile.binding.auth_owner,
                    endpoint_identity: &profile.binding.endpoint_identity,
                    capacity_pool_ids: &profile.binding.capacity_pool_ids,
                    account_home,
                },
                claude: &fixture.executable,
                claude_version: "pinned-sha256-v1",
                embedded_host: &prepared.bootstrap_host,
                embedded_host_version: "pinned-sha256-v1",
                worktree: &worktree,
                claim_roots: &task.contract.claim_roots,
                system_root,
                private_prompt: prompt,
                permission_profile: PermissionProfile::Orbit,
                barrier_timeout: preflight.spec.barrier_timeout,
                execution_timeout: preflight.spec.execution_timeout,
                settlement_timeout: preflight.spec.settlement_timeout,
                output_limit: preflight.spec.output_limit,
            })?;
            if actual.spec.arguments != preflight.spec.arguments
                || actual.spec.environment != preflight.spec.environment
                || actual.spec.executable != preflight.spec.executable
                || actual.spec.bootstrap_host != preflight.spec.bootstrap_host
                || actual.spec.transport != preflight.spec.transport
                || actual.spec.output_limit != preflight.spec.output_limit
                || actual.spec.execution_timeout != preflight.spec.execution_timeout
            {
                bail!("Claude admitted launch differs from its native preflight");
            }
            fixture.observation.launch = Some(actual.launch.clone());
            fixture
                .observation
                .qualification
                .as_mut()
                .context("Claude native qualification disappeared")?
                .launch_fingerprint = qualification_fingerprint(
                &profile.profile,
                &profile.binding,
                &actual.executable,
                &actual.launch,
            )?;
            prepared.prompt = prompt.clone();
            prepared.proposal_template = Some(actual);
        } else {
            match (hosted_prompt.as_ref(), fixture.hosted.as_mut()) {
                (Some(prompt), Some(prepared)) if prepared.prompt.is_empty() => {
                    let launch = fixture
                        .observation
                        .launch
                        .as_mut()
                        .context("synthetic hosted fixture launch is absent")?;
                    if launch.transport != LaunchTransport::HostSubprocess
                        || launch.private_stdin_digest
                            != Some(Digest::of_bytes(b"unbound-synthetic-hosted-prompt"))
                    {
                        bail!("synthetic hosted fixture was already bound or changed");
                    }
                    launch.private_stdin_digest = Some(Digest::of_bytes(prompt));
                    let qualification = fixture
                        .observation
                        .qualification
                        .as_mut()
                        .context("synthetic hosted fixture qualification is absent")?;
                    qualification.launch_fingerprint = qualification_fingerprint(
                        &profile.profile,
                        &profile.binding,
                        &fixture.observation.executable,
                        launch,
                    )?;
                    fixture.observation.auth_status = Readiness::Ready;
                    fixture.observation.dispatch_supported = true;
                    prepared.prompt = prompt.clone();
                }
                (None, None) => {}
                _ => bail!("synthetic hosted fixture mode changed before admission"),
            }
        }
        store.register_routing_qualification(
            &scope,
            &format!(
                "{}:{}:fixture-qualification:{ordinal}",
                scope.run_id.0, task.contract.task_id.0
            ),
            fixture
                .observation
                .qualification
                .as_ref()
                .context("fixture qualification missing")?,
        )?;
        qualified.push((profile, fixture));
        #[cfg(feature = "routed-test-faults")]
        if mission.policy.mode == RoutingMode::Shadow
            && mission.policy.advisor_recipient.as_deref()
                == Some(pytxo_planner::advisor::HOSTED_RECIPIENT)
            && ((ordinal == 0
                && std::env::var_os("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_FIRST_QUALIFICATION")
                    .is_some())
                || (ordinal == 1
                    && std::env::var_os(
                        "PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_SECOND_QUALIFICATION",
                    )
                    .is_some()))
        {
            panic!("injected crash after hosted Shadow qualification");
        }
    }
    let observed_at_ms = qualified
        .iter()
        .map(|(_, fixture)| fixture.observation.observed_at_ms)
        .min()
        .context("fixture observations are absent")?;
    let expires_at_ms = qualified
        .iter()
        .map(|(_, fixture)| fixture.observation.expires_at_ms)
        .min()
        .context("fixture observations are absent")?;
    let mut facts = RoutingFacts {
        now_ms: u64::try_from(chrono::Utc::now().timestamp_millis())?,
        observed_at_ms,
        expires_at_ms,
        base: task.contract.base.clone(),
        plan_digest: mission.authorization.plan_digest.clone(),
        permission_profile: PermissionProfile::Orbit,
        observations: qualified
            .iter()
            .map(|(_, fixture)| fixture.observation.clone())
            .collect(),
        manual_target: None,
        packet_digest: None,
        advice_request_id: None,
    };
    #[cfg(feature = "routed-test-faults")]
    let fake_shadow = std::env::var_os("PYTXO_TEST_ROUTED_SHADOW_FAKE_ADVICE").is_some();
    #[cfg(feature = "routed-test-faults")]
    if fake_shadow {
        // Test-only local data path. A hosted advisor must run before this
        // gate and requires separate workspace consent for the exact packet.
        let packet = crate::flow::reviewed_task_advisor_packet(&task.contract)?;
        facts.packet_digest = Some(packet.digest());
        facts.advice_request_id = Some(AdviceRequestId(format!(
            "{}:{}:fake-shadow",
            scope.run_id.0, task_id.0
        )));
    }
    #[cfg(feature = "routed-test-faults")]
    let snapshot = store.routing_snapshot(&scope, &task_id, &facts)?;
    #[cfg(feature = "routed-test-faults")]
    let mut advice_json = if fake_shadow {
        Some(serde_json::to_string(&AdviceEnvelope {
            schema_version: 1,
            request_id: facts
                .advice_request_id
                .clone()
                .context("test advice ID absent")?,
            packet_digest: facts
                .packet_digest
                .clone()
                .context("test packet digest absent")?,
            policy_version: mission.policy.version.clone(),
            template_version: mission.policy.advice_template.clone(),
            model_id: "test-fixture-shadow-v1".into(),
            task_revision: snapshot.task_revision,
            task_state_revision: snapshot.task_state_revision,
            authorization_revision: snapshot.authorization_revision,
            cancel_epoch: snapshot.cancel_epoch,
            consent_revision: mission.authorization.consent_revision,
            received_at_ms: facts.now_ms,
            expires_at_ms: facts.now_ms.saturating_add(60_000),
            packet_complete: true,
            choice: AdviceChoice::EverydayFit,
            distribution: AdviceDistribution {
                everyday_fit: 1.0,
                strong_needed: 0.0,
                unclear: 0.0,
            },
            usage_status: AdviceUsageStatus::Unknown,
            usage_receipt_id: None,
            elapsed_ms: 0,
        })?)
    } else {
        None
    };
    #[cfg(not(feature = "routed-test-faults"))]
    let mut advice_json: Option<String> = None;
    let local_advisor_present = advisor.is_some();
    if let Some(advisor) = advisor {
        #[cfg(feature = "routed-test-faults")]
        if fake_shadow {
            bail!("synthetic shadow and journal-backed advisor cannot share a decision");
        }
        let packet = crate::flow::reviewed_task_advisor_packet(&task.contract)?;
        if let Some((observed_facts, observed_response)) =
            crate::routed_advisor::observe_shadow_once(
                store, mission, &scope, &task_id, &facts, packet, advisor,
            )?
        {
            facts = observed_facts;
            advice_json = Some(observed_response);
        }
    }
    if let Some(client) = hosted_client {
        if local_advisor_present
            || mission.policy.advisor_recipient.as_deref()
                != Some(pytxo_planner::advisor::HOSTED_RECIPIENT)
            || mission.policy.mode != RoutingMode::Shadow
            || task_index != 0
            || ordinal != 1
        {
            bail!("hosted Shadow client is outside the reviewed initial decision");
        }
        #[cfg(feature = "routed-test-faults")]
        if fake_shadow {
            bail!("synthetic Shadow and hosted Shadow cannot share a decision");
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var("PYTXO_TEST_HOSTED_SHADOW_DISPATCH").as_deref() == Ok("1") {
            crate::flow::validated_claimed_hosted_advisor_packet(catalog, &plan.draft_id)
                .context("fake-client hosted claim is not valid before admission")?;
        }
        if let Some((observed_facts, observed_response)) =
            crate::routed_hosted_shadow::observe_hosted_shadow_once(
                catalog,
                &plan.draft_id,
                &scope,
                &task_id,
                &facts,
                client,
            )?
        {
            facts = observed_facts;
            advice_json = Some(observed_response);
        }
    }
    let snapshot = store.routing_snapshot(&scope, &task_id, &facts)?;
    let decision =
        store.preview_routing_decision(&scope, &task_id, &facts, advice_json.as_deref())?;
    let observation_event_id = format!(
        "{}:{}:{}:selected-observation",
        scope.run_id.0, task_id.0, snapshot.next_ordinal
    );
    let observed = crate::routed_advisor::observe_with_rules_fallback(
        store,
        ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: observation_event_id.clone(),
            task_id: task_id.clone(),
            expected_cancel_epoch: snapshot.cancel_epoch,
            expected_next_ordinal: snapshot.next_ordinal,
            facts: facts.clone(),
            decision: decision.clone(),
            advice_json: advice_json.clone(),
        },
    )?;
    let decision = observed.decision;
    let advice_json = observed.advice_json;
    let RouteSelection::Selected(target) = &decision.selection else {
        bail!("local fixture route is not eligible under deterministic Core filtering");
    };
    let worker_arguments = if kind == BoundedRouteKind::ClaudeProposal {
        crate::routed_claude::claude_proposal_arguments(
            &mission
                .profiles
                .iter()
                .find(|profile| profile.profile.id == target.profile_id)
                .context("selected Claude profile disappeared")?
                .profile
                .requested_model,
            &task.contract.claim_roots,
        )?
    } else {
        fixture_worker_arguments(goal, &target.profile_id.0)?
    };
    let selected_index = qualified
        .iter()
        .position(|(profile, _)| {
            profile.profile.id == target.profile_id && profile.binding.id == target.binding_id
        })
        .context("selected local fixture profile was not qualified")?;
    let agent_id = format!("agent-{}", uuid::Uuid::new_v4());
    let reservation_id = format!("capacity-{}", uuid::Uuid::new_v4());
    let launch_token = format!("launch-{}", uuid::Uuid::new_v4());
    let pool = &qualified[selected_index].0.binding.capacity_pool_ids;
    if pool.len() != 1 || task.contract.required_resources != *pool {
        bail!("selected local fixture has no exact reviewed host pool");
    }
    let pid = std::process::id();
    let start = pytxo_runner::process_start_identity(pid)?
        .context("routed supervisor has no native process identity")?;
    let now_ms = u64::try_from(chrono::Utc::now().timestamp_millis())?;
    let reservation = CapacityReservationRequest {
        reservation_id: reservation_id.clone(),
        domain_id: scope.domain_id.0.clone(),
        run_id: scope.run_id.0.clone(),
        attempt_id: attempt_id.0.clone(),
        owner: CapacityOwner {
            process_id: pid,
            process_start_identity: start,
        },
        resources: vec![CapacityResourceRequest {
            resource_id: pool
                .iter()
                .next()
                .context("fixture host pool vanished")?
                .clone(),
            units: 1,
        }],
        requested_at_ms: now_ms,
    };
    store.register_capacity_intent(
        catalog,
        &RoutingCapacityIntentRequest {
            scope: scope.clone(),
            task_id: task_id.clone(),
            reservation: reservation.clone(),
            event_id: format!("{}:intent", attempt_id.0),
        },
    )?;
    let input_ref = store.put_private_artifact(
        &PrivateArtifactClaim {
            scope: scope.clone(),
            task_id: task_id.clone(),
            attempt_id: attempt_id.clone(),
            reservation_id: reservation_id.clone(),
            kind: PrivateArtifactKind::InputManifest,
            artifact_id: format!("{}:inputs", attempt_id.0),
            event_id: format!("{}:inputs-retained", attempt_id.0),
        },
        &serde_json::to_vec(&inputs)?,
    )?;
    let handoff_ref = if snapshot.resolved_dependencies.is_empty() {
        None
    } else {
        let handoff = PortableHandoffManifest {
            schema_version: 1,
            canonicalization_version: 1,
            manifest_digest: Digest::of_bytes(b"unsealed handoff"),
            origin: HandoffOrigin {
                domain_id: scope.domain_id.clone(),
                run_id: scope.run_id.clone(),
                task_id: task_id.clone(),
                attempt_id: attempt_id.clone(),
                task_revision: task.contract.revision,
                plan_id: mission.authorization.plan_id.clone(),
                plan_digest: mission.authorization.plan_digest.clone(),
                authorization_revision: mission.authorization.revision,
            },
            task_contract_ref: task.contract.digest()?,
            base: task.contract.base.clone(),
            dependencies: snapshot.resolved_dependencies.clone(),
            repair_input: None,
            changes: vec![],
            evidence: vec![],
            requirements: HandoffRequirements {
                capabilities: task.contract.required_capabilities.clone(),
                skill_tool_bundle_digest: task.contract.skill_tool_bundle_digest.clone(),
            },
            notes: vec![],
        }
        .seal()?;
        handoff.validate(
            &task.contract,
            &handoff.origin,
            &snapshot.resolved_dependencies,
        )?;
        Some(store.put_private_artifact(
            &handoff_manifest_claim(&scope, &task_id, &attempt_id, &reservation_id),
            &serde_json::to_vec(&handoff)?,
        )?)
    };
    store.mark_capacity_reserve_may_have_started(
        &reservation_id,
        &format!("{}:reserve-start", attempt_id.0),
    )?;
    catalog.reserve_capacity(&reservation)?;
    let admitted = store.admit_routing_attempt(&AdmitRoutingAttempt {
        scope: scope.clone(),
        event_id: format!("{}:admit", attempt_id.0),
        task_id: task_id.clone(),
        attempt_id: attempt_id.clone(),
        agent_id: agent_id.clone(),
        facts: facts.clone(),
        decision,
        capacity_reservation: reservation_id.clone(),
        input_manifest: input_ref.clone(),
        handoff: handoff_ref,
        advice_json,
        observation_event_id: Some(observation_event_id),
    })?;
    let history = store
        .routing_history(&scope)?
        .context("admitted fixture disappeared")?;
    let task_revision = history
        .tasks
        .iter()
        .find(|row| row.registration.contract.task_id == task_id)
        .context("admitted fixture task disappeared")?
        .revision;
    let mut latest_facts = facts.clone();
    latest_facts.now_ms = u64::try_from(chrono::Utc::now().timestamp_millis())?;
    store.transition_routing_attempt(&TransitionRoutingAttempt {
        scope: scope.clone(),
        event_id: format!("{}:preparing", attempt_id.0),
        attempt_id: attempt_id.clone(),
        expected_attempt_revision: admitted.revision,
        expected_task_revision: task_revision,
        to: AttemptState::Preparing,
        facts: latest_facts.clone(),
        receipts: RoutingReceipts {
            inputs: Some(input_ref.digest.clone()),
            ..Default::default()
        },
        failure: None,
    })?;
    catalog.bind_capacity_reservation(&CapacityBindRequest {
        reservation_id: reservation_id.clone(),
        attempt_id: attempt_id.0.clone(),
        launch_token: launch_token.clone(),
        bound_at_ms: now_ms,
    })?;
    store.prepare_launch_ownership(
        catalog,
        &LaunchOwnershipRequest {
            scope: scope.clone(),
            task_id: task_id.clone(),
            attempt_id: attempt_id.clone(),
            reservation_id: reservation_id.clone(),
            launch_token: launch_token.clone(),
            event_id: format!("{}:owner-prepared", attempt_id.0),
        },
    )?;
    let selected = qualified.swap_remove(selected_index).1;
    drop(gate);
    Ok(AdmittedFixture {
        draft_id: plan.draft_id.clone(),
        scope,
        task_id,
        attempt_id,
        agent_id,
        reservation_id,
        launch_token,
        worktree,
        inputs,
        facts: latest_facts,
        selected,
        worker_arguments,
    })
}

/// Git preparation happens after the durable Prepared owner exists. The
/// supervisor can close that owner with an exact no-create receipt on failure.
pub(crate) fn prepare_admitted_worktree(
    store: &PytxoStore,
    repo_root: &Path,
    admitted: &AdmittedFixture,
) -> anyhow::Result<()> {
    prepare_reviewed_worktree(repo_root, &admitted.worktree, &admitted.inputs.base)?;
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_WORKTREE_PREPARED").is_some() {
        bail!("injected post-worktree preparation failure");
    }
    let attempt = store
        .routing_history(&admitted.scope)?
        .context("admitted fixture history disappeared")?
        .attempts
        .into_iter()
        .find(|attempt| attempt.attempt_id == admitted.attempt_id)
        .context("admitted fixture attempt disappeared")?;
    store.read_routing_attempt_handoff(&attempt)?;
    match crate::routed_worker::inherited_snapshot_for_attempt(store, &attempt)? {
        Some(inherited) => materialize_dependency_output(
            repo_root,
            &admitted.worktree,
            &admitted.inputs,
            &inherited,
        )?,
        None => materialize_reviewed_inputs(repo_root, &admitted.worktree, &admitted.inputs)?,
    }
    store.insert_agent_with_root(
        &admitted.agent_id,
        &admitted.scope.run_id.0,
        &admitted.task_id.0,
        1,
        Some(&admitted.worktree.to_string_lossy()),
        &attempt.selected.profile.harness_id,
        None,
    )?;
    store.record_routed_worktree_instance(
        &admitted.agent_id,
        &admitted.scope.run_id.0,
        &admitted.task_id.0,
        &RoutedWorktreeInstance {
            path: admitted.worktree.to_string_lossy().into_owned(),
            git_file_identity: pytxo_runner::file_identity(&admitted.worktree.join(".git"))?,
        },
    )?;
    Ok(())
}

pub(crate) fn fixture_opted_in() -> bool {
    std::env::var("PYTXO_EXPERIMENTAL_ROUTED_LOCAL_FIXTURE")
        .ok()
        .as_deref()
        == Some("1")
}

/// This switch is deliberately separate from the synthetic fixture. Dispatch
/// still requires a reviewed Claude subscription pair and fresh native probes.
pub(crate) fn claude_proposal_opted_in() -> bool {
    std::env::var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL")
        .ok()
        .as_deref()
        == Some("1")
}

/// A second subscription invocation must be visible in the reviewed mission
/// and separately enabled at dispatch. It is never inferred from the first
/// attempt's failure alone.
pub(crate) fn claude_repair_opted_in() -> bool {
    std::env::var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_REPAIR")
        .ok()
        .as_deref()
        == Some("1")
}

#[cfg(windows)]
pub(crate) fn validate_claude_probe_location() -> anyhow::Result<PathBuf> {
    let probe_root = std::env::var_os("PYTXO_ROUTED_CLAUDE_PROBE_ROOT")
        .map(PathBuf::from)
        .context("explicit Claude probe root outside the account home is absent")?;
    let account_home = std::env::var_os("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME")
        .map(PathBuf::from)
        .context("selected Claude subscription account home is absent")?;
    if !probe_root.is_absolute() || !probe_root.is_dir() || !account_home.is_dir() {
        bail!("Claude probe root and account home must be separate existing directories");
    }
    crate::routed_claude::reject_reparse_components(&probe_root)?;
    crate::routed_claude::reject_reparse_components(&account_home)?;
    let root = std::fs::canonicalize(&probe_root)?;
    let home = std::fs::canonicalize(&account_home)?;
    if crate::routed_worker::windows_path_starts_with(&root, &home)?
        || crate::routed_worker::windows_path_starts_with(&home, &root)?
    {
        bail!("Claude probe root overlaps the selected subscription account home");
    }
    Ok(root)
}

#[cfg(windows)]
pub(crate) struct ClaudeProposalProbeContext {
    probe_parent: PathBuf,
    cli: PinnedFile,
    embedded_host: PinnedFile,
    account_home: PathBuf,
    system_root: PathBuf,
}

#[cfg(windows)]
pub(crate) fn observe_claude_proposal_probe_context() -> anyhow::Result<ClaudeProposalProbeContext>
{
    let probe_parent = validate_claude_probe_location()?;
    let cli_path = std::env::var_os("PYTXO_ROUTED_CLAUDE_EXE")
        .map(PathBuf::from)
        .context("selected Claude subscription executable is absent")?;
    let account_home = std::env::var_os("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME")
        .map(PathBuf::from)
        .context("selected Claude subscription account home is absent")?;
    let system_root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .context("Windows system root is absent")?;
    let cli = PinnedFile::observe(cli_path)?;
    let host_path = std::env::current_exe()?;
    #[cfg(feature = "routed-test-faults")]
    let host_path = std::env::var_os("PYTXO_TEST_EMBEDDED_HOST")
        .map(PathBuf::from)
        .unwrap_or(host_path);
    let embedded_host = PinnedFile::observe(host_path)?;
    Ok(ClaudeProposalProbeContext {
        probe_parent,
        cli,
        embedded_host,
        account_home,
        system_root,
    })
}

#[cfg(windows)]
pub(crate) fn qualify_claude_proposal(
    profile: &RegisteredProfile,
    target: &pytxo_core::routing::RouteTarget,
    catalog: &Catalog,
    context: &ClaudeProposalProbeContext,
    draft_id: &str,
    run_id: &str,
) -> anyhow::Result<QualifiedFixture> {
    use crate::routed_claude::{
        build_claude_proposal_template, observe_owned_claude_subscription_auth,
        parse_owned_claude_one_file_proposal, ClaudeAccountSource, ClaudeAuthProbeInput,
        ClaudeTemplateInput, CLAUDE_SUBSCRIPTION_ENDPOINT,
    };

    if !claude_proposal_opted_in() {
        bail!("reviewed Claude proposal experiment is not enabled");
    }
    ensure_routed_probe_not_stopped(catalog, draft_id, run_id)?;
    let cli = &context.cli;
    let embedded_host = &context.embedded_host;
    let account_home = &context.account_home;
    let system_root = &context.system_root;
    let probe_dir = context
        .probe_parent
        .join(format!("pytxo-claude-probe-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&probe_dir)?;
    let source = probe_dir.join("result.txt");
    let sibling = probe_dir.join("sibling.txt");
    std::fs::write(&source, b"old\n")?;
    std::fs::write(&sibling, b"untouched\n")?;
    let cleanup_probe = || -> anyhow::Result<()> {
        std::fs::remove_file(&source)?;
        std::fs::remove_file(&sibling)?;
        std::fs::remove_dir(&probe_dir)?;
        Ok(())
    };
    let auth_result = observe_owned_claude_subscription_auth(
        ClaudeAuthProbeInput {
            claude: cli,
            embedded_host,
            account_home,
            system_root,
            probe_directory: &probe_dir,
        },
        &mut ProbeCallbacks {
            registered_at: None,
        },
        || routed_stop_requested(catalog, draft_id, run_id),
    );
    let auth = match auth_result {
        Ok(auth) => auth,
        Err(error) if error.is::<RoutedPreflightStopped>() => {
            cleanup_probe()?;
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    if routed_stop_requested(catalog, draft_id, run_id)? {
        cleanup_probe()?;
        return Err(RoutedPreflightStopped.into());
    }
    let prompt = crate::routed_prompt::render_claude_proposal_payload(
        b"Replace the old line with changed and preserve its trailing newline.".to_vec(),
        "result.txt",
        "old\n",
    )?;
    let template = build_claude_proposal_template(ClaudeTemplateInput {
        profile: &profile.profile,
        binding: &profile.binding,
        selected_target: target,
        account: ClaudeAccountSource {
            binding_id: &profile.binding.id,
            billing_source_id: &profile.binding.billing_source_id,
            auth_owner: &profile.binding.auth_owner,
            endpoint_identity: &profile.binding.endpoint_identity,
            capacity_pool_ids: &profile.binding.capacity_pool_ids,
            account_home,
        },
        claude: cli,
        claude_version: "pinned-sha256-v1",
        embedded_host,
        embedded_host_version: "pinned-sha256-v1",
        worktree: &probe_dir,
        claim_roots: &["result.txt".into()],
        system_root,
        private_prompt: &prompt,
        permission_profile: PermissionProfile::Orbit,
        barrier_timeout: Duration::from_secs(10),
        execution_timeout: Duration::from_secs(300),
        settlement_timeout: Duration::from_secs(10),
        output_limit: 64 * 1024,
    })?;
    #[cfg(feature = "routed-test-faults")]
    let synthetic_qualification =
        std::env::var_os("PYTXO_TEST_ROUTED_CLAUDE_SYNTHETIC_QUALIFICATION").is_some();
    #[cfg(not(feature = "routed-test-faults"))]
    let synthetic_qualification = false;
    let native_probe_digest = if synthetic_qualification {
        // Fault-test builds alone can exercise the admitted worker with the
        // selected real CLI without repeating pre-admission inference. This
        // receipt is synthetic and never qualifies a release build.
        canonical_digest(&(1_u32, "test-only-synthetic-claude-qualification"), 1)?
    } else {
        let success = run_owned_launch(
            &template.spec,
            &mut ProbeCallbacks {
                registered_at: None,
            },
            || routed_stop_requested(catalog, draft_id, run_id),
        )?;
        if owned_probe_was_stopped(&success) {
            cleanup_probe()?;
            return Err(RoutedPreflightStopped.into());
        }
        if parse_owned_claude_one_file_proposal(&template, &success)? != b"changed\n"
            || std::fs::read(&source)? != b"old\n"
            || std::fs::read(&sibling)? != b"untouched\n"
        {
            bail!("Claude proposal probe changed its worktree or returned unexpected text");
        }
        let registered_at = Arc::new(Mutex::new(None::<Instant>));
        let cancelled = run_owned_launch(
            &template.spec,
            &mut ProbeCallbacks {
                registered_at: Some(Arc::clone(&registered_at)),
            },
            || {
                let at = *registered_at.lock().map_err(|_| {
                    pytxo_core::PytxoError::Runner("Claude probe clock unavailable".into())
                })?;
                Ok(routed_stop_requested(catalog, draft_id, run_id)?
                    || at.is_some_and(|at| at.elapsed() >= Duration::from_millis(100)))
            },
        )?;
        if routed_stop_requested(catalog, draft_id, run_id)? && owned_probe_was_stopped(&cancelled)
        {
            cleanup_probe()?;
            return Err(RoutedPreflightStopped.into());
        }
        if cancelled.outcome != OwnedOutcome::Cancelled
            || !cancelled.process_registered
            || !cancelled.barrier_released
            || cancelled.active_processes != Some(0)
            || cancelled.error.is_some()
            || std::fs::read(&source)? != b"old\n"
            || std::fs::read(&sibling)? != b"untouched\n"
        {
            bail!("Claude proposal cancellation or Job-zero probe failed");
        }
        canonical_digest(
            &(
                1_u32,
                "claude-one-file-proposal-native-probe",
                &success.intent.job_name,
                success.payload_exit_code,
                &cancelled.intent.job_name,
                cancelled.active_processes,
            ),
            1,
        )?
    };
    let stop_requested = routed_stop_requested(catalog, draft_id, run_id)?;
    cleanup_probe()?;
    if stop_requested {
        return Err(RoutedPreflightStopped.into());
    }
    let now_ms = u64::try_from(chrono::Utc::now().timestamp_millis())?;
    let pool = profile
        .binding
        .capacity_pool_ids
        .iter()
        .next()
        .context("Claude subscription has no reviewed account pool")?;
    let capacity_ready = catalog
        .capacity_pool_status(pool)?
        .is_some_and(|status| status.available_units > 0);
    let qualification = AdapterQualification {
        receipt_digest: canonical_digest(
            &(
                1_u32,
                "claude-one-file-proposal-qualified",
                &auth.subscription_type,
                &auth.account_policy_digest,
                &auth.executable_digest,
                &native_probe_digest,
                &template.launch,
            ),
            1,
        )?,
        launch_fingerprint: qualification_fingerprint(
            &profile.profile,
            &profile.binding,
            &template.executable,
            &template.launch,
        )?,
        permission_profile: PermissionProfile::Orbit,
        tool_probe_passed: true,
        cancellation_probe_passed: true,
        quiescence_probe_passed: true,
        capabilities: BTreeSet::from(["edit".into()]),
        allowed_egress: BTreeSet::from([CLAUDE_SUBSCRIPTION_ENDPOINT.into()]),
        capacity_pool_ids: profile.binding.capacity_pool_ids.clone(),
    };
    Ok(QualifiedFixture {
        observation: ProfileObservation {
            schema_version: 1,
            profile_digest: template.profile_digest.clone(),
            binding_digest: template.binding_digest.clone(),
            executable: template.executable.clone(),
            launch: Some(template.launch.clone()),
            observed_at_ms: now_ms,
            expires_at_ms: now_ms + 10 * 60_000,
            auth_status: Readiness::Ready,
            dispatch_supported: true,
            qualification: Some(qualification),
            requested_model: template.requested_model.clone(),
            reported_model: None,
            model_identity_level: template.model_identity_level,
            metering_support: template.metering_support,
            hard_spend_limit_verified: false,
            capacity_ready,
        },
        executable: cli.clone(),
        hosted: Some(HostedFixture {
            bootstrap_host: embedded_host.clone(),
            prompt: vec![],
            proposal_template: Some(template),
        }),
    })
}

pub(crate) fn hosted_fixture_opted_in() -> bool {
    #[cfg(feature = "routed-test-faults")]
    {
        fixture_opted_in()
            && std::env::var("PYTXO_TEST_ROUTED_HOSTED_OWNED")
                .ok()
                .as_deref()
                == Some("1")
    }
    #[cfg(not(feature = "routed-test-faults"))]
    {
        false
    }
}

pub(crate) fn qualify_local_fixture(
    profile: &RegisteredProfile,
    catalog: &Catalog,
    probe_parent: &Path,
    worker_arguments: &[String],
    draft_id: &str,
    run_id: &str,
) -> anyhow::Result<QualifiedFixture> {
    #[cfg(feature = "routed-test-faults")]
    if hosted_fixture_opted_in() {
        return qualify_local_hosted_fixture(
            profile,
            catalog,
            probe_parent,
            worker_arguments,
            draft_id,
            run_id,
        );
    }
    ensure_routed_probe_not_stopped(catalog, draft_id, run_id)?;
    if !fixture_opted_in() {
        bail!("experimental routed local fixture is not opted in");
    }
    let execution = &profile.profile;
    let binding = &profile.binding;
    let adapter = if powershell_handoff_fixture_opted_in() && execution.id.0 == "strong" {
        #[cfg(feature = "routed-test-faults")]
        {
            POWERSHELL_FIXTURE_ADAPTER
        }
        #[cfg(not(feature = "routed-test-faults"))]
        {
            FIXTURE_ADAPTER
        }
    } else {
        FIXTURE_ADAPTER
    };
    if execution.harness_id != adapter
        || execution.backend != ExecutionBackend::Subprocess
        || execution.adapter_contract_version != "1"
        || execution.adapter_digest != Digest::of_bytes(adapter.as_bytes())
        || binding.endpoint_identity != adapter
        || binding.billing_mode != BillingSourceMode::Local
        || binding.credential_reference.is_some()
        || execution.requested_model.provider != "local-fixture"
        || binding.capacity_pool_ids.len() != 1
    {
        bail!("reviewed route is not the exact local fixture adapter");
    }
    let system_root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .context("Windows system root is missing")?;
    let executable = PinnedFile::observe(if adapter == FIXTURE_ADAPTER {
        system_root.join("System32").join("cmd.exe")
    } else {
        system_root
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe")
    })?;
    let parent_metadata = std::fs::symlink_metadata(probe_parent)?;
    if !probe_parent.is_absolute()
        || !parent_metadata.file_type().is_dir()
        || parent_metadata.file_type().is_symlink()
    {
        bail!("fixture probe parent is not a plain domain directory");
    }
    let probe_dir = probe_parent.join(format!("routing-probe-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&probe_dir)?;
    let spec = |arguments: Vec<String>| DirectOwnedLaunchSpec {
        executable: executable.clone(),
        arguments,
        windows_cmd_verbatim_tail: false,
        environment: BTreeMap::new(),
        working_directory: probe_dir.clone(),
        execution_timeout: PROBE_TIMEOUT,
        settlement_timeout: Duration::from_secs(3),
        output_limit: 4096,
    };
    let mut callbacks = ProbeCallbacks {
        registered_at: None,
    };
    let success_arguments = if adapter == FIXTURE_ADAPTER {
        vec![
            "/D".into(),
            "/Q".into(),
            "/C".into(),
            "echo".into(),
            "pytxo-probe>probe.txt".into(),
        ]
    } else {
        vec![
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            "[System.IO.File]::WriteAllText('probe.txt', 'pytxo-probe' + [Environment]::NewLine, [System.Text.Encoding]::ASCII)".into(),
        ]
    };
    let success = run_direct_owned_launch(&spec(success_arguments), &mut callbacks, || {
        routed_stop_requested(catalog, draft_id, run_id)
    })?;
    if owned_probe_was_stopped(&success) {
        if probe_dir.join("probe.txt").exists() {
            std::fs::remove_file(probe_dir.join("probe.txt"))?;
        }
        std::fs::remove_dir(&probe_dir)?;
        return Err(RoutedPreflightStopped.into());
    }
    if !positive_success(&success) {
        bail!(
            "local fixture tool or quiescence probe failed: {success:?}; error={:?}",
            success.error
        );
    }
    let probe_bytes = std::fs::read(probe_dir.join("probe.txt"))?;
    if probe_bytes != b"pytxo-probe\r\n" {
        bail!("local fixture edit probe did not write expected bytes");
    }
    let registered_at = Arc::new(Mutex::new(None::<Instant>));
    let mut callbacks = ProbeCallbacks {
        registered_at: Some(Arc::clone(&registered_at)),
    };
    let cancellation_arguments = if adapter == FIXTURE_ADAPTER {
        vec![
            "/D".into(),
            "/Q".into(),
            "/C".into(),
            "for".into(),
            "/L".into(),
            "%i".into(),
            "in".into(),
            "(1,1,2147483647)".into(),
            "do".into(),
            "@rem".into(),
        ]
    } else {
        vec![
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            "[System.Threading.Thread]::Sleep(30000)".into(),
        ]
    };
    let cancelled = run_direct_owned_launch(&spec(cancellation_arguments), &mut callbacks, || {
        let observed = *registered_at.lock().map_err(|_| {
            pytxo_core::PytxoError::Runner("fixture probe clock lock poisoned".into())
        })?;
        Ok(routed_stop_requested(catalog, draft_id, run_id)?
            || observed.is_some_and(|began| began.elapsed() >= Duration::from_millis(100)))
    })?;
    if routed_stop_requested(catalog, draft_id, run_id)? && owned_probe_was_stopped(&cancelled) {
        std::fs::remove_file(probe_dir.join("probe.txt"))?;
        std::fs::remove_dir(&probe_dir)?;
        return Err(RoutedPreflightStopped.into());
    }
    if cancelled.outcome != OwnedOutcome::Cancelled
        || !cancelled.process_registered
        || !cancelled.barrier_released
        || cancelled.active_processes != Some(0)
        || cancelled.error.is_some()
    {
        bail!("local fixture cancellation or Job-zero probe failed: {cancelled:?}");
    }
    std::fs::remove_file(probe_dir.join("probe.txt"))?;
    std::fs::remove_dir(&probe_dir)?;
    ensure_routed_probe_not_stopped(catalog, draft_id, run_id)?;
    let pool_id = binding
        .capacity_pool_ids
        .iter()
        .next()
        .context("fixture has no reviewed host pool")?;
    let capacity_ready = catalog
        .capacity_pool_status(pool_id)?
        .is_some_and(|pool| pool.available_units > 0);
    let now_ms = u64::try_from(chrono::Utc::now().timestamp_millis())?;
    let executable_identity = ExecutableIdentity {
        path: executable.path.to_string_lossy().into_owned(),
        version: "pinned-sha256-v1".into(),
        digest: Digest(executable.sha256.clone()),
    };
    let launch = LaunchContract {
        schema_version: 1,
        transport: LaunchTransport::DirectSubprocess,
        host: None,
        dependencies: vec![],
        arguments_digest: canonical_digest(&worker_arguments, 1)?,
        environment_policy_digest: canonical_digest(&BTreeMap::<String, String>::new(), 1)?,
        working_directory_policy: "reviewed_attempt_worktree_v1".into(),
        stdin_delivery: StdinDelivery::Closed,
        private_stdin_digest: None,
        output_protocol: "pytxo-direct-suspended/1".into(),
        argument_lowering: "windows-createprocess-structured-argv/v1".into(),
        barrier_timeout_ms: None,
        execution_timeout_ms: u64::try_from(fixture_worker_timeout().as_millis())?,
        settlement_timeout_ms: 5_000,
        output_limit_bytes: 4096,
    };
    let probe_receipt = canonical_digest(
        &(
            1_u32,
            &success.protocol,
            &executable.sha256,
            &success.intent.job_name,
            success.active_processes,
            success.payload_exit_code,
            Digest::of_bytes(&probe_bytes),
            &cancelled.intent.job_name,
            cancelled.active_processes,
            cancelled.terminated_job,
        ),
        1,
    )?;
    let qualification = AdapterQualification {
        receipt_digest: probe_receipt,
        launch_fingerprint: qualification_fingerprint(
            execution,
            binding,
            &executable_identity,
            &launch,
        )?,
        permission_profile: PermissionProfile::Orbit,
        tool_probe_passed: true,
        cancellation_probe_passed: true,
        quiescence_probe_passed: true,
        capabilities: execution
            .capabilities
            .intersection(&BTreeSet::from(["edit".to_owned()]))
            .cloned()
            .collect(),
        allowed_egress: Default::default(),
        capacity_pool_ids: binding.capacity_pool_ids.clone(),
    };
    let observation = ProfileObservation {
        schema_version: 1,
        profile_digest: execution.digest()?,
        binding_digest: binding.digest()?,
        executable: executable_identity,
        launch: Some(launch),
        observed_at_ms: now_ms,
        expires_at_ms: now_ms + 10 * 60_000,
        auth_status: Readiness::Ready,
        dispatch_supported: true,
        qualification: Some(qualification),
        requested_model: execution.requested_model.clone(),
        reported_model: None,
        model_identity_level: ModelIdentityLevel::Requested,
        metering_support: MeteringSupport::Verified,
        hard_spend_limit_verified: false,
        capacity_ready,
    };
    Ok(QualifiedFixture {
        observation,
        executable,
        hosted: None,
    })
}

/// Test-only qualification of the owned host protocol with a local command.
/// No provider identity or subscription readiness is inferred from this probe.
#[cfg(feature = "routed-test-faults")]
fn qualify_local_hosted_fixture(
    profile: &RegisteredProfile,
    catalog: &Catalog,
    probe_parent: &Path,
    worker_arguments: &[String],
    draft_id: &str,
    run_id: &str,
) -> anyhow::Result<QualifiedFixture> {
    ensure_routed_probe_not_stopped(catalog, draft_id, run_id)?;
    let execution = &profile.profile;
    let binding = &profile.binding;
    if !hosted_fixture_opted_in()
        || execution.harness_id != FIXTURE_ADAPTER
        || execution.backend != ExecutionBackend::Subprocess
        || execution.adapter_contract_version != "1"
        || execution.adapter_digest != Digest::of_bytes(FIXTURE_ADAPTER.as_bytes())
        || binding.endpoint_identity != FIXTURE_ADAPTER
        || binding.billing_mode != BillingSourceMode::Local
        || binding.credential_reference.is_some()
        || execution.requested_model.provider != "local-fixture"
        || binding.capacity_pool_ids.len() != 1
    {
        bail!("synthetic hosted route is not the exact local fixture adapter");
    }
    let host_path = PathBuf::from(
        std::env::var_os("PYTXO_TEST_EMBEDDED_HOST")
            .context("test-only embedded host pin is absent")?,
    );
    if !host_path.is_absolute() {
        bail!("test-only embedded host pin must be absolute");
    }
    let host_path = std::fs::canonicalize(host_path)?;
    if !host_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("pytxo-embedded-hosted-test.exe"))
    {
        bail!("synthetic hosted fixture requires the separate embedded test host");
    }
    let bootstrap_host = PinnedFile::observe(host_path)?;
    let payload_path = PathBuf::from(
        std::env::var_os("PYTXO_TEST_ROUTED_HOSTED_PAYLOAD")
            .context("test-only synthetic payload pin is absent")?,
    );
    if !payload_path.is_absolute() {
        bail!("test-only synthetic payload pin must be absolute");
    }
    let payload_path = std::fs::canonicalize(payload_path)?;
    if !payload_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("pytxo-synthetic-payload.exe"))
    {
        bail!("synthetic hosted fixture requires the separate local payload");
    }
    let executable = PinnedFile::observe(payload_path)?;
    let parent_metadata = std::fs::symlink_metadata(probe_parent)?;
    if !probe_parent.is_absolute()
        || !parent_metadata.file_type().is_dir()
        || parent_metadata.file_type().is_symlink()
    {
        bail!("synthetic hosted probe parent is not a plain directory");
    }
    let probe_dir = probe_parent.join(format!("routing-hosted-probe-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&probe_dir)?;
    let probe_stdin = b"synthetic owned-host probe; no provider call".to_vec();
    let spec = |arguments: Vec<String>| OwnedLaunchSpec {
        bootstrap_host: bootstrap_host.clone(),
        executable: executable.clone(),
        dependencies: vec![],
        arguments,
        environment: BTreeMap::new(),
        working_directory: probe_dir.clone(),
        stdin: probe_stdin.clone(),
        transport: OwnedTransport::Subprocess,
        barrier_timeout: Duration::from_secs(5),
        execution_timeout: PROBE_TIMEOUT,
        settlement_timeout: Duration::from_secs(3),
        output_limit: 4096,
    };
    let mut callbacks = ProbeCallbacks {
        registered_at: None,
    };
    let success = run_owned_launch(&spec(vec!["probe".into()]), &mut callbacks, || {
        routed_stop_requested(catalog, draft_id, run_id)
    })?;
    if owned_probe_was_stopped(&success) {
        if probe_dir.join("probe.txt").exists() {
            std::fs::remove_file(probe_dir.join("probe.txt"))?;
        }
        std::fs::remove_dir(&probe_dir)?;
        return Err(RoutedPreflightStopped.into());
    }
    if !positive_success(&success) || success.protocol != "pytxo-attempt-host/1" {
        bail!("synthetic hosted tool or Job-zero probe failed: {success:?}");
    }
    let probe_bytes = std::fs::read(probe_dir.join("probe.txt"))?;
    if probe_bytes != b"pytxo-probe\r\n" {
        bail!("synthetic hosted edit probe wrote unexpected bytes");
    }
    let registered_at = Arc::new(Mutex::new(None::<Instant>));
    let mut callbacks = ProbeCallbacks {
        registered_at: Some(Arc::clone(&registered_at)),
    };
    let cancelled = run_owned_launch(&spec(vec!["wait".into()]), &mut callbacks, || {
        let observed = *registered_at.lock().map_err(|_| {
            pytxo_core::PytxoError::Runner("hosted fixture probe clock lock poisoned".into())
        })?;
        Ok(routed_stop_requested(catalog, draft_id, run_id)?
            || observed.is_some_and(|began| began.elapsed() >= Duration::from_millis(100)))
    })?;
    if routed_stop_requested(catalog, draft_id, run_id)? && owned_probe_was_stopped(&cancelled) {
        std::fs::remove_file(probe_dir.join("probe.txt"))?;
        std::fs::remove_dir(&probe_dir)?;
        return Err(RoutedPreflightStopped.into());
    }
    if cancelled.outcome != OwnedOutcome::Cancelled
        || !cancelled.process_registered
        || !cancelled.barrier_released
        || cancelled.active_processes != Some(0)
        || cancelled.error.is_some()
    {
        bail!("synthetic hosted cancellation or Job-zero probe failed: {cancelled:?}");
    }
    std::fs::remove_file(probe_dir.join("probe.txt"))?;
    std::fs::remove_dir(&probe_dir)?;
    ensure_routed_probe_not_stopped(catalog, draft_id, run_id)?;
    let pool_id = binding
        .capacity_pool_ids
        .iter()
        .next()
        .context("synthetic hosted fixture has no reviewed host pool")?;
    let capacity_ready = catalog
        .capacity_pool_status(pool_id)?
        .is_some_and(|pool| pool.available_units > 0);
    let now_ms = u64::try_from(chrono::Utc::now().timestamp_millis())?;
    let executable_identity = ExecutableIdentity {
        path: executable.path.to_string_lossy().into_owned(),
        version: "pinned-sha256-v1".into(),
        digest: Digest(executable.sha256.clone()),
    };
    let host_identity = ExecutableIdentity {
        path: bootstrap_host.path.to_string_lossy().into_owned(),
        version: "pinned-sha256-v1".into(),
        digest: Digest(bootstrap_host.sha256.clone()),
    };
    // This pre-run observation is deliberately not dispatchable. Admission
    // binds the exact attempt ID and reviewed private prompt before Store sees it.
    let launch = LaunchContract {
        schema_version: 1,
        transport: LaunchTransport::HostSubprocess,
        host: Some(host_identity),
        dependencies: vec![],
        arguments_digest: canonical_digest(&worker_arguments, 1)?,
        environment_policy_digest: canonical_digest(&BTreeMap::<String, String>::new(), 1)?,
        working_directory_policy: "reviewed_attempt_worktree_v1".into(),
        stdin_delivery: StdinDelivery::PrivateHostPipe,
        private_stdin_digest: Some(Digest::of_bytes(b"unbound-synthetic-hosted-prompt")),
        output_protocol: "pytxo-attempt-host/1".into(),
        argument_lowering: "rust-std-command-windows-structured-argv/v1".into(),
        barrier_timeout_ms: Some(5_000),
        execution_timeout_ms: u64::try_from(fixture_worker_timeout().as_millis())?,
        settlement_timeout_ms: 5_000,
        output_limit_bytes: 4096,
    };
    let probe_receipt = canonical_digest(
        &(
            1_u32,
            "synthetic-owned-host-probe",
            &bootstrap_host.sha256,
            &executable.sha256,
            &success.protocol,
            &success.intent.job_name,
            success.active_processes,
            success.payload_exit_code,
            Digest::of_bytes(&probe_bytes),
            &cancelled.intent.job_name,
            cancelled.active_processes,
            cancelled.terminated_job,
        ),
        1,
    )?;
    let qualification = AdapterQualification {
        receipt_digest: probe_receipt,
        launch_fingerprint: qualification_fingerprint(
            execution,
            binding,
            &executable_identity,
            &launch,
        )?,
        permission_profile: PermissionProfile::Orbit,
        tool_probe_passed: true,
        cancellation_probe_passed: true,
        quiescence_probe_passed: true,
        capabilities: execution
            .capabilities
            .intersection(&BTreeSet::from(["edit".to_owned()]))
            .cloned()
            .collect(),
        allowed_egress: Default::default(),
        capacity_pool_ids: binding.capacity_pool_ids.clone(),
    };
    let observation = ProfileObservation {
        schema_version: 1,
        profile_digest: execution.digest()?,
        binding_digest: binding.digest()?,
        executable: executable_identity,
        launch: Some(launch),
        observed_at_ms: now_ms,
        expires_at_ms: now_ms + 10 * 60_000,
        auth_status: Readiness::Unknown,
        dispatch_supported: false,
        qualification: Some(qualification),
        requested_model: execution.requested_model.clone(),
        reported_model: None,
        model_identity_level: ModelIdentityLevel::Requested,
        metering_support: MeteringSupport::Verified,
        hard_spend_limit_verified: false,
        capacity_ready,
    };
    Ok(QualifiedFixture {
        observation,
        executable,
        hosted: Some(HostedFixture {
            bootstrap_host,
            prompt: vec![],
            proposal_template: None,
        }),
    })
}

fn positive_success(receipt: &OwnedLaunchReceipt) -> bool {
    receipt.outcome == OwnedOutcome::Succeeded
        && receipt.process_registered
        && receipt.barrier_released
        && receipt.active_processes == Some(0)
        && receipt.payload_exit_code == Some(0)
        && !receipt.output_truncated
        && receipt.output_complete
        && receipt.error.is_none()
}

struct ProbeCallbacks {
    registered_at: Option<Arc<Mutex<Option<Instant>>>>,
}
struct ProbeGuard {
    registered_at: Option<Arc<Mutex<Option<Instant>>>>,
}

impl LaunchCallbacks for ProbeCallbacks {
    fn authorize(&mut self, _: &LaunchIntent) -> pytxo_core::Result<Box<dyn LaunchGuard + '_>> {
        Ok(Box::new(ProbeGuard {
            registered_at: self.registered_at.clone(),
        }))
    }

    fn settle(&mut self, _: &OwnedLaunchReceipt) -> pytxo_core::Result<()> {
        Ok(())
    }
}

impl LaunchGuard for ProbeGuard {
    fn permit_create(&mut self, _: &LaunchIntent) -> pytxo_core::Result<()> {
        Ok(())
    }

    fn register(&mut self, process: &OwnedProcess) -> pytxo_core::Result<()> {
        if process.pid == 0 || process.start_identity.is_none() || process.job_name.is_empty() {
            return Err(pytxo_core::PytxoError::Runner(
                "fixture probe native identity is absent".into(),
            ));
        }
        if let Some(clock) = &self.registered_at {
            *clock.lock().map_err(|_| {
                pytxo_core::PytxoError::Runner("fixture probe clock lock poisoned".into())
            })? = Some(Instant::now());
        }
        #[cfg(feature = "routed-test-faults")]
        if let Some(root) = std::env::var_os("PYTXO_TEST_ROUTED_PAUSE_PROBE_REGISTER") {
            let root = PathBuf::from(root);
            if !root.is_absolute() || !root.is_dir() {
                return Err(pytxo_core::PytxoError::Runner(
                    "routed probe test pause root is invalid".into(),
                ));
            }
            std::fs::write(root.join("entered"), b"")?;
            let deadline = Instant::now() + Duration::from_secs(20);
            while !root.join("release").exists() {
                if Instant::now() >= deadline {
                    return Err(pytxo_core::PytxoError::Runner(
                        "routed probe test pause timed out".into(),
                    ));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        Ok(())
    }

    fn cancelled(&mut self) -> pytxo_core::Result<bool> {
        Ok(false)
    }
}

#[cfg(all(test, feature = "routed-test-faults", windows))]
mod powershell_handoff_tests {
    use super::powershell_worker_arguments;

    #[test]
    fn child_requires_exact_inherited_seed_bytes() {
        let root = tempfile::tempdir().unwrap();
        let shell = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe");
        let arguments =
            powershell_worker_arguments("pytxo-local-fixture-v1:write-result-from-seed").unwrap();
        let run = || {
            std::process::Command::new(&shell)
                .args(&arguments)
                .current_dir(root.path())
                .output()
                .unwrap()
        };
        assert!(!run().status.success());
        assert!(!root.path().join("result.txt").exists());
        std::fs::write(root.path().join("seed.txt"), b"pytxo-routed \r\n").unwrap();
        assert!(!run().status.success());
        assert!(!root.path().join("result.txt").exists());
        std::fs::write(root.path().join("seed.txt"), b"pytxo-routed\r\n").unwrap();
        assert!(run().status.success());
        assert_eq!(
            std::fs::read(root.path().join("result.txt")).unwrap(),
            b"pytxo-routed\r\n"
        );
    }
}
