//! One owned, ordered routed checker over retained scoped output bytes.

use std::path::Path;

use pytxo_core::routing::BlobRef;
use pytxo_core::{PytxoError, Result};
use pytxo_runner::owned_launch::{
    run_direct_owned_launch, DirectOwnedLaunchSpec, LaunchCallbacks, LaunchGuard, LaunchIntent,
    OwnedLaunchReceipt, OwnedOutcome, OwnedProcess,
};
use pytxo_runner::{
    registry_path, verify_sealed_output_view, ProcessEntry, ProcessRegistryFile,
    SealedOutputSnapshot,
};
use pytxo_store::routing::TaskRoutingState;
use pytxo_store::routing_checker::{
    CheckerCreateOutcome, CheckerCreateRequest, CheckerOwnershipPhase, CheckerOwnershipRequest,
    CheckerProcessRegistration, CheckerSettlement,
};
use pytxo_store::routing_private::{
    CheckerNativeOutcome, ControllerObservation, ControllerReceiptEnvelope, PrivateArtifactClaim,
    PrivateArtifactKind, ReceiptSource,
};
use pytxo_store::{Catalog, PytxoStore};

use crate::flow::{load_reviewed_staged_mission, FlowPlan};
use crate::{ActiveRunGate, ActiveRunState};

pub(crate) struct RoutedOwnedChecker<'a> {
    pub store: &'a PytxoStore,
    pub catalog: &'a Catalog,
    pub plan: &'a FlowPlan,
    pub repo_root: &'a Path,
    pub data_dir: &'a Path,
    pub view_root: &'a Path,
    pub snapshot: &'a SealedOutputSnapshot,
    pub sealed_ref: &'a BlobRef,
    pub request: CheckerOwnershipRequest,
    pub spec: DirectOwnedLaunchSpec,
}

pub(crate) fn run_prepared_owned_checker(
    checker: &RoutedOwnedChecker<'_>,
) -> Result<OwnedLaunchReceipt> {
    verify_sealed_output_view(checker.view_root, checker.snapshot)?;
    let mut callbacks = CheckerCallbacks { checker };
    let receipt = run_direct_owned_launch(&checker.spec, &mut callbacks, || {
        let registry = ProcessRegistryFile::load(&registry_path(checker.data_dir))?;
        Ok(crate::routed_fixture::routed_stop_requested(
            checker.catalog,
            &checker.plan.draft_id,
            &checker.request.scope.run_id.0,
        )? || registry
            .cancelled_runs
            .contains(&checker.request.scope.run_id.0))
    })?;
    if receipt.process_registered && receipt.active_processes == Some(0) {
        let owner = checker
            .store
            .checker_ownership(&checker.request.attempt_id, checker.request.ordinal)?
            .ok_or_else(|| failure("owned routed checker disappeared"))?;
        if owner.phase != CheckerOwnershipPhase::Settled {
            return Err(failure(
                "owned routed checker Job-zero was not durably settled",
            ));
        }
    }
    Ok(receipt)
}

struct CheckerCallbacks<'a, 'b> {
    checker: &'a RoutedOwnedChecker<'b>,
}

struct CheckerGuard<'a, 'b, 'c> {
    callbacks: &'a mut CheckerCallbacks<'b, 'c>,
    _gate: ActiveRunGate,
}

impl CheckerCallbacks<'_, '_> {
    fn catalog_stop_requested(&self) -> Result<bool> {
        crate::routed_fixture::routed_stop_requested(
            self.checker.catalog,
            &self.checker.plan.draft_id,
            &self.checker.request.scope.run_id.0,
        )
    }

    fn assert_authority(&self) -> Result<()> {
        let checker = self.checker;
        let active_path = checker.data_dir.join("active_run.json");
        let active: ActiveRunState = serde_json::from_slice(&std::fs::read(&active_path)?)
            .map_err(|error| failure(&format!("routed checker active marker invalid: {error}")))?;
        let current_start = pytxo_runner::process_start_identity(std::process::id())?
            .ok_or_else(|| failure("routed checker supervisor has no start identity"))?;
        if active.run_id != checker.request.scope.run_id.0
            || active.repo_root != checker.repo_root.to_string_lossy()
            || active.supervisor_pid != std::process::id()
            || active.supervisor_start_identity.as_deref() != Some(&current_start)
        {
            return Err(failure("routed checker active marker changed"));
        }
        let registry = ProcessRegistryFile::load(&registry_path(checker.data_dir))?;
        if self.catalog_stop_requested()?
            || registry
                .cancelled_runs
                .contains(&checker.request.scope.run_id.0)
        {
            return Err(PytxoError::Cancelled(
                checker.request.scope.run_id.0.clone(),
            ));
        }
        let reviewed = load_reviewed_staged_mission(checker.store, checker.plan)
            .map_err(|error| failure(&error.to_string()))?;
        let registered = checker
            .store
            .load_registered_mission_with_recipe_integrity(&checker.request.scope)?;
        if reviewed != registered {
            return Err(failure("routed checker reviewed mission changed"));
        }
        let history = checker
            .store
            .routing_history(&checker.request.scope)?
            .ok_or_else(|| failure("routed checker mission disappeared"))?;
        if history.cancelled || history.cancel_epoch != reviewed.authorization.cancel_epoch {
            return Err(PytxoError::Cancelled(
                checker.request.scope.run_id.0.clone(),
            ));
        }
        let task = history
            .tasks
            .iter()
            .find(|task| task.registration.contract.task_id == checker.request.task_id)
            .ok_or_else(|| failure("routed checker task disappeared"))?;
        if task.state != TaskRoutingState::Active
            || task.current_attempt.as_ref() != Some(&checker.request.attempt_id)
        {
            return Err(failure("routed checker task is not current"));
        }
        let attempt = history
            .attempts
            .iter()
            .find(|attempt| attempt.attempt_id == checker.request.attempt_id)
            .ok_or_else(|| failure("routed checker attempt disappeared"))?;
        if attempt.state != pytxo_core::routing::AttemptState::Verifying
            || attempt.receipts.sealed_output.as_ref() != Some(&checker.sealed_ref.digest)
            || attempt.capacity_reservation != checker.request.reservation_id
        {
            return Err(failure("routed checker attempt or sealed output changed"));
        }
        verify_sealed_output_view(checker.view_root, checker.snapshot)?;
        Ok(())
    }

