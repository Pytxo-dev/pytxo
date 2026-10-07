use std::collections::HashSet;

use pytxo_core::{PytxoError, Result};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;

const CAPACITY_POOLS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS capacity_pools (
    resource_id TEXT PRIMARY KEY NOT NULL,
    capacity_units INTEGER NOT NULL CHECK (capacity_units >= 0),
    revision INTEGER NOT NULL CHECK (revision > 0),
    configured_at_ms INTEGER NOT NULL CHECK (configured_at_ms >= 0)
)"#;

const CAPACITY_RESERVATIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS capacity_reservations (
    reservation_id TEXT PRIMARY KEY NOT NULL,
    domain_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    attempt_id TEXT NOT NULL,
    owner_process_id INTEGER NOT NULL CHECK (owner_process_id > 0),
    owner_process_start_identity TEXT NOT NULL CHECK (length(owner_process_start_identity) > 0),
    state TEXT NOT NULL CHECK (state IN ('provisional', 'bound', 'recovery_required', 'released')),
    launch_token TEXT,
    requested_at_ms INTEGER NOT NULL CHECK (requested_at_ms >= 0),
    bound_at_ms INTEGER,
    recovery_evidence_id TEXT,
    recovery_reason TEXT,
    recovery_observed_at_ms INTEGER,
    release_evidence_kind TEXT CHECK (release_evidence_kind IN ('known_unused', 'quiescence_reconciled')),
    release_receipt_id TEXT,
    release_evidence_digest TEXT,
    release_observed_at_ms INTEGER,
    UNIQUE (domain_id, run_id, attempt_id),
    CHECK (launch_token IS NULL OR length(launch_token) > 0),
    CHECK (bound_at_ms IS NULL OR bound_at_ms >= 0),
    CHECK (recovery_observed_at_ms IS NULL OR recovery_observed_at_ms >= 0),
    CHECK (release_observed_at_ms IS NULL OR release_observed_at_ms >= 0),
    CHECK (
        (state = 'provisional' AND launch_token IS NULL AND bound_at_ms IS NULL)
        OR (state = 'bound' AND launch_token IS NOT NULL AND bound_at_ms IS NOT NULL)
        OR (state IN ('recovery_required', 'released'))
    ),
    CHECK (
        (recovery_evidence_id IS NULL AND recovery_reason IS NULL
            AND recovery_observed_at_ms IS NULL)
        OR (state IN ('recovery_required', 'released')
            AND recovery_evidence_id IS NOT NULL AND length(recovery_evidence_id) > 0
            AND recovery_reason IS NOT NULL AND length(recovery_reason) > 0
            AND recovery_observed_at_ms IS NOT NULL)
    ),
    CHECK (
        (state = 'released' AND release_evidence_kind IS NOT NULL
            AND release_receipt_id IS NOT NULL AND release_evidence_digest IS NOT NULL
            AND release_observed_at_ms IS NOT NULL)
        OR (state != 'released' AND release_evidence_kind IS NULL
            AND release_receipt_id IS NULL AND release_evidence_digest IS NULL
            AND release_observed_at_ms IS NULL)
    )
)"#;

const CAPACITY_RESERVATION_RESOURCES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS capacity_reservation_resources (
    reservation_id TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    units INTEGER NOT NULL CHECK (units > 0),
    pool_revision INTEGER NOT NULL CHECK (pool_revision > 0),
    capacity_units_at_reserve INTEGER NOT NULL CHECK (capacity_units_at_reserve >= 0),
    PRIMARY KEY (reservation_id, resource_id),
    FOREIGN KEY (reservation_id) REFERENCES capacity_reservations(reservation_id) ON DELETE RESTRICT,
    FOREIGN KEY (resource_id) REFERENCES capacity_pools(resource_id) ON DELETE RESTRICT
)"#;

const CAPACITY_STATE_INDEX: &str =
    "CREATE INDEX IF NOT EXISTS idx_capacity_reservations_state ON capacity_reservations(state)";
