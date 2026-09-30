//! Reviewed local fixture execution with owned workers, frozen checks, and one
//! bounded strong repair after the deliberate native-exit fixture failure.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use pytxo_core::routing::{
    canonical_digest, AttemptFailureClass, AttemptId, AttemptState, BillingSourceMode, BlobRef,
    ExecutableIdentity, RepairEvidence,
};
use pytxo_core::{ExecutionPlan, PreparedRunFileKind, PreparedRunManifest, PytxoConfig, RunId};
use pytxo_planner::advisor::Advisor;
#[cfg(feature = "routed-test-faults")]
use pytxo_runner::owned_launch::OwnedLaunchReceipt;
use pytxo_runner::owned_launch::{
    DirectOwnedLaunchSpec, OwnedLaunchSpec, OwnedOutcome, OwnedTransport, PinnedFile,
};
use pytxo_runner::{
    capture_reviewed_inputs, capture_sealed_output, capture_sealed_output_after_dependency,
    create_plain_verification_view, materialize_sealed_output, prepare_review_package,
    registry_path, seal_one_existing_claimed_text_proposal, verify_sealed_output_view,
    AgentWorkspaceInput, ProcessRegistryFile, ReviewedInputManifest, SealedOutputSnapshot,
};
use pytxo_store::capacity::{
    CapacityReleaseEvidence, CapacityReleaseEvidenceKind, CapacityReservationState,
};
use pytxo_store::routing::{
    RoutedAttemptRecord, RoutedUsage, RoutingControlEvent, RoutingFacts, RoutingMission,
    RoutingReceipts, RoutingScope, TransitionRoutingAttempt,
};
use pytxo_store::routing_capacity_intent::{
    AdmittedNoLaunchRelease, AdmittedQuiescentRelease, CapacityIntentPhase, QuiescentCheckerReceipt,
};
use pytxo_store::routing_checker::{
    CheckerOwnershipPhase, CheckerOwnershipRequest, CheckerSettlement,
};
use pytxo_store::routing_launch::{LaunchOwnershipPhase, LaunchSettlement};
use pytxo_store::routing_private::{
    CheckerNativeOutcome, ControllerObservation, ControllerReceiptEnvelope, PrivateArtifactClaim,
    PrivateArtifactKind, ReceiptSource,
};
use pytxo_store::{Catalog, PytxoStore};

use crate::flow::{load_reviewed_staged_mission, FlowPlan};
use crate::hypervisor::routed_review_authority;
use crate::routed_checker::{checker_claim, run_prepared_owned_checker, RoutedOwnedChecker};
use crate::routed_fixture::{
    admit_one_claude_proposal, admit_one_local_fixture, fixture_worker_timeout,
    prepare_admitted_worktree, AdmittedFixture, QualifiedFixture,
};
use crate::routed_worker::{
    inherited_snapshot_for_attempt, run_prepared_hosted_owned_worker, run_prepared_owned_worker,
    RoutedHostedWorker, RoutedOwnedWorker,
};
use crate::ActiveRunGate;
use crate::RunEnforcementEnvelope;

fn now_ms() -> Result<u64> {
    Ok(u64::try_from(chrono::Utc::now().timestamp_millis())?)
}

// Test-only, opt-in native diagnostics. Never format the receipt itself: its
// stdout, stderr, error, intent, and executable pins can contain private data.
#[cfg(feature = "routed-test-faults")]
fn fixture_launch_diagnostic<E>(
    stage: &'static str,
    task_index: usize,
    result: &std::result::Result<OwnedLaunchReceipt, E>,
) -> String {
    match result {
        Ok(receipt) => format!(
            "routed_fixture stage={stage} task_index={task_index} call=ok outcome={:?} \
             exit_code={:?} payload_exit_code={:?} elapsed_ms={} \
             process_registered={} barrier_released={} active_processes={:?} \
             terminated_job={} output_complete={} output_truncated={} error_present={}",
            receipt.outcome,
            receipt.exit_code,
            receipt.payload_exit_code,
            receipt.elapsed_ms,
            receipt.process_registered,
            receipt.barrier_released,
            receipt.active_processes,
            receipt.terminated_job,
            receipt.output_complete,
            receipt.output_truncated,
            receipt.error.is_some(),
        ),
        Err(_) => format!("routed_fixture stage={stage} task_index={task_index} call=error"),
    }
}

#[cfg(feature = "routed-test-faults")]
fn trace_fixture_launch<E>(
    stage: &'static str,
    task_index: usize,
    result: &std::result::Result<OwnedLaunchReceipt, E>,
) {
    if std::env::var_os("PYTXO_TEST_ROUTED_DIAGNOSTICS").is_some() {
        eprintln!("{}", fixture_launch_diagnostic(stage, task_index, result));
    }
}

#[cfg(all(test, feature = "routed-test-faults"))]
mod fixture_diagnostic_tests {
    use super::*;

    #[test]
    fn native_error_diagnostic_never_includes_error_text() {
        let failed: Result<OwnedLaunchReceipt> =
            Err(anyhow::anyhow!("private path and credential marker"));
        assert_eq!(
            fixture_launch_diagnostic("worker", 0, &failed),
            "routed_fixture stage=worker task_index=0 call=error",
        );
    }
}

fn current_attempt(store: &PytxoStore, admitted: &AdmittedFixture) -> Result<RoutedAttemptRecord> {
    store
        .routing_history(&admitted.scope)?
        .context("routed mission disappeared")?
        .attempts
        .into_iter()
        .find(|attempt| attempt.attempt_id == admitted.attempt_id)
        .context("routed attempt disappeared")
}

/// The one-task Review must describe exactly the byte and mode changes in the
/// owner-bound sealed snapshot, independent of a fixture filename or harness.
/// Candidate verification and Apply still own the repository-wide authority.
fn verify_one_task_review_matches_sealed(
    input: &ReviewedInputManifest,
    sealed: &SealedOutputSnapshot,
    review: &PreparedRunManifest,
    task_id: &str,
    agent_id: &str,
) -> Result<()> {
    if input.base != sealed.base || review.base_revision != sealed.base.git_revision {
        bail!("routed Review base differs from retained winner");
    }
    let evidence = pytxo_core::require_candidate_verification_contract(review)?;
    let base_inventory: BTreeMap<_, _> = evidence
        .base_inventory
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let candidate_inventory: BTreeMap<_, _> = evidence
        .candidate_inventory
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let mut after = BTreeMap::new();
    for file in &sealed.files {
        if after
            .insert(
                file.path.as_str(),
                (file.content_digest.as_str(), file.byte_length, file.mode),
            )
            .is_some()
        {
            bail!("routed sealed output contains duplicate paths");
        }
    }
    let mut before = BTreeMap::new();
    for file in &input.files {
        let mode = if let Some(entry) = base_inventory.get(file.path.as_str()) {
            if entry.sha256 != file.content_digest.0
                || (cfg!(unix)
                    && entry
                        .mode
                        .is_none_or(|mode| (mode & 0o111 != 0) != file.executable))
            {
                bail!("routed source inventory differs from reviewed Git input");
            }
            entry.mode
        } else {
            // Candidate verification excludes configured sparse paths. Such a
            // path cannot enter Review. The one-task worktree materializer
            // gives Git inputs exact canonical modes, so require those too.
            let Some((digest, _, mode)) = after.get(file.path.as_str()).copied() else {
                bail!("excluded routed input was deleted");
            };
            let canonical_mode = if cfg!(unix) {
                Some(if file.executable { 0o755 } else { 0o644 })
            } else {
                None
            };
            if digest != file.content_digest.as_str() || mode != canonical_mode {
                bail!("excluded routed input changed");
            }
            mode
        };
        if before
            .insert(
                file.path.as_str(),
                (file.content_digest.as_str(), file.byte_length, mode),
            )
            .is_some()
        {
            bail!("routed reviewed input contains duplicate paths");
        }
    }
    for (path, (digest, _, mode)) in &after {
        if let Some(entry) = candidate_inventory.get(path) {
            if entry.sha256 != *digest || entry.mode != *mode {
                bail!("routed candidate inventory differs from sealed output");
            }
        } else if !before.contains_key(path) {
            bail!("routed sealed output contains a new excluded path");
        }
    }
    let paths: BTreeSet<_> = before.keys().chain(after.keys()).copied().collect();
    let mut expected = BTreeMap::new();
    for path in paths {
        let old = before.get(path).copied();
        let new = after.get(path).copied();
        if old == new {
            continue;
        }
        let kind = match (old, new) {
            (None, Some(_)) => PreparedRunFileKind::Add,
            (Some(_), None) => PreparedRunFileKind::Delete,
            (Some(_), Some(_)) => PreparedRunFileKind::Modify,
            (None, None) => unreachable!("union path must exist on one side"),
        };
        expected.insert(path, (kind, old, new));
    }
    if expected.is_empty() || review.files.len() != expected.len() {
        bail!("routed Review does not contain the exact sealed winner changes");
    }
    for file in &review.files {
        let Some((kind, old, new)) = expected.remove(file.path.as_str()) else {
            bail!("routed Review includes an unsealed or duplicate path");
        };
        let before_digest = old.map(|value| value.0);
        let after_digest = new.map(|value| value.0);
        if file.task_id != task_id
            || file.agent_id != agent_id
            || file.kind != kind
            || file.before_sha256.as_deref() != before_digest
            || file.after_sha256.as_deref() != after_digest
            || file.blob_digest.as_deref() != after_digest
            || file.before_byte_count != old.map_or(0, |value| value.1)
            || file.after_byte_count != new.map_or(0, |value| value.1)
            || file.byte_count != new.or(old).map_or(0, |value| value.1)
            || file.before_mode != old.and_then(|value| value.2)
            || file.after_mode != new.and_then(|value| value.2)
        {
            bail!("routed Review file differs from the sealed winner");
        }
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep winner, retained bytes, reviewed plan, and domain authority explicit"
)]
fn prepare_winning_review(
    store: &PytxoStore,
    admitted: &AdmittedFixture,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    output_claim: &PrivateArtifactClaim,
    output_ref: &BlobRef,
    sealed: &SealedOutputSnapshot,
) -> Result<pytxo_core::PreparedRunManifest> {
    if mission.tasks.len() == 2 {
        let history = store
            .routing_history(&admitted.scope)?
            .context("two-wave routed Review history disappeared")?;
        let final_winner = history
            .tasks
            .iter()
            .find(|task| task.registration.contract.task_id == admitted.task_id)
            .and_then(|task| task.winner.as_ref())
            .context("two-wave routed Review has no final winner")?;
        if history.cancelled
            || load_reviewed_staged_mission(store, plan)? != *mission
            || final_winner.winning_attempt_id != admitted.attempt_id
            || final_winner.output_digest != output_ref.digest
            || current_attempt(store, admitted)?.state != AttemptState::Passed
            || serde_json::from_slice::<SealedOutputSnapshot>(
                &store.read_private_artifact(output_claim, output_ref)?,
            )? != *sealed
        {
            bail!("two-wave routed winner changed before Review");
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW").is_some() {
            bail!("injected pre-projection routed Review failure");
        }
        let domain = crate::default_hypervisor().ensure_domain(repo_root, cfg)?;
        let (execution, enforcement) = routed_review_authority(plan, &domain, cfg)?;
        if !store.begin_run_preparation(&admitted.scope.run_id.0)? {
            bail!("two-wave routed Review was already prepared or changed");
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_REVIEW_BEGIN").is_some() {
            bail!("injected post-winner Review failure");
        }
        return build_two_wave_review(
            store,
            &admitted.scope,
            cfg,
            repo_root,
            data_dir,
            &execution,
            &enforcement,
            TwoWaveReviewMode::Initial,
        );
    }
    let history = store
        .routing_history(&admitted.scope)?
        .context("routed winner history disappeared")?;
    let task = history
        .tasks
        .iter()
        .find(|task| task.registration.contract.task_id == admitted.task_id)
        .context("routed winner task disappeared")?;
    let winner = task.winner.as_ref().context("routed task has no winner")?;
    let attempt = current_attempt(store, admitted)?;
    let checks = store.owned_checker_pass_digest(&admitted.attempt_id)?;
    if history.cancelled
        || load_reviewed_staged_mission(store, plan)? != *mission
        || attempt.state != AttemptState::Passed
        || !attempt.ownership_released
        || winner.winning_attempt_id != admitted.attempt_id
        || winner.output_digest != output_ref.digest
        || winner.verification_receipt_digest != checks
        || attempt.receipts.sealed_output.as_ref() != Some(&output_ref.digest)
        || attempt.receipts.checks.as_ref() != Some(&checks)
        || sealed.base != task.registration.contract.base
    {
        bail!("routed winner cannot be projected into Review");
    }
    let retained: SealedOutputSnapshot =
        serde_json::from_slice(&store.read_private_artifact(output_claim, output_ref)?)?;
    if retained != *sealed {
        bail!("routed retained winner bytes changed before Review");
    }
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW").is_some() {
        bail!("injected pre-projection routed Review failure");
    }
    let domain = crate::default_hypervisor().ensure_domain(repo_root, cfg)?;
    let (execution, enforcement) = routed_review_authority(plan, &domain, cfg)?;
    let contract = store
        .get_run_contract(&admitted.scope.run_id.0)?
        .context("routed run has no insert-once review contract")?;
    if contract.apply_status != "pending"
        || contract.base_revision.as_deref() != Some(sealed.base.git_revision.as_str())
        || contract.plan_json.as_deref() != Some(serde_json::to_string(&execution)?.as_str())
        || contract.enforcement_json.as_deref()
            != Some(serde_json::to_string(&enforcement)?.as_str())
    {
        bail!("routed Review authority differs from the reviewed execution");
    }
    let review_view = create_plain_verification_view(
        data_dir,
        &admitted.scope.run_id.0,
        &format!("{}-review", admitted.attempt_id.0),
    )?;
    if std::fs::read_dir(&review_view)?.next().is_none() {
        materialize_sealed_output(&review_view, &retained)?;
    }
    verify_sealed_output_view(&review_view, &retained)?;
    store.promote_routed_winner_workspace(
        &admitted.agent_id,
        &admitted.scope.run_id.0,
        &admitted.task_id.0,
        &admitted.worktree.to_string_lossy(),
        &review_view.to_string_lossy(),
    )?;
    store.finish_agent(&admitted.agent_id, Some(0), "completed")?;
    if !store.begin_run_preparation(&admitted.scope.run_id.0)? {
        bail!("routed Review was already prepared or changed");
    }
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_REVIEW_BEGIN").is_some() {
        bail!("injected post-winner Review failure");
    }
    let workspace = AgentWorkspaceInput {
        agent_id: admitted
            .agent_id
            .strip_prefix(&format!("{}:", admitted.scope.run_id.0))
            .unwrap_or(&admitted.agent_id)
            .to_owned(),
        task_id: admitted.task_id.0.clone(),
        workspace_path: review_view.clone(),
        claims: task.registration.contract.claim_roots.clone(),
        depends_on: vec![],
    };
    let manifest = prepare_review_package(
        repo_root,
        data_dir,
        &admitted.scope.run_id.0,
        &sealed.base.git_revision,
        std::slice::from_ref(&workspace),
        &cfg.blast.sparse_exclude,
    )?;
    verify_sealed_output_view(&review_view, &retained)?;
    let verified =
        crate::verify_combined_candidate(&domain, cfg, &execution, &enforcement, manifest)?;
    verify_sealed_output_view(&review_view, &retained)?;
    let reviewed_input = capture_reviewed_inputs(repo_root, &retained.base)?;
    if reviewed_input != admitted.inputs {
        bail!("routed Review input manifest changed since admission");
    }
    verify_one_task_review_matches_sealed(
        &reviewed_input,
        &retained,
        &verified,
        &admitted.task_id.0,
        &workspace.agent_id,
    )?;
    Ok(verified)
}