    fn claim(&self) -> PrivateArtifactClaim {
        checker_claim(&self.checker.request)
    }
}

impl LaunchCallbacks for CheckerCallbacks<'_, '_> {
    fn authorize(&mut self, _intent: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
        let path = self.checker.data_dir.join("active_run.json");
        let gate = ActiveRunGate::acquire(&path).map_err(|error| failure(&error.to_string()))?;
        self.assert_authority()?;
        let request = &self.checker.request;
        let owner = self
            .checker
            .store
            .checker_ownership(&request.attempt_id, request.ordinal)?
            .ok_or_else(|| failure("routed checker owner is absent"))?;
        if owner.phase != CheckerOwnershipPhase::Prepared || owner.request != *request {
            return Err(failure("routed checker native create is not fresh"));
        }
        Ok(Box::new(CheckerGuard {
            callbacks: self,
            _gate: gate,
        }))
    }

    fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()> {
        if receipt.active_processes != Some(0) {
            return Ok(());
        }
        // Job-zero is already observed. Preserve that ownership fact even if
        // the checker changed (or made unreadable) its isolated view.
        let view_intact =
            verify_sealed_output_view(self.checker.view_root, self.checker.snapshot).is_ok();
        let request = &self.checker.request;
        let owner = self
            .checker
            .store
            .checker_ownership(&request.attempt_id, request.ordinal)?
            .ok_or_else(|| failure("routed checker owner disappeared before settlement"))?;
        if owner.phase == CheckerOwnershipPhase::Prepared
            && !receipt.process_registered
            && receipt.bootstrap.is_none()
        {
            // Existing Prepared no-create reconciliation owns this outcome.
            return Ok(());
        }
        if owner.phase != CheckerOwnershipPhase::Registered {
            return Err(failure("routed checker native process was not registered"));
        }
        let root = receipt
            .bootstrap
            .as_ref()
            .ok_or_else(|| failure("routed checker native root identity is absent"))?;
        if owner.pid != Some(root.pid)
            || owner.start_identity != root.start_identity
            || owner.job_name.as_deref() != Some(root.job_name.as_str())
            || owner.launch_nonce.as_deref() != Some(receipt.intent.launch_nonce.as_str())
        {
            return Err(failure(
                "routed checker native root differs from durable registration",
            ));
        }
        let claim = self.claim();
        let result = ControllerReceiptEnvelope {
            schema_version: 1,
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            attempt_id: request.attempt_id.clone(),
            reservation_id: request.reservation_id.clone(),
            observed_at_ms: now_ms()?,
            evidence_id: claim.event_id.clone(),
            source: ReceiptSource::OwnedJobObservation,
            observation: ControllerObservation::NativeCheckerResult {
                check_id: request.check_id.clone(),
                ordinal: request.ordinal,
                job_name: receipt.intent.job_name.clone(),
                launch_nonce: receipt.intent.launch_nonce.clone(),
                active_processes: 0,
                exit_code: receipt
                    .exit_code
                    .and_then(|code| i32::try_from(code).ok())
                    .unwrap_or(-1),
                payload_exit_code: receipt.payload_exit_code,
                process_registered: receipt.process_registered,
                barrier_released: receipt.barrier_released,
                native_outcome: match receipt.outcome {
                    _ if !view_intact => CheckerNativeOutcome::Failed,
                    OwnedOutcome::Succeeded => CheckerNativeOutcome::Succeeded,
                    OwnedOutcome::Failed => CheckerNativeOutcome::Failed,
                    OwnedOutcome::Cancelled => CheckerNativeOutcome::Cancelled,
                    OwnedOutcome::RecoveryRequired => CheckerNativeOutcome::RecoveryRequired,
                },
                stdout_complete: receipt.output_complete,
                stderr_complete: receipt.output_complete,
                output_truncated: receipt.output_truncated,
                error_present: receipt.error.is_some() || !view_intact,
                sealed_view_before: self.checker.sealed_ref.clone(),
                sealed_view_after: view_intact.then(|| self.checker.sealed_ref.clone()),
            },
        };
        let reference = self
            .checker
            .store
            .put_controller_receipt(&claim, &result)
            .map_err(|error| failure(&error.to_string()))?;
        self.checker
            .store
            .settle_checker_ownership(&CheckerSettlement {
                scope: request.scope.clone(),
                attempt_id: request.attempt_id.clone(),
                ordinal: request.ordinal,
                event_id: format!(
                    "{}:checker-{}:settled",
                    request.attempt_id.0, request.ordinal
                ),
                receipt_claim: claim,
                receipt_ref: reference,
            })?;
        Ok(())
    }
}

