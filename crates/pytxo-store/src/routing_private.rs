//! Retained private routed bytes. These records are not worker-supplied facts,
//! process evidence, permission grants, or a public artifact service.

use pytxo_core::routing::{AttemptId, AttemptState, BlobRef, CheckId, Digest};
use pytxo_core::{PytxoError, Result as CoreResult, TaskId};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::capacity::CapacityReservationState;
use crate::routing::{RoutedAttemptRecord, RoutingControlEvent, RoutingScope};
use crate::routing_capacity_intent::{
    exact_catalog_reservation, load_intent, require_live_registered_task, CapacityIntentPhase,
};
use crate::routing_launch::{load_ownership, LaunchOwnershipPhase};
use crate::{Catalog, PytxoStore};

const MAX_BLOB_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RUN_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RUN_ARTIFACTS: u64 = 4_096;
const MAX_ID_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateArtifactKind {
    InputManifest,
    HandoffManifest,
    ScopedOutput,
    ControllerReceipt,
    NoWorkerReceipt,
}

impl PrivateArtifactKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::InputManifest => "input_manifest",
            Self::HandoffManifest => "handoff_manifest",
            Self::ScopedOutput => "scoped_output",
            Self::ControllerReceipt => "controller_receipt",
            Self::NoWorkerReceipt => "no_worker_receipt",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivateArtifactClaim {
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub reservation_id: String,
    pub kind: PrivateArtifactKind,
    pub artifact_id: String,
    pub event_id: String,
}

#[derive(Debug)]
pub enum RoutingArtifactError {
    TooLarge {
        requested_bytes: u64,
    },
    RunQuotaExceeded {
        held_bytes: u64,
        requested_bytes: u64,
    },
    RunArtifactCountExceeded {
        held_count: u64,
    },
    Store(PytxoError),
}

