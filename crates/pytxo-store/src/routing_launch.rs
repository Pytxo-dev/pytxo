//! Store-only one-use native creation ownership. The future supervisor must
//! hold ActiveRunGate across authority CAS and call Runner only on `Fresh`.
//! A stored receipt is trusted controller evidence, not an OS observation.

use pytxo_core::routing::{AttemptId, AttemptState, BlobRef, CheckId};
use pytxo_core::{DomainId, PytxoError, Result, RunId, TaskId};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::capacity::CapacityReservationState;
use crate::routing::{
    RoutedAttemptRecord, RoutedTaskRecord, RoutingReceipts, RoutingScope, TaskRoutingState,
};
use crate::routing_capacity_intent::{exact_catalog_reservation, load_intent, CapacityIntentPhase};
use crate::routing_private::{
    exact_admitted_attempt, exact_retained_receipt_by_id, require_cancelled_event,
    retained_attempt_reference_matches, retained_reference_matches, ControllerObservation,
    PrivateArtifactClaim, PrivateArtifactKind, ReceiptSource,
};
use crate::{Catalog, PytxoStore};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchOwnershipRequest {
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub reservation_id: String,
    pub launch_token: String,
    pub event_id: String,
}

impl std::fmt::Debug for LaunchOwnershipRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LaunchOwnershipRequest")
            .field("attempt_id", &self.attempt_id)
            .field("reservation_id", &self.reservation_id)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchOwnershipPhase {
    Prepared,
    CreateMayHaveStarted,
    Registered,
    Settled,
    ClosedNoLaunch,
}
impl LaunchOwnershipPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::CreateMayHaveStarted => "create_may_have_started",
            Self::Registered => "registered",
            Self::Settled => "settled",
            Self::ClosedNoLaunch => "closed_no_launch",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchOwnershipRecord {
    pub request: LaunchOwnershipRequest,
    pub phase: LaunchOwnershipPhase,
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
}

impl std::fmt::Debug for LaunchOwnershipRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LaunchOwnershipRecord")
            .field("attempt_id", &self.request.attempt_id)
            .field("phase", &self.phase)
            .field("revision", &self.revision)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchCreateOutcome {
    /// Only this result permits one native `ops.spawn` call in this process.
    Fresh(LaunchOwnershipRecord),
    /// A durable prior CAS exists. It never permits another native create.
    Replay(LaunchOwnershipRecord),
}