/// Rebuild a failed routed Review only from the private, owner-bound winner.
/// A mutable worker/review workspace and an existing package are never inputs.
pub(crate) fn refresh_routed_winner_review(
    store: &PytxoStore,
    scope: &RoutingScope,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    execution: &ExecutionPlan,
    enforcement: &RunEnforcementEnvelope,
) -> Result<pytxo_core::PreparedRunManifest> {
    let history = store
        .routing_history(scope)?
        .context("routed Review history disappeared")?;
    if history.tasks.len() == 2 {
        return build_two_wave_review(
            store,
            scope,
            cfg,
            repo_root,
            data_dir,
            execution,
            enforcement,
            TwoWaveReviewMode::Refresh,
        );
    }
    if history.cancelled || history.tasks.len() != 1 || !(1..=2).contains(&history.attempts.len()) {
        bail!("routed Review requires one uncancelled, settled winner");
    }
    let task = &history.tasks[0];
    let winner = task
        .winner
        .as_ref()
        .context("routed Review has no winner")?;
    let attempt = history
        .attempts
        .iter()
        .find(|attempt| attempt.attempt_id == winner.winning_attempt_id)
        .context("routed Review winner attempt disappeared")?;
    if history.attempts.len() == 2 {
        let prior = history
            .attempts
            .iter()
            .find(|prior| Some(&prior.attempt_id) == attempt.predecessor.as_ref())
            .context("routed Review repair predecessor disappeared")?;
        if attempt.ordinal != 2
            || prior.ordinal != 1
            || prior.task_id != attempt.task_id
            || prior.state != AttemptState::Failed
            || !prior.ownership_released
            || !prior.failure.as_ref().is_some_and(|failure| {
                matches!(
                    failure.failure_class,
                    AttemptFailureClass::Implementation | AttemptFailureClass::Check
                ) && failure.actionable_evidence_digest.is_some()
            })
        {
            bail!("routed Review repair predecessor is not a settled actionable failure");
        }
    }
    let checks = store.owned_checker_pass_digest(&attempt.attempt_id)?;
    if task.state != pytxo_store::routing::TaskRoutingState::Succeeded
        || attempt.scope != *scope
        || attempt.task_id != task.registration.contract.task_id
        || winner.task_id != attempt.task_id
        || winner.winning_attempt_id != attempt.attempt_id
        || attempt.state != AttemptState::Passed
        || !attempt.ownership_released
        || attempt.receipts.sealed_output.as_ref() != Some(&winner.output_digest)
        || attempt.receipts.checks.as_ref() != Some(&winner.verification_receipt_digest)
        || checks != winner.verification_receipt_digest
    {
        bail!("routed Review winner evidence changed");
    }
    let planned = execution.waves.iter().flatten().collect::<Vec<_>>();
    if planned.len() != 1
        || planned[0].task_id != attempt.task_id
        || planned[0].paths != task.registration.contract.claim_roots
    {
        bail!("routed Review execution plan differs from its winner");
    }
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: attempt.task_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        reservation_id: attempt.capacity_reservation.clone(),
        kind: PrivateArtifactKind::ScopedOutput,
        artifact_id: format!("{}:sealed-output", attempt.attempt_id.0),
        event_id: format!("{}:sealed-output:observed", attempt.attempt_id.0),
    };
    let retained: SealedOutputSnapshot = serde_json::from_slice(
        &store.read_private_artifact_matching_digest(&claim, &winner.output_digest)?,
    )?;
    let contract = store
        .get_run_contract(&scope.run_id.0)?
        .context("routed Review contract disappeared")?;
    if retained.base != task.registration.contract.base
        || contract.base_revision.as_deref() != Some(retained.base.git_revision.as_str())
        || contract.plan_json.as_deref() != Some(serde_json::to_string(execution)?.as_str())
        || contract.enforcement_json.as_deref()
            != Some(serde_json::to_string(enforcement)?.as_str())
        || crate::current_head_revision(repo_root)? != retained.base.git_revision
    {
        bail!("routed Review base or authority changed; rerun the mission");
    }
    if crate::flow::observe_experimental_routed_git_base(repo_root)? != retained.base {
        bail!("routed Review primary checkout changed; rerun the mission");
    }
    let reviewed_input = capture_reviewed_inputs(repo_root, &retained.base)?;
    let view = create_plain_verification_view(
        data_dir,
        &scope.run_id.0,
        &format!(
            "{}-refresh-{}",
            attempt.attempt_id.0,
            uuid::Uuid::new_v4().simple()
        ),
    )?;
    materialize_sealed_output(&view, &retained)?;
    let workspace = AgentWorkspaceInput {
        agent_id: attempt
            .agent_id
            .strip_prefix(&format!("{}:", scope.run_id.0))
            .unwrap_or(&attempt.agent_id)
            .to_owned(),
        task_id: attempt.task_id.0.clone(),
        workspace_path: view.clone(),
        claims: task.registration.contract.claim_roots.clone(),
        depends_on: vec![],
    };
    let manifest = prepare_review_package(
        repo_root,
        data_dir,
        &scope.run_id.0,
        &retained.base.git_revision,
        std::slice::from_ref(&workspace),
        &cfg.blast.sparse_exclude,
    )?;
    verify_sealed_output_view(&view, &retained)?;
    let domain = crate::default_hypervisor().ensure_domain(repo_root, cfg)?;
    let verified =
        crate::verify_combined_candidate(&domain, cfg, execution, enforcement, manifest)?;
    verify_sealed_output_view(&view, &retained)?;
    verify_one_task_review_matches_sealed(
        &reviewed_input,
        &retained,
        &verified,
        &attempt.task_id.0,
        &workspace.agent_id,
    )?;
    let actor = store
        .get_agent(&attempt.agent_id)?
        .context("routed Review actor disappeared during refresh")?;
    let previous_view = actor
        .worktree_path
        .as_deref()
        .context("routed Review actor lost its workspace")?;
    store.refresh_routed_winner_workspace(
        &attempt.agent_id,
        &scope.run_id.0,
        &attempt.task_id.0,
        previous_view,
        &view.to_string_lossy(),
    )?;
    Ok(verified)
}

#[derive(Clone, Copy)]
enum TwoWaveReviewMode {
    Initial,
    Refresh,
}

/// Compose the two exact, owner-bound winners of a reviewed linear or
/// independent pair. Private Store artifacts supply candidate bytes.
#[expect(
    clippy::too_many_arguments,
    reason = "keep retained winners and reviewed execution authority explicit"
)]
fn build_two_wave_review(
    store: &PytxoStore,
    scope: &RoutingScope,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    execution: &ExecutionPlan,
    enforcement: &RunEnforcementEnvelope,
    mode: TwoWaveReviewMode,
) -> Result<pytxo_core::PreparedRunManifest> {
    let history = store
        .routing_history(scope)?
        .context("two-wave routed Review history disappeared")?;
    let parallel = execution.waves.len() == 1 && execution.waves[0].len() == 2;
    if history.cancelled
        || history.tasks.len() != 2
        || history.attempts.len() != 2
        || !(parallel
            || (execution.waves.len() == 2 && execution.waves.iter().all(|wave| wave.len() == 1)))
    {
        bail!("two-wave routed Review requires two settled linear winners");
    }
    let base = &history.tasks[0].registration.contract.base;
    let contract = store
        .get_run_contract(&scope.run_id.0)?
        .context("two-wave routed Review contract disappeared")?;
    if contract.apply_status != "preparing"
        || contract.base_revision.as_deref() != Some(base.git_revision.as_str())
        || contract.plan_json.as_deref() != Some(serde_json::to_string(execution)?.as_str())
        || contract.enforcement_json.as_deref()
            != Some(serde_json::to_string(enforcement)?.as_str())
        || crate::current_head_revision(repo_root)? != base.git_revision
        || crate::flow::observe_experimental_routed_git_base(repo_root)? != *base
    {
        bail!("two-wave routed Review base or authority changed");
    }
    capture_reviewed_inputs(repo_root, base)?;
    let mut workspaces = Vec::with_capacity(2);
    let mut views = Vec::with_capacity(2);
    let mut expected_files = BTreeMap::new();
    let mut predecessor = None;
    let mut predecessor_snapshot: Option<SealedOutputSnapshot> = None;
    let parent_id = execution.waves[0][0].task_id.clone();
    for ordinal in 0..2 {
        let planned = if parallel {
            &execution.waves[0][ordinal]
        } else {
            &execution.waves[ordinal][0]
        };
        let task = history
            .tasks
            .iter()
            .find(|task| task.registration.contract.task_id == planned.task_id)
            .context("two-wave reviewed task disappeared")?;
        let output_path = if ordinal == 0 {
            "seed.txt"
        } else {
            "result.txt"
        };
        let task_id = &task.registration.contract.task_id;
        let expected_dependencies = if ordinal == 0 || parallel {
            vec![]
        } else {
            vec![parent_id.clone()]
        };
        let winner = task
            .winner
            .as_ref()
            .context("two-wave task has no winner")?;
        let attempt = history
            .attempts
            .iter()
            .find(|attempt| attempt.attempt_id == winner.winning_attempt_id)
            .context("two-wave winner attempt disappeared")?;
        let checks = store.owned_checker_pass_digest(&attempt.attempt_id)?;
        if task.state != pytxo_store::routing::TaskRoutingState::Succeeded
            || task.current_attempt.as_ref() != Some(&attempt.attempt_id)
            || task.registration.contract.base != *base
            || task.registration.contract.claim_roots != [output_path]
            || task.registration.contract.dependencies != expected_dependencies
            || planned.task_id != *task_id
            || planned.paths != task.registration.contract.claim_roots
            || planned.depends_on
                != expected_dependencies
                    .iter()
                    .map(|id| id.0.clone())
                    .collect::<Vec<_>>()
            || winner.task_id != *task_id
            || attempt.scope != *scope
            || attempt.task_id != *task_id
            || attempt.state != AttemptState::Passed
            || !attempt.ownership_released
            || attempt.receipts.sealed_output.as_ref() != Some(&winner.output_digest)
            || attempt.receipts.checks.as_ref() != Some(&winner.verification_receipt_digest)
            || checks != winner.verification_receipt_digest
        {
            bail!("two-wave Review winner or execution plan differs from Core evidence");
        }
        if parallel && !attempt.dependencies.is_empty() {
            bail!("parallel sibling unexpectedly inherited output");
        } else if let Some(parent) = &predecessor {
            let inherited = store.read_routing_dependency_output(scope, task_id, &parent_id)?;
            if &inherited.winner != parent || attempt.dependencies != [inherited.winner] {
                bail!("two-wave child did not inherit the exact parent winner");
            }
        } else if !attempt.dependencies.is_empty() {
            bail!("two-wave parent unexpectedly has inherited output");
        }
        let claim = PrivateArtifactClaim {
            scope: scope.clone(),
            task_id: task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            reservation_id: attempt.capacity_reservation.clone(),
            kind: PrivateArtifactKind::ScopedOutput,
            artifact_id: format!("{}:sealed-output", attempt.attempt_id.0),
            event_id: format!("{}:sealed-output:observed", attempt.attempt_id.0),
        };
        let retained: SealedOutputSnapshot = serde_json::from_slice(
            &store.read_private_artifact_matching_digest(&claim, &winner.output_digest)?,
        )?;
        if retained.base != *base {
            bail!("two-wave retained winner has a different Git base");
        }
        let output = retained
            .files
            .iter()
            .find(|file| file.path == output_path)
            .context("two-wave retained winner has no claimed output")?;
        if output.content_digest != pytxo_core::routing::Digest::of_bytes(b"pytxo-routed\r\n")
            || output.byte_length != b"pytxo-routed\r\n".len() as u64
        {
            bail!("two-wave retained winner bytes differ from the bounded fixture");
        }
        if let Some(parent_snapshot) = &predecessor_snapshot {
            let parent_seed = parent_snapshot
                .files
                .iter()
                .find(|file| file.path == "seed.txt")
                .context("two-wave parent seed disappeared")?;
            if retained.files.iter().find(|file| file.path == "seed.txt") != Some(parent_seed) {
                bail!("two-wave child changed its inherited parent output");
            }
        }
        expected_files.insert(output_path.to_owned(), output.content_digest.0.clone());
        let view = create_plain_verification_view(
            data_dir,
            &scope.run_id.0,
            &format!(
                "{}-review-{}",
                attempt.attempt_id.0,
                uuid::Uuid::new_v4().simple()
            ),
        )?;
        materialize_sealed_output(&view, &retained)?;
        verify_sealed_output_view(&view, &retained)?;
        let actor = store
            .get_agent(&attempt.agent_id)?
            .context("two-wave Review actor disappeared")?;
        if actor.run_id != scope.run_id.0
            || actor.task_id != task_id.0
            || !(actor.status == "running"
                || (actor.status == "completed" && actor.exit_code == Some(0)))
        {
            bail!("two-wave Review actor changed from its passed attempt");
        }
        match mode {
            TwoWaveReviewMode::Initial => {
                let original = store
                    .routed_worktree_instance(&attempt.agent_id, &scope.run_id.0, &task_id.0)?
                    .context("two-wave winner lost its registered worktree")?;
                store.promote_routed_winner_workspace(
                    &attempt.agent_id,
                    &scope.run_id.0,
                    &task_id.0,
                    &original.path,
                    &view.to_string_lossy(),
                )?;
                store.finish_agent(&attempt.agent_id, Some(0), "completed")?;
            }
            TwoWaveReviewMode::Refresh => {
                if store
                    .routed_original_worktree(&attempt.agent_id, &scope.run_id.0)?
                    .is_some()
                {
                    // An interrupted initial Review may have projected the
                    // actor before recording its terminal status.
                    store.finish_agent(&attempt.agent_id, Some(0), "completed")?;
                    store.refresh_routed_winner_workspace(
                        &attempt.agent_id,
                        &scope.run_id.0,
                        &task_id.0,
                        actor
                            .worktree_path
                            .as_deref()
                            .context("two-wave actor has no prior view")?,
                        &view.to_string_lossy(),
                    )?;
                } else {
                    let original = store
                        .routed_worktree_instance(&attempt.agent_id, &scope.run_id.0, &task_id.0)?
                        .context("two-wave winner lost its registered worktree")?;
                    store.promote_routed_winner_workspace(
                        &attempt.agent_id,
                        &scope.run_id.0,
                        &task_id.0,
                        &original.path,
                        &view.to_string_lossy(),
                    )?;
                    store.finish_agent(&attempt.agent_id, Some(0), "completed")?;
                }
            }
        }
        workspaces.push(AgentWorkspaceInput {
            agent_id: attempt
                .agent_id
                .strip_prefix(&format!("{}:", scope.run_id.0))
                .unwrap_or(&attempt.agent_id)
                .to_owned(),
            task_id: task_id.0.clone(),
            workspace_path: view.clone(),
            claims: task.registration.contract.claim_roots.clone(),
            depends_on: expected_dependencies
                .iter()
                .map(|id| id.0.clone())
                .collect(),
        });
        views.push((view, retained.clone()));
        if !parallel {
            predecessor = Some(winner.clone());
            predecessor_snapshot = Some(retained);
        }
    }
    let manifest = prepare_review_package(
        repo_root,
        data_dir,
        &scope.run_id.0,
        &base.git_revision,
        &workspaces,
        &cfg.blast.sparse_exclude,
    )?;
    for (view, snapshot) in &views {
        verify_sealed_output_view(view, snapshot)?;
    }
    let domain = crate::default_hypervisor().ensure_domain(repo_root, cfg)?;
    let verified =
        crate::verify_combined_candidate(&domain, cfg, execution, enforcement, manifest)?;
    for (view, snapshot) in &views {
        verify_sealed_output_view(view, snapshot)?;
    }
    if verified.files.len() != expected_files.len()
        || verified.files.iter().any(|file| {
            expected_files.get(&file.path).map(String::as_str) != file.after_sha256.as_deref()
        })
    {
        bail!("two-wave Review package differs from retained winner outputs");
    }
    Ok(verified)
}

