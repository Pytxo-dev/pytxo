//! Private one-use ownership for each reviewed routed check. This ledger
//! records controller claims; the native supervisor must independently observe
//! the exact Job before retaining a settlement receipt.

use pytxo_core::routing::{canonical_digest, AttemptId, AttemptState, BlobRef, CheckId, Digest};
use pytxo_core::{PytxoError, Result, TaskId};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::capacity::CapacityReservationState;
use crate::routing::{
    require_history, require_registered_mission_with_recipe_integrity, save_attempt,
    RoutedAttemptRecord, RoutingScope,
};
use crate::routing_capacity_intent::{exact_catalog_reservation, load_intent, CapacityIntentPhase};
use crate::routing_launch::{load_ownership, LaunchOwnershipPhase, LaunchOwnershipRequest};
use crate::routing_private::{
    exact_admitted_attempt, exact_retained_receipt_by_id, require_cancelled_event,
    retained_attempt_reference_matches, retained_reference_matches, CheckerNativeOutcome,
    ControllerObservation, PrivateArtifactClaim, PrivateArtifactKind, ReceiptSource,
};
use crate::{Catalog, PytxoStore};

#[derive(Serialize)]
struct OwnedCheckPassV1 {
    schema_version: u32,
    scope: RoutingScope,
    task_id: TaskId,
    attempt_id: AttemptId,
    worker_receipt: BlobRef,
    sealed_output: BlobRef,
    checks: Vec<OwnedCheckPassEntryV1>,
}

