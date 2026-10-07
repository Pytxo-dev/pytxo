//! Private domain ownership spanning a Store intent and one exact host Catalog.
//! No method here grants worker launch, billing authority, or verification.

use std::collections::BTreeSet;
use std::path::PathBuf;

use pytxo_core::routing::{canonical_digest, AttemptState, BlobRef};
use pytxo_core::{DomainId, PytxoError, Result, RunId, TaskId};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::capacity::{
    canonical_resources, validate_release_request, validate_reservation_request,
    CapacityBoundReleaseRequest, CapacityReleaseEvidence, CapacityReleaseEvidenceKind,
    CapacityReleaseRequest, CapacityReservationRecord, CapacityReservationRequest,
    CapacityReservationState, CapacityResourceRequest,
};
use crate::routing::{RoutedAttemptRecord, RoutedTaskRecord, RoutingScope, TaskRoutingState};
use crate::routing_checker::{
    load_checker, require_checkers_settled_for_release, CheckerOwnershipPhase,
};
use crate::routing_launch::{
    closed_no_launch_token_for_release, load_ownership, LaunchOwnershipPhase,
};
use crate::routing_private::{
    exact_admitted_attempt, exact_retained_receipt_by_id, require_cancelled_event,
    ControllerObservation, PrivateArtifactClaim, PrivateArtifactKind, ReceiptSource,
};
use crate::{Catalog, PytxoStore};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingCapacityIntentRequest {
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub reservation: CapacityReservationRequest,
    pub event_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapacityIntentPhase {
    Prepared,
    ReserveMayHaveStarted,
    ReleaseProofBound,
    Closed,
}

impl CapacityIntentPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::ReserveMayHaveStarted => "reserve_may_have_started",
            Self::ReleaseProofBound => "release_proof_bound",
            Self::Closed => "closed",
        }
    }
}

/// Controller-private record. The Catalog locator is only a lookup hint; a
/// later file at this path does not prove that a reserve call never happened.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingCapacityIntent {
    pub request: RoutingCapacityIntentRequest,
    pub catalog_locator: PathBuf,
    pub phase: CapacityIntentPhase,
    pub revision: u64,
    pub reserve_event_id: Option<String>,
    pub release_proof_event_id: Option<String>,
    pub expected_release: Option<CapacityReleaseEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admitted_no_launch_release: Option<AdmittedNoLaunchRelease>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admitted_quiescent_release: Option<AdmittedQuiescentRelease>,
    pub close_event_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmittedNoLaunchRelease {
    pub launch_token: String,
    pub receipt_claim: PrivateArtifactClaim,
    pub receipt_ref: BlobRef,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmittedQuiescentRelease {
    pub launch_token: String,
    pub worker_receipt_claim: PrivateArtifactClaim,
    pub worker_receipt_ref: BlobRef,
    pub checker_receipts: Vec<QuiescentCheckerReceipt>,
}

impl std::fmt::Debug for AdmittedQuiescentRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdmittedQuiescentRelease")
            .field("attempt_id", &self.worker_receipt_claim.attempt_id)
            .field("checker_count", &self.checker_receipts.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuiescentCheckerReceipt {
    pub ordinal: u32,
    pub receipt_claim: PrivateArtifactClaim,
    pub receipt_ref: BlobRef,
}

impl std::fmt::Debug for RoutingCapacityIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoutingCapacityIntent")
            .field("reservation_id", &self.request.reservation.reservation_id)
            .field("phase", &self.phase)
            .field("revision", &self.revision)
            .finish_non_exhaustive()
    }
}