const CAPACITY_RESOURCE_INDEX: &str = "CREATE INDEX IF NOT EXISTS idx_capacity_reservation_resources_resource ON capacity_reservation_resources(resource_id, reservation_id)";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityPoolConfig {
    pub resource_id: String,
    pub capacity_units: u64,
    /// `None` creates a missing pool. `Some(revision)` updates only that revision.
    pub expected_revision: Option<u64>,
    pub configured_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityPoolStatus {
    pub resource_id: String,
    pub capacity_units: u64,
    pub held_units: u64,
    pub available_units: u64,
    pub revision: u64,
    pub configured_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityOwner {
    pub process_id: u32,
    pub process_start_identity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityResourceRequest {
    pub resource_id: String,
    pub units: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityReservedResource {
    pub resource_id: String,
    pub units: u64,
    pub pool_revision: u64,
    pub capacity_units_at_reserve: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityReservationRequest {
    pub reservation_id: String,
    pub domain_id: String,
    pub run_id: String,
    pub attempt_id: String,
    pub owner: CapacityOwner,
    pub resources: Vec<CapacityResourceRequest>,
    pub requested_at_ms: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapacityReservationState {
    Provisional,
    Bound,
    RecoveryRequired,
    Released,
}

impl CapacityReservationState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Provisional => "provisional",
            Self::Bound => "bound",
            Self::RecoveryRequired => "recovery_required",
            Self::Released => "released",
        }
    }

    fn parse(value: &str) -> rusqlite::Result<Self> {
        match value {
            "provisional" => Ok(Self::Provisional),
            "bound" => Ok(Self::Bound),
            "recovery_required" => Ok(Self::RecoveryRequired),
            "released" => Ok(Self::Released),
            other => Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                format!("invalid capacity reservation state {other}").into(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapacityReleaseEvidenceKind {
    /// Positive evidence that this reservation's worker was never launched and that the
    /// attempt/launch token can no longer launch through the shared domain gate.
    KnownUnused,
    /// Positive evidence that a launched worker and its owned descendants are quiescent.
    QuiescenceReconciled,
}

impl CapacityReleaseEvidenceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::KnownUnused => "known_unused",
            Self::QuiescenceReconciled => "quiescence_reconciled",
        }
    }

    fn parse(value: &str) -> rusqlite::Result<Self> {
        match value {
            "known_unused" => Ok(Self::KnownUnused),
            "quiescence_reconciled" => Ok(Self::QuiescenceReconciled),
            other => Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                format!("invalid capacity release evidence kind {other}").into(),
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityReleaseEvidence {
    pub kind: CapacityReleaseEvidenceKind,
    pub receipt_id: String,
    pub evidence_digest: String,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityReservationRecord {
    pub reservation_id: String,
    pub domain_id: String,
    pub run_id: String,
    pub attempt_id: String,
    pub owner: CapacityOwner,
    pub resources: Vec<CapacityReservedResource>,
    pub state: CapacityReservationState,
    pub launch_token: Option<String>,
    pub requested_at_ms: u64,
    pub bound_at_ms: Option<u64>,
    pub recovery_evidence_id: Option<String>,
    pub recovery_reason: Option<String>,
    pub recovery_observed_at_ms: Option<u64>,
    pub release_evidence: Option<CapacityReleaseEvidence>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityBindRequest {
    pub reservation_id: String,
    pub attempt_id: String,
    pub launch_token: String,
    pub bound_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityRecoveryRequest {
    pub reservation_id: String,
    pub evidence_id: String,
    pub reason: String,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityReleaseRequest {
    pub reservation_id: String,
    pub evidence: CapacityReleaseEvidence,
}

/// Host release of a reservation whose launch token was bound. The trusted
/// controller must establish matching Store ownership and positive no-create
/// or Job-zero evidence before calling this Catalog operation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapacityBoundReleaseRequest {
    pub reservation_id: String,
    pub domain_id: String,
    pub run_id: String,
    pub attempt_id: String,
    pub owner: CapacityOwner,
    pub launch_token: String,
    pub evidence: CapacityReleaseEvidence,
}

impl Catalog {
    /// Configure one controller-owned host-local pool with compare-and-set semantics.
    pub fn configure_capacity_pool(
        &self,
        config: &CapacityPoolConfig,
    ) -> Result<CapacityPoolStatus> {
        require_id("resource_id", &config.resource_id)?;
        let capacity = to_i64("capacity_units", config.capacity_units)?;
        let configured_at = to_i64("configured_at_ms", config.configured_at_ms)?;
        let tx = immediate(&self.conn)?;
        let current = pool_status(&tx, &config.resource_id)?;
        match (current.as_ref(), config.expected_revision) {
            (None, None) => {
                tx.execute(
                    "INSERT INTO capacity_pools(resource_id,capacity_units,revision,configured_at_ms)
                     VALUES (?1,?2,1,?3)",
                    params![config.resource_id, capacity, configured_at],
                )
                .map_err(store_error)?;
            }
            (Some(status), Some(expected)) if status.revision == expected => {
                let next_revision = status
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| PytxoError::Store("capacity pool revision overflow".into()))?;
                let changed = tx
                    .execute(
                        "UPDATE capacity_pools SET capacity_units=?1,revision=?2,
                         configured_at_ms=?3 WHERE resource_id=?4 AND revision=?5",
                        params![
                            capacity,
                            to_i64("next capacity revision", next_revision)?,
                            configured_at,
                            config.resource_id,
                            to_i64("expected_revision", expected)?
                        ],
                    )
                    .map_err(store_error)?;
                if changed != 1 {
                    return Err(PytxoError::Store(
                        "capacity pool revision changed during update".into(),
                    ));
                }
            }
            (None, Some(_)) => {
                return Err(PytxoError::Store(format!(
                    "capacity pool {} does not exist at the expected revision",
                    config.resource_id
                )));
            }
            (Some(status), _) => {
                return Err(PytxoError::Store(format!(
                    "capacity pool {} revision conflict: current {}",
                    config.resource_id, status.revision
                )));
            }
        }
        let status = pool_status(&tx, &config.resource_id)?.ok_or_else(|| {
            PytxoError::Store("configured capacity pool was not persisted".into())
        })?;
        tx.commit().map_err(store_error)?;
        Ok(status)
    }

    /// Atomically reserve every requested resource or reserve none of them.
    pub fn reserve_capacity(
        &self,
        request: &CapacityReservationRequest,
    ) -> Result<CapacityReservationRecord> {
        validate_reservation_request(request)?;
        let resources = canonical_resources(&request.resources)?;
        let tx = immediate(&self.conn)?;

        if let Some(existing) = load_reservation(&tx, &request.reservation_id)? {
            if same_reservation_request(&existing, request, &resources) {
                tx.commit().map_err(store_error)?;
                return Ok(existing);
            }
            return Err(PytxoError::Store(format!(
                "capacity reservation {} conflicts with its immutable request",
                request.reservation_id
            )));
        }

        let existing_attempt: Option<String> = tx
            .query_row(
                "SELECT reservation_id FROM capacity_reservations
                 WHERE domain_id=?1 AND run_id=?2 AND attempt_id=?3",
                params![request.domain_id, request.run_id, request.attempt_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(store_error)?;
        if let Some(existing_id) = existing_attempt {
            return Err(PytxoError::Store(format!(
                "attempt already owns capacity reservation {existing_id}"
            )));
        }

        let mut snapshots = Vec::with_capacity(resources.len());
        for resource in &resources {
            let pool: Option<(i64, i64)> = tx
                .query_row(
                    "SELECT capacity_units,revision FROM capacity_pools WHERE resource_id=?1",
                    params![resource.resource_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(store_error)?;
            let (capacity, revision) = pool.ok_or_else(|| {
                PytxoError::Store(format!(
                    "capacity pool {} is not configured",
                    resource.resource_id
                ))
            })?;
            let held: i64 = tx
                .query_row(
                    "SELECT COALESCE(SUM(rr.units),0)
                     FROM capacity_reservation_resources rr
                     JOIN capacity_reservations r ON r.reservation_id=rr.reservation_id
                     WHERE rr.resource_id=?1 AND r.state!='released'",
                    params![resource.resource_id],
                    |row| row.get(0),
                )
                .map_err(store_error)?;
            let requested = to_i64("resource units", resource.units)?;
            let after = held.checked_add(requested).ok_or_else(|| {
                PytxoError::Store(format!(
                    "capacity occupancy overflow for {}",
                    resource.resource_id
                ))
            })?;
            if requested > capacity || after > capacity {
                return Err(PytxoError::Store(format!(
                    "insufficient capacity for {}: configured {capacity}, held {held}, requested {requested}",
                    resource.resource_id
                )));
            }
            snapshots.push(CapacityReservedResource {
                resource_id: resource.resource_id.clone(),
                units: resource.units,
                pool_revision: from_i64("pool revision", revision)?,
                capacity_units_at_reserve: from_i64("capacity units", capacity)?,
            });
        }

        tx.execute(
            "INSERT INTO capacity_reservations(
                reservation_id,domain_id,run_id,attempt_id,owner_process_id,
                owner_process_start_identity,state,requested_at_ms
             ) VALUES (?1,?2,?3,?4,?5,?6,'provisional',?7)",
            params![
                request.reservation_id,
                request.domain_id,
                request.run_id,
                request.attempt_id,
                i64::from(request.owner.process_id),
                request.owner.process_start_identity,
                to_i64("requested_at_ms", request.requested_at_ms)?
            ],
        )
        .map_err(store_error)?;
        for resource in &snapshots {
            tx.execute(
                "INSERT INTO capacity_reservation_resources(
                    reservation_id,resource_id,units,pool_revision,capacity_units_at_reserve
                 ) VALUES (?1,?2,?3,?4,?5)",
                params![
                    request.reservation_id,
                    resource.resource_id,
                    to_i64("resource units", resource.units)?,
                    to_i64("pool revision", resource.pool_revision)?,
                    to_i64(
                        "capacity units at reserve",
                        resource.capacity_units_at_reserve
                    )?
                ],
            )
            .map_err(store_error)?;
        }
        let record = required_reservation(&tx, &request.reservation_id)?;
        tx.commit().map_err(store_error)?;
        Ok(record)
    }

    pub fn bind_capacity_reservation(
        &self,
        request: &CapacityBindRequest,
    ) -> Result<CapacityReservationRecord> {
        require_id("reservation_id", &request.reservation_id)?;
        require_id("attempt_id", &request.attempt_id)?;
        require_id("launch_token", &request.launch_token)?;
        let bound_at = to_i64("bound_at_ms", request.bound_at_ms)?;
        let tx = immediate(&self.conn)?;
        let current = required_reservation(&tx, &request.reservation_id)?;
        if current.attempt_id != request.attempt_id {
            return Err(PytxoError::Store(
                "capacity binding attempt does not match the reservation".into(),
            ));
        }
        match current.state {
            CapacityReservationState::Provisional => {
                tx.execute(
                    "UPDATE capacity_reservations
                     SET state='bound',launch_token=?1,bound_at_ms=?2
                     WHERE reservation_id=?3 AND state='provisional'",
                    params![request.launch_token, bound_at, request.reservation_id],
                )
                .map_err(store_error)?;
            }
            CapacityReservationState::Bound
                if current.launch_token.as_deref() == Some(&request.launch_token)
                    && current.bound_at_ms == Some(request.bound_at_ms) => {}
            CapacityReservationState::Bound => {
                return Err(PytxoError::Store(
                    "capacity reservation is already bound to different launch evidence".into(),
                ));
            }
            CapacityReservationState::RecoveryRequired => {
                return Err(PytxoError::Store(
                    "capacity reservation requires recovery and cannot be rebound".into(),
                ));
            }
            CapacityReservationState::Released => {
                return Err(PytxoError::Store(
                    "released capacity reservation cannot be rebound".into(),
                ));
            }
        }
        let result = required_reservation(&tx, &request.reservation_id)?;
        tx.commit().map_err(store_error)?;
        Ok(result)
    }

    pub fn mark_capacity_recovery_required(
        &self,
        request: &CapacityRecoveryRequest,
    ) -> Result<CapacityReservationRecord> {
        require_id("reservation_id", &request.reservation_id)?;
        require_id("recovery evidence_id", &request.evidence_id)?;
        require_id("recovery reason", &request.reason)?;
        let observed_at = to_i64("recovery observed_at_ms", request.observed_at_ms)?;
        let tx = immediate(&self.conn)?;
        let current = required_reservation(&tx, &request.reservation_id)?;
        match current.state {
            CapacityReservationState::Provisional | CapacityReservationState::Bound => {
                tx.execute(
                    "UPDATE capacity_reservations SET
                       state='recovery_required',recovery_evidence_id=?1,
                       recovery_reason=?2,recovery_observed_at_ms=?3
                     WHERE reservation_id=?4 AND state IN ('provisional','bound')",
                    params![
                        request.evidence_id,
                        request.reason,
                        observed_at,
                        request.reservation_id
                    ],
                )
                .map_err(store_error)?;
            }
            CapacityReservationState::RecoveryRequired
                if current.recovery_evidence_id.as_deref() == Some(&request.evidence_id)
                    && current.recovery_reason.as_deref() == Some(&request.reason)
                    && current.recovery_observed_at_ms == Some(request.observed_at_ms) => {}
            CapacityReservationState::RecoveryRequired => {
                return Err(PytxoError::Store(
                    "capacity recovery evidence conflicts with the existing hold".into(),
                ));
            }
            CapacityReservationState::Released => {
                return Err(PytxoError::Store(
                    "released capacity reservation cannot require recovery".into(),
                ));
            }
        }
        let result = required_reservation(&tx, &request.reservation_id)?;
        tx.commit().map_err(store_error)?;
        Ok(result)
    }

    /// Release a never-bound provisional or recovery-required reservation from
    /// trusted positive no-launch evidence. A bound launch token requires the
    /// distinct exact-owner API.
    pub fn release_capacity_reservation(
        &self,
        request: &CapacityReleaseRequest,
    ) -> Result<CapacityReservationRecord> {
        self.release_capacity_reservation_inner(request, None)
    }

    /// Release a previously bound reservation only after trusted domain proof
    /// identifies the same owner and launch token. Catalog checks identity and
    /// host occupancy; it does not observe the native Job itself.
    pub fn release_bound_capacity_reservation(
        &self,
        request: &CapacityBoundReleaseRequest,
    ) -> Result<CapacityReservationRecord> {
        require_id("bound domain_id", &request.domain_id)?;
        require_id("bound run_id", &request.run_id)?;
        require_id("bound attempt_id", &request.attempt_id)?;
        require_id("bound launch_token", &request.launch_token)?;
        if request.owner.process_id == 0 {
            return Err(PytxoError::Store("bound owner process_id is zero".into()));
        }
        require_id(
            "bound owner process_start_identity",
            &request.owner.process_start_identity,
        )?;
        self.release_capacity_reservation_inner(
            &CapacityReleaseRequest {
                reservation_id: request.reservation_id.clone(),
                evidence: request.evidence.clone(),
            },
            Some(request),
        )
    }

    fn release_capacity_reservation_inner(
        &self,
        request: &CapacityReleaseRequest,
        bound: Option<&CapacityBoundReleaseRequest>,
    ) -> Result<CapacityReservationRecord> {
        validate_release_request(request)?;
        let tx = immediate(&self.conn)?;
        let current = required_reservation(&tx, &request.reservation_id)?;
        if current.launch_token.is_some() != current.bound_at_ms.is_some() {
            return Err(PytxoError::Store(
                "capacity reservation has inconsistent launch binding".into(),
            ));
        }
        let was_bound = current.launch_token.is_some();
        match bound {
            Some(exact)
                if was_bound
                    && current.domain_id == exact.domain_id
                    && current.run_id == exact.run_id
                    && current.attempt_id == exact.attempt_id
                    && current.owner == exact.owner
                    && current.launch_token.as_deref() == Some(exact.launch_token.as_str()) => {}
            None if !was_bound => {}
            _ => {
                return Err(PytxoError::Store(
                    "capacity release requires matching bound owner and launch token".into(),
                ));
            }
        }
        if current.state == CapacityReservationState::Released {
            if current.release_evidence.as_ref() == Some(&request.evidence) {
                tx.commit().map_err(store_error)?;
                return Ok(current);
            }
            return Err(PytxoError::Store(
                "capacity reservation was released with different evidence".into(),
            ));
        }
        let allowed = match current.state {
            CapacityReservationState::Provisional => {
                !was_bound && request.evidence.kind == CapacityReleaseEvidenceKind::KnownUnused
            }
            CapacityReservationState::Bound => was_bound,
            CapacityReservationState::RecoveryRequired => {
                was_bound || request.evidence.kind == CapacityReleaseEvidenceKind::KnownUnused
            }
            CapacityReservationState::Released => unreachable!(),
        };
        if !allowed {
            return Err(PytxoError::Store(format!(
                "{} reservation cannot be released by {} evidence",
                current.state.as_str(),
                request.evidence.kind.as_str()
            )));
        }
        tx.execute(
            "UPDATE capacity_reservations SET
               state='released',release_evidence_kind=?1,release_receipt_id=?2,
               release_evidence_digest=?3,release_observed_at_ms=?4
             WHERE reservation_id=?5 AND state!='released'",
            params![
                request.evidence.kind.as_str(),
                request.evidence.receipt_id,
                request.evidence.evidence_digest,
                to_i64("release observed_at_ms", request.evidence.observed_at_ms)?,
                request.reservation_id
            ],
        )
        .map_err(store_error)?;
        let result = required_reservation(&tx, &request.reservation_id)?;
        tx.commit().map_err(store_error)?;
        Ok(result)
    }

    pub fn capacity_reservation(
        &self,
        reservation_id: &str,
    ) -> Result<Option<CapacityReservationRecord>> {
        require_id("reservation_id", reservation_id)?;
        load_reservation(&self.conn, reservation_id)
    }

    pub fn unresolved_capacity_reservations(&self) -> Result<Vec<CapacityReservationRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT reservation_id FROM capacity_reservations
                 WHERE state!='released' ORDER BY reservation_id",
            )
            .map_err(store_error)?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(store_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(store_error)?;
        ids.into_iter()
            .map(|id| required_reservation(&self.conn, &id))
            .collect()
    }

    pub fn capacity_pool_status(&self, resource_id: &str) -> Result<Option<CapacityPoolStatus>> {
        require_id("resource_id", resource_id)?;
        pool_status(&self.conn, resource_id)
    }
}

pub(crate) fn migrate_catalog_capacity(conn: &Connection) -> Result<()> {
    conn.execute_batch(&format!(
        "{CAPACITY_POOLS_TABLE};
         {CAPACITY_RESERVATIONS_TABLE};
         {CAPACITY_RESERVATION_RESOURCES_TABLE};
         {CAPACITY_STATE_INDEX};
         {CAPACITY_RESOURCE_INDEX};"
    ))
    .map_err(store_error)?;
    verify_catalog_capacity(conn)?;
    conn.pragma_update(None, "user_version", 2)
        .map_err(store_error)
}

pub(crate) fn verify_catalog_capacity(conn: &Connection) -> Result<()> {
    let expected = Connection::open_in_memory().map_err(store_error)?;
    expected
        .execute_batch(&format!(
            "{CAPACITY_POOLS_TABLE};
             {CAPACITY_RESERVATIONS_TABLE};
             {CAPACITY_RESERVATION_RESOURCES_TABLE};
             {CAPACITY_STATE_INDEX};
             {CAPACITY_RESOURCE_INDEX};"
        ))
        .map_err(store_error)?;
    verify_schema_object(conn, &expected, "table", "capacity_pools")?;
    verify_schema_object(conn, &expected, "table", "capacity_reservations")?;
    verify_schema_object(conn, &expected, "table", "capacity_reservation_resources")?;
    verify_schema_object(conn, &expected, "index", "idx_capacity_reservations_state")?;
    verify_schema_object(
        conn,
        &expected,
        "index",
        "idx_capacity_reservation_resources_resource",
    )
}

fn verify_schema_object(
    conn: &Connection,
    expected: &Connection,
    kind: &str,
    name: &str,
) -> Result<()> {
    let expected_sql: String = expected
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
            params![kind, name],
            |row| row.get(0),
        )
        .map_err(store_error)?;
    let actual: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
            params![kind, name],
            |row| row.get(0),
        )
        .optional()
        .map_err(store_error)?;
    let actual = actual.ok_or_else(|| {
        PytxoError::Store(format!("catalog capacity schema is missing {kind} {name}"))
    })?;
    if actual != expected_sql {
        return Err(PytxoError::Store(format!(
            "catalog capacity schema has malformed {kind} {name}"
        )));
    }
    Ok(())
}

fn immediate(conn: &Connection) -> Result<Transaction<'_>> {
    Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(store_error)
}

pub(crate) fn validate_reservation_request(request: &CapacityReservationRequest) -> Result<()> {
    require_id("reservation_id", &request.reservation_id)?;
    require_id("domain_id", &request.domain_id)?;
    require_id("run_id", &request.run_id)?;
    require_id("attempt_id", &request.attempt_id)?;
    if request.owner.process_id == 0 {
        return Err(PytxoError::Store(
            "capacity owner process_id must be positive".into(),
        ));
    }
    require_id(
        "owner process_start_identity",
        &request.owner.process_start_identity,
    )?;
    to_i64("requested_at_ms", request.requested_at_ms)?;
    canonical_resources(&request.resources).map(|_| ())
}

pub(crate) fn canonical_resources(
    resources: &[CapacityResourceRequest],
) -> Result<Vec<CapacityResourceRequest>> {
    if resources.is_empty() {
        return Err(PytxoError::Store(
            "capacity reservation requires at least one resource".into(),
        ));
    }
    let mut canonical = resources.to_vec();
    canonical.sort_by(|left, right| left.resource_id.cmp(&right.resource_id));
    let mut seen = HashSet::with_capacity(canonical.len());
    for resource in &canonical {
        require_id("resource_id", &resource.resource_id)?;
        if resource.units == 0 {
            return Err(PytxoError::Store(format!(
                "capacity resource {} requested zero units",
                resource.resource_id
            )));
        }
        to_i64("resource units", resource.units)?;
        if !seen.insert(resource.resource_id.as_str()) {
            return Err(PytxoError::Store(format!(
                "duplicate capacity resource {}",
                resource.resource_id
            )));
        }
    }
    Ok(canonical)
}

fn same_reservation_request(
    existing: &CapacityReservationRecord,
    request: &CapacityReservationRequest,
    resources: &[CapacityResourceRequest],
) -> bool {
    existing.reservation_id == request.reservation_id
        && existing.domain_id == request.domain_id
        && existing.run_id == request.run_id
        && existing.attempt_id == request.attempt_id
        && existing.owner == request.owner
        && existing.requested_at_ms == request.requested_at_ms
        && existing
            .resources
            .iter()
            .map(|resource| CapacityResourceRequest {
                resource_id: resource.resource_id.clone(),
                units: resource.units,
            })
            .eq(resources.iter().cloned())
}

pub(crate) fn validate_release_request(request: &CapacityReleaseRequest) -> Result<()> {
    require_id("reservation_id", &request.reservation_id)?;
    require_id("release receipt_id", &request.evidence.receipt_id)?;
    require_id("release evidence_digest", &request.evidence.evidence_digest)?;
    to_i64("release observed_at_ms", request.evidence.observed_at_ms)?;
    Ok(())
}

fn require_id(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(PytxoError::Store(format!("{label} must not be empty")));
    }
    Ok(())
}

fn pool_status(conn: &Connection, resource_id: &str) -> Result<Option<CapacityPoolStatus>> {
    let configured: Option<(i64, i64, i64)> = conn
        .query_row(
            "SELECT capacity_units,revision,configured_at_ms
             FROM capacity_pools WHERE resource_id=?1",
            params![resource_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(store_error)?;
    let Some((capacity, revision, configured_at)) = configured else {
        return Ok(None);
    };
    let held: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(rr.units),0)
             FROM capacity_reservation_resources rr
             JOIN capacity_reservations r ON r.reservation_id=rr.reservation_id
             WHERE rr.resource_id=?1 AND r.state!='released'",
            params![resource_id],
            |row| row.get(0),
        )
        .map_err(store_error)?;
    let capacity_units = from_i64("capacity units", capacity)?;
    let held_units = from_i64("held units", held)?;
    Ok(Some(CapacityPoolStatus {
        resource_id: resource_id.to_owned(),
        capacity_units,
        held_units,
        available_units: capacity_units.saturating_sub(held_units),
        revision: from_i64("pool revision", revision)?,
        configured_at_ms: from_i64("configured_at_ms", configured_at)?,
    }))
}

#[allow(clippy::type_complexity)]
fn load_reservation(
    conn: &Connection,
    reservation_id: &str,
) -> Result<Option<CapacityReservationRecord>> {
    let row = conn
        .query_row(
            "SELECT domain_id,run_id,attempt_id,owner_process_id,
                    owner_process_start_identity,state,launch_token,requested_at_ms,bound_at_ms,
                    recovery_evidence_id,recovery_reason,recovery_observed_at_ms,
                    release_evidence_kind,release_receipt_id,release_evidence_digest,
                    release_observed_at_ms
             FROM capacity_reservations WHERE reservation_id=?1",
            params![reservation_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<i64>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, Option<String>>(14)?,
                    row.get::<_, Option<i64>>(15)?,
                ))
            },
        )
        .optional()
        .map_err(store_error)?;
    let Some((
        domain_id,
        run_id,
        attempt_id,
        owner_process_id,
        owner_process_start_identity,
        state,
        launch_token,
        requested_at_ms,
        bound_at_ms,
        recovery_evidence_id,
        recovery_reason,
        recovery_observed_at_ms,
        release_kind,
        release_receipt_id,
        release_evidence_digest,
        release_observed_at_ms,
    )) = row
    else {
        return Ok(None);
    };

    let mut statement = conn
        .prepare(
            "SELECT resource_id,units,pool_revision,capacity_units_at_reserve
             FROM capacity_reservation_resources
             WHERE reservation_id=?1 ORDER BY resource_id",
        )
        .map_err(store_error)?;
    let resources = statement
        .query_map(params![reservation_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(store_error)?
        .map(|row| {
            let (resource_id, units, pool_revision, capacity_at_reserve) =
                row.map_err(store_error)?;
            Ok(CapacityReservedResource {
                resource_id,
                units: from_i64("resource units", units)?,
                pool_revision: from_i64("pool revision", pool_revision)?,
                capacity_units_at_reserve: from_i64(
                    "capacity units at reserve",
                    capacity_at_reserve,
                )?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let release_evidence = match (
        release_kind,
        release_receipt_id,
        release_evidence_digest,
        release_observed_at_ms,
    ) {
        (None, None, None, None) => None,
        (Some(kind), Some(receipt_id), Some(evidence_digest), Some(observed_at_ms)) => {
            Some(CapacityReleaseEvidence {
                kind: CapacityReleaseEvidenceKind::parse(&kind).map_err(store_error)?,
                receipt_id,
                evidence_digest,
                observed_at_ms: from_i64("release observed_at_ms", observed_at_ms)?,
            })
        }
        _ => {
            return Err(PytxoError::Store(format!(
                "capacity reservation {reservation_id} has incomplete release evidence"
            )));
        }
    };
    Ok(Some(CapacityReservationRecord {
        reservation_id: reservation_id.to_owned(),
        domain_id,
        run_id,
        attempt_id,
        owner: CapacityOwner {
            process_id: u32::try_from(owner_process_id).map_err(|_| {
                PytxoError::Store("capacity owner process_id is outside u32".into())
            })?,
            process_start_identity: owner_process_start_identity,
        },
        resources,
        state: CapacityReservationState::parse(&state).map_err(store_error)?,
        launch_token,
        requested_at_ms: from_i64("requested_at_ms", requested_at_ms)?,
        bound_at_ms: bound_at_ms
            .map(|value| from_i64("bound_at_ms", value))
            .transpose()?,
        recovery_evidence_id,
        recovery_reason,
        recovery_observed_at_ms: recovery_observed_at_ms
            .map(|value| from_i64("recovery observed_at_ms", value))
            .transpose()?,
        release_evidence,
    }))
}

fn required_reservation(
    conn: &Connection,
    reservation_id: &str,
) -> Result<CapacityReservationRecord> {
    load_reservation(conn, reservation_id)?.ok_or_else(|| {
        PytxoError::Store(format!("capacity reservation not found: {reservation_id}"))
    })
}

fn to_i64(label: &str, value: u64) -> Result<i64> {
    i64::try_from(value)
        .map_err(|_| PytxoError::Store(format!("{label} exceeds SQLite integer range")))
}

fn from_i64(label: &str, value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|_| PytxoError::Store(format!("{label} is negative")))
}

fn store_error(error: rusqlite::Error) -> PytxoError {
    PytxoError::Store(error.to_string())
}