fn transition(
    store: &PytxoStore,
    admitted: &AdmittedFixture,
    current: &RoutedAttemptRecord,
    to: AttemptState,
    receipts: RoutingReceipts,
) -> Result<RoutedAttemptRecord> {
    let history = store
        .routing_history(&admitted.scope)?
        .context("routed task history disappeared")?;
    let task = history
        .tasks
        .iter()
        .find(|task| task.registration.contract.task_id == admitted.task_id)
        .context("routed task disappeared")?;
    let mut facts: RoutingFacts = admitted.facts.clone();
    facts.now_ms = now_ms()?.max(current.updated_at_ms);
    Ok(store.transition_routing_attempt(&TransitionRoutingAttempt {
        scope: admitted.scope.clone(),
        event_id: format!(
            "{}:{}:{}",
            admitted.attempt_id.0,
            current.revision,
            to.as_str()
        ),
        attempt_id: admitted.attempt_id.clone(),
        expected_attempt_revision: current.revision,
        expected_task_revision: task.revision,
        to,
        facts,
        receipts,
        failure: None,
    })?)
}

fn fail_actionable_attempt(
    store: &PytxoStore,
    admitted: &AdmittedFixture,
    current: &RoutedAttemptRecord,
    failure_class: AttemptFailureClass,
    evidence_digest: pytxo_core::routing::Digest,
) -> Result<RoutedAttemptRecord> {
    let history = store
        .routing_history(&admitted.scope)?
        .context("routed task history disappeared")?;
    let task = history
        .tasks
        .iter()
        .find(|task| task.registration.contract.task_id == admitted.task_id)
        .context("routed task disappeared")?;
    let mut facts = admitted.facts.clone();
    facts.now_ms = now_ms()?.max(current.updated_at_ms);
    Ok(store.transition_routing_attempt(&TransitionRoutingAttempt {
        scope: admitted.scope.clone(),
        event_id: format!("{}:{}:failed", admitted.attempt_id.0, current.revision),
        attempt_id: admitted.attempt_id.clone(),
        expected_attempt_revision: current.revision,
        expected_task_revision: task.revision,
        to: AttemptState::Failed,
        facts,
        receipts: RoutingReceipts::default(),
        failure: Some(RepairEvidence {
            attempt_id: admitted.attempt_id.clone(),
            ordinal: current.ordinal,
            state: AttemptState::Failed,
            failure_class,
            actionable_evidence_digest: Some(evidence_digest),
        }),
    })?)
}

fn claim(
    admitted: &AdmittedFixture,
    kind: PrivateArtifactKind,
    name: &str,
) -> PrivateArtifactClaim {
    PrivateArtifactClaim {
        scope: admitted.scope.clone(),
        task_id: admitted.task_id.clone(),
        attempt_id: admitted.attempt_id.clone(),
        reservation_id: admitted.reservation_id.clone(),
        kind,
        artifact_id: format!("{}:{name}", admitted.attempt_id.0),
        event_id: format!("{}:{name}:observed", admitted.attempt_id.0),
    }
}

fn require_closed_capacity_replay(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    evidence: &CapacityReleaseEvidence,
) -> Result<()> {
    let intent = store
        .capacity_intent(&admitted.reservation_id)?
        .context("closed routed capacity intent disappeared")?;
    let reservation = catalog
        .capacity_reservation(&admitted.reservation_id)?
        .context("closed routed Catalog reservation disappeared")?;
    let close_event_id = format!("{}:capacity-closed", admitted.attempt_id.0);
    if intent.phase != CapacityIntentPhase::Closed
        || intent.expected_release.as_ref() != Some(evidence)
        || intent.close_event_id.as_deref() != Some(close_event_id.as_str())
        || reservation.state != CapacityReservationState::Released
        || reservation.launch_token.as_deref() != Some(admitted.launch_token.as_str())
        || reservation.release_evidence.as_ref() != Some(evidence)
    {
        bail!("closed routed capacity replay differs from exact release proof");
    }
    Ok(())
}

fn release_terminal(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    data_dir: &Path,
) -> Result<()> {
    let _gate = ActiveRunGate::acquire(&data_dir.join("active_run.json"))?;
    release_terminal_unlocked(store, catalog, admitted)
}

/// Caller holds the exact active-run gate across terminal publication.
fn release_terminal_unlocked(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
) -> Result<()> {
    let attempt = current_attempt(store, admitted)?;
    if !attempt.state.is_terminal() || !attempt.ownership_released {
        bail!("routed attempt is not terminal with released domain ownership");
    }
    let owner = store
        .launch_ownership(&admitted.attempt_id)?
        .context("routed worker owner disappeared before release")?;
    if owner.phase != LaunchOwnershipPhase::Settled {
        bail!("routed worker has no settled Job-zero owner");
    }
    let worker_receipt_claim = claim(
        admitted,
        PrivateArtifactKind::ControllerReceipt,
        "native-job-zero",
    );
    let worker_receipt_ref = owner
        .settlement_blob
        .context("worker Job-zero bytes absent")?;
    let worker_receipt =
        store.read_controller_receipt(&worker_receipt_claim, &worker_receipt_ref)?;
    let mut observed_at_ms = worker_receipt.observed_at_ms;
    let mut checker_receipts = Vec::new();
    for ordinal in 1..=attempt.owned_checker_count {
        let checker = store
            .checker_ownership(&admitted.attempt_id, ordinal)?
            .context("routed checker owner disappeared before release")?;
        let receipt_ref = checker
            .settlement_blob
            .context("checker Job-zero bytes absent")?;
        let receipt_claim = checker_claim(&checker.request);
        let receipt = store.read_controller_receipt(&receipt_claim, &receipt_ref)?;
        observed_at_ms = observed_at_ms.max(receipt.observed_at_ms);
        checker_receipts.push(QuiescentCheckerReceipt {
            ordinal,
            receipt_claim,
            receipt_ref,
        });
    }
    let release = AdmittedQuiescentRelease {
        launch_token: admitted.launch_token.clone(),
        worker_receipt_claim: worker_receipt_claim.clone(),
        worker_receipt_ref,
        checker_receipts,
    };
    let evidence = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::QuiescenceReconciled,
        receipt_id: worker_receipt_claim.event_id,
        evidence_digest: canonical_digest(&release, 1)?.0,
        observed_at_ms,
    };
    store.bind_admitted_quiescent_release_proof(
        catalog,
        &admitted.reservation_id,
        &format!("{}:release-proof", admitted.attempt_id.0),
        &evidence,
        &release,
    )?;
    if store
        .capacity_intent(&admitted.reservation_id)?
        .context("routed capacity intent disappeared after proof")?
        .phase
        == CapacityIntentPhase::Closed
    {
        require_closed_capacity_replay(store, catalog, admitted, &evidence)?;
    } else {
        store.release_admitted_quiescent_capacity(catalog, &admitted.reservation_id)?;
        store.close_released_capacity_intent(
            catalog,
            &admitted.reservation_id,
            &format!("{}:capacity-closed", admitted.attempt_id.0),
        )?;
    }
    Ok(())
}

fn finish_local_run(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    data_dir: &Path,
    status: &str,
    worker_exit: Option<u32>,
) -> Result<RunId> {
    let active_path = data_dir.join("active_run.json");
    let _gate = ActiveRunGate::acquire(&active_path)?;
    finish_local_run_unlocked(store, catalog, admitted, data_dir, status, worker_exit)
}

/// Caller holds the exact active-run gate across run/marker settlement.
fn finish_local_run_unlocked(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    data_dir: &Path,
    status: &str,
    worker_exit: Option<u32>,
) -> Result<RunId> {
    let active_path = data_dir.join("active_run.json");
    let final_status =
        settle_local_attempt(store, catalog, admitted, data_dir, status, worker_exit)?;
    let cancelled = final_status == "cancelled";
    if !store.finish_run_if_status(&admitted.scope.run_id.0, "starting", &final_status)? {
        let observed = store
            .get_run_status(&admitted.scope.run_id.0)?
            .map(|row| row.0);
        if observed.as_deref() != Some(final_status.as_str())
            && !(observed.as_deref() == Some("cancelled") && cancelled)
        {
            bail!("routed run status changed before terminal settlement: {observed:?}");
        }
    }
    ProcessRegistryFile::update(&registry_path(data_dir), |registry| {
        registry.remove_run(&admitted.scope.run_id.0);
        Ok(())
    })?;
    super::clear_active_run_unlocked(&active_path, &admitted.scope.run_id.0)?;
    Ok(admitted.scope.run_id.clone())
}

/// Settle one attempt without ending a serial two-wave run. The parent must
/// remain active until the child reaches its own terminal Review boundary.
fn settle_local_attempt(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    data_dir: &Path,
    status: &str,
    worker_exit: Option<u32>,
) -> Result<String> {
    let terminal = current_attempt(store, admitted)?;
    let expected_usage = settled_attempt_usage(
        &terminal.selected.profile.harness_id,
        terminal.selected.binding.billing_mode,
        &terminal.scope,
        &terminal.attempt_id,
        &terminal.selected.observation.executable,
    )?;
    match &terminal.usage {
        recorded if recorded == &expected_usage => {}
        RoutedUsage::Unreported => {
            store.settle_routing_usage(
                &admitted.scope,
                &format!("{}:usage", admitted.attempt_id.0),
                &admitted.attempt_id,
                terminal.revision,
                &expected_usage,
                now_ms()?.max(terminal.updated_at_ms),
            )?;
        }
        _ => bail!("routed usage differs from the selected billing source"),
    }
    let cancelled = reconcile_user_stop(store, catalog, admitted, data_dir)?;
    if status == "failed" && !cancelled && terminal.state != AttemptState::Passed {
        let history = store
            .routing_history(&admitted.scope)?
            .context("routed failed mission disappeared before settlement")?;
        if !history.cancelled {
            store.cancel_routing_mission(
                &admitted.scope,
                &format!("{}:terminal-failure-fence", admitted.attempt_id.0),
                history.cancel_epoch,
                now_ms()?,
            )?;
        }
    }
    let final_status = if cancelled { "cancelled" } else { status };
    if let Some(agent) = store.get_agent(&admitted.agent_id)? {
        if agent.run_id != admitted.scope.run_id.0 || agent.task_id != admitted.task_id.0 {
            bail!("routed agent ledger identity changed before settlement");
        }
    } else {
        store.insert_agent_with_root(
            &admitted.agent_id,
            &admitted.scope.run_id.0,
            &admitted.task_id.0,
            1,
            Some(&admitted.worktree.to_string_lossy()),
            &terminal.selected.profile.harness_id,
            None,
        )?;
    }
    store.finish_agent(
        &admitted.agent_id,
        worker_exit.map(|code| code as i32),
        if terminal.state == AttemptState::Passed {
            "completed"
        } else if status == "retryable_failed" {
            "failed"
        } else {
            final_status
        },
    )?;
    Ok(final_status.to_owned())
}

/// Settlement follows the reviewed billing source. A subscription CLI has no
/// verified per-attempt charge receipt, even when its account auth is Ready.
fn settled_attempt_usage(
    harness_id: &str,
    billing_mode: BillingSourceMode,
    scope: &RoutingScope,
    attempt_id: &AttemptId,
    executable: &ExecutableIdentity,
) -> Result<RoutedUsage> {
    match (harness_id, billing_mode) {
        ("pytxo-local-fixture-v1", BillingSourceMode::Local) => Ok(RoutedUsage::Known {
            nano_usd: 0,
            receipt: canonical_digest(
                &(
                    1_u32,
                    "local-fixture-no-provider-charge",
                    scope,
                    attempt_id,
                    executable,
                ),
                1,
            )?,
        }),
        #[cfg(feature = "routed-test-faults")]
        ("pytxo-local-fixture-powershell-v1", BillingSourceMode::Local) => Ok(RoutedUsage::Known {
            nano_usd: 0,
            receipt: canonical_digest(
                &(
                    1_u32,
                    "local-fixture-no-provider-charge",
                    scope,
                    attempt_id,
                    executable,
                ),
                1,
            )?,
        }),
        ("claude", BillingSourceMode::Subscription) => Ok(RoutedUsage::Unknown {
            reason: "Claude subscription usage has no verified per-attempt cost receipt".into(),
        }),
        _ => bail!("routed billing source has no owned settlement policy"),
    }
}