#[derive(Serialize)]
struct OwnedCheckPassEntryV1 {
    check_id: CheckId,
    ordinal: u32,
    recipe_digest: Digest,
    native_receipt: BlobRef,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckerOwnershipRequest {
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub reservation_id: String,
    pub launch_token: String,
    pub check_id: CheckId,
    pub ordinal: u32,
    pub recipe_digest: Digest,
    pub sealed_view: BlobRef,
    pub prepare_event_id: String,
    pub now_ms: u64,
}

impl std::fmt::Debug for CheckerOwnershipRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckerOwnershipRequest")
            .field("attempt_id", &self.attempt_id)
            .field("check_id", &self.check_id)
            .field("ordinal", &self.ordinal)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckerOwnershipPhase {
    Prepared,
    CreateMayHaveStarted,
    Registered,
    Settled,
    ClosedNoCreate,
}
impl CheckerOwnershipPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::CreateMayHaveStarted => "create_may_have_started",
            Self::Registered => "registered",
            Self::Settled => "settled",
            Self::ClosedNoCreate => "closed_no_create",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckerOwnershipRecord {
    pub request: CheckerOwnershipRequest,
    pub phase: CheckerOwnershipPhase,
    pub revision: u64,
    pub create_event_id: Option<String>,
    pub job_name: Option<String>,
    pub launch_nonce: Option<String>,
    pub register_event_id: Option<String>,
    pub pid: Option<u32>,
    pub start_identity: Option<String>,
    pub settlement_event_id: Option<String>,
    pub settlement_artifact_id: Option<String>,
    pub settlement_blob: Option<BlobRef>,
    pub passed: Option<bool>,
}

impl std::fmt::Debug for CheckerOwnershipRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckerOwnershipRecord")
            .field("request", &self.request)
            .field("phase", &self.phase)
            .field("revision", &self.revision)
            .field("passed", &self.passed)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckerCreateOutcome {
    /// Only Fresh permits a single native create in the current process.
    Fresh(CheckerOwnershipRecord),
    Replay(CheckerOwnershipRecord),
}

#[derive(Clone, PartialEq, Eq)]
pub struct CheckerCreateRequest {
    pub scope: RoutingScope,
    pub attempt_id: AttemptId,
    pub ordinal: u32,
    pub event_id: String,
    pub job_name: String,
    pub launch_nonce: String,
    pub now_ms: u64,
}

impl std::fmt::Debug for CheckerCreateRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckerCreateRequest")
            .field("attempt_id", &self.attempt_id)
            .field("ordinal", &self.ordinal)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckerProcessRegistration {
    pub scope: RoutingScope,
    pub attempt_id: AttemptId,
    pub ordinal: u32,
    pub event_id: String,
    pub observed_job_name: String,
    pub pid: u32,
    pub start_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckerSettlement {
    pub scope: RoutingScope,
    pub attempt_id: AttemptId,
    pub ordinal: u32,
    pub event_id: String,
    pub receipt_claim: PrivateArtifactClaim,
    pub receipt_ref: BlobRef,
}

impl PytxoStore {
    /// A deterministic digest of complete, retained, independently owned
    /// checker receipts. This is controller evidence; it cannot create OS
    /// evidence or grant permission by itself.
    pub fn owned_checker_pass_digest(&self, attempt_id: &AttemptId) -> Result<Digest> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        crate::routing_launch::validate_launch_ownership_implication(&tx)?;
        let raw: String = tx
            .query_row(
                "SELECT record_json FROM routing_attempts WHERE attempt_id=?1",
                [&attempt_id.0],
                |row| row.get(0),
            )
            .optional()
            .map_err(err)?
            .ok_or_else(|| invalid("owned checker attempt absent"))?;
        let attempt: RoutedAttemptRecord = serde_json::from_str(&raw).map_err(err)?;
        require(
            attempt.attempt_id == *attempt_id && json(&attempt)? == raw,
            "owned checker attempt projection changed",
        )?;
        let digest = require_owned_checker_pass_digest(&tx, &attempt)?;
        tx.commit().map_err(err)?;
        Ok(digest)
    }

    /// Prepare one exact next check after the worker Job has settled. Catalog
    /// and domain Store remain separate databases; the caller must hold the
    /// shared launch/Stop gate through the later one-use create callback.
    pub fn prepare_checker_ownership(
        &self,
        catalog: &Catalog,
        request: &CheckerOwnershipRequest,
    ) -> Result<CheckerOwnershipRecord> {
        validate_request(request)?;
        let observed_intent = load_intent(&self.conn, &request.reservation_id)?
            .ok_or_else(|| invalid("checker has no capacity intent"))?;
        require(
            observed_intent.phase == CapacityIntentPhase::ReserveMayHaveStarted,
            "checker requires a bound unresolved reserve intent",
        )?;
        let capacity = exact_catalog_reservation(catalog, &observed_intent)?;
        require(
            capacity.state == CapacityReservationState::Bound
                && capacity.launch_token.as_deref() == Some(&request.launch_token)
                && capacity.bound_at_ms.is_some()
                && capacity.release_evidence.is_none(),
            "checker requires the exact bound host reservation",
        )?;
        let tx = immediate(&self.conn)?;
        require(
            load_intent(&tx, &request.reservation_id)?.as_ref() == Some(&observed_intent),
            "checker capacity intent changed during Catalog read",
        )?;
        require(
            observed_intent.request.scope == request.scope
                && observed_intent.request.task_id == request.task_id
                && observed_intent.request.reservation.attempt_id == request.attempt_id.0,
            "checker capacity intent owner changed",
        )?;
        if let Some(existing) = load_checker(&tx, &request.attempt_id, request.ordinal)? {
            require(
                existing.request == *request,
                "checker preparation replay changed",
            )?;
            require(
                marked_attempt(&tx, &existing)?.owned_checker_count >= request.ordinal,
                "checker preparation replay lost its attempt marker",
            )?;
            tx.commit().map_err(err)?;
            return Ok(existing);
        }
        let mut attempt = active_checker_attempt(&tx, request, request.now_ms)?;
        require_record_matches_frozen_recipe(&tx, request)?;
        require(
            attempt.owned_checker_count.checked_add(1) == Some(request.ordinal),
            "checker ordinal is not the next prepared check",
        )?;
        if request.ordinal > 1 {
            let previous = load_checker(&tx, &request.attempt_id, request.ordinal - 1)?
                .ok_or_else(|| invalid("previous checker owner missing"))?;
            require(
                previous.phase == CheckerOwnershipPhase::Settled && previous.passed == Some(true),
                "previous ordered checker has not passed",
            )?;
        }
        let record = CheckerOwnershipRecord {
            request: request.clone(),
            phase: CheckerOwnershipPhase::Prepared,
            revision: 1,
            create_event_id: None,
            job_name: None,
            launch_nonce: None,
            register_event_id: None,
            pid: None,
            start_identity: None,
            settlement_event_id: None,
            settlement_artifact_id: None,
            settlement_blob: None,
            passed: None,
        };
        tx.execute(
            "INSERT INTO attempt_checker_ownership
             (attempt_id,ordinal,check_id,domain_id,run_id,task_id,reservation_id,launch_token,
              recipe_digest,sealed_view_digest,sealed_view_length,prepare_event_id,job_name,
              launch_nonce,pid,start_identity,settlement_artifact_id,passed,phase,revision,record_json)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,NULL,NULL,NULL,NULL,NULL,NULL,?13,1,?14)",
            params![request.attempt_id.0,request.ordinal,request.check_id.0,
                request.scope.domain_id.0,request.scope.run_id.0,request.task_id.0,
                request.reservation_id,request.launch_token,request.recipe_digest.0,
                request.sealed_view.digest.0,request.sealed_view.byte_length,
                request.prepare_event_id,record.phase.as_str(),json(&record)?],
        ).map_err(err)?;
        attempt.owned_checker_count = request.ordinal;
        save_attempt(&tx, &mut attempt)?;
        tx.commit().map_err(err)?;
        self.checker_ownership(&request.attempt_id, request.ordinal)?
            .ok_or_else(|| invalid("checker preparation not persisted"))
    }