impl PytxoStore {
    /// Persist the immutable exact request before any Catalog reserve call.
    /// The caller must hold the domain gate when preparing a live attempt.
    pub fn register_capacity_intent(
        &self,
        catalog: &Catalog,
        request: &RoutingCapacityIntentRequest,
    ) -> Result<RoutingCapacityIntent> {
        validate_intent_request(request)?;
        let locator = catalog.opened_path();
        if let Some(existing) = self.capacity_intent(&request.reservation.reservation_id)? {
            require(
                existing.request == *request && existing.catalog_locator == locator,
                "capacity intent replay differs from immutable request or Catalog",
            )?;
            return Ok(existing);
        }
        require(
            self.unreconciled_registered_routing_scopes()?
                .contains(&request.scope),
            "capacity intent requires an uncancelled registered mission",
        )?;

        let tx = immediate(&self.conn)?;
        if let Some(existing) = load_intent(&tx, &request.reservation.reservation_id)? {
            require(
                existing.request == *request && existing.catalog_locator == locator,
                "capacity intent replay differs from immutable request or Catalog",
            )?;
            tx.commit().map_err(err)?;
            return Ok(existing);
        }
        require_live_registered_task(&tx, request)?;
        let record = RoutingCapacityIntent {
            request: request.clone(),
            catalog_locator: locator.to_path_buf(),
            phase: CapacityIntentPhase::Prepared,
            revision: 1,
            reserve_event_id: None,
            release_proof_event_id: None,
            expected_release: None,
            admitted_no_launch_release: None,
            admitted_quiescent_release: None,
            close_event_id: None,
        };
        tx.execute(
            "INSERT INTO routing_capacity_intents
             (reservation_id,domain_id,run_id,task_id,attempt_id,creation_event_id,phase,revision,record_json)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                record.request.reservation.reservation_id,
                record.request.scope.domain_id.0,
                record.request.scope.run_id.0,
                record.request.task_id.0,
                record.request.reservation.attempt_id,
                record.request.event_id,
                record.phase.as_str(),
                record.revision,
                json(&record)?,
            ],
        )
        .map_err(err)?;
        tx.commit().map_err(err)?;
        persisted(self, &request.reservation.reservation_id)
    }

    /// Must be durable before the controller invokes `Catalog::reserve_capacity`.
    pub fn mark_capacity_reserve_may_have_started(
        &self,
        reservation_id: &str,
        event_id: &str,
    ) -> Result<RoutingCapacityIntent> {
        require_id(event_id, "reserve event")?;
        let tx = immediate(&self.conn)?;
        let mut record = required_intent(&tx, reservation_id)?;
        if record.reserve_event_id.as_deref() == Some(event_id) {
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CapacityIntentPhase::Prepared,
            "reserve call cannot begin from this capacity intent phase",
        )?;
        require_live_registered_task(&tx, &record.request)?;
        record.phase = CapacityIntentPhase::ReserveMayHaveStarted;
        record.reserve_event_id = Some(event_id.into());
        save_intent(&tx, &mut record, CapacityIntentPhase::Prepared)?;
        tx.commit().map_err(err)?;
        persisted(self, reservation_id)
    }

    /// Distinct from post-call closure: this cannot close an ambiguous reserve.
    pub fn close_unissued_capacity_intent(
        &self,
        reservation_id: &str,
        event_id: &str,
    ) -> Result<RoutingCapacityIntent> {
        require_id(event_id, "pre-call closure event")?;
        let tx = immediate(&self.conn)?;
        let mut record = required_intent(&tx, reservation_id)?;
        if record.phase == CapacityIntentPhase::Closed
            && record.reserve_event_id.is_none()
            && record.close_event_id.as_deref() == Some(event_id)
        {
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CapacityIntentPhase::Prepared,
            "capacity reserve may have started; absence cannot close intent",
        )?;
        record.phase = CapacityIntentPhase::Closed;
        record.close_event_id = Some(event_id.into());
        save_intent(&tx, &mut record, CapacityIntentPhase::Prepared)?;
        tx.commit().map_err(err)?;
        persisted(self, reservation_id)
    }

    /// Persist positive no-worker release proof before asking Catalog to release.
    /// The trusted caller must hold the domain gate and publish cancellation.
    pub fn bind_capacity_release_proof(
        &self,
        catalog: &Catalog,
        reservation_id: &str,
        event_id: &str,
        proof: &CapacityReleaseEvidence,
    ) -> Result<RoutingCapacityIntent> {
        require_id(event_id, "release proof event")?;
        validate_release_request(&CapacityReleaseRequest {
            reservation_id: reservation_id.into(),
            evidence: proof.clone(),
        })?;
        require(
            proof.kind == CapacityReleaseEvidenceKind::KnownUnused,
            "no-worker intent requires KnownUnused proof",
        )?;
        let observed = required_intent(&self.conn, reservation_id)?;
        let observed_reservation = if observed.phase == CapacityIntentPhase::ReserveMayHaveStarted {
            Some(exact_catalog_reservation(catalog, &observed)?)
        } else {
            None
        };
        let tx = immediate(&self.conn)?;
        let mut record = required_intent(&tx, reservation_id)?;
        require(
            record == observed,
            "capacity intent changed during Catalog readback",
        )?;
        if record.release_proof_event_id.as_deref() == Some(event_id)
            && record.expected_release.as_ref() == Some(proof)
            && record.admitted_no_launch_release.is_none()
            && record.admitted_quiescent_release.is_none()
        {
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CapacityIntentPhase::ReserveMayHaveStarted,
            "release proof requires a started reserve intent",
        )?;
        let cancelled: Option<i64> = tx
            .query_row(
                "SELECT cancelled FROM routing_missions WHERE run_id=?1 AND domain_id=?2",
                params![
                    record.request.scope.run_id.0,
                    record.request.scope.domain_id.0
                ],
                |row| row.get(0),
            )
            .optional()
            .map_err(err)?;
        require(
            cancelled == Some(1),
            "KnownUnused requires a durable mission cancellation fence",
        )?;
        let attempt_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM routing_attempts WHERE attempt_id=?1)",
                [&record.request.reservation.attempt_id],
                |row| row.get(0),
            )
            .map_err(err)?;
        require(
            !attempt_exists,
            "KnownUnused cannot cover an admitted attempt",
        )?;
        let reservation = observed_reservation
            .ok_or_else(|| invalid("release proof requires a started reserve intent"))?;
        require(
            reservation.state == CapacityReservationState::Provisional
                && reservation.launch_token.is_none()
                && reservation.bound_at_ms.is_none()
                && reservation.recovery_evidence_id.is_none()
                && reservation.release_evidence.is_none(),
            "KnownUnused requires a provisional unbound Catalog reservation",
        )?;
        record.phase = CapacityIntentPhase::ReleaseProofBound;
        record.release_proof_event_id = Some(event_id.into());
        record.expected_release = Some(proof.clone());
        save_intent(&tx, &mut record, CapacityIntentPhase::ReserveMayHaveStarted)?;
        tx.commit().map_err(err)?;
        persisted(self, reservation_id)
    }

    /// Bind a distinct admitted-attempt KnownUnused proof. A native create
    /// must never have been authorized, and the exact retained no-launch
    /// receipt must already have closed the one-use launch owner.
    pub fn bind_admitted_no_launch_release_proof(
        &self,
        catalog: &Catalog,
        reservation_id: &str,
        event_id: &str,
        proof: &CapacityReleaseEvidence,
        admitted: &AdmittedNoLaunchRelease,
    ) -> Result<RoutingCapacityIntent> {
        require_id(event_id, "admitted release proof event")?;
        validate_release_request(&CapacityReleaseRequest {
            reservation_id: reservation_id.into(),
            evidence: proof.clone(),
        })?;
        let observed = required_intent(&self.conn, reservation_id)?;
        let reservation = exact_catalog_reservation(catalog, &observed)?;
        let tx = immediate(&self.conn)?;
        let mut record = required_intent(&tx, reservation_id)?;
        require(
            record == observed,
            "admitted release intent changed during Catalog readback",
        )?;
        require_admitted_no_launch_proof(&tx, &record, admitted, proof)?;
        if record.release_proof_event_id.as_deref() == Some(event_id)
            && record.expected_release.as_ref() == Some(proof)
            && record.admitted_no_launch_release.as_ref() == Some(admitted)
        {
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CapacityIntentPhase::ReserveMayHaveStarted,
            "admitted release proof requires a started reserve intent",
        )?;
        require(
            matches!(
                reservation.state,
                CapacityReservationState::Bound | CapacityReservationState::RecoveryRequired
            ) && reservation.launch_token.as_deref() == Some(admitted.launch_token.as_str())
                && reservation.bound_at_ms.is_some()
                && reservation.release_evidence.is_none(),
            "admitted release requires the exact held Bound Catalog reservation",
        )?;
        record.phase = CapacityIntentPhase::ReleaseProofBound;
        record.release_proof_event_id = Some(event_id.into());
        record.expected_release = Some(proof.clone());
        record.admitted_no_launch_release = Some(admitted.clone());
        save_intent(&tx, &mut record, CapacityIntentPhase::ReserveMayHaveStarted)?;
        tx.commit().map_err(err)?;
        persisted(self, reservation_id)
    }

    /// Bind a terminal admitted attempt to its exact retained worker and
    /// checker observations. This validates controller receipts, not the OS.
    /// The caller must hold the shared run gate while fencing future creates.
    pub fn bind_admitted_quiescent_release_proof(
        &self,
        catalog: &Catalog,
        reservation_id: &str,
        event_id: &str,
        proof: &CapacityReleaseEvidence,
        admitted: &AdmittedQuiescentRelease,
    ) -> Result<RoutingCapacityIntent> {
        require_id(event_id, "admitted quiescent proof event")?;
        validate_release_request(&CapacityReleaseRequest {
            reservation_id: reservation_id.into(),
            evidence: proof.clone(),
        })?;
        let observed = required_intent(&self.conn, reservation_id)?;
        let reservation = exact_catalog_reservation(catalog, &observed)?;
        let tx = immediate(&self.conn)?;
        let mut record = required_intent(&tx, reservation_id)?;
        require(
            record == observed,
            "quiescent release intent changed during Catalog readback",
        )?;
        require_admitted_quiescent_proof(&tx, &record, admitted, proof)?;
        if record.release_proof_event_id.as_deref() == Some(event_id)
            && record.expected_release.as_ref() == Some(proof)
            && record.admitted_quiescent_release.as_ref() == Some(admitted)
        {
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CapacityIntentPhase::ReserveMayHaveStarted,
            "quiescent release proof requires a started reserve intent",
        )?;
        require(
            matches!(
                reservation.state,
                CapacityReservationState::Bound | CapacityReservationState::RecoveryRequired
            ) && reservation.launch_token.as_deref() == Some(admitted.launch_token.as_str())
                && reservation.bound_at_ms.is_some()
                && reservation.release_evidence.is_none(),
            "quiescent release requires the exact held Bound Catalog reservation",
        )?;
        record.phase = CapacityIntentPhase::ReleaseProofBound;
        record.release_proof_event_id = Some(event_id.into());
        record.expected_release = Some(proof.clone());
        record.admitted_quiescent_release = Some(admitted.clone());
        save_intent(&tx, &mut record, CapacityIntentPhase::ReserveMayHaveStarted)?;
        tx.commit().map_err(err)?;
        persisted(self, reservation_id)
    }

    /// Issue or replay the exact host release after its durable Store proof.
    /// Catalog occupancy changes only through its bound owner/token CAS.
    pub fn release_admitted_no_launch_capacity(
        &self,
        catalog: &Catalog,
        reservation_id: &str,
    ) -> Result<CapacityReservationRecord> {
        let record = required_intent(&self.conn, reservation_id)?;
        require(
            record.phase == CapacityIntentPhase::ReleaseProofBound,
            "admitted host release requires a persisted proof",
        )?;
        let admitted = record
            .admitted_no_launch_release
            .as_ref()
            .ok_or_else(|| invalid("admitted host release context absent"))?;
        let proof = record
            .expected_release
            .as_ref()
            .ok_or_else(|| invalid("admitted host release evidence absent"))?;
        let observed = exact_catalog_reservation(catalog, &record)?;
        require(
            observed.launch_token.as_deref() == Some(admitted.launch_token.as_str())
                && observed.bound_at_ms.is_some()
                && matches!(
                    observed.state,
                    CapacityReservationState::Bound
                        | CapacityReservationState::RecoveryRequired
                        | CapacityReservationState::Released
                ),
            "admitted host release Catalog launch binding changed",
        )?;
        let reservation = &record.request.reservation;
        let released =
            catalog.release_bound_capacity_reservation(&CapacityBoundReleaseRequest {
                reservation_id: reservation.reservation_id.clone(),
                domain_id: reservation.domain_id.clone(),
                run_id: reservation.run_id.clone(),
                attempt_id: reservation.attempt_id.clone(),
                owner: reservation.owner.clone(),
                launch_token: admitted.launch_token.clone(),
                evidence: proof.clone(),
            })?;
        require(
            released.state == CapacityReservationState::Released
                && released.release_evidence.as_ref() == Some(proof),
            "admitted host release readback changed",
        )?;
        Ok(released)
    }

    /// Release exactly the bound host hold proven by the admitted quiescent
    /// ledger. Retry uses the same immutable Catalog owner, token and proof.
    pub fn release_admitted_quiescent_capacity(
        &self,
        catalog: &Catalog,
        reservation_id: &str,
    ) -> Result<CapacityReservationRecord> {
        let record = required_intent(&self.conn, reservation_id)?;
        require(
            record.phase == CapacityIntentPhase::ReleaseProofBound,
            "quiescent host release requires a persisted proof",
        )?;
        let admitted = record
            .admitted_quiescent_release
            .as_ref()
            .ok_or_else(|| invalid("quiescent release context absent"))?;
        let proof = record
            .expected_release
            .as_ref()
            .ok_or_else(|| invalid("quiescent release evidence absent"))?;
        let observed = exact_catalog_reservation(catalog, &record)?;
        require(
            observed.launch_token.as_deref() == Some(admitted.launch_token.as_str())
                && observed.bound_at_ms.is_some()
                && matches!(
                    observed.state,
                    CapacityReservationState::Bound
                        | CapacityReservationState::RecoveryRequired
                        | CapacityReservationState::Released
                ),
            "quiescent host release Catalog launch binding changed",
        )?;
        let reservation = &record.request.reservation;
        let released =
            catalog.release_bound_capacity_reservation(&CapacityBoundReleaseRequest {
                reservation_id: reservation.reservation_id.clone(),
                domain_id: reservation.domain_id.clone(),
                run_id: reservation.run_id.clone(),
                attempt_id: reservation.attempt_id.clone(),
                owner: reservation.owner.clone(),
                launch_token: admitted.launch_token.clone(),
                evidence: proof.clone(),
            })?;
        require(
            released.state == CapacityReservationState::Released
                && released.release_evidence.as_ref() == Some(proof),
            "quiescent host release readback changed",
        )?;
        Ok(released)
    }

    /// Close only after an exact matching immutable Released Catalog readback.
    /// A missing/replaced Catalog or absent row after may-started remains held.
    pub fn close_released_capacity_intent(
        &self,
        catalog: &Catalog,
        reservation_id: &str,
        event_id: &str,
    ) -> Result<RoutingCapacityIntent> {
        require_id(event_id, "released closure event")?;
        let observed = required_intent(&self.conn, reservation_id)?;
        let observed_reservation = if observed.phase == CapacityIntentPhase::ReleaseProofBound {
            Some(exact_catalog_reservation(catalog, &observed)?)
        } else {
            None
        };
        let tx = immediate(&self.conn)?;
        let mut record = required_intent(&tx, reservation_id)?;
        require(
            record == observed,
            "capacity intent changed during Catalog readback",
        )?;
        if record.phase == CapacityIntentPhase::Closed
            && record.reserve_event_id.is_some()
            && record.close_event_id.as_deref() == Some(event_id)
        {
            tx.commit().map_err(err)?;
            return Ok(record);
        }
        require(
            record.phase == CapacityIntentPhase::ReleaseProofBound,
            "released closure requires persisted release proof",
        )?;
        let reservation = observed_reservation
            .ok_or_else(|| invalid("released closure requires persisted release proof"))?;
        let bound_token = record
            .admitted_no_launch_release
            .as_ref()
            .map(|admitted| admitted.launch_token.as_str())
            .or_else(|| {
                record
                    .admitted_quiescent_release
                    .as_ref()
                    .map(|admitted| admitted.launch_token.as_str())
            });
        let release_matches = if let Some(token) = bound_token {
            reservation.launch_token.as_deref() == Some(token) && reservation.bound_at_ms.is_some()
        } else {
            reservation.launch_token.is_none()
                && reservation.bound_at_ms.is_none()
                && reservation.recovery_evidence_id.is_none()
        };
        require(
            reservation.state == CapacityReservationState::Released
                && release_matches
                && reservation.release_evidence == record.expected_release,
            "Catalog release does not match the capacity intent proof",
        )?;
        record.phase = CapacityIntentPhase::Closed;
        record.close_event_id = Some(event_id.into());
        save_intent(&tx, &mut record, CapacityIntentPhase::ReleaseProofBound)?;
        tx.commit().map_err(err)?;
        persisted(self, reservation_id)
    }

    pub fn capacity_intent(&self, reservation_id: &str) -> Result<Option<RoutingCapacityIntent>> {
        require_id(reservation_id, "reservation id")?;
        load_intent(&self.conn, reservation_id)
    }

    /// Validate every private projection, including closed rows, then return
    /// only run scopes whose Catalog ownership is not durably closed.
    pub fn unresolved_capacity_intent_scopes(&self) -> Result<Vec<RoutingScope>> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        let mut stmt = tx
            .prepare("SELECT reservation_id FROM routing_capacity_intents ORDER BY reservation_id")
            .map_err(err)?;
        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        let mut scopes = BTreeSet::new();
        for id in ids {
            let record = required_intent(&tx, &id)?;
            if record.phase != CapacityIntentPhase::Closed {
                scopes.insert((
                    record.request.scope.domain_id.0,
                    record.request.scope.run_id.0,
                ));
            }
        }
        Ok(scopes
            .into_iter()
            .map(|(domain_id, run_id)| RoutingScope {
                domain_id: DomainId(domain_id),
                run_id: RunId(run_id),
            })
            .collect())
    }
}