#[cfg(test)]
mod usage_tests {
    use super::*;
    use pytxo_core::{DomainId, RunId};

    #[test]
    fn subscription_usage_is_unknown_and_never_fixture_zero() {
        let scope = RoutingScope {
            domain_id: DomainId("repo".into()),
            run_id: RunId("run".into()),
        };
        let attempt = AttemptId("attempt".into());
        let executable = ExecutableIdentity {
            path: "claude.exe".into(),
            version: "pinned".into(),
            digest: pytxo_core::routing::Digest::of_bytes(b"claude"),
        };
        assert!(matches!(
            settled_attempt_usage(
                "claude",
                BillingSourceMode::Subscription,
                &scope,
                &attempt,
                &executable
            )
            .unwrap(),
            RoutedUsage::Unknown { .. }
        ));
        assert!(settled_attempt_usage(
            "claude",
            BillingSourceMode::Api,
            &scope,
            &attempt,
            &executable
        )
        .is_err());
        assert!(matches!(
            settled_attempt_usage(
                "pytxo-local-fixture-v1",
                BillingSourceMode::Local,
                &scope,
                &attempt,
                &executable
            )
            .unwrap(),
            RoutedUsage::Known { nano_usd: 0, .. }
        ));
    }
}

/// The controller also cancels a mission to fence native creation after an
/// internal failure. Only Stop is a user cancellation for run status.
fn user_stop_recorded(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    data_dir: &Path,
) -> Result<bool> {
    let registry_stop = ProcessRegistryFile::load(&registry_path(data_dir))?
        .cancelled_runs
        .contains(&admitted.scope.run_id.0);
    let history = store
        .routing_history(&admitted.scope)?
        .context("routed mission disappeared before Stop classification")?;
    Ok(crate::routed_fixture::routed_stop_requested(
        catalog,
        &admitted.draft_id,
        &admitted.scope.run_id.0,
    )? || registry_stop
        || history.events.iter().any(|entry| {
            entry.event_id == "controller.stop.cancel.v1"
                && matches!(&entry.event, RoutingControlEvent::Cancelled { .. })
        }))
}

/// A Catalog Stop can outrun Desktop's exact Stop IPC. Carry that user request
/// into Core before terminalizing. Parallel siblings may reach this point
/// together, so an already recorded cancellation wins the CAS race safely.
fn reconcile_user_stop(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    data_dir: &Path,
) -> Result<bool> {
    let stopped = user_stop_recorded(store, catalog, admitted, data_dir)?;
    if stopped {
        let history = store
            .routing_history(&admitted.scope)?
            .context("routed mission disappeared before Stop reconciliation")?;
        if !history.cancelled {
            if let Err(error) = store.cancel_routing_mission(
                &admitted.scope,
                "controller.stop.cancel.v1",
                history.cancel_epoch,
                now_ms()?,
            ) {
                let observed = store
                    .routing_history(&admitted.scope)?
                    .context("routed mission disappeared during Stop reconciliation")?;
                if !observed.cancelled {
                    return Err(error.into());
                }
            }
        }
    }
    Ok(stopped)
}

/// A Prepared owner proves native create was never authorized. The retained
/// no-create receipt and bound release are replayable independently, so a
/// failure between Store and Catalog writes keeps the exact hold recoverable.
fn reconcile_no_create_failure(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    data_dir: &Path,
    sibling_mode: bool,
) -> Result<RunId> {
    let _gate = ActiveRunGate::acquire(&data_dir.join("active_run.json"))?;
    let owner = store
        .launch_ownership(&admitted.attempt_id)?
        .context("routed no-create owner is absent; retaining capacity")?;
    if owner.request.scope != admitted.scope
        || owner.request.task_id != admitted.task_id
        || owner.request.attempt_id != admitted.attempt_id
        || owner.request.reservation_id != admitted.reservation_id
        || owner.request.launch_token != admitted.launch_token
        || !matches!(
            owner.phase,
            LaunchOwnershipPhase::Prepared | LaunchOwnershipPhase::ClosedNoLaunch
        )
    {
        bail!("routed native create may have started; retaining capacity");
    }
    let history = store
        .routing_history(&admitted.scope)?
        .context("routed no-create mission disappeared")?;
    let own_event = format!("{}:controller-no-create", admitted.attempt_id.0);
    let (cancellation_event_id, cancellation_at_ms) = if history.cancelled {
        history
            .events
            .iter()
            .find_map(|entry| match &entry.event {
                RoutingControlEvent::Cancelled { now_ms, .. }
                    if entry.event_id == own_event
                        || entry.event_id == "controller.stop.cancel.v1"
                        || sibling_mode =>
                {
                    Some((entry.event_id.clone(), *now_ms))
                }
                _ => None,
            })
            .context("routed no-create cancellation has no witnessed event")?
    } else {
        let cancelled_at = now_ms()?;
        store.cancel_routing_mission(
            &admitted.scope,
            &own_event,
            history.cancel_epoch,
            cancelled_at,
        )?;
        (own_event, cancelled_at)
    };
    let receipt_claim = claim(
        admitted,
        PrivateArtifactKind::ControllerReceipt,
        "native-no-create",
    );
    let receipt_ref = if owner.phase == LaunchOwnershipPhase::Prepared {
        let receipt = ControllerReceiptEnvelope {
            schema_version: 1,
            scope: admitted.scope.clone(),
            task_id: admitted.task_id.clone(),
            attempt_id: admitted.attempt_id.clone(),
            reservation_id: admitted.reservation_id.clone(),
            observed_at_ms: cancellation_at_ms,
            evidence_id: receipt_claim.event_id.clone(),
            source: ReceiptSource::TrustedController,
            observation: ControllerObservation::AdmittedNoLaunch {
                cancellation_event_id: cancellation_event_id.clone(),
            },
        };
        let reference = store.put_controller_receipt(&receipt_claim, &receipt)?;
        store.close_prepared_launch_without_worker(&LaunchSettlement {
            scope: admitted.scope.clone(),
            attempt_id: admitted.attempt_id.clone(),
            event_id: format!("{}:native-no-create-closed", admitted.attempt_id.0),
            receipt_claim: receipt_claim.clone(),
            receipt_ref: reference.clone(),
        })?;
        reference
    } else {
        owner
            .settlement_blob
            .context("closed no-create owner lost its receipt")?
    };
    let receipt = store.read_controller_receipt(&receipt_claim, &receipt_ref)?;
    if receipt.source != ReceiptSource::TrustedController
        || receipt.observed_at_ms != cancellation_at_ms
        || !matches!(
            &receipt.observation,
            ControllerObservation::AdmittedNoLaunch { cancellation_event_id: id }
                if id == &cancellation_event_id
        )
    {
        bail!("closed no-create receipt differs from durable cancellation");
    }
    let attempt = current_attempt(store, admitted)?;
    if !attempt.state.is_terminal() {
        if !matches!(
            attempt.state,
            AttemptState::Preparing | AttemptState::Launching
        ) {
            bail!("routed no-create attempt has no safe terminal edge");
        }
        transition(
            store,
            admitted,
            &attempt,
            AttemptState::FailedNoLaunch,
            RoutingReceipts {
                no_worker_created: Some(receipt_ref.digest.clone()),
                ..Default::default()
            },
        )?;
    } else if attempt.state != AttemptState::FailedNoLaunch
        || !attempt.ownership_released
        || attempt.receipts.no_worker_created.as_ref() != Some(&receipt_ref.digest)
    {
        bail!("routed no-create terminal attempt differs from retained receipt");
    }
    let release = AdmittedNoLaunchRelease {
        launch_token: admitted.launch_token.clone(),
        receipt_claim: receipt_claim.clone(),
        receipt_ref: receipt_ref.clone(),
    };
    let evidence = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::KnownUnused,
        receipt_id: receipt_claim.event_id,
        evidence_digest: receipt_ref.digest.0,
        observed_at_ms: receipt.observed_at_ms,
    };
    store.bind_admitted_no_launch_release_proof(
        catalog,
        &admitted.reservation_id,
        &format!("{}:release-proof", admitted.attempt_id.0),
        &evidence,
        &release,
    )?;
    if store
        .capacity_intent(&admitted.reservation_id)?
        .context("routed no-create capacity intent disappeared after proof")?
        .phase
        == CapacityIntentPhase::Closed
    {
        require_closed_capacity_replay(store, catalog, admitted, &evidence)?;
    } else {
        store.release_admitted_no_launch_capacity(catalog, &admitted.reservation_id)?;
        store.close_released_capacity_intent(
            catalog,
            &admitted.reservation_id,
            &format!("{}:capacity-closed", admitted.attempt_id.0),
        )?;
    }
    if sibling_mode {
        settle_local_attempt(store, catalog, admitted, data_dir, "failed", None)?;
        Ok(admitted.scope.run_id.clone())
    } else {
        finish_local_run_unlocked(store, catalog, admitted, data_dir, "failed", None)
    }
}