impl From<PytxoError> for RoutingArtifactError {
    fn from(error: PytxoError) -> Self {
        Self::Store(error)
    }
}
impl std::fmt::Display for RoutingArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge { requested_bytes } => write!(f, "private routed artifact exceeds 16 MiB limit: {requested_bytes} bytes"),
            Self::RunQuotaExceeded { held_bytes, requested_bytes } => write!(f, "private routed artifacts exceed 64 MiB run limit: held {held_bytes}, requested {requested_bytes}"),
            Self::RunArtifactCountExceeded { held_count } => write!(f, "private routed artifacts exceed 4096 rows per run: held {held_count}"),
            Self::Store(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for RoutingArtifactError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptSource {
    TrustedController,
    OwnedJobObservation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckerNativeOutcome {
    Succeeded,
    Failed,
    Cancelled,
    RecoveryRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ControllerObservation {
    NoWorker {
        cancellation_event_id: String,
    },
    AdmittedNoLaunch {
        cancellation_event_id: String,
    },
    /// Trusted controller's pre-create authority snapshot. Native process
    /// identity and Job-zero remain separate owned observations.
    LaunchChecks {
        mission_digest: Digest,
        qualification_digest: Digest,
        launch_fingerprint: Digest,
        launch_token: String,
    },
    NativeJobZero {
        job_name: String,
        launch_nonce: String,
        active_processes: u32,
    },
    NativeCheckerResult {
        check_id: CheckId,
        ordinal: u32,
        job_name: String,
        launch_nonce: String,
        active_processes: u32,
        exit_code: i32,
        payload_exit_code: Option<u32>,
        process_registered: bool,
        barrier_released: bool,
        native_outcome: CheckerNativeOutcome,
        stdout_complete: bool,
        stderr_complete: bool,
        output_truncated: bool,
        error_present: bool,
        sealed_view_before: BlobRef,
        /// None when the final view cannot be safely read or verified. A
        /// failed view must never be represented by the expected BlobRef.
        sealed_view_after: Option<BlobRef>,
    },
    CheckerNotCreated {
        check_id: CheckId,
        ordinal: u32,
        cancellation_event_id: String,
    },
}

/// Canonical bytes are `serde_json::to_vec` of this versioned, ordered struct.
/// It has no diagnostic excerpt, raw output, credential, or floating point field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControllerReceiptEnvelope {
    pub schema_version: u32,
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub reservation_id: String,
    pub observed_at_ms: u64,
    pub evidence_id: String,
    pub source: ReceiptSource,
    pub observation: ControllerObservation,
}

impl PytxoStore {
    /// Persist exact bytes in the domain transaction. Only the trusted
    /// controller may call this; receipt kinds require the typed methods below.
    pub fn put_private_artifact(
        &self,
        claim: &PrivateArtifactClaim,
        bytes: &[u8],
    ) -> std::result::Result<BlobRef, RoutingArtifactError> {
        validate_claim(claim)?;
        let tx = immediate(&self.conn)?;
        if let Some(existing) = load_artifact(&tx, &claim.artifact_id)? {
            require(
                existing.claim == *claim && existing.bytes == bytes,
                "artifact replay changed",
            )?;
            tx.commit().map_err(err)?;
            return Ok(existing.reference);
        }
        match claim.kind {
            PrivateArtifactKind::InputManifest | PrivateArtifactKind::HandoffManifest => {
                require_pre_admission(&tx, claim)?;
            }
            PrivateArtifactKind::ScopedOutput => {
                require_admitted_attempt(&tx, claim, true)?;
            }
            PrivateArtifactKind::ControllerReceipt | PrivateArtifactKind::NoWorkerReceipt => {
                return Err(invalid("controller receipts require typed retention"));
            }
        }
        let reference = insert_artifact(&tx, claim, bytes)?;
        tx.commit().map_err(err)?;
        let persisted = self.read_private_artifact(claim, &reference)?;
        require(persisted == bytes, "persisted artifact readback changed")?;
        Ok(reference)
    }

    /// A pre-admission no-worker receipt is allowed only after durable mission
    /// cancellation, for an exact unresolved intent and provisional unbound
    /// Catalog row. Catalog read precedes the domain write transaction.
    pub fn put_no_worker_receipt(
        &self,
        catalog: &Catalog,
        claim: &PrivateArtifactClaim,
        receipt: &ControllerReceiptEnvelope,
    ) -> std::result::Result<BlobRef, RoutingArtifactError> {
        require(
            claim.kind == PrivateArtifactKind::NoWorkerReceipt,
            "wrong receipt artifact kind",
        )?;
        validate_receipt(claim, receipt)?;
        let observed_intent = load_intent(&self.conn, &claim.reservation_id)?
            .ok_or_else(|| invalid("no-worker receipt has no capacity intent"))?;
        let observed_catalog = exact_catalog_reservation(catalog, &observed_intent)?;
        require(
            observed_catalog.state == CapacityReservationState::Provisional
                && observed_catalog.launch_token.is_none()
                && observed_catalog.bound_at_ms.is_none()
                && observed_catalog.recovery_evidence_id.is_none()
                && observed_catalog.release_evidence.is_none(),
            "no-worker receipt requires a provisional unbound Catalog reservation",
        )?;
        let bytes = receipt_bytes(receipt)?;
        let tx = immediate(&self.conn)?;
        let intent = load_intent(&tx, &claim.reservation_id)?
            .ok_or_else(|| invalid("no-worker receipt capacity intent vanished"))?;
        require(
            intent == observed_intent,
            "no-worker receipt intent changed during Catalog read",
        )?;
        require_intent_owner(claim, &intent)?;
        require(
            intent.phase == CapacityIntentPhase::ReserveMayHaveStarted,
            "no-worker receipt requires a started reserve intent",
        )?;
        let cancelled: Option<bool> = tx
            .query_row(
                "SELECT cancelled FROM routing_missions WHERE run_id=?1 AND domain_id=?2",
                params![claim.scope.run_id.0, claim.scope.domain_id.0],
                |row| row.get(0),
            )
            .optional()
            .map_err(err)?;
        require(
            cancelled == Some(true),
            "no-worker receipt requires durable cancellation",
        )?;
        require_admission_absent(&tx, claim)?;
        let ControllerObservation::NoWorker {
            cancellation_event_id,
        } = &receipt.observation
        else {
            return Err(invalid("no-worker receipt observation mismatch"));
        };
        require_cancelled_event(&tx, claim, cancellation_event_id)?;
        let reference = insert_or_replay(&tx, claim, &bytes)?;
        tx.commit().map_err(err)?;
        self.read_controller_receipt(claim, &reference)?;
        Ok(reference)
    }

    pub fn put_controller_receipt(
        &self,
        claim: &PrivateArtifactClaim,
        receipt: &ControllerReceiptEnvelope,
    ) -> std::result::Result<BlobRef, RoutingArtifactError> {
        require(
            claim.kind == PrivateArtifactKind::ControllerReceipt,
            "wrong receipt artifact kind",
        )?;
        validate_receipt(claim, receipt)?;
        let bytes = receipt_bytes(receipt)?;
        let tx = immediate(&self.conn)?;
        let attempt = require_admitted_attempt(&tx, claim, true)?;
        match &receipt.observation {
            ControllerObservation::AdmittedNoLaunch {
                cancellation_event_id,
            }
            | ControllerObservation::CheckerNotCreated {
                cancellation_event_id,
                ..
            } => require_cancelled_event(&tx, claim, cancellation_event_id)?,
            ControllerObservation::LaunchChecks { .. } => {
                require(
                    attempt.state == AttemptState::Preparing,
                    "launch checks must precede the owned launch transition",
                )?;
                validate_launch_checks(&tx, &attempt, receipt)?;
            }
            _ => {}
        }
        let reference = insert_or_replay(&tx, claim, &bytes)?;
        tx.commit().map_err(err)?;
        self.read_controller_receipt(claim, &reference)?;
        Ok(reference)
    }

    /// Every read rechecks exact SQL/JSON projection, owner, digest, length and
    /// retained bytes. A digest alone is never an authorization key.
    pub fn read_private_artifact(
        &self,
        claim: &PrivateArtifactClaim,
        reference: &BlobRef,
    ) -> CoreResult<Vec<u8>> {
        validate_claim(claim)?;
        let artifact = load_artifact(&self.conn, &claim.artifact_id)?
            .ok_or_else(|| store_error("private artifact absent"))?;
        if artifact.claim != *claim || artifact.reference != *reference {
            return Err(store_error(
                "private artifact owner, kind, event or reference mismatch",
            ));
        }
        Ok(artifact.bytes)
    }

    /// Recover a retained artifact when the winner journal pins its digest but
    /// does not retain the artifact's byte length. The complete owner claim is
    /// still required; a digest alone is never a read capability.
    pub fn read_private_artifact_matching_digest(
        &self,
        claim: &PrivateArtifactClaim,
        expected_digest: &Digest,
    ) -> CoreResult<Vec<u8>> {
        validate_claim(claim)?;
        let artifact = load_artifact(&self.conn, &claim.artifact_id)?
            .ok_or_else(|| store_error("private artifact absent"))?;
        if artifact.claim != *claim || artifact.reference.digest != *expected_digest {
            return Err(store_error(
                "private artifact owner or winner digest mismatch",
            ));
        }
        self.read_private_artifact(claim, &artifact.reference)
    }

    pub fn read_controller_receipt(
        &self,
        claim: &PrivateArtifactClaim,
        reference: &BlobRef,
    ) -> CoreResult<ControllerReceiptEnvelope> {
        if !matches!(
            claim.kind,
            PrivateArtifactKind::ControllerReceipt | PrivateArtifactKind::NoWorkerReceipt
        ) {
            return Err(store_error("artifact is not a controller receipt"));
        }
        let bytes = self.read_private_artifact(claim, reference)?;
        let receipt: ControllerReceiptEnvelope = serde_json::from_slice(&bytes).map_err(err)?;
        if receipt_bytes(&receipt)? != bytes {
            return Err(store_error("controller receipt bytes are not canonical"));
        }
        validate_receipt(claim, &receipt)?;
        Ok(receipt)
    }
}

/// The handoff owner is derived from the admitted attempt, never supplied as
/// a free-form artifact identity by a worker or advisor.
pub fn handoff_manifest_claim(
    scope: &RoutingScope,
    task_id: &TaskId,
    attempt_id: &AttemptId,
    reservation_id: &str,
) -> PrivateArtifactClaim {
    PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: task_id.clone(),
        attempt_id: attempt_id.clone(),
        reservation_id: reservation_id.to_owned(),
        kind: PrivateArtifactKind::HandoffManifest,
        artifact_id: format!("{}:handoff", attempt_id.0),
        event_id: format!("{}:handoff-retained", attempt_id.0),
    }
}

pub(crate) fn retained_handoff_bytes(
    conn: &Connection,
    claim: &PrivateArtifactClaim,
    reference: &BlobRef,
) -> CoreResult<Vec<u8>> {
    if claim.kind != PrivateArtifactKind::HandoffManifest {
        return Err(store_error("expected a handoff manifest owner"));
    }
    let artifact = load_artifact(conn, &claim.artifact_id)?
        .ok_or_else(|| store_error("handoff manifest is not retained"))?;
    if artifact.claim != *claim || artifact.reference != *reference {
        return Err(store_error("handoff owner or reference changed"));
    }
    Ok(artifact.bytes)
}

struct StoredArtifact {
    claim: PrivateArtifactClaim,
    reference: BlobRef,
    bytes: Vec<u8>,
}

fn load_artifact(conn: &Connection, artifact_id: &str) -> CoreResult<Option<StoredArtifact>> {
    #[derive(Debug)]
    struct Projection {
        domain_id: String,
        run_id: String,
        task_id: String,
        attempt_id: String,
        reservation_id: String,
        kind: String,
        event_id: String,
        digest: String,
        byte_length: u64,
        claim_json: String,
        bytes: Vec<u8>,
    }
    let row: Option<Projection> = conn
        .query_row(
            "SELECT domain_id,run_id,task_id,attempt_id,reservation_id,kind,event_id,digest,byte_length,claim_json,bytes
             FROM routing_private_artifacts WHERE artifact_id=?1",
            [artifact_id],
            |row| Ok(Projection {
                domain_id: row.get(0)?,
                run_id: row.get(1)?,
                task_id: row.get(2)?,
                attempt_id: row.get(3)?,
                reservation_id: row.get(4)?,
                kind: row.get(5)?,
                event_id: row.get(6)?,
                digest: row.get(7)?,
                byte_length: row.get(8)?,
                claim_json: row.get(9)?,
                bytes: row.get(10)?,
            }),
        )
        .optional()
        .map_err(err)?;
    let Some(row) = row else { return Ok(None) };
    let claim: PrivateArtifactClaim = serde_json::from_str(&row.claim_json).map_err(err)?;
    validate_claim(&claim)?;
    let reference = BlobRef {
        digest: Digest(row.digest),
        byte_length: row.byte_length,
    };
    if claim.artifact_id != artifact_id
        || claim.scope.domain_id.0 != row.domain_id
        || claim.scope.run_id.0 != row.run_id
        || claim.task_id.0 != row.task_id
        || claim.attempt_id.0 != row.attempt_id
        || claim.reservation_id != row.reservation_id
        || claim.kind.as_str() != row.kind
        || claim.event_id != row.event_id
        || serde_json::to_string(&claim).map_err(err)? != row.claim_json
        || reference.verify(&row.bytes).is_err()
        || row.byte_length > MAX_BLOB_BYTES
    {
        return Err(store_error(
            "private artifact SQL projection or bytes changed",
        ));
    }
    Ok(Some(StoredArtifact {
        claim,
        reference,
        bytes: row.bytes,
    }))
}

/// Resolve a winner's retained output from its exact owned attempt. The
/// caller has already checked the approved dependency edge and winner ledger;
/// no caller-supplied artifact path or event ID is accepted here.
pub(crate) fn retained_winner_output(
    conn: &Connection,
    attempt: &RoutedAttemptRecord,
    expected_digest: &Digest,
) -> CoreResult<(BlobRef, Vec<u8>)> {
    let mut statement = conn
        .prepare(
            "SELECT artifact_id FROM routing_private_artifacts
             WHERE domain_id=?1 AND run_id=?2 AND task_id=?3 AND attempt_id=?4
               AND reservation_id=?5 AND kind='scoped_output' AND digest=?6",
        )
        .map_err(err)?;
    let ids = statement
        .query_map(
            params![
                attempt.scope.domain_id.0,
                attempt.scope.run_id.0,
                attempt.task_id.0,
                attempt.attempt_id.0,
                attempt.capacity_reservation,
                expected_digest.0,
            ],
            |row| row.get::<_, String>(0),
        )
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    if ids.len() != 1 {
        return Err(store_error("winner has no unique retained scoped output"));
    }
    let artifact = load_artifact(conn, &ids[0])?
        .ok_or_else(|| store_error("winner scoped output disappeared"))?;
    if artifact.claim.scope != attempt.scope
        || artifact.claim.task_id != attempt.task_id
        || artifact.claim.attempt_id != attempt.attempt_id
        || artifact.claim.reservation_id != attempt.capacity_reservation
        || artifact.claim.kind != PrivateArtifactKind::ScopedOutput
        || artifact.reference.digest != *expected_digest
    {
        return Err(store_error("winner scoped output owner or digest changed"));
    }
    Ok((artifact.reference, artifact.bytes))
}

pub(crate) fn exact_retained_receipt_by_id(
    conn: &Connection,
    artifact_id: &str,
    reference: &BlobRef,
) -> CoreResult<(PrivateArtifactClaim, ControllerReceiptEnvelope)> {
    let artifact = load_artifact(conn, artifact_id)?
        .ok_or_else(|| store_error("launch settlement receipt absent"))?;
    if artifact.reference != *reference
        || artifact.claim.kind != PrivateArtifactKind::ControllerReceipt
    {
        return Err(store_error(
            "launch settlement receipt reference or kind changed",
        ));
    }
    let receipt: ControllerReceiptEnvelope =
        serde_json::from_slice(&artifact.bytes).map_err(err)?;
    if receipt_bytes(&receipt)? != artifact.bytes {
        return Err(store_error(
            "launch settlement receipt bytes are not canonical",
        ));
    }
    validate_receipt(&artifact.claim, &receipt)?;
    Ok((artifact.claim, receipt))
}

/// An owned launch may use only exact retained controller bytes, not a digest
/// field supplied in a transition request. The native Job still requires its
/// own one-use create and positive settlement protocol.
pub(crate) fn require_retained_launch_checks(
    conn: &Connection,
    attempt: &RoutedAttemptRecord,
    digest: &Digest,
) -> CoreResult<()> {
    let artifact_id: Option<String> = conn
        .query_row(
            "SELECT artifact_id FROM routing_private_artifacts
             WHERE run_id=?1 AND task_id=?2 AND attempt_id=?3
               AND reservation_id=?4 AND kind='controller_receipt' AND digest=?5
             LIMIT 1",
            params![
                attempt.scope.run_id.0,
                attempt.task_id.0,
                attempt.attempt_id.0,
                attempt.capacity_reservation,
                digest.0
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(err)?;
    let artifact_id =
        artifact_id.ok_or_else(|| store_error("owned launch checks are not retained"))?;
    let artifact = load_artifact(conn, &artifact_id)?
        .ok_or_else(|| store_error("owned launch checks disappeared"))?;
    if artifact.reference.digest != *digest
        || artifact.claim.scope != attempt.scope
        || artifact.claim.task_id != attempt.task_id
        || artifact.claim.attempt_id != attempt.attempt_id
        || artifact.claim.reservation_id != attempt.capacity_reservation
    {
        return Err(store_error("owned launch-check receipt owner changed"));
    }
    let receipt: ControllerReceiptEnvelope =
        serde_json::from_slice(&artifact.bytes).map_err(err)?;
    if receipt_bytes(&receipt)? != artifact.bytes {
        return Err(store_error("owned launch-check receipt bytes changed"));
    }
    validate_receipt(&artifact.claim, &receipt)?;
    validate_launch_checks(conn, attempt, &receipt)
}

fn validate_launch_checks(
    conn: &Connection,
    attempt: &RoutedAttemptRecord,
    receipt: &ControllerReceiptEnvelope,
) -> CoreResult<()> {
    let ControllerObservation::LaunchChecks {
        mission_digest,
        qualification_digest,
        launch_fingerprint,
        launch_token,
    } = &receipt.observation
    else {
        return Err(store_error("owned launch receipt is not launch checks"));
    };
    let owner = load_ownership(conn, &attempt.attempt_id)?
        .ok_or_else(|| store_error("owned launch-check owner is absent"))?;
    let registered_digest: String = conn
        .query_row(
            "SELECT registration_digest FROM routing_missions WHERE run_id=?1 AND domain_id=?2",
            params![attempt.scope.run_id.0, attempt.scope.domain_id.0],
            |row| row.get(0),
        )
        .map_err(err)?;
    let selected_qualification = attempt
        .selected
        .observation
        .qualification
        .as_ref()
        .ok_or_else(|| store_error("owned launch qualification is absent"))?
        .digest()
        .map_err(err)?;
    if receipt.source != ReceiptSource::TrustedController
        || receipt.scope != attempt.scope
        || receipt.task_id != attempt.task_id
        || receipt.attempt_id != attempt.attempt_id
        || receipt.reservation_id != attempt.capacity_reservation
        || !attempt.owned_launch_required
        || !matches!(
            attempt.state,
            AttemptState::Preparing | AttemptState::Launching
        )
        || owner.phase != LaunchOwnershipPhase::Prepared
        || owner.request.scope != attempt.scope
        || owner.request.task_id != attempt.task_id
        || owner.request.attempt_id != attempt.attempt_id
        || owner.request.reservation_id != attempt.capacity_reservation
        || owner.request.launch_token != *launch_token
        || registered_digest != mission_digest.0
        || selected_qualification != *qualification_digest
        || attempt.launch_fingerprint != *launch_fingerprint
    {
        return Err(store_error(
            "owned launch-check receipt differs from live authority",
        ));
    }
    Ok(())
}

fn insert_or_replay(
    tx: &Transaction<'_>,
    claim: &PrivateArtifactClaim,
    bytes: &[u8],
) -> std::result::Result<BlobRef, RoutingArtifactError> {
    if let Some(existing) = load_artifact(tx, &claim.artifact_id)? {
        require(
            existing.claim == *claim && existing.bytes == bytes,
            "artifact replay changed",
        )?;
        return Ok(existing.reference);
    }
    insert_artifact(tx, claim, bytes)
}

fn insert_artifact(
    tx: &Transaction<'_>,
    claim: &PrivateArtifactClaim,
    bytes: &[u8],
) -> std::result::Result<BlobRef, RoutingArtifactError> {
    validate_claim(claim)?;
    let requested_bytes = u64::try_from(bytes.len()).map_err(err)?;
    if requested_bytes > MAX_BLOB_BYTES {
        return Err(RoutingArtifactError::TooLarge { requested_bytes });
    }
    let (held_bytes, lengths_match, held_count): (u64, bool, u64) = tx
        .query_row(
            "SELECT COALESCE(SUM(byte_length),0),
                    COALESCE(MIN(CASE WHEN length(bytes)=byte_length THEN 1 ELSE 0 END),1),
                    COUNT(*)
             FROM routing_private_artifacts WHERE run_id=?1",
            [&claim.scope.run_id.0],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(err)?;
    require(
        lengths_match && held_bytes <= MAX_RUN_BYTES,
        "existing private artifact length or run quota is inconsistent",
    )?;
    if held_count >= MAX_RUN_ARTIFACTS {
        return Err(RoutingArtifactError::RunArtifactCountExceeded { held_count });
    }
    if held_bytes
        .checked_add(requested_bytes)
        .is_none_or(|total| total > MAX_RUN_BYTES)
    {
        return Err(RoutingArtifactError::RunQuotaExceeded {
            held_bytes,
            requested_bytes,
        });
    }
    let reference = BlobRef {
        digest: Digest::of_bytes(bytes),
        byte_length: requested_bytes,
    };
    tx.execute(
        "INSERT INTO routing_private_artifacts
         (artifact_id,domain_id,run_id,task_id,attempt_id,reservation_id,kind,event_id,digest,byte_length,claim_json,bytes)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            claim.artifact_id,
            claim.scope.domain_id.0,
            claim.scope.run_id.0,
            claim.task_id.0,
            claim.attempt_id.0,
            claim.reservation_id,
            claim.kind.as_str(),
            claim.event_id,
            reference.digest.0,
            reference.byte_length,
            serde_json::to_string(claim).map_err(err)?,
            bytes,
        ],
    )
    .map_err(err)?;
    Ok(reference)
}

fn require_pre_admission(conn: &Connection, claim: &PrivateArtifactClaim) -> CoreResult<()> {
    let intent = load_intent(conn, &claim.reservation_id)?
        .ok_or_else(|| store_error("pre-admission artifact has no capacity intent"))?;
    require_intent_owner(claim, &intent)?;
    if intent.phase == CapacityIntentPhase::Closed {
        return Err(store_error("pre-admission capacity intent is closed"));
    }
    require_live_registered_task(conn, &intent.request)
}

fn require_intent_owner(
    claim: &PrivateArtifactClaim,
    intent: &crate::routing_capacity_intent::RoutingCapacityIntent,
) -> CoreResult<()> {
    if intent.request.scope != claim.scope
        || intent.request.task_id != claim.task_id
        || intent.request.reservation.attempt_id != claim.attempt_id.0
        || intent.request.reservation.reservation_id != claim.reservation_id
    {
        return Err(store_error("artifact does not match exact capacity intent"));
    }
    Ok(())
}

pub(crate) fn exact_admitted_attempt(
    conn: &Connection,
    claim: &PrivateArtifactClaim,
) -> CoreResult<Option<RoutedAttemptRecord>> {
    let row: Option<(String, String, String, String, u32, u64, String)> = conn
        .query_row(
            "SELECT agent_id,capacity_reservation,run_id,task_id,ordinal,revision,record_json
             FROM routing_attempts WHERE attempt_id=?1",
            [&claim.attempt_id.0],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .optional()
        .map_err(err)?;
    let Some((agent, reservation, run, task, ordinal, revision, body)) = row else {
        return Ok(None);
    };
    let attempt: RoutedAttemptRecord = serde_json::from_str(&body).map_err(err)?;
    if attempt.scope != claim.scope
        || attempt.task_id != claim.task_id
        || attempt.attempt_id != claim.attempt_id
        || attempt.capacity_reservation != claim.reservation_id
        || attempt.agent_id != agent
        || attempt.capacity_reservation != reservation
        || attempt.scope.run_id.0 != run
        || attempt.task_id.0 != task
        || attempt.ordinal != ordinal
        || attempt.revision != revision
        || serde_json::to_string(&attempt).map_err(err)? != body
    {
        return Err(store_error(
            "admitted attempt projection or artifact owner mismatch",
        ));
    }
    Ok(Some(attempt))
}

fn require_admitted_attempt(
    conn: &Connection,
    claim: &PrivateArtifactClaim,
    unreleased: bool,
) -> CoreResult<RoutedAttemptRecord> {
    let attempt = exact_admitted_attempt(conn, claim)?
        .ok_or_else(|| store_error("artifact attempt is not admitted"))?;
    if unreleased && attempt.ownership_released {
        return Err(store_error("artifact attempt ownership is released"));
    }
    Ok(attempt)
}

fn require_admission_absent(conn: &Connection, claim: &PrivateArtifactClaim) -> CoreResult<()> {
    if exact_admitted_attempt(conn, claim)?.is_some() {
        return Err(store_error(
            "no-worker receipt cannot cover an admitted attempt",
        ));
    }
    Ok(())
}

pub(crate) fn retained_reference_matches(
    conn: &Connection,
    claim: &PrivateArtifactClaim,
    kind: PrivateArtifactKind,
    reference: &BlobRef,
) -> CoreResult<bool> {
    let mut stmt = conn
        .prepare(
            "SELECT artifact_id FROM routing_private_artifacts
         WHERE run_id=?1 AND task_id=?2 AND attempt_id=?3 AND reservation_id=?4 AND kind=?5",
        )
        .map_err(err)?;
    let ids = stmt
        .query_map(
            params![
                claim.scope.run_id.0,
                claim.task_id.0,
                claim.attempt_id.0,
                claim.reservation_id,
                kind.as_str()
            ],
            |row| row.get::<_, String>(0),
        )
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    for id in ids {
        let artifact =
            load_artifact(conn, &id)?.ok_or_else(|| store_error("retained artifact vanished"))?;
        if artifact.claim == *claim
            && artifact.claim.kind == kind
            && artifact.reference == *reference
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Admission stores a BlobRef but not the artifact event identity. Resolve its
/// one scoped retained row, then verify that row through the exact claim path.
pub(crate) fn retained_attempt_reference_matches(
    conn: &Connection,
    owner: &PrivateArtifactClaim,
    kind: PrivateArtifactKind,
    reference: &BlobRef,
) -> CoreResult<bool> {
    let mut stmt = conn
        .prepare(
            "SELECT artifact_id FROM routing_private_artifacts
         WHERE domain_id=?1 AND run_id=?2 AND task_id=?3 AND attempt_id=?4
           AND reservation_id=?5 AND kind=?6",
        )
        .map_err(err)?;
    let ids = stmt
        .query_map(
            params![
                owner.scope.domain_id.0,
                owner.scope.run_id.0,
                owner.task_id.0,
                owner.attempt_id.0,
                owner.reservation_id,
                kind.as_str()
            ],
            |row| row.get::<_, String>(0),
        )
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    for id in ids {
        let artifact =
            load_artifact(conn, &id)?.ok_or_else(|| store_error("retained artifact vanished"))?;
        if artifact.reference == *reference
            && retained_reference_matches(conn, &artifact.claim, kind, reference)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate_claim(claim: &PrivateArtifactClaim) -> CoreResult<()> {
    for value in [
        &claim.scope.domain_id.0,
        &claim.scope.run_id.0,
        &claim.task_id.0,
        &claim.attempt_id.0,
        &claim.reservation_id,
        &claim.artifact_id,
        &claim.event_id,
    ] {
        if value.trim().is_empty() || value.len() > MAX_ID_BYTES {
            return Err(store_error(
                "private artifact identity is empty or exceeds 256 bytes",
            ));
        }
    }
    Ok(())
}

pub(crate) fn require_cancelled_event(
    conn: &Connection,
    claim: &PrivateArtifactClaim,
    cancellation_event_id: &str,
) -> CoreResult<()> {
    let cancelled: Option<bool> = conn
        .query_row(
            "SELECT cancelled FROM routing_missions WHERE run_id=?1 AND domain_id=?2",
            params![claim.scope.run_id.0, claim.scope.domain_id.0],
            |row| row.get(0),
        )
        .optional()
        .map_err(err)?;
    if cancelled != Some(true) {
        return Err(store_error(
            "controller no-launch receipt requires durable cancellation",
        ));
    }
    let event_json: Option<String> = conn.query_row(
        "SELECT event_json FROM routing_control_events WHERE run_id=?1 AND domain_id=?2 AND event_id=?3",
        params![claim.scope.run_id.0, claim.scope.domain_id.0, cancellation_event_id],
        |row| row.get(0),
    ).optional().map_err(err)?;
    if !event_json
        .as_deref()
        .and_then(|body| serde_json::from_str::<RoutingControlEvent>(body).ok())
        .is_some_and(|event| matches!(event, RoutingControlEvent::Cancelled { .. }))
    {
        return Err(store_error(
            "controller no-launch cancellation event not retained",
        ));
    }
    Ok(())
}

fn validate_receipt(
    claim: &PrivateArtifactClaim,
    receipt: &ControllerReceiptEnvelope,
) -> CoreResult<()> {
    validate_claim(claim)?;
    if receipt.schema_version != 1
        || receipt.scope != claim.scope
        || receipt.task_id != claim.task_id
        || receipt.attempt_id != claim.attempt_id
        || receipt.reservation_id != claim.reservation_id
        || receipt.evidence_id != claim.event_id
    {
        return Err(store_error(
            "controller receipt identity or schema mismatch",
        ));
    }
    match (&claim.kind, &receipt.source, &receipt.observation) {
        (
            PrivateArtifactKind::NoWorkerReceipt,
            ReceiptSource::TrustedController,
            ControllerObservation::NoWorker {
                cancellation_event_id,
            },
        ) if !cancellation_event_id.trim().is_empty() => Ok(()),
        (
            PrivateArtifactKind::ControllerReceipt,
            ReceiptSource::TrustedController,
            ControllerObservation::AdmittedNoLaunch {
                cancellation_event_id,
            },
        ) if !cancellation_event_id.trim().is_empty() => Ok(()),
        (
            PrivateArtifactKind::ControllerReceipt,
            ReceiptSource::TrustedController,
            ControllerObservation::LaunchChecks {
                mission_digest,
                qualification_digest,
                launch_fingerprint,
                launch_token,
            },
        ) if mission_digest.is_valid()
            && qualification_digest.is_valid()
            && launch_fingerprint.is_valid()
            && !launch_token.trim().is_empty() =>
        {
            Ok(())
        }
        (
            PrivateArtifactKind::ControllerReceipt,
            ReceiptSource::OwnedJobObservation,
            ControllerObservation::NativeJobZero {
                job_name,
                launch_nonce,
                active_processes,
            },
        ) if !job_name.trim().is_empty()
            && !launch_nonce.trim().is_empty()
            && *active_processes == 0 =>
        {
            Ok(())
        }
        (
            PrivateArtifactKind::ControllerReceipt,
            ReceiptSource::OwnedJobObservation,
            ControllerObservation::NativeCheckerResult {
                check_id,
                ordinal,
                job_name,
                launch_nonce,
                active_processes,
                native_outcome,
                error_present,
                sealed_view_before,
                sealed_view_after,
                ..
            },
        ) if !check_id.0.trim().is_empty()
            && (1..=16).contains(ordinal)
            && !job_name.trim().is_empty()
            && !launch_nonce.trim().is_empty()
            && *active_processes == 0
            && sealed_view_before.digest.is_valid()
            && sealed_view_after
                .as_ref()
                .is_none_or(|view| view.digest.is_valid())
            && (sealed_view_after.is_some()
                || (*error_present && *native_outcome != CheckerNativeOutcome::Succeeded)) =>
        {
            Ok(())
        }
        (
            PrivateArtifactKind::ControllerReceipt,
            ReceiptSource::TrustedController,
            ControllerObservation::CheckerNotCreated {
                check_id,
                ordinal,
                cancellation_event_id,
            },
        ) if !check_id.0.trim().is_empty()
            && (1..=16).contains(ordinal)
            && !cancellation_event_id.trim().is_empty() =>
        {
            Ok(())
        }
        _ => Err(store_error(
            "controller receipt kind, source or observation mismatch",
        )),
    }
}

fn receipt_bytes(receipt: &ControllerReceiptEnvelope) -> CoreResult<Vec<u8>> {
    serde_json::to_vec(receipt).map_err(err)
}

fn immediate(conn: &Connection) -> CoreResult<Transaction<'_>> {
    Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(err)
}
fn err(error: impl std::fmt::Display) -> PytxoError {
    store_error(&error.to_string())
}
fn store_error(message: &str) -> PytxoError {
    PytxoError::Store(message.into())
}
fn invalid(message: &str) -> RoutingArtifactError {
    RoutingArtifactError::Store(store_error(message))
}
fn require(condition: bool, message: &str) -> std::result::Result<(), RoutingArtifactError> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