fn validate_intent_request(request: &RoutingCapacityIntentRequest) -> Result<()> {
    require_id(&request.event_id, "intent event")?;
    require_id(&request.task_id.0, "task id")?;
    validate_reservation_request(&request.reservation)?;
    require(
        request.scope.domain_id.0 == request.reservation.domain_id
            && request.scope.run_id.0 == request.reservation.run_id,
        "capacity intent reservation scope mismatch",
    )?;
    require(
        request.reservation.resources == canonical_resources(&request.reservation.resources)?,
        "capacity intent resources must be canonical",
    )
}

pub(crate) fn require_live_registered_task(
    conn: &Connection,
    request: &RoutingCapacityIntentRequest,
) -> Result<()> {
    let mission_state: Option<(String, i64, String)> = conn
        .query_row(
            "SELECT m.domain_id,m.cancelled,r.status FROM routing_missions m
             JOIN runs r ON r.id=m.run_id WHERE m.run_id=?1",
            [&request.scope.run_id.0],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(err)?;
    require(
        mission_state.is_some_and(|(domain, cancelled, status)| {
            domain == request.scope.domain_id.0
                && cancelled == 0
                && matches!(status.as_str(), "starting" | "running")
        }),
        "capacity intent requires an active uncancelled registered mission",
    )?;
    let task_projection: Option<(u64, String)> = conn
        .query_row(
            "SELECT revision,record_json FROM routing_tasks WHERE run_id=?1 AND task_id=?2",
            params![request.scope.run_id.0, request.task_id.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    let (task_revision, task_json) =
        task_projection.ok_or_else(|| invalid("capacity intent task is not registered"))?;
    let task: RoutedTaskRecord = parse(&task_json)?;
    let prior_repair_released = if let Some(previous_id) = &task.current_attempt {
        let previous_json: Option<String> = conn
            .query_row(
                "SELECT record_json FROM routing_attempts WHERE run_id=?1 AND attempt_id=?2",
                params![request.scope.run_id.0, previous_id.0],
                |row| row.get(0),
            )
            .optional()
            .map_err(err)?;
        if let Some(previous_json) = previous_json {
            let previous: RoutedAttemptRecord = parse(&previous_json)?;
            previous.scope == request.scope
                && previous.task_id == request.task_id
                && previous.attempt_id == *previous_id
                && previous.ordinal == 1
                && previous.state == AttemptState::Failed
                && previous.ownership_released
                && previous.failure.as_ref().is_some_and(|failure| {
                    matches!(
                        failure.failure_class,
                        pytxo_core::routing::AttemptFailureClass::Implementation
                            | pytxo_core::routing::AttemptFailureClass::Check
                    ) && failure.actionable_evidence_digest.is_some()
                })
                && task.next_ordinal == 2
                && json(&previous)? == previous_json
        } else {
            false
        }
    } else {
        true
    };
    require(
        task.registration.contract.task_id == request.task_id
            && task.revision == task_revision
            && task.state == TaskRoutingState::Ready
            && prior_repair_released
            && json(&task)? == task_json,
        "capacity intent requires an exact Ready registered task",
    )?;
    let attempt_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM routing_attempts WHERE attempt_id=?1)",
            [&request.reservation.attempt_id],
            |row| row.get(0),
        )
        .map_err(err)?;
    require(
        !attempt_exists,
        "capacity intent attempt is already admitted",
    )
}

fn require_admitted_no_launch_proof(
    conn: &Connection,
    record: &RoutingCapacityIntent,
    admitted: &AdmittedNoLaunchRelease,
    proof: &CapacityReleaseEvidence,
) -> Result<()> {
    let claim = &admitted.receipt_claim;
    require(
        claim.kind == PrivateArtifactKind::ControllerReceipt
            && claim.scope == record.request.scope
            && claim.task_id == record.request.task_id
            && claim.attempt_id.0 == record.request.reservation.attempt_id
            && claim.reservation_id == record.request.reservation.reservation_id,
        "admitted release receipt owner mismatch",
    )?;
    let (retained_claim, receipt) =
        exact_retained_receipt_by_id(conn, &claim.artifact_id, &admitted.receipt_ref)?;
    require(
        retained_claim == *claim
            && receipt.source == ReceiptSource::TrustedController
            && proof.kind == CapacityReleaseEvidenceKind::KnownUnused
            && proof.receipt_id == claim.event_id
            && proof.evidence_digest == admitted.receipt_ref.digest.0
            && proof.observed_at_ms == receipt.observed_at_ms,
        "admitted release evidence does not match retained no-launch receipt",
    )?;
    let ControllerObservation::AdmittedNoLaunch {
        cancellation_event_id,
    } = &receipt.observation
    else {
        return Err(invalid("admitted release receipt is not no-launch"));
    };
    require_cancelled_event(conn, claim, cancellation_event_id)?;
    let attempt = exact_admitted_attempt(conn, claim)?
        .ok_or_else(|| invalid("admitted release attempt absent"))?;
    require(
        attempt.state == AttemptState::FailedNoLaunch
            && attempt.ownership_released
            && attempt.receipts.no_worker_created.as_ref() == Some(&admitted.receipt_ref.digest)
            && attempt.receipts.process_identity.is_none(),
        "admitted release attempt is not terminal no-launch",
    )?;
    require(
        closed_no_launch_token_for_release(conn, claim, &admitted.receipt_ref)?
            == admitted.launch_token,
        "admitted release launch token changed",
    )
}

fn require_admitted_quiescent_proof(
    conn: &Connection,
    record: &RoutingCapacityIntent,
    admitted: &AdmittedQuiescentRelease,
    proof: &CapacityReleaseEvidence,
) -> Result<()> {
    let claim = &admitted.worker_receipt_claim;
    require(
        claim.kind == PrivateArtifactKind::ControllerReceipt
            && claim.scope == record.request.scope
            && claim.task_id == record.request.task_id
            && claim.attempt_id.0 == record.request.reservation.attempt_id
            && claim.reservation_id == record.request.reservation.reservation_id,
        "quiescent worker receipt owner mismatch",
    )?;
    let attempt = exact_admitted_attempt(conn, claim)?
        .ok_or_else(|| invalid("quiescent admitted attempt absent"))?;
    require(
        attempt.state.is_terminal()
            && attempt.state != AttemptState::FailedNoLaunch
            && attempt.ownership_released
            && attempt.owned_launch_required
            && attempt.receipts.process_identity.is_some()
            && attempt.receipts.no_worker_created.is_none()
            && attempt.receipts.quiescence.as_ref() == Some(&admitted.worker_receipt_ref.digest),
        "quiescent release requires a terminal process attempt",
    )?;
    let worker = load_ownership(conn, &attempt.attempt_id)?
        .ok_or_else(|| invalid("quiescent worker owner absent"))?;
    require(
        worker.phase == LaunchOwnershipPhase::Settled
            && worker.request.scope == record.request.scope
            && worker.request.task_id == record.request.task_id
            && worker.request.reservation_id == record.request.reservation.reservation_id
            && worker.request.launch_token == admitted.launch_token
            && worker.settlement_artifact_id.as_deref() == Some(claim.artifact_id.as_str())
            && worker.settlement_blob.as_ref() == Some(&admitted.worker_receipt_ref),
        "quiescent worker owner or token changed",
    )?;
    let (retained_claim, worker_receipt) =
        exact_retained_receipt_by_id(conn, &claim.artifact_id, &admitted.worker_receipt_ref)?;
    require(
        retained_claim == *claim
            && worker_receipt.source == ReceiptSource::OwnedJobObservation
            && matches!(
                worker_receipt.observation,
                ControllerObservation::NativeJobZero {
                    active_processes: 0,
                    ..
                }
            ),
        "quiescent worker receipt is not exact Job-zero",
    )?;
    require_checkers_settled_for_release(conn, &attempt)?;
    require(
        admitted.checker_receipts.len() == attempt.owned_checker_count as usize,
        "quiescent checker count differs from attempt marker",
    )?;
    let mut observed_at_ms = worker_receipt.observed_at_ms;
    for (index, checker) in admitted.checker_receipts.iter().enumerate() {
        let ordinal = (index + 1) as u32;
        require(
            checker.ordinal == ordinal
                && checker.receipt_claim.kind == PrivateArtifactKind::ControllerReceipt
                && checker.receipt_claim.scope == record.request.scope
                && checker.receipt_claim.task_id == record.request.task_id
                && checker.receipt_claim.attempt_id == attempt.attempt_id
                && checker.receipt_claim.reservation_id
                    == record.request.reservation.reservation_id,
            "quiescent checker receipt owner or order changed",
        )?;
        let owned = load_checker(conn, &attempt.attempt_id, ordinal)?
            .ok_or_else(|| invalid("quiescent checker owner absent"))?;
        require(
            matches!(
                owned.phase,
                CheckerOwnershipPhase::Settled | CheckerOwnershipPhase::ClosedNoCreate
            ) && owned.request.scope == record.request.scope
                && owned.request.task_id == record.request.task_id
                && owned.request.reservation_id == record.request.reservation.reservation_id
                && owned.request.launch_token == admitted.launch_token
                && owned.settlement_artifact_id.as_deref()
                    == Some(checker.receipt_claim.artifact_id.as_str())
                && owned.settlement_blob.as_ref() == Some(&checker.receipt_ref),
            "quiescent checker ownership or settlement changed",
        )?;
        let (retained_claim, receipt) = exact_retained_receipt_by_id(
            conn,
            &checker.receipt_claim.artifact_id,
            &checker.receipt_ref,
        )?;
        let valid_observation = match (&owned.phase, receipt.source, receipt.observation) {
            (
                CheckerOwnershipPhase::Settled,
                ReceiptSource::OwnedJobObservation,
                ControllerObservation::NativeCheckerResult {
                    check_id,
                    ordinal: observed_ordinal,
                    active_processes: 0,
                    ..
                },
            ) => check_id == owned.request.check_id && observed_ordinal == ordinal,
            (
                CheckerOwnershipPhase::ClosedNoCreate,
                ReceiptSource::TrustedController,
                ControllerObservation::CheckerNotCreated {
                    check_id,
                    ordinal: observed_ordinal,
                    ..
                },
            ) => check_id == owned.request.check_id && observed_ordinal == ordinal,
            _ => false,
        };
        require(
            retained_claim == checker.receipt_claim && valid_observation,
            "quiescent checker receipt changed",
        )?;
        observed_at_ms = observed_at_ms.max(receipt.observed_at_ms);
    }
    require(
        proof.kind == CapacityReleaseEvidenceKind::QuiescenceReconciled
            && proof.receipt_id == claim.event_id
            && proof.evidence_digest == canonical_digest(admitted, 1).map_err(err)?.0
            && proof.observed_at_ms == observed_at_ms,
        "quiescent release proof does not bind every retained owner",
    )
}

fn validate_record(record: &RoutingCapacityIntent) -> Result<()> {
    validate_intent_request(&record.request)?;
    require(
        record.catalog_locator.is_absolute() && record.revision > 0,
        "capacity intent locator or revision is invalid",
    )?;
    if let Some(event) = &record.reserve_event_id {
        require_id(event, "reserve event")?;
    }
    if let Some(event) = &record.release_proof_event_id {
        require_id(event, "release proof event")?;
    }
    if let Some(event) = &record.close_event_id {
        require_id(event, "close event")?;
    }
    if let Some(proof) = &record.expected_release {
        validate_release_request(&CapacityReleaseRequest {
            reservation_id: record.request.reservation.reservation_id.clone(),
            evidence: proof.clone(),
        })?;
        require(
            match (
                &record.admitted_no_launch_release,
                &record.admitted_quiescent_release,
            ) {
                (None, None) | (Some(_), None) => {
                    proof.kind == CapacityReleaseEvidenceKind::KnownUnused
                }
                (None, Some(_)) => proof.kind == CapacityReleaseEvidenceKind::QuiescenceReconciled,
                (Some(_), Some(_)) => false,
            },
            "capacity intent release context or proof kind changed",
        )?;
    }
    if let Some(admitted) = &record.admitted_no_launch_release {
        require(
            matches!(
                record.phase,
                CapacityIntentPhase::ReleaseProofBound | CapacityIntentPhase::Closed
            ) && record.expected_release.is_some()
                && !admitted.launch_token.trim().is_empty(),
            "admitted release context is outside proof or closure",
        )?;
    }
    if let Some(admitted) = &record.admitted_quiescent_release {
        require(
            matches!(
                record.phase,
                CapacityIntentPhase::ReleaseProofBound | CapacityIntentPhase::Closed
            ) && record.expected_release.is_some()
                && !admitted.launch_token.trim().is_empty()
                && record.admitted_no_launch_release.is_none(),
            "quiescent release context is outside proof or closure",
        )?;
    }
    let valid = match record.phase {
        CapacityIntentPhase::Prepared => {
            record.reserve_event_id.is_none()
                && record.release_proof_event_id.is_none()
                && record.expected_release.is_none()
                && record.close_event_id.is_none()
        }
        CapacityIntentPhase::ReserveMayHaveStarted => {
            record.reserve_event_id.is_some()
                && record.release_proof_event_id.is_none()
                && record.expected_release.is_none()
                && record.close_event_id.is_none()
        }
        CapacityIntentPhase::ReleaseProofBound => {
            record.reserve_event_id.is_some()
                && record.release_proof_event_id.is_some()
                && record.expected_release.is_some()
                && record.close_event_id.is_none()
        }
        CapacityIntentPhase::Closed => {
            record.close_event_id.is_some()
                && ((record.reserve_event_id.is_none()
                    && record.release_proof_event_id.is_none()
                    && record.expected_release.is_none())
                    || (record.reserve_event_id.is_some()
                        && record.release_proof_event_id.is_some()
                        && record.expected_release.is_some()))
        }
    };
    require(valid, "capacity intent phase projection is inconsistent")
}

pub(crate) fn exact_catalog_reservation(
    catalog: &Catalog,
    record: &RoutingCapacityIntent,
) -> Result<CapacityReservationRecord> {
    require(
        catalog.opened_path() == record.catalog_locator,
        "capacity intent Catalog locator mismatch",
    )?;
    let request = &record.request.reservation;
    let actual = catalog
        .capacity_reservation(&request.reservation_id)?
        .ok_or_else(|| invalid("capacity reservation absent after reserve may have started"))?;
    let actual_resources: Vec<_> = actual
        .resources
        .iter()
        .map(|resource| CapacityResourceRequest {
            resource_id: resource.resource_id.clone(),
            units: resource.units,
        })
        .collect();
    require(
        actual.reservation_id == request.reservation_id
            && actual.domain_id == request.domain_id
            && actual.run_id == request.run_id
            && actual.attempt_id == request.attempt_id
            && actual.owner == request.owner
            && actual.requested_at_ms == request.requested_at_ms
            && actual_resources == request.resources,
        "capacity reservation does not match durable intent owner and resources",
    )?;
    Ok(actual)
}

pub(crate) fn load_intent(
    conn: &Connection,
    reservation_id: &str,
) -> Result<Option<RoutingCapacityIntent>> {
    struct Projection {
        domain: String,
        run: String,
        task: String,
        attempt: String,
        creation_event: String,
        phase: String,
        revision: u64,
        body: String,
    }
    let row: Option<Projection> = conn
        .query_row(
            "SELECT domain_id,run_id,task_id,attempt_id,creation_event_id,phase,revision,record_json
             FROM routing_capacity_intents WHERE reservation_id=?1",
            [reservation_id],
            |row| Ok(Projection {
                domain: row.get(0)?,
                run: row.get(1)?,
                task: row.get(2)?,
                attempt: row.get(3)?,
                creation_event: row.get(4)?,
                phase: row.get(5)?,
                revision: row.get(6)?,
                body: row.get(7)?,
            }),
        )
        .optional()
        .map_err(err)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let record: RoutingCapacityIntent = parse(&row.body)?;
    validate_record(&record)?;
    require(
        record.request.reservation.reservation_id == reservation_id
            && record.request.scope.domain_id.0 == row.domain
            && record.request.scope.run_id.0 == row.run
            && record.request.task_id.0 == row.task
            && record.request.reservation.attempt_id == row.attempt
            && record.request.event_id == row.creation_event
            && record.phase.as_str() == row.phase
            && record.revision == row.revision
            && json(&record)? == row.body,
        "capacity intent SQL/JSON projection mismatch",
    )?;
    if let (Some(admitted), Some(proof)) =
        (&record.admitted_no_launch_release, &record.expected_release)
    {
        require_admitted_no_launch_proof(conn, &record, admitted, proof)?;
    }
    if let (Some(admitted), Some(proof)) =
        (&record.admitted_quiescent_release, &record.expected_release)
    {
        require_admitted_quiescent_proof(conn, &record, admitted, proof)?;
    }
    Ok(Some(record))
}

fn required_intent(conn: &Connection, reservation_id: &str) -> Result<RoutingCapacityIntent> {
    load_intent(conn, reservation_id)?
        .ok_or_else(|| invalid("capacity intent reservation not found"))
}

fn save_intent(
    tx: &Transaction<'_>,
    record: &mut RoutingCapacityIntent,
    expected_phase: CapacityIntentPhase,
) -> Result<()> {
    let previous_revision = record.revision;
    record.revision = previous_revision
        .checked_add(1)
        .filter(|revision| *revision <= i64::MAX as u64)
        .ok_or_else(|| invalid("capacity intent revision exhausted"))?;
    validate_record(record)?;
    let updated = tx
        .execute(
            "UPDATE routing_capacity_intents SET phase=?1,revision=?2,record_json=?3
             WHERE reservation_id=?4 AND phase=?5 AND revision=?6",
            params![
                record.phase.as_str(),
                record.revision,
                json(record)?,
                record.request.reservation.reservation_id,
                expected_phase.as_str(),
                previous_revision,
            ],
        )
        .map_err(err)?;
    require(updated == 1, "capacity intent phase CAS conflict")
}

fn persisted(store: &PytxoStore, reservation_id: &str) -> Result<RoutingCapacityIntent> {
    store
        .capacity_intent(reservation_id)?
        .ok_or_else(|| invalid("persisted capacity intent missing"))
}

fn immediate(conn: &Connection) -> Result<Transaction<'_>> {
    Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(err)
}

fn json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(err)
}

fn parse<T: serde::de::DeserializeOwned>(body: &str) -> Result<T> {
    serde_json::from_str(body).map_err(err)
}

fn require_id(value: &str, label: &str) -> Result<()> {
    require(
        !value.trim().is_empty(),
        &format!("{label} must not be empty"),
    )
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}

fn err(error: impl std::fmt::Display) -> PytxoError {
    invalid(&error.to_string())
}

fn invalid(message: &str) -> PytxoError {
    PytxoError::Store(message.into())
}