/// An error after an observed worker Job-zero can release the exact attempt
/// only when every prepared checker is known not to have started. Ambiguous
/// native ownership remains held for explicit recovery.
fn reconcile_quiescent_failure(
    store: &PytxoStore,
    catalog: &Catalog,
    admitted: &AdmittedFixture,
    data_dir: &Path,
    worker_exit: Option<u32>,
    sibling_mode: bool,
) -> Result<RunId> {
    let active_path = data_dir.join("active_run.json");
    let gate = ActiveRunGate::acquire(&active_path)?;
    let owner = store
        .launch_ownership(&admitted.attempt_id)?
        .context("routed worker owner is absent during failure reconciliation")?;
    if owner.phase != LaunchOwnershipPhase::Settled {
        bail!("routed worker Job-zero is not proven; retaining capacity");
    }
    if owner.request.scope != admitted.scope
        || owner.request.task_id != admitted.task_id
        || owner.request.attempt_id != admitted.attempt_id
        || owner.request.reservation_id != admitted.reservation_id
        || owner.request.launch_token != admitted.launch_token
    {
        bail!("settled worker owner differs from admitted attempt; retaining capacity");
    }
    let mut attempt = current_attempt(store, admitted)?;
    if attempt.state == AttemptState::Launching {
        // Registration can commit before the callback records Running. The
        // exact native identity and Job-zero are already durable; reconstruct
        // only that missing edge, with no new launch authorization.
        let job = owner
            .job_name
            .as_deref()
            .context("settled worker has no Job name")?;
        let pid = owner.pid.context("settled worker has no PID")?;
        let start = owner
            .start_identity
            .as_deref()
            .context("settled worker has no start identity")?;
        let identity = canonical_digest(&(1_u32, job, pid, start), 1)?;
        attempt = transition(
            store,
            admitted,
            &attempt,
            AttemptState::Running,
            RoutingReceipts {
                process_identity: Some(identity),
                ..Default::default()
            },
        )?;
    }
    if attempt.state == AttemptState::Running {
        let quiescence = owner
            .settlement_blob
            .context("routed worker Job-zero bytes are absent")?;
        attempt = transition(
            store,
            admitted,
            &attempt,
            AttemptState::Sealing,
            RoutingReceipts {
                quiescence: Some(quiescence.digest.clone()),
                ..Default::default()
            },
        )?;
    }
    if !attempt.state.is_terminal() {
        if !matches!(
            attempt.state,
            AttemptState::Sealing | AttemptState::Verifying
        ) {
            bail!("routed failure has no safe terminal edge; retaining capacity");
        }
        for ordinal in 1..=attempt.owned_checker_count {
            let checker = store
                .checker_ownership(&admitted.attempt_id, ordinal)?
                .context("routed checker owner is absent during failure reconciliation")?;
            match checker.phase {
                CheckerOwnershipPhase::Settled | CheckerOwnershipPhase::ClosedNoCreate => {}
                CheckerOwnershipPhase::Prepared => {
                    let history = store
                        .routing_history(&admitted.scope)?
                        .context("routed mission vanished during checker closure")?;
                    let own_event = format!("{}:controller-abort", admitted.attempt_id.0);
                    let (cancellation_event_id, cancellation_at_ms) = if history.cancelled {
                        history
                            .events
                            .iter()
                            .find_map(|entry| match &entry.event {
                                RoutingControlEvent::Cancelled { now_ms, .. }
                                    if entry.event_id == own_event
                                        || entry.event_id == "controller.stop.cancel.v1"
                                        || sibling_mode =>
                                {
                                    Some((entry.event_id.clone(), *now_ms))
                                }
                                _ => None,
                            })
                            .context("routed cancellation has no witnessed no-create event")?
                    } else {
                        let cancelled_at = now_ms()?;
                        store.cancel_routing_mission(
                            &admitted.scope,
                            &own_event,
                            history.cancel_epoch,
                            cancelled_at,
                        )?;
                        (own_event, cancelled_at)
                    };
                    let receipt_claim = checker_claim(&checker.request);
                    let receipt = ControllerReceiptEnvelope {
                        schema_version: 1,
                        scope: admitted.scope.clone(),
                        task_id: admitted.task_id.clone(),
                        attempt_id: admitted.attempt_id.clone(),
                        reservation_id: admitted.reservation_id.clone(),
                        observed_at_ms: cancellation_at_ms,
                        evidence_id: receipt_claim.event_id.clone(),
                        source: ReceiptSource::TrustedController,
                        observation: ControllerObservation::CheckerNotCreated {
                            check_id: checker.request.check_id.clone(),
                            ordinal,
                            cancellation_event_id,
                        },
                    };
                    let receipt_ref = store.put_controller_receipt(&receipt_claim, &receipt)?;
                    store.close_prepared_checker_without_create(&CheckerSettlement {
                        scope: admitted.scope.clone(),
                        attempt_id: admitted.attempt_id.clone(),
                        ordinal,
                        event_id: format!(
                            "{}:checker-{ordinal}:not-created",
                            admitted.attempt_id.0
                        ),
                        receipt_claim,
                        receipt_ref,
                    })?;
                }
                CheckerOwnershipPhase::CreateMayHaveStarted | CheckerOwnershipPhase::Registered => {
                    bail!("routed checker native ownership is ambiguous; retaining capacity");
                }
            }
        }
        let cancelled = reconcile_user_stop(store, catalog, admitted, data_dir)?;
        let target = if cancelled {
            AttemptState::Cancelled
        } else {
            AttemptState::Failed
        };
        let latest = current_attempt(store, admitted)?;
        transition(store, admitted, &latest, target, RoutingReceipts::default())?;
        attempt = current_attempt(store, admitted)?;
    }
    let status = if attempt.state == AttemptState::Passed {
        "completed"
    } else if attempt.state == AttemptState::Cancelled {
        "cancelled"
    } else {
        "failed"
    };
    if attempt.state == AttemptState::Passed {
        release_terminal_unlocked(store, catalog, admitted)?;
        if sibling_mode {
            settle_local_attempt(store, catalog, admitted, data_dir, status, worker_exit)?;
            return Ok(admitted.scope.run_id.clone());
        }
        return finish_local_run_unlocked(store, catalog, admitted, data_dir, status, worker_exit);
    }
    drop(gate);
    release_terminal(store, catalog, admitted, data_dir)?;
    if sibling_mode {
        settle_local_attempt(store, catalog, admitted, data_dir, status, worker_exit)?;
        Ok(admitted.scope.run_id.clone())
    } else {
        finish_local_run(store, catalog, admitted, data_dir, status, worker_exit)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep the two native attempt owners and reviewed run authority explicit"
)]
fn run_two_sibling_fixtures(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    qualified: Vec<Vec<QualifiedFixture>>,
    advisor: Option<Arc<dyn Advisor>>,
) -> Result<RunId> {
    if !crate::flow::routed_parallel_siblings(plan)
        || qualified.len() != 2
        || load_reviewed_staged_mission(store, plan)? != *mission
    {
        bail!("parallel fixture differs from the exact reviewed sibling wave");
    }
    let capacity = catalog
        .capacity_pool_status("host")?
        .context("parallel fixture host pool disappeared")?;
    if mission.tasks.iter().any(|task| {
        task.contract.required_resources.len() != 1
            || !task.contract.required_resources.contains("host")
    }) || capacity.available_units < 2
    {
        settle_between_wave_admission_failure(store, catalog, mission, plan, repo_root, data_dir)?;
        bail!("parallel fixture has fewer than two available reviewed host slots");
    }
    let mut admitted = Vec::with_capacity(2);
    for (index, observed) in qualified.into_iter().enumerate() {
        match admit_one_local_fixture(
            store,
            catalog,
            mission,
            plan,
            cfg,
            repo_root,
            data_dir,
            index,
            observed,
            advisor.clone(),
            None,
        ) {
            Ok(attempt) => {
                admitted.push(attempt);
                #[cfg(feature = "routed-test-faults")]
                if index == 0
                    && std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_ADMITTED")
                        .is_some()
                {
                    panic!("injected controller death after first sibling admission");
                }
            }
            Err(error) => {
                for attempt in &admitted {
                    reconcile_no_create_failure(store, catalog, attempt, data_dir, true)
                        .context("admitted sibling could not close without native creation")?;
                }
                settle_between_wave_admission_failure(
                    store, catalog, mission, plan, repo_root, data_dir,
                )?;
                return Err(error).context("parallel sibling admission failed");
            }
        }
    }
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_SIBLINGS_ADMITTED").is_some() {
        panic!("injected controller death after both sibling admissions");
    }
    let store_path = cfg.db_path_at(repo_root);
    let catalog_path = catalog.opened_path().to_path_buf();
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_PASSED").is_some()
        || std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_FAILED").is_some()
    {
        let worker_store = PytxoStore::open_existing_read_write(&store_path)?;
        let worker_catalog = Catalog::open_existing_for_capacity_recovery(&catalog_path)?;
        execute_admitted_fixture(
            &worker_store,
            &worker_catalog,
            mission,
            plan,
            cfg,
            repo_root,
            data_dir,
            0,
            false,
            true,
            None,
            admitted[0].clone(),
        )?;
        panic!("injected controller death after first sibling settled");
    }
    let first = admitted[0].clone();
    let second = admitted[1].clone();
    // Git worktree registration mutates shared repository metadata. Keep that
    // short preparation phase serial while native workers still run together.
    let preparation_gate = std::sync::Mutex::new(());
    let launch = |index, attempt| -> Result<RunId> {
        let worker_store = PytxoStore::open_existing_read_write(&store_path)?;
        let worker_catalog = Catalog::open_existing_for_capacity_recovery(&catalog_path)?;
        execute_admitted_fixture(
            &worker_store,
            &worker_catalog,
            mission,
            plan,
            cfg,
            repo_root,
            data_dir,
            index,
            false,
            true,
            Some(&preparation_gate),
            attempt,
        )
    };
    let results = std::thread::scope(|threads| {
        let launch = &launch;
        let one = threads.spawn(move || launch(0, first));
        let two = threads.spawn(move || launch(1, second));
        [one.join(), two.join()]
    });
    let mut errors = Vec::new();
    for (index, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(run_id)) if run_id == mission.authorization.run_id => {}
            Ok(Ok(_)) => errors.push(format!("sibling {index} returned a different run")),
            Ok(Err(error)) => errors.push(format!("sibling {index}: {error:#}")),
            Err(_) => errors.push(format!("sibling {index} controller panicked")),
        }
    }
    let scope = RoutingScope {
        domain_id: mission.authorization.domain_id.clone(),
        run_id: mission.authorization.run_id.clone(),
    };
    let history = store
        .routing_history(&scope)?
        .context("parallel sibling history disappeared after native execution")?;
    let settled = history.attempts.len() == 2
        && history
            .attempts
            .iter()
            .all(|attempt| attempt.state.is_terminal() && attempt.ownership_released)
        && !store
            .unresolved_routing_attempts()?
            .iter()
            .any(|attempt| attempt.scope == scope)
        && !store.unresolved_capacity_intent_scopes()?.contains(&scope)
        && store
            .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))?
            .is_empty()
        && !catalog
            .unresolved_capacity_reservations()?
            .iter()
            .any(|reservation| {
                reservation.domain_id == scope.domain_id.0 && reservation.run_id == scope.run_id.0
            });
    if !settled {
        bail!(
            "parallel sibling ownership remains unresolved; active run retained for recovery: {}",
            errors.join("; ")
        );
    }
    let both_passed = !history.cancelled
        && history.tasks.len() == 2
        && history.tasks.iter().all(|task| {
            task.winner.as_ref().is_some_and(|winner| {
                history.attempts.iter().any(|attempt| {
                    attempt.attempt_id == winner.winning_attempt_id
                        && attempt.task_id == task.registration.contract.task_id
                        && attempt.state == AttemptState::Passed
                        && attempt.ownership_released
                })
            })
        });
    if !errors.is_empty() || !both_passed {
        settle_between_wave_admission_failure(store, catalog, mission, plan, repo_root, data_dir)?;
        bail!("parallel siblings did not both pass: {}", errors.join("; "));
    }
    let review = (|| -> Result<RunId> {
        let domain = crate::default_hypervisor().ensure_domain(repo_root, cfg)?;
        let (execution, enforcement) = routed_review_authority(plan, &domain, cfg)?;
        if load_reviewed_staged_mission(store, plan)? != *mission
            || !store.begin_run_preparation(&scope.run_id.0)?
        {
            bail!("parallel sibling Review authority changed before preparation");
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS").is_some() {
            panic!("injected controller death after settled sibling attempts");
        }
        let manifest = build_two_wave_review(
            store,
            &scope,
            cfg,
            repo_root,
            data_dir,
            &execution,
            &enforcement,
            TwoWaveReviewMode::Initial,
        )?;
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_SIBLING_REVIEW").is_some() {
            let saved = catalog
                .get_flow_draft(&plan.draft_id)?
                .context("injected sibling Stop lost reviewed Flow")?;
            let plan_json = saved
                .plan_json
                .context("injected sibling Stop lost reviewed plan")?;
            if !catalog.request_routed_flow_stop(&plan.draft_id, &plan_json, &scope.run_id.0)? {
                bail!("injected sibling Stop did not match the reviewed run");
            }
        }
        let active_path = data_dir.join("active_run.json");
        let _gate = ActiveRunGate::acquire(&active_path)?;
        if crate::routed_fixture::routed_stop_requested(catalog, &plan.draft_id, &scope.run_id.0)?
            || ProcessRegistryFile::load(&registry_path(data_dir))?
                .cancelled_runs
                .contains(&scope.run_id.0)
            || store
                .routing_history(&scope)?
                .context("parallel mission disappeared")?
                .cancelled
            || load_reviewed_staged_mission(store, plan)? != *mission
        {
            bail!("parallel sibling Review authority changed before publication");
        }
        store.finish_routed_review_and_run(&scope.run_id.0, &manifest, "starting")?;
        finish_local_run_unlocked(store, catalog, &admitted[1], data_dir, "completed", Some(0))
    })();
    match review {
        Ok(run_id) => Ok(run_id),
        Err(error) => {
            let _gate = ActiveRunGate::acquire(&data_dir.join("active_run.json"))?;
            let contract = store
                .get_run_contract(&scope.run_id.0)?
                .context("parallel sibling Review contract disappeared")?;
            if contract.apply_status == "pending" {
                store.begin_run_preparation(&scope.run_id.0)?;
            }
            if matches!(contract.apply_status.as_str(), "pending" | "preparing") {
                store.fail_run_preparation(
                    &scope.run_id.0,
                    &crate::run_apply_error(
                        "routed_review_failed",
                        &error.to_string(),
                        false,
                        None,
                    ),
                )?;
                finish_local_run_unlocked(
                    store,
                    catalog,
                    &admitted[1],
                    data_dir,
                    "failed",
                    Some(0),
                )?;
            }
            Err(error).context("parallel sibling Review failed after both passed attempts")
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep reviewed authority and store boundaries explicit"
)]
pub(crate) fn run_one_local_fixture(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    qualified: Vec<Vec<QualifiedFixture>>,
    advisor: Option<Arc<dyn Advisor>>,
    hosted_client: Option<Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>>,
) -> Result<RunId> {
    if qualified.len() != mission.tasks.len() || !(1..=2).contains(&qualified.len()) {
        bail!("local fixture qualification waves differ from the reviewed mission");
    }
    if crate::flow::routed_parallel_siblings(plan) {
        if hosted_client.is_some() {
            bail!("hosted Shadow experiment supports exactly one task");
        }
        return run_two_sibling_fixtures(
            store, catalog, mission, plan, cfg, repo_root, data_dir, qualified, advisor,
        );
    }
    let mut completed = None;
    let last = qualified.len() - 1;
    for (task_index, observed) in qualified.into_iter().enumerate() {
        let mut repair_round = 0;
        let wave = loop {
            let wave = run_local_fixture_wave(
                store,
                catalog,
                mission,
                plan,
                cfg,
                repo_root,
                data_dir,
                task_index,
                task_index == last,
                observed.clone(),
                advisor.clone(),
                hosted_client.clone(),
            );
            if wave.is_err() || task_index != last || mission.tasks.len() != 1 {
                break wave;
            }
            let run_id = wave.as_ref().expect("checked successful wave");
            if store
                .get_run_status(&run_id.0)?
                .context("routed run disappeared after repairable attempt")?
                .0
                != "starting"
            {
                break wave;
            }
            let scope = RoutingScope {
                domain_id: mission.authorization.domain_id.clone(),
                run_id: mission.authorization.run_id.clone(),
            };
            let history = store
                .routing_history(&scope)?
                .context("routed mission disappeared after repairable attempt")?;
            let task = &history.tasks[task_index];
            if history.cancelled
                || task.next_ordinal != 2
                || task.state != pytxo_store::routing::TaskRoutingState::Ready
            {
                break wave;
            }
            if repair_round != 0 {
                bail!("routed fixture exceeded one strong repair attempt");
            }
            #[cfg(feature = "routed-test-faults")]
            if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_REPAIRABLE_FAILURE").is_some() {
                panic!("injected controller death after settled repairable failure");
            }
            repair_round += 1;
        };
        match wave {
            Ok(run_id) => {
                if task_index < last {
                    let status = store
                        .get_run_status(&run_id.0)?
                        .context("parent wave run disappeared")?
                        .0;
                    if status != "starting" {
                        return Ok(run_id);
                    }
                    let scope = RoutingScope {
                        domain_id: mission.authorization.domain_id.clone(),
                        run_id: mission.authorization.run_id.clone(),
                    };
                    let history = store
                        .routing_history(&scope)?
                        .context("parent wave mission disappeared")?;
                    let parent_id = &mission.tasks[task_index].contract.task_id;
                    let parent = history
                        .tasks
                        .iter()
                        .find(|task| task.registration.contract.task_id == *parent_id)
                        .context("reviewed parent task disappeared")?;
                    let winner = parent
                        .winner
                        .as_ref()
                        .context("parent has no passed winner")?;
                    let attempt = history
                        .attempts
                        .iter()
                        .find(|attempt| attempt.attempt_id == winner.winning_attempt_id)
                        .context("parent winner attempt disappeared")?;
                    if history.cancelled
                        || attempt.state != AttemptState::Passed
                        || !attempt.ownership_released
                        || attempt.task_id != *parent_id
                        || attempt.receipts.sealed_output.as_ref() != Some(&winner.output_digest)
                        || attempt.receipts.checks.as_ref()
                            != Some(&winner.verification_receipt_digest)
                    {
                        bail!("parent wave is not a settled passed winner");
                    }
                    #[cfg(feature = "routed-test-faults")]
                    if std::env::var("PYTXO_TEST_ROUTED_POWERSHELL_HANDOFF")
                        .ok()
                        .as_deref()
                        == Some("1")
                    {
                        let original = store
                            .routed_worktree_instance(
                                &attempt.agent_id,
                                &scope.run_id.0,
                                &parent_id.0,
                            )?
                            .context("cross-adapter fixture lost producer worktree")?;
                        let seed = Path::new(&original.path).join("seed.txt");
                        if std::fs::read(&seed)? != b"pytxo-routed\r\n" {
                            bail!("cross-adapter fixture producer bytes were not sealed first");
                        }
                        // The child must read Store's retained winner, not a mutable
                        // producer workspace that changed after settlement.
                        std::fs::write(&seed, b"untrusted-worker-mutation\r\n")?;
                    }
                    #[cfg(feature = "routed-test-faults")]
                    if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_PARENT_PASS").is_some() {
                        panic!("injected controller death after settled parent wave");
                    }
                    #[cfg(feature = "routed-test-faults")]
                    if std::env::var_os("PYTXO_TEST_ROUTED_STOP_BETWEEN_WAVES").is_some() {
                        let _gate = ActiveRunGate::acquire(&data_dir.join("active_run.json"))?;
                        store.cancel_routing_mission(
                            &scope,
                            "controller.stop.cancel.v1",
                            history.cancel_epoch,
                            now_ms()?,
                        )?;
                        ProcessRegistryFile::update(&registry_path(data_dir), |registry| {
                            if !registry.cancelled_runs.contains(&run_id.0) {
                                registry.cancelled_runs.push(run_id.0.clone());
                            }
                            Ok(())
                        })?;
                    }
                }
                completed = Some(run_id);
            }
            Err(error) => {
                if task_index > 0 || repair_round > 0 {
                    settle_between_wave_admission_failure(
                        store, catalog, mission, plan, repo_root, data_dir,
                    )
                    .with_context(|| {
                        format!("child wave recovery ownership is unresolved: {error:#}")
                    })?;
                }
                return Err(error);
            }
        }
    }
    completed.context("routed local fixture had no reviewed wave")
}

