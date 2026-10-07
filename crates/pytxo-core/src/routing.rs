//! Pure, versioned routing contracts. These snapshots never grant authority or probe a host.
use crate::{DomainId, ExecutionBackend, PermissionProfile, RunId, TaskId};
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeSet;

macro_rules! opaque_ids {
    ($($name:ident),+ $(,)?) => { $(
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);
        impl $name { pub fn as_str(&self) -> &str { &self.0 } }
        impl std::fmt::Display for $name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }
    )+ };
}
opaque_ids!(
    ProfileId,
    BindingId,
    BillingSourceId,
    PlanId,
    AttemptId,
    CheckId,
    AdviceRequestId,
    Digest
);

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ContractError {
    #[error("unsupported contract or canonicalization version")]
    UnsupportedVersion,
    #[error("invalid authoritative manifest: {0}")]
    InvalidManifest(String),
    #[error("invalid attempt transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: AttemptState,
        to: AttemptState,
    },
}
impl Digest {
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self(format!("{:x}", Sha256::digest(bytes)))
    }
    pub fn is_valid(&self) -> bool {
        self.0.len() == 64
            && self
                .0
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }
}
/// Canonicalization v1 sorts object keys, preserves array order and explicit nulls,
/// and rejects JSON floating point numbers. Authoritative contract fields are integer-only.
pub fn canonical_digest<T: Serialize>(value: &T, version: u32) -> Result<Digest, ContractError> {
    if version != 1 {
        return Err(ContractError::UnsupportedVersion);
    }
    fn write(value: &serde_json::Value, out: &mut Vec<u8>) -> Result<(), ContractError> {
        match value {
            serde_json::Value::Object(map) => {
                out.push(b'{');
                let mut entries: Vec<_> = map.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0));
                for (index, (key, value)) in entries.into_iter().enumerate() {
                    if index > 0 {
                        out.push(b',');
                    }
                    out.extend(serde_json::to_vec(key).map_err(json_error)?);
                    out.push(b':');
                    write(value, out)?;
                }
                out.push(b'}');
            }
            serde_json::Value::Array(values) => {
                out.push(b'[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        out.push(b',');
                    }
                    write(value, out)?;
                }
                out.push(b']');
            }
            serde_json::Value::Number(number) if !number.is_i64() && !number.is_u64() => {
                return Err(invalid("floats are not authoritative"))
            }
            _ => out.extend(serde_json::to_vec(value).map_err(json_error)?),
        }
        Ok(())
    }
    let mut bytes = Vec::new();
    write(
        &serde_json::to_value(value).map_err(json_error)?,
        &mut bytes,
    )?;
    Ok(Digest::of_bytes(&bytes))
}
fn json_error(error: serde_json::Error) -> ContractError {
    invalid(&error.to_string())
}
fn invalid(message: &str) -> ContractError {
    ContractError::InvalidManifest(message.into())
}
fn versioned_digest<T: Serialize>(
    value: &T,
    schema: u32,
    canonicalization: u32,
) -> Result<Digest, ContractError> {
    if schema != 1 {
        return Err(ContractError::UnsupportedVersion);
    }
    canonical_digest(value, canonicalization)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelIdentity {
    pub provider: String,
    pub model: String,
    pub reasoning: Option<String>,
    pub revision: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelIdentityLevel {
    Requested,
    HarnessReported,
    ProviderAttested,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionProfile {
    pub schema_version: u32,
    pub canonicalization_version: u32,
    pub id: ProfileId,
    pub revision: u64,
    pub harness_id: String,
    pub adapter_contract_version: String,
    pub adapter_digest: Digest,
    pub requested_model: ModelIdentity,
    pub skill_tool_bundle_digest: Digest,
    pub backend: ExecutionBackend,
    pub capabilities: BTreeSet<String>,
}
impl ExecutionProfile {
    pub fn digest(&self) -> Result<Digest, ContractError> {
        versioned_digest(self, self.schema_version, self.canonicalization_version)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingSourceMode {
    Subscription,
    Api,
    Local,
    Managed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileBinding {
    pub schema_version: u32,
    pub canonicalization_version: u32,
    pub id: BindingId,
    pub revision: u64,
    pub profile_digest: Digest,
    pub credential_reference: Option<String>,
    pub auth_owner: String,
    pub billing_source_id: BillingSourceId,
    pub billing_mode: BillingSourceMode,
    pub endpoint_identity: String,
    pub trust_class: String,
    pub capacity_pool_ids: BTreeSet<String>,
}
impl ProfileBinding {
    pub fn digest(&self) -> Result<Digest, ContractError> {
        versioned_digest(self, self.schema_version, self.canonicalization_version)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutableIdentity {
    pub path: String,
    pub version: String,
    pub digest: Digest,
}
/// The adapter's qualified launch shape. This contains identities and policy
/// digests, never prompt bytes, credentials, or resolved environment values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchTransport {
    DirectSubprocess,
    HostSubprocess,
    HostPty,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StdinDelivery {
    Closed,
    PrivateHostPipe,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchContract {
    pub schema_version: u32,
    pub transport: LaunchTransport,
    pub host: Option<ExecutableIdentity>,
    pub dependencies: Vec<ExecutableIdentity>,
    /// Canonical digest of the adapter's exact non-secret argv template.
    pub arguments_digest: Digest,
    /// Canonical digest of allowed environment names and their binding sources.
    pub environment_policy_digest: Digest,
    pub working_directory_policy: String,
    pub stdin_delivery: StdinDelivery,
    /// Digest of the exact private payload bytes delivered to a hosted agent.
    /// The bytes themselves never enter the public launch contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_stdin_digest: Option<Digest>,
    pub output_protocol: String,
    pub argument_lowering: String,
    pub barrier_timeout_ms: Option<u64>,
    pub execution_timeout_ms: u64,
    pub settlement_timeout_ms: u64,
    pub output_limit_bytes: u64,
}
impl LaunchContract {
    pub fn valid_for(&self, backend: ExecutionBackend) -> bool {
        let direct = self.transport == LaunchTransport::DirectSubprocess
            && backend == ExecutionBackend::Subprocess
            && self.host.is_none()
            && self.stdin_delivery == StdinDelivery::Closed
            && self.barrier_timeout_ms.is_none()
            && self.output_protocol == "pytxo-direct-suspended/1";
        let hosted = matches!(
            (self.transport, backend),
            (
                LaunchTransport::HostSubprocess,
                ExecutionBackend::Subprocess
            ) | (LaunchTransport::HostPty, ExecutionBackend::Pty)
        ) && self.host.as_ref().is_some_and(valid_executable)
            && self.stdin_delivery == StdinDelivery::PrivateHostPipe
            && self.barrier_timeout_ms.is_some_and(|timeout| timeout > 0)
            && self.output_protocol == "pytxo-attempt-host/1";
        self.schema_version == 1
            && self.arguments_digest.is_valid()
            && self.environment_policy_digest.is_valid()
            && match self.stdin_delivery {
                StdinDelivery::Closed => self.private_stdin_digest.is_none(),
                StdinDelivery::PrivateHostPipe => self
                    .private_stdin_digest
                    .as_ref()
                    .is_some_and(Digest::is_valid),
            }
            && self.working_directory_policy == "reviewed_attempt_worktree_v1"
            && self.execution_timeout_ms > 0
            && self.settlement_timeout_ms > 0
            && self.output_limit_bytes > 0
            && self.argument_lowering
                == if direct {
                    "windows-createprocess-structured-argv/v1"
                } else {
                    "rust-std-command-windows-structured-argv/v1"
                }
            && self.dependencies.iter().all(valid_executable)
            && (direct || hosted)
    }
}
fn valid_executable(executable: &ExecutableIdentity) -> bool {
    !executable.path.is_empty() && !executable.version.is_empty() && executable.digest.is_valid()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Readiness {
    Ready,
    Unavailable,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeteringSupport {
    Verified,
    Estimated,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterQualification {
    pub receipt_digest: Digest,
    /// Stable adapter shape. Hosted private-stdin content is separately bound
    /// by each admitted attempt's exact launch fingerprint and native gate.
    pub launch_fingerprint: Digest,
    pub permission_profile: PermissionProfile,
    pub tool_probe_passed: bool,
    pub cancellation_probe_passed: bool,
    pub quiescence_probe_passed: bool,
    pub capabilities: BTreeSet<String>,
    pub allowed_egress: BTreeSet<String>,
    pub capacity_pool_ids: BTreeSet<String>,
}
impl AdapterQualification {
    /// Identity of the complete locally validated qualification, including its scope.
    pub fn digest(&self) -> Result<Digest, ContractError> {
        canonical_digest(self, 1)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileObservation {
    pub schema_version: u32,
    pub profile_digest: Digest,
    pub binding_digest: Digest,
    pub executable: ExecutableIdentity,
    /// Missing on older observations; those observations fail closed at admission.
    #[serde(default)]
    pub launch: Option<LaunchContract>,
    pub observed_at_ms: u64,
    pub expires_at_ms: u64,
    pub auth_status: Readiness,
    pub dispatch_supported: bool,
    pub qualification: Option<AdapterQualification>,
    pub requested_model: ModelIdentity,
    pub reported_model: Option<ModelIdentity>,
    pub model_identity_level: ModelIdentityLevel,
    pub metering_support: MeteringSupport,
    pub hard_spend_limit_verified: bool,
    pub capacity_ready: bool,
}
pub fn launch_fingerprint(
    profile: &ExecutionProfile,
    binding: &ProfileBinding,
    executable: &ExecutableIdentity,
    launch: &LaunchContract,
) -> Result<Digest, ContractError> {
    if !valid_executable(executable) || !launch.valid_for(profile.backend) {
        return Err(invalid("unqualified launch shape"));
    }
    canonical_digest(
        &(
            2_u32,
            profile.digest()?,
            binding.digest()?,
            executable,
            launch,
        ),
        1,
    )
}

/// Fingerprint a demonstrated adapter capability without tying its proof to
/// one task's private prompt. Direct launches remain byte-for-byte compatible
/// with their exact launch fingerprint. Hosted launches mask only the private
/// stdin digest; every other profile, account and launch-policy field remains
/// in the qualification. Admission and native launch still use the exact
/// `launch_fingerprint` and check the actual stdin digest separately.
pub fn qualification_fingerprint(
    profile: &ExecutionProfile,
    binding: &ProfileBinding,
    executable: &ExecutableIdentity,
    launch: &LaunchContract,
) -> Result<Digest, ContractError> {
    let exact = launch_fingerprint(profile, binding, executable, launch)?;
    if launch.stdin_delivery == StdinDelivery::Closed {
        return Ok(exact);
    }
    let mut stable = launch.clone();
    stable.private_stdin_digest = Some(Digest::of_bytes(b"pytxo-private-stdin-shape-v1"));
    canonical_digest(
        &(
            3_u32,
            profile.digest()?,
            binding.digest()?,
            executable,
            &stable,
        ),
        1,
    )
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteTarget {
    pub profile_id: ProfileId,
    pub binding_id: BindingId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileCandidate {
    pub profile: ExecutionProfile,
    pub binding: ProfileBinding,
    pub observation: ProfileObservation,
}
impl ProfileCandidate {
    pub fn target(&self) -> RouteTarget {
        RouteTarget {
            profile_id: self.profile.id.clone(),
            binding_id: self.binding.id.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovedProfile {
    pub target: RouteTarget,
    pub profile_digest: Digest,
    pub binding_digest: Digest,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseSnapshot {
    pub repository_identity: String,
    pub git_revision: String,
    pub snapshot_digest: Digest,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckRecipe {
    pub id: CheckId,
    pub recipe_digest: Digest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    Documentation,
    Formatting,
    Rename,
    LocalTransformation,
    Diagnosis,
    Architecture,
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedDependencyOutput {
    pub task_id: TaskId,
    pub winning_attempt_id: AttemptId,
    pub output_digest: Digest,
    pub verification_receipt_digest: Digest,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskContract {
    pub schema_version: u32,
    pub canonicalization_version: u32,
    pub task_id: TaskId,
    pub revision: u64,
    pub plan_digest: Digest,
    pub base: BaseSnapshot,
    pub goal: String,
    pub constraints: Vec<String>,
    pub claim_roots: Vec<String>,
    pub dependencies: Vec<TaskId>,
    pub task_kind: Option<TaskKind>,
    pub task_kind_evidence: Option<Digest>,
    pub required_capabilities: BTreeSet<String>,
    pub checks: Vec<CheckRecipe>,
    pub required_resources: BTreeSet<String>,
    pub skill_tool_bundle_digest: Digest,
    pub permission_profile: PermissionProfile,
    pub required_egress: BTreeSet<String>,
    pub required_target: Option<RouteTarget>,
    pub strong_only: bool,
    pub cross_component_requirement: Option<bool>,
    pub context_complete: bool,
    /// Reviewer-declared diagnosis cues. Absent fields preserve old contract
    /// bytes and mean unknown, never an inferred absence from the task text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeatable_symptom_supplied: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specific_cause_hypothesis_supplied: Option<bool>,
}
impl TaskContract {
    pub fn digest(&self) -> Result<Digest, ContractError> {
        versioned_digest(self, self.schema_version, self.canonicalization_version)
    }

    /// Necessary reviewed facts for a future advice envelope to be recordable.
    /// This does not authorize a disclosure or make an envelope valid by itself.
    pub fn has_recordable_advice_context(&self) -> bool {
        self.context_complete
            && self.task_kind.is_some()
            && self
                .task_kind_evidence
                .as_ref()
                .is_some_and(Digest::is_valid)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpendGuarantee {
    Hard,
    RiskBounded,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissionLimits {
    pub deadline_ms: u64,
    pub max_workers: u32,
    pub max_attempts: u32,
    pub max_spend_nano_usd: Option<u64>,
    pub spend_guarantee: SpendGuarantee,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissionAuthorization {
    pub schema_version: u32,
    pub domain_id: DomainId,
    pub run_id: RunId,
    pub plan_id: PlanId,
    pub plan_digest: Digest,
    pub revision: u64,
    pub cancel_epoch: u64,
    pub allowed_task_digests: BTreeSet<Digest>,
    pub allowed_profiles: Vec<ApprovedProfile>,
    pub allowed_billing_sources: BTreeSet<BillingSourceId>,
    pub allowed_billing_modes: BTreeSet<BillingSourceMode>,
    pub permission_profile: PermissionProfile,
    pub allowed_egress: BTreeSet<String>,
    pub minimum_model_identity: ModelIdentityLevel,
    pub limits: MissionLimits,
    pub policy_digest: Digest,
    pub live_advice_authorized: bool,
    pub consent_revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingSnapshot {
    pub domain_id: DomainId,
    pub run_id: RunId,
    pub plan_id: PlanId,
    pub task: TaskContract,
    pub authorization: MissionAuthorization,
    /// Frozen reviewed TaskContract revision, not the mutable ledger state revision.
    pub task_revision: u64,
    /// Mutable TaskState CAS revision; independent of the frozen task contract revision.
    pub task_state_revision: u64,
    pub authorization_revision: u64,
    pub cancel_epoch: u64,
    pub next_ordinal: u32,
    pub now_ms: u64,
    /// Current immutable winners read from the domain ledger, in planned dependency order.
    pub resolved_dependencies: Vec<VerifiedDependencyOutput>,
    /// Digests of complete qualifications read from the trusted local adapter registry.
    /// Never populate this set from worker, profile-catalog or advice proposals.
    pub qualification_digests: BTreeSet<Digest>,
    /// Prior outcome and actionable evidence read from the durable attempt ledger.
    pub previous_attempt: Option<RepairEvidence>,
    pub cancelled: bool,
    pub scope_valid: bool,
    pub dependencies_ready: bool,
    pub ownership_resolved: bool,
    pub active_workers: u32,
    pub admitted_attempts: u32,
    pub spent_and_reserved_nano_usd: Option<u64>,
    pub manual_target: Option<RouteTarget>,
    pub packet_digest: Option<Digest>,
    pub advice_request_id: Option<AdviceRequestId>,
}
impl RoutingSnapshot {
    /// Necessary reviewed task facts for any advice to become valid evidence.
    /// A transport should check this before spending on a hosted request;
    /// the complete envelope is still checked at observation time.
    pub fn has_recordable_advice_context(&self) -> bool {
        self.task.has_recordable_advice_context()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    #[default]
    Disabled,
    Rules,
    Shadow,
    Live,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingPolicy {
    pub schema_version: u32,
    pub version: String,
    pub mode: RoutingMode,
    pub everyday: RouteTarget,
    pub strong: RouteTarget,
    pub evaluated_manifest_digest: Option<Digest>,
    pub everyday_threshold_ppm: u32,
    pub unclear_ceiling_ppm: u32,
    pub advice_model: String,
    pub advice_template: String,
    /// Workspace disclosure grant identity: recipient, packet projection and
    /// wire template, excluding a task-specific request digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disclosure_scope_digest: Option<Digest>,
    /// Versioned disclosure recipient. Omission preserves existing mission
    /// bytes and means the no-network fixture, never hosted authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisor_recipient: Option<String>,
}
impl RoutingPolicy {
    pub fn digest(&self) -> Result<Digest, ContractError> {
        versioned_digest(self, self.schema_version, 1)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    InvalidContract,
    UnapprovedProfile,
    UnapprovedBinding,
    BindingDigestMismatch,
    UnsupportedHarness,
    StaleObservation,
    UnqualifiedAdapter,
    ModelMismatch,
    InsufficientModelEvidence,
    BindingNotReady,
    FingerprintMismatch,
    CapabilityMismatch,
    SkillToolMismatch,
    PermissionMismatch,
    BillingMismatch,
    SpendGuaranteeUnavailable,
    CapacityUnavailable,
    ResourceMismatch,
    AdmissionBlocked,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileExclusion {
    pub target: RouteTarget,
    pub reasons: Vec<ExclusionReason>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EligibleCatalog {
    pub snapshot_digest: Digest,
    pub catalog_digest: Digest,
    pub eligible: Vec<ProfileCandidate>,
    pub exclusions: Vec<ProfileExclusion>,
}
/// Builds a deterministic complete-tuple catalog from caller-supplied local evidence.
/// Observation provenance/freshness must also be checked again by the launch gate.
pub fn build_eligible_catalog(
    snapshot: &RoutingSnapshot,
    candidates: &[ProfileCandidate],
) -> EligibleCatalog {
    let mut sorted = candidates.to_vec();
    sorted.sort_by_key(|c| (c.target(), canonical_digest(c, 1).ok()));
    let mut catalog = EligibleCatalog {
        snapshot_digest: canonical_digest(snapshot, 1)
            .expect("routing snapshots contain only serializable integers"),
        catalog_digest: canonical_digest(&sorted, 1)
            .expect("profile tuples contain only serializable integers"),
        eligible: Vec::new(),
        exclusions: Vec::new(),
    };
    for candidate in &sorted {
        let mut reasons = exclusion_reasons(snapshot, candidate);
        if sorted
            .iter()
            .filter(|other| other.target() == candidate.target())
            .count()
            != 1
        {
            reasons.push(ExclusionReason::InvalidContract);
        }
        if reasons.is_empty() {
            catalog.eligible.push(candidate.clone());
        } else {
            catalog.exclusions.push(ProfileExclusion {
                target: candidate.target(),
                reasons,
            });
        }
    }
    catalog
}

fn exclusion_reasons(
    snapshot: &RoutingSnapshot,
    candidate: &ProfileCandidate,
) -> Vec<ExclusionReason> {
    use ExclusionReason::*;
    let (p, b, o, a, t) = (
        &candidate.profile,
        &candidate.binding,
        &candidate.observation,
        &snapshot.authorization,
        &snapshot.task,
    );
    let mut reasons = Vec::new();
    if admission_blocker(snapshot).is_some() {
        reasons.push(AdmissionBlocked);
    }
    let pd = p.digest().ok();
    let bd = b.digest().ok();
    if pd.is_none()
        || bd.is_none()
        || o.schema_version != 1
        || p.id.0.is_empty()
        || b.id.0.is_empty()
        || p.revision == 0
        || b.revision == 0
        || p.harness_id.is_empty()
        || p.adapter_contract_version.is_empty()
        || !p.adapter_digest.is_valid()
        || !p.skill_tool_bundle_digest.is_valid()
        || p.requested_model.provider.is_empty()
        || p.requested_model.model.is_empty()
        || b.auth_owner.is_empty()
        || b.billing_source_id.0.is_empty()
        || b.endpoint_identity.is_empty()
        || b.trust_class.is_empty()
        || b.credential_reference
            .as_ref()
            .is_some_and(|r| r.is_empty())
        || !valid_executable(&o.executable)
        || !o
            .launch
            .as_ref()
            .is_some_and(|launch| launch.valid_for(p.backend))
    {
        reasons.push(InvalidContract);
    }
    let approved = a
        .allowed_profiles
        .iter()
        .find(|approved| approved.target == candidate.target());
    if approved.is_none_or(|approved| Some(&approved.profile_digest) != pd.as_ref()) {
        reasons.push(UnapprovedProfile);
    }
    if approved.is_none_or(|approved| Some(&approved.binding_digest) != bd.as_ref()) {
        reasons.push(UnapprovedBinding);
    }
    if pd.as_ref() != Some(&b.profile_digest)
        || pd.as_ref() != Some(&o.profile_digest)
        || bd.as_ref() != Some(&o.binding_digest)
    {
        reasons.push(BindingDigestMismatch);
    }
    if !o.dispatch_supported || p.backend == ExecutionBackend::Cloud {
        reasons.push(UnsupportedHarness);
    }
    if o.observed_at_ms > snapshot.now_ms
        || snapshot.now_ms >= o.expires_at_ms
        || o.expires_at_ms <= o.observed_at_ms
    {
        reasons.push(StaleObservation);
    }
    match &o.qualification {
        None => reasons.push(UnqualifiedAdapter),
        Some(q) => {
            if !q.receipt_digest.is_valid()
                || !q.tool_probe_passed
                || !q.cancellation_probe_passed
                || !q.quiescence_probe_passed
                || !q
                    .digest()
                    .ok()
                    .is_some_and(|digest| snapshot.qualification_digests.contains(&digest))
            {
                reasons.push(UnqualifiedAdapter);
            }
            if o.launch
                .as_ref()
                .and_then(|launch| qualification_fingerprint(p, b, &o.executable, launch).ok())
                .as_ref()
                != Some(&q.launch_fingerprint)
            {
                reasons.push(FingerprintMismatch);
            }
            if q.permission_profile != a.permission_profile
                || !t.required_egress.is_subset(&q.allowed_egress)
            {
                reasons.push(PermissionMismatch);
            }
            if !t.required_capabilities.is_subset(&q.capabilities) {
                reasons.push(CapabilityMismatch);
            }
            if !t.required_resources.is_subset(&q.capacity_pool_ids) {
                reasons.push(ResourceMismatch);
            }
        }
    }
    if o.requested_model != p.requested_model
        || o.reported_model
            .as_ref()
            .is_some_and(|m| m != &p.requested_model)
    {
        reasons.push(ModelMismatch);
    }
    if o.model_identity_level < a.minimum_model_identity
        || (o.model_identity_level > ModelIdentityLevel::Requested && o.reported_model.is_none())
    {
        reasons.push(InsufficientModelEvidence);
    }
    if o.auth_status != Readiness::Ready {
        reasons.push(BindingNotReady);
    }
    if !t.required_capabilities.is_subset(&p.capabilities) {
        reasons.push(CapabilityMismatch);
    }
    if p.skill_tool_bundle_digest != t.skill_tool_bundle_digest {
        reasons.push(SkillToolMismatch);
    }
    if t.permission_profile != a.permission_profile
        || !t.required_egress.is_subset(&a.allowed_egress)
    {
        reasons.push(PermissionMismatch);
    }
    if !a.allowed_billing_sources.contains(&b.billing_source_id)
        || !a.allowed_billing_modes.contains(&b.billing_mode)
    {
        reasons.push(BillingMismatch);
    }
    if a.limits.spend_guarantee == SpendGuarantee::Hard
        && (!o.hard_spend_limit_verified || a.limits.max_spend_nano_usd.is_none())
    {
        reasons.push(SpendGuaranteeUnavailable);
    }
    if !o.capacity_ready || snapshot.active_workers >= a.limits.max_workers {
        reasons.push(CapacityUnavailable);
    }
    if !t.required_resources.is_subset(&b.capacity_pool_ids) {
        reasons.push(ResourceMismatch);
    }
    reasons
}

fn admission_blocker(s: &RoutingSnapshot) -> Option<RouteBlocker> {
    let a = &s.authorization;
    if s.cancelled || s.cancel_epoch != a.cancel_epoch {
        return Some(RouteBlocker::Cancelled);
    }
    if !s.scope_valid
        || s.task_revision != s.task.revision
        || s.authorization_revision != a.revision
        || s.domain_id != a.domain_id
        || s.run_id != a.run_id
        || s.plan_id != a.plan_id
        || s.task.plan_digest != a.plan_digest
        || !s
            .task
            .digest()
            .ok()
            .is_some_and(|digest| a.allowed_task_digests.contains(&digest))
    {
        return Some(RouteBlocker::ScopeDrift);
    }
    if a.schema_version != 1
        || a.revision == 0
        || s.task.task_id.0.is_empty()
        || s.task.revision == 0
        || s.next_ordinal == 0
        || s.next_ordinal > 2
        || s.admitted_attempts < s.next_ordinal.saturating_sub(1)
        || s.task.goal.trim().is_empty()
        || !s.task.base.snapshot_digest.is_valid()
        || !mandatory_checks_valid(&s.task.checks)
        || s.task.claim_roots.is_empty()
    {
        return Some(RouteBlocker::InvalidContract);
    }
    if !s.dependencies_ready || !dependencies_match(&s.task, &s.resolved_dependencies) {
        return Some(RouteBlocker::FailedPrerequisite);
    }
    if !s.ownership_resolved {
        return Some(RouteBlocker::OwnershipUnresolved);
    }
    if s.next_ordinal == 1 && s.previous_attempt.is_some() {
        return Some(RouteBlocker::InvalidContract);
    }
    if s.next_ordinal == 2
        && !s.previous_attempt.as_ref().is_some_and(|previous| {
            previous.ordinal == 1
                && !previous.attempt_id.0.is_empty()
                && previous.state == AttemptState::Failed
                && matches!(
                    previous.failure_class,
                    AttemptFailureClass::Implementation | AttemptFailureClass::Check
                )
                && previous
                    .actionable_evidence_digest
                    .as_ref()
                    .is_some_and(Digest::is_valid)
        })
    {
        return Some(RouteBlocker::RepairNotAllowed);
    }
    if s.now_ms >= a.limits.deadline_ms
        || s.admitted_attempts >= a.limits.max_attempts
        || s.next_ordinal > a.limits.max_attempts
        || a.limits.max_workers == 0
        || a.limits.max_spend_nano_usd.is_some_and(|limit| {
            s.spent_and_reserved_nano_usd
                .is_none_or(|spent| spent >= limit)
        })
    {
        return Some(RouteBlocker::BudgetExhausted);
    }
    None
}
fn mandatory_checks_valid(checks: &[CheckRecipe]) -> bool {
    let mut ids = BTreeSet::new();
    !checks.is_empty()
        && checks.iter().all(|check| {
            !check.id.0.trim().is_empty()
                && check.recipe_digest.is_valid()
                && ids.insert(check.id.as_str())
        })
}
fn dependencies_match(task: &TaskContract, resolved: &[VerifiedDependencyOutput]) -> bool {
    let mut seen = BTreeSet::new();
    task.dependencies.len() == resolved.len()
        && task.dependencies.iter().zip(resolved).all(|(id, winner)| {
            *id == winner.task_id
                && *id != task.task_id
                && !id.0.is_empty()
                && seen.insert(id.0.as_str())
                && !winner.winning_attempt_id.0.is_empty()
                && winner.output_digest.is_valid()
                && winner.verification_receipt_digest.is_valid()
        })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteBlocker {
    Disabled,
    Cancelled,
    ScopeDrift,
    FailedPrerequisite,
    OwnershipUnresolved,
    BudgetExhausted,
    PreferredUnavailable,
    InvalidPolicy,
    StaleCatalog,
    InvalidContract,
    CapacityUnavailable,
    ConflictingRequirement,
    RepairNotAllowed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteSelection {
    Selected(RouteTarget),
    WaitForCapacity {
        target: RouteTarget,
        max_wait_ms: u64,
    },
    Blocked(RouteBlocker),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteReason {
    Blocked,
    Manual,
    Required,
    MechanicalEveryday,
    StrongDefault,
    AdviceEveryday,
    StrongRepair,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdviceStatus {
    NotUsed,
    InvalidOrStale,
    ShadowRecorded,
    Applied,
    RulesFallback,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDecision {
    /// The exact mutable TaskState revision Store must compare atomically at admission.
    pub task_state_revision: u64,
    pub selection: RouteSelection,
    pub reason: RouteReason,
    pub advice_status: AdviceStatus,
    pub snapshot_digest: Digest,
    pub catalog_digest: Digest,
    pub policy_digest: Digest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdviceChoice {
    EverydayFit,
    StrongNeeded,
    Unclear,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdviceDistribution {
    pub everyday_fit: f64,
    pub strong_needed: f64,
    pub unclear: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdviceUsageStatus {
    Known,
    Estimated,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdviceEnvelope {
    pub schema_version: u32,
    pub request_id: AdviceRequestId,
    pub packet_digest: Digest,
    pub policy_version: String,
    pub template_version: String,
    pub model_id: String,
    pub task_revision: u64,
    pub task_state_revision: u64,
    pub authorization_revision: u64,
    pub cancel_epoch: u64,
    pub consent_revision: u64,
    pub received_at_ms: u64,
    pub expires_at_ms: u64,
    pub packet_complete: bool,
    pub choice: AdviceChoice,
    pub distribution: AdviceDistribution,
    pub usage_status: AdviceUsageStatus,
    pub usage_receipt_id: Option<String>,
    pub elapsed_ms: u64,
}
/// Advice is observational data, and cannot modify eligibility or authorization.
/// Store must compare the returned snapshot/catalog/policy digests during admission.
pub fn select_route(
    snapshot: &RoutingSnapshot,
    catalog: &EligibleCatalog,
    policy: &RoutingPolicy,
    advice: Option<&AdviceEnvelope>,
) -> RouteDecision {
    let policy_digest = policy
        .digest()
        .unwrap_or_else(|_| Digest::of_bytes(b"invalid-policy"));
    let mut decision = RouteDecision {
        task_state_revision: snapshot.task_state_revision,
        selection: RouteSelection::Blocked(RouteBlocker::Disabled),
        reason: RouteReason::Blocked,
        advice_status: AdviceStatus::NotUsed,
        snapshot_digest: catalog.snapshot_digest.clone(),
        catalog_digest: catalog.catalog_digest.clone(),
        policy_digest: policy_digest.clone(),
    };
    let blocker = if policy.mode == RoutingMode::Disabled {
        Some(RouteBlocker::Disabled)
    } else if let Some(blocker) = admission_blocker(snapshot) {
        Some(blocker)
    } else if policy.schema_version != 1
        || policy.version.is_empty()
        || policy.everyday == policy.strong
        || policy.everyday.profile_id == policy.strong.profile_id
        || policy.everyday_threshold_ppm > 1_000_000
        || policy.unclear_ceiling_ppm > 1_000_000
        || policy_digest != snapshot.authorization.policy_digest
    {
        Some(RouteBlocker::InvalidPolicy)
    } else if canonical_digest(snapshot, 1).ok().as_ref() != Some(&catalog.snapshot_digest) {
        Some(RouteBlocker::StaleCatalog)
    } else {
        None
    };
    if let Some(blocker) = blocker {
        decision.selection = RouteSelection::Blocked(blocker);
        return decision;
    }
    if snapshot
        .manual_target
        .as_ref()
        .is_some_and(|target| target != &policy.everyday && target != &policy.strong)
        || snapshot
            .task
            .required_target
            .as_ref()
            .is_some_and(|target| target != &policy.everyday && target != &policy.strong)
    {
        decision.selection = RouteSelection::Blocked(RouteBlocker::ConflictingRequirement);
        return decision;
    }
    if snapshot.manual_target.as_ref().is_some_and(|manual| {
        snapshot
            .task
            .required_target
            .as_ref()
            .is_some_and(|required| required != manual)
            || ((snapshot.task.strong_only || snapshot.next_ordinal == 2)
                && *manual != policy.strong)
    }) || ((snapshot.task.strong_only || snapshot.next_ordinal == 2)
        && snapshot
            .task
            .required_target
            .as_ref()
            .is_some_and(|required| *required != policy.strong))
    {
        decision.selection = RouteSelection::Blocked(RouteBlocker::ConflictingRequirement);
        return decision;
    }
    let (target, reason) = if snapshot.next_ordinal == 2 {
        (policy.strong.clone(), RouteReason::StrongRepair)
    } else if let Some(target) = &snapshot.manual_target {
        (target.clone(), RouteReason::Manual)
    } else if let Some(target) = &snapshot.task.required_target {
        (target.clone(), RouteReason::Required)
    } else if snapshot.task.strong_only {
        (policy.strong.clone(), RouteReason::Required)
    } else if mechanical_everyday(&snapshot.task) {
        (policy.everyday.clone(), RouteReason::MechanicalEveryday)
    } else {
        (policy.strong.clone(), RouteReason::StrongDefault)
    };
    decision.reason = reason;
    decision.selection = select_target(snapshot, catalog, &target);
    if matches!(policy.mode, RoutingMode::Rules | RoutingMode::Disabled) {
        return decision;
    }
    let Some(advice) = advice else {
        return decision;
    };
    if !advice.is_valid_for(snapshot, policy) {
        decision.advice_status = AdviceStatus::InvalidOrStale;
        return decision;
    }
    if policy.mode == RoutingMode::Shadow {
        decision.advice_status = AdviceStatus::ShadowRecorded;
        return decision;
    }
    decision.advice_status = AdviceStatus::RulesFallback;
    let ambiguous_initial =
        snapshot.next_ordinal == 1 && decision.reason == RouteReason::StrongDefault;
    let live_gate = snapshot.authorization.live_advice_authorized
        && policy
            .evaluated_manifest_digest
            .as_ref()
            .is_some_and(Digest::is_valid)
        // The v1 packet has only reviewed coarse facts. An unknown kind or
        // unresolved cross-component need cannot justify a cheaper launch.
        && snapshot.task.task_kind == Some(TaskKind::Diagnosis)
        && snapshot.task.cross_component_requirement == Some(false);
    if ambiguous_initial
        && live_gate
        && advice.choice == AdviceChoice::EverydayFit
        && is_eligible(snapshot, catalog, &policy.everyday)
        && is_eligible(snapshot, catalog, &policy.strong)
        && advice.distribution.everyday_fit
            >= f64::from(policy.everyday_threshold_ppm) / 1_000_000.0
        && advice.distribution.unclear < f64::from(policy.unclear_ceiling_ppm) / 1_000_000.0
    {
        decision.selection = RouteSelection::Selected(policy.everyday.clone());
        decision.reason = RouteReason::AdviceEveryday;
        decision.advice_status = AdviceStatus::Applied;
    }
    decision
}

fn mechanical_everyday(task: &TaskContract) -> bool {
    matches!(
        task.task_kind,
        Some(
            TaskKind::Documentation
                | TaskKind::Formatting
                | TaskKind::Rename
                | TaskKind::LocalTransformation
        )
    ) && task
        .task_kind_evidence
        .as_ref()
        .is_some_and(Digest::is_valid)
        && task.context_complete
        && task.claim_roots.len() <= 2
        && task.dependencies.len() <= 1
        && !task.checks.is_empty()
        && task.cross_component_requirement == Some(false)
}
fn is_eligible(
    snapshot: &RoutingSnapshot,
    catalog: &EligibleCatalog,
    target: &RouteTarget,
) -> bool {
    let mut matches = catalog
        .eligible
        .iter()
        .filter(|candidate| candidate.target() == *target);
    let first = matches.next();
    first.is_some_and(|candidate| exclusion_reasons(snapshot, candidate).is_empty())
        && matches.next().is_none()
}
fn select_target(
    snapshot: &RoutingSnapshot,
    catalog: &EligibleCatalog,
    target: &RouteTarget,
) -> RouteSelection {
    if is_eligible(snapshot, catalog, target) {
        return RouteSelection::Selected(target.clone());
    }
    let capacity_only = catalog.exclusions.iter().any(|excluded| {
        excluded.target == *target && excluded.reasons == [ExclusionReason::CapacityUnavailable]
    });
    if capacity_only {
        RouteSelection::WaitForCapacity {
            target: target.clone(),
            max_wait_ms: 30_000.min(
                snapshot
                    .authorization
                    .limits
                    .deadline_ms
                    .saturating_sub(snapshot.now_ms),
            ),
        }
    } else {
        RouteSelection::Blocked(RouteBlocker::PreferredUnavailable)
    }
}
impl AdviceEnvelope {
    pub fn is_valid_for(&self, snapshot: &RoutingSnapshot, policy: &RoutingPolicy) -> bool {
        let d = &self.distribution;
        let probabilities = [d.everyday_fit, d.strong_needed, d.unclear];
        let chosen = match self.choice {
            AdviceChoice::EverydayFit => d.everyday_fit,
            AdviceChoice::StrongNeeded => d.strong_needed,
            AdviceChoice::Unclear => d.unclear,
        };
        self.schema_version == 1
            && self.policy_version == policy.version
            && self.template_version == policy.advice_template
            && self.model_id == policy.advice_model
            && Some(&self.request_id) == snapshot.advice_request_id.as_ref()
            && Some(&self.packet_digest) == snapshot.packet_digest.as_ref()
            && self.packet_digest.is_valid()
            && !self.request_id.0.is_empty()
            && !self.model_id.is_empty()
            && !self.template_version.is_empty()
            && self.task_revision == snapshot.task_revision
            && self.task_state_revision == snapshot.task_state_revision
            && self.authorization_revision == snapshot.authorization_revision
            && self.cancel_epoch == snapshot.cancel_epoch
            && self.consent_revision == snapshot.authorization.consent_revision
            && self.received_at_ms <= snapshot.now_ms
            && snapshot.now_ms < self.expires_at_ms
            && self.packet_complete
            && snapshot.has_recordable_advice_context()
            && probabilities
                .iter()
                .all(|p| p.is_finite() && (0.0..=1.0).contains(p))
            && (probabilities.iter().sum::<f64>() - 1.0).abs() <= 0.000_001
            && probabilities.iter().all(|p| chosen >= *p)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptState {
    Admitted,
    Preparing,
    Launching,
    Running,
    Sealing,
    Verifying,
    Passed,
    Failed,
    FailedNoLaunch,
    Cancelled,
    RecoveryRequired,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptFailureClass {
    Implementation,
    Check,
    Authentication,
    Environment,
    Quota,
    Cancellation,
    UnsupportedCapability,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepairEvidence {
    pub attempt_id: AttemptId,
    pub ordinal: u32,
    pub state: AttemptState,
    pub failure_class: AttemptFailureClass,
    pub actionable_evidence_digest: Option<Digest>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptTransitionEvidence {
    pub inputs_bound: bool,
    pub launch_checks_passed: bool,
    pub process_registered: bool,
    pub no_worker_created: bool,
    pub quiescent: bool,
    pub output_sealed: bool,
    pub checks_passed: bool,
    pub authority_current: bool,
    pub reconciliation_complete: bool,
}
impl AttemptState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Preparing => "preparing",
            Self::Launching => "launching",
            Self::Running => "running",
            Self::Sealing => "sealing",
            Self::Verifying => "verifying",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::FailedNoLaunch => "failed_no_launch",
            Self::Cancelled => "cancelled",
            Self::RecoveryRequired => "recovery_required",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "admitted" => Some(Self::Admitted),
            "preparing" => Some(Self::Preparing),
            "launching" => Some(Self::Launching),
            "running" => Some(Self::Running),
            "sealing" => Some(Self::Sealing),
            "verifying" => Some(Self::Verifying),
            "passed" => Some(Self::Passed),
            "failed" => Some(Self::Failed),
            "failed_no_launch" => Some(Self::FailedNoLaunch),
            "cancelled" => Some(Self::Cancelled),
            "recovery_required" => Some(Self::RecoveryRequired),
            _ => None,
        }
    }
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Passed | Self::Failed | Self::FailedNoLaunch | Self::Cancelled
        )
    }
    /// Evidence booleans are the controller's summary of durable receipts, never worker assertions.
    /// Recovery retains ownership and has no edge back to launching.
    pub fn transition(
        &mut self,
        to: Self,
        e: &AttemptTransitionEvidence,
    ) -> Result<(), ContractError> {
        use AttemptState::*;
        // Conflicting receipt summaries cannot establish either launch or no-launch.
        // The controller must reconcile the facts instead of releasing ownership.
        if e.no_worker_created && e.process_registered {
            return Err(ContractError::InvalidTransition { from: *self, to });
        }
        let allowed = match (*self, to) {
            (Admitted, Preparing) => e.inputs_bound,
            (Admitted, Cancelled) => true,
            (Admitted, FailedNoLaunch) => e.no_worker_created,
            (Preparing, Launching) => e.launch_checks_passed && e.authority_current,
            (Preparing, FailedNoLaunch | Cancelled) => e.no_worker_created,
            (Launching, Running) => e.process_registered,
            (Launching, FailedNoLaunch | Cancelled) => e.no_worker_created,
            (Launching | Running | Sealing | Verifying, RecoveryRequired) => true,
            (Running, Sealing | Cancelled) => e.quiescent,
            (Sealing, Verifying) => e.quiescent && e.output_sealed,
            (Sealing, Failed | Cancelled) => e.quiescent,
            (Verifying, Passed) => {
                e.quiescent && e.output_sealed && e.checks_passed && e.authority_current
            }
            (Verifying, Failed | Cancelled) => e.quiescent,
            (RecoveryRequired, Sealing | Failed | Cancelled) => {
                e.reconciliation_complete && e.quiescent
            }
            (RecoveryRequired, FailedNoLaunch) => e.reconciliation_complete && e.no_worker_created,
            _ => false,
        };
        if !allowed {
            return Err(ContractError::InvalidTransition { from: *self, to });
        }
        *self = to;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlobRef {
    pub digest: Digest,
    pub byte_length: u64,
}
impl BlobRef {
    pub fn verify(&self, bytes: &[u8]) -> Result<(), ContractError> {
        if bytes.len() as u64 != self.byte_length || Digest::of_bytes(bytes) != self.digest {
            return Err(invalid("blob digest or length mismatch"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffOrigin {
    pub domain_id: DomainId,
    pub run_id: RunId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub task_revision: u64,
    pub plan_id: PlanId,
    pub plan_digest: Digest,
    pub authorization_revision: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairInputTrust {
    Untrusted,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedRepairInput {
    pub failed_attempt_id: AttemptId,
    pub task_id: TaskId,
    pub scoped_change_manifest: BlobRef,
    pub trust: RepairInputTrust,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOperation {
    Add,
    Modify,
    Delete,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffChange {
    pub path: String,
    pub operation: ChangeOperation,
    pub preimage_digest: Option<Digest>,
    pub result: Option<BlobRef>,
    pub executable: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceProvenance {
    Observed,
    Claimed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    ProcessOutcome,
    Quiescence,
    CheckResult,
    Diagnostic,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffEvidence {
    pub kind: EvidenceKind,
    pub provenance: EvidenceProvenance,
    pub receipt: BlobRef,
    pub check_id: Option<CheckId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffRequirements {
    pub capabilities: BTreeSet<String>,
    pub skill_tool_bundle_digest: Digest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteProvenance {
    Claimed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimedWorkerNote {
    pub text: String,
    pub provenance: NoteProvenance,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableHandoffManifest {
    pub schema_version: u32,
    pub canonicalization_version: u32,
    pub manifest_digest: Digest,
    pub origin: HandoffOrigin,
    pub task_contract_ref: Digest,
    pub base: BaseSnapshot,
    pub dependencies: Vec<VerifiedDependencyOutput>,
    pub repair_input: Option<UntrustedRepairInput>,
    pub changes: Vec<HandoffChange>,
    pub evidence: Vec<HandoffEvidence>,
    pub requirements: HandoffRequirements,
    pub notes: Vec<ClaimedWorkerNote>,
}
impl PortableHandoffManifest {
    pub fn digest(&self) -> Result<Digest, ContractError> {
        if self.schema_version != 1 {
            return Err(ContractError::UnsupportedVersion);
        }
        let mut value = serde_json::to_value(self).map_err(json_error)?;
        value
            .as_object_mut()
            .ok_or_else(|| invalid("handoff is not an object"))?
            .remove("manifest_digest");
        canonical_digest(&value, self.canonicalization_version)
    }
    pub fn seal(mut self) -> Result<Self, ContractError> {
        self.manifest_digest = self.digest()?;
        Ok(self)
    }
    /// Validates portable metadata against trusted ledger inputs. The receiver must
    /// additionally verify every blob and reject reparse traversal/file kinds on disk.
    /// This method does not attest that an exported evidence receipt is true.
    pub fn validate(
        &self,
        task: &TaskContract,
        expected_origin: &HandoffOrigin,
        expected_dependencies: &[VerifiedDependencyOutput],
    ) -> Result<(), ContractError> {
        if self.digest()? != self.manifest_digest {
            return Err(invalid("handoff digest mismatch"));
        }
        if &self.origin != expected_origin
            || self.origin.task_id != task.task_id
            || self.origin.task_revision != task.revision
            || self.origin.plan_digest != task.plan_digest
            || self.task_contract_ref != task.digest()?
            || self.base != task.base
            || self.dependencies != expected_dependencies
            || !dependencies_match(task, expected_dependencies)
        {
            return Err(invalid(
                "handoff does not match the approved origin, task, base or dependencies",
            ));
        }
        if self.requirements.capabilities != task.required_capabilities
            || self.requirements.skill_tool_bundle_digest != task.skill_tool_bundle_digest
        {
            return Err(invalid("handoff requirements differ from the task"));
        }
        if let Some(repair) = &self.repair_input {
            if repair.task_id != task.task_id
                || repair.failed_attempt_id == self.origin.attempt_id
                || !repair.scoped_change_manifest.digest.is_valid()
            {
                return Err(invalid(
                    "repair input must name a different failed attempt in this task",
                ));
            }
        }
        if task.claim_roots.iter().any(|root| !portable_path(root)) {
            return Err(invalid("invalid task claim root"));
        }
        let mut paths = BTreeSet::new();
        for change in &self.changes {
            if !portable_path(&change.path)
                || protected_handoff_path(&change.path)
                || !paths.insert(change.path.to_lowercase())
                || !task.claim_roots.iter().any(|root| {
                    change.path == *root || change.path.starts_with(&format!("{root}/"))
                })
            {
                return Err(invalid(
                    "unsafe, duplicate, protected or unclaimed handoff path",
                ));
            }
            let shape_valid = match change.operation {
                ChangeOperation::Add => change.preimage_digest.is_none() && change.result.is_some(),
                ChangeOperation::Modify => {
                    change.preimage_digest.is_some() && change.result.is_some()
                }
                ChangeOperation::Delete => {
                    change.preimage_digest.is_some()
                        && change.result.is_none()
                        && !change.executable
                }
            };
            if !shape_valid
                || change
                    .preimage_digest
                    .as_ref()
                    .is_some_and(|d| !d.is_valid())
                || change.result.as_ref().is_some_and(|b| !b.digest.is_valid())
            {
                return Err(invalid("invalid change preimage/result"));
            }
        }
        if self.notes.len() > 32 || self.notes.iter().any(|note| note.text.len() > 4096) {
            return Err(invalid("worker notes exceed the bounded supplement"));
        }
        for evidence in &self.evidence {
            if !evidence.receipt.digest.is_valid()
                || evidence
                    .check_id
                    .as_ref()
                    .is_some_and(|id| !task.checks.iter().any(|check| check.id == *id))
                || (evidence.kind == EvidenceKind::CheckResult && evidence.check_id.is_none())
            {
                return Err(invalid(
                    "evidence references an unknown check or invalid blob",
                ));
            }
        }
        Ok(())
    }
}

fn portable_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path.split('/').all(|part| {
            let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            !part.is_empty()
                && part != "."
                && part != ".."
                && !part.ends_with(['.', ' '])
                && !part.contains(['<', '>', '"', '|', '?', '*'])
                && !matches!(
                    stem.as_str(),
                    "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
                )
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
}
fn protected_handoff_path(path: &str) -> bool {
    path.split('/').any(|part| {
        let part = part.to_ascii_lowercase();
        matches!(
            part.as_str(),
            ".git"
                | ".pytxo"
                | ".codex"
                | ".claude"
                | ".agents"
                | "agents.md"
                | "agents.override.md"
                | "claude.md"
                | "skill.md"
                | ".mcp.json"
                | "mcp.json"
                | ".env"
        ) || part.starts_with(".env.")
    })
}