/// An exact, validated owner snapshot for Stop. Prepared and ambiguous create
/// rows are included so Stop cannot mistake a missing PID for no ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnedJobStopKind {
    Worker,
    Checker { check_id: CheckId, ordinal: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnedJobStopPhase {
    Prepared,
    CreateMayHaveStarted,
    Registered,
}

#[derive(Clone, PartialEq, Eq)]
pub struct OwnedJobStopTarget {
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub kind: OwnedJobStopKind,
    pub phase: OwnedJobStopPhase,
    pub revision: u64,
    pub job_name: Option<String>,
    pub launch_nonce: Option<String>,
    pub pid: Option<u32>,
    pub start_identity: Option<String>,
}

impl std::fmt::Debug for OwnedJobStopTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedJobStopTarget")
            .field("attempt_id", &self.attempt_id)
            .field("kind", &self.kind)
            .field("phase", &self.phase)
            .field("revision", &self.revision)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct LaunchCreateRequest {
    pub scope: RoutingScope,
    pub attempt_id: AttemptId,
    pub event_id: String,
    pub job_name: String,
    pub launch_nonce: String,
}

impl std::fmt::Debug for LaunchCreateRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LaunchCreateRequest")
            .field("attempt_id", &self.attempt_id)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchProcessRegistration {
    pub scope: RoutingScope,
    pub attempt_id: AttemptId,
    pub event_id: String,
    pub observed_job_name: String,
    pub pid: u32,
    pub start_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchSettlement {
    pub scope: RoutingScope,
    pub attempt_id: AttemptId,
    pub event_id: String,
    pub receipt_claim: PrivateArtifactClaim,
    pub receipt_ref: BlobRef,
}

impl PytxoStore {
    /// Read every unresolved native owner under one Store snapshot. The full
    /// implication scan makes a missing independent marker or changed SQL/JSON
    /// projection a recovery error, including for a terminal legacy run.
    pub fn owned_job_stop_snapshot(
        &self,
        domain: &DomainId,
        run: Option<&RunId>,
    ) -> Result<Vec<OwnedJobStopTarget>> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        validate_launch_ownership_implication(&tx)?;
        let mut targets = Vec::new();
        let mut workers = tx
            .prepare("SELECT attempt_id FROM attempt_launch_ownership ORDER BY attempt_id")
            .map_err(err)?;
        let worker_ids = workers
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        for id in worker_ids {
            let record = load_ownership(&tx, &AttemptId(id))?
                .ok_or_else(|| invalid("validated worker Stop owner disappeared"))?;
            if record.request.scope.domain_id != *domain
                || run.is_some_and(|run| record.request.scope.run_id != *run)
            {
                continue;
            }
            let phase = match record.phase {
                LaunchOwnershipPhase::Prepared => OwnedJobStopPhase::Prepared,
                LaunchOwnershipPhase::CreateMayHaveStarted => {
                    OwnedJobStopPhase::CreateMayHaveStarted
                }
                LaunchOwnershipPhase::Registered => OwnedJobStopPhase::Registered,
                LaunchOwnershipPhase::Settled | LaunchOwnershipPhase::ClosedNoLaunch => continue,
            };
            targets.push(OwnedJobStopTarget {
                scope: record.request.scope,
                task_id: record.request.task_id,
                attempt_id: record.request.attempt_id,
                kind: OwnedJobStopKind::Worker,
                phase,
                revision: record.revision,
                job_name: record.job_name,
                launch_nonce: record.launch_nonce,
                pid: record.pid,
                start_identity: record.start_identity,
            });
        }
        drop(workers);
        let mut checkers = tx
            .prepare(
                "SELECT attempt_id,ordinal FROM attempt_checker_ownership ORDER BY attempt_id,ordinal",
            )
            .map_err(err)?;
        let checker_ids = checkers
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        for (id, ordinal) in checker_ids {
            let record = crate::routing_checker::load_checker(&tx, &AttemptId(id), ordinal)?
                .ok_or_else(|| invalid("validated checker Stop owner disappeared"))?;
            if record.request.scope.domain_id != *domain
                || run.is_some_and(|run| record.request.scope.run_id != *run)
            {
                continue;
            }
            let phase = match record.phase {
                crate::routing_checker::CheckerOwnershipPhase::Prepared => {
                    OwnedJobStopPhase::Prepared
                }
                crate::routing_checker::CheckerOwnershipPhase::CreateMayHaveStarted => {
                    OwnedJobStopPhase::CreateMayHaveStarted
                }
                crate::routing_checker::CheckerOwnershipPhase::Registered => {
                    OwnedJobStopPhase::Registered
                }
                crate::routing_checker::CheckerOwnershipPhase::Settled
                | crate::routing_checker::CheckerOwnershipPhase::ClosedNoCreate => continue,
            };
            targets.push(OwnedJobStopTarget {
                scope: record.request.scope,
                task_id: record.request.task_id,
                attempt_id: record.request.attempt_id,
                kind: OwnedJobStopKind::Checker {
                    check_id: record.request.check_id,
                    ordinal,
                },
                phase,
                revision: record.revision,
                job_name: record.job_name,
                launch_nonce: record.launch_nonce,
                pid: record.pid,
                start_identity: record.start_identity,
            });
        }
        drop(checkers);
        tx.commit().map_err(err)?;
        Ok(targets)
    }

    /// The Catalog read is outside the domain write transaction. It proves
    /// the exact reservation was bound, not that a process exists or is safe.
    pub fn prepare_launch_ownership(
        &self,
        catalog: &Catalog,
        request: &LaunchOwnershipRequest,
    ) -> Result<LaunchOwnershipRecord> {
        validate_request(request)?;
        let observed_intent = load_intent(&self.conn, &request.reservation_id)?
            .ok_or_else(|| invalid("launch ownership has no capacity intent"))?;
        require(
            observed_intent.phase == CapacityIntentPhase::ReserveMayHaveStarted,
            "launch requires an unresolved started reserve intent",
        )?;
        let capacity = exact_catalog_reservation(catalog, &observed_intent)?;
        require(
            capacity.state == CapacityReservationState::Bound
                && capacity.launch_token.as_deref() == Some(&request.launch_token)
                && capacity.bound_at_ms.is_some()
                && capacity.recovery_evidence_id.is_none()
                && capacity.release_evidence.is_none(),
            "launch requires the exact bound Catalog reservation",
        )?;
        let tx = immediate(&self.conn)?;
        let intent = load_intent(&tx, &request.reservation_id)?
            .ok_or_else(|| invalid("launch capacity intent vanished"))?;
        require(
            intent == observed_intent,
            "launch capacity intent changed during Catalog read",
        )?;
        require(
            intent.request.scope == request.scope
                && intent.request.task_id == request.task_id
                && intent.request.reservation.attempt_id == request.attempt_id.0,
            "launch owner differs from capacity intent",
        )?;
        if let Some(existing) = load_ownership(&tx, &request.attempt_id)? {
            require(
                existing.request == *request,
                "launch ownership replay changed",
            )?;
            let claim = owner_claim(request, PrivateArtifactKind::InputManifest);
            require(
                exact_admitted_attempt(&tx, &claim)?
                    .is_some_and(|attempt| attempt.owned_launch_required),
                "launch ownership replay lost its independent attempt marker",
            )?;
            tx.commit().map_err(err)?;
            return Ok(existing);
        }
        let mut attempt = require_active_attempt(&tx, request)?;
        require(
            attempt.state == AttemptState::Preparing && !attempt.owned_launch_required,
            "launch ownership can only be prepared before launch checks",
        )?;
        let claim = owner_claim(request, PrivateArtifactKind::InputManifest);
        require(
            retained_attempt_reference_matches(
                &tx,
                &claim,
                PrivateArtifactKind::InputManifest,
                &attempt.input_manifest,
            )?,
            "launch input bytes are not retained for the exact attempt",
        )?;
        if let Some(handoff) = &attempt.handoff {
            require(
                retained_attempt_reference_matches(
                    &tx,
                    &claim,
                    PrivateArtifactKind::HandoffManifest,
                    handoff,
                )?,
                "launch handoff manifest is not retained for the exact attempt",
            )?;
        }
        let record = LaunchOwnershipRecord {
            request: request.clone(),
            phase: LaunchOwnershipPhase::Prepared,
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
        };
        tx.execute(
            "INSERT INTO attempt_launch_ownership
             (attempt_id,domain_id,run_id,task_id,reservation_id,launch_token,job_name,launch_nonce,pid,start_identity,settlement_artifact_id,phase,revision,record_json)
             VALUES (?1,?2,?3,?4,?5,?6,NULL,NULL,NULL,NULL,NULL,?7,?8,?9)",
            params![record.request.attempt_id.0,record.request.scope.domain_id.0,
                record.request.scope.run_id.0,record.request.task_id.0,
                record.request.reservation_id,record.request.launch_token,
                record.phase.as_str(),record.revision,json(&record)?],
        ).map_err(err)?;
        // Independent attempt marker and owner row commit together. A lost
        // child row can no longer make this attempt look legacy/unowned.
        attempt.owned_launch_required = true;
        crate::routing::save_attempt(&tx, &mut attempt)?;
        tx.commit().map_err(err)?;
        self.launch_ownership(&request.attempt_id)?
            .ok_or_else(|| invalid("launch ownership not persisted"))
    }

    /// Called by Runner authorize with its immutable Job name/nonce, before
    /// native process creation. A crash after Fresh remains recovery-owned.
    pub fn authorize_launch_create(
        &self,
        request: &LaunchCreateRequest,
    ) -> Result<LaunchCreateOutcome> {
        for id in [&request.event_id, &request.job_name, &request.launch_nonce] {
            require_id(id)?;
        }
        let tx = immediate(&self.conn)?;
        let mut record = required_ownership(&tx, &request.attempt_id)?;
        require(
            record.request.scope == request.scope,
            "launch create scope mismatch",
        )?;
        if record.create_event_id.as_deref() == Some(&request.event_id) {
            require(
                record.job_name.as_deref() == Some(&request.job_name)
                    && record.launch_nonce.as_deref() == Some(&request.launch_nonce)
                    && record.phase != LaunchOwnershipPhase::Prepared,
                "launch create replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(LaunchCreateOutcome::Replay(record));
        }
        require(
            record.phase == LaunchOwnershipPhase::Prepared,
            "native create already may have started",
        )?;
        require_distinct_checker_job_name(&tx, &request.job_name)?;
        let intent = load_intent(&tx, &record.request.reservation_id)?
            .ok_or_else(|| invalid("launch capacity intent absent"))?;
        require(
            intent.phase == CapacityIntentPhase::ReserveMayHaveStarted
                && intent.request.scope == record.request.scope,
            "launch capacity ownership changed",
        )?;
        let attempt = require_active_attempt(&tx, &record.request)?;
        require(
            attempt.state == AttemptState::Launching && attempt.owned_launch_required,
            "native create requires the exact pre-spawn launching state",
        )?;
        crate::routing_private::require_retained_launch_checks(
            &tx,
            &attempt,
            attempt
                .receipts
                .launch_checks
                .as_ref()
                .ok_or_else(|| invalid("owned launch checks are absent"))?,
        )?;
        record.phase = LaunchOwnershipPhase::CreateMayHaveStarted;
        record.create_event_id = Some(request.event_id.clone());
        record.job_name = Some(request.job_name.clone());
        record.launch_nonce = Some(request.launch_nonce.clone());
        save_ownership(&tx, &mut record, LaunchOwnershipPhase::Prepared)?;
        tx.commit().map_err(err)?;
        let persisted = self
            .launch_ownership(&request.attempt_id)?
            .ok_or_else(|| invalid("launch create CAS not persisted"))?;
        Ok(LaunchCreateOutcome::Fresh(persisted))
    }

    pub fn register_launch_process(
        &self,
        request: &LaunchProcessRegistration,
    ) -> Result<LaunchOwnershipRecord> {
        require_id(&request.event_id)?;
        require_id(&request.observed_job_name)?;
        require_id(&request.start_identity)?;
        require(request.pid > 0, "registered native PID is zero")?;
        let tx = immediate(&self.conn)?;
        let mut record = required_ownership(&tx, &request.attempt_id)?;
        require(
            record.request.scope == request.scope,
            "registered process scope mismatch",
        )?;
        if record.register_event_id.as_deref() == Some(&request.event_id) {
            require(
                record.job_name.as_deref() == Some(&request.observed_job_name)
                    && record.pid == Some(request.pid)
                    && record.start_identity.as_deref() == Some(&request.start_identity),
                "registered process replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == LaunchOwnershipPhase::CreateMayHaveStarted
                && record.job_name.as_deref() == Some(&request.observed_job_name),
            "registered process does not match one-use create",
        )?;
        record.phase = LaunchOwnershipPhase::Registered;
        record.register_event_id = Some(request.event_id.clone());
        record.pid = Some(request.pid);
        record.start_identity = Some(request.start_identity.clone());
        save_ownership(&tx, &mut record, LaunchOwnershipPhase::CreateMayHaveStarted)?;
        tx.commit().map_err(err)?;
        self.launch_ownership(&request.attempt_id)?
            .ok_or_else(|| invalid("process registration not persisted"))
    }

    /// The receipt is read back by exact owner/kind/event/bytes. This records
    /// trusted controller attestation; the later supervisor must observe the
    /// actual named Job at zero and keep the receipt bytes before calling us.
    pub fn settle_launch_ownership(
        &self,
        request: &LaunchSettlement,
    ) -> Result<LaunchOwnershipRecord> {
        require_id(&request.event_id)?;
        require(
            request.receipt_claim.kind == PrivateArtifactKind::ControllerReceipt,
            "launch settlement requires a controller receipt",
        )?;
        let receipt = self.read_controller_receipt(&request.receipt_claim, &request.receipt_ref)?;
        let tx = immediate(&self.conn)?;
        let mut record = required_ownership(&tx, &request.attempt_id)?;
        require(
            record.request.scope == request.scope
                && request.receipt_claim.scope == record.request.scope
                && request.receipt_claim.task_id == record.request.task_id
                && request.receipt_claim.attempt_id == record.request.attempt_id
                && request.receipt_claim.reservation_id == record.request.reservation_id,
            "settlement receipt owner mismatch",
        )?;
        require(
            retained_reference_matches(
                &tx,
                &request.receipt_claim,
                PrivateArtifactKind::ControllerReceipt,
                &request.receipt_ref,
            )?,
            "settlement receipt changed before CAS",
        )?;
        require(
            receipt.source == ReceiptSource::OwnedJobObservation,
            "settlement requires owned Job observation",
        )?;
        let ControllerObservation::NativeJobZero {
            job_name,
            launch_nonce,
            active_processes,
        } = receipt.observation
        else {
            return Err(invalid("settlement receipt is not Job-zero"));
        };
        require(
            record.job_name.as_deref() == Some(&job_name)
                && record.launch_nonce.as_deref() == Some(&launch_nonce)
                && active_processes == 0,
            "settlement Job identity or active count mismatch",
        )?;
        if record.settlement_event_id.as_deref() == Some(&request.event_id) {
            require(
                record.settlement_artifact_id.as_deref()
                    == Some(&request.receipt_claim.artifact_id)
                    && record.settlement_blob.as_ref() == Some(&request.receipt_ref),
                "launch settlement replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == LaunchOwnershipPhase::Registered,
            "launch settlement requires registered native identity",
        )?;
        record.phase = LaunchOwnershipPhase::Settled;
        record.settlement_event_id = Some(request.event_id.clone());
        record.settlement_artifact_id = Some(request.receipt_claim.artifact_id.clone());
        record.settlement_blob = Some(request.receipt_ref.clone());
        save_ownership(&tx, &mut record, LaunchOwnershipPhase::Registered)?;
        tx.commit().map_err(err)?;
        self.launch_ownership(&request.attempt_id)?
            .ok_or_else(|| invalid("launch settlement not persisted"))
    }

    /// Close a Prepared owner only after an exact admitted-attempt no-launch
    /// receipt and durable cancellation. The attempt may already be Launching,
    /// but the owner must still be Prepared: no native create was authorized.
    pub fn close_prepared_launch_without_worker(
        &self,
        request: &LaunchSettlement,
    ) -> Result<LaunchOwnershipRecord> {
        require_id(&request.event_id)?;
        require(
            request.receipt_claim.kind == PrivateArtifactKind::ControllerReceipt,
            "prepared closure requires admitted controller receipt",
        )?;
        let receipt = self.read_controller_receipt(&request.receipt_claim, &request.receipt_ref)?;
        require(
            receipt.source == ReceiptSource::TrustedController,
            "prepared closure requires controller no-launch observation",
        )?;
        let ControllerObservation::AdmittedNoLaunch {
            cancellation_event_id,
        } = receipt.observation
        else {
            return Err(invalid("prepared closure receipt is not no-launch"));
        };
        let tx = immediate(&self.conn)?;
        let mut record = required_ownership(&tx, &request.attempt_id)?;
        require(
            record.request.scope == request.scope
                && request.receipt_claim.scope == record.request.scope
                && request.receipt_claim.task_id == record.request.task_id
                && request.receipt_claim.attempt_id == record.request.attempt_id
                && request.receipt_claim.reservation_id == record.request.reservation_id,
            "prepared closure receipt owner mismatch",
        )?;
        require(
            retained_reference_matches(
                &tx,
                &request.receipt_claim,
                PrivateArtifactKind::ControllerReceipt,
                &request.receipt_ref,
            )?,
            "prepared closure receipt changed before CAS",
        )?;
        require_cancelled_event(&tx, &request.receipt_claim, &cancellation_event_id)?;
        if record.settlement_event_id.as_deref() == Some(&request.event_id) {
            require(
                record.phase == LaunchOwnershipPhase::ClosedNoLaunch
                    && record.settlement_artifact_id.as_deref()
                        == Some(&request.receipt_claim.artifact_id)
                    && record.settlement_blob.as_ref() == Some(&request.receipt_ref),
                "prepared closure replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        let owner = owner_claim(&record.request, PrivateArtifactKind::InputManifest);
        let attempt = exact_admitted_attempt(&tx, &owner)?
            .ok_or_else(|| invalid("prepared closure attempt absent"))?;
        require(
            matches!(
                attempt.state,
                AttemptState::Preparing | AttemptState::Launching | AttemptState::RecoveryRequired
            ) && !attempt.ownership_released
                && attempt.receipts.process_identity.is_none(),
            "no-launch closure requires a pre-create attempt state",
        )?;
        require(
            record.phase == LaunchOwnershipPhase::Prepared,
            "native creation may have started; no-launch closure denied",
        )?;
        record.phase = LaunchOwnershipPhase::ClosedNoLaunch;
        record.settlement_event_id = Some(request.event_id.clone());
        record.settlement_artifact_id = Some(request.receipt_claim.artifact_id.clone());
        record.settlement_blob = Some(request.receipt_ref.clone());
        save_ownership(&tx, &mut record, LaunchOwnershipPhase::Prepared)?;
        tx.commit().map_err(err)?;
        self.launch_ownership(&request.attempt_id)?
            .ok_or_else(|| invalid("prepared closure not persisted"))
    }

    pub fn launch_ownership(
        &self,
        attempt_id: &AttemptId,
    ) -> Result<Option<LaunchOwnershipRecord>> {
        load_ownership(&self.conn, attempt_id)
    }

    pub fn unresolved_launch_ownership_scopes(&self) -> Result<Vec<RoutingScope>> {
        validate_launch_ownership_implication(&self.conn)?;
        let mut stmt = self
            .conn
            .prepare("SELECT attempt_id FROM attempt_launch_ownership ORDER BY attempt_id")
            .map_err(err)?;
        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        let mut scopes = Vec::new();
        for id in ids {
            let record = load_ownership(&self.conn, &AttemptId(id))?
                .ok_or_else(|| invalid("launch ownership vanished during scan"))?;
            if !matches!(
                record.phase,
                LaunchOwnershipPhase::Settled | LaunchOwnershipPhase::ClosedNoLaunch
            ) && !scopes.contains(&record.request.scope)
            {
                scopes.push(record.request.scope);
            }
        }
        Ok(scopes)
    }
}

/// Called inside the routing transition transaction before domain ownership
/// can be released. Existing claim/Stop scans then remain conservative.
pub(crate) fn require_launch_settled_for_release(
    conn: &Connection,
    attempt: &RoutedAttemptRecord,
) -> Result<()> {
    let owner = load_ownership(conn, &attempt.attempt_id)?;
    require(
        !attempt.owned_launch_required || owner.is_some(),
        "required native launch ownership row is missing",
    )?;
    if let Some(record) = owner {
        require(
            matches!(
                record.phase,
                LaunchOwnershipPhase::Settled | LaunchOwnershipPhase::ClosedNoLaunch
            ),
            "native launch ownership is not settled",
        )?;
    }
    crate::routing_checker::require_checkers_settled_for_release(conn, attempt)?;
    Ok(())
}

/// An admitted KnownUnused host release must refer to the exact immutable
/// no-create closure, not merely a terminal attempt or an arbitrary digest.
pub(crate) fn closed_no_launch_token_for_release(
    conn: &Connection,
    claim: &PrivateArtifactClaim,
    reference: &BlobRef,
) -> Result<String> {
    let record = required_ownership(conn, &claim.attempt_id)?;
    require(
        record.phase == LaunchOwnershipPhase::ClosedNoLaunch
            && record.request.scope == claim.scope
            && record.request.task_id == claim.task_id
            && record.request.attempt_id == claim.attempt_id
            && record.request.reservation_id == claim.reservation_id
            && record.settlement_artifact_id.as_deref() == Some(claim.artifact_id.as_str())
            && record.settlement_blob.as_ref() == Some(reference),
        "admitted host release lacks exact closed no-launch owner",
    )?;
    Ok(record.request.launch_token)
}

/// A positive Prepared no-create closure is terminal for that launch token.
/// The matching retained receipt is required to release the attempt hold.
pub(crate) fn require_owned_process_transition(
    conn: &Connection,
    attempt: &RoutedAttemptRecord,
    from: AttemptState,
    to: AttemptState,
    receipts: &RoutingReceipts,
) -> Result<()> {
    let owner = load_ownership(conn, &attempt.attempt_id)?;
    require(
        !attempt.owned_launch_required || owner.is_some(),
        "required native launch ownership row is missing",
    )?;
    if let Some(record) = owner {
        if record.phase == LaunchOwnershipPhase::ClosedNoLaunch {
            require(
                ((matches!(from, AttemptState::Preparing | AttemptState::Launching)
                    && matches!(to, AttemptState::FailedNoLaunch | AttemptState::Cancelled))
                    || (from == AttemptState::RecoveryRequired
                        && to == AttemptState::FailedNoLaunch))
                    && record.settlement_blob.as_ref().is_some_and(|blob| {
                        receipts.no_worker_created.as_ref() == Some(&blob.digest)
                    }),
                "closed no-launch ownership cannot become a process or winner",
            )?;
        } else if to == AttemptState::Sealing || to.is_terminal() {
            let settled = record.settlement_blob.as_ref();
            require(
                record.phase == LaunchOwnershipPhase::Settled
                    && settled
                        .is_some_and(|blob| receipts.quiescence.as_ref() == Some(&blob.digest))
                    && to != AttemptState::FailedNoLaunch,
                "owned worker cannot seal or release without its retained Job-zero receipt",
            )?;
            if to == AttemptState::Passed {
                let aggregate =
                    crate::routing_checker::require_owned_checker_pass_digest(conn, attempt)?;
                require(
                    receipts.checks.as_ref() == Some(&aggregate),
                    "owned worker check digest does not bind every reviewed checker",
                )?;
            }
        }
    }
    Ok(())
}

/// Fail closed during the existing Store scan used by claim and Stop if a
/// launch row is malformed or an unsettled owner lost its attempt hold.
pub(crate) fn validate_launch_ownership_implication(conn: &Connection) -> Result<()> {
    let mut stmt = conn
        .prepare("SELECT attempt_id FROM attempt_launch_ownership ORDER BY attempt_id")
        .map_err(err)?;
    let ids = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    for id in ids {
        let record = required_ownership(conn, &AttemptId(id))?;
        let owner = owner_claim(&record.request, PrivateArtifactKind::InputManifest);
        let attempt = exact_admitted_attempt(conn, &owner)?
            .ok_or_else(|| invalid("launch owner has no exact admitted attempt"))?;
        require(
            attempt.owned_launch_required,
            "launch owner has no independent attempt marker",
        )?;
        if attempt.state == AttemptState::Passed {
            let aggregate =
                crate::routing_checker::require_owned_checker_pass_digest(conn, &attempt)?;
            require(
                attempt.receipts.checks.as_ref() == Some(&aggregate),
                "passed owned worker lost its checker aggregate",
            )?;
        }
        if !matches!(
            record.phase,
            LaunchOwnershipPhase::Settled | LaunchOwnershipPhase::ClosedNoLaunch
        ) {
            require(
                !attempt.ownership_released,
                "unsettled launch owner lost attempt domain hold",
            )?;
        } else if record.phase == LaunchOwnershipPhase::ClosedNoLaunch {
            let settlement = record
                .settlement_blob
                .as_ref()
                .ok_or_else(|| invalid("closed no-launch owner lost settlement"))?;
            let consistent = if attempt.state.is_terminal() {
                matches!(
                    attempt.state,
                    AttemptState::FailedNoLaunch | AttemptState::Cancelled
                ) && attempt.ownership_released
                    && attempt.receipts.no_worker_created.as_ref() == Some(&settlement.digest)
            } else {
                matches!(
                    attempt.state,
                    AttemptState::Preparing
                        | AttemptState::Launching
                        | AttemptState::RecoveryRequired
                ) && !attempt.ownership_released
                    && attempt.receipts.no_worker_created.is_none()
            };
            require(
                consistent && attempt.receipts.process_identity.is_none(),
                "closed no-launch owner conflicts with attempt process state",
            )?;
        }
    }
    let mut stmt = conn
        .prepare("SELECT attempt_id,revision,record_json FROM routing_attempts ORDER BY attempt_id")
        .map_err(err)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(err)?;
    for row in rows {
        let (id, revision, raw) = row.map_err(err)?;
        let attempt: RoutedAttemptRecord = serde_json::from_str(&raw).map_err(err)?;
        require(
            attempt.attempt_id.0 == id && attempt.revision == revision,
            "launch attempt marker projection changed",
        )?;
        if attempt.owned_launch_required {
            let owner = load_ownership(conn, &attempt.attempt_id)?
                .ok_or_else(|| invalid("required launch owner row is missing"))?;
            require(
                owner.request.scope == attempt.scope
                    && owner.request.task_id == attempt.task_id
                    && owner.request.attempt_id == attempt.attempt_id
                    && owner.request.reservation_id == attempt.capacity_reservation,
                "launch owner marker differs from admitted attempt",
            )?;
        }
    }
    crate::routing_checker::validate_checker_ownership_implication(conn)
}

fn owner_claim(
    request: &LaunchOwnershipRequest,
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

pub(crate) fn require_active_attempt(
    conn: &Connection,
    request: &LaunchOwnershipRequest,
) -> Result<RoutedAttemptRecord> {
    let mission: Option<(String, u64, bool, String)> = conn
        .query_row(
            "SELECT m.domain_id,m.cancel_epoch,m.cancelled,r.status FROM routing_missions m
         JOIN runs r ON r.id=m.run_id WHERE m.run_id=?1",
            [&request.scope.run_id.0],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(err)?;
    let Some((domain, epoch, cancelled, status)) = mission else {
        return Err(invalid("launch mission absent"));
    };
    require(
        domain == request.scope.domain_id.0
            && !cancelled
            && matches!(status.as_str(), "starting" | "running"),
        "launch mission is not active and uncancelled",
    )?;
    let task: Option<(u64, String)> = conn
        .query_row(
            "SELECT revision,record_json FROM routing_tasks WHERE run_id=?1 AND task_id=?2",
            params![request.scope.run_id.0, request.task_id.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    let Some((revision, body)) = task else {
        return Err(invalid("launch task absent"));
    };
    let task: RoutedTaskRecord = serde_json::from_str(&body).map_err(err)?;
    require(
        task.revision == revision
            && task.registration.contract.task_id == request.task_id
            && task.state == TaskRoutingState::Active
            && task.current_attempt.as_ref() == Some(&request.attempt_id)
            && json(&task)? == body,
        "launch task projection or active owner changed",
    )?;
    let claim = owner_claim(request, PrivateArtifactKind::InputManifest);
    let attempt =
        exact_admitted_attempt(conn, &claim)?.ok_or_else(|| invalid("launch attempt absent"))?;
    require(
        !attempt.ownership_released
            && !attempt.state.is_terminal()
            && attempt.cancel_epoch == epoch,
        "launch attempt ownership or cancellation epoch changed",
    )?;
    Ok(attempt)
}

pub(crate) fn load_ownership(
    conn: &Connection,
    attempt_id: &AttemptId,
) -> Result<Option<LaunchOwnershipRecord>> {
    struct Projection {
        domain: String,
        run: String,
        task: String,
        reservation: String,
        token: String,
        job: Option<String>,
        nonce: Option<String>,
        pid: Option<u32>,
        start: Option<String>,
        settlement: Option<String>,
        phase: String,
        revision: u64,
        body: String,
    }
    let row: Option<Projection> = conn.query_row(
        "SELECT domain_id,run_id,task_id,reservation_id,launch_token,job_name,launch_nonce,pid,start_identity,settlement_artifact_id,phase,revision,record_json
         FROM attempt_launch_ownership WHERE attempt_id=?1",
        [&attempt_id.0], |row| Ok(Projection {
            domain:row.get(0)?,run:row.get(1)?,task:row.get(2)?,reservation:row.get(3)?,
            token:row.get(4)?,job:row.get(5)?,nonce:row.get(6)?,pid:row.get(7)?,
            start:row.get(8)?,settlement:row.get(9)?,phase:row.get(10)?,
            revision:row.get(11)?,body:row.get(12)?,
        }),
    ).optional().map_err(err)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let record: LaunchOwnershipRecord = serde_json::from_str(&row.body).map_err(err)?;
    validate_record(&record)?;
    require(
        record.request.attempt_id == *attempt_id
            && record.request.scope.domain_id.0 == row.domain
            && record.request.scope.run_id.0 == row.run
            && record.request.task_id.0 == row.task
            && record.request.reservation_id == row.reservation
            && record.request.launch_token == row.token
            && record.job_name == row.job
            && record.launch_nonce == row.nonce
            && record.pid == row.pid
            && record.start_identity == row.start
            && record.settlement_artifact_id == row.settlement
            && record.phase.as_str() == row.phase
            && record.revision == row.revision
            && json(&record)? == row.body,
        "launch ownership SQL/JSON projection changed",
    )?;
    if let Some(job_name) = &record.job_name {
        require_distinct_checker_job_name(conn, job_name)?;
    }
    if let (Some(artifact_id), Some(reference)) =
        (&record.settlement_artifact_id, &record.settlement_blob)
    {
        let (claim, receipt) = exact_retained_receipt_by_id(conn, artifact_id, reference)?;
        require(
            claim.scope == record.request.scope
                && claim.task_id == record.request.task_id
                && claim.attempt_id == record.request.attempt_id
                && claim.reservation_id == record.request.reservation_id,
            "launch settlement receipt owner changed",
        )?;
        match (&record.phase, receipt.source, receipt.observation) {
            (
                LaunchOwnershipPhase::Settled,
                ReceiptSource::OwnedJobObservation,
                ControllerObservation::NativeJobZero {
                    job_name,
                    launch_nonce,
                    active_processes,
                },
            ) => {
                require(
                    record.job_name.as_deref() == Some(&job_name)
                        && record.launch_nonce.as_deref() == Some(&launch_nonce)
                        && active_processes == 0,
                    "launch settlement Job-zero receipt changed",
                )?;
            }
            (
                LaunchOwnershipPhase::ClosedNoLaunch,
                ReceiptSource::TrustedController,
                ControllerObservation::AdmittedNoLaunch {
                    cancellation_event_id,
                },
            ) => {
                require_cancelled_event(conn, &claim, &cancellation_event_id)?;
            }
            _ => return Err(invalid("launch settlement receipt observation changed")),
        }
    }
    Ok(Some(record))
}

fn required_ownership(conn: &Connection, attempt_id: &AttemptId) -> Result<LaunchOwnershipRecord> {
    load_ownership(conn, attempt_id)?.ok_or_else(|| invalid("launch ownership absent"))
}

fn require_distinct_checker_job_name(conn: &Connection, job_name: &str) -> Result<()> {
    let collides: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM attempt_checker_ownership WHERE job_name=?1)",
            [job_name],
            |row| row.get(0),
        )
        .map_err(err)?;
    require(!collides, "worker Job name collides with checker ownership")
}

fn save_ownership(
    tx: &Transaction<'_>,
    record: &mut LaunchOwnershipRecord,
    prior: LaunchOwnershipPhase,
) -> Result<()> {
    let prior_revision = record.revision;
    record.revision = prior_revision
        .checked_add(1)
        .ok_or_else(|| invalid("launch revision overflow"))?;
    validate_record(record)?;
    let changed = tx.execute(
        "UPDATE attempt_launch_ownership SET job_name=?1,launch_nonce=?2,pid=?3,start_identity=?4,
         settlement_artifact_id=?5,phase=?6,revision=?7,record_json=?8
         WHERE attempt_id=?9 AND phase=?10 AND revision=?11",
        params![record.job_name,record.launch_nonce,record.pid,record.start_identity,
            record.settlement_artifact_id,record.phase.as_str(),record.revision,
            json(record)?,record.request.attempt_id.0,prior.as_str(),prior_revision],
    ).map_err(err)?;
    require(changed == 1, "launch ownership CAS conflict")
}

fn validate_request(request: &LaunchOwnershipRequest) -> Result<()> {
    for id in [
        &request.scope.domain_id.0,
        &request.scope.run_id.0,
        &request.task_id.0,
        &request.attempt_id.0,
        &request.reservation_id,
        &request.launch_token,
        &request.event_id,
    ] {
        require_id(id)?;
    }
    Ok(())
}
fn validate_record(record: &LaunchOwnershipRecord) -> Result<()> {
    validate_request(&record.request)?;
    require(record.revision > 0, "launch ownership revision invalid")?;
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
        LaunchOwnershipPhase::Prepared => {
            record.create_event_id.is_none()
                && record.job_name.is_none()
                && record.launch_nonce.is_none()
                && record.register_event_id.is_none()
                && record.pid.is_none()
                && record.start_identity.is_none()
                && record.settlement_event_id.is_none()
                && record.settlement_artifact_id.is_none()
                && record.settlement_blob.is_none()
        }
        LaunchOwnershipPhase::CreateMayHaveStarted => {
            record.create_event_id.is_some()
                && record.job_name.is_some()
                && record.launch_nonce.is_some()
                && record.register_event_id.is_none()
                && record.pid.is_none()
                && record.start_identity.is_none()
                && record.settlement_event_id.is_none()
                && record.settlement_artifact_id.is_none()
                && record.settlement_blob.is_none()
        }
        LaunchOwnershipPhase::Registered => {
            record.create_event_id.is_some()
                && record.job_name.is_some()
                && record.launch_nonce.is_some()
                && record.register_event_id.is_some()
                && record.pid.is_some_and(|pid| pid > 0)
                && record.start_identity.is_some()
                && record.settlement_event_id.is_none()
                && record.settlement_artifact_id.is_none()
                && record.settlement_blob.is_none()
        }
        LaunchOwnershipPhase::Settled => {
            record.create_event_id.is_some()
                && record.job_name.is_some()
                && record.launch_nonce.is_some()
                && record.register_event_id.is_some()
                && record.pid.is_some_and(|pid| pid > 0)
                && record.start_identity.is_some()
                && record.settlement_event_id.is_some()
                && record.settlement_artifact_id.is_some()
                && record.settlement_blob.is_some()
        }
        LaunchOwnershipPhase::ClosedNoLaunch => {
            record.create_event_id.is_none()
                && record.job_name.is_none()
                && record.launch_nonce.is_none()
                && record.register_event_id.is_none()
                && record.pid.is_none()
                && record.start_identity.is_none()
                && record.settlement_event_id.is_some()
                && record.settlement_artifact_id.is_some()
                && record.settlement_blob.is_some()
        }
    };
    require(valid, "launch ownership phase projection inconsistent")
}

fn immediate(conn: &Connection) -> Result<Transaction<'_>> {
    Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(err)
}
fn json(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(err)
}
fn require_id(id: &str) -> Result<()> {
    require(!id.trim().is_empty(), "launch identity empty")
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
