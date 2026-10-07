//! Durable routed execution. Only the trusted controller may write observations and
//! receipts here; this ledger does not inspect processes or attest their contents.
mod advisor;
pub use advisor::*;

use std::collections::{BTreeMap, BTreeSet};

use pytxo_core::routing::*;
use pytxo_core::{
    DomainId, PermissionProfile, PreparedRunFileKind, PreparedRunManifest, PytxoError, Result,
    RunId, TaskId,
};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::routing_private::{handoff_manifest_claim, retained_handoff_bytes};
use crate::PytxoStore;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckPlatform {
    Windows,
    Posix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckCwdKind {
    FreshSealedVerificationView,
}

/// Private, reviewed data for an independent local checker. This is not an
/// enforcement receipt and cannot attest network isolation or a passed check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenCheckExecutorV1 {
    pub policy_version: u32,
    pub platform: CheckPlatform,
    pub shell: ExecutableIdentity,
    pub shell_args: Vec<String>,
    pub cwd_kind: CheckCwdKind,
    pub permission_profile: PermissionProfile,
    pub stdin_closed: bool,
    pub environment_policy_version: u32,
    pub network_policy_version: u32,
    pub timeout_ms: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenCheckRecipeV1 {
    pub schema_version: u32,
    pub id: CheckId,
    pub ordinal: u32,
    pub command: String,
    pub executor: FrozenCheckExecutorV1,
}

impl FrozenCheckRecipeV1 {
    pub fn reference(&self) -> Result<CheckRecipe> {
        validate_frozen_recipe(self)?;
        Ok(CheckRecipe {
            id: self.id.clone(),
            recipe_digest: hash(self)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredTask {
    pub contract: TaskContract,
    /// Frozen approved reservation per attempt; never supplied by an admission request.
    pub attempt_budget_nano_usd: u64,
    /// Missing on v8/v9 registrations. Those rows remain readable but cannot
    /// pass the experimental routed launch gate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub check_recipes: Vec<FrozenCheckRecipeV1>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredProfile {
    pub profile: ExecutionProfile,
    pub binding: ProfileBinding,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingMission {
    pub authorization: MissionAuthorization,
    pub policy: RoutingPolicy,
    pub tasks: Vec<RegisteredTask>,
    pub profiles: Vec<RegisteredProfile>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingScope {
    pub domain_id: DomainId,
    pub run_id: RunId,
}
/// Safe review identity. The full mission is retained only in the private
/// per-domain Store; possession of this reference alone never authorizes launch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagedRoutingMissionRef {
    pub domain_id: DomainId,
    pub run_id: RunId,
    pub draft_id: String,
    pub plan_digest: Digest,
    pub mission_digest: Digest,
}
/// Fresh facts from the trusted controller, not a worker/advisor request.
/// Scope is compared with the stored reviewed contract. No authority or counters
/// are accepted here. Observations cannot introduce qualification trust.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingFacts {
    pub now_ms: u64,
    pub observed_at_ms: u64,
    pub expires_at_ms: u64,
    pub base: BaseSnapshot,
    pub plan_digest: Digest,
    pub permission_profile: PermissionProfile,
    pub observations: Vec<ProfileObservation>,
    pub manual_target: Option<RouteTarget>,
    pub packet_digest: Option<Digest>,
    pub advice_request_id: Option<AdviceRequestId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskRoutingState {
    WaitingDependencies,
    Ready,
    WaitingInput,
    Active,
    Succeeded,
    Failed,
    BlockedDependency,
    Cancelled,
    RecoveryRequired,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutedTaskRecord {
    pub registration: RegisteredTask,
    pub revision: u64,
    pub state: TaskRoutingState,
    pub current_attempt: Option<AttemptId>,
    pub winner: Option<VerifiedDependencyOutput>,
    pub next_ordinal: u32,
    pub last_decision: Option<RouteDecision>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutedUsage {
    Unreported,
    Known { nano_usd: u64, receipt: Digest },
    Estimated { nano_usd: u64, receipt: Digest },
    Unknown { reason: String },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingReceipts {
    pub inputs: Option<Digest>,
    pub launch_checks: Option<Digest>,
    pub process_identity: Option<Digest>,
    pub no_worker_created: Option<Digest>,
    pub quiescence: Option<Digest>,
    pub sealed_output: Option<Digest>,
    pub checks: Option<Digest>,
    pub reconciliation: Option<Digest>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutedAttemptRecord {
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub agent_id: String,
    pub ordinal: u32,
    pub predecessor: Option<AttemptId>,
    pub state: AttemptState,
    pub revision: u64,
    /// Set atomically with the private one-use launch owner. If that owner row
    /// later disappears, this independent marker keeps transitions and Stop
    /// fail closed. Historical attempts default to the legacy unmarked state.
    #[serde(default, skip_serializing_if = "is_false")]
    pub owned_launch_required: bool,
    /// Highest checker ordinal durably prepared. The counter commits with each
    /// checker owner so loss of an individual row cannot erase Stop ownership.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub owned_checker_count: u32,
    pub selected: ProfileCandidate,
    pub launch_fingerprint: Digest,
    pub decision: RouteDecision,
    pub capacity_reservation: String,
    pub input_manifest: BlobRef,
    pub handoff: Option<BlobRef>,
    pub dependencies: Vec<VerifiedDependencyOutput>,
    pub admitted_at_ms: u64,
    pub updated_at_ms: u64,
    pub cancel_epoch: u64,
    pub reserved_nano_usd: u64,
    pub usage: RoutedUsage,
    pub receipts: RoutingReceipts,
    pub failure: Option<RepairEvidence>,
    /// Domain ownership only; the separate capacity catalog must reconcile/release
    /// its own reservation using these durable receipts.
    pub ownership_released: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}
fn is_zero(value: &u32) -> bool {
    *value == 0
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmitRoutingAttempt {
    pub scope: RoutingScope,
    pub event_id: String,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub agent_id: String,
    pub facts: RoutingFacts,
    pub decision: RouteDecision,
    pub capacity_reservation: String,
    pub input_manifest: BlobRef,
    pub handoff: Option<BlobRef>,
    /// Exact untrusted JSON response bytes, retained in the journal; never part
    /// of an authoritative float-bearing manifest.
    pub advice_json: Option<String>,
    /// Optional for legacy journal rows. Instrumented attempts must link the
    /// selected pre-admission observation; this ID grants no admission right.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_event_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionRoutingAttempt {
    pub scope: RoutingScope,
    pub event_id: String,
    pub attempt_id: AttemptId,
    pub expected_attempt_revision: u64,
    pub expected_task_revision: u64,
    pub to: AttemptState,
    pub facts: RoutingFacts,
    pub receipts: RoutingReceipts,
    pub failure: Option<RepairEvidence>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRoutingBlock {
    pub scope: RoutingScope,
    pub event_id: String,
    pub task_id: TaskId,
    pub facts: RoutingFacts,
    pub decision: RouteDecision,
    pub advice_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_event_id: Option<String>,
}
/// Controller-supplied facts are checked at the current Store revision and
/// reduced to a digest before journaling. This record is not an admission or
/// launch permit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserveRoutingDecision {
    pub scope: RoutingScope,
    pub event_id: String,
    pub task_id: TaskId,
    pub expected_cancel_epoch: u64,
    pub expected_next_ordinal: u32,
    pub facts: RoutingFacts,
    pub decision: RouteDecision,
    pub advice_json: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedRoutingDecision {
    pub schema_version: u32,
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub request_digest: Digest,
    pub decision: RouteDecision,
    /// Raw classifier signal only. It is not a counterfactual route because
    /// Live also applies thresholds and eligibility gates.
    pub shadow_choice: Option<AdviceChoice>,
    /// Correlation only; this is not a paid-operation or usage receipt.
    pub advice_request_id: Option<AdviceRequestId>,
    /// Exact local response-byte identity without publishing the response.
    pub advice_digest: Option<Digest>,
    pub cancel_epoch: u64,
    pub next_ordinal: u32,
    pub observed_at_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreAdmissionStop {
    CapacityUnavailable,
    AdmissionRejected,
    Cancelled,
    Superseded,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RoutingObservationOutcome {
    Admitted {
        attempt_id: AttemptId,
    },
    NotAdmitted {
        stage: PreAdmissionStop,
        /// Correlation with controller evidence, not a Store attestation.
        controller_evidence_digest: Option<Digest>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingObservationResolution {
    pub schema_version: u32,
    pub scope: RoutingScope,
    pub observation_event_id: String,
    pub outcome: RoutingObservationOutcome,
    pub resolved_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloseUnadmittedRoutingObservation {
    pub scope: RoutingScope,
    pub observation_event_id: String,
    pub stage: PreAdmissionStop,
    pub controller_evidence_digest: Option<Digest>,
    pub now_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum RoutingControlEvent {
    Registered {
        digest: Digest,
    },
    Qualified {
        qualification: AdapterQualification,
    },
    Admitted(Box<AdmitRoutingAttempt>),
    DecisionObserved(Box<ObservedRoutingDecision>),
    DecisionResolved(Box<RoutingObservationResolution>),
    Transitioned(Box<TransitionRoutingAttempt>),
    Blocked(Box<RecordRoutingBlock>),
    Cancelled {
        expected_epoch: u64,
        now_ms: u64,
    },
    UsageSettled {
        attempt_id: AttemptId,
        expected_revision: u64,
        usage: RoutedUsage,
        now_ms: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingJournalEntry {
    pub sequence: u64,
    pub event_id: String,
    pub event: RoutingControlEvent,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingHistory {
    pub mission: RoutingMission,
    pub routing_revision: u64,
    pub cancel_epoch: u64,
    pub cancelled: bool,
    pub tasks: Vec<RoutedTaskRecord>,
    pub attempts: Vec<RoutedAttemptRecord>,
    pub events: Vec<RoutingJournalEntry>,
}

/// Private local bytes for one approved predecessor. This is an input to a
/// trusted composition step, not permission to launch or Apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedDependencyOutput {
    pub winner: VerifiedDependencyOutput,
    pub sealed_output: BlobRef,
    pub bytes: Vec<u8>,
}

/// Local Desktop projection. Deliberately excludes goals, bindings, packets,
/// advice JSON, artifacts, arbitrary failure text and the private event journal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingDisplaySummary {
    pub domain_id: String,
    pub run_id: String,
    pub routing_revision: String,
    pub mode: RoutingMode,
    pub cancelled: bool,
    pub tasks: Vec<RoutingDisplayTask>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingDisplayTask {
    pub task_id: String,
    pub state: TaskRoutingState,
    pub dependency_task_ids: Vec<String>,
    pub current_attempt_id: Option<String>,
    pub winning_attempt_id: Option<String>,
    pub last_decision: Option<RoutingDisplayDecision>,
    /// A selected route that has not become a worker attempt. This is a
    /// non-authoritative history projection, not an admission or launch permit.
    pub pre_admission: Option<RoutingDisplayPreAdmission>,
    pub attempts: Vec<RoutingDisplayAttempt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingDisplayPreAdmission {
    pub ordinal: u32,
    pub decision: RoutingDisplayDecision,
    pub outcome: RoutingDisplayPreAdmissionOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RoutingDisplayPreAdmissionOutcome {
    Pending,
    NotAdmitted { stage: PreAdmissionStop },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingDisplayDecision {
    pub selection: RoutingDisplaySelection,
    pub reason: RouteReason,
    pub advice_status: AdviceStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RoutingDisplaySelection {
    Selected { role: RoutingDisplayRole },
    WaitForCapacity { role: RoutingDisplayRole },
    Blocked { code: RouteBlocker },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingDisplayRole {
    Everyday,
    Strong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingDisplayUsageStatus {
    Unreported,
    Known,
    Estimated,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingDisplayAttempt {
    pub attempt_id: String,
    pub agent_id: String,
    pub ordinal: u32,
    pub predecessor_id: Option<String>,
    pub state: AttemptState,
    pub role: RoutingDisplayRole,
    pub decision: RoutingDisplayDecision,
    pub profile_id: String,
    pub harness_id: String,
    pub billing_mode: BillingSourceMode,
    pub handoff_referenced: bool,
    pub sealed_output_recorded: bool,
    pub checks_receipt_recorded: bool,
    pub usage_status: RoutingDisplayUsageStatus,
    pub ownership_released: bool,
    /// Wall timestamps are for order/display only, never elapsed latency.
    pub admitted_at_ms: String,
    pub updated_at_ms: String,
}

/// Whitelisted local measurement trace. Identifiers and receipt digests are
/// assignment-keyed aliases so an export does not expose private bytes or
/// stable cross-assignment correlations. The caller keeps a random key private.
/// This is evidence for an external assignment, not an acceptance or cost
/// verdict; those require independent measurement and reconciliation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingBenchmarkTrace {
    pub schema_version: u32,
    pub scope_digest: Digest,
    pub task_digest: Digest,
    /// Assignment-keyed reviewed mission identities. These do not attest the
    /// physical executable, provider account, or repository checkout.
    pub run_pins: RoutingBenchmarkRunPins,
    pub routing_revision: u64,
    pub mode: RoutingMode,
    /// Only Store decision/admission links are complete; independent quality,
    /// cost, timing and authority review remain outside this trace.
    pub decision_links_complete: bool,
    pub events: Vec<RoutingBenchmarkEvent>,
    pub attempts: Vec<RoutingBenchmarkAttempt>,
    /// Every may-send journal row for this task, including uncertain calls.
    /// These are local evidence, not provider billing receipts.
    pub advisor_requests: Vec<RoutingBenchmarkAdvisorRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingBenchmarkRunPins {
    pub task_contract_digest: Digest,
    pub snapshot_digest: Digest,
    pub profile_pair_digest: Digest,
    pub policy_digest: Digest,
    pub adapter_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingBenchmarkAdvisorRequest {
    pub request_digest: Digest,
    pub ordinal: u32,
    pub packet_digest: Digest,
    pub phase: AdvisorSendPhase,
    pub result_digest: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingBenchmarkEvent {
    pub sequence: u64,
    pub kind: RoutingBenchmarkEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RoutingBenchmarkEventKind {
    DecisionObserved {
        observation_digest: Digest,
        ordinal: u32,
        decision: RoutingDisplayDecision,
        shadow_choice: Option<AdviceChoice>,
        advice_request_digest: Option<Digest>,
        advice_digest: Option<Digest>,
    },
    DecisionResolved {
        observation_digest: Digest,
        outcome: RoutingBenchmarkResolution,
    },
    Blocked {
        decision: RoutingDisplayDecision,
    },
    Admitted {
        attempt_digest: Digest,
        observation_digest: Option<Digest>,
    },
    Transitioned {
        attempt_digest: Digest,
        state: AttemptState,
    },
    UsageSettled {
        attempt_digest: Digest,
        usage: RoutingBenchmarkUsage,
    },
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RoutingBenchmarkResolution {
    Admitted { attempt_digest: Digest },
    NotAdmitted { stage: PreAdmissionStop },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RoutingBenchmarkUsage {
    Unreported,
    Known { nano_usd: u64, receipt: Digest },
    Estimated { nano_usd: u64, receipt: Digest },
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutingBenchmarkAttempt {
    pub attempt_digest: Digest,
    pub ordinal: u32,
    pub predecessor_digest: Option<Digest>,
    pub state: AttemptState,
    pub role: RoutingDisplayRole,
    pub billing_mode: BillingSourceMode,
    pub handoff_digest: Option<Digest>,
    pub receipt_aliases: RoutingBenchmarkReceiptAliases,
    pub usage: RoutingBenchmarkUsage,
    pub ownership_released: bool,
}

/// Assignment-scoped aliases only. These are not authoritative receipt IDs.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RoutingBenchmarkReceiptAliases {
    pub inputs: Option<Digest>,
    pub launch_checks: Option<Digest>,
    pub process_identity: Option<Digest>,
    pub no_worker_created: Option<Digest>,
    pub quiescence: Option<Digest>,
    pub sealed_output: Option<Digest>,
    pub checks: Option<Digest>,
    pub reconciliation: Option<Digest>,
}

impl PytxoStore {
    /// Persist the complete trusted mission before a run row is created.
    /// An existing run ID cannot be rebound to another draft or changed bytes.
    pub fn stage_routing_mission(
        &self,
        draft_id: &str,
        mission: &RoutingMission,
    ) -> Result<StagedRoutingMissionRef> {
        validate_registration(mission)?;
        require(
            !draft_id.is_empty() && draft_id.trim() == draft_id,
            "invalid routing draft identity",
        )?;
        let reviewed = StagedRoutingMissionRef {
            domain_id: mission.authorization.domain_id.clone(),
            run_id: mission.authorization.run_id.clone(),
            draft_id: draft_id.into(),
            plan_digest: mission.authorization.plan_digest.clone(),
            mission_digest: hash(mission)?,
        };
        let body = json(mission)?;
        let tx = immediate(&self.conn)?;
        let existing: Option<(String, String, String, String, String)> = tx
            .query_row(
                "SELECT domain_id,draft_id,plan_digest,mission_digest,mission_json
                 FROM routing_mission_stages WHERE run_id=?1",
                [&reviewed.run_id.0],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(err)?;
        if let Some((domain, draft, plan_digest, mission_digest, mission_json)) = existing {
            require(
                domain == reviewed.domain_id.0
                    && draft == reviewed.draft_id
                    && plan_digest == reviewed.plan_digest.0
                    && mission_digest == reviewed.mission_digest.0
                    && mission_json == body,
                "routing stage identity content conflict",
            )?;
            return Ok(reviewed);
        }
        let run_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM runs WHERE id=?1)",
                [&reviewed.run_id.0],
                |r| r.get(0),
            )
            .map_err(err)?;
        require(!run_exists, "routing stage requires review before run")?;
        tx.execute(
            "INSERT INTO routing_mission_stages
             (run_id,domain_id,draft_id,plan_digest,mission_digest,mission_json)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                &reviewed.run_id.0,
                &reviewed.domain_id.0,
                &reviewed.draft_id,
                &reviewed.plan_digest.0,
                &reviewed.mission_digest.0,
                &body
            ],
        )
        .map_err(err)?;
        tx.commit().map_err(err)?;
        Ok(reviewed)
    }

    /// Resolve only the exact reviewed identity. A stale or missing stage is
    /// unavailable; the caller must separately check current Flow/CAS authority.
    pub fn load_staged_routing_mission(
        &self,
        reviewed: &StagedRoutingMissionRef,
    ) -> Result<RoutingMission> {
        require(
            !reviewed.domain_id.0.trim().is_empty()
                && !reviewed.run_id.0.trim().is_empty()
                && !reviewed.draft_id.is_empty()
                && reviewed.draft_id.trim() == reviewed.draft_id
                && reviewed.plan_digest.is_valid()
                && reviewed.mission_digest.is_valid(),
            "invalid routing review identity",
        )?;
        let row: Option<(String, String, String, String, String)> = self
            .conn
            .query_row(
                "SELECT domain_id,draft_id,plan_digest,mission_digest,mission_json
                 FROM routing_mission_stages WHERE run_id=?1",
                [&reviewed.run_id.0],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(err)?;
        let Some((domain, draft, plan_digest, mission_digest, body)) = row else {
            return fail("reviewed routing stage unavailable or stale");
        };
        require(
            domain == reviewed.domain_id.0
                && draft == reviewed.draft_id
                && plan_digest == reviewed.plan_digest.0
                && mission_digest == reviewed.mission_digest.0,
            "reviewed routing stage unavailable or stale",
        )?;
        let mission: RoutingMission =
            parse(&body).map_err(|_| error("invalid private routing stage"))?;
        validate_registration(&mission).map_err(|_| error("invalid private routing stage"))?;
        require(
            json(&mission)? == body
                && mission.authorization.domain_id == reviewed.domain_id
                && mission.authorization.run_id == reviewed.run_id
                && mission.authorization.plan_digest == reviewed.plan_digest
                && hash(&mission)? == reviewed.mission_digest,
            "invalid private routing stage",
        )?;
        Ok(mission)
    }

    /// Read the registered private check contract and its task projections in
    /// one snapshot. This is an integrity prerequisite, not launch authority.
    /// Historical registrations without frozen recipes remain available from
    /// `routing_history` but are intentionally rejected here.
    pub fn load_registered_mission_with_recipe_integrity(
        &self,
        scope: &RoutingScope,
    ) -> Result<RoutingMission> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        require_registered_mission_with_recipe_integrity(&tx, scope)
    }
}

/// Store-internal form for an IMMEDIATE transaction at a launch/check CAS.
/// Never substitute a mutable task projection for the registered mission.
pub(crate) fn require_registered_mission_with_recipe_integrity(
    conn: &Connection,
    scope: &RoutingScope,
) -> Result<RoutingMission> {
    let (body, stored_digest): (String, String) = conn
            .query_row(
                "SELECT registration_json,registration_digest FROM routing_missions WHERE run_id=?1 AND domain_id=?2",
                params![scope.run_id.0, scope.domain_id.0],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(err)?;
    let mission: RoutingMission = parse(&body)?;
    require(
        mission_scope(&mission) == *scope
            && json(&mission)? == body
            && hash(&mission)?.0 == stored_digest,
        "registered routing mission integrity mismatch",
    )?;
    validate_registration(&mission)?;
    require_launchable_check_recipes(&mission)?;
    let history = load_history(conn, scope, false)?
        .ok_or_else(|| error("registered routing mission unavailable"))?;
    validate_display_projection_keys(conn, scope, &history)?;
    require(
        history.tasks.len() == mission.tasks.len(),
        "registered task projection count mismatch",
    )?;
    for task in &mission.tasks {
        require(
            history
                .tasks
                .iter()
                .filter(|record| record.registration.contract.task_id == task.contract.task_id)
                .count()
                == 1
                && history
                    .tasks
                    .iter()
                    .any(|record| record.registration == *task),
            "registered task projection differs from mission",
        )?;
    }
    Ok(mission)
}

/// Final Core predicate for publishing a routed Review. The caller runs this
/// inside the same SQLite transaction that changes the Review and Run states.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedReviewSnapshot {
    schema_version: u32,
    base: BaseSnapshot,
    files: Vec<RetainedReviewFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedReviewFile {
    path: String,
    content_base64: String,
    content_digest: Digest,
    byte_length: u64,
    #[serde(default)]
    mode: Option<u32>,
}

pub(crate) fn require_routed_review_publishable(
    conn: &Connection,
    run_id: &str,
    manifest: &PreparedRunManifest,
) -> Result<()> {
    let domain_id: String = conn
        .query_row(
            "SELECT domain_id FROM routing_missions WHERE run_id=?1",
            [run_id],
            |row| row.get(0),
        )
        .map_err(err)?;
    let scope = RoutingScope {
        domain_id: pytxo_core::DomainId(domain_id),
        run_id: RunId(run_id.to_owned()),
    };
    let mission = require_registered_mission_with_recipe_integrity(conn, &scope)?;
    let history = load_history(conn, &scope, false)?
        .ok_or_else(|| error("routed Review mission disappeared"))?;
    require(
        !history.cancelled
            && history.tasks.len() == mission.tasks.len()
            && manifest.run_id == run_id,
        "routed Review mission is cancelled or incomplete",
    )?;
    require(
        manifest.package_digest == pytxo_core::prepared_manifest_digest(manifest)?,
        "routed Review package digest changed",
    )?;
    let evidence = pytxo_core::require_candidate_verification_contract(manifest)?;
    let expected_checks = mission
        .tasks
        .iter()
        .flat_map(|task| {
            task.check_recipes.iter().map(move |recipe| {
                (
                    task.contract.task_id.0.as_str(),
                    recipe.command.as_str(),
                    task.contract.permission_profile.as_str(),
                )
            })
        })
        .collect::<Vec<_>>();
    require(
        evidence.checks.len() == expected_checks.len()
            && evidence.checks.iter().zip(expected_checks).all(
                |(check, (task_id, command, profile))| {
                    check.task_id == task_id
                        && check.command == command
                        && check.effective_profile == profile
                        && check.enforcement["effective_profile"].as_str() == Some(profile)
                        && check.enforcement["execution_domain"].as_str()
                            == Some(scope.domain_id.as_str())
                },
            ),
        "routed Review candidate checks differ from registered recipes",
    )?;
    let mut expected_files = BTreeMap::new();
    for registered in &mission.tasks {
        let task_id = &registered.contract.task_id;
        let task = find_task(&history, task_id)?;
        let winner = task
            .winner
            .as_ref()
            .ok_or_else(|| error("routed Review task has no verified winner"))?;
        let attempt = find_attempt(&history, &winner.winning_attempt_id)?;
        require(
            task.state == TaskRoutingState::Succeeded
                && task.current_attempt.as_ref() == Some(&attempt.attempt_id)
                && winner.task_id == *task_id
                && attempt.scope == scope
                && attempt.task_id == *task_id
                && attempt.state == AttemptState::Passed
                && attempt.owned_launch_required
                && attempt.ownership_released
                && !matches!(attempt.usage, RoutedUsage::Unreported)
                && attempt.receipts.sealed_output.as_ref() == Some(&winner.output_digest)
                && attempt.receipts.checks.as_ref() == Some(&winner.verification_receipt_digest),
            "routed Review winner differs from owned attempt",
        )?;
        let expected_dependencies = registered
            .contract
            .dependencies
            .iter()
            .map(|dependency_id| {
                find_task(&history, dependency_id)?
                    .winner
                    .clone()
                    .ok_or_else(|| error("routed Review dependency has no winner"))
            })
            .collect::<Result<Vec<_>>>()?;
        require(
            attempt.dependencies == expected_dependencies,
            "routed Review attempt inherited different winners",
        )?;
        crate::routing_launch::require_launch_settled_for_release(conn, attempt)?;
        require_owned_pass_integrity(conn, attempt)?;
        let (_, retained_bytes) =
            crate::routing_private::retained_winner_output(conn, attempt, &winner.output_digest)?;
        let retained: RetainedReviewSnapshot =
            serde_json::from_slice(&retained_bytes).map_err(err)?;
        require(
            matches!(retained.schema_version, 1 | 2)
                && retained.base == registered.contract.base
                && manifest.base_revision == retained.base.git_revision,
            "routed Review retained output base changed",
        )?;
        for claim in &registered.contract.claim_roots {
            let file = retained
                .files
                .iter()
                .find(|file| &file.path == claim)
                .ok_or_else(|| error("routed Review has no exact claimed output"))?;
            require(
                !file.content_base64.is_empty()
                    && file.content_digest.is_valid()
                    && file.mode.is_none_or(|mode| mode <= 0o7777)
                    && (retained.schema_version != 1 || file.mode.is_none())
                    && (!cfg!(unix) || file.mode.is_some()),
                "routed Review retained output projection is invalid",
            )?;
            require(
                expected_files
                    .insert(
                        claim.clone(),
                        (
                            file.content_digest.clone(),
                            file.byte_length,
                            file.mode,
                            task_id,
                            &attempt.agent_id,
                        ),
                    )
                    .is_none(),
                "routed Review has overlapping claimed outputs",
            )?;
        }
    }
    require(
        manifest.files.len() == expected_files.len(),
        "routed Review candidate has different output count",
    )?;
    for file in &manifest.files {
        let (digest, length, mode, task_id, agent_id) = expected_files
            .remove(&file.path)
            .ok_or_else(|| error("routed Review candidate has an unclaimed output"))?;
        require(
            file.kind != PreparedRunFileKind::Delete
                && file.after_sha256.as_deref() == Some(digest.0.as_str())
                && file.after_byte_count == length
                && file.after_mode == mode
                && file.task_id == task_id.0
                && file.agent_id.as_str() == agent_id.as_str(),
            "routed Review candidate differs from retained winner bytes",
        )?;
    }
    Ok(())
}

impl PytxoStore {
    /// Store a reviewed immutable mission; calling this never activates a worker.
    pub fn register_routing_mission(&self, mission: &RoutingMission) -> Result<()> {
        validate_registration(mission)?;
        let scope = mission_scope(mission);
        let digest = hash(mission)?;
        let tx = immediate(&self.conn)?;
        if let Some(existing) = load_history(&tx, &scope, false)? {
            require(
                hash(&existing.mission)? == digest,
                "mission identity content conflict",
            )?;
            return Ok(());
        }
        let run_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM runs WHERE id=?1)",
                [&scope.run_id.0],
                |r| r.get(0),
            )
            .map_err(err)?;
        require(run_exists, "routing requires an existing run")?;
        tx.execute("INSERT INTO routing_missions(run_id,domain_id,registration_json,registration_digest,cancel_epoch,cancelled,revision) VALUES (?1,?2,?3,?4,?5,0,1)", params![scope.run_id.0,scope.domain_id.0,json(mission)?,digest.0,mission.authorization.cancel_epoch]).map_err(err)?;
        for registered in &mission.tasks {
            let record = RoutedTaskRecord {
                registration: registered.clone(),
                revision: 1,
                state: if registered.contract.dependencies.is_empty() {
                    TaskRoutingState::Ready
                } else {
                    TaskRoutingState::WaitingDependencies
                },
                current_attempt: None,
                winner: None,
                next_ordinal: 1,
                last_decision: None,
            };
            tx.execute("INSERT INTO routing_tasks(run_id,task_id,revision,record_json) VALUES (?1,?2,1,?3)", params![scope.run_id.0,registered.contract.task_id.0,json(&record)?]).map_err(err)?;
        }
        append_event(
            &tx,
            &scope,
            "registered",
            &RoutingControlEvent::Registered { digest },
            &(),
        )?;
        tx.commit().map_err(err)
    }

    /// The caller must be Core's local adapter qualification controller, never
    /// a worker/profile/advice endpoint. The complete qualification is immutable.
    pub fn register_routing_qualification(
        &self,
        scope: &RoutingScope,
        event_id: &str,
        qualification: &AdapterQualification,
    ) -> Result<()> {
        let tx = immediate(&self.conn)?;
        require_history(&tx, scope)?;
        let event = RoutingControlEvent::Qualified {
            qualification: qualification.clone(),
        };
        if replay::<()>(&tx, scope, event_id, &event)?.is_some() {
            return Ok(());
        }
        require(
            qualification.receipt_digest.is_valid() && qualification.launch_fingerprint.is_valid(),
            "invalid qualification identity",
        )?;
        let digest = qualification.digest().map_err(err)?;
        tx.execute("INSERT OR IGNORE INTO routing_qualifications(run_id,digest,qualification_json) VALUES (?1,?2,?3)",params![scope.run_id.0,digest.0,json(qualification)?]).map_err(err)?;
        append_event(&tx, scope, event_id, &event, &())?;
        tx.commit().map_err(err)
    }

    pub fn routing_snapshot(
        &self,
        scope: &RoutingScope,
        task: &TaskId,
        facts: &RoutingFacts,
    ) -> Result<RoutingSnapshot> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        snapshot(&tx, &require_history(&tx, scope)?, task, facts)
    }

    /// Read only an immutable, verified predecessor named by the consumer's
    /// reviewed task graph. The winning output must still exist under its
    /// exact owned attempt in the private domain ledger.
    pub fn read_routing_dependency_output(
        &self,
        scope: &RoutingScope,
        consumer_task: &TaskId,
        dependency_task: &TaskId,
    ) -> Result<RetainedDependencyOutput> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        let history = require_history(&tx, scope)?;
        require(!history.cancelled, "routed mission is cancelled")?;
        // A task row is a mutable lifecycle projection. Recheck it against the
        // registered mission before treating its dependency list as authority.
        let registered = require_registered_mission_with_recipe_integrity(&tx, scope)?;
        require(
            registered
                .tasks
                .iter()
                .find(|task| task.contract.task_id == *consumer_task)
                .ok_or_else(|| error("consumer is not in the reviewed task graph"))?
                .contract
                .dependencies
                .contains(dependency_task),
            "dependency is not in the reviewed task graph",
        )?;
        let producer = find_task(&history, dependency_task)?;
        let winner = producer
            .winner
            .as_ref()
            .ok_or_else(|| error("dependency has no verified winner"))?;
        let attempt = find_attempt(&history, &winner.winning_attempt_id)?;
        require(
            producer.state == TaskRoutingState::Succeeded
                && producer.current_attempt.as_ref() == Some(&winner.winning_attempt_id)
                && winner.task_id == *dependency_task
                && attempt.scope == *scope
                && attempt.task_id == *dependency_task
                && attempt.state == AttemptState::Passed
                && attempt.ownership_released
                && attempt.owned_launch_required
                && attempt.receipts.sealed_output.as_ref() == Some(&winner.output_digest)
                && attempt.receipts.checks.as_ref() == Some(&winner.verification_receipt_digest),
            "dependency winner differs from its owned attempt",
        )?;
        crate::routing_launch::require_launch_settled_for_release(&tx, attempt)?;
        require_owned_pass_integrity(&tx, attempt)?;
        let (sealed_output, bytes) =
            crate::routing_private::retained_winner_output(&tx, attempt, &winner.output_digest)?;
        Ok(RetainedDependencyOutput {
            winner: winner.clone(),
            sealed_output,
            bytes,
        })
    }

    pub fn preview_routing_decision(
        &self,
        scope: &RoutingScope,
        task: &TaskId,
        facts: &RoutingFacts,
        advice: Option<&str>,
    ) -> Result<RouteDecision> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        let h = require_history(&tx, scope)?;
        let s = snapshot(&tx, &h, task, facts)?;
        decision(&h, &s, facts, advice)
    }

    /// Digest of the exact locally observed candidate tuples used by Core.
    /// The advisor may see this identity, but cannot supply or alter it.
    pub fn routing_catalog_digest(
        &self,
        scope: &RoutingScope,
        task: &TaskId,
        facts: &RoutingFacts,
    ) -> Result<Digest> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        let history = require_history(&tx, scope)?;
        let snapshot = snapshot(&tx, &history, task, facts)?;
        Ok(build_eligible_catalog(&snapshot, &candidates(&history.mission, facts)?).catalog_digest)
    }

    /// Preserve an exact selected decision before capacity reservation. The
    /// journal contains no observation paths or raw advisor response, and
    /// admission still recomputes eligibility, policy, budgets and task CAS.
    pub fn observe_routing_decision(
        &self,
        request: &ObserveRoutingDecision,
    ) -> Result<ObservedRoutingDecision> {
        let tx = immediate(&self.conn)?;
        let h = require_history(&tx, &request.scope)?;
        let journaled_advice = if matches!(
            h.mission.policy.mode,
            RoutingMode::Shadow | RoutingMode::Live
        ) && matches!(
            request.decision.advice_status,
            AdviceStatus::ShadowRecorded | AdviceStatus::Applied | AdviceStatus::RulesFallback
        ) {
            request
                .advice_json
                .as_deref()
                .and_then(|raw| serde_json::from_str::<AdviceEnvelope>(raw).ok())
        } else {
            None
        };
        let observed = ObservedRoutingDecision {
            schema_version: 1,
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            request_digest: hash(request)?,
            decision: request.decision.clone(),
            shadow_choice: if h.mission.policy.mode == RoutingMode::Shadow {
                journaled_advice.as_ref().map(|advice| advice.choice)
            } else {
                None
            },
            advice_request_id: journaled_advice.map(|advice| advice.request_id),
            advice_digest: request
                .advice_json
                .as_ref()
                .map(|raw| Digest::of_bytes(raw.as_bytes())),
            cancel_epoch: request.expected_cancel_epoch,
            next_ordinal: request.expected_next_ordinal,
            observed_at_ms: request.facts.now_ms,
        };
        let event = RoutingControlEvent::DecisionObserved(Box::new(observed.clone()));
        if let Some(previous) = replay(&tx, &request.scope, &request.event_id, &event)? {
            return Ok(previous);
        }
        let task = find_task(&h, &request.task_id)?;
        require(
            matches!(
                task.state,
                TaskRoutingState::Ready | TaskRoutingState::WaitingInput
            ) && task.winner.is_none()
                && request.expected_cancel_epoch == h.cancel_epoch
                && request.decision.task_state_revision == task.revision,
            "selected decision is not current and ready",
        )?;
        let s = snapshot(&tx, &h, &request.task_id, &request.facts)?;
        require(
            s.next_ordinal == request.expected_next_ordinal
                && decision(&h, &s, &request.facts, request.advice_json.as_deref())?
                    == request.decision
                && matches!(request.decision.selection, RouteSelection::Selected(_)),
            "selected decision changed before observation",
        )?;
        require_jev_advice_provenance(
            &tx,
            &h,
            &request.scope,
            &request.task_id,
            &s,
            &request.decision,
            request.advice_json.as_deref(),
            true,
        )?;
        require(
            observation_state_for_task(
                &tx,
                &request.scope,
                &request.task_id,
                request.expected_next_ordinal,
            )?
            .pending
            .is_none(),
            "prior selected observation has no outcome",
        )?;
        append_event(&tx, &request.scope, &request.event_id, &event, &observed)?;
        tx.commit().map_err(err)?;
        Ok(observed)
    }

    /// Close a selected observation when no attempt was admitted. The stage
    /// is a controller claim for history; it releases no budget or capacity.
    pub fn close_unadmitted_routing_observation(
        &self,
        request: &CloseUnadmittedRoutingObservation,
    ) -> Result<RoutingObservationResolution> {
        let tx = immediate(&self.conn)?;
        let observed =
            require_observed_decision(&tx, &request.scope, &request.observation_event_id)?;
        let resolution = RoutingObservationResolution {
            schema_version: 1,
            scope: request.scope.clone(),
            observation_event_id: request.observation_event_id.clone(),
            outcome: RoutingObservationOutcome::NotAdmitted {
                stage: request.stage,
                controller_evidence_digest: request.controller_evidence_digest.clone(),
            },
            resolved_at_ms: request.now_ms,
        };
        let event = RoutingControlEvent::DecisionResolved(Box::new(resolution.clone()));
        let event_id =
            observation_resolution_event_id(&request.scope, &request.observation_event_id)?;
        if let Some(previous) = replay(&tx, &request.scope, &event_id, &event)? {
            return Ok(previous);
        }
        require(
            request.now_ms >= observed.observed_at_ms
                && request
                    .controller_evidence_digest
                    .as_ref()
                    .is_none_or(Digest::is_valid),
            "invalid pre-admission stop evidence",
        )?;
        let history = require_history(&tx, &request.scope)?;
        find_task(&history, &observed.task_id)?;
        require(
            !history.attempts.iter().any(|attempt| {
                attempt.task_id == observed.task_id && attempt.ordinal == observed.next_ordinal
            }) && (request.stage != PreAdmissionStop::Cancelled || history.cancelled),
            "pre-admission stop cannot close an admitted task",
        )?;
        append_event(&tx, &request.scope, &event_id, &event, &resolution)?;
        tx.commit().map_err(err)?;
        Ok(resolution)
    }

    pub fn record_routing_block(&self, request: &RecordRoutingBlock) -> Result<RoutedTaskRecord> {
        let tx = immediate(&self.conn)?;
        let h = require_history(&tx, &request.scope)?;
        let event = RoutingControlEvent::Blocked(Box::new(request.clone()));
        if let Some(previous) = replay(&tx, &request.scope, &request.event_id, &event)? {
            if let Some(id) = &request.observation_event_id {
                require_matching_resolution(
                    &tx,
                    &request.scope,
                    &observation_resolution(
                        &request.scope,
                        id,
                        RoutingObservationOutcome::NotAdmitted {
                            stage: PreAdmissionStop::Superseded,
                            controller_evidence_digest: None,
                        },
                        request.facts.now_ms,
                    ),
                )?;
            }
            return Ok(previous);
        }
        let mut task = find_task(&h, &request.task_id)?.clone();
        require(
            matches!(
                task.state,
                TaskRoutingState::Ready
                    | TaskRoutingState::WaitingDependencies
                    | TaskRoutingState::WaitingInput
            ),
            "task cannot be blocked from its current state",
        )?;
        let s = snapshot(&tx, &h, &request.task_id, &request.facts)?;
        require(
            observation_state_for_task(&tx, &request.scope, &request.task_id, s.next_ordinal)?
                .pending
                .as_ref()
                == request.observation_event_id.as_ref(),
            "blocked decision must resolve the pending observation",
        )?;
        require(
            decision(&h, &s, &request.facts, request.advice_json.as_deref())? == request.decision,
            "stale block decision",
        )?;
        require(
            !matches!(request.decision.selection, RouteSelection::Selected(_)),
            "selected routes require admission",
        )?;
        if let Some(id) = &request.observation_event_id {
            validate_observation_link(
                &tx,
                &request.scope,
                id,
                &s,
                &request.decision,
                request.advice_json.as_deref(),
                false,
            )?;
        }
        task.last_decision = Some(request.decision.clone());
        task.state = if matches!(
            request.decision.selection,
            RouteSelection::Blocked(RouteBlocker::FailedPrerequisite)
        ) {
            TaskRoutingState::WaitingDependencies
        } else {
            TaskRoutingState::WaitingInput
        };
        save_task(&tx, &request.scope, &mut task)?;
        append_event(&tx, &request.scope, &request.event_id, &event, &task)?;
        if let Some(id) = &request.observation_event_id {
            append_observation_resolution(
                &tx,
                &observation_resolution(
                    &request.scope,
                    id,
                    RoutingObservationOutcome::NotAdmitted {
                        stage: PreAdmissionStop::Superseded,
                        controller_evidence_digest: None,
                    },
                    request.facts.now_ms,
                ),
            )?;
        }
        tx.commit().map_err(err)?;
        Ok(task)
    }

    pub fn admit_routing_attempt(
        &self,
        request: &AdmitRoutingAttempt,
    ) -> Result<RoutedAttemptRecord> {
        let tx = immediate(&self.conn)?;
        let h = require_history(&tx, &request.scope)?;
        let event = RoutingControlEvent::Admitted(Box::new(request.clone()));
        if let Some(previous) = replay(&tx, &request.scope, &request.event_id, &event)? {
            if let Some(id) = &request.observation_event_id {
                require_matching_resolution(
                    &tx,
                    &request.scope,
                    &observation_resolution(
                        &request.scope,
                        id,
                        RoutingObservationOutcome::Admitted {
                            attempt_id: request.attempt_id.clone(),
                        },
                        request.facts.now_ms,
                    ),
                )?;
            }
            return Ok(previous);
        }
        let mut task = find_task(&h, &request.task_id)?.clone();
        let s = snapshot(&tx, &h, &request.task_id, &request.facts)?;
        let observation_state =
            observation_state_for_task(&tx, &request.scope, &request.task_id, s.next_ordinal)?;
        require(
            observation_state.pending.as_ref() == request.observation_event_id.as_ref()
                && (!observation_state.seen || request.observation_event_id.is_some()),
            "admission must resolve a fresh observed decision",
        )?;
        let computed = decision(&h, &s, &request.facts, request.advice_json.as_deref())?;
        require(
            computed == request.decision,
            "stale or altered route decision",
        )?;
        if jev_advice_requires_journal(&h, &computed, request.advice_json.as_deref()) {
            require(
                request.observation_event_id.is_some(),
                "Jev admission requires a journal-backed observation",
            )?;
        }
        require_jev_advice_provenance(
            &tx,
            &h,
            &request.scope,
            &request.task_id,
            &s,
            &computed,
            request.advice_json.as_deref(),
            computed.advice_status == AdviceStatus::Applied,
        )?;
        let RouteSelection::Selected(target) = &computed.selection else {
            return fail("route not selected");
        };
        if let Some(id) = &request.observation_event_id {
            validate_observation_link(
                &tx,
                &request.scope,
                id,
                &s,
                &computed,
                request.advice_json.as_deref(),
                true,
            )?;
        }
        require(
            s.ownership_resolved && !h.cancelled && task.winner.is_none(),
            "task ownership unavailable",
        )?;
        let reserved = task.registration.attempt_budget_nano_usd;
        if let Some(limit) = h.mission.authorization.limits.max_spend_nano_usd {
            let total = s
                .spent_and_reserved_nano_usd
                .ok_or_else(|| error("usage unknown"))?;
            require(
                total
                    .checked_add(reserved)
                    .is_some_and(|amount| amount <= limit),
                "attempt reservation exceeds remaining mission budget",
            )?;
        }
        require(
            !request.attempt_id.0.trim().is_empty()
                && !request.agent_id.trim().is_empty()
                && !request.capacity_reservation.trim().is_empty(),
            "missing attempt/agent/capacity identity",
        )?;
        let agent_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM agents WHERE id=?1)",
                [&request.agent_id],
                |r| r.get(0),
            )
            .map_err(err)?;
        require(
            !agent_exists,
            "agent identity already belongs to legacy execution",
        )?;
        validate_blob(&request.input_manifest)?;
        if let Some(handoff) = &request.handoff {
            validate_blob(handoff)?;
            let claim = handoff_manifest_claim(
                &request.scope,
                &request.task_id,
                &request.attempt_id,
                &request.capacity_reservation,
            );
            let bytes = retained_handoff_bytes(&tx, &claim, handoff)?;
            let manifest: PortableHandoffManifest = serde_json::from_slice(&bytes).map_err(err)?;
            require(
                serde_json::to_vec(&manifest).map_err(err)? == bytes,
                "handoff manifest bytes are not canonical",
            )?;
            manifest
                .validate(
                    &task.registration.contract,
                    &HandoffOrigin {
                        domain_id: request.scope.domain_id.clone(),
                        run_id: request.scope.run_id.clone(),
                        task_id: request.task_id.clone(),
                        attempt_id: request.attempt_id.clone(),
                        task_revision: task.registration.contract.revision,
                        plan_id: h.mission.authorization.plan_id.clone(),
                        plan_digest: h.mission.authorization.plan_digest.clone(),
                        authorization_revision: h.mission.authorization.revision,
                    },
                    &s.resolved_dependencies,
                )
                .map_err(err)?;
            require(
                manifest.repair_input.is_none()
                    && manifest.changes.is_empty()
                    && manifest.evidence.is_empty()
                    && manifest.notes.is_empty(),
                "routed v1 handoff has unsupported unretained payload references",
            )?;
        }
        let candidates = candidates(&h.mission, &request.facts)?;
        let selected = candidates
            .into_iter()
            .find(|c| c.target() == *target)
            .ok_or_else(|| error("selected tuple absent"))?;
        let fingerprint = launch_fingerprint(
            &selected.profile,
            &selected.binding,
            &selected.observation.executable,
            selected
                .observation
                .launch
                .as_ref()
                .ok_or_else(|| error("selected launch contract absent"))?,
        )
        .map_err(err)?;
        let attempt = RoutedAttemptRecord {
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            attempt_id: request.attempt_id.clone(),
            agent_id: request.agent_id.clone(),
            ordinal: task.next_ordinal,
            predecessor: task.current_attempt.clone(),
            state: AttemptState::Admitted,
            revision: 1,
            owned_launch_required: false,
            owned_checker_count: 0,
            selected,
            launch_fingerprint: fingerprint,
            decision: computed.clone(),
            capacity_reservation: request.capacity_reservation.clone(),
            input_manifest: request.input_manifest.clone(),
            handoff: request.handoff.clone(),
            dependencies: s.resolved_dependencies,
            admitted_at_ms: request.facts.now_ms,
            updated_at_ms: request.facts.now_ms,
            cancel_epoch: h.cancel_epoch,
            reserved_nano_usd: reserved,
            usage: RoutedUsage::Unreported,
            receipts: RoutingReceipts::default(),
            failure: None,
            ownership_released: false,
        };
        tx.execute("INSERT INTO routing_attempts(attempt_id,agent_id,capacity_reservation,run_id,task_id,ordinal,revision,record_json) VALUES (?1,?2,?3,?4,?5,?6,1,?7)",params![attempt.attempt_id.0,attempt.agent_id,attempt.capacity_reservation,request.scope.run_id.0,attempt.task_id.0,attempt.ordinal,json(&attempt)?]).map_err(err)?;
        task.current_attempt = Some(attempt.attempt_id.clone());
        task.next_ordinal = task
            .next_ordinal
            .checked_add(1)
            .ok_or_else(|| error("ordinal exhausted"))?;
        task.state = TaskRoutingState::Active;
        task.last_decision = Some(computed);
        save_task(&tx, &request.scope, &mut task)?;
        append_event(&tx, &request.scope, &request.event_id, &event, &attempt)?;
        if let Some(id) = &request.observation_event_id {
            append_observation_resolution(
                &tx,
                &observation_resolution(
                    &request.scope,
                    id,
                    RoutingObservationOutcome::Admitted {
                        attempt_id: attempt.attempt_id.clone(),
                    },
                    request.facts.now_ms,
                ),
            )?;
        }
        tx.commit().map_err(err)?;
        Ok(attempt)
    }

    /// Re-read the exact private handoff and current approved dependency winners
    /// before a controller materializes inputs or permits a native launch.
    pub fn read_routing_attempt_handoff(
        &self,
        attempt: &RoutedAttemptRecord,
    ) -> Result<Option<PortableHandoffManifest>> {
        let tx = immediate(&self.conn)?;
        let history = require_history(&tx, &attempt.scope)?;
        let current = find_attempt(&history, &attempt.attempt_id)?;
        require(current == attempt, "handoff attempt record changed")?;
        let task = find_task(&history, &attempt.task_id)?;
        let winners = task
            .registration
            .contract
            .dependencies
            .iter()
            .map(|id| {
                find_task(&history, id)?
                    .winner
                    .clone()
                    .ok_or_else(|| error("handoff dependency winner is absent"))
            })
            .collect::<Result<Vec<_>>>()?;
        require(
            winners == attempt.dependencies,
            "handoff dependency winners changed",
        )?;
        let reference = match &attempt.handoff {
            Some(reference) => reference,
            None if winners.is_empty() => return Ok(None),
            None => return fail("dependent attempt has no retained handoff"),
        };
        let claim = handoff_manifest_claim(
            &attempt.scope,
            &attempt.task_id,
            &attempt.attempt_id,
            &attempt.capacity_reservation,
        );
        let bytes = retained_handoff_bytes(&tx, &claim, reference)?;
        let manifest: PortableHandoffManifest = serde_json::from_slice(&bytes).map_err(err)?;
        require(
            serde_json::to_vec(&manifest).map_err(err)? == bytes,
            "handoff manifest bytes are not canonical",
        )?;
        manifest
            .validate(
                &task.registration.contract,
                &HandoffOrigin {
                    domain_id: attempt.scope.domain_id.clone(),
                    run_id: attempt.scope.run_id.clone(),
                    task_id: attempt.task_id.clone(),
                    attempt_id: attempt.attempt_id.clone(),
                    task_revision: task.registration.contract.revision,
                    plan_id: history.mission.authorization.plan_id.clone(),
                    plan_digest: history.mission.authorization.plan_digest.clone(),
                    authorization_revision: history.mission.authorization.revision,
                },
                &winners,
            )
            .map_err(err)?;
        require(
            manifest.repair_input.is_none()
                && manifest.changes.is_empty()
                && manifest.evidence.is_empty()
                && manifest.notes.is_empty(),
            "routed v1 handoff has unsupported unretained payload references",
        )?;
        tx.commit().map_err(err)?;
        Ok(Some(manifest))
    }

    /// Receipt digests refer to controller-retained immutable evidence, not
    /// Store-generated OS attestations. Store enforces identity, ordering,
    /// current authority and the Core transition against its durable state.
    pub fn transition_routing_attempt(
        &self,
        request: &TransitionRoutingAttempt,
    ) -> Result<RoutedAttemptRecord> {
        let tx = immediate(&self.conn)?;
        let h = require_history(&tx, &request.scope)?;
        let event = RoutingControlEvent::Transitioned(Box::new(request.clone()));
        if let Some(previous) =
            replay::<RoutedAttemptRecord>(&tx, &request.scope, &request.event_id, &event)?
        {
            if previous.state == AttemptState::Passed {
                let current = find_attempt(&h, &request.attempt_id)?;
                require(
                    current.state == AttemptState::Passed
                        && current.owned_launch_required == previous.owned_launch_required
                        && current.scope == previous.scope
                        && current.attempt_id == previous.attempt_id
                        && current.task_id == previous.task_id
                        && current.receipts.sealed_output == previous.receipts.sealed_output
                        && current.receipts.checks == previous.receipts.checks,
                    "replayed owned winner differs from current attempt",
                )?;
                require_owned_pass_integrity(&tx, current)?;
            }
            return Ok(previous);
        }
        let mut attempt = find_attempt(&h, &request.attempt_id)?.clone();
        let mut task = find_task(&h, &attempt.task_id)?.clone();
        require(
            attempt.revision == request.expected_attempt_revision
                && task.revision == request.expected_task_revision,
            "stale attempt or task revision",
        )?;
        require(
            task.current_attempt.as_ref() == Some(&attempt.attempt_id),
            "attempt is no longer current",
        )?;
        require(
            request.facts.now_ms >= attempt.updated_at_ms,
            "observation time regressed",
        )?;
        validate_receipt_phase(attempt.state, request.to, &request.receipts)?;
        merge_receipts(&mut attempt.receipts, &request.receipts)?;
        if request.to == AttemptState::Launching && attempt.owned_launch_required {
            let digest = attempt
                .receipts
                .launch_checks
                .as_ref()
                .ok_or_else(|| error("owned launch checks are absent"))?;
            crate::routing_private::require_retained_launch_checks(&tx, &attempt, digest)?;
        }
        let current_snapshot = snapshot(&tx, &h, &attempt.task_id, &request.facts)?;
        let authority_current = current_snapshot.scope_valid
            && !h.cancelled
            && attempt.cancel_epoch == h.cancel_epoch
            && request.facts.now_ms < h.mission.authorization.limits.deadline_ms
            && attempt.dependencies == current_snapshot.resolved_dependencies;
        // A cancelled/deadline-expired worker may seal/reconcile for recovery,
        // but cannot launch, start new preparation, or publish a winner.
        if matches!(
            request.to,
            AttemptState::Preparing | AttemptState::Launching | AttemptState::Passed
        ) {
            require(authority_current, "mission authority no longer current")?;
        }
        if request.to == AttemptState::Launching {
            // Another attempt may have settled above its reservation since this
            // slot was admitted. Its own reservation is still held, so the full
            // ledger must remain funded before translating to Core's admission
            // counters. Subtraction below avoids double-counting, not funding.
            if let Some(limit) = h.mission.authorization.limits.max_spend_nano_usd {
                require(
                    current_snapshot
                        .spent_and_reserved_nano_usd
                        .is_some_and(|total| total <= limit),
                    "launch reservation is underfunded or usage is unknown",
                )?;
            }
            let matching = candidates(&h.mission, &request.facts)?
                .into_iter()
                .find(|c| c.target() == attempt.selected.target())
                .ok_or_else(|| error("launch observation absent"))?;
            let mut launch_snapshot = current_snapshot;
            // This is the already admitted slot, not a new attempt admission.
            // Eligibility is rechecked under the same original ordinal/budget.
            launch_snapshot.next_ordinal = attempt.ordinal;
            launch_snapshot.previous_attempt = attempt
                .predecessor
                .as_ref()
                .and_then(|id| h.attempts.iter().find(|a| &a.attempt_id == id))
                .and_then(|a| a.failure.clone());
            launch_snapshot.ownership_resolved = true;
            launch_snapshot.active_workers = launch_snapshot.active_workers.saturating_sub(1);
            launch_snapshot.admitted_attempts = launch_snapshot.admitted_attempts.saturating_sub(1);
            launch_snapshot.spent_and_reserved_nano_usd = launch_snapshot
                .spent_and_reserved_nano_usd
                .and_then(|n| n.checked_sub(attempt.reserved_nano_usd));
            require(
                launch_fingerprint(
                    &matching.profile,
                    &matching.binding,
                    &matching.observation.executable,
                    matching
                        .observation
                        .launch
                        .as_ref()
                        .ok_or_else(|| error("current launch contract absent"))?,
                )
                .map_err(err)?
                    == attempt.launch_fingerprint,
                "launch fingerprint changed",
            )?;
            let catalog = build_eligible_catalog(&launch_snapshot, &[matching]);
            require(
                !catalog.eligible.is_empty(),
                "launch tuple no longer eligible",
            )?;
        }
        let r = &attempt.receipts;
        let evidence = AttemptTransitionEvidence {
            inputs_bound: r.inputs.is_some(),
            launch_checks_passed: r.launch_checks.is_some(),
            process_registered: r.process_identity.is_some(),
            no_worker_created: r.no_worker_created.is_some(),
            quiescent: r.quiescence.is_some(),
            output_sealed: r.sealed_output.is_some(),
            checks_passed: r.checks.is_some(),
            authority_current,
            reconciliation_complete: r.reconciliation.is_some(),
        };
        crate::routing_launch::require_owned_process_transition(
            &tx,
            &attempt,
            attempt.state,
            request.to,
            &attempt.receipts,
        )?;
        attempt
            .state
            .transition(request.to, &evidence)
            .map_err(err)?;
        if let Some(failure) = &request.failure {
            require(
                failure.attempt_id == attempt.attempt_id
                    && failure.ordinal == attempt.ordinal
                    && failure.state == attempt.state
                    && matches!(
                        attempt.state,
                        AttemptState::Failed | AttemptState::FailedNoLaunch
                    ),
                "failure evidence does not bind outcome",
            )?;
            require(
                failure
                    .actionable_evidence_digest
                    .as_ref()
                    .is_none_or(Digest::is_valid),
                "invalid actionable evidence digest",
            )?;
            attempt.failure = Some(failure.clone());
        } else if matches!(
            attempt.state,
            AttemptState::Failed | AttemptState::FailedNoLaunch
        ) {
            attempt.failure = Some(RepairEvidence {
                attempt_id: attempt.attempt_id.clone(),
                ordinal: attempt.ordinal,
                state: attempt.state,
                failure_class: AttemptFailureClass::Unknown,
                actionable_evidence_digest: None,
            });
        }
        if attempt.state == AttemptState::Passed {
            require(task.winner.is_none(), "winner already published")?;
            task.winner = Some(VerifiedDependencyOutput {
                task_id: attempt.task_id.clone(),
                winning_attempt_id: attempt.attempt_id.clone(),
                output_digest: r
                    .sealed_output
                    .clone()
                    .ok_or_else(|| error("sealed output missing"))?,
                verification_receipt_digest: r
                    .checks
                    .clone()
                    .ok_or_else(|| error("check receipt missing"))?,
            });
            task.state = TaskRoutingState::Succeeded;
        } else if h.cancelled || attempt.state == AttemptState::Cancelled {
            task.state = TaskRoutingState::Cancelled;
        } else if attempt.state == AttemptState::RecoveryRequired {
            task.state = TaskRoutingState::RecoveryRequired;
        } else if attempt.state.is_terminal() {
            let repairable = attempt.ordinal == 1
                && attempt.state == AttemptState::Failed
                && attempt.failure.as_ref().is_some_and(|f| {
                    matches!(
                        f.failure_class,
                        AttemptFailureClass::Implementation | AttemptFailureClass::Check
                    ) && f.actionable_evidence_digest.is_some()
                });
            task.state = if repairable {
                TaskRoutingState::Ready
            } else {
                TaskRoutingState::Failed
            };
        } else {
            task.state = TaskRoutingState::Active;
        }
        attempt.ownership_released = attempt.state.is_terminal();
        if attempt.ownership_released {
            crate::routing_launch::require_launch_settled_for_release(&tx, &attempt)?;
        }
        attempt.updated_at_ms = request.facts.now_ms;
        save_attempt(&tx, &mut attempt)?;
        save_task(&tx, &request.scope, &mut task)?;
        refresh_dependents(&tx, &request.scope)?;
        append_event(&tx, &request.scope, &request.event_id, &event, &attempt)?;
        tx.commit().map_err(err)?;
        Ok(attempt)
    }

    /// Durable cancellation intent only. A held process reservation stays held.
    pub fn cancel_routing_mission(
        &self,
        scope: &RoutingScope,
        event_id: &str,
        expected_epoch: u64,
        now_ms: u64,
    ) -> Result<u64> {
        let tx = immediate(&self.conn)?;
        let h = require_history(&tx, scope)?;
        let event = RoutingControlEvent::Cancelled {
            expected_epoch,
            now_ms,
        };
        if let Some(previous) = replay(&tx, scope, event_id, &event)? {
            return Ok(previous);
        }
        require(h.cancel_epoch == expected_epoch, "stale cancellation epoch")?;
        let epoch = increment(expected_epoch)?;
        let affected=tx.execute("UPDATE routing_missions SET cancelled=1,cancel_epoch=?1 WHERE run_id=?2 AND domain_id=?3 AND cancel_epoch=?4",params![epoch,scope.run_id.0,scope.domain_id.0,expected_epoch]).map_err(err)?;
        require(affected == 1, "cancellation CAS conflict")?;
        for mut task in h.tasks {
            if !matches!(
                task.state,
                TaskRoutingState::Succeeded | TaskRoutingState::Cancelled
            ) {
                task.state = TaskRoutingState::Cancelled;
                save_task(&tx, scope, &mut task)?;
            }
        }
        append_event(&tx, scope, event_id, &event, &epoch)?;
        tx.commit().map_err(err)?;
        Ok(epoch)
    }

    pub fn settle_routing_usage(
        &self,
        scope: &RoutingScope,
        event_id: &str,
        attempt_id: &AttemptId,
        expected_revision: u64,
        usage: &RoutedUsage,
        now_ms: u64,
    ) -> Result<RoutedAttemptRecord> {
        let tx = immediate(&self.conn)?;
        let h = require_history(&tx, scope)?;
        let event = RoutingControlEvent::UsageSettled {
            attempt_id: attempt_id.clone(),
            expected_revision,
            usage: usage.clone(),
            now_ms,
        };
        if let Some(previous) = replay(&tx, scope, event_id, &event)? {
            return Ok(previous);
        }
        let mut a = find_attempt(&h, attempt_id)?.clone();
        require(
            a.revision == expected_revision && a.state.is_terminal() && a.ownership_released,
            "usage settlement requires terminal ownership and current revision",
        )?;
        require(now_ms >= a.updated_at_ms, "settlement time regressed")?;
        require(
            !matches!(a.usage, RoutedUsage::Known { .. }),
            "final usage already recorded",
        )?;
        match (&a.usage, usage) {
            (
                RoutedUsage::Estimated {
                    nano_usd: prior, ..
                },
                RoutedUsage::Estimated { nano_usd: next, .. },
            ) => require(
                next >= prior,
                "estimated usage cannot release unproven spend",
            )?,
            (RoutedUsage::Unknown { .. }, RoutedUsage::Estimated { .. }) => {
                return fail("estimated usage cannot resolve unknown cost")
            }
            _ => {}
        }
        match usage {
            RoutedUsage::Known { receipt, .. } | RoutedUsage::Estimated { receipt, .. } => {
                require(receipt.is_valid(), "invalid usage receipt")?
            }
            RoutedUsage::Unknown { reason } => {
                require(!reason.trim().is_empty(), "unknown usage needs a reason")?
            }
            RoutedUsage::Unreported => return fail("cannot settle usage as unreported"),
        }
        a.usage = usage.clone();
        a.updated_at_ms = now_ms;
        save_attempt(&tx, &mut a)?;
        append_event(&tx, scope, event_id, &event, &a)?;
        tx.commit().map_err(err)?;
        Ok(a)
    }

    pub fn routing_history(&self, scope: &RoutingScope) -> Result<Option<RoutingHistory>> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        load_history(&tx, scope, true)
    }

    /// Export only typed measurements for a local, frozen benchmark assignment.
    /// The caller supplies a private, random 32-byte assignment key and must
    /// separately bind this scope/task to the assignment and
    /// supply independent acceptance, timing, authority and cost evidence.
    pub fn routing_benchmark_trace(
        &self,
        scope: &RoutingScope,
        task_id: &TaskId,
        alias_key: &[u8; 32],
    ) -> Result<Option<RoutingBenchmarkTrace>> {
        require(*alias_key != [0; 32], "benchmark alias key must be nonzero")?;
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        let Some(history) = load_history(&tx, scope, true)? else {
            return Ok(None);
        };
        validate_display_projection_keys(&tx, scope, &history)?;
        crate::routing_launch::validate_launch_ownership_implication(&tx)?;
        let stored_digest: String = tx
            .query_row(
                "SELECT registration_digest FROM routing_missions WHERE run_id=?1 AND domain_id=?2",
                params![scope.run_id.0, scope.domain_id.0],
                |row| row.get(0),
            )
            .map_err(err)?;
        require(
            mission_scope(&history.mission) == *scope && hash(&history.mission)?.0 == stored_digest,
            "benchmark registration identity mismatch",
        )?;
        validate_registration(&history.mission)?;
        let task = find_task(&history, task_id)?;
        require(
            history
                .mission
                .tasks
                .iter()
                .filter(|registered| registered.contract.task_id == *task_id)
                .count()
                == 1
                && history
                    .mission
                    .tasks
                    .iter()
                    .any(|registered| registered == &task.registration),
            "benchmark task projection differs from reviewed mission",
        )?;
        let run_pins =
            benchmark_run_pins(&history.mission, &task.registration.contract, alias_key)?;
        let advisor_requests = advisor::load_requests_for_task(&tx, scope, task_id)?;
        let advisor_by_id = advisor_requests
            .iter()
            .map(|request| (&request.request_id.0, request))
            .collect::<BTreeMap<_, _>>();
        for request in &advisor_requests {
            require(
                request.policy_digest == history.mission.policy.digest().map_err(err)?
                    && request.consent_revision == history.mission.authorization.consent_revision,
                "benchmark advisor request changed from reviewed authority",
            )?;
        }

        let mut observed = BTreeMap::<String, (&ObservedRoutingDecision, u64)>::new();
        let mut admitted = BTreeMap::<String, (&AdmitRoutingAttempt, u64)>::new();
        for entry in &history.events {
            match &entry.event {
                RoutingControlEvent::DecisionObserved(value) if value.task_id == *task_id => {
                    if matches!(
                        history.mission.policy.mode,
                        RoutingMode::Shadow | RoutingMode::Live
                    ) && history.mission.policy.advice_model == "jev-1.13.0"
                        && value.advice_digest.is_some()
                    {
                        let request = value
                            .advice_request_id
                            .as_ref()
                            .and_then(|id| advisor_by_id.get(&id.0))
                            .ok_or_else(|| error("benchmark Jev observation lacks send journal"))?;
                        require(
                            request.scope == *scope
                                && request.task_id == value.task_id
                                && request.ordinal == value.next_ordinal
                                && request.phase == AdvisorSendPhase::Completed
                                && request.result_digest == value.advice_digest
                                && request.created_at_ms <= request.updated_at_ms
                                && request.updated_at_ms <= value.observed_at_ms
                                && request.policy_digest == value.decision.policy_digest
                                && request.consent_revision
                                    == history.mission.authorization.consent_revision
                                && request.cancel_epoch == value.cancel_epoch,
                            "benchmark Jev observation differs from completed send",
                        )?;
                    }
                    require(
                        value.scope == *scope
                            && observed
                                .insert(entry.event_id.clone(), (value.as_ref(), entry.sequence))
                                .is_none(),
                        "benchmark observation identity changed",
                    )?;
                }
                RoutingControlEvent::Admitted(value) if value.task_id == *task_id => {
                    require(
                        value.scope == *scope
                            && admitted
                                .insert(
                                    value.attempt_id.0.clone(),
                                    (value.as_ref(), entry.sequence),
                                )
                                .is_none(),
                        "benchmark admission identity changed",
                    )?;
                }
                _ => {}
            }
        }
        for (attempt_id, (admission, admitted_sequence)) in &admitted {
            let attempt = find_attempt(&history, &AttemptId(attempt_id.clone()))?;
            require(
                attempt.scope == *scope
                    && attempt.task_id == admission.task_id
                    && attempt.attempt_id == admission.attempt_id
                    && attempt.agent_id == admission.agent_id
                    && attempt.decision == admission.decision
                    && attempt.capacity_reservation == admission.capacity_reservation
                    && attempt.input_manifest == admission.input_manifest
                    && attempt.handoff == admission.handoff
                    && attempt.admitted_at_ms == admission.facts.now_ms,
                "benchmark admission lacks its attempt projection",
            )?;
            if let Some(observation) = &admission.observation_event_id {
                let journaled_advice = if matches!(
                    history.mission.policy.mode,
                    RoutingMode::Shadow | RoutingMode::Live
                ) && matches!(
                    admission.decision.advice_status,
                    AdviceStatus::ShadowRecorded
                        | AdviceStatus::Applied
                        | AdviceStatus::RulesFallback
                ) {
                    admission
                        .advice_json
                        .as_deref()
                        .and_then(|raw| serde_json::from_str::<AdviceEnvelope>(raw).ok())
                } else {
                    None
                };
                require(
                    observed
                        .get(observation)
                        .is_some_and(|(observed, observed_sequence)| {
                            observed.task_id == admission.task_id
                                && observed.next_ordinal == attempt.ordinal
                                && observed.decision == admission.decision
                                && observed.cancel_epoch == attempt.cancel_epoch
                                && observed.advice_digest
                                    == admission
                                        .advice_json
                                        .as_ref()
                                        .map(|raw| Digest::of_bytes(raw.as_bytes()))
                                && observed.shadow_choice
                                    == if history.mission.policy.mode == RoutingMode::Shadow {
                                        journaled_advice.as_ref().map(|advice| advice.choice)
                                    } else {
                                        None
                                    }
                                && observed.advice_request_id
                                    == journaled_advice
                                        .as_ref()
                                        .map(|advice| advice.request_id.clone())
                                && observed_sequence < admitted_sequence
                        }),
                    "benchmark admission precedes or differs from its observation",
                )?;
            }
        }
        let mut resolved = BTreeMap::<String, (RoutingObservationOutcome, u64)>::new();
        for entry in &history.events {
            if let RoutingControlEvent::DecisionResolved(value) = &entry.event {
                // The export is one assignment/task. A sibling's resolution
                // belongs to its own keyed trace and send journal.
                if !observed.contains_key(&value.observation_event_id) {
                    continue;
                }
                require(
                    value.scope == *scope
                        && observed
                            .get(&value.observation_event_id)
                            .is_some_and(|(_, sequence)| *sequence < entry.sequence)
                        && resolved
                            .insert(
                                value.observation_event_id.clone(),
                                (value.outcome.clone(), entry.sequence),
                            )
                            .is_none(),
                    "benchmark decision resolution is missing or duplicated",
                )?;
                match &value.outcome {
                    RoutingObservationOutcome::Admitted { attempt_id } => {
                        require(
                            admitted.get(&attempt_id.0).is_some_and(
                                |(admission, admitted_sequence)| {
                                    observed.get(&value.observation_event_id).is_some_and(
                                        |(observed, _)| observed.task_id == admission.task_id,
                                    ) && admission.observation_event_id.as_ref()
                                        == Some(&value.observation_event_id)
                                        && *admitted_sequence < entry.sequence
                                },
                            ),
                            "benchmark admitted resolution differs from its attempt",
                        )?;
                    }
                    RoutingObservationOutcome::NotAdmitted { .. } => {
                        require(
                            !admitted.values().any(|(admission, _)| {
                                admission.observation_event_id.as_ref()
                                    == Some(&value.observation_event_id)
                            }),
                            "benchmark closed observation also admitted an attempt",
                        )?;
                    }
                }
            }
        }

        let attempts = history
            .attempts
            .iter()
            .filter(|attempt| attempt.task_id == *task_id)
            .map(|attempt| {
                require(
                    attempt.scope == *scope
                        && admitted
                            .get(&attempt.attempt_id.0)
                            .is_some_and(|(admission, _)| admission.task_id == *task_id),
                    "benchmark attempt lacks its admission event",
                )?;
                require(
                    attempt.ownership_released == attempt.state.is_terminal()
                        && history.mission.profiles.iter().any(|registered| {
                            registered.profile == attempt.selected.profile
                                && registered.binding == attempt.selected.binding
                        })
                        && attempt.selected.profile.digest().map_err(err)?
                            == attempt.selected.observation.profile_digest
                        && attempt.selected.binding.digest().map_err(err)?
                            == attempt.selected.observation.binding_digest
                        && launch_fingerprint(
                            &attempt.selected.profile,
                            &attempt.selected.binding,
                            &attempt.selected.observation.executable,
                            attempt
                                .selected
                                .observation
                                .launch
                                .as_ref()
                                .ok_or_else(|| error("admitted launch contract absent"))?,
                        )
                        .map_err(err)?
                            == attempt.launch_fingerprint
                        && matches!(&attempt.decision.selection, RouteSelection::Selected(target) if target == &attempt.selected.target()),
                    "benchmark attempt selected profile differs from admitted decision",
                )?;
                let role = display_role(&history.mission.policy, &attempt.selected.target())?;
                Ok(RoutingBenchmarkAttempt {
                    attempt_digest: benchmark_alias(alias_key, "attempt", &attempt.attempt_id.0),
                    ordinal: attempt.ordinal,
                    predecessor_digest: attempt
                        .predecessor
                        .as_ref()
                        .map(|id| benchmark_alias(alias_key, "attempt", &id.0)),
                    state: attempt.state,
                    role,
                    billing_mode: attempt.selected.binding.billing_mode,
                    handoff_digest: attempt
                        .handoff
                        .as_ref()
                        .map(|blob| benchmark_alias(alias_key, "digest", &blob.digest.0)),
                    receipt_aliases: benchmark_receipts(alias_key, &attempt.receipts),
                    usage: benchmark_usage(alias_key, &attempt.usage),
                    ownership_released: attempt.ownership_released,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let mut events = Vec::new();
        let mut recorded_decision = false;
        for entry in &history.events {
            let kind = match &entry.event {
                RoutingControlEvent::DecisionObserved(value) if value.task_id == *task_id => {
                    recorded_decision = true;
                    Some(RoutingBenchmarkEventKind::DecisionObserved {
                        observation_digest: benchmark_alias(alias_key, "event", &entry.event_id),
                        ordinal: value.next_ordinal,
                        decision: display_decision(&history.mission.policy, &value.decision)?,
                        shadow_choice: value.shadow_choice,
                        advice_request_digest: value
                            .advice_request_id
                            .as_ref()
                            .map(|id| benchmark_alias(alias_key, "request", &id.0)),
                        advice_digest: value
                            .advice_digest
                            .as_ref()
                            .map(|digest| benchmark_alias(alias_key, "digest", &digest.0)),
                    })
                }
                RoutingControlEvent::DecisionResolved(value)
                    if observed
                        .get(&value.observation_event_id)
                        .is_some_and(|(observed, _)| observed.task_id == *task_id) =>
                {
                    let outcome = match &value.outcome {
                        RoutingObservationOutcome::Admitted { attempt_id } => {
                            RoutingBenchmarkResolution::Admitted {
                                attempt_digest: benchmark_alias(
                                    alias_key,
                                    "attempt",
                                    &attempt_id.0,
                                ),
                            }
                        }
                        RoutingObservationOutcome::NotAdmitted { stage, .. } => {
                            RoutingBenchmarkResolution::NotAdmitted { stage: *stage }
                        }
                    };
                    Some(RoutingBenchmarkEventKind::DecisionResolved {
                        observation_digest: benchmark_alias(
                            alias_key,
                            "event",
                            &value.observation_event_id,
                        ),
                        outcome,
                    })
                }
                RoutingControlEvent::Blocked(value) if value.task_id == *task_id => {
                    recorded_decision = true;
                    Some(RoutingBenchmarkEventKind::Blocked {
                        decision: display_decision(&history.mission.policy, &value.decision)?,
                    })
                }
                RoutingControlEvent::Admitted(value) if value.task_id == *task_id => {
                    Some(RoutingBenchmarkEventKind::Admitted {
                        attempt_digest: benchmark_alias(alias_key, "attempt", &value.attempt_id.0),
                        observation_digest: value
                            .observation_event_id
                            .as_ref()
                            .map(|id| benchmark_alias(alias_key, "event", id)),
                    })
                }
                RoutingControlEvent::Transitioned(value)
                    if find_attempt(&history, &value.attempt_id)?.task_id == *task_id =>
                {
                    Some(RoutingBenchmarkEventKind::Transitioned {
                        attempt_digest: benchmark_alias(alias_key, "attempt", &value.attempt_id.0),
                        state: value.to,
                    })
                }
                RoutingControlEvent::UsageSettled {
                    attempt_id, usage, ..
                } if find_attempt(&history, attempt_id)?.task_id == *task_id => {
                    Some(RoutingBenchmarkEventKind::UsageSettled {
                        attempt_digest: benchmark_alias(alias_key, "attempt", &attempt_id.0),
                        usage: benchmark_usage(alias_key, usage),
                    })
                }
                RoutingControlEvent::Cancelled { .. } => Some(RoutingBenchmarkEventKind::Cancelled),
                _ => None,
            };
            if let Some(kind) = kind {
                events.push(RoutingBenchmarkEvent {
                    sequence: entry.sequence,
                    kind,
                });
            }
        }
        let recorded_attempts_linked = admitted
            .iter()
            .filter(|(_, (admission, _))| admission.task_id == *task_id)
            .all(|(attempt_id, (admission, _))| {
                admission.observation_event_id.as_ref().is_some_and(|id| {
                    observed
                        .get(id)
                        .is_some_and(|(observed, _)| observed.task_id == *task_id)
                        && resolved.get(id).is_some_and(|(outcome, _)| {
                            matches!(outcome, RoutingObservationOutcome::Admitted { attempt_id: resolved_attempt } if resolved_attempt.0 == *attempt_id)
                        })
                })
            });
        let decisions_resolved = observed
            .iter()
            .filter(|(_, (observed, _))| observed.task_id == *task_id)
            .all(|(id, _)| resolved.contains_key(id));
        Ok(Some(RoutingBenchmarkTrace {
            schema_version: 3,
            scope_digest: benchmark_alias(alias_key, "digest", &hash(scope)?.0),
            task_digest: benchmark_alias(alias_key, "digest", &hash(task_id)?.0),
            run_pins,
            routing_revision: history.routing_revision,
            mode: history.mission.policy.mode,
            decision_links_complete: recorded_decision
                && recorded_attempts_linked
                && decisions_resolved,
            events,
            attempts,
            advisor_requests: advisor_requests
                .iter()
                .map(|request| RoutingBenchmarkAdvisorRequest {
                    request_digest: benchmark_alias(alias_key, "request", &request.request_id.0),
                    ordinal: request.ordinal,
                    packet_digest: benchmark_alias(alias_key, "digest", &request.packet_digest.0),
                    phase: request.phase,
                    result_digest: request
                        .result_digest
                        .as_ref()
                        .map(|digest| benchmark_alias(alias_key, "digest", &digest.0)),
                })
                .collect(),
        }))
    }

    /// Cheap, private-payload-free change identity for Desktop snapshots.
    /// A run ID present under another domain is an error, not a legacy run.
    pub fn routing_revision(&self, scope: &RoutingScope) -> Result<Option<u64>> {
        let row: Option<(String, u64)> = self
            .conn
            .query_row(
                "SELECT domain_id,revision FROM routing_missions WHERE run_id=?1",
                [&scope.run_id.0],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(err)?;
        match row {
            None => Ok(None),
            Some((domain_id, revision)) => {
                require(domain_id == scope.domain_id.0, "execution domain mismatch")?;
                Ok(Some(revision))
            }
        }
    }

    /// A coherent, scoped read for Work/History. No raw mission or journal type
    /// crosses this API boundary; malformed private projections fail closed.
    pub fn routing_display_summary(
        &self,
        scope: &RoutingScope,
    ) -> Result<Option<RoutingDisplaySummary>> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        let Some(history) = load_history(&tx, scope, false)? else {
            return Ok(None);
        };
        crate::routing_launch::validate_launch_ownership_implication(&tx)?;
        validate_display_projection_keys(&tx, scope, &history)?;
        let stored_digest: String = tx
            .query_row(
                "SELECT registration_digest FROM routing_missions WHERE run_id=?1 AND domain_id=?2",
                params![scope.run_id.0, scope.domain_id.0],
                |row| row.get(0),
            )
            .map_err(err)?;
        require(
            mission_scope(&history.mission) == *scope && hash(&history.mission)?.0 == stored_digest,
            "routing display registration identity mismatch",
        )?;
        validate_registration(&history.mission)?;
        require(
            history.tasks.len() == history.mission.tasks.len(),
            "routing display task projection count mismatch",
        )?;

        let mut tasks = Vec::with_capacity(history.tasks.len());
        let mut projected_attempts = 0_usize;
        for task in &history.tasks {
            let task_id = &task.registration.contract.task_id;
            require(
                history
                    .mission
                    .tasks
                    .iter()
                    .filter(|registered| registered.contract.task_id == *task_id)
                    .count()
                    == 1
                    && history
                        .mission
                        .tasks
                        .iter()
                        .any(|registered| registered == &task.registration),
                "routing display task projection differs from registration",
            )?;
            let mut attempts = Vec::new();
            for attempt in history
                .attempts
                .iter()
                .filter(|attempt| attempt.task_id == *task_id)
            {
                require(
                    attempt.scope == *scope
                        && attempt.ownership_released == attempt.state.is_terminal()
                        && history.mission.profiles.iter().any(|registered| {
                            registered.profile == attempt.selected.profile
                                && registered.binding == attempt.selected.binding
                        }),
                    "routing display attempt scope or profile mismatch",
                )?;
                let role = display_role(&history.mission.policy, &attempt.selected.target())?;
                require(
                    matches!(&attempt.decision.selection, RouteSelection::Selected(target) if target == &attempt.selected.target()),
                    "routing display attempt decision differs from selected profile",
                )?;
                let attempt_decision =
                    display_decision(&history.mission.policy, &attempt.decision)?;
                let usage_status = match &attempt.usage {
                    RoutedUsage::Unreported => RoutingDisplayUsageStatus::Unreported,
                    RoutedUsage::Known { .. } => RoutingDisplayUsageStatus::Known,
                    RoutedUsage::Estimated { .. } => RoutingDisplayUsageStatus::Estimated,
                    RoutedUsage::Unknown { .. } => RoutingDisplayUsageStatus::Unknown,
                };
                attempts.push(RoutingDisplayAttempt {
                    attempt_id: attempt.attempt_id.0.clone(),
                    agent_id: attempt.agent_id.clone(),
                    ordinal: attempt.ordinal,
                    predecessor_id: attempt.predecessor.as_ref().map(|id| id.0.clone()),
                    state: attempt.state,
                    role,
                    decision: attempt_decision,
                    profile_id: attempt.selected.profile.id.0.clone(),
                    harness_id: attempt.selected.profile.harness_id.clone(),
                    billing_mode: attempt.selected.binding.billing_mode,
                    handoff_referenced: attempt.handoff.is_some(),
                    sealed_output_recorded: attempt.receipts.sealed_output.is_some(),
                    checks_receipt_recorded: attempt.receipts.checks.is_some(),
                    usage_status,
                    ownership_released: attempt.ownership_released,
                    admitted_at_ms: attempt.admitted_at_ms.to_string(),
                    updated_at_ms: attempt.updated_at_ms.to_string(),
                });
                projected_attempts += 1;
            }
            if let Some(current) = &task.current_attempt {
                require(
                    attempts
                        .iter()
                        .any(|attempt| attempt.attempt_id == current.0),
                    "routing display current attempt missing",
                )?;
            }
            if let Some(winner) = &task.winner {
                require(
                    task.state == TaskRoutingState::Succeeded
                        && task.current_attempt.as_ref() == Some(&winner.winning_attempt_id)
                        && winner.task_id == *task_id
                        && winner.output_digest.is_valid()
                        && winner.verification_receipt_digest.is_valid()
                        && history.attempts.iter().any(|attempt| {
                            attempt.task_id == *task_id
                                && attempt.attempt_id == winner.winning_attempt_id
                                && attempt.state == AttemptState::Passed
                                && attempt.receipts.sealed_output.as_ref()
                                    == Some(&winner.output_digest)
                                && attempt.receipts.checks.as_ref()
                                    == Some(&winner.verification_receipt_digest)
                        }),
                    "routing display winner projection mismatch",
                )?;
            } else {
                require(
                    task.state != TaskRoutingState::Succeeded,
                    "routing display succeeded task has no winner",
                )?;
            }
            require(
                history
                    .attempts
                    .iter()
                    .filter(|attempt| {
                        attempt.task_id == *task_id && attempt.state == AttemptState::Passed
                    })
                    .all(|attempt| {
                        task.winner.as_ref().is_some_and(|winner| {
                            winner.winning_attempt_id == attempt.attempt_id
                                && attempt.receipts.sealed_output.as_ref()
                                    == Some(&winner.output_digest)
                                && attempt.receipts.checks.as_ref()
                                    == Some(&winner.verification_receipt_digest)
                        })
                    }),
                "routing display passed attempt has no matching winner and receipts",
            )?;
            let last_decision = task
                .last_decision
                .as_ref()
                .map(|decision| display_decision(&history.mission.policy, decision))
                .transpose()?;
            let pre_admission = display_pre_admission_observation(
                &tx,
                scope,
                task_id,
                &history.mission.policy,
                &history.attempts,
            )?;
            tasks.push(RoutingDisplayTask {
                task_id: task_id.0.clone(),
                state: task.state.clone(),
                dependency_task_ids: task
                    .registration
                    .contract
                    .dependencies
                    .iter()
                    .map(|id| id.0.clone())
                    .collect(),
                current_attempt_id: task.current_attempt.as_ref().map(|id| id.0.clone()),
                winning_attempt_id: task
                    .winner
                    .as_ref()
                    .map(|winner| winner.winning_attempt_id.0.clone()),
                last_decision,
                pre_admission,
                attempts,
            });
        }
        require(
            projected_attempts == history.attempts.len(),
            "routing display attempt task missing",
        )?;
        Ok(Some(RoutingDisplaySummary {
            domain_id: scope.domain_id.0.clone(),
            run_id: scope.run_id.0.clone(),
            routing_revision: history.routing_revision.to_string(),
            mode: history.mission.policy.mode,
            cancelled: history.cancelled,
            tasks,
        }))
    }

    /// Scope-only recovery ownership for uncancelled registered missions.
    /// Every row is checked in one read snapshot before a completed winner is
    /// exempted; private mission and task bytes never leave the Store API.
    pub fn unreconciled_registered_routing_scopes(&self) -> Result<Vec<RoutingScope>> {
        let tx = self.conn.unchecked_transaction().map_err(err)?;
        let registrations = {
            let mut stmt = tx
                .prepare(
                    "SELECT m.run_id,m.domain_id,m.registration_json,m.registration_digest,m.cancel_epoch,r.status
                     FROM routing_missions m LEFT JOIN runs r ON r.id=m.run_id
                     WHERE m.cancelled=0 ORDER BY m.run_id",
                )
                .map_err(err)?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, u64>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                })
                .map_err(err)?;
            rows.collect::<std::result::Result<Vec<_>, _>>()
                .map_err(err)?
        };
        let mut scopes = Vec::new();
        for (run_id, domain_id, registration_json, registration_digest, cancel_epoch, status) in
            registrations
        {
            let scope = RoutingScope {
                domain_id: DomainId(domain_id),
                run_id: RunId(run_id),
            };
            let mission: RoutingMission = parse(&registration_json)?;
            require(
                mission_scope(&mission) == scope
                    && json(&mission)? == registration_json
                    && hash(&mission)?.0 == registration_digest
                    && cancel_epoch == mission.authorization.cancel_epoch,
                "registered routing mission identity or digest mismatch",
            )?;
            validate_registration(&mission)?;
            let status = status.ok_or_else(|| error("registered run status missing"))?;
            require(
                matches!(
                    status.as_str(),
                    "starting"
                        | "running"
                        | "completed"
                        | "failed"
                        | "failed_startup"
                        | "cancelled"
                ),
                "registered run has invalid status",
            )?;

            let tasks = {
                let mut stmt = tx
                    .prepare("SELECT task_id,revision,record_json FROM routing_tasks WHERE run_id=?1 ORDER BY task_id")
                    .map_err(err)?;
                let rows = stmt
                    .query_map([&scope.run_id.0], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, u64>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    })
                    .map_err(err)?;
                let mut tasks = Vec::new();
                for row in rows {
                    let (key, revision, body) = row.map_err(err)?;
                    let task: RoutedTaskRecord = parse(&body)?;
                    require(
                        key == task.registration.contract.task_id.0
                            && revision == task.revision
                            && json(&task)? == body
                            && mission
                                .tasks
                                .iter()
                                .any(|registered| registered == &task.registration),
                        "registered routing task projection mismatch",
                    )?;
                    tasks.push(task);
                }
                tasks
            };
            require(
                tasks.len() == mission.tasks.len(),
                "registered routing task projection count mismatch",
            )?;

            let attempts = {
                let mut stmt = tx
                    .prepare(
                        "SELECT attempt_id,agent_id,capacity_reservation,task_id,ordinal,revision,record_json
                         FROM routing_attempts WHERE run_id=?1 ORDER BY task_id,ordinal",
                    )
                    .map_err(err)?;
                let rows = stmt
                    .query_map([&scope.run_id.0], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, u32>(4)?,
                            row.get::<_, u64>(5)?,
                            row.get::<_, String>(6)?,
                        ))
                    })
                    .map_err(err)?;
                let mut attempts = Vec::new();
                for row in rows {
                    let (
                        attempt_id,
                        agent_id,
                        capacity_reservation,
                        task_id,
                        ordinal,
                        revision,
                        body,
                    ) = row.map_err(err)?;
                    let attempt: RoutedAttemptRecord = parse(&body)?;
                    require(
                        attempt.scope == scope
                            && attempt.attempt_id.0 == attempt_id
                            && attempt.agent_id == agent_id
                            && attempt.capacity_reservation == capacity_reservation
                            && attempt.task_id.0 == task_id
                            && attempt.ordinal == ordinal
                            && attempt.revision == revision
                            && json(&attempt)? == body
                            && attempt.ownership_released == attempt.state.is_terminal()
                            && mission
                                .tasks
                                .iter()
                                .any(|task| task.contract.task_id == attempt.task_id),
                        "registered routing attempt projection mismatch",
                    )?;
                    attempts.push(attempt);
                }
                attempts
            };

            for task in &tasks {
                match (&task.state, &task.winner) {
                    (TaskRoutingState::Succeeded, Some(winner)) => {
                        let matching_attempts: Vec<_> = attempts
                            .iter()
                            .filter(|attempt| attempt.attempt_id == winner.winning_attempt_id)
                            .collect();
                        require(
                            winner.task_id == task.registration.contract.task_id
                                && winner.output_digest.is_valid()
                                && winner.verification_receipt_digest.is_valid()
                                && matching_attempts.len() == 1
                                && matching_attempts[0].task_id == winner.task_id
                                && matching_attempts[0].state == AttemptState::Passed
                                && matching_attempts[0].ownership_released
                                && matching_attempts[0].receipts.sealed_output.as_ref()
                                    == Some(&winner.output_digest)
                                && matching_attempts[0].receipts.checks.as_ref()
                                    == Some(&winner.verification_receipt_digest),
                            "registered routing winner projection mismatch",
                        )?;
                    }
                    (TaskRoutingState::Succeeded, None) | (_, Some(_)) => {
                        return fail("registered routing winner projection mismatch");
                    }
                    _ => {}
                }
            }
            let healthy_completed = status == "completed"
                && tasks
                    .iter()
                    .all(|task| task.state == TaskRoutingState::Succeeded)
                && attempts.iter().all(|attempt| attempt.ownership_released);
            let discarded_passed_review = if status == "failed"
                && tasks
                    .iter()
                    .all(|task| task.state == TaskRoutingState::Succeeded)
                && attempts.iter().all(|attempt| {
                    attempt.ownership_released
                        && matches!(attempt.state, AttemptState::Passed | AttemptState::Failed)
                }) {
                struct ReviewOutcome {
                    apply_status: String,
                    last_error: Option<String>,
                    prepared: Option<String>,
                    digest: Option<String>,
                    applied: Option<String>,
                }
                let contract: Option<ReviewOutcome> = tx
                    .query_row(
                        "SELECT apply_status,last_apply_error_json,prepared_manifest_json,
                                prepared_digest,apply_manifest_json
                         FROM run_contracts WHERE run_id=?1",
                        [&scope.run_id.0],
                        |row| {
                            Ok(ReviewOutcome {
                                apply_status: row.get(0)?,
                                last_error: row.get(1)?,
                                prepared: row.get(2)?,
                                digest: row.get(3)?,
                                applied: row.get(4)?,
                            })
                        },
                    )
                    .optional()
                    .map_err(err)?;
                if let Some(contract) = contract {
                    contract.apply_status == "discarded"
                        && contract.prepared.is_none()
                        && contract.digest.is_none()
                        && contract.applied.is_none()
                        && contract
                            .last_error
                            .as_deref()
                            .map(parse::<pytxo_core::RunApplyError>)
                            .transpose()?
                            .is_some_and(|error| error.code == "routed_review_failed")
                } else {
                    false
                }
            } else {
                false
            };
            if !healthy_completed && !discarded_passed_review {
                scopes.push(scope);
            }
        }
        Ok(scopes)
    }

    /// Includes uncertainty even if the legacy run label is already terminal.
    pub fn unresolved_routing_attempts(&self) -> Result<Vec<RoutedAttemptRecord>> {
        // Claim/Stop use this scan. A launch owner must never be invisible when
        // an attempt row is terminal or its projection has drifted.
        crate::routing_launch::validate_launch_ownership_implication(&self.conn)?;
        let mut stmt = self
            .conn
            .prepare("SELECT record_json FROM routing_attempts ORDER BY run_id,task_id,ordinal")
            .map_err(err)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(err)?;
        let mut result = Vec::new();
        for row in rows {
            let record: RoutedAttemptRecord = parse(&row.map_err(err)?)?;
            if !record.ownership_released {
                result.push(record);
            }
        }
        Ok(result)
    }
}

fn immediate(conn: &Connection) -> Result<Transaction<'_>> {
    Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(err)
}
fn err(e: impl std::fmt::Display) -> PytxoError {
    error(&e.to_string())
}
fn error(message: &str) -> PytxoError {
    PytxoError::Store(message.into())
}
fn fail<T>(message: &str) -> Result<T> {
    Err(error(message))
}
fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        fail(message)
    }
}
fn json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(err)
}
fn parse<T: DeserializeOwned>(value: &str) -> Result<T> {
    serde_json::from_str(value).map_err(err)
}
fn hash<T: Serialize>(value: &T) -> Result<Digest> {
    canonical_digest(value, 1).map_err(err)
}
fn increment(value: u64) -> Result<u64> {
    value
        .checked_add(1)
        .filter(|n| *n <= i64::MAX as u64)
        .ok_or_else(|| error("routing revision exhausted"))
}
fn mission_scope(m: &RoutingMission) -> RoutingScope {
    RoutingScope {
        domain_id: m.authorization.domain_id.clone(),
        run_id: m.authorization.run_id.clone(),
    }
}

fn display_role(policy: &RoutingPolicy, target: &RouteTarget) -> Result<RoutingDisplayRole> {
    if target == &policy.everyday {
        Ok(RoutingDisplayRole::Everyday)
    } else if target == &policy.strong {
        Ok(RoutingDisplayRole::Strong)
    } else {
        fail("routing display target outside reviewed roles")
    }
}

fn display_decision(
    policy: &RoutingPolicy,
    decision: &RouteDecision,
) -> Result<RoutingDisplayDecision> {
    let selection = match &decision.selection {
        RouteSelection::Selected(target) => RoutingDisplaySelection::Selected {
            role: display_role(policy, target)?,
        },
        RouteSelection::WaitForCapacity { target, .. } => {
            RoutingDisplaySelection::WaitForCapacity {
                role: display_role(policy, target)?,
            }
        }
        RouteSelection::Blocked(code) => RoutingDisplaySelection::Blocked { code: code.clone() },
    };
    Ok(RoutingDisplayDecision {
        selection,
        reason: decision.reason.clone(),
        advice_status: decision.advice_status,
    })
}
fn validate_blob(blob: &BlobRef) -> Result<()> {
    require(blob.digest.is_valid(), "invalid immutable blob reference")
}

fn validate_frozen_recipe(recipe: &FrozenCheckRecipeV1) -> Result<()> {
    let executor = &recipe.executor;
    let shell_path = executor.shell.path.as_bytes();
    let absolute_shell = match executor.platform {
        CheckPlatform::Windows => {
            shell_path.len() >= 3
                && shell_path[0].is_ascii_alphabetic()
                && shell_path[1] == b':'
                && matches!(shell_path[2], b'\\' | b'/')
                && executor.shell_args == ["/D", "/C"]
        }
        CheckPlatform::Posix => {
            executor.shell.path.starts_with('/') && executor.shell_args == ["-c"]
        }
    };
    require(
        recipe.schema_version == 1
            && recipe.ordinal > 0
            && !recipe.id.0.trim().is_empty()
            && !recipe.command.is_empty()
            && recipe.command == recipe.command.trim()
            && recipe.command.len() <= 4096
            && !recipe.command.chars().any(char::is_control)
            && executor.policy_version == 1
            && absolute_shell
            && executor.shell.path == executor.shell.path.trim()
            && !executor.shell.path.chars().any(char::is_control)
            && !executor.shell.version.trim().is_empty()
            && executor.shell.digest.is_valid()
            && executor.cwd_kind == CheckCwdKind::FreshSealedVerificationView
            && executor.permission_profile == PermissionProfile::Orbit
            && executor.stdin_closed
            && executor.environment_policy_version == 1
            && executor.network_policy_version == 1
            && executor.timeout_ms == 120_000
            && executor.max_stdout_bytes == 262_144
            && executor.max_stderr_bytes == 262_144,
        "unsupported or invalid frozen check recipe",
    )
}

pub fn validate_registered_check_recipes(task: &RegisteredTask) -> Result<()> {
    if task.check_recipes.is_empty() {
        // Pre-extension registrations have Core check references but no
        // executable recipe. Their history is valid; launch is a separate gate.
        return Ok(());
    }
    require(
        task.check_recipes.len() == task.contract.checks.len(),
        "frozen check count differs from task contract",
    )?;
    let mut commands = BTreeSet::new();
    for (index, (recipe, approved)) in task
        .check_recipes
        .iter()
        .zip(&task.contract.checks)
        .enumerate()
    {
        require(
            recipe.ordinal == u32::try_from(index + 1).map_err(err)?
                && recipe.id.0 == format!("{}:verify:{:04}", task.contract.task_id.0, index + 1)
                && recipe.reference()? == *approved
                && commands.insert(recipe.command.as_str()),
            "frozen check differs from reviewed task contract",
        )?;
    }
    Ok(())
}

pub fn require_launchable_check_recipes(mission: &RoutingMission) -> Result<()> {
    require(!mission.tasks.is_empty(), "routing mission has no tasks")?;
    for task in &mission.tasks {
        require(
            !task.check_recipes.is_empty(),
            "routed check recipes are unavailable for launch",
        )?;
        validate_registered_check_recipes(task)?;
    }
    Ok(())
}

fn validate_registration(m: &RoutingMission) -> Result<()> {
    let a = &m.authorization;
    require(
        a.schema_version == 1
            && a.revision > 0
            && !a.domain_id.0.trim().is_empty()
            && !a.run_id.0.trim().is_empty()
            && !a.plan_id.0.trim().is_empty()
            && a.plan_digest.is_valid(),
        "invalid mission identity",
    )?;
    require(
        m.policy.digest().map_err(err)? == a.policy_digest,
        "policy differs from authorization",
    )?;
    require(
        !m.tasks.is_empty() && !m.profiles.is_empty(),
        "empty routing mission",
    )?;
    require(
        m.profiles.len() == 2
            && m.policy.everyday != m.policy.strong
            && m.policy.everyday.profile_id != m.policy.strong.profile_id,
        "routing v1 requires two distinct profile roles",
    )?;
    let mut ids = BTreeSet::new();
    let mut digests = BTreeSet::new();
    for t in &m.tasks {
        validate_registered_check_recipes(t)?;
        require(
            ids.insert(t.contract.task_id.0.clone()),
            "duplicate task identity",
        )?;
        require(
            t.contract.plan_digest == a.plan_digest
                && t.contract.permission_profile == a.permission_profile,
            "task scope differs from mission",
        )?;
        require(
            t.contract
                .required_target
                .as_ref()
                .is_none_or(|target| target == &m.policy.everyday || target == &m.policy.strong),
            "required target outside reviewed routing roles",
        )?;
        require(
            a.limits.max_spend_nano_usd.is_none_or(|limit| {
                t.attempt_budget_nano_usd > 0 && t.attempt_budget_nano_usd <= limit
            }),
            "invalid approved attempt reservation",
        )?;
        digests.insert(t.contract.digest().map_err(err)?);
    }
    require(
        digests == a.allowed_task_digests,
        "reviewed task set mismatch",
    )?;
    let mut resolved = BTreeSet::new();
    while resolved.len() < m.tasks.len() {
        let previous = resolved.len();
        for t in &m.tasks {
            let mut unique = BTreeSet::new();
            require(
                t.contract.dependencies.iter().all(|id| {
                    ids.contains(&id.0) && unique.insert(&id.0) && id != &t.contract.task_id
                }),
                "invalid task dependency",
            )?;
            if t.contract
                .dependencies
                .iter()
                .all(|id| resolved.contains(&id.0))
            {
                resolved.insert(t.contract.task_id.0.clone());
            }
        }
        require(resolved.len() > previous, "cyclic task dependencies")?;
    }
    let mut targets = BTreeSet::new();
    require(
        m.profiles.len() == a.allowed_profiles.len(),
        "approved profile set mismatch",
    )?;
    for p in &m.profiles {
        let target = RouteTarget {
            profile_id: p.profile.id.clone(),
            binding_id: p.binding.id.clone(),
        };
        let pd = p.profile.digest().map_err(err)?;
        let bd = p.binding.digest().map_err(err)?;
        require(
            targets.insert(target.clone())
                && p.binding.profile_digest == pd
                && a.allowed_profiles
                    .iter()
                    .filter(|approved| {
                        approved.target == target
                            && approved.profile_digest == pd
                            && approved.binding_digest == bd
                    })
                    .count()
                    == 1,
            "profile tuple not exactly approved",
        )?;
    }
    require(
        targets.contains(&m.policy.everyday) && targets.contains(&m.policy.strong),
        "routing role target not registered",
    )?;
    hash(m)?;
    Ok(())
}

fn load_history(
    conn: &Connection,
    scope: &RoutingScope,
    include_journal: bool,
) -> Result<Option<RoutingHistory>> {
    let row:Option<(String,String,u64,bool,u64)>=conn.query_row("SELECT domain_id,registration_json,cancel_epoch,cancelled,revision FROM routing_missions WHERE run_id=?1",[&scope.run_id.0],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(err)?;
    let Some((domain, registration, cancel_epoch, cancelled, routing_revision)) = row else {
        return Ok(None);
    };
    require(domain == scope.domain_id.0, "execution domain mismatch")?;
    let mut task_stmt = conn
        .prepare("SELECT record_json FROM routing_tasks WHERE run_id=?1 ORDER BY task_id")
        .map_err(err)?;
    let tasks = task_stmt
        .query_map([&scope.run_id.0], |r| r.get::<_, String>(0))
        .map_err(err)?
        .map(|row| parse(&row.map_err(err)?))
        .collect::<Result<Vec<_>>>()?;
    let mut attempt_stmt = conn
        .prepare(
            "SELECT record_json FROM routing_attempts WHERE run_id=?1 ORDER BY task_id,ordinal",
        )
        .map_err(err)?;
    let attempts = attempt_stmt
        .query_map([&scope.run_id.0], |r| r.get::<_, String>(0))
        .map_err(err)?
        .map(|row| parse(&row.map_err(err)?))
        .collect::<Result<Vec<_>>>()?;
    // Admission and lifecycle transactions read authoritative projections, not
    // the potentially large historical advice/audit stream. No event replay is
    // needed to reconstruct task state; explicit history queries load the journal.
    let events = if include_journal {
        let mut event_stmt=conn.prepare("SELECT sequence,event_id,payload_digest,event_json,result_json FROM routing_control_events WHERE run_id=?1 AND domain_id=?2 ORDER BY sequence").map_err(err)?;
        let events = event_stmt
            .query_map(params![scope.run_id.0, scope.domain_id.0], |r| {
                Ok((
                    r.get::<_, u64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .map_err(err)?
            .map(|row| {
                let (sequence, event_id, payload_digest, payload, result) = row.map_err(err)?;
                let event: RoutingControlEvent = parse(&payload)?;
                require(
                    hash(&event)?.0 == payload_digest,
                    "routing journal payload digest mismatch",
                )?;
                match &event {
                    RoutingControlEvent::Admitted(value) => {
                        let admitted: RoutedAttemptRecord = parse(&result)?;
                        require(
                            admitted.scope == value.scope
                                && admitted.task_id == value.task_id
                                && admitted.attempt_id == value.attempt_id
                                && admitted.agent_id == value.agent_id
                                && admitted.decision == value.decision
                                && admitted.capacity_reservation == value.capacity_reservation
                                && admitted.input_manifest == value.input_manifest
                                && admitted.handoff == value.handoff
                                && admitted.state == AttemptState::Admitted,
                            "routing admission result differs from its event",
                        )?;
                    }
                    RoutingControlEvent::DecisionObserved(value) => require(
                        parse::<ObservedRoutingDecision>(&result)? == **value,
                        "routing observation result differs from its event",
                    )?,
                    RoutingControlEvent::DecisionResolved(value) => require(
                        parse::<RoutingObservationResolution>(&result)? == **value,
                        "routing resolution result differs from its event",
                    )?,
                    RoutingControlEvent::Transitioned(value) => {
                        let transitioned: RoutedAttemptRecord = parse(&result)?;
                        require(
                            transitioned.scope == value.scope
                                && transitioned.attempt_id == value.attempt_id
                                && transitioned.state == value.to,
                            "routing transition result differs from its event",
                        )?;
                    }
                    RoutingControlEvent::Blocked(value) => {
                        let blocked: RoutedTaskRecord = parse(&result)?;
                        require(
                            blocked.registration.contract.task_id == value.task_id
                                && blocked.last_decision.as_ref() == Some(&value.decision),
                            "routing block result differs from its event",
                        )?;
                    }
                    RoutingControlEvent::Cancelled { expected_epoch, .. } => require(
                        parse::<u64>(&result)? == increment(*expected_epoch)?,
                        "routing cancellation result differs from its event",
                    )?,
                    RoutingControlEvent::UsageSettled {
                        attempt_id, usage, ..
                    } => {
                        let settled: RoutedAttemptRecord = parse(&result)?;
                        require(
                            settled.attempt_id == *attempt_id && settled.usage == *usage,
                            "routing usage result differs from its event",
                        )?;
                    }
                    _ => {}
                }
                Ok(RoutingJournalEntry {
                    sequence,
                    event_id,
                    event,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        events
    } else {
        Vec::new()
    };
    Ok(Some(RoutingHistory {
        mission: parse(&registration)?,
        routing_revision,
        cancel_epoch,
        cancelled,
        tasks,
        attempts,
        events,
    }))
}

/// The JSON records carry the useful display data, while SQL columns own row
/// identity and CAS revisions. Compare both in the same read transaction so a
/// corrupt projection cannot be attributed to another task or attempt.
fn validate_display_projection_keys(
    conn: &Connection,
    scope: &RoutingScope,
    history: &RoutingHistory,
) -> Result<()> {
    let mut task_stmt = conn
        .prepare("SELECT task_id,revision FROM routing_tasks WHERE run_id=?1 ORDER BY task_id")
        .map_err(err)?;
    let task_keys = task_stmt
        .query_map([&scope.run_id.0], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    require(
        task_keys.len() == history.tasks.len(),
        "routing display task row count changed",
    )?;
    for ((task_id, revision), task) in task_keys.iter().zip(&history.tasks) {
        require(
            *task_id == task.registration.contract.task_id.0 && *revision == task.revision,
            "routing display task row identity mismatch",
        )?;
    }

    let mut attempt_stmt = conn
        .prepare("SELECT attempt_id,agent_id,capacity_reservation,task_id,ordinal,revision FROM routing_attempts WHERE run_id=?1 ORDER BY task_id,ordinal")
        .map_err(err)?;
    let attempt_keys = attempt_stmt
        .query_map([&scope.run_id.0], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, u32>(4)?,
                row.get::<_, u64>(5)?,
            ))
        })
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    require(
        attempt_keys.len() == history.attempts.len(),
        "routing display attempt row count changed",
    )?;
    for ((attempt_id, agent_id, reservation, task_id, ordinal, revision), attempt) in
        attempt_keys.iter().zip(&history.attempts)
    {
        require(
            *attempt_id == attempt.attempt_id.0
                && *agent_id == attempt.agent_id
                && *reservation == attempt.capacity_reservation
                && *task_id == attempt.task_id.0
                && *ordinal == attempt.ordinal
                && *revision == attempt.revision
                && attempt.scope == *scope,
            "routing display attempt row identity mismatch",
        )?;
    }
    Ok(())
}
pub(crate) fn require_history(conn: &Connection, scope: &RoutingScope) -> Result<RoutingHistory> {
    load_history(conn, scope, false)?.ok_or_else(|| error("run has no routing authority"))
}
pub(crate) fn find_task<'a>(h: &'a RoutingHistory, id: &TaskId) -> Result<&'a RoutedTaskRecord> {
    h.tasks
        .iter()
        .find(|t| t.registration.contract.task_id == *id)
        .ok_or_else(|| error("task not registered"))
}
pub(crate) fn find_attempt<'a>(
    h: &'a RoutingHistory,
    id: &AttemptId,
) -> Result<&'a RoutedAttemptRecord> {
    h.attempts
        .iter()
        .find(|a| a.attempt_id == *id)
        .ok_or_else(|| error("attempt not registered in this scope"))
}

fn scope_current(h: &RoutingHistory, t: &RoutedTaskRecord, f: &RoutingFacts) -> bool {
    let c = &t.registration.contract;
    f.observed_at_ms <= f.now_ms
        && f.now_ms < f.expires_at_ms
        && f.base == c.base
        && f.plan_digest == c.plan_digest
        && f.permission_profile == c.permission_profile
        && c.plan_digest == h.mission.authorization.plan_digest
}
fn snapshot(
    conn: &Connection,
    h: &RoutingHistory,
    id: &TaskId,
    f: &RoutingFacts,
) -> Result<RoutingSnapshot> {
    let t = find_task(h, id)?;
    let a = &h.mission.authorization;
    let mut deps = Vec::new();
    for dependency_id in &t.registration.contract.dependencies {
        let dependency = find_task(h, dependency_id)?;
        require(
            (dependency.state == TaskRoutingState::Succeeded) == dependency.winner.is_some(),
            "dependency task and winner state differ",
        )?;
        if let Some(winner) = &dependency.winner {
            let attempt = find_attempt(h, &winner.winning_attempt_id)?;
            require(
                winner.task_id == *dependency_id
                    && dependency.current_attempt.as_ref() == Some(&winner.winning_attempt_id)
                    && attempt.state == AttemptState::Passed
                    && attempt.ownership_released
                    && attempt.task_id == *dependency_id
                    && attempt.receipts.sealed_output.as_ref() == Some(&winner.output_digest)
                    && attempt.receipts.checks.as_ref()
                        == Some(&winner.verification_receipt_digest),
                "dependency winner differs from its task or attempt",
            )?;
            require_owned_pass_integrity(conn, attempt)?;
            deps.push(winner.clone());
        }
    }
    let mut qualifications = BTreeSet::new();
    let mut stmt = conn
        .prepare("SELECT digest,qualification_json FROM routing_qualifications WHERE run_id=?1")
        .map_err(err)?;
    for row in stmt
        .query_map([&a.run_id.0], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(err)?
    {
        let (digest, body) = row.map_err(err)?;
        let q: AdapterQualification = parse(&body)?;
        require(
            q.digest().map_err(err)?.0 == digest,
            "corrupt qualification record",
        )?;
        qualifications.insert(Digest(digest));
    }
    let active_workers =
        u32::try_from(h.attempts.iter().filter(|a| !a.ownership_released).count()).map_err(err)?;
    let admitted_attempts = u32::try_from(h.attempts.len()).map_err(err)?;
    let mut total = Some(0_u64);
    for attempt in &h.attempts {
        let amount = match &attempt.usage {
            RoutedUsage::Known { nano_usd, .. } => Some(*nano_usd),
            RoutedUsage::Estimated { nano_usd, .. } => {
                Some((*nano_usd).max(attempt.reserved_nano_usd))
            }
            RoutedUsage::Unknown { .. } => None,
            RoutedUsage::Unreported => Some(attempt.reserved_nano_usd),
        };
        total = total.and_then(|n| amount.and_then(|amount| n.checked_add(amount)));
    }
    let run_active: bool = conn
        .query_row(
            "SELECT status IN ('starting','running') FROM runs WHERE id=?1",
            [&a.run_id.0],
            |r| r.get(0),
        )
        .map_err(err)?;
    Ok(RoutingSnapshot {
        domain_id: a.domain_id.clone(),
        run_id: a.run_id.clone(),
        plan_id: a.plan_id.clone(),
        task: t.registration.contract.clone(),
        authorization: a.clone(),
        task_revision: t.registration.contract.revision,
        task_state_revision: t.revision,
        authorization_revision: a.revision,
        cancel_epoch: h.cancel_epoch,
        next_ordinal: t.next_ordinal,
        now_ms: f.now_ms,
        dependencies_ready: deps.len() == t.registration.contract.dependencies.len(),
        resolved_dependencies: deps,
        qualification_digests: qualifications,
        previous_attempt: t
            .current_attempt
            .as_ref()
            .and_then(|id| h.attempts.iter().find(|a| &a.attempt_id == id))
            .and_then(|a| a.failure.clone()),
        cancelled: h.cancelled,
        scope_valid: run_active && scope_current(h, t, f),
        ownership_resolved: matches!(
            t.state,
            TaskRoutingState::Ready
                | TaskRoutingState::WaitingDependencies
                | TaskRoutingState::WaitingInput
        ) && t.winner.is_none()
            && t.current_attempt.as_ref().is_none_or(|id| {
                h.attempts
                    .iter()
                    .any(|a| &a.attempt_id == id && a.ownership_released)
            }),
        active_workers,
        admitted_attempts,
        spent_and_reserved_nano_usd: total,
        manual_target: f.manual_target.clone(),
        packet_digest: f.packet_digest.clone(),
        advice_request_id: f.advice_request_id.clone(),
    })
}

fn require_owned_pass_integrity(conn: &Connection, attempt: &RoutedAttemptRecord) -> Result<()> {
    if attempt.owned_launch_required && attempt.state == AttemptState::Passed {
        let aggregate = crate::routing_checker::require_owned_checker_pass_digest(conn, attempt)?;
        require(
            attempt.receipts.checks.as_ref() == Some(&aggregate),
            "passed owned worker lost its checker aggregate",
        )?;
    }
    Ok(())
}

fn benchmark_alias(key: &[u8; 32], kind: &str, raw: &str) -> Digest {
    let mut input = Vec::with_capacity(64 + kind.len() + raw.len());
    input.extend_from_slice(b"pytxo-benchmark-alias-v1\0");
    input.extend_from_slice(key);
    input.extend_from_slice(&(kind.len() as u32).to_le_bytes());
    input.extend_from_slice(kind.as_bytes());
    input.extend_from_slice(raw.as_bytes());
    Digest::of_bytes(&input)
}

fn benchmark_run_pins(
    mission: &RoutingMission,
    task: &TaskContract,
    key: &[u8; 32],
) -> Result<RoutingBenchmarkRunPins> {
    let role = |target: &RouteTarget| -> Result<&RegisteredProfile> {
        let mut matches = mission.profiles.iter().filter(|registered| {
            registered.profile.id == target.profile_id && registered.binding.id == target.binding_id
        });
        let profile = matches
            .next()
            .ok_or_else(|| error("benchmark reviewed profile role is absent"))?;
        require(
            matches.next().is_none(),
            "benchmark reviewed profile role is ambiguous",
        )?;
        Ok(profile)
    };
    let everyday = role(&mission.policy.everyday)?;
    let strong = role(&mission.policy.strong)?;
    require(
        everyday.profile.adapter_digest == strong.profile.adapter_digest,
        "benchmark profiles differ in declared adapter identity",
    )?;
    let pair = hash(&[
        everyday.profile.digest().map_err(err)?.0,
        everyday.binding.digest().map_err(err)?.0,
        strong.profile.digest().map_err(err)?.0,
        strong.binding.digest().map_err(err)?.0,
    ])?;
    let alias = |digest: &Digest| benchmark_alias(key, "digest", &digest.0);
    Ok(RoutingBenchmarkRunPins {
        task_contract_digest: alias(&task.digest().map_err(err)?),
        snapshot_digest: alias(&task.base.snapshot_digest),
        profile_pair_digest: alias(&pair),
        policy_digest: alias(&mission.policy.digest().map_err(err)?),
        adapter_digest: alias(&everyday.profile.adapter_digest),
    })
}

fn benchmark_receipts(
    key: &[u8; 32],
    receipts: &RoutingReceipts,
) -> RoutingBenchmarkReceiptAliases {
    let alias = |digest: &Option<Digest>| {
        digest
            .as_ref()
            .map(|digest| benchmark_alias(key, "digest", &digest.0))
    };
    RoutingBenchmarkReceiptAliases {
        inputs: alias(&receipts.inputs),
        launch_checks: alias(&receipts.launch_checks),
        process_identity: alias(&receipts.process_identity),
        no_worker_created: alias(&receipts.no_worker_created),
        quiescence: alias(&receipts.quiescence),
        sealed_output: alias(&receipts.sealed_output),
        checks: alias(&receipts.checks),
        reconciliation: alias(&receipts.reconciliation),
    }
}

fn benchmark_usage(key: &[u8; 32], usage: &RoutedUsage) -> RoutingBenchmarkUsage {
    match usage {
        RoutedUsage::Unreported => RoutingBenchmarkUsage::Unreported,
        RoutedUsage::Known { nano_usd, receipt } => RoutingBenchmarkUsage::Known {
            nano_usd: *nano_usd,
            receipt: benchmark_alias(key, "digest", &receipt.0),
        },
        RoutedUsage::Estimated { nano_usd, receipt } => RoutingBenchmarkUsage::Estimated {
            nano_usd: *nano_usd,
            receipt: benchmark_alias(key, "digest", &receipt.0),
        },
        RoutedUsage::Unknown { .. } => RoutingBenchmarkUsage::Unknown,
    }
}
fn candidates(m: &RoutingMission, f: &RoutingFacts) -> Result<Vec<ProfileCandidate>> {
    let mut seen = BTreeSet::new();
    for o in &f.observations {
        require(
            seen.insert((o.profile_digest.clone(), o.binding_digest.clone())),
            "duplicate profile observation",
        )?;
    }
    let mut result = Vec::new();
    for p in &m.profiles {
        let pd = p.profile.digest().map_err(err)?;
        let bd = p.binding.digest().map_err(err)?;
        if let Some(o) = f
            .observations
            .iter()
            .find(|o| o.profile_digest == pd && o.binding_digest == bd)
        {
            result.push(ProfileCandidate {
                profile: p.profile.clone(),
                binding: p.binding.clone(),
                observation: o.clone(),
            });
        }
    }
    Ok(result)
}
fn jev_advice_requires_journal(
    history: &RoutingHistory,
    decision: &RouteDecision,
    response: Option<&str>,
) -> bool {
    decision.advice_status == AdviceStatus::Applied
        || (matches!(
            history.mission.policy.mode,
            RoutingMode::Shadow | RoutingMode::Live
        ) && history.mission.policy.advice_model == "jev-1.13.0"
            && (response.is_some() || decision.advice_status == AdviceStatus::ShadowRecorded))
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep journal identity and current authority explicit"
)]
fn require_jev_advice_provenance(
    conn: &Connection,
    history: &RoutingHistory,
    scope: &RoutingScope,
    task_id: &TaskId,
    snapshot: &RoutingSnapshot,
    decision: &RouteDecision,
    response: Option<&str>,
    require_current_consent: bool,
) -> Result<()> {
    if !jev_advice_requires_journal(history, decision, response) {
        return Ok(());
    }
    let raw = response.ok_or_else(|| error("Jev observation lacks response"))?;
    let advice: AdviceEnvelope = serde_json::from_str(raw).map_err(err)?;
    let sent = advisor::load_request(conn, &advice.request_id)?
        .ok_or_else(|| error("Jev observation lacks send journal"))?;
    let consent = match sent.recipient_identity.as_deref() {
        Some(recipient) => advisor::load_hosted_consent(conn, &scope.domain_id, recipient)?,
        None => advisor::load_consent(conn, &scope.domain_id)?,
    };
    require(
        (!require_current_consent
            || (consent.enabled
                && consent.revision == sent.consent_revision
                && consent.scope_digest == history.mission.policy.disclosure_scope_digest))
            && sent.recipient_identity == history.mission.policy.advisor_recipient
            && (sent.recipient_identity.is_none() && sent.scope_digest.is_none()
                || sent.scope_digest == history.mission.policy.disclosure_scope_digest)
            && sent.scope == *scope
            && sent.task_id == *task_id
            && sent.ordinal == snapshot.next_ordinal
            && sent.phase == AdvisorSendPhase::Completed
            && sent.result_digest.as_ref() == Some(&Digest::of_bytes(raw.as_bytes()))
            && sent.created_at_ms <= advice.received_at_ms
            && advice.received_at_ms <= sent.updated_at_ms
            && sent.packet_digest == advice.packet_digest
            && sent.policy_digest == decision.policy_digest
            && sent.task_revision == snapshot.task_revision
            && sent.task_state_revision == snapshot.task_state_revision
            && sent.authorization_revision == snapshot.authorization_revision
            && sent.cancel_epoch == snapshot.cancel_epoch
            && sent.consent_revision == snapshot.authorization.consent_revision,
        "Jev observation is not a current completed send",
    )
}

fn decision(
    h: &RoutingHistory,
    s: &RoutingSnapshot,
    f: &RoutingFacts,
    advice: Option<&str>,
) -> Result<RouteDecision> {
    let parsed = advice.and_then(|raw| serde_json::from_str::<AdviceEnvelope>(raw).ok());
    let catalog = build_eligible_catalog(s, &candidates(&h.mission, f)?);
    let mut selected = select_route(s, &catalog, &h.mission.policy, parsed.as_ref());
    if advice.is_some()
        && parsed.is_none()
        && matches!(
            h.mission.policy.mode,
            RoutingMode::Shadow | RoutingMode::Live
        )
        && selected.reason != RouteReason::Blocked
    {
        // A supplied but undecodable response is still an attempted advisory
        // decision. Preserve the rules route while making the fallback visible.
        selected.advice_status = AdviceStatus::InvalidOrStale;
    }
    // Core checks the mission ceiling. Store also knows this task's frozen
    // per-attempt reservation, so expose insufficient headroom as a durable
    // blocker rather than advertising a selection admission cannot reserve.
    let reserved = find_task(h, &s.task.task_id)?
        .registration
        .attempt_budget_nano_usd;
    if matches!(
        selected.selection,
        RouteSelection::Selected(_) | RouteSelection::WaitForCapacity { .. }
    ) && s
        .authorization
        .limits
        .max_spend_nano_usd
        .is_some_and(|limit| {
            s.spent_and_reserved_nano_usd
                .and_then(|spent| spent.checked_add(reserved))
                .is_none_or(|total| total > limit)
        })
    {
        selected.selection = RouteSelection::Blocked(RouteBlocker::BudgetExhausted);
        selected.reason = RouteReason::Blocked;
    }
    Ok(selected)
}
fn save_task(tx: &Transaction<'_>, scope: &RoutingScope, t: &mut RoutedTaskRecord) -> Result<()> {
    let old = t.revision;
    t.revision = increment(old)?;
    let n=tx.execute("UPDATE routing_tasks SET revision=?1,record_json=?2 WHERE run_id=?3 AND task_id=?4 AND revision=?5",params![t.revision,json(t)?,scope.run_id.0,t.registration.contract.task_id.0,old]).map_err(err)?;
    require(n == 1, "task CAS conflict")
}
pub(crate) fn save_attempt(tx: &Transaction<'_>, a: &mut RoutedAttemptRecord) -> Result<()> {
    let old = a.revision;
    a.revision = increment(old)?;
    let n=tx.execute("UPDATE routing_attempts SET revision=?1,record_json=?2 WHERE attempt_id=?3 AND run_id=?4 AND revision=?5",params![a.revision,json(a)?,a.attempt_id.0,a.scope.run_id.0,old]).map_err(err)?;
    require(n == 1, "attempt CAS conflict")
}
fn observation_resolution_event_id(
    scope: &RoutingScope,
    observation_event_id: &str,
) -> Result<String> {
    require(
        !observation_event_id.trim().is_empty(),
        "empty observation event ID",
    )?;
    Ok(format!(
        "routing-observation-resolved:{}",
        hash(&(scope, observation_event_id))?.0
    ))
}
fn require_observed_decision(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    observation_event_id: &str,
) -> Result<ObservedRoutingDecision> {
    require(
        !observation_event_id.trim().is_empty(),
        "empty observation event ID",
    )?;
    let row: Option<(String, String, String)> = tx
        .query_row(
            "SELECT domain_id,payload_digest,event_json FROM routing_control_events WHERE run_id=?1 AND event_id=?2",
            params![scope.run_id.0, observation_event_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(err)?;
    let (domain, digest, raw) = row.ok_or_else(|| error("selected observation absent"))?;
    let event: RoutingControlEvent = parse(&raw)?;
    require(
        domain == scope.domain_id.0 && digest == hash(&event)?.0,
        "selected observation journal identity mismatch",
    )?;
    let RoutingControlEvent::DecisionObserved(observed) = event else {
        return fail("event is not a selected observation");
    };
    require(
        observed.schema_version == 1
            && observed.scope == *scope
            && observed.request_digest.is_valid()
            && matches!(observed.decision.selection, RouteSelection::Selected(_)),
        "selected observation is malformed",
    )?;
    Ok(*observed)
}
fn observation_resolution(
    scope: &RoutingScope,
    observation_event_id: &str,
    outcome: RoutingObservationOutcome,
    resolved_at_ms: u64,
) -> RoutingObservationResolution {
    RoutingObservationResolution {
        schema_version: 1,
        scope: scope.clone(),
        observation_event_id: observation_event_id.to_owned(),
        outcome,
        resolved_at_ms,
    }
}
fn validate_observation_link(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    observation_event_id: &str,
    snapshot: &RoutingSnapshot,
    decision: &RouteDecision,
    advice_json: Option<&str>,
    exact_decision: bool,
) -> Result<()> {
    let observed = require_observed_decision(tx, scope, observation_event_id)?;
    require(
        observed.task_id == snapshot.task.task_id
            && observed.next_ordinal == snapshot.next_ordinal
            && observed.cancel_epoch == snapshot.cancel_epoch
            && observed.decision.task_state_revision == decision.task_state_revision
            && observed.observed_at_ms <= snapshot.now_ms
            && observed.advice_digest == advice_json.map(|raw| Digest::of_bytes(raw.as_bytes()))
            && (!exact_decision || observed.decision == *decision),
        "selected observation does not match current outcome",
    )?;
    require_resolution_absent(tx, scope, observation_event_id)?;
    Ok(())
}
fn append_observation_resolution(
    tx: &Transaction<'_>,
    resolution: &RoutingObservationResolution,
) -> Result<()> {
    let event_id =
        require_resolution_absent(tx, &resolution.scope, &resolution.observation_event_id)?;
    let event = RoutingControlEvent::DecisionResolved(Box::new(resolution.clone()));
    append_event(tx, &resolution.scope, &event_id, &event, resolution)
}
fn read_observation_resolution(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    observation_event_id: &str,
) -> Result<Option<RoutingObservationResolution>> {
    let event_id = observation_resolution_event_id(scope, observation_event_id)?;
    let row: Option<(String, String, String)> = tx
        .query_row(
            "SELECT domain_id,payload_digest,event_json FROM routing_control_events WHERE run_id=?1 AND event_id=?2",
            params![scope.run_id.0, event_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(err)?;
    let Some((domain, digest, raw)) = row else {
        return Ok(None);
    };
    let event: RoutingControlEvent = parse(&raw)?;
    require(
        domain == scope.domain_id.0 && digest == hash(&event)?.0,
        "observation resolution journal identity mismatch",
    )?;
    let RoutingControlEvent::DecisionResolved(resolution) = event else {
        return fail("observation resolution ID has wrong event kind");
    };
    require(
        resolution.schema_version == 1
            && resolution.scope == *scope
            && resolution.observation_event_id == observation_event_id,
        "observation resolution owner mismatch",
    )?;
    Ok(Some(*resolution))
}
struct RoutingObservationState {
    seen: bool,
    pending: Option<String>,
}
fn observation_state_for_task(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    task_id: &TaskId,
    next_ordinal: u32,
) -> Result<RoutingObservationState> {
    const MAX_SCANNED_OBSERVATIONS: usize = 4_096;
    let mut stmt = tx
        .prepare("SELECT event_id,payload_digest,event_json FROM routing_control_events WHERE run_id=?1 AND domain_id=?2 AND json_extract(event_json,'$.kind')='decision_observed' ORDER BY sequence LIMIT 4097")
        .map_err(err)?;
    let rows = stmt
        .query_map(params![scope.run_id.0, scope.domain_id.0], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(err)?;
    let mut seen = 0_usize;
    let mut candidates = Vec::new();
    let mut seen_task = false;
    for row in rows {
        seen += 1;
        require(
            seen <= MAX_SCANNED_OBSERVATIONS,
            "routing observation journal scan limit",
        )?;
        let (event_id, digest, raw) = row.map_err(err)?;
        let event: RoutingControlEvent = parse(&raw)?;
        require(
            digest == hash(&event)?.0,
            "routing journal event digest mismatch",
        )?;
        if let RoutingControlEvent::DecisionObserved(observed) = event {
            if observed.scope == *scope && observed.task_id == *task_id {
                seen_task = true;
                if observed.next_ordinal == next_ordinal {
                    candidates.push(event_id);
                }
            }
        }
    }
    let mut pending = None;
    for id in candidates {
        if read_observation_resolution(tx, scope, &id)?.is_none() {
            require(
                pending.is_none(),
                "multiple unresolved routing observations",
            )?;
            pending = Some(id);
        }
    }
    Ok(RoutingObservationState {
        seen: seen_task,
        pending,
    })
}

/// The latest selected observation can be shown without exposing the private
/// journal. Admitted observations are already represented by their attempt.
fn display_pre_admission_observation(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    task_id: &TaskId,
    policy: &RoutingPolicy,
    attempts: &[RoutedAttemptRecord],
) -> Result<Option<RoutingDisplayPreAdmission>> {
    const MAX_SCANNED_OBSERVATIONS: usize = 4_096;
    let mut stmt = tx
        .prepare("SELECT event_id,payload_digest,event_json FROM routing_control_events WHERE run_id=?1 AND domain_id=?2 AND json_extract(event_json,'$.kind')='decision_observed' ORDER BY sequence DESC LIMIT 4097")
        .map_err(err)?;
    let rows = stmt
        .query_map(params![scope.run_id.0, scope.domain_id.0], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(err)?;
    let mut admitted_later_ordinals = BTreeSet::new();
    for (scanned, row) in rows.enumerate() {
        require(
            scanned < MAX_SCANNED_OBSERVATIONS,
            "routing display observation scan limit",
        )?;
        let (event_id, digest, raw) = row.map_err(err)?;
        let event: RoutingControlEvent = parse(&raw)?;
        require(
            digest == hash(&event)?.0,
            "routing display observation digest mismatch",
        )?;
        let RoutingControlEvent::DecisionObserved(observed) = event else {
            return fail("routing display observation has wrong event kind");
        };
        if observed.task_id != *task_id {
            continue;
        }
        require(
            observed.schema_version == 1
                && observed.scope == *scope
                && observed.next_ordinal > 0
                && matches!(observed.decision.selection, RouteSelection::Selected(_)),
            "routing display observation identity mismatch",
        )?;
        let resolution = read_observation_resolution(tx, scope, &event_id)?;
        let outcome = match resolution {
            None => RoutingDisplayPreAdmissionOutcome::Pending,
            Some(resolution) => {
                require(
                    resolution.resolved_at_ms >= observed.observed_at_ms,
                    "routing display observation time regressed",
                )?;
                match resolution.outcome {
                    RoutingObservationOutcome::Admitted { attempt_id } => {
                        require(
                            attempts.iter().any(|attempt| {
                                attempt.scope == *scope
                                    && attempt.task_id == *task_id
                                    && attempt.attempt_id == attempt_id
                                    && attempt.ordinal == observed.next_ordinal
                                    && attempt.decision == observed.decision
                            }),
                            "routing display admitted observation has no matching attempt",
                        )?;
                        admitted_later_ordinals.insert(observed.next_ordinal);
                        continue;
                    }
                    RoutingObservationOutcome::NotAdmitted { stage, .. } => {
                        RoutingDisplayPreAdmissionOutcome::NotAdmitted { stage }
                    }
                }
            }
        };
        let has_attempt = attempts.iter().any(|attempt| {
            attempt.scope == *scope
                && attempt.task_id == *task_id
                && attempt.ordinal == observed.next_ordinal
        });
        require(
            !has_attempt
                || (matches!(
                    outcome,
                    RoutingDisplayPreAdmissionOutcome::NotAdmitted { .. }
                ) && admitted_later_ordinals.contains(&observed.next_ordinal)),
            "routing display unadmitted observation has an unlinked attempt",
        )?;
        return Ok(Some(RoutingDisplayPreAdmission {
            ordinal: observed.next_ordinal,
            decision: display_decision(policy, &observed.decision)?,
            outcome,
        }));
    }
    Ok(None)
}

fn require_resolution_absent(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    observation_event_id: &str,
) -> Result<String> {
    let event_id = observation_resolution_event_id(scope, observation_event_id)?;
    require(
        read_observation_resolution(tx, scope, observation_event_id)?.is_none(),
        "selected observation already resolved",
    )?;
    Ok(event_id)
}
fn require_matching_resolution(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    resolution: &RoutingObservationResolution,
) -> Result<()> {
    require(
        read_observation_resolution(tx, scope, &resolution.observation_event_id)?.as_ref()
            == Some(resolution),
        "linked observation resolution missing or changed",
    )
}
fn replay<T: DeserializeOwned>(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    id: &str,
    event: &RoutingControlEvent,
) -> Result<Option<T>> {
    require(!id.trim().is_empty(), "empty routing event ID")?;
    let row:Option<(String,String,String)>=tx.query_row("SELECT domain_id,payload_digest,result_json FROM routing_control_events WHERE run_id=?1 AND event_id=?2",params![scope.run_id.0,id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(err)?;
    let Some((domain, digest, result)) = row else {
        return Ok(None);
    };
    require(
        domain == scope.domain_id.0 && digest == hash(event)?.0,
        "event identity payload conflict",
    )?;
    Ok(Some(parse(&result)?))
}
fn append_event<T: Serialize>(
    tx: &Transaction<'_>,
    scope: &RoutingScope,
    id: &str,
    event: &RoutingControlEvent,
    result: &T,
) -> Result<()> {
    tx.execute("INSERT INTO routing_control_events(run_id,domain_id,event_id,payload_digest,event_json,result_json) VALUES (?1,?2,?3,?4,?5,?6)",params![scope.run_id.0,scope.domain_id.0,id,hash(event)?.0,json(event)?,json(result)?]).map_err(err)?;
    let n=tx.execute("UPDATE routing_missions SET revision=revision+1 WHERE run_id=?1 AND domain_id=?2 AND revision < 9223372036854775807",params![scope.run_id.0,scope.domain_id.0]).map_err(err)?;
    require(n == 1, "mission revision exhausted")?;
    crate::store::append_domain_change(tx, "routing", &scope.run_id.0)
}
fn merge_receipts(target: &mut RoutingReceipts, incoming: &RoutingReceipts) -> Result<()> {
    for (stored, new) in [
        (&mut target.inputs, &incoming.inputs),
        (&mut target.launch_checks, &incoming.launch_checks),
        (&mut target.process_identity, &incoming.process_identity),
        (&mut target.no_worker_created, &incoming.no_worker_created),
        (&mut target.quiescence, &incoming.quiescence),
        (&mut target.sealed_output, &incoming.sealed_output),
        (&mut target.checks, &incoming.checks),
        (&mut target.reconciliation, &incoming.reconciliation),
    ] {
        if let Some(value) = new {
            require(value.is_valid(), "invalid lifecycle receipt digest")?;
            require(
                stored.as_ref().is_none_or(|old| old == value),
                "immutable lifecycle receipt conflict",
            )?;
            *stored = Some(value.clone());
        }
    }
    Ok(())
}
fn validate_receipt_phase(from: AttemptState, to: AttemptState, r: &RoutingReceipts) -> Result<()> {
    use AttemptState::*;
    require(
        r.inputs.is_none() || to == Preparing,
        "input receipt outside preparation",
    )?;
    require(
        r.launch_checks.is_none() || to == Launching,
        "launch receipt outside launch gate",
    )?;
    require(
        r.process_identity.is_none() || matches!(to, Running | RecoveryRequired),
        "process receipt outside registration/recovery",
    )?;
    require(
        r.no_worker_created.is_none() || matches!(to, FailedNoLaunch | Cancelled),
        "no-launch receipt outside terminal reconciliation",
    )?;
    require(
        r.quiescence.is_none() || matches!(from, Running | Sealing | Verifying | RecoveryRequired),
        "quiescence receipt predates execution",
    )?;
    require(
        r.sealed_output.is_none() || to == Verifying,
        "output receipt outside sealing",
    )?;
    require(
        r.checks.is_none() || to == Passed,
        "passing check receipt outside verification",
    )?;
    require(
        r.reconciliation.is_none() || from == RecoveryRequired,
        "reconciliation receipt without unresolved ownership",
    )
}
fn refresh_dependents(tx: &Transaction<'_>, scope: &RoutingScope) -> Result<()> {
    loop {
        let h = require_history(tx, scope)?;
        let mut changed = false;
        for mut task in h.tasks.clone() {
            if !matches!(
                task.state,
                TaskRoutingState::WaitingDependencies
                    | TaskRoutingState::WaitingInput
                    | TaskRoutingState::Ready
            ) || task.current_attempt.is_some()
            {
                continue;
            }
            let deps = &task.registration.contract.dependencies;
            let state = if deps.iter().any(|id| {
                find_task(&h, id).is_ok_and(|t| {
                    matches!(
                        t.state,
                        TaskRoutingState::Failed
                            | TaskRoutingState::BlockedDependency
                            | TaskRoutingState::Cancelled
                    )
                })
            }) {
                TaskRoutingState::BlockedDependency
            } else if !deps.is_empty()
                && deps
                    .iter()
                    .all(|id| find_task(&h, id).is_ok_and(|t| t.winner.is_some()))
                && task.state == TaskRoutingState::WaitingDependencies
            {
                TaskRoutingState::Ready
            } else {
                task.state.clone()
            };
            if state != task.state {
                task.state = state;
                save_task(tx, scope, &mut task)?;
                changed = true;
            }
        }
        if !changed {
            return Ok(());
        }
    }
}