impl LaunchGuard for CheckerGuard<'_, '_, '_> {
    fn permit_create(&mut self, intent: &LaunchIntent) -> Result<()> {
        let callbacks = &mut self.callbacks;
        #[cfg(feature = "routed-test-faults")]
        if std::env::var_os("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_CHECKER_CREATE").is_some() {
            let saved = callbacks
                .checker
                .catalog
                .get_flow_draft(&callbacks.checker.plan.draft_id)?
                .ok_or_else(|| failure("routed checker Stop fixture lost reviewed Flow"))?;
            let exact_plan = saved
                .plan_json
                .ok_or_else(|| failure("routed checker Stop fixture lost saved plan"))?;
            if !callbacks.checker.catalog.request_routed_flow_stop(
                &callbacks.checker.plan.draft_id,
                &exact_plan,
                &callbacks.checker.request.scope.run_id.0,
            )? {
                return Err(failure(
                    "routed checker Stop fixture could not stage exact Stop",
                ));
            }
        }
        if callbacks.catalog_stop_requested()? {
            return Err(PytxoError::Cancelled(
                callbacks.checker.request.scope.run_id.0.clone(),
            ));
        }
        let request = &callbacks.checker.request;
        let created = callbacks.checker.store.authorize_checker_create(
            callbacks.checker.catalog,
            &CheckerCreateRequest {
                scope: request.scope.clone(),
                attempt_id: request.attempt_id.clone(),
                ordinal: request.ordinal,
                event_id: format!(
                    "{}:checker-{}:create",
                    request.attempt_id.0, request.ordinal
                ),
                job_name: intent.job_name.clone(),
                launch_nonce: intent.launch_nonce.clone(),
                now_ms: now_ms()?,
            },
        )?;
        if !matches!(created, CheckerCreateOutcome::Fresh(_)) {
            return Err(failure("routed checker native create token was replayed"));
        }
        Ok(())
    }

    fn register(&mut self, process: &OwnedProcess) -> Result<()> {
        let callbacks = &mut self.callbacks;
        let request = &callbacks.checker.request;
        let start = process
            .start_identity
            .as_ref()
            .ok_or_else(|| failure("routed checker native start identity is absent"))?;
        callbacks
            .checker
            .store
            .register_checker_process(&CheckerProcessRegistration {
                scope: request.scope.clone(),
                attempt_id: request.attempt_id.clone(),
                ordinal: request.ordinal,
                event_id: format!(
                    "{}:checker-{}:register",
                    request.attempt_id.0, request.ordinal
                ),
                observed_job_name: process.job_name.clone(),
                pid: process.pid,
                start_identity: start.clone(),
            })?;
        ProcessRegistryFile::update(&registry_path(callbacks.checker.data_dir), |registry| {
            registry.push(ProcessEntry {
                run_id: request.scope.run_id.0.clone(),
                repo_root: callbacks.checker.repo_root.to_string_lossy().into_owned(),
                agent_key: checker_agent_key(request),
                pid: process.pid,
                start_identity: Some(start.clone()),
                worktree_path: callbacks.checker.view_root.to_string_lossy().into_owned(),
                branch: String::new(),
            });
            Ok(())
        })?;
        Ok(())
    }

    fn cancelled(&mut self) -> Result<bool> {
        match self.callbacks.assert_authority() {
            Ok(()) => Ok(false),
            Err(PytxoError::Cancelled(_)) => Ok(true),
            Err(error) => Err(error),
        }
    }
}

pub(crate) fn checker_agent_key(request: &CheckerOwnershipRequest) -> String {
    format!(
        "{}:{}:checker-{}",
        request.scope.run_id.0, request.attempt_id.0, request.ordinal
    )
}

pub(crate) fn checker_claim(request: &CheckerOwnershipRequest) -> PrivateArtifactClaim {
    PrivateArtifactClaim {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
        reservation_id: request.reservation_id.clone(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: format!(
            "{}:checker-{}:result",
            request.attempt_id.0, request.ordinal
        ),
        event_id: format!(
            "{}:checker-{}:observed",
            request.attempt_id.0, request.ordinal
        ),
    }
}

fn now_ms() -> Result<u64> {
    u64::try_from(chrono::Utc::now().timestamp_millis())
        .map_err(|error| failure(&format!("routed checker clock invalid: {error}")))
}

fn failure(message: &str) -> PytxoError {
    PytxoError::Runner(message.into())
}