/// The real subscription route stays one task. A separately reviewed repair
/// policy may admit one strong clean restart after an owned frozen check fails.
#[expect(
    clippy::too_many_arguments,
    reason = "keep reviewed authority and selected native observations explicit"
)]
pub(crate) fn run_one_claude_proposal(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    qualified: Vec<QualifiedFixture>,
    hosted_client: Option<Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>>,
) -> Result<RunId> {
    if mission.tasks.len() != 1 || qualified.len() != 2 {
        bail!("Claude proposal route is outside the reviewed one-task pair");
    }
    if load_reviewed_staged_mission(store, plan)? != *mission {
        bail!("Claude proposal review changed before admission");
    }
    let first = admit_one_claude_proposal(
        store,
        catalog,
        mission,
        plan,
        cfg,
        repo_root,
        data_dir,
        qualified.clone(),
        hosted_client,
    )?;
    let first_result = execute_admitted_fixture(
        store, catalog, mission, plan, cfg, repo_root, data_dir, 0, true, false, None, first,
    )?;
    if mission.authorization.limits.max_attempts != 2 {
        return Ok(first_result);
    }
    let scope = RoutingScope {
        domain_id: mission.authorization.domain_id.clone(),
        run_id: mission.authorization.run_id.clone(),
    };
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_STOP_AFTER_REPAIRABLE_FAILURE").is_some()
        && store
            .get_run_status(&scope.run_id.0)?
            .is_some_and(|status| status.0 == "starting")
    {
        let _gate = ActiveRunGate::acquire(&data_dir.join("active_run.json"))?;
        let before_stop = store
            .routing_history(&scope)?
            .context("Claude repair fault lost its first attempt")?;
        store.cancel_routing_mission(
            &scope,
            "controller.stop.cancel.v1",
            before_stop.cancel_epoch,
            now_ms()?,
        )?;
        ProcessRegistryFile::update(&registry_path(data_dir), |registry| {
            if !registry.cancelled_runs.contains(&scope.run_id.0) {
                registry.cancelled_runs.push(scope.run_id.0.clone());
            }
            Ok(())
        })?;
    }
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_CATALOG_STOP_AFTER_REPAIRABLE_FAILURE").is_some()
        && store
            .get_run_status(&scope.run_id.0)?
            .is_some_and(|status| status.0 == "starting")
    {
        let saved = catalog
            .get_flow_draft(&plan.draft_id)?
            .context("Claude repair fault lost its reviewed Flow")?;
        let exact_plan = saved
            .plan_json
            .context("Claude repair fault lost its saved plan")?;
        if !catalog.request_routed_flow_stop(&plan.draft_id, &exact_plan, &scope.run_id.0)? {
            bail!("Claude repair fault could not stage exact Catalog Stop");
        }
    }
    let history = store
        .routing_history(&scope)?
        .context("Claude repair mission disappeared after first attempt")?;
    let repair_ready = !history.cancelled
        && history.tasks.len() == 1
        && history.tasks[0].state == pytxo_store::routing::TaskRoutingState::Ready
        && history.tasks[0].next_ordinal == 2
        && history.attempts.len() == 1
        && history.attempts[0].state == AttemptState::Failed
        && history.attempts[0].ownership_released
        && history.attempts[0].selected.profile.id == mission.policy.everyday.profile_id
        && history.attempts[0].failure.as_ref().is_some_and(|failure| {
            failure.failure_class == AttemptFailureClass::Check
                && failure.actionable_evidence_digest.is_some()
        });
    if !repair_ready {
        if history.cancelled
            && store
                .get_run_status(&scope.run_id.0)?
                .is_some_and(|status| status.0 == "starting")
        {
            settle_between_wave_admission_failure(
                store, catalog, mission, plan, repo_root, data_dir,
            )?;
        }
        return Ok(first_result);
    }
    if store
        .get_run_status(&scope.run_id.0)?
        .context("Claude repair run disappeared")?
        .0
        != "starting"
    {
        bail!("Claude repair task is ready after a terminal run");
    }
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_REPAIRABLE_FAILURE").is_some() {
        panic!("injected controller death after settled Claude check failure");
    }
    let second = match admit_one_claude_proposal(
        store, catalog, mission, plan, cfg, repo_root, data_dir, qualified, None,
    ) {
        Ok(second) => second,
        Err(error) => {
            settle_between_wave_admission_failure(
                store, catalog, mission, plan, repo_root, data_dir,
            )
            .with_context(|| format!("Claude repair admission left unresolved work: {error:#}"))?;
            return Err(error);
        }
    };
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_CLAUDE_REPAIR_ADMITTED").is_some() {
        panic!("injected controller death after Claude strong repair admission");
    }
    execute_admitted_fixture(
        store, catalog, mission, plan, cfg, repo_root, data_dir, 0, true, false, None, second,
    )
}