    pub fn authorize_checker_create(
        &self,
        catalog: &Catalog,
        request: &CheckerCreateRequest,
    ) -> Result<CheckerCreateOutcome> {
        for id in [&request.event_id, &request.job_name, &request.launch_nonce] {
            require_id(id)?;
        }
        let existing = self
            .checker_ownership(&request.attempt_id, request.ordinal)?
            .ok_or_else(|| invalid("checker ownership absent"))?;
        let observed_intent = load_intent(&self.conn, &existing.request.reservation_id)?
            .ok_or_else(|| invalid("checker capacity intent absent"))?;
        let capacity = exact_catalog_reservation(catalog, &observed_intent)?;
        require(
            capacity.state == CapacityReservationState::Bound
                && capacity.launch_token.as_deref() == Some(&existing.request.launch_token)
                && capacity.release_evidence.is_none(),
            "checker create lost exact bound host capacity",
        )?;
        let tx = immediate(&self.conn)?;
        let mut record = required_checker(&tx, &request.attempt_id, request.ordinal)?;
        require(
            record.request.scope == request.scope
                && load_intent(&tx, &record.request.reservation_id)?.as_ref()
                    == Some(&observed_intent),
            "checker create owner or intent changed",
        )?;
        require(
            observed_intent.request.scope == record.request.scope
                && observed_intent.request.task_id == record.request.task_id
                && observed_intent.request.reservation.attempt_id == record.request.attempt_id.0,
            "checker create capacity intent owner changed",
        )?;
        if record.create_event_id.as_deref() == Some(&request.event_id) {
            require(
                record.job_name.as_deref() == Some(&request.job_name)
                    && record.launch_nonce.as_deref() == Some(&request.launch_nonce),
                "checker create replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(CheckerCreateOutcome::Replay(record));
        }
        require(
            record.phase == CheckerOwnershipPhase::Prepared,
            "checker create was already consumed",
        )?;
        require_record_matches_frozen_recipe(&tx, &record.request)?;
        active_checker_attempt(&tx, &record.request, request.now_ms)?;
        require_distinct_worker_job_name(&tx, &request.job_name)?;
        record.phase = CheckerOwnershipPhase::CreateMayHaveStarted;
        record.create_event_id = Some(request.event_id.clone());
        record.job_name = Some(request.job_name.clone());
        record.launch_nonce = Some(request.launch_nonce.clone());
        save_checker(&tx, &mut record, CheckerOwnershipPhase::Prepared)?;
        tx.commit().map_err(err)?;
        let persisted = self
            .checker_ownership(&request.attempt_id, request.ordinal)?
            .ok_or_else(|| invalid("checker create CAS not persisted"))?;
        require(persisted == record, "checker create changed after CAS")?;
        Ok(CheckerCreateOutcome::Fresh(persisted))
    }

    /// Registration remains possible after cancellation so Stop can still find
    /// a process created immediately before its fence.
    pub fn register_checker_process(
        &self,
        request: &CheckerProcessRegistration,
    ) -> Result<CheckerOwnershipRecord> {
        for id in [
            &request.event_id,
            &request.observed_job_name,
            &request.start_identity,
        ] {
            require_id(id)?;
        }
        require(request.pid > 0, "checker PID must be positive")?;
        let tx = immediate(&self.conn)?;
        let mut record = required_checker(&tx, &request.attempt_id, request.ordinal)?;
        require(
            record.request.scope == request.scope,
            "checker scope changed",
        )?;
        if record.register_event_id.as_deref() == Some(&request.event_id) {
            require(
                record.job_name.as_deref() == Some(&request.observed_job_name)
                    && record.pid == Some(request.pid)
                    && record.start_identity.as_deref() == Some(&request.start_identity),
                "checker registration replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CheckerOwnershipPhase::CreateMayHaveStarted
                && record.job_name.as_deref() == Some(&request.observed_job_name),
            "checker registration lacks one-use create",
        )?;
        record.phase = CheckerOwnershipPhase::Registered;
        record.register_event_id = Some(request.event_id.clone());
        record.pid = Some(request.pid);
        record.start_identity = Some(request.start_identity.clone());
        save_checker(
            &tx,
            &mut record,
            CheckerOwnershipPhase::CreateMayHaveStarted,
        )?;
        tx.commit().map_err(err)?;
        self.checker_ownership(&request.attempt_id, request.ordinal)?
            .ok_or_else(|| invalid("checker registration not persisted"))
    }

    pub fn settle_checker_ownership(
        &self,
        request: &CheckerSettlement,
    ) -> Result<CheckerOwnershipRecord> {
        require_id(&request.event_id)?;
        require(
            request.receipt_claim.kind == PrivateArtifactKind::ControllerReceipt,
            "checker settlement requires a controller receipt",
        )?;
        let receipt = self.read_controller_receipt(&request.receipt_claim, &request.receipt_ref)?;
        let tx = immediate(&self.conn)?;
        let mut record = required_checker(&tx, &request.attempt_id, request.ordinal)?;
        require_receipt_owner(&record, request)?;
        require(
            retained_reference_matches(
                &tx,
                &request.receipt_claim,
                PrivateArtifactKind::ControllerReceipt,
                &request.receipt_ref,
            )?,
            "checker receipt changed before settlement CAS",
        )?;
        require(
            receipt.source == ReceiptSource::OwnedJobObservation,
            "checker settlement requires native Job observation",
        )?;
        let ControllerObservation::NativeCheckerResult {
            check_id,
            ordinal,
            job_name,
            launch_nonce,
            active_processes,
            exit_code,
            payload_exit_code,
            process_registered,
            barrier_released,
            native_outcome,
            stdout_complete,
            stderr_complete,
            output_truncated,
            error_present,
            sealed_view_before,
            sealed_view_after,
            ..
        } = receipt.observation
        else {
            return Err(invalid("checker settlement receipt is not a native result"));
        };
        require(
            check_id == record.request.check_id
                && ordinal == record.request.ordinal
                && record.job_name.as_deref() == Some(&job_name)
                && record.launch_nonce.as_deref() == Some(&launch_nonce)
                && active_processes == 0
                && sealed_view_before == record.request.sealed_view,
            "checker result identity or before-view mismatch",
        )?;
        let passed = exit_code == 0
            && payload_exit_code == Some(0)
            && process_registered
            && barrier_released
            && native_outcome == CheckerNativeOutcome::Succeeded
            && stdout_complete
            && stderr_complete
            && !output_truncated
            && !error_present
            && sealed_view_after.as_ref() == Some(&record.request.sealed_view);
        if record.settlement_event_id.as_deref() == Some(&request.event_id) {
            require(
                record.phase == CheckerOwnershipPhase::Settled
                    && record.settlement_artifact_id.as_deref()
                        == Some(&request.receipt_claim.artifact_id)
                    && record.settlement_blob.as_ref() == Some(&request.receipt_ref)
                    && record.passed == Some(passed),
                "checker settlement replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CheckerOwnershipPhase::Registered,
            "checker settlement requires registered native identity",
        )?;
        record.phase = CheckerOwnershipPhase::Settled;
        record.settlement_event_id = Some(request.event_id.clone());
        record.settlement_artifact_id = Some(request.receipt_claim.artifact_id.clone());
        record.settlement_blob = Some(request.receipt_ref.clone());
        record.passed = Some(passed);
        save_checker(&tx, &mut record, CheckerOwnershipPhase::Registered)?;
        tx.commit().map_err(err)?;
        self.checker_ownership(&request.attempt_id, request.ordinal)?
            .ok_or_else(|| invalid("checker settlement not persisted"))
    }

    pub fn close_prepared_checker_without_create(
        &self,
        request: &CheckerSettlement,
    ) -> Result<CheckerOwnershipRecord> {
        require_id(&request.event_id)?;
        require(
            request.receipt_claim.kind == PrivateArtifactKind::ControllerReceipt,
            "checker no-create closure requires controller receipt",
        )?;
        let receipt = self.read_controller_receipt(&request.receipt_claim, &request.receipt_ref)?;
        require(
            receipt.source == ReceiptSource::TrustedController,
            "checker no-create closure requires trusted controller source",
        )?;
        let ControllerObservation::CheckerNotCreated {
            check_id,
            ordinal,
            cancellation_event_id,
        } = receipt.observation
        else {
            return Err(invalid("checker no-create observation mismatch"));
        };
        let tx = immediate(&self.conn)?;
        let mut record = required_checker(&tx, &request.attempt_id, request.ordinal)?;
        require_receipt_owner(&record, request)?;
        require(
            record.request.check_id == check_id && record.request.ordinal == ordinal,
            "checker no-create ID changed",
        )?;
        require_cancelled_event(&tx, &request.receipt_claim, &cancellation_event_id)?;
        require(
            retained_reference_matches(
                &tx,
                &request.receipt_claim,
                PrivateArtifactKind::ControllerReceipt,
                &request.receipt_ref,
            )?,
            "checker no-create receipt changed before CAS",
        )?;
        if record.settlement_event_id.as_deref() == Some(&request.event_id) {
            require(
                record.phase == CheckerOwnershipPhase::ClosedNoCreate
                    && record.settlement_artifact_id.as_deref()
                        == Some(&request.receipt_claim.artifact_id)
                    && record.settlement_blob.as_ref() == Some(&request.receipt_ref),
                "checker no-create replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CheckerOwnershipPhase::Prepared,
            "checker native creation may have started",
        )?;
        record.phase = CheckerOwnershipPhase::ClosedNoCreate;
        record.settlement_event_id = Some(request.event_id.clone());
        record.settlement_artifact_id = Some(request.receipt_claim.artifact_id.clone());
        record.settlement_blob = Some(request.receipt_ref.clone());
        save_checker(&tx, &mut record, CheckerOwnershipPhase::Prepared)?;
        tx.commit().map_err(err)?;
        self.checker_ownership(&request.attempt_id, request.ordinal)?
            .ok_or_else(|| invalid("checker no-create closure not persisted"))
    }

    pub fn checker_ownership(
        &self,
        attempt_id: &AttemptId,
        ordinal: u32,
    ) -> Result<Option<CheckerOwnershipRecord>> {
        load_checker(&self.conn, attempt_id, ordinal)
    }

    pub fn unresolved_checker_ownership_scopes(&self) -> Result<Vec<RoutingScope>> {
        validate_checker_ownership_implication(&self.conn)?;
        let mut stmt = self.conn.prepare(
            "SELECT attempt_id,ordinal FROM attempt_checker_ownership ORDER BY attempt_id,ordinal",
        ).map_err(err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        let mut scopes = Vec::new();
        for (id, ordinal) in rows {
            let record = required_checker(&self.conn, &AttemptId(id), ordinal)?;
            if !matches!(
                record.phase,
                CheckerOwnershipPhase::Settled | CheckerOwnershipPhase::ClosedNoCreate
            ) && !scopes.contains(&record.request.scope)
            {
                scopes.push(record.request.scope);
            }
        }
        Ok(scopes)
    }
}

pub(crate) fn require_checkers_settled_for_release(
    conn: &Connection,
    attempt: &RoutedAttemptRecord,
) -> Result<()> {
    require_checker_marker_consistency(conn, attempt)?;
    for ordinal in 1..=attempt.owned_checker_count {
        let row = required_checker(conn, &attempt.attempt_id, ordinal)?;
        require(
            row.request.scope == attempt.scope
                && row.request.task_id == attempt.task_id
                && row.request.reservation_id == attempt.capacity_reservation
                && matches!(
                    row.phase,
                    CheckerOwnershipPhase::Settled | CheckerOwnershipPhase::ClosedNoCreate
                ),
            "checker ownership is missing, changed or unsettled",
        )?;
    }
    Ok(())
}

pub(crate) fn require_owned_checker_pass_digest(
    conn: &Connection,
    attempt: &RoutedAttemptRecord,
) -> Result<Digest> {
    require(
        attempt.owned_launch_required
            && matches!(
                attempt.state,
                AttemptState::Verifying | AttemptState::Passed
            ),
        "owned checker aggregate requires a verifying worker",
    )?;
    require_checker_marker_consistency(conn, attempt)?;
    let mission = require_registered_mission_with_recipe_integrity(conn, &attempt.scope)?;
    let task = mission
        .tasks
        .iter()
        .find(|task| task.contract.task_id == attempt.task_id)
        .ok_or_else(|| invalid("owned checker task absent from reviewed mission"))?;
    require(
        !task.check_recipes.is_empty()
            && task.check_recipes.len() <= 16
            && task.check_recipes.len() == attempt.owned_checker_count as usize,
        "owned checker aggregate does not cover every reviewed check",
    )?;
    let worker = load_ownership(conn, &attempt.attempt_id)?
        .ok_or_else(|| invalid("owned checker worker owner absent"))?;
    let worker_receipt = worker
        .settlement_blob
        .ok_or_else(|| invalid("owned checker worker has no Job-zero receipt"))?;
    require(
        worker.phase == LaunchOwnershipPhase::Settled
            && worker.request.scope == attempt.scope
            && worker.request.task_id == attempt.task_id
            && worker.request.reservation_id == attempt.capacity_reservation
            && attempt.receipts.quiescence.as_ref() == Some(&worker_receipt.digest),
        "owned checker worker settlement changed",
    )?;
    let mut sealed_output = None;
    let mut checks = Vec::with_capacity(task.check_recipes.len());
    for recipe in &task.check_recipes {
        let checker = load_checker(conn, &attempt.attempt_id, recipe.ordinal)?
            .ok_or_else(|| invalid("owned checker aggregate row missing"))?;
        let native_receipt = checker
            .settlement_blob
            .ok_or_else(|| invalid("owned checker native receipt missing"))?;
        require(
            checker.phase == CheckerOwnershipPhase::Settled
                && checker.passed == Some(true)
                && checker.request.scope == attempt.scope
                && checker.request.task_id == attempt.task_id
                && checker.request.reservation_id == attempt.capacity_reservation
                && checker.request.check_id == recipe.id
                && checker.request.ordinal == recipe.ordinal
                && checker.request.recipe_digest == recipe.reference()?.recipe_digest
                && sealed_output
                    .as_ref()
                    .is_none_or(|output| output == &checker.request.sealed_view),
            "owned checker aggregate contains an incomplete or changed check",
        )?;
        sealed_output = Some(checker.request.sealed_view);
        checks.push(OwnedCheckPassEntryV1 {
            check_id: checker.request.check_id,
            ordinal: checker.request.ordinal,
            recipe_digest: checker.request.recipe_digest,
            native_receipt,
        });
    }
    let sealed_output = sealed_output.ok_or_else(|| invalid("owned checker output absent"))?;
    require(
        attempt.receipts.sealed_output.as_ref() == Some(&sealed_output.digest),
        "owned checker aggregate sealed output changed",
    )?;
    canonical_digest(
        &OwnedCheckPassV1 {
            schema_version: 1,
            scope: attempt.scope.clone(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            worker_receipt,
            sealed_output,
            checks,
        },
        1,
    )
    .map_err(err)
}

fn require_checker_marker_consistency(
    conn: &Connection,
    attempt: &RoutedAttemptRecord,
) -> Result<()> {
    require(
        attempt.owned_checker_count <= 16,
        "checker marker exceeds reviewed maximum",
    )?;
    let mut stmt = conn
        .prepare(
            "SELECT ordinal FROM attempt_checker_ownership WHERE attempt_id=?1 ORDER BY ordinal",
        )
        .map_err(err)?;
    let ordinals = stmt
        .query_map([&attempt.attempt_id.0], |row| row.get::<_, u32>(0))
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    require(
        ordinals.len() == attempt.owned_checker_count as usize,
        "checker owner count differs from independent attempt marker",
    )?;
    for (index, ordinal) in ordinals.into_iter().enumerate() {
        require(ordinal == (index + 1) as u32, "checker owner ordinal gap")?;
        let row = required_checker(conn, &attempt.attempt_id, ordinal)?;
        require(
            row.request.scope == attempt.scope
                && row.request.task_id == attempt.task_id
                && row.request.reservation_id == attempt.capacity_reservation,
            "checker owner differs from admitted attempt",
        )?;
    }
    Ok(())
}

pub(crate) fn validate_checker_ownership_implication(conn: &Connection) -> Result<()> {
    let mut stmt = conn
        .prepare(
            "SELECT attempt_id,ordinal FROM attempt_checker_ownership ORDER BY attempt_id,ordinal",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    for (id, ordinal) in rows {
        let row = required_checker(conn, &AttemptId(id), ordinal)?;
        let attempt = marked_attempt(conn, &row)?;
        require(
            attempt.owned_checker_count >= ordinal,
            "checker row has no independent attempt marker",
        )?;
        if !matches!(
            row.phase,
            CheckerOwnershipPhase::Settled | CheckerOwnershipPhase::ClosedNoCreate
        ) {
            require(
                !attempt.ownership_released,
                "unsettled checker lost attempt hold",
            )?;
        }
    }
    let mut stmt = conn
        .prepare("SELECT attempt_id,record_json FROM routing_attempts ORDER BY attempt_id")
        .map_err(err)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(err)?;
    for row in rows {
        let (id, body) = row.map_err(err)?;
        let attempt: RoutedAttemptRecord = serde_json::from_str(&body).map_err(err)?;
        require(
            attempt.attempt_id.0 == id,
            "checker attempt marker identity changed",
        )?;
        require(
            attempt.owned_checker_count <= 16,
            "checker marker exceeds reviewed maximum",
        )?;
        for ordinal in 1..=attempt.owned_checker_count {
            let checker = required_checker(conn, &attempt.attempt_id, ordinal)?;
            require(
                checker.request.scope == attempt.scope
                    && checker.request.task_id == attempt.task_id
                    && checker.request.reservation_id == attempt.capacity_reservation,
                "checker marker owner differs from attempt",
            )?;
        }
    }
    Ok(())
}

fn owner_claim(
    request: &CheckerOwnershipRequest,
    kind: PrivateArtifactKind,
) -> PrivateArtifactClaim {
    PrivateArtifactClaim {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
        reservation_id: request.reservation_id.clone(),
        kind,
        artifact_id: String::new(),
        event_id: String::new(),
    }
}

fn require_record_matches_frozen_recipe(
    conn: &Connection,
    request: &CheckerOwnershipRequest,
) -> Result<()> {
    let mission = require_registered_mission_with_recipe_integrity(conn, &request.scope)?;
    let task = mission
        .tasks
        .iter()
        .find(|task| task.contract.task_id == request.task_id)
        .ok_or_else(|| invalid("checker task is absent from registered mission"))?;
    let index = usize::try_from(request.ordinal - 1).map_err(err)?;
    let recipe = task
        .check_recipes
        .get(index)
        .ok_or_else(|| invalid("checker ordinal is outside reviewed recipes"))?;
    require(
        recipe.ordinal == request.ordinal
            && recipe.id == request.check_id
            && recipe.reference()?.recipe_digest == request.recipe_digest,
        "checker recipe differs from reviewed registration",
    )
}

fn marked_attempt(conn: &Connection, row: &CheckerOwnershipRecord) -> Result<RoutedAttemptRecord> {
    let claim = owner_claim(&row.request, PrivateArtifactKind::ScopedOutput);
    let attempt = exact_admitted_attempt(conn, &claim)?
        .ok_or_else(|| invalid("checker has no exact admitted attempt"))?;
    require(
        attempt.scope == row.request.scope
            && attempt.task_id == row.request.task_id
            && attempt.capacity_reservation == row.request.reservation_id,
        "checker attempt ownership changed",
    )?;
    Ok(attempt)
}

fn active_checker_attempt(
    conn: &Connection,
    request: &CheckerOwnershipRequest,
    now_ms: u64,
) -> Result<RoutedAttemptRecord> {
    let launch_request = LaunchOwnershipRequest {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
        reservation_id: request.reservation_id.clone(),
        launch_token: request.launch_token.clone(),
        event_id: request.prepare_event_id.clone(),
    };
    let attempt = crate::routing_launch::require_active_attempt(conn, &launch_request)?;
    require_checker_marker_consistency(conn, &attempt)?;
    require(
        attempt.state == AttemptState::Verifying
            && attempt.owned_launch_required
            && now_ms >= attempt.updated_at_ms
            && now_ms >= request.now_ms
            && attempt.receipts.sealed_output.as_ref() == Some(&request.sealed_view.digest),
        "checker requires owned verifying attempt and sealed output",
    )?;
    let history = require_history(conn, &request.scope)?;
    require(
        now_ms < history.mission.authorization.limits.deadline_ms,
        "checker deadline expired",
    )?;
    let worker = load_ownership(conn, &request.attempt_id)?
        .ok_or_else(|| invalid("checker worker owner absent"))?;
    require(
        worker.phase == LaunchOwnershipPhase::Settled
            && worker.request.scope == request.scope
            && worker.request.reservation_id == request.reservation_id
            && worker.request.launch_token == request.launch_token
            && worker
                .settlement_blob
                .as_ref()
                .is_some_and(|blob| attempt.receipts.quiescence.as_ref() == Some(&blob.digest)),
        "checker requires settled exact worker Job",
    )?;
    require(
        retained_attempt_reference_matches(
            conn,
            &owner_claim(request, PrivateArtifactKind::ScopedOutput),
            PrivateArtifactKind::ScopedOutput,
            &request.sealed_view,
        )?,
        "checker sealed view is not retained for this attempt",
    )?;
    Ok(attempt)
}

fn require_receipt_owner(
    record: &CheckerOwnershipRecord,
    request: &CheckerSettlement,
) -> Result<()> {
    require(
        record.request.scope == request.scope
            && record.request.attempt_id == request.attempt_id
            && record.request.ordinal == request.ordinal
            && request.receipt_claim.scope == record.request.scope
            && request.receipt_claim.task_id == record.request.task_id
            && request.receipt_claim.attempt_id == record.request.attempt_id
            && request.receipt_claim.reservation_id == record.request.reservation_id,
        "checker settlement receipt owner mismatch",
    )
}

pub(crate) fn load_checker(
    conn: &Connection,
    attempt_id: &AttemptId,
    ordinal: u32,
) -> Result<Option<CheckerOwnershipRecord>> {
    type Projection = (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        u64,
        String,
        Option<String>,
        Option<String>,
        Option<u32>,
        Option<String>,
        Option<String>,
        Option<bool>,
        String,
        u64,
        String,
    );
    let row: Option<Projection> = conn
        .query_row(
            "SELECT check_id,domain_id,run_id,task_id,reservation_id,launch_token,recipe_digest,
                sealed_view_digest,sealed_view_length,prepare_event_id,job_name,launch_nonce,
                pid,start_identity,settlement_artifact_id,passed,phase,revision,record_json
         FROM attempt_checker_ownership WHERE attempt_id=?1 AND ordinal=?2",
            params![attempt_id.0, ordinal],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                ))
            },
        )
        .optional()
        .map_err(err)?;
    let Some(row) = row else { return Ok(None) };
    let record: CheckerOwnershipRecord = serde_json::from_str(&row.18).map_err(err)?;
    validate_record(&record)?;
    require(
        record.request.attempt_id == *attempt_id
            && record.request.ordinal == ordinal
            && record.request.check_id.0 == row.0
            && record.request.scope.domain_id.0 == row.1
            && record.request.scope.run_id.0 == row.2
            && record.request.task_id.0 == row.3
            && record.request.reservation_id == row.4
            && record.request.launch_token == row.5
            && record.request.recipe_digest.0 == row.6
            && record.request.sealed_view.digest.0 == row.7
            && record.request.sealed_view.byte_length == row.8
            && record.request.prepare_event_id == row.9
            && record.job_name == row.10
            && record.launch_nonce == row.11
            && record.pid == row.12
            && record.start_identity == row.13
            && record.settlement_artifact_id == row.14
            && record.passed == row.15
            && record.phase.as_str() == row.16
            && record.revision == row.17
            && json(&record)? == row.18,
        "checker ownership SQL/JSON projection changed",
    )?;
    require_record_matches_frozen_recipe(conn, &record.request)?;
    let worker = load_ownership(conn, &record.request.attempt_id)?
        .ok_or_else(|| invalid("checker worker owner disappeared"))?;
    require(
        worker.phase == LaunchOwnershipPhase::Settled
            && worker.request.scope == record.request.scope
            && worker.request.task_id == record.request.task_id
            && worker.request.reservation_id == record.request.reservation_id
            && worker.request.launch_token == record.request.launch_token,
        "checker worker launch authority changed",
    )?;
    let attempt = marked_attempt(conn, &record)?;
    require(
        attempt.owned_checker_count >= record.request.ordinal
            && attempt.receipts.sealed_output.as_ref() == Some(&record.request.sealed_view.digest)
            && worker
                .settlement_blob
                .as_ref()
                .is_some_and(|blob| attempt.receipts.quiescence.as_ref() == Some(&blob.digest)),
        "checker attempt marker or sealed worker receipts changed",
    )?;
    require(
        retained_attempt_reference_matches(
            conn,
            &owner_claim(&record.request, PrivateArtifactKind::ScopedOutput),
            PrivateArtifactKind::ScopedOutput,
            &record.request.sealed_view,
        )?,
        "checker retained sealed view changed",
    )?;
    if let Some(job_name) = &record.job_name {
        require_distinct_worker_job_name(conn, job_name)?;
    }
    if let (Some(id), Some(reference)) = (&record.settlement_artifact_id, &record.settlement_blob) {
        let (claim, receipt) = exact_retained_receipt_by_id(conn, id, reference)?;
        require(
            claim.scope == record.request.scope
                && claim.task_id == record.request.task_id
                && claim.attempt_id == record.request.attempt_id
                && claim.reservation_id == record.request.reservation_id,
            "checker settlement receipt owner changed",
        )?;
        match (record.phase, receipt.source, receipt.observation) {
            (
                CheckerOwnershipPhase::Settled,
                ReceiptSource::OwnedJobObservation,
                ControllerObservation::NativeCheckerResult {
                    check_id,
                    ordinal,
                    job_name,
                    launch_nonce,
                    active_processes,
                    exit_code,
                    payload_exit_code,
                    process_registered,
                    barrier_released,
                    native_outcome,
                    stdout_complete,
                    stderr_complete,
                    output_truncated,
                    error_present,
                    sealed_view_before,
                    sealed_view_after,
                    ..
                },
            ) => {
                let passed = exit_code == 0
                    && payload_exit_code == Some(0)
                    && process_registered
                    && barrier_released
                    && native_outcome == CheckerNativeOutcome::Succeeded
                    && stdout_complete
                    && stderr_complete
                    && !output_truncated
                    && !error_present
                    && sealed_view_after.as_ref() == Some(&record.request.sealed_view);
                require(
                    check_id == record.request.check_id
                        && ordinal == record.request.ordinal
                        && record.job_name.as_deref() == Some(&job_name)
                        && record.launch_nonce.as_deref() == Some(&launch_nonce)
                        && active_processes == 0
                        && sealed_view_before == record.request.sealed_view
                        && record.passed == Some(passed),
                    "checker native receipt changed",
                )?;
            }
            (
                CheckerOwnershipPhase::ClosedNoCreate,
                ReceiptSource::TrustedController,
                ControllerObservation::CheckerNotCreated {
                    check_id,
                    ordinal,
                    cancellation_event_id,
                },
            ) => {
                require(
                    check_id == record.request.check_id
                        && ordinal == record.request.ordinal
                        && record.passed.is_none(),
                    "checker no-create receipt changed",
                )?;
                require_cancelled_event(conn, &claim, &cancellation_event_id)?;
            }
            _ => return Err(invalid("checker settlement observation changed")),
        }
    }
    Ok(Some(record))
}

fn required_checker(
    conn: &Connection,
    attempt_id: &AttemptId,
    ordinal: u32,
) -> Result<CheckerOwnershipRecord> {
    load_checker(conn, attempt_id, ordinal)?.ok_or_else(|| invalid("checker ownership absent"))
}

fn require_distinct_worker_job_name(conn: &Connection, job_name: &str) -> Result<()> {
    let collides: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM attempt_launch_ownership WHERE job_name=?1)",
            [job_name],
            |row| row.get(0),
        )
        .map_err(err)?;
    require(!collides, "checker Job name collides with worker ownership")
}

fn save_checker(
    tx: &Transaction<'_>,
    record: &mut CheckerOwnershipRecord,
    prior: CheckerOwnershipPhase,
) -> Result<()> {
    let old = record.revision;
    record.revision = old
        .checked_add(1)
        .ok_or_else(|| invalid("checker revision overflow"))?;
    validate_record(record)?;
    let changed = tx.execute(
        "UPDATE attempt_checker_ownership SET job_name=?1,launch_nonce=?2,pid=?3,start_identity=?4,
         settlement_artifact_id=?5,passed=?6,phase=?7,revision=?8,record_json=?9
         WHERE attempt_id=?10 AND ordinal=?11 AND phase=?12 AND revision=?13",
        params![record.job_name,record.launch_nonce,record.pid,record.start_identity,
            record.settlement_artifact_id,record.passed,record.phase.as_str(),record.revision,
            json(record)?,record.request.attempt_id.0,record.request.ordinal,prior.as_str(),old],
    ).map_err(err)?;
    require(changed == 1, "checker ownership CAS conflict")
}

fn validate_request(request: &CheckerOwnershipRequest) -> Result<()> {
    for id in [
        &request.scope.domain_id.0,
        &request.scope.run_id.0,
        &request.task_id.0,
        &request.attempt_id.0,
        &request.reservation_id,
        &request.launch_token,
        &request.check_id.0,
        &request.prepare_event_id,
    ] {
        require_id(id)?;
    }
    require(
        (1..=16).contains(&request.ordinal)
            && request.recipe_digest.is_valid()
            && request.sealed_view.digest.is_valid()
            && request.sealed_view.byte_length <= 16 * 1024 * 1024,
        "checker request recipe, ordinal or sealed view invalid",
    )
}

fn validate_record(record: &CheckerOwnershipRecord) -> Result<()> {
    validate_request(&record.request)?;
    require(record.revision > 0, "checker ownership revision invalid")?;
    for id in [
        &record.create_event_id,
        &record.job_name,
        &record.launch_nonce,
        &record.register_event_id,
        &record.start_identity,
        &record.settlement_event_id,
        &record.settlement_artifact_id,
    ]
    .into_iter()
    .flatten()
    {
        require_id(id)?;
    }
    let valid = match record.phase {
        CheckerOwnershipPhase::Prepared => {
            record.create_event_id.is_none()
                && record.job_name.is_none()
                && record.launch_nonce.is_none()
                && record.register_event_id.is_none()
                && record.pid.is_none()
                && record.start_identity.is_none()
                && record.settlement_event_id.is_none()
                && record.settlement_artifact_id.is_none()
                && record.settlement_blob.is_none()
                && record.passed.is_none()
        }
        CheckerOwnershipPhase::CreateMayHaveStarted => {
            record.create_event_id.is_some()
                && record.job_name.is_some()
                && record.launch_nonce.is_some()
                && record.register_event_id.is_none()
                && record.pid.is_none()
                && record.start_identity.is_none()
                && record.settlement_event_id.is_none()
                && record.settlement_artifact_id.is_none()
                && record.settlement_blob.is_none()
                && record.passed.is_none()
        }
        CheckerOwnershipPhase::Registered => {
            record.create_event_id.is_some()
                && record.job_name.is_some()
                && record.launch_nonce.is_some()
                && record.register_event_id.is_some()
                && record.pid.is_some_and(|pid| pid > 0)
                && record.start_identity.is_some()
                && record.settlement_event_id.is_none()
                && record.settlement_artifact_id.is_none()
                && record.settlement_blob.is_none()
                && record.passed.is_none()
        }
        CheckerOwnershipPhase::Settled => {
            record.create_event_id.is_some()
                && record.job_name.is_some()
                && record.launch_nonce.is_some()
                && record.register_event_id.is_some()
                && record.pid.is_some_and(|pid| pid > 0)
                && record.start_identity.is_some()
                && record.settlement_event_id.is_some()
                && record.settlement_artifact_id.is_some()
                && record.settlement_blob.is_some()
                && record.passed.is_some()
        }
        CheckerOwnershipPhase::ClosedNoCreate => {
            record.create_event_id.is_none()
                && record.job_name.is_none()
                && record.launch_nonce.is_none()
                && record.register_event_id.is_none()
                && record.pid.is_none()
                && record.start_identity.is_none()
                && record.settlement_event_id.is_some()
                && record.settlement_artifact_id.is_some()
                && record.settlement_blob.is_some()
                && record.passed.is_none()
        }
    };
    require(valid, "checker phase projection inconsistent")
}

fn immediate(conn: &Connection) -> Result<Transaction<'_>> {
    Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(err)
}
fn json(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(err)
}
fn require_id(id: &str) -> Result<()> {
    require(
        !id.trim().is_empty() && id.len() <= 256,
        "checker identity invalid",
    )
}
fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
fn invalid(message: &str) -> PytxoError {
    PytxoError::Store(message.into())
}
fn err(error: impl std::fmt::Display) -> PytxoError {
    invalid(&error.to_string())
}