/// If a child cannot even be admitted, close only a quiescent run. Any
/// ambiguous owner or capacity intent remains under startup recovery instead.
fn settle_between_wave_admission_failure(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    repo_root: &Path,
    data_dir: &Path,
) -> Result<()> {
    let run_id = &mission.authorization.run_id.0;
    let active_path = data_dir.join("active_run.json");
    let _gate = ActiveRunGate::acquire(&active_path)?;
    if store
        .get_run_status(run_id)?
        .as_ref()
        .map(|status| status.0.as_str())
        != Some("starting")
    {
        return Ok(());
    }
    if store
        .unresolved_routing_attempts()?
        .iter()
        .any(|attempt| attempt.scope.run_id.0 == *run_id)
        || store
            .unresolved_capacity_intent_scopes()?
            .iter()
            .any(|scope| scope.run_id.0 == *run_id)
    {
        bail!("child admission left unresolved owned work");
    }
    let scope = RoutingScope {
        domain_id: mission.authorization.domain_id.clone(),
        run_id: mission.authorization.run_id.clone(),
    };
    let history = store
        .routing_history(&scope)?
        .context("between-wave mission disappeared")?;
    if !store
        .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))?
        .is_empty()
        || catalog
            .unresolved_capacity_reservations()?
            .iter()
            .any(|reservation| {
                reservation.domain_id == scope.domain_id.0 && reservation.run_id == *run_id
            })
        || !crate::flow::registry_matches_settled_routed_owners(
            &ProcessRegistryFile::load(&registry_path(data_dir))?,
            store,
            &history,
            &scope,
            repo_root,
        )?
    {
        bail!("between-wave terminal ownership or registry remains unresolved");
    }
    let stopped = crate::routed_fixture::routed_stop_requested(catalog, &plan.draft_id, run_id)?
        || ProcessRegistryFile::load(&registry_path(data_dir))?
            .cancelled_runs
            .contains(run_id)
        || history
            .events
            .iter()
            .any(|entry| entry.event_id == "controller.stop.cancel.v1");
    if !history.cancelled {
        store.cancel_routing_mission(
            &scope,
            &format!("{}:child-admission-failure-fence", run_id),
            history.cancel_epoch,
            now_ms()?,
        )?;
    }
    let status = if stopped { "cancelled" } else { "failed" };
    if !store.finish_run_if_status(run_id, "starting", status)? {
        bail!("between-wave run changed before terminal settlement");
    }
    ProcessRegistryFile::update(&registry_path(data_dir), |registry| {
        registry.remove_run(run_id);
        Ok(())
    })?;
    super::clear_active_run_unlocked(&active_path, run_id)?;
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep reviewed authority and store boundaries explicit"
)]
fn run_local_fixture_wave(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    task_index: usize,
    final_wave: bool,
    qualified: Vec<QualifiedFixture>,
    advisor: Option<Arc<dyn Advisor>>,
    hosted_client: Option<Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>>,
) -> Result<RunId> {
    let reviewed = load_reviewed_staged_mission(store, plan)?;
    if reviewed != *mission {
        bail!("routed mission changed before admission");
    }
    let admitted = admit_one_local_fixture(
        store,
        catalog,
        mission,
        plan,
        cfg,
        repo_root,
        data_dir,
        task_index,
        qualified,
        advisor,
        hosted_client,
    )?;
    #[cfg(feature = "routed-test-faults")]
    if mission.tasks.len() == 1
        && task_index == 0
        && std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_ONE_ADMITTED").is_some()
    {
        panic!("injected controller death after one prepared routed attempt");
    }
    execute_admitted_fixture(
        store, catalog, mission, plan, cfg, repo_root, data_dir, task_index, final_wave, false,
        None, admitted,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep reviewed authority and native attempt ownership explicit"
)]
fn execute_admitted_fixture(
    store: &PytxoStore,
    catalog: &Catalog,
    mission: &RoutingMission,
    plan: &FlowPlan,
    cfg: &PytxoConfig,
    repo_root: &Path,
    data_dir: &Path,
    task_index: usize,
    final_wave: bool,
    sibling_mode: bool,
    preparation_gate: Option<&std::sync::Mutex<()>>,
    admitted: AdmittedFixture,
) -> Result<RunId> {
    let mut worker_exit = None;
    let execution = (|| -> Result<RunId> {
        if let Some(preparation_gate) = preparation_gate {
            let _serial = preparation_gate
                .lock()
                .map_err(|_| anyhow::anyhow!("parallel worktree preparation lock poisoned"))?;
            prepare_admitted_worktree(store, repo_root, &admitted)?;
        } else {
            prepare_admitted_worktree(store, repo_root, &admitted)?;
        }
        if load_reviewed_staged_mission(store, plan)? != *mission {
            bail!("routed mission changed after admission");
        }
        let task = &mission.tasks[task_index];
        let worker_result = if let Some(hosted) = &admitted.selected.hosted {
            let admitted_prompt = if hosted.proposal_template.is_some() {
                crate::routed_prompt::render_admitted_claude_proposal_prompt(
                    store,
                    &admitted.scope,
                    &admitted.attempt_id,
                    repo_root,
                    &admitted.inputs,
                )?
            } else {
                crate::routed_prompt::render_admitted_attempt_prompt(
                    store,
                    &admitted.scope,
                    &admitted.attempt_id,
                )?
            };
            if admitted_prompt != hosted.prompt {
                bail!("hosted prompt changed after Store admission");
            }
            let prompt_digest = pytxo_core::routing::Digest::of_bytes(&admitted_prompt);
            let spec = if let Some(template) = &hosted.proposal_template {
                if template.spec.stdin != admitted_prompt
                    || template.spec.working_directory != admitted.worktree
                    || template.spec.arguments != admitted.worker_arguments
                {
                    bail!("Claude proposal launch changed after admission");
                }
                template.spec.clone()
            } else {
                OwnedLaunchSpec {
                    bootstrap_host: hosted.bootstrap_host.clone(),
                    executable: admitted.selected.executable.clone(),
                    dependencies: vec![],
                    arguments: admitted.worker_arguments.clone(),
                    environment: BTreeMap::new(),
                    working_directory: admitted.worktree.clone(),
                    stdin: admitted_prompt,
                    transport: OwnedTransport::Subprocess,
                    barrier_timeout: Duration::from_secs(5),
                    execution_timeout: fixture_worker_timeout(),
                    settlement_timeout: Duration::from_secs(5),
                    output_limit: 4096,
                }
            };
            let worker = RoutedHostedWorker {
                store,
                catalog,
                plan,
                repo_root,
                data_dir,
                worktree: &admitted.worktree,
                inputs: &admitted.inputs,
                scope: admitted.scope.clone(),
                task_id: admitted.task_id.clone(),
                attempt_id: admitted.attempt_id.clone(),
                facts: admitted.facts.clone(),
                spec,
            };
            let result = run_prepared_hosted_owned_worker(&worker);
            if hosted.proposal_template.is_none() {
                if let Ok(receipt) = &result {
                    let attestation = format!("synthetic-stdin-sha256:{}", prompt_digest.0);
                    if receipt.outcome == OwnedOutcome::Succeeded
                        && !receipt
                            .stdout
                            .lines()
                            .any(|line| line.trim() == attestation)
                    {
                        bail!(
                            "synthetic hosted payload did not consume the admitted private stdin"
                        );
                    }
                }
            }
            result
        } else {
            let worker = RoutedOwnedWorker {
                store,
                catalog,
                plan,
                repo_root,
                data_dir,
                worktree: &admitted.worktree,
                inputs: &admitted.inputs,
                scope: admitted.scope.clone(),
                task_id: admitted.task_id.clone(),
                attempt_id: admitted.attempt_id.clone(),
                facts: admitted.facts.clone(),
                spec: DirectOwnedLaunchSpec {
                    executable: admitted.selected.executable.clone(),
                    arguments: admitted.worker_arguments.clone(),
                    windows_cmd_verbatim_tail: false,
                    environment: BTreeMap::new(),
                    working_directory: admitted.worktree.clone(),
                    execution_timeout: fixture_worker_timeout(),
                    settlement_timeout: Duration::from_secs(5),
                    output_limit: 4096,
                },
            };
            run_prepared_owned_worker(&worker)
        };
        #[cfg(feature = "routed-test-faults")]
        trace_fixture_launch("worker", task_index, &worker_result);
        let worker_receipt = worker_result?;
        worker_exit = worker_receipt.exit_code;
        if !worker_receipt.process_registered || worker_receipt.active_processes != Some(0) {
            bail!("routed worker native ownership remains unresolved");
        }
        let owner = store
            .launch_ownership(&admitted.attempt_id)?
            .context("routed worker owner absent after native settlement")?;
        let quiescence = owner
            .settlement_blob
            .context("worker quiescence receipt absent")?;
        let current = current_attempt(store, &admitted)?;
        let sealing = transition(
            store,
            &admitted,
            &current,
            AttemptState::Sealing,
            RoutingReceipts {
                quiescence: Some(quiescence.digest.clone()),
                ..Default::default()
            },
        )?;
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_JOB_ZERO").is_some() {
            bail!("injected post-quiescence failure");
        }
        #[allow(unused_mut)]
        let mut worker_succeeded = worker_receipt.outcome == OwnedOutcome::Succeeded;
        #[cfg(feature = "routed-test-faults")]
        if task_index == 0
            && std::env::var_os("PYTXO_TEST_ROUTED_FORCE_PARENT_WORKER_FAILURE").is_some()
        {
            worker_succeeded = false;
        }
        if !worker_succeeded {
            let deliberate_repair = mission.tasks.len() == 1
                && task.contract.goal == "pytxo-local-fixture-v1:write-result-with-repair"
                && sealing.ordinal == 1
                && admitted.selected.observation.requested_model.model == "everyday"
                && worker_receipt.outcome == OwnedOutcome::Failed
                && worker_receipt.exit_code == Some(7)
                && worker_receipt.payload_exit_code == Some(7)
                && worker_receipt.output_complete
                && !worker_receipt.output_truncated
                && worker_receipt.error.is_none();
            if deliberate_repair {
                let evidence_digest = canonical_digest(
                    &(
                        1_u32,
                        "bounded-native-worker-exit",
                        &admitted.scope,
                        &admitted.attempt_id,
                        &quiescence.digest,
                        worker_receipt.exit_code,
                        worker_receipt.payload_exit_code,
                    ),
                    1,
                )?;
                fail_actionable_attempt(
                    store,
                    &admitted,
                    &sealing,
                    AttemptFailureClass::Implementation,
                    evidence_digest,
                )?;
                release_terminal(store, catalog, &admitted, data_dir)?;
                let settled = settle_local_attempt(
                    store,
                    catalog,
                    &admitted,
                    data_dir,
                    "retryable_failed",
                    worker_receipt.exit_code,
                )?;
                if settled == "cancelled" {
                    return finish_local_run(
                        store,
                        catalog,
                        &admitted,
                        data_dir,
                        "cancelled",
                        worker_receipt.exit_code,
                    );
                }
                return Ok(admitted.scope.run_id.clone());
            }
            transition(
                store,
                &admitted,
                &sealing,
                AttemptState::Failed,
                RoutingReceipts::default(),
            )?;
            release_terminal(store, catalog, &admitted, data_dir)?;
            if sibling_mode {
                settle_local_attempt(
                    store,
                    catalog,
                    &admitted,
                    data_dir,
                    "failed",
                    worker_receipt.exit_code,
                )?;
                return Ok(admitted.scope.run_id.clone());
            }
            return finish_local_run(
                store,
                catalog,
                &admitted,
                data_dir,
                "failed",
                worker_receipt.exit_code,
            );
        }
        let proposal = admitted
            .selected
            .hosted
            .as_ref()
            .and_then(|hosted| hosted.proposal_template.as_ref());
        let snapshot = if let Some(template) = proposal {
            if inherited_snapshot_for_attempt(store, &sealing)?.is_some() {
                bail!("first Claude proposal cannot inherit a dependency snapshot");
            }
            let [claim] = task.contract.claim_roots.as_slice() else {
                bail!("Claude proposal lost its single reviewed claim");
            };
            let proposed = crate::routed_claude::parse_owned_claude_one_file_proposal(
                template,
                &worker_receipt,
            );
            #[cfg(feature = "routed-test-faults")]
            if std::env::var_os("PYTXO_TEST_ROUTED_DIAGNOSTICS").is_some() {
                if let Err(error) = &proposed {
                    eprintln!("routed_claude proposal refused: {error:#}");
                }
            }
            let proposed = proposed?;
            seal_one_existing_claimed_text_proposal(
                repo_root,
                &admitted.worktree,
                &admitted.inputs,
                claim,
                &proposed,
            )?
        } else {
            match inherited_snapshot_for_attempt(store, &sealing)? {
                Some(inherited) => capture_sealed_output_after_dependency(
                    repo_root,
                    &admitted.worktree,
                    &admitted.inputs,
                    &inherited,
                    &task.contract.claim_roots,
                )?,
                None => capture_sealed_output(
                    repo_root,
                    &admitted.worktree,
                    &admitted.inputs,
                    &task.contract.claim_roots,
                )?,
            }
        };
        let output_path = task
            .contract
            .claim_roots
            .first()
            .context("bounded local fixture has no claimed output")?;
        if proposal.is_none()
            && !snapshot.files.iter().any(|file| {
                file.path == *output_path
                    && file.byte_length == b"pytxo-routed\r\n".len() as u64
                    && file.content_digest
                        == pytxo_core::routing::Digest::of_bytes(b"pytxo-routed\r\n")
            })
        {
            bail!("routed local fixture result bytes differ from the bounded action");
        }
        let output_claim = claim(
            &admitted,
            PrivateArtifactKind::ScopedOutput,
            "sealed-output",
        );
        let output_ref =
            store.put_private_artifact(&output_claim, &serde_json::to_vec(&snapshot)?)?;
        let retained: pytxo_runner::SealedOutputSnapshot =
            serde_json::from_slice(&store.read_private_artifact(&output_claim, &output_ref)?)?;
        if retained != snapshot {
            bail!("routed sealed output bytes changed after retention");
        }
        let verifying = transition(
            store,
            &admitted,
            &sealing,
            AttemptState::Verifying,
            RoutingReceipts {
                sealed_output: Some(output_ref.digest.clone()),
                ..Default::default()
            },
        )?;
        let recipe = &task.check_recipes[0];
        let shell = PinnedFile {
            path: recipe.executor.shell.path.clone().into(),
            sha256: recipe.executor.shell.digest.0.clone(),
        };
        #[cfg(feature = "routed-test-faults")]
        let test_powershell_worker = sealing.selected.profile.harness_id
            == "pytxo-local-fixture-powershell-v1"
            && admitted
                .selected
                .executable
                .path
                .file_name()
                .is_some_and(|name| {
                    name.to_string_lossy()
                        .eq_ignore_ascii_case("powershell.exe")
                });
        #[cfg(not(feature = "routed-test-faults"))]
        let test_powershell_worker = false;
        if PinnedFile::observe(shell.path.clone())? != shell
            || (admitted.selected.hosted.is_none()
                && shell != admitted.selected.executable
                && !test_powershell_worker)
            || recipe.executor.shell_args != ["/D", "/C"]
            || (proposal.is_none()
                && recipe.command != format!("if exist {output_path} (exit /b 0) else (exit /b 1)"))
        {
            bail!("frozen routed checker executable or built-in command changed");
        }
        let view = create_plain_verification_view(
            data_dir,
            &admitted.scope.run_id.0,
            &admitted.attempt_id.0,
        )?;
        materialize_sealed_output(&view, &retained)?;
        let request = CheckerOwnershipRequest {
            scope: admitted.scope.clone(),
            task_id: admitted.task_id.clone(),
            attempt_id: admitted.attempt_id.clone(),
            reservation_id: admitted.reservation_id.clone(),
            launch_token: admitted.launch_token.clone(),
            check_id: recipe.id.clone(),
            ordinal: recipe.ordinal,
            recipe_digest: recipe.reference()?.recipe_digest,
            sealed_view: output_ref,
            prepare_event_id: format!("{}:checker-prepared", admitted.attempt_id.0),
            now_ms: now_ms()?.max(verifying.updated_at_ms),
        };
        store.prepare_checker_ownership(catalog, &request)?;
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_PREPARED").is_some() {
            let history = store
                .routing_history(&admitted.scope)?
                .context("injected Stop lost routed mission")?;
            store.cancel_routing_mission(
                &admitted.scope,
                "controller.stop.cancel.v1",
                history.cancel_epoch,
                now_ms()?,
            )?;
            bail!("injected Stop after checker prepare");
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_CHECKER_PREPARED").is_some() {
            bail!("injected prepared-checker failure");
        }
        let sealed_ref = request.sealed_view.clone();
        let checker = RoutedOwnedChecker {
            store,
            catalog,
            plan,
            repo_root,
            data_dir,
            view_root: &view,
            snapshot: &retained,
            sealed_ref: &sealed_ref,
            request,
            spec: DirectOwnedLaunchSpec {
                executable: shell,
                arguments: if proposal.is_some() {
                    vec!["/D".into(), "/C".into(), recipe.command.clone()]
                } else {
                    vec![
                        "/D".into(),
                        "/C".into(),
                        "if".into(),
                        "exist".into(),
                        output_path.clone(),
                        "(exit".into(),
                        "/b".into(),
                        "0)".into(),
                        "else".into(),
                        "(exit".into(),
                        "/b".into(),
                        "1)".into(),
                    ]
                },
                windows_cmd_verbatim_tail: proposal.is_some(),
                environment: BTreeMap::new(),
                working_directory: view.clone(),
                execution_timeout: Duration::from_millis(recipe.executor.timeout_ms),
                settlement_timeout: Duration::from_secs(5),
                output_limit: usize::try_from(
                    recipe
                        .executor
                        .max_stdout_bytes
                        .max(recipe.executor.max_stderr_bytes),
                )?,
            },
        };
        let checker_result = run_prepared_owned_checker(&checker);
        #[cfg(feature = "routed-test-faults")]
        trace_fixture_launch("checker", task_index, &checker_result);
        let checker_receipt = checker_result?;
        if !checker_receipt.process_registered || checker_receipt.active_processes != Some(0) {
            bail!("routed checker native ownership remains unresolved");
        }
        let checked = store
            .checker_ownership(&admitted.attempt_id, 1)?
            .context("routed checker owner disappeared")?;
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_DIAGNOSTICS").is_some() {
            eprintln!(
                "routed_fixture stage=checker_verdict task_index={task_index} passed={:?}",
                checked.passed,
            );
        }
        if checker_receipt.outcome != OwnedOutcome::Succeeded || checked.passed != Some(true) {
            let latest = current_attempt(store, &admitted)?;
            let trusted_failed_verdict = if let Some(reference) = &checked.settlement_blob {
                let receipt =
                    store.read_controller_receipt(&checker_claim(&checked.request), reference)?;
                receipt.source == ReceiptSource::OwnedJobObservation
                    && receipt.scope == admitted.scope
                    && receipt.task_id == admitted.task_id
                    && receipt.attempt_id == admitted.attempt_id
                    && receipt.reservation_id == admitted.reservation_id
                    && matches!(
                        receipt.observation,
                        ControllerObservation::NativeCheckerResult {
                            check_id,
                            ordinal,
                            job_name,
                            launch_nonce,
                            active_processes: 0,
                            exit_code,
                            payload_exit_code,
                            process_registered: true,
                            barrier_released: true,
                            native_outcome: CheckerNativeOutcome::Failed,
                            stdout_complete: true,
                            stderr_complete: true,
                            output_truncated: false,
                            error_present: false,
                            sealed_view_before,
                            sealed_view_after: Some(sealed_view_after),
                            ..
                        } if check_id == recipe.id
                            && ordinal == recipe.ordinal
                            && checked.job_name.as_deref() == Some(job_name.as_str())
                            && checked.launch_nonce.as_deref() == Some(launch_nonce.as_str())
                            && exit_code > 0
                            && payload_exit_code == u32::try_from(exit_code).ok()
                            && i32::try_from(checker_receipt.exit_code.unwrap_or_default()).ok()
                                == Some(exit_code)
                            && payload_exit_code == checker_receipt.payload_exit_code
                            && sealed_view_before == sealed_ref
                            && sealed_view_after == sealed_ref
                    )
            } else {
                false
            };
            let actionable_claude_check = proposal.is_some()
                && mission.authorization.limits.max_attempts == 2
                && latest.ordinal == 1
                && latest.selected.profile.id == mission.policy.everyday.profile_id
                && latest.state == AttemptState::Verifying
                && checker_receipt.outcome == OwnedOutcome::Failed
                && checker_receipt.exit_code.is_some_and(|code| code != 0)
                && checker_receipt.payload_exit_code == checker_receipt.exit_code
                && checker_receipt.output_complete
                && !checker_receipt.output_truncated
                && checker_receipt.error.is_none()
                && checked.phase == CheckerOwnershipPhase::Settled
                && checked.passed == Some(false)
                && checked.request.scope == admitted.scope
                && checked.request.task_id == admitted.task_id
                && checked.request.attempt_id == admitted.attempt_id
                && checked.request.check_id == recipe.id
                && checked.request.ordinal == recipe.ordinal
                && checked.request.sealed_view == sealed_ref
                && trusted_failed_verdict
                && !user_stop_recorded(store, catalog, &admitted, data_dir)?;
            if actionable_claude_check {
                let evidence_digest = canonical_digest(
                    &(
                        1_u32,
                        "claude-owned-frozen-check-failed",
                        &admitted.scope,
                        &admitted.attempt_id,
                        &quiescence.digest,
                        &sealed_ref.digest,
                        &checked
                            .settlement_blob
                            .as_ref()
                            .context("settled checker lost its receipt")?
                            .digest,
                        checker_receipt.exit_code,
                    ),
                    1,
                )?;
                fail_actionable_attempt(
                    store,
                    &admitted,
                    &latest,
                    AttemptFailureClass::Check,
                    evidence_digest,
                )?;
                release_terminal(store, catalog, &admitted, data_dir)?;
                let settled = settle_local_attempt(
                    store,
                    catalog,
                    &admitted,
                    data_dir,
                    "retryable_failed",
                    worker_receipt.exit_code,
                )?;
                if settled == "cancelled" {
                    return finish_local_run(
                        store,
                        catalog,
                        &admitted,
                        data_dir,
                        "cancelled",
                        worker_receipt.exit_code,
                    );
                }
                return Ok(admitted.scope.run_id.clone());
            }
            transition(
                store,
                &admitted,
                &latest,
                AttemptState::Failed,
                RoutingReceipts::default(),
            )?;
            release_terminal(store, catalog, &admitted, data_dir)?;
            if sibling_mode {
                settle_local_attempt(
                    store,
                    catalog,
                    &admitted,
                    data_dir,
                    "failed",
                    worker_receipt.exit_code,
                )?;
                return Ok(admitted.scope.run_id.clone());
            }
            return finish_local_run(
                store,
                catalog,
                &admitted,
                data_dir,
                "failed",
                worker_receipt.exit_code,
            );
        }
        let checks = store.owned_checker_pass_digest(&admitted.attempt_id)?;
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_JOB_ZERO").is_some() {
            let history = store
                .routing_history(&admitted.scope)?
                .context("injected Stop lost routed mission")?;
            store.cancel_routing_mission(
                &admitted.scope,
                "controller.stop.cancel.v1",
                history.cancel_epoch,
                now_ms()?,
            )?;
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_CATALOG_STOP_AFTER_CHECKER_JOB_ZERO").is_some() {
            let saved = catalog
                .get_flow_draft(&plan.draft_id)?
                .context("injected Catalog Stop lost reviewed Flow")?;
            let plan_json = saved
                .plan_json
                .context("injected Catalog Stop lost reviewed plan")?;
            if !catalog.request_routed_flow_stop(
                &plan.draft_id,
                &plan_json,
                &admitted.scope.run_id.0,
            )? {
                bail!("injected Catalog Stop did not match the reviewed run");
            }
        }
        let active_path = data_dir.join("active_run.json");
        let gate = ActiveRunGate::acquire(&active_path)?;
        if crate::routed_fixture::routed_stop_requested(
            catalog,
            &plan.draft_id,
            &admitted.scope.run_id.0,
        )? || ProcessRegistryFile::load(&registry_path(data_dir))?
            .cancelled_runs
            .contains(&admitted.scope.run_id.0)
            || store
                .routing_history(&admitted.scope)?
                .context("routed mission disappeared before winner publication")?
                .cancelled
            || load_reviewed_staged_mission(store, plan)? != *mission
        {
            bail!("routed winner authority changed before publication");
        }
        let latest = current_attempt(store, &admitted)?;
        transition(
            store,
            &admitted,
            &latest,
            AttemptState::Passed,
            RoutingReceipts {
                checks: Some(checks),
                ..Default::default()
            },
        )?;
        release_terminal_unlocked(store, catalog, &admitted)?;
        if !final_wave {
            if settle_local_attempt(
                store,
                catalog,
                &admitted,
                data_dir,
                "completed",
                worker_receipt.exit_code,
            )? != "completed"
            {
                bail!("routed parent wave was cancelled before child admission");
            }
            drop(gate);
            return Ok(admitted.scope.run_id.clone());
        }
        drop(gate);
        #[allow(unused_mut)]
        let mut manifest = prepare_winning_review(
            store,
            &admitted,
            mission,
            plan,
            cfg,
            repo_root,
            data_dir,
            &output_claim,
            &sealed_ref,
            &retained,
        )?;
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_MUTATE_MANIFEST_BEFORE_PUBLICATION").is_some() {
            manifest
                .files
                .first_mut()
                .context("injected manifest mutation has no output")?
                .after_sha256 = Some("0".repeat(64));
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_EMPTY_CANDIDATE_CHECKS_BEFORE_PUBLICATION").is_some()
        {
            manifest
                .candidate_verification
                .as_mut()
                .context("injected candidate has no verification evidence")?
                .checks
                .clear();
            manifest.package_digest = pytxo_core::prepared_manifest_digest(&manifest)?;
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_WRONG_CANDIDATE_COMMAND_BEFORE_PUBLICATION")
            .is_some()
        {
            manifest
                .candidate_verification
                .as_mut()
                .context("injected candidate has no verification evidence")?
                .checks
                .first_mut()
                .context("injected candidate has no check")?
                .command = "different-approved-command".into();
            manifest.package_digest = pytxo_core::prepared_manifest_digest(&manifest)?;
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_WRONG_CANDIDATE_MODE_BEFORE_PUBLICATION").is_some() {
            manifest
                .files
                .first_mut()
                .context("injected candidate has no file")?
                .after_mode = Some(0o755);
            manifest.package_digest = pytxo_core::prepared_manifest_digest(&manifest)?;
        }
        let gate = ActiveRunGate::acquire(&active_path)?;
        if ProcessRegistryFile::load(&registry_path(data_dir))?
            .cancelled_runs
            .contains(&admitted.scope.run_id.0)
            || store
                .routing_history(&admitted.scope)?
                .context("routed mission disappeared before Review publication")?
                .cancelled
            || load_reviewed_staged_mission(store, plan)? != *mission
        {
            bail!("routed Review authority changed before publication");
        }
        if settle_local_attempt(
            store,
            catalog,
            &admitted,
            data_dir,
            "completed",
            worker_receipt.exit_code,
        )? != "completed"
        {
            bail!("routed Review was cancelled before publication");
        }
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS").is_some() {
            panic!("injected controller death after settled final wave");
        }
        store.finish_routed_review_and_run(&admitted.scope.run_id.0, &manifest, "starting")?;
        let completed = finish_local_run_unlocked(
            store,
            catalog,
            &admitted,
            data_dir,
            "completed",
            worker_receipt.exit_code,
        );
        drop(gate);
        completed
    })();
    match execution {
        Ok(run_id) => Ok(run_id),
        Err(error) => {
            if current_attempt(store, &admitted)?.state == AttemptState::Passed {
                let gate = ActiveRunGate::acquire(&data_dir.join("active_run.json"))?;
                if !final_wave {
                    let history = store
                        .routing_history(&admitted.scope)?
                        .context("passed routed parent lost its mission")?;
                    if !history.cancelled {
                        store.cancel_routing_mission(
                            &admitted.scope,
                            &format!("{}:between-wave-failure-fence", admitted.attempt_id.0),
                            history.cancel_epoch,
                            now_ms()?,
                        )?;
                    }
                    let settled = if sibling_mode {
                        settle_local_attempt(
                            store,
                            catalog,
                            &admitted,
                            data_dir,
                            "failed",
                            worker_exit,
                        )
                        .map(|_| admitted.scope.run_id.clone())
                    } else {
                        finish_local_run_unlocked(
                            store,
                            catalog,
                            &admitted,
                            data_dir,
                            "failed",
                            worker_exit,
                        )
                    };
                    drop(gate);
                    settled.with_context(|| format!("parent wave settlement failed: {error:#}"))?;
                    return Err(error).context("routed parent wave failed after a passed attempt");
                }
                let contract = store
                    .get_run_contract(&admitted.scope.run_id.0)?
                    .context("passed routed attempt lost its Review contract")?;
                if contract.apply_status == "pending"
                    && !store.begin_run_preparation(&admitted.scope.run_id.0)?
                {
                    bail!("passed routed Review could not enter failure settlement");
                }
                if matches!(contract.apply_status.as_str(), "pending" | "preparing") {
                    let review_error = crate::run_apply_error(
                        "routed_review_failed",
                        &error.to_string(),
                        false,
                        None,
                    );
                    store.fail_run_preparation(&admitted.scope.run_id.0, &review_error)?;
                }
                let settled = finish_local_run_unlocked(
                    store,
                    catalog,
                    &admitted,
                    data_dir,
                    "failed",
                    worker_exit,
                );
                drop(gate);
                settled.with_context(|| format!("routed Review settlement failed: {error:#}"))?;
                return Err(error).context("routed Review failed after a passed attempt");
            }
            let owner = store.launch_ownership(&admitted.attempt_id)?;
            let recovery = match owner.as_ref().map(|record| record.phase) {
                Some(LaunchOwnershipPhase::Prepared | LaunchOwnershipPhase::ClosedNoLaunch) => {
                    reconcile_no_create_failure(store, catalog, &admitted, data_dir, sibling_mode)
                }
                _ => reconcile_quiescent_failure(
                    store,
                    catalog,
                    &admitted,
                    data_dir,
                    worker_exit,
                    sibling_mode,
                ),
            };
            recovery.with_context(|| format!("routed execution failed: {error:#}"))
        }
    }
}

#[cfg(test)]
mod routed_review_snapshot_tests {
    use super::*;
    use base64::Engine;
    use pytxo_core::routing::{BaseSnapshot, Digest};
    use pytxo_core::{
        CandidateCheckEvidence, CandidateInventoryFile, CandidateVerificationEvidence,
        PreparedRunFile, PreparedRunFileKind, PreparedRunManifest, PreparedRunSummary,
    };
    use pytxo_runner::{ReviewedInputFile, ReviewedInputManifest, SealedOutputFile};

    fn digest(bytes: &[u8]) -> String {
        Digest::of_bytes(bytes).0
    }

    fn source_mode(path: &str) -> Option<u32> {
        if cfg!(unix) {
            Some(if path == "a.txt" { 0o664 } else { 0o644 })
        } else {
            None
        }
    }

    fn output_mode() -> Option<u32> {
        if cfg!(unix) {
            Some(0o644)
        } else {
            None
        }
    }

    fn output(path: &str, bytes: &[u8]) -> SealedOutputFile {
        SealedOutputFile {
            path: path.into(),
            content_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            content_digest: Digest::of_bytes(bytes),
            byte_length: bytes.len() as u64,
            mode: output_mode(),
        }
    }

    fn reviewed_file(
        path: &str,
        kind: PreparedRunFileKind,
        before: Option<&[u8]>,
        after: Option<&[u8]>,
    ) -> PreparedRunFile {
        PreparedRunFile {
            path: path.into(),
            kind,
            before_sha256: before.map(digest),
            after_sha256: after.map(digest),
            byte_count: after.or(before).unwrap().len() as u64,
            task_id: "task-1".into(),
            agent_id: "agent-1".into(),
            blob_digest: after.map(digest),
            before_mode: before.and_then(|_| source_mode(path)),
            after_mode: after.and_then(|_| output_mode()),
            before_byte_count: before.map_or(0, |bytes| bytes.len() as u64),
            after_byte_count: after.map_or(0, |bytes| bytes.len() as u64),
            before_is_binary: before.map(|_| false),
            after_is_binary: after.map(|_| false),
            before_chunks: vec![],
            after_chunks: vec![],
        }
    }

    fn contract() -> (
        ReviewedInputManifest,
        SealedOutputSnapshot,
        PreparedRunManifest,
    ) {
        let base = BaseSnapshot {
            repository_identity: "repo".into(),
            git_revision: "a".repeat(40),
            snapshot_digest: Digest::of_bytes(b"base"),
        };
        let input = ReviewedInputManifest {
            schema_version: 1,
            base: base.clone(),
            files: vec![
                ReviewedInputFile {
                    path: "a.txt".into(),
                    git_blob_oid: "a".repeat(40),
                    executable: false,
                    content_digest: Digest::of_bytes(b"old"),
                    byte_length: 3,
                },
                ReviewedInputFile {
                    path: "b.txt".into(),
                    git_blob_oid: "b".repeat(40),
                    executable: false,
                    content_digest: Digest::of_bytes(b"gone"),
                    byte_length: 4,
                },
            ],
        };
        let sealed = SealedOutputSnapshot {
            schema_version: 2,
            base,
            files: vec![output("a.txt", b"new"), output("c.txt", b"added")],
        };
        let review = PreparedRunManifest {
            version: 3,
            run_id: "run-1".into(),
            base_revision: "a".repeat(40),
            prepared_at: "now".into(),
            package_digest: "digest".into(),
            summary: PreparedRunSummary::default(),
            files: vec![
                reviewed_file(
                    "a.txt",
                    PreparedRunFileKind::Modify,
                    Some(b"old"),
                    Some(b"new"),
                ),
                reviewed_file("b.txt", PreparedRunFileKind::Delete, Some(b"gone"), None),
                reviewed_file("c.txt", PreparedRunFileKind::Add, None, Some(b"added")),
            ],
            candidate_verification: Some(CandidateVerificationEvidence {
                version: 1,
                base_inventory: vec![
                    CandidateInventoryFile {
                        path: "a.txt".into(),
                        sha256: digest(b"old"),
                        mode: source_mode("a.txt"),
                    },
                    CandidateInventoryFile {
                        path: "b.txt".into(),
                        sha256: digest(b"gone"),
                        mode: source_mode("b.txt"),
                    },
                ],
                candidate_inventory: vec![
                    CandidateInventoryFile {
                        path: "a.txt".into(),
                        sha256: digest(b"new"),
                        mode: output_mode(),
                    },
                    CandidateInventoryFile {
                        path: "c.txt".into(),
                        sha256: digest(b"added"),
                        mode: output_mode(),
                    },
                ],
                exclusions: vec![".git".into(), ".pytxo".into()],
                checks: vec![CandidateCheckEvidence {
                    task_id: "task-1".into(),
                    command: "check".into(),
                    effective_profile: "orbit".into(),
                    passed: true,
                    enforcement: serde_json::json!({}),
                }],
                verified_at: "now".into(),
            }),
        };
        (input, sealed, review)
    }

    #[test]
    fn one_task_review_accepts_exact_non_fixture_add_modify_and_delete() {
        let (input, sealed, review) = contract();
        verify_one_task_review_matches_sealed(&input, &sealed, &review, "task-1", "agent-1")
            .unwrap();
    }

    #[test]
    fn one_task_review_rejects_missing_extra_or_misattributed_changes() {
        let (input, sealed, review) = contract();
        let mut missing = review.clone();
        missing.files.pop();
        assert!(verify_one_task_review_matches_sealed(
            &input, &sealed, &missing, "task-1", "agent-1"
        )
        .is_err());
        let mut extra = review.clone();
        extra.files.push(reviewed_file(
            "extra.txt",
            PreparedRunFileKind::Add,
            None,
            Some(b"x"),
        ));
        assert!(verify_one_task_review_matches_sealed(
            &input, &sealed, &extra, "task-1", "agent-1"
        )
        .is_err());
        let mut wrong_actor = review.clone();
        wrong_actor.files[0].task_id = "task-2".into();
        assert!(verify_one_task_review_matches_sealed(
            &input,
            &sealed,
            &wrong_actor,
            "task-1",
            "agent-1"
        )
        .is_err());
        let mut wrong_agent = review;
        wrong_agent.files[0].agent_id = "agent-2".into();
        assert!(verify_one_task_review_matches_sealed(
            &input,
            &sealed,
            &wrong_agent,
            "task-1",
            "agent-1"
        )
        .is_err());
    }

    #[test]
    fn one_task_review_rejects_wrong_digest_mode_and_base() {
        let (input, sealed, review) = contract();
        let mut wrong_digest = review.clone();
        wrong_digest.files[0].after_sha256 = Some(digest(b"different"));
        assert!(verify_one_task_review_matches_sealed(
            &input,
            &sealed,
            &wrong_digest,
            "task-1",
            "agent-1"
        )
        .is_err());
        let mut wrong_mode = review.clone();
        wrong_mode.files[0].after_mode = Some(0o755);
        assert!(verify_one_task_review_matches_sealed(
            &input,
            &sealed,
            &wrong_mode,
            "task-1",
            "agent-1"
        )
        .is_err());
        let mut wrong_base = sealed;
        wrong_base.base.git_revision = "b".repeat(40);
        assert!(verify_one_task_review_matches_sealed(
            &input,
            &wrong_base,
            &review,
            "task-1",
            "agent-1"
        )
        .is_err());
    }

    #[test]
    fn one_task_review_keeps_excluded_tracked_bytes_and_mode_unchanged() {
        let (mut input, mut sealed, review) = contract();
        input.files.push(ReviewedInputFile {
            path: "excluded.txt".into(),
            git_blob_oid: "c".repeat(40),
            executable: false,
            content_digest: Digest::of_bytes(b"excluded"),
            byte_length: 8,
        });
        sealed.files.push(output("excluded.txt", b"excluded"));
        verify_one_task_review_matches_sealed(&input, &sealed, &review, "task-1", "agent-1")
            .unwrap();
        sealed.files.last_mut().unwrap().mode = Some(0o600);
        assert!(verify_one_task_review_matches_sealed(
            &input, &sealed, &review, "task-1", "agent-1"
        )
        .is_err());
    }
}
