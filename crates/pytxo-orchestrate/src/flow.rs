//! Text-first Flow planning for one execution domain.
//!
//! Flow is an orchestration facade: the selected repository is canonicalized into a single
//! execution domain, while planner, scheduler, permission, Blast Shield, Race Shield, and ADE
//! checks remain in their owning layers. Desktop presents telemetry and intent; it never
//! enforces policy.

use std::collections::{BTreeSet, HashMap};
use std::path::{Component, Path};
use std::sync::Arc;

use anyhow::{bail, Context};
use chrono::Utc;
use pytxo_core::routing::{
    canonical_digest, AttemptState, BaseSnapshot, BillingSourceMode, CheckId, Digest,
    ExecutableIdentity, MissionAuthorization, RoutingMode, TaskContract,
};
use pytxo_core::{
    ade_can_dispatch, ade_on_path, all_ade_clis, resolve_ade, ExecutionBackend, PermissionProfile,
    PytxoConfig, Task, TaskId,
};
use pytxo_core::{DomainId, PytxoError, RunId};
use pytxo_planner::advisor::{
    hosted_scope_digest, request_template_identity, HOSTED_RECIPIENT, MODEL_ID, TEMPLATE_VERSION,
};
use pytxo_planner::{MissionSpec, PlannerContext};
use pytxo_runner::{registry_path, ProcessRegistryFile};
use pytxo_store::{
    capacity::{CapacityReleaseEvidence, CapacityReleaseEvidenceKind, CapacityReservationState},
    routing::{
        require_launchable_check_recipes, AdvisorSendPhase, CheckCwdKind, CheckPlatform,
        FrozenCheckExecutorV1, FrozenCheckRecipeV1, RoutedAttemptRecord, RoutedUsage,
        RoutingControlEvent, RoutingFacts, RoutingHistory, RoutingMission, RoutingReceipts,
        RoutingScope, StagedRoutingMissionRef, TaskRoutingState, TransitionRoutingAttempt,
    },
    routing_capacity_intent::{AdmittedNoLaunchRelease, CapacityIntentPhase},
    routing_checker::CheckerOwnershipPhase,
    routing_launch::{LaunchOwnershipPhase, LaunchSettlement, OwnedJobStopKind, OwnedJobStopPhase},
    routing_private::{
        CheckerNativeOutcome, ControllerObservation, ControllerReceiptEnvelope,
        PrivateArtifactClaim, PrivateArtifactKind, ReceiptSource,
    },
    Catalog, FlowDraftRecord, HostedAdvisorConsentFence, HostedAdvisorConsentReview,
    HostedGrantState, PytxoStore, RoutedFlowDispatchOwner,
};
use serde::{Deserialize, Serialize};

use crate::{
    clear_stopped_active_run_if_safe, dispatch_run_with_config_snapshot, ensure_repo_trusted,
    load_config_for_repo, plan_tasks, read_active_run_state, resolve_repo_root, ActiveRunGate,
    RunOptions,
};

#[path = "advisor_projection.rs"]
mod advisor_projection;
#[cfg(test)]
use advisor_projection::redacted_reviewed_task_description;
pub(crate) use advisor_projection::reviewed_task_advisor_packet;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowSource {
    Text,
    Voice,
}

impl FlowSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Voice => "voice",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowStatus {
    Draft,
    Transcribing,
    Planning,
    Ready,
    ReviewOnly,
    Blocked,
    Dispatching,
    Dispatched,
    Cancelled,
    Failed,
}

impl FlowStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Transcribing => "transcribing",
            Self::Planning => "planning",
            Self::Ready => "ready",
            Self::ReviewOnly => "review_only",
            Self::Blocked => "blocked",
            Self::Dispatching => "dispatching",
            Self::Dispatched => "dispatched",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FlowDraftInput {
    pub id: String,
    pub title: String,
    pub mission_text: String,
    pub source: FlowSource,
    pub domain_id: Option<String>,
    pub project_id: Option<String>,
    /// Optional ADE explicitly selected by Desktop. If selected, it must be registered and on PATH.
    pub ade_id: Option<String>,
    /// Explicit per-run concurrency; omitted CLI requests retain repository configuration.
    #[serde(default)]
    pub max_workers: Option<usize>,
    /// Additional operator-selected checks, validated by a fresh preview. Never edits config.
    #[serde(default)]
    pub verification_commands: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlowPlanTask {
    pub id: String,
    pub agent: String,
    pub prompt: String,
    pub paths: Vec<String>,
    pub dependencies: Vec<String>,
    pub root: Option<String>,
    #[serde(default)]
    pub verify: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlowWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FlowBlockedReason {
    CheckoutUnavailable {
        message: String,
    },
    InvalidRoot {
        message: String,
    },
    InvalidPath {
        task_id: String,
        path: String,
    },
    PermissionViolation {
        message: String,
    },
    AdeUnavailable {
        ade_id: String,
    },
    OverlappingPathClaims {
        task_a: String,
        task_b: String,
        paths: Vec<String>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlowAdeSummary {
    pub requested: Option<String>,
    pub available: bool,
    pub installed: Vec<String>,
    pub command: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FlowPlan {
    pub draft_id: String,
    pub domain_id: String,
    pub project_id: Option<String>,
    pub status: FlowStatus,
    pub tasks: Vec<FlowPlanTask>,
    pub waves: Vec<Vec<String>>,
    /// Zero marks a legacy preview that must be regenerated before dispatch.
    #[serde(default)]
    pub max_workers: usize,
    pub permission_profile: String,
    pub isolation_mode: String,
    pub isolation_backend_intent: String,
    pub execution_backend: String,
    pub ade: FlowAdeSummary,
    pub warnings: Vec<FlowWarning>,
    pub blocked_reasons: Vec<FlowBlockedReason>,
    pub estimated_tokens: Option<u64>,
    pub estimated_cost_usd: Option<f64>,
    pub previewed_at: String,
    /// Experimental Rust-owned contract. No legacy Flow endpoint may dispatch it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutedFlowReview>,
}

/// Public review data excludes private profile bindings and credential references.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoutedFlowReview {
    pub authorization: MissionAuthorization,
    pub mission_digest: Digest,
}

/// Bytes generated by the current advisor serializer from one reviewed task.
/// This preview is local data only; it grants no permission to disclose it.
/// V1 uses fixed wording and reviewed coarse categories; even those can
/// reveal task type. Inspection is not consent or hosted-routing readiness.
#[derive(Clone, Serialize)]
pub struct RoutedAdvisorPacketPreview {
    pub domain_id: String,
    pub run_id: String,
    pub task_id: String,
    pub reviewed_consent_revision: u64,
    pub recipient_identity: String,
    pub packet_digest: Digest,
    pub request_digest: Digest,
    pub request_body: Vec<u8>,
}

/// A read-only disclosure candidate derived from the same current, persisted
/// local Shadow review. The review authorizes only the no-network recipient;
/// this candidate has no hosted consent, request ID, token, or send authority.
/// `packet_body` is the proposed proxy packet, not the complete future wire
/// request or the server-generated Jev request body.
#[derive(Clone, Serialize)]
pub struct ProposedHostedAdvisorPacketPreview {
    pub domain_id: String,
    pub run_id: String,
    pub task_id: String,
    pub source_review_recipient_identity: String,
    pub recipient_identity: String,
    pub scope_digest: Digest,
    pub packet_digest: Digest,
    pub wire_schema_version: u32,
    pub decision_kind: String,
    pub question_set_version: String,
    pub packet_body: Vec<u8>,
}

/// Exact coarse packet from a persisted one-task review naming the hosted
/// recipient. This is disclosure data only; a separate account grant, local
/// consent and guarded client are required before a network send.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReviewedHostedAdvisorPacketPreview {
    pub domain_id: String,
    /// Physical Store file inspected for this review. Consent must pin the
    /// same file even if a replacement carries identical logical rows.
    pub store_db_file_identity: String,
    pub run_id: String,
    pub task_id: String,
    pub reviewed_consent_revision: u64,
    pub recipient_identity: String,
    pub scope_digest: Digest,
    pub packet_digest: Digest,
    pub request_digest: Digest,
    pub wire_schema_version: u32,
    pub decision_kind: String,
    pub question_set_version: String,
    pub packet_body: Vec<u8>,
    /// False means Core would reject any Shadow answer for the reviewed task.
    pub recordable_shadow_context: bool,
}

/// Local workspace grant for the currently reviewed no-network Shadow fixture.
/// It is not a hosted-service account, disclosure receipt, or Live authority.
#[derive(Clone, Debug, Serialize)]
pub struct RoutedAdvisorConsentStatus {
    pub domain_id: String,
    pub revision: u64,
    pub enabled: bool,
    pub current_scope: bool,
    pub recipient_identity: String,
    pub updated_at_ms: u64,
}

fn require_windows_advisor_consent() -> anyhow::Result<()> {
    if !cfg!(windows) {
        bail!("experimental routing grants require a Windows Store identity guard");
    }
    Ok(())
}

fn advisor_consent_location(
    catalog: &Catalog,
    domain_id: &str,
) -> anyhow::Result<(DomainId, std::path::PathBuf)> {
    if domain_id.is_empty() {
        bail!("advisor consent execution domain is empty");
    }
    if let Some(pinned) = catalog.routing_advisor_consent_store(domain_id)? {
        let path = std::path::PathBuf::from(pinned.store_db_path);
        if pytxo_runner::file_identity(&path)? != pinned.store_db_file_identity {
            bail!("advisor consent Store file identity changed");
        }
        return Ok((DomainId(domain_id.into()), path));
    }
    advisor_current_consent_location(domain_id)
}

fn advisor_current_consent_location(
    domain_id: &str,
) -> anyhow::Result<(DomainId, std::path::PathBuf)> {
    let repo = resolve_repo_root(Some(Path::new(domain_id)))?;
    if repo.to_string_lossy() != domain_id {
        bail!("advisor consent execution domain changed");
    }
    let cfg = load_config_for_repo(None, &repo)?;
    Ok((DomainId(domain_id.into()), cfg.db_path_at(&repo)))
}

fn advisor_consent_status(
    domain_id: &DomainId,
    consent: pytxo_store::routing::RoutingAdvisorConsent,
) -> RoutedAdvisorConsentStatus {
    RoutedAdvisorConsentStatus {
        domain_id: domain_id.0.clone(),
        revision: consent.revision,
        enabled: consent.enabled,
        current_scope: consent.enabled
            && consent.scope_digest == Some(reviewed_routing_advisor_disclosure_scope_digest()),
        recipient_identity: REVIEWED_ADVISOR_RECIPIENT.into(),
        updated_at_ms: consent.updated_at_ms,
    }
}

fn hosted_advisor_consent_status(
    domain_id: &DomainId,
    consent: pytxo_store::routing::RoutingAdvisorConsent,
) -> RoutedAdvisorConsentStatus {
    RoutedAdvisorConsentStatus {
        domain_id: domain_id.0.clone(),
        revision: consent.revision,
        enabled: consent.enabled,
        current_scope: consent.enabled && consent.scope_digest == Some(hosted_scope_digest()),
        recipient_identity: HOSTED_RECIPIENT.into(),
        updated_at_ms: consent.updated_at_ms,
    }
}

pub fn read_experimental_routed_advisor_consent(
    catalog: &Catalog,
    domain_id: &str,
) -> anyhow::Result<RoutedAdvisorConsentStatus> {
    require_windows_advisor_consent()?;
    let (domain, db_path) = advisor_consent_location(catalog, domain_id)?;
    #[cfg(windows)]
    let store_file_guard = pytxo_runner::FileIdentityGuard::acquire(&db_path)?;
    let store = PytxoStore::open_existing_read_only(&db_path)?;
    if let Some(pinned) = catalog.routing_advisor_consent_store(domain_id)? {
        #[cfg(windows)]
        if store_file_guard.identity() != pinned.store_db_file_identity {
            bail!("advisor consent Store file identity changed during open");
        }
        if pytxo_runner::file_identity(&db_path)? != pinned.store_db_file_identity {
            bail!("advisor consent Store file identity changed during open");
        }
    }
    Ok(advisor_consent_status(
        &domain,
        store.routing_advisor_consent(&domain)?,
    ))
}

/// The caller must present the exact request digest and recipient it just
/// displayed. The controller regenerates both from the persisted reviewed
/// mission before changing the workspace grant. This enables no transport.
pub fn enable_experimental_routed_advisor_consent(
    catalog: &Catalog,
    draft_id: &str,
    expected_domain_id: &str,
    expected_request_digest: &str,
    expected_recipient_identity: &str,
    expected_revision: u64,
) -> anyhow::Result<RoutedAdvisorConsentStatus> {
    require_windows_advisor_consent()?;
    let preview = preview_experimental_routed_advisor_packet(catalog, draft_id)?;
    if preview.domain_id != expected_domain_id
        || preview.request_digest.0 != expected_request_digest
        || preview.recipient_identity != expected_recipient_identity
        || preview.recipient_identity != REVIEWED_ADVISOR_RECIPIENT
        || expected_revision.checked_add(1) != Some(preview.reviewed_consent_revision)
    {
        bail!("advisor grant differs from the inspected Shadow packet");
    }
    let (domain, db_path) = advisor_current_consent_location(expected_domain_id)?;
    ensure_repo_trusted(Path::new(expected_domain_id))?;
    #[cfg(windows)]
    let store_file_guard = pytxo_runner::FileIdentityGuard::acquire(&db_path)?;
    let store = PytxoStore::open_existing_read_write(&db_path)?;
    let canonical_db_path = std::fs::canonicalize(&db_path)?;
    let store_file_identity = pytxo_runner::file_identity(&db_path)?;
    #[cfg(windows)]
    if store_file_guard.identity() != store_file_identity {
        bail!("advisor consent Store file identity changed during open");
    }
    if store.routing_advisor_consent(&domain)?.revision != expected_revision {
        bail!("advisor grant has a stale workspace revision");
    }
    let fresh = preview_experimental_routed_advisor_packet(catalog, draft_id)?;
    if fresh.domain_id != preview.domain_id
        || fresh.run_id != preview.run_id
        || fresh.task_id != preview.task_id
        || fresh.reviewed_consent_revision != preview.reviewed_consent_revision
        || fresh.recipient_identity != preview.recipient_identity
        || fresh.packet_digest != preview.packet_digest
        || fresh.request_digest != preview.request_digest
        || fresh.request_body != preview.request_body
    {
        bail!("advisor packet changed before workspace grant");
    }
    catalog.bind_routing_advisor_consent_store(
        expected_domain_id,
        canonical_db_path
            .to_str()
            .context("advisor consent Store path is not UTF-8")?,
        &store_file_identity,
    )?;
    let now_ms = u64::try_from(Utc::now().timestamp_millis())?;
    let consent = store.set_routing_advisor_consent(
        &domain,
        expected_revision,
        true,
        Some(reviewed_routing_advisor_disclosure_scope_digest()),
        now_ms,
    )?;
    Ok(advisor_consent_status(&domain, consent))
}

/// Revocation needs only the canonical workspace and the last seen revision;
/// a stale or already-dispatched review cannot prevent it.
pub fn revoke_experimental_routed_advisor_consent(
    catalog: &Catalog,
    domain_id: &str,
    expected_revision: u64,
) -> anyhow::Result<RoutedAdvisorConsentStatus> {
    require_windows_advisor_consent()?;
    let (domain, db_path) = advisor_consent_location(catalog, domain_id)?;
    #[cfg(windows)]
    let store_file_guard = pytxo_runner::FileIdentityGuard::acquire(&db_path)?;
    let store = PytxoStore::open_existing_read_write(&db_path)?;
    if let Some(pinned) = catalog.routing_advisor_consent_store(domain_id)? {
        #[cfg(windows)]
        if store_file_guard.identity() != pinned.store_db_file_identity {
            bail!("advisor consent Store file identity changed during open");
        }
        if pytxo_runner::file_identity(&db_path)? != pinned.store_db_file_identity {
            bail!("advisor consent Store file identity changed during open");
        }
    }
    let now_ms = u64::try_from(Utc::now().timestamp_millis())?;
    let consent =
        store.set_routing_advisor_consent(&domain, expected_revision, false, None, now_ms)?;
    Ok(advisor_consent_status(&domain, consent))
}

/// Read the local hosted-recipient consent. This has no Link grant status and
/// therefore cannot be interpreted as permission for an outbound request.
pub fn read_experimental_hosted_advisor_local_consent(
    catalog: &Catalog,
    domain_id: &str,
) -> anyhow::Result<RoutedAdvisorConsentStatus> {
    require_windows_advisor_consent()?;
    let (domain, db_path) = advisor_consent_location(catalog, domain_id)?;
    #[cfg(windows)]
    let store_file_guard = pytxo_runner::FileIdentityGuard::acquire(&db_path)?;
    let store = PytxoStore::open_existing_read_only(&db_path)?;
    let pinned = catalog.routing_advisor_consent_store(domain_id)?;
    if let Some(pinned) = &pinned {
        #[cfg(windows)]
        if store_file_guard.identity() != pinned.store_db_file_identity {
            bail!("hosted consent Store file identity changed during open");
        }
        if pytxo_runner::file_identity(&db_path)? != pinned.store_db_file_identity {
            bail!("hosted consent Store file identity changed during open");
        }
    }
    let consent = store.routing_hosted_advisor_consent(&domain, HOSTED_RECIPIENT)?;
    if consent.enabled {
        let pinned = pinned.context("hosted consent Store has no reviewed Flow identity pin")?;
        let fence = catalog
            .hosted_advisor_consent_fence(domain_id)?
            .context("enabled hosted consent has no current authority fence")?;
        if !fence.enabled
            || fence.recipient_identity != HOSTED_RECIPIENT
            || fence.consent_revision != consent.revision
            || fence.store_db_file_identity != pinned.store_db_file_identity
        {
            bail!("enabled hosted consent differs from its authority fence");
        }
        let receipt = catalog
            .hosted_advisor_consent_review(domain_id)?
            .context("enabled hosted consent has no reviewed Flow receipt")?;
        if receipt.recipient_identity != HOSTED_RECIPIENT
            || receipt.consent_revision != consent.revision
            || receipt.scope_digest != hosted_scope_digest().0
            || receipt.store_db_file_identity != pinned.store_db_file_identity
            || consent.scope_digest != Some(hosted_scope_digest())
        {
            bail!("enabled hosted consent differs from its reviewed Flow receipt");
        }
        let preview = current_hosted_advisor_packet_for_status(catalog, &receipt.draft_id)?;
        if preview.domain_id != domain_id
            || preview.recipient_identity != receipt.recipient_identity
            || preview.reviewed_consent_revision != receipt.consent_revision
            || preview.scope_digest.0 != receipt.scope_digest
            || preview.packet_digest.0 != receipt.packet_digest
            || preview.request_digest.0 != receipt.request_digest
            || preview.store_db_file_identity != receipt.store_db_file_identity
        {
            bail!("enabled hosted consent review changed after grant");
        }
    } else if pinned.is_none() && consent.revision != 0 {
        bail!("hosted consent Store has no reviewed Flow identity pin");
    }
    Ok(hosted_advisor_consent_status(&domain, consent))
}

/// Stage only the local half of hosted consent from a persisted hosted review.
/// The future send controller must separately reconcile the Link grant and
/// token, then rederive these packet bytes under a fresh physical Store guard.
pub fn enable_experimental_hosted_advisor_local_consent(
    catalog: &Catalog,
    draft_id: &str,
    inspected: &ReviewedHostedAdvisorPacketPreview,
    expected_revision: u64,
) -> anyhow::Result<RoutedAdvisorConsentStatus> {
    require_windows_advisor_consent()?;
    let preview = preview_reviewed_hosted_advisor_packet(catalog, draft_id)?;
    if &preview != inspected
        || preview.recipient_identity != HOSTED_RECIPIENT
        || expected_revision.checked_add(1) != Some(preview.reviewed_consent_revision)
    {
        bail!("hosted local consent differs from the inspected Shadow packet");
    }
    if !preview.recordable_shadow_context {
        bail!("hosted Shadow review needs a complete, classified task before workspace opt-in");
    }
    if catalog
        .hosted_grant(&inspected.domain_id)?
        .is_some_and(|grant| grant.state != HostedGrantState::Revoked)
    {
        bail!("revoke the existing hosted Link grant before enabling a new consent");
    }
    let (domain, db_path) = advisor_current_consent_location(&inspected.domain_id)?;
    ensure_repo_trusted(Path::new(&inspected.domain_id))?;
    #[cfg(windows)]
    let store_file_guard = pytxo_runner::FileIdentityGuard::acquire(&db_path)?;
    let store = PytxoStore::open_existing_read_write(&db_path)?;
    let canonical_db_path = std::fs::canonicalize(&db_path)?;
    let store_file_identity = pytxo_runner::file_identity(&db_path)?;
    #[cfg(windows)]
    if store_file_guard.identity() != store_file_identity {
        bail!("hosted consent Store file identity changed during open");
    }
    if store_file_identity != inspected.store_db_file_identity {
        bail!("hosted consent Store differs from the inspected Shadow packet");
    }
    if store
        .routing_hosted_advisor_consent(&domain, HOSTED_RECIPIENT)?
        .revision
        != expected_revision
    {
        bail!("hosted local consent has a stale workspace revision");
    }
    if preview_reviewed_hosted_advisor_packet(catalog, draft_id)? != preview {
        bail!("hosted packet changed before local consent");
    }
    let now_ms = u64::try_from(Utc::now().timestamp_millis())?;
    let consent = store.set_routing_hosted_advisor_consent(
        &domain,
        HOSTED_RECIPIENT,
        expected_revision,
        true,
        Some(hosted_scope_digest()),
        now_ms,
    )?;
    catalog.bind_routing_advisor_consent_store(
        &inspected.domain_id,
        canonical_db_path
            .to_str()
            .context("hosted consent Store path is not UTF-8")?,
        &store_file_identity,
    )?;
    catalog.record_hosted_advisor_consent_review(&HostedAdvisorConsentReview {
        domain_id: inspected.domain_id.clone(),
        recipient_identity: inspected.recipient_identity.clone(),
        draft_id: draft_id.into(),
        consent_revision: consent.revision,
        scope_digest: inspected.scope_digest.0.clone(),
        packet_digest: inspected.packet_digest.0.clone(),
        request_digest: inspected.request_digest.0.clone(),
        store_db_file_identity: store_file_identity.clone(),
    })?;
    catalog.advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
        domain_id: inspected.domain_id.clone(),
        recipient_identity: inspected.recipient_identity.clone(),
        consent_revision: consent.revision,
        enabled: true,
        store_db_file_identity: store_file_identity.clone(),
    })?;
    let verified = read_experimental_hosted_advisor_local_consent(catalog, &inspected.domain_id)?;
    if verified.revision != consent.revision || !verified.enabled || !verified.current_scope {
        bail!("hosted local consent changed before grant receipt");
    }
    Ok(verified)
}

/// Fence local hosted requests before any later remote-revoke reconciliation.
/// No Link grant is created or revoked by this local-only operation.
pub fn revoke_experimental_hosted_advisor_local_consent(
    catalog: &Catalog,
    domain_id: &str,
    expected_revision: u64,
) -> anyhow::Result<RoutedAdvisorConsentStatus> {
    require_windows_advisor_consent()?;
    let (domain, db_path) = advisor_consent_location(catalog, domain_id)?;
    #[cfg(windows)]
    let store_file_guard = pytxo_runner::FileIdentityGuard::acquire(&db_path)?;
    let store = PytxoStore::open_existing_read_write(&db_path)?;
    let pinned = catalog.routing_advisor_consent_store(domain_id)?;
    if let Some(pinned) = &pinned {
        #[cfg(windows)]
        if store_file_guard.identity() != pinned.store_db_file_identity {
            bail!("hosted consent Store file identity changed during open");
        }
        if pytxo_runner::file_identity(&db_path)? != pinned.store_db_file_identity {
            bail!("hosted consent Store file identity changed during open");
        }
    }
    let unpinned_identity = if pinned.is_none() {
        let store_file_identity = pytxo_runner::file_identity(&db_path)?;
        #[cfg(windows)]
        if store_file_guard.identity() != store_file_identity {
            bail!("hosted consent Store file identity changed during open");
        }
        let canonical_db_path = std::fs::canonicalize(&db_path)?;
        Some((canonical_db_path, store_file_identity))
    } else {
        None
    };
    let current = store.routing_hosted_advisor_consent(&domain, HOSTED_RECIPIENT)?;
    let consent = if current.revision == expected_revision {
        let now_ms = u64::try_from(Utc::now().timestamp_millis())?;
        store.set_routing_hosted_advisor_consent(
            &domain,
            HOSTED_RECIPIENT,
            expected_revision,
            false,
            None,
            now_ms,
        )?
    } else if current.revision == expected_revision.saturating_add(1)
        && !current.enabled
        && current.scope_digest.is_none()
    {
        // The Store commit can complete before the Catalog fence is written.
        // Reuse that exact disabled revision under the original file guard.
        current
    } else {
        bail!("stale hosted advisor consent revision");
    };
    if let Some((canonical_db_path, store_file_identity)) = unpinned_identity {
        catalog.bind_routing_advisor_consent_store(
            domain_id,
            canonical_db_path
                .to_str()
                .context("hosted consent Store path is not UTF-8")?,
            &store_file_identity,
        )?;
    }
    let store_file_identity = pytxo_runner::file_identity(&db_path)?;
    #[cfg(windows)]
    if store_file_guard.identity() != store_file_identity {
        bail!("hosted consent Store file identity changed before revoke fence");
    }
    catalog.advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
        domain_id: domain_id.into(),
        recipient_identity: HOSTED_RECIPIENT.into(),
        consent_revision: consent.revision,
        enabled: false,
        store_db_file_identity: store_file_identity,
    })?;
    Ok(hosted_advisor_consent_status(&domain, consent))
}

/// Inspect a persisted, ready Shadow review without claiming a run, recording
/// consent, creating a journal entry, or invoking an advisor transport.
pub fn preview_experimental_routed_advisor_packet(
    catalog: &Catalog,
    draft_id: &str,
) -> anyhow::Result<RoutedAdvisorPacketPreview> {
    Ok(validated_routed_advisor_packet(catalog, draft_id)?.0)
}

/// Preview the exact coarse packet fields currently accepted by the proposed
/// hosted proxy contract. This does not turn the local review into a hosted
/// grant and deliberately omits the future client request ID.
pub fn preview_proposed_hosted_advisor_packet(
    catalog: &Catalog,
    draft_id: &str,
) -> anyhow::Result<ProposedHostedAdvisorPacketPreview> {
    let (local, packet_body, _, _) = validated_routed_advisor_packet(catalog, draft_id)?;
    if Digest::of_bytes(&packet_body) != local.packet_digest {
        bail!("proposed hosted packet differs from reviewed coarse packet");
    }
    Ok(ProposedHostedAdvisorPacketPreview {
        domain_id: local.domain_id,
        run_id: local.run_id,
        task_id: local.task_id,
        source_review_recipient_identity: local.recipient_identity,
        recipient_identity: HOSTED_RECIPIENT.into(),
        scope_digest: hosted_scope_digest(),
        packet_digest: local.packet_digest,
        wire_schema_version: 1,
        decision_kind: "initial_demand".into(),
        question_set_version: TEMPLATE_VERSION.into(),
        packet_body,
    })
}

/// Inspect a hosted-recipient review. This does not create a Link grant,
/// issue a token, prepare a journal request or perform a network operation.
pub fn preview_reviewed_hosted_advisor_packet(
    catalog: &Catalog,
    draft_id: &str,
) -> anyhow::Result<ReviewedHostedAdvisorPacketPreview> {
    let (reviewed, packet_body, store_db_file_identity, recordable_shadow_context) =
        validated_routed_advisor_packet_for(
            catalog,
            draft_id,
            Some(HOSTED_RECIPIENT),
            AdvisorPacketStage::Review,
        )?;
    if Digest::of_bytes(&packet_body) != reviewed.packet_digest {
        bail!("reviewed hosted packet bytes changed");
    }
    Ok(ReviewedHostedAdvisorPacketPreview {
        domain_id: reviewed.domain_id,
        store_db_file_identity,
        run_id: reviewed.run_id,
        task_id: reviewed.task_id,
        reviewed_consent_revision: reviewed.reviewed_consent_revision,
        recipient_identity: reviewed.recipient_identity,
        scope_digest: hosted_scope_digest(),
        packet_digest: reviewed.packet_digest,
        request_digest: reviewed.request_digest,
        wire_schema_version: 1,
        decision_kind: "initial_demand".into(),
        question_set_version: TEMPLATE_VERSION.into(),
        packet_body,
        recordable_shadow_context,
    })
}

/// Rebuild the reviewed packet only while this process owns the exact claimed
/// Run. A draft preview is deliberately insufficient once dispatch has begun.
pub(crate) fn validated_claimed_hosted_advisor_packet(
    catalog: &Catalog,
    draft_id: &str,
) -> anyhow::Result<ReviewedHostedAdvisorPacketPreview> {
    let (reviewed, packet_body, store_db_file_identity, recordable_shadow_context) =
        validated_routed_advisor_packet_for(
            catalog,
            draft_id,
            Some(HOSTED_RECIPIENT),
            AdvisorPacketStage::ActiveClaim,
        )?;
    if Digest::of_bytes(&packet_body) != reviewed.packet_digest {
        bail!("claimed hosted packet bytes changed");
    }
    Ok(ReviewedHostedAdvisorPacketPreview {
        domain_id: reviewed.domain_id,
        store_db_file_identity,
        run_id: reviewed.run_id,
        task_id: reviewed.task_id,
        reviewed_consent_revision: reviewed.reviewed_consent_revision,
        recipient_identity: reviewed.recipient_identity,
        scope_digest: hosted_scope_digest(),
        packet_digest: reviewed.packet_digest,
        request_digest: reviewed.request_digest,
        wire_schema_version: 1,
        decision_kind: "initial_demand".into(),
        question_set_version: TEMPLATE_VERSION.into(),
        packet_body,
        recordable_shadow_context,
    })
}

/// Consent status may still refer to the immutable review after its single
/// dispatch was consumed. This read-only path never qualifies a send.
fn current_hosted_advisor_packet_for_status(
    catalog: &Catalog,
    draft_id: &str,
) -> anyhow::Result<ReviewedHostedAdvisorPacketPreview> {
    if let Ok(reviewed) = preview_reviewed_hosted_advisor_packet(catalog, draft_id) {
        return Ok(reviewed);
    }
    let (reviewed, packet_body, store_db_file_identity, recordable_shadow_context) =
        validated_routed_advisor_packet_for(
            catalog,
            draft_id,
            Some(HOSTED_RECIPIENT),
            AdvisorPacketStage::ConsumedReview,
        )?;
    if Digest::of_bytes(&packet_body) != reviewed.packet_digest {
        bail!("consumed hosted packet bytes changed");
    }
    Ok(ReviewedHostedAdvisorPacketPreview {
        domain_id: reviewed.domain_id,
        store_db_file_identity,
        run_id: reviewed.run_id,
        task_id: reviewed.task_id,
        reviewed_consent_revision: reviewed.reviewed_consent_revision,
        recipient_identity: reviewed.recipient_identity,
        scope_digest: hosted_scope_digest(),
        packet_digest: reviewed.packet_digest,
        request_digest: reviewed.request_digest,
        wire_schema_version: 1,
        decision_kind: "initial_demand".into(),
        question_set_version: TEMPLATE_VERSION.into(),
        packet_body,
        recordable_shadow_context,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AdvisorPacketStage {
    Review,
    ActiveClaim,
    ConsumedReview,
}

fn validated_routed_advisor_packet(
    catalog: &Catalog,
    draft_id: &str,
) -> anyhow::Result<(RoutedAdvisorPacketPreview, Vec<u8>, String, bool)> {
    validated_routed_advisor_packet_for(catalog, draft_id, None, AdvisorPacketStage::Review)
}

fn validated_routed_advisor_packet_for(
    catalog: &Catalog,
    draft_id: &str,
    expected_recipient: Option<&str>,
    stage: AdvisorPacketStage,
) -> anyhow::Result<(RoutedAdvisorPacketPreview, Vec<u8>, String, bool)> {
    if stage != AdvisorPacketStage::Review && expected_recipient != Some(HOSTED_RECIPIENT) {
        bail!("claimed packet requires the hosted Shadow recipient");
    }
    let draft = catalog
        .get_flow_draft(draft_id)?
        .with_context(|| format!("Flow draft not found: {draft_id}"))?;
    let review_status = if expected_recipient == Some(HOSTED_RECIPIENT) {
        FlowStatus::ReviewOnly
    } else {
        FlowStatus::Ready
    };
    let valid_status = match stage {
        AdvisorPacketStage::Review => {
            draft.status == review_status.as_str() && draft.dispatched_run_id.is_none()
        }
        AdvisorPacketStage::ActiveClaim => draft.status == FlowStatus::Dispatching.as_str(),
        AdvisorPacketStage::ConsumedReview => matches!(
            draft.status.as_str(),
            "dispatching" | "dispatched" | "cancelled" | "failed" | "recovery_required"
        ),
    };
    if !valid_status {
        bail!("advisor packet preview requires a persisted Flow review");
    }
    let plan: FlowPlan = serde_json::from_str(
        draft
            .plan_json
            .as_deref()
            .context("advisor packet preview requires a persisted ready Flow plan")?,
    )
    .context("invalid persisted Flow preview")?;
    let [task] = plan.tasks.as_slice() else {
        bail!("advisor packet preview requires exactly one reviewed task");
    };
    if plan.draft_id != draft.id
        || draft.domain_id.as_deref() != Some(plan.domain_id.as_str())
        || draft.project_id != plan.project_id
        || plan.status != review_status
        || !plan.blocked_reasons.is_empty()
        || plan.max_workers != 1
        || plan.waves != [vec![task.id.clone()]]
        || !task.dependencies.is_empty()
        || plan.permission_profile != PermissionProfile::Orbit.as_str()
    {
        bail!("advisor packet preview differs from the ready Flow review");
    }
    let review = plan
        .routing
        .as_ref()
        .context("advisor packet preview requires an experimental routed review")?;
    if review.authorization.plan_digest != routed_plan_digest(&plan)?
        || review.authorization.live_advice_authorized
    {
        bail!("advisor packet preview differs from routed authority");
    }
    if stage != AdvisorPacketStage::Review
        && draft.dispatched_run_id.as_deref() != Some(review.authorization.run_id.0.as_str())
    {
        bail!("claimed hosted packet RunId differs from the review");
    }
    let repo = resolve_repo_root(Some(Path::new(&plan.domain_id)))?;
    if repo.to_string_lossy() != plan.domain_id {
        bail!("advisor packet preview execution domain changed");
    }
    let cfg = load_config_for_repo(None, &repo)?;
    if cfg.requested_permission_profile != Some(PermissionProfile::Orbit)
        || cfg.permission_profile != PermissionProfile::Orbit
        || cfg.max_agents < 1
        || cfg.isolation.as_str() != plan.isolation_mode
        || pytxo_runner::effective_isolation_mode(&cfg).as_str() != plan.isolation_backend_intent
        || format!("{:?}", cfg.execution_backend).to_ascii_lowercase() != plan.execution_backend
        || cfg.resolve_profile_for_agent(&task.agent) != PermissionProfile::Orbit
    {
        bail!("advisor packet preview execution policy changed");
    }
    validate_dispatch_task_plan(
        &runtime_tasks(&plan),
        &cfg,
        &plan.waves,
        plan.project_id.as_deref(),
    )?;
    let store_path = cfg.db_path_at(&repo);
    #[cfg(windows)]
    let store_file_guard = pytxo_runner::FileIdentityGuard::acquire(&store_path)?;
    let store = PytxoStore::open_existing_read_only(&store_path)?;
    let store_db_file_identity = pytxo_runner::file_identity(&store_path)?;
    #[cfg(windows)]
    if store_file_guard.identity() != store_db_file_identity {
        bail!("advisor packet Store file identity changed during open");
    }
    if stage != AdvisorPacketStage::Review {
        let owner = catalog
            .routed_flow_dispatch_owner(draft_id, &review.authorization.run_id.0)?
            .context("claimed hosted packet has no routed owner")?;
        let pid = std::process::id();
        let process_identity = pytxo_runner::process_start_identity(pid)?
            .context("claimed hosted controller identity is unavailable")?;
        let canonical_store = std::fs::canonicalize(&store_path)?;
        if (stage == AdvisorPacketStage::ActiveClaim
            && (owner.controller_pid != pid || owner.controller_start_identity != process_identity))
            || owner.store_db_path != canonical_store.to_string_lossy()
            || owner.store_db_file_identity.as_deref() != Some(store_db_file_identity.as_str())
        {
            bail!("claimed hosted packet owner, Store, Run, or Stop state changed");
        }
        if stage == AdvisorPacketStage::ActiveClaim {
            if catalog.routed_flow_stop_requested(draft_id, &review.authorization.run_id.0)?
                != Some(false)
                || store
                    .get_run_status(&review.authorization.run_id.0)?
                    .as_ref()
                    .is_none_or(|(status, _)| status != "starting")
            {
                bail!("claimed hosted Run is stopped or not starting");
            }
            let active_path = repo.join(&cfg.data_dir).join("active_run.json");
            let active = read_active_run_state(&active_path)?
                .context("claimed hosted Run has no active owner marker")?;
            if active.run_id != review.authorization.run_id.0
                || active.supervisor_pid != owner.controller_pid
                || active.supervisor_start_identity.as_deref()
                    != Some(owner.controller_start_identity.as_str())
                || active.repo_root != repo.to_string_lossy()
            {
                bail!("claimed hosted Run active owner changed");
            }
        }
    }
    let mission = load_reviewed_staged_mission(&store, &plan)?;
    if mission.policy.mode != RoutingMode::Shadow
        || mission.policy.advice_model != MODEL_ID
        || mission.policy.advisor_recipient.as_deref() != expected_recipient
    {
        bail!("advisor packet preview requires the current Shadow advisor contract");
    }
    if stage == AdvisorPacketStage::ActiveClaim {
        let scope = RoutingScope {
            domain_id: review.authorization.domain_id.clone(),
            run_id: review.authorization.run_id.clone(),
        };
        let history = store
            .routing_history(&scope)?
            .context("claimed hosted mission is not registered")?;
        if history.mission != mission || history.cancelled {
            bail!("claimed hosted mission changed or was cancelled");
        }
    }
    let [registered] = mission.tasks.as_slice() else {
        bail!("advisor packet preview requires exactly one private task");
    };
    let packet = reviewed_task_advisor_packet(&registered.contract)?;
    let packet_body = serde_json::to_vec(&packet)?;
    let request_body = packet
        .request_body()
        .map_err(|reason| anyhow::anyhow!("advisor request body unavailable: {reason:?}"))?;
    let request_digest = Digest::of_bytes(&request_body);
    let (recipient_identity, expected_scope, expected_template) = match expected_recipient {
        None => (
            REVIEWED_ADVISOR_RECIPIENT,
            reviewed_routing_advisor_disclosure_scope_digest(),
            reviewed_routing_advisor_identity_for_task(&registered.contract)?,
        ),
        Some(HOSTED_RECIPIENT) => (
            HOSTED_RECIPIENT,
            hosted_scope_digest(),
            reviewed_hosted_routing_advisor_identity_for_task(&registered.contract)?,
        ),
        Some(_) => bail!("unsupported reviewed advisor recipient"),
    };
    if mission.policy.disclosure_scope_digest != Some(expected_scope)
        || mission.policy.advice_template != expected_template
    {
        bail!("advisor request bytes differ from the reviewed Shadow policy");
    }
    let result = RoutedAdvisorPacketPreview {
        domain_id: review.authorization.domain_id.0.clone(),
        run_id: review.authorization.run_id.0.clone(),
        task_id: registered.contract.task_id.0.clone(),
        reviewed_consent_revision: review.authorization.consent_revision,
        recipient_identity: recipient_identity.into(),
        packet_digest: packet.digest(),
        request_digest,
        request_body,
    };
    if catalog.get_flow_draft(draft_id)?.as_ref() != Some(&draft) {
        bail!("Flow draft changed during advisor packet preview");
    }
    if pytxo_runner::file_identity(&store_path)? != store_db_file_identity {
        bail!("advisor packet Store file identity changed during preview");
    }
    Ok((
        result,
        packet_body,
        store_db_file_identity,
        registered.contract.has_recordable_advice_context(),
    ))
}

/// The only reviewed recipient in the current Shadow fixture has no network
/// transport. A future sponsored service must introduce its own reviewed
/// recipient identity and explicit user-facing disclosure action.
pub const REVIEWED_ADVISOR_RECIPIENT: &str = "pytxo-local-advisor-fixture/no-network/v1";

/// Identity of the wire serializer, complete trusted packet projection source,
/// and exact recipient. Shadow missions also append the one-task request digest.
pub fn reviewed_routing_advisor_identity() -> String {
    let projection_source = include_str!("advisor_projection.rs").replace("\r\n", "\n");
    format!(
        "{}:projection:{}:recipient:{}",
        request_template_identity(),
        Digest::of_bytes(projection_source.as_bytes()).0,
        Digest::of_bytes(REVIEWED_ADVISOR_RECIPIENT.as_bytes()).0
    )
}

/// One workspace grant can cover tasks using the same reviewed recipient,
/// packet projection and wire template. Exact task bytes remain separately
/// frozen in `advice_template` for each Shadow review.
pub fn reviewed_routing_advisor_disclosure_scope_digest() -> Digest {
    Digest::of_bytes(reviewed_routing_advisor_identity().as_bytes())
}

/// The complete request identity frozen into the reviewed one-task Shadow
/// policy. A code change that alters even task-specific outbound bytes now
/// requires a new review, even when the fixed template probe is unchanged.
pub fn reviewed_routing_advisor_identity_for_task(task: &TaskContract) -> anyhow::Result<String> {
    let body = reviewed_task_advisor_packet(task)?
        .request_body()
        .map_err(|reason| anyhow::anyhow!("advisor request body unavailable: {reason:?}"))?;
    Ok(format!(
        "{}:{}",
        reviewed_routing_advisor_identity(),
        Digest::of_bytes(&body).0
    ))
}

/// Bind a hosted review to its exact recipient/scope, proxy packet and
/// provider request body. A future client must still rederive these bytes
/// from the persisted mission at the send boundary.
pub fn reviewed_hosted_routing_advisor_identity_for_task(
    task: &TaskContract,
) -> anyhow::Result<String> {
    let packet = reviewed_task_advisor_packet(task)?;
    let request_body = packet
        .request_body()
        .map_err(|reason| anyhow::anyhow!("hosted advisor request body unavailable: {reason:?}"))?;
    Ok(format!(
        "pytxo-hosted-routing-request/v1:{}:{}:{}",
        hosted_scope_digest().0,
        packet.digest().0,
        Digest::of_bytes(&request_body).0
    ))
}

/// Build a reviewed routed contract with a RunId allocated by Pytxo before review.
/// The builder is a trusted in-process controller seam, never a Desktop payload.
pub fn preview_experimental_routed_flow(
    catalog: &Catalog,
    input: FlowDraftInput,
    builder: impl FnOnce(&FlowPlan, &RunId, &Digest) -> anyhow::Result<RoutingMission>,
) -> anyhow::Result<FlowPlan> {
    if input.ade_id.is_some() {
        bail!(
            "experimental routed Flow selects profiles in its reviewed contract, not a legacy ADE"
        );
    }
    preview_flow_scoped(catalog, input, false, Some(Box::new(builder)))
}

/// Dispatches only the explicitly opted-in, reviewed local fixture route.
/// Other profiles remain blocked until their owned adapter is qualified.
pub fn dispatch_experimental_routed_flow(
    catalog: &Catalog,
    draft_id: &str,
) -> anyhow::Result<String> {
    dispatch_flow_scoped(catalog, draft_id, false, true, None)
}

/// Offline, explicitly opted-in hosted Shadow experiment. The supplied client
/// is injected by the test harness; no production transport or paid send is
/// enabled by this entrypoint.
#[cfg(feature = "routed-test-faults")]
#[doc(hidden)]
pub fn dispatch_experimental_hosted_shadow_with_client(
    catalog: &Catalog,
    draft_id: &str,
    client: Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>,
) -> anyhow::Result<String> {
    if std::env::var("PYTXO_TEST_HOSTED_SHADOW_DISPATCH").as_deref() != Ok("1") {
        bail!("hosted Shadow fake-client dispatch is not opted in");
    }
    dispatch_flow_scoped(catalog, draft_id, false, true, Some(client))
}

/// Persist an exact reviewed Stop before touching an active run. The return
/// only acknowledges the request; the dispatch controller or exact Stop must
/// still prove native quiescence and a terminal state.
#[derive(Debug)]
pub struct RoutedStopTarget {
    pub(crate) repo: std::path::PathBuf,
    pub(crate) data_dir: std::path::PathBuf,
    pub(crate) store_path: std::path::PathBuf,
    pub(crate) store_file_identity: String,
}

pub fn request_stop_experimental_routed_flow(
    catalog: &Catalog,
    draft_id: &str,
    run_id: &str,
) -> anyhow::Result<Option<RoutedStopTarget>> {
    request_stop_experimental_routed_flow_after_review(catalog, draft_id, run_id, || {})
}

#[cfg(feature = "routed-test-faults")]
#[doc(hidden)]
pub fn request_stop_experimental_routed_flow_with_test_race(
    catalog: &Catalog,
    draft_id: &str,
    run_id: &str,
    after_review: impl FnOnce(),
) -> anyhow::Result<Option<RoutedStopTarget>> {
    request_stop_experimental_routed_flow_after_review(catalog, draft_id, run_id, after_review)
}

fn request_stop_experimental_routed_flow_after_review(
    catalog: &Catalog,
    draft_id: &str,
    run_id: &str,
    after_review: impl FnOnce(),
) -> anyhow::Result<Option<RoutedStopTarget>> {
    let draft = catalog
        .get_flow_draft(draft_id)?
        .with_context(|| format!("Flow draft not found: {draft_id}"))?;
    let plan_json = draft
        .plan_json
        .as_deref()
        .context("routed Stop has no reviewed Flow plan")?;
    let plan: FlowPlan = serde_json::from_str(plan_json)?;
    let review = plan
        .routing
        .as_ref()
        .context("routed Stop requires a reviewed route")?;
    if plan.draft_id != draft_id
        || draft.domain_id.as_deref() != Some(plan.domain_id.as_str())
        || review.authorization.run_id.0 != run_id
        || review.authorization.domain_id.as_str() != plan.domain_id
    {
        bail!("routed Stop does not match the reviewed Flow and RunId");
    }
    after_review();
    // The conditional reviewed cancellation and routed dispatch claim are both
    // Catalog writes. If dispatch wins after our initial read, this CAS loses
    // and we must follow the newly claimed owner through its launch gate.
    let cancelled = match plan.status {
        FlowStatus::Ready => catalog.cancel_ready_routed_flow(draft_id, plan_json)?,
        FlowStatus::ReviewOnly => {
            catalog.cancel_review_only_hosted_shadow_flow(draft_id, plan_json)?
        }
        _ => bail!("routed Stop requires a reviewed ready or hosted Shadow Flow"),
    };
    if cancelled {
        return Ok(None);
    }
    let current = catalog
        .get_flow_draft(draft_id)?
        .context("routed Stop Flow disappeared after the Ready race")?;
    if current.status == FlowStatus::Cancelled.as_str()
        && current.plan_json.as_deref() == Some(plan_json)
    {
        return Ok(None);
    }
    if current.status != FlowStatus::Dispatching.as_str()
        || current.dispatched_run_id.as_deref() != Some(run_id)
        || current.plan_json.as_deref() != Some(plan_json)
    {
        bail!("routed Stop cannot target this Flow state or RunId");
    }
    let repo = resolve_repo_root(Some(Path::new(&plan.domain_id)))?;
    let owner = catalog
        .routed_flow_dispatch_owner(draft_id, run_id)?
        .context("routed Stop cannot resolve its original dispatch owner")?;
    let identity = owner
        .store_db_file_identity
        .context("routed Stop cannot resolve its original Store identity")?;
    let store_path = std::path::PathBuf::from(owner.store_db_path);
    if !store_path.is_absolute() || store_path.file_name() != Some(std::ffi::OsStr::new("pytxo.db"))
    {
        bail!("routed Stop cannot resolve its original Store locator");
    }
    let store_guard = pytxo_runner::FileIdentityGuard::acquire(&store_path)
        .context("routed Stop cannot pin its original Store")?;
    if store_guard.identity() != identity {
        bail!("routed Stop original Store identity changed");
    }
    let data_dir = store_path
        .parent()
        .context("routed Stop original Store directory is missing")?
        .to_path_buf();
    let active_path = data_dir.join("active_run.json");
    // A Stop acknowledgement and native CreateProcess must have one order.
    // The original gate remains pinned even if configuration changed.
    let _stop_launch_gate = ActiveRunGate::acquire(&active_path)?;
    if !catalog.request_routed_flow_stop(draft_id, plan_json, run_id)? {
        bail!("routed Stop cannot target this Flow state or RunId");
    }
    if let Some(active) = crate::read_active_run_state(&active_path)? {
        if active.run_id != run_id
            || active.repo_root != repo.to_string_lossy()
            || active.supervisor_pid != owner.controller_pid
            || active.supervisor_start_identity.as_deref()
                != Some(owner.controller_start_identity.as_str())
        {
            bail!("routed Stop was recorded but another run owns the execution domain");
        }
        return Ok(Some(RoutedStopTarget {
            repo,
            data_dir,
            store_path,
            store_file_identity: identity,
        }));
    }
    Ok(None)
}

type RoutedMissionBuilder<'a> =
    Box<dyn FnOnce(&FlowPlan, &RunId, &Digest) -> anyhow::Result<RoutingMission> + 'a>;

fn routed_git_command(repo: &Path) -> anyhow::Result<std::process::Command> {
    let mut command = pytxo_core::background_command("git");
    let inert_hooks =
        std::env::temp_dir().join(format!("pytxo-disabled-git-hooks-{}", RunId::new().0));
    if !inert_hooks.is_absolute() || inert_hooks.exists() {
        bail!("cannot isolate routed Git hooks");
    }
    let hook_config = format!("core.hooksPath={}", inert_hooks.to_string_lossy());
    command
        .args(["-c", &hook_config, "-c", "core.fsmonitor=false"])
        .current_dir(repo);
    // A caller's Git environment must not redirect these observations to a
    // different repository, index, namespace, or object database.
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("GIT_")
        {
            command.env_remove(key);
        }
    }
    Ok(command)
}

fn routed_git_oid(repo: &Path, revision: &str) -> anyhow::Result<String> {
    let output = routed_git_command(repo)?
        .args(["rev-parse", "--verify", revision])
        .output()?;
    if !output.status.success() {
        bail!("routed Git base snapshot is unavailable");
    }
    let oid = String::from_utf8(output.stdout)?.trim().to_owned();
    if !matches!(oid.len(), 40 | 64)
        || !oid
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("routed Git base snapshot has an invalid object identity");
    }
    Ok(oid)
}

/// Observe the tracked Git base for an experimental routed mission builder.
/// The digest identifies the Git tree object, not the physical worktree bytes.
/// A launcher must use the reviewed commit as its worktree start point and
/// verify its exact inputs again before admitting a native process.
pub fn observe_experimental_routed_git_base(repo: &Path) -> anyhow::Result<BaseSnapshot> {
    let repo = resolve_repo_root(Some(repo))?;
    let root = routed_git_command(&repo)?
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !root.status.success()
        || crate::resolve_repo_root(Some(Path::new(String::from_utf8(root.stdout)?.trim())))?
            != repo
    {
        bail!("routed Git base snapshot belongs to a different repository");
    }
    let commit = routed_git_oid(&repo, "HEAD^{commit}")?;
    let tree = routed_git_oid(&repo, "HEAD^{tree}")?;
    if commit.len() != tree.len() {
        bail!("routed Git base snapshot mixes object formats");
    }
    let status = routed_git_command(&repo)?
        .args([
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            ".",
            ":(exclude).pytxo",
            ":(exclude).pytxo/**",
        ])
        .output()?;
    if !status.status.success() || !status.stdout.is_empty() {
        bail!("routed Git base snapshot requires a clean primary checkout");
    }
    if routed_git_oid(&repo, "HEAD^{commit}")? != commit
        || routed_git_oid(&repo, "HEAD^{tree}")? != tree
    {
        bail!("routed Git base snapshot changed while observing");
    }
    let base = BaseSnapshot {
        repository_identity: repo.to_string_lossy().into_owned(),
        git_revision: commit,
        snapshot_digest: canonical_digest(&(1_u32, "git-head-tree", tree), 1)?,
    };
    // Worker inputs are exact Git blobs, while the current Review/Apply layer
    // uses physical primary-checkout preimages. Reject transformed checkouts
    // before a routed draft can reach account qualification or worker launch.
    let cfg = load_config_for_repo(None, &repo)?;
    pytxo_runner::require_exact_reviewed_checkout(&repo, &base, &cfg.blast.sparse_exclude)?;
    Ok(base)
}

fn validate_final_routed_checks(commands: &[String]) -> anyhow::Result<()> {
    if commands.is_empty()
        || commands.len() > 16
        || commands.iter().any(|command| {
            command.is_empty()
                || command != command.trim()
                || command.len() > 4096
                || command.chars().any(char::is_control)
        })
    {
        bail!("experimental routed Flow requires 1 to 16 exact, single-line verification commands of at most 4096 bytes");
    }
    let mut seen = std::collections::BTreeSet::new();
    if commands.iter().any(|command| !seen.insert(command)) {
        bail!("experimental routed Flow has duplicate final verification commands");
    }
    Ok(())
}

fn observe_local_check_executor() -> anyhow::Result<FrozenCheckExecutorV1> {
    // This experimental no-worker path pins a fixed local shell file. It never
    // resolves through PATH or a user-supplied ComSpec. Other Windows system
    // locations require a new reviewed preview once explicitly supported.
    let (platform, path, shell_args) = if cfg!(windows) {
        (
            CheckPlatform::Windows,
            std::path::PathBuf::from(r"C:\Windows\System32\cmd.exe"),
            vec!["/D".into(), "/C".into()],
        )
    } else {
        (
            CheckPlatform::Posix,
            std::fs::canonicalize("/bin/sh")?,
            vec!["-c".into()],
        )
    };
    let metadata = std::fs::symlink_metadata(&path)?;
    if !metadata.file_type().is_file() {
        bail!("routed local check shell is not a regular file");
    }
    let bytes = std::fs::read(&path)?;
    let shell = ExecutableIdentity {
        path: path.to_string_lossy().into_owned(),
        version: "pinned-file-sha256-v1".into(),
        digest: Digest::of_bytes(&bytes),
    };
    Ok(FrozenCheckExecutorV1 {
        policy_version: 1,
        platform,
        shell,
        shell_args,
        cwd_kind: CheckCwdKind::FreshSealedVerificationView,
        permission_profile: PermissionProfile::Orbit,
        stdin_closed: true,
        environment_policy_version: 1,
        // Intent only. Orbit's network enforcement is advisory, not an OS
        // socket fence. A future owned checker must record the actual receipt.
        network_policy_version: 1,
        timeout_ms: 120_000,
        max_stdout_bytes: 262_144,
        max_stderr_bytes: 262_144,
    })
}

/// Trusted builder helper. The private result is inert until exact Flow,
/// mission, permission and process gates independently admit it.
pub fn freeze_experimental_routed_checks(
    task_id: &str,
    commands: &[String],
) -> anyhow::Result<Vec<FrozenCheckRecipeV1>> {
    validate_final_routed_checks(commands)?;
    if task_id.trim().is_empty() || task_id != task_id.trim() {
        bail!("invalid routed task identity for verification checks");
    }
    let executor = observe_local_check_executor()?;
    let recipes = commands
        .iter()
        .enumerate()
        .map(|(index, command)| FrozenCheckRecipeV1 {
            schema_version: 1,
            id: CheckId(format!("{task_id}:verify:{:04}", index + 1)),
            ordinal: u32::try_from(index + 1).expect("routed check list is bounded to 16"),
            command: command.clone(),
            executor: executor.clone(),
        })
        .collect::<Vec<_>>();
    for recipe in &recipes {
        recipe.reference()?;
    }
    Ok(recipes)
}

fn routed_plan_digest(plan: &FlowPlan) -> anyhow::Result<Digest> {
    Ok(canonical_digest(
        &(
            &plan.domain_id,
            &plan.project_id,
            &plan.tasks,
            &plan.waves,
            plan.max_workers,
            &plan.permission_profile,
            &plan.isolation_mode,
            &plan.isolation_backend_intent,
            &plan.execution_backend,
        ),
        1,
    )?)
}

/// Bounded routed fixtures support one task, one dependent pair, or two
/// independent tasks with disjoint claims in one reviewed wave.
pub(crate) fn routed_parallel_siblings(plan: &FlowPlan) -> bool {
    let [first, second] = plan.tasks.as_slice() else {
        return false;
    };
    plan.max_workers == 2
        && plan.waves.as_slice() == [vec![first.id.clone(), second.id.clone()]]
        && first.dependencies.is_empty()
        && second.dependencies.is_empty()
        && first.paths.iter().all(|left| {
            second
                .paths
                .iter()
                .all(|right| !pytxo_scheduler::paths_overlap(left, right))
        })
}

fn validate_routed_local_wave_shape(plan: &FlowPlan) -> anyhow::Result<()> {
    let sequential = plan.tasks.len() == plan.waves.len()
        && plan
            .tasks
            .iter()
            .zip(&plan.waves)
            .all(|(task, wave)| wave.len() == 1 && wave[0] == task.id);
    let dependencies = match plan.tasks.as_slice() {
        [only] => only.dependencies.is_empty(),
        [first, second] => {
            first.dependencies.is_empty() && second.dependencies == [first.id.as_str()]
        }
        _ => false,
    };
    if !(routed_parallel_siblings(plan) || (sequential && dependencies && plan.max_workers == 1)) {
        bail!("experimental routed Flow supports one task, two dependent waves, or two disjoint siblings");
    }
    Ok(())
}

/// A reviewed Claude route has a deliberately smaller shape than the local
/// scheduling fixture. This checks intent only: it does not qualify an account,
/// grant a native launch, or make either profile Ready.
pub(crate) fn validate_one_task_claude_route_shape(
    plan: &FlowPlan,
    mission: &RoutingMission,
) -> anyhow::Result<()> {
    use crate::routed_claude::{
        claude_proposal_arguments, CLAUDE_PROPOSAL_ADAPTER_ID, CLAUDE_PROPOSAL_TOOL_BUNDLE_ID,
        CLAUDE_SUBSCRIPTION_ENDPOINT,
    };

    // The provider transport is explicit reviewed egress. This is policy
    // identity, not an OS socket restriction on the native Claude process.
    let provider_egress = BTreeSet::from([CLAUDE_SUBSCRIPTION_ENDPOINT.to_owned()]);

    let [planned] = plan.tasks.as_slice() else {
        bail!("Claude routing requires exactly one reviewed task");
    };
    let [task] = mission.tasks.as_slice() else {
        bail!("Claude routing requires exactly one registered task");
    };
    if plan.waves.as_slice() != [vec![planned.id.clone()]]
        || plan.max_workers != 1
        || plan.permission_profile != PermissionProfile::Orbit.as_str()
        || plan.execution_backend != "subprocess"
        || plan.isolation_backend_intent != "worktree"
        || !planned.dependencies.is_empty()
        || !task.contract.dependencies.is_empty()
        || task.contract.claim_roots.len() != 1
        || task.contract.permission_profile != PermissionProfile::Orbit
        || task.contract.required_target.is_some()
        || task.contract.task_kind.is_none()
        || task
            .contract
            .task_kind_evidence
            .as_ref()
            .is_none_or(|evidence| !evidence.is_valid())
        || task.contract.strong_only
            != (task.contract.task_kind == Some(pytxo_core::routing::TaskKind::Architecture)
                || task.contract.cross_component_requirement == Some(true))
        || task.contract.required_capabilities != BTreeSet::from(["edit".into()])
        || task.contract.required_egress != provider_egress
        || task.contract.skill_tool_bundle_digest
            != Digest::of_bytes(CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes())
        || !matches!(
            mission.policy.mode,
            RoutingMode::Rules | RoutingMode::Shadow
        )
        || !matches!(
            (
                mission.policy.mode,
                mission.policy.advisor_recipient.as_deref()
            ),
            (RoutingMode::Rules, None) | (RoutingMode::Shadow, Some(HOSTED_RECIPIENT))
        )
        || mission.authorization.live_advice_authorized
        || mission.authorization.permission_profile != PermissionProfile::Orbit
        || mission.authorization.limits.max_workers != 1
        || !matches!(
            (
                mission.authorization.limits.max_attempts,
                mission.policy.version.as_str(),
                mission.policy.mode,
            ),
            (1, "claude-proposal-rules-v1", RoutingMode::Rules)
                | (1, "claude-proposal-hosted-shadow-v1", RoutingMode::Shadow)
                | (2, "claude-proposal-rules-repair-v1", RoutingMode::Rules)
        )
        || (mission.authorization.limits.max_attempts == 2
            && !crate::routed_fixture::claude_repair_opted_in())
        || mission.authorization.minimum_model_identity
            != pytxo_core::routing::ModelIdentityLevel::Requested
        || mission.authorization.limits.spend_guarantee
            != pytxo_core::routing::SpendGuarantee::RiskBounded
        || mission.authorization.allowed_egress != provider_egress
        || mission.authorization.allowed_billing_modes
            != BTreeSet::from([BillingSourceMode::Subscription])
    {
        bail!("Claude route exceeds the one-task subscription experiment");
    }
    let [everyday, strong] = mission.profiles.as_slice() else {
        bail!("Claude route requires one everyday and one strong profile");
    };
    let expected = [
        (everyday, &mission.policy.everyday, "haiku"),
        (strong, &mission.policy.strong, "sonnet"),
    ];
    let shared_source = &everyday.binding.billing_source_id;
    let shared_pool = &everyday.binding.capacity_pool_ids;
    if shared_source.0.is_empty()
        || shared_pool.len() != 1
        || task.contract.required_resources != *shared_pool
        || mission.authorization.allowed_billing_sources != BTreeSet::from([shared_source.clone()])
    {
        bail!("Claude route has no single reviewed subscription account pool");
    }
    for (registered, target, model) in expected {
        let profile = &registered.profile;
        let binding = &registered.binding;
        if profile.id != target.profile_id
            || binding.id != target.binding_id
            || profile.schema_version != 1
            || profile.canonicalization_version != 1
            || profile.revision == 0
            || profile.harness_id != "claude"
            || profile.adapter_contract_version != "1"
            || profile.adapter_digest != Digest::of_bytes(CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes())
            || profile.skill_tool_bundle_digest
                != Digest::of_bytes(CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes())
            || profile.backend != ExecutionBackend::Subprocess
            || profile.capabilities != BTreeSet::from(["read".into(), "edit".into()])
            || profile.requested_model.provider != "anthropic"
            || profile.requested_model.model != model
            || profile.requested_model.reasoning.is_some()
            || profile.requested_model.revision.is_some()
            || binding.schema_version != 1
            || binding.canonicalization_version != 1
            || binding.revision == 0
            || binding.profile_digest != profile.digest()?
            || binding.billing_mode != BillingSourceMode::Subscription
            || binding.billing_source_id != *shared_source
            || binding.capacity_pool_ids != *shared_pool
            || binding.credential_reference.is_some()
            || binding.auth_owner != "Claude"
            || binding.endpoint_identity != CLAUDE_SUBSCRIPTION_ENDPOINT
            || binding.trust_class != "vendor"
            || claude_proposal_arguments(&profile.requested_model, &task.contract.claim_roots)
                .is_err()
        {
            bail!("Claude route profile or account binding differs from its reviewed adapter");
        }
    }
    Ok(())
}

fn review_routed_mission(
    plan: &FlowPlan,
    run_id: &RunId,
    plan_digest: &Digest,
    mission: &RoutingMission,
) -> anyhow::Result<RoutedFlowReview> {
    let authorization = &mission.authorization;
    validate_routed_local_wave_shape(plan)?;
    if mission.profiles.iter().any(|registered| {
        registered.profile.harness_id == "claude"
            || registered.profile.adapter_digest
                == Digest::of_bytes(crate::routed_claude::CLAUDE_ADAPTER_ID.as_bytes())
            || registered.profile.adapter_digest
                == Digest::of_bytes(crate::routed_claude::CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes())
            || registered.profile.skill_tool_bundle_digest
                == Digest::of_bytes(crate::routed_claude::CLAUDE_TOOL_BUNDLE_ID.as_bytes())
            || registered.profile.skill_tool_bundle_digest
                == Digest::of_bytes(crate::routed_claude::CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes())
            || registered.profile.requested_model.provider == "anthropic"
            || registered.binding.auth_owner == "Claude"
            || registered.binding.endpoint_identity
                == crate::routed_claude::CLAUDE_SUBSCRIPTION_ENDPOINT
    }) {
        validate_one_task_claude_route_shape(plan, mission)?;
    }
    if authorization.run_id != *run_id
        || authorization.domain_id.as_str() != plan.domain_id
        || authorization.plan_digest != *plan_digest
        || authorization.permission_profile.as_str() != plan.permission_profile
        || authorization.limits.max_workers != u32::try_from(plan.max_workers)?
        || authorization.policy_digest != mission.policy.digest()?
        || authorization.live_advice_authorized
        || !matches!(
            mission.policy.mode,
            RoutingMode::Rules | RoutingMode::Shadow
        )
    {
        bail!("routed mission authority differs from the reviewed Flow scope");
    }
    if mission.tasks.len() != plan.tasks.len() {
        bail!("routed mission task count differs from the reviewed Flow");
    }
    if mission.policy.mode != RoutingMode::Shadow && mission.policy.advisor_recipient.is_some() {
        bail!("hosted advisor recipient requires a Shadow review");
    }
    if mission.policy.mode == RoutingMode::Shadow {
        let [registered] = mission.tasks.as_slice() else {
            bail!("experimental Shadow review requires exactly one task");
        };
        let reviewed_advisor_contract = match mission.policy.advisor_recipient.as_deref() {
            None => {
                mission.policy.disclosure_scope_digest
                    == Some(reviewed_routing_advisor_disclosure_scope_digest())
                    && mission.policy.advice_template
                        == reviewed_routing_advisor_identity_for_task(&registered.contract)?
            }
            Some(HOSTED_RECIPIENT) => {
                mission.policy.disclosure_scope_digest == Some(hosted_scope_digest())
                    && mission.policy.advice_template
                        == reviewed_hosted_routing_advisor_identity_for_task(&registered.contract)?
            }
            Some(_) => false,
        };
        if mission.policy.advice_model != MODEL_ID || !reviewed_advisor_contract {
            bail!("experimental Shadow review does not bind the current exact request");
        }
    }
    let observed_base = observe_experimental_routed_git_base(Path::new(&plan.domain_id))?;
    let mut task_digests = std::collections::BTreeSet::new();
    for (reviewed_task, registered_task) in plan.tasks.iter().zip(&mission.tasks) {
        let task = &registered_task.contract;
        let expected_recipes =
            freeze_experimental_routed_checks(&reviewed_task.id, &reviewed_task.verify)?;
        let expected_checks = expected_recipes
            .iter()
            .map(FrozenCheckRecipeV1::reference)
            .collect::<pytxo_core::Result<Vec<_>>>()?;
        if task.base != observed_base {
            bail!("routed Git base snapshot differs from the reviewed task");
        }
        if task.task_id.0 != reviewed_task.id
            || task.plan_digest != *plan_digest
            || task.goal != reviewed_task.prompt
            || task.claim_roots != reviewed_task.paths
            || task.dependencies
                != reviewed_task
                    .dependencies
                    .iter()
                    .cloned()
                    .map(pytxo_core::TaskId)
                    .collect::<Vec<_>>()
            || task.checks != expected_checks
            || registered_task.check_recipes != expected_recipes
            || task.permission_profile != authorization.permission_profile
        {
            bail!("routed task contract differs from the reviewed Flow task");
        }
        task_digests.insert(task.digest()?);
    }
    if authorization.allowed_task_digests != task_digests {
        bail!("routed task digests differ from reviewed authority");
    }
    require_launchable_check_recipes(mission)?;
    if mission.profiles.len() != 2 || authorization.allowed_profiles.len() != 2 {
        bail!("experimental routed Flow requires one everyday and one strong profile");
    }
    let approved: std::collections::BTreeSet<_> = authorization
        .allowed_profiles
        .iter()
        .map(|p| {
            (
                p.target.clone(),
                p.profile_digest.clone(),
                p.binding_digest.clone(),
            )
        })
        .collect();
    let mut registered = std::collections::BTreeSet::new();
    let reviewed_backend = ExecutionBackend::parse(&plan.execution_backend)
        .context("reviewed routed execution backend is invalid")?;
    if !matches!(
        reviewed_backend,
        ExecutionBackend::Pty | ExecutionBackend::Subprocess
    ) {
        bail!("reviewed routed execution backend has no owned local launcher");
    }
    for entry in &mission.profiles {
        if entry.profile.backend != reviewed_backend {
            bail!("routed profile execution backend differs from the reviewed Flow");
        }
        let profile_digest = entry.profile.digest()?;
        let binding_digest = entry.binding.digest()?;
        if entry.binding.profile_digest != profile_digest
            || !authorization
                .allowed_billing_sources
                .contains(&entry.binding.billing_source_id)
            || !authorization
                .allowed_billing_modes
                .contains(&entry.binding.billing_mode)
        {
            bail!("routed profile binding is outside reviewed authority");
        }
        registered.insert((
            pytxo_core::routing::RouteTarget {
                profile_id: entry.profile.id.clone(),
                binding_id: entry.binding.id.clone(),
            },
            profile_digest,
            binding_digest,
        ));
    }
    if approved != registered
        || mission.policy.everyday == mission.policy.strong
        || !approved
            .iter()
            .any(|(target, _, _)| *target == mission.policy.everyday)
        || !approved
            .iter()
            .any(|(target, _, _)| *target == mission.policy.strong)
    {
        bail!("routed everyday and strong profiles differ from reviewed authority");
    }
    let mission_digest = canonical_digest(&mission, 1)?;
    Ok(RoutedFlowReview {
        authorization: mission.authorization.clone(),
        mission_digest,
    })
}

fn staged_routing_ref(plan: &FlowPlan) -> anyhow::Result<StagedRoutingMissionRef> {
    let review = plan
        .routing
        .as_ref()
        .context("experimental routed Flow requires a routed review")?;
    Ok(StagedRoutingMissionRef {
        domain_id: review.authorization.domain_id.clone(),
        run_id: review.authorization.run_id.clone(),
        draft_id: plan.draft_id.clone(),
        plan_digest: review.authorization.plan_digest.clone(),
        mission_digest: review.mission_digest.clone(),
    })
}

/// The private Store record is data, never review authority by itself. Rebind it
/// to the complete current public Flow plan before dispatch or registration.
pub(crate) fn load_reviewed_staged_mission(
    store: &PytxoStore,
    plan: &FlowPlan,
) -> anyhow::Result<RoutingMission> {
    let reviewed = staged_routing_ref(plan)?;
    let mission = store.load_staged_routing_mission(&reviewed)?;
    let recomputed =
        review_routed_mission(plan, &reviewed.run_id, &reviewed.plan_digest, &mission)?;
    if plan.routing.as_ref() != Some(&recomputed) {
        bail!("reviewed routing stage differs from current Flow review");
    }
    Ok(mission)
}

/// Persist draft intent only. Execution state and plan snapshots are Rust-owned and cannot be
/// supplied by Desktop.
pub fn save_flow_draft(
    catalog: &Catalog,
    input: FlowDraftInput,
) -> anyhow::Result<FlowDraftRecord> {
    if input.id.trim().is_empty() {
        bail!("Flow draft ID must not be empty");
    }
    let now = Utc::now().to_rfc3339();
    let created_at = catalog
        .get_flow_draft(&input.id)?
        .map(|draft| draft.created_at)
        .unwrap_or_else(|| now.clone());
    let draft = FlowDraftRecord {
        id: input.id,
        title: input.title,
        mission_text: input.mission_text,
        source: input.source.as_str().into(),
        domain_id: input.domain_id,
        project_id: input.project_id,
        status: FlowStatus::Draft.as_str().into(),
        plan_json: None,
        dispatched_run_id: None,
        created_at,
        updated_at: now,
    };
    if !catalog.upsert_flow_draft_intent(&draft)? {
        bail!("Flow draft is already dispatching or dispatched");
    }
    Ok(draft)
}

/// Produce and persist the mandatory dry-run preview for an explicit Flow request.
pub fn preview_flow(catalog: &Catalog, input: FlowDraftInput) -> anyhow::Result<FlowPlan> {
    preview_flow_scoped(catalog, input, false, None)
}

/// Desktop Beta supports one Codex worker under Orbit in one local domain.
/// This admission scope does not alter saved configuration or CLI capabilities.
pub fn preview_desktop_beta_flow(
    catalog: &Catalog,
    input: FlowDraftInput,
) -> anyhow::Result<FlowPlan> {
    preview_flow_scoped(catalog, input, true, None)
}

fn preview_flow_scoped(
    catalog: &Catalog,
    input: FlowDraftInput,
    desktop_beta: bool,
    routed_builder: Option<RoutedMissionBuilder<'_>>,
) -> anyhow::Result<FlowPlan> {
    let routed = routed_builder.is_some();
    if input.mission_text.trim().is_empty() {
        bail!("Flow mission must not be empty");
    }
    let selected_domain = input
        .domain_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .context("Flow requires a selected repository/execution domain")?;
    let repo = resolve_repo_root(Some(Path::new(selected_domain)))
        .context("invalid Flow repository/execution domain")?;
    let cfg = load_config_for_repo(None, &repo)?;
    let entitlements = crate::entitlements::effective_entitlements(&cfg)
        .map_err(|error| anyhow::anyhow!(error))?;
    let (mut cfg, ceiling_blocker) =
        apply_flow_permission_ceiling(&cfg, entitlements.permission_ceiling);
    if let Some(workers) = input.max_workers {
        if workers == 0 || workers > cfg.max_agents {
            bail!(
                "Requested workers must be between 1 and the configured limit ({})",
                cfg.max_agents
            );
        }
        cfg.max_agents = workers;
    }
    if input.verification_commands.len() > 16
        || input.verification_commands.iter().any(|command| {
            command.trim().is_empty()
                || command.len() > 4096
                || command.contains(['\0', '\n', '\r'])
        })
    {
        bail!("Use up to 16 non-empty verification commands, one command per line (maximum 4096 bytes each)");
    }

    // Explicit Flow / mission requests use the mission planner (ADR-0031).
    let mission = MissionSpec {
        text: input.mission_text.clone(),
    };
    // A routed preview cannot inherit ambient LLM planner opt-in. Its reviewed
    // task graph is produced locally; the existing Flow path remains unchanged.
    let planner: Box<dyn pytxo_planner::MissionPlanner> = if routed {
        Box::new(pytxo_planner::SignalBackedPlanner)
    } else {
        pytxo_planner::default_mission_planner(&cfg)
    };
    let mut planned = planner.decompose(
        &mission,
        &PlannerContext {
            repo: &repo,
            config: &cfg,
        },
    )?;
    for task in &mut planned.tasks {
        for command in &input.verification_commands {
            let command = command.trim().to_owned();
            if !task.verify.contains(&command) {
                task.verify.push(command);
            }
        }
    }
    let execution = plan_tasks(&planned.tasks, &cfg)?;

    let mut blocked_reasons = validate_task_claims(&planned.tasks, input.project_id.as_deref());
    blocked_reasons.extend(ceiling_blocker);
    blocked_reasons.extend(validate_permission_scope(&planned.tasks, &cfg));
    if desktop_beta {
        blocked_reasons.extend(desktop_beta_blockers(
            &cfg,
            &planned.tasks,
            input.ade_id.as_deref(),
        ));
    }
    let checkout_check = if routed {
        observe_experimental_routed_git_base(&repo).map(|_| ())
    } else {
        validate_flow_checkout(&repo, &planned.tasks, &cfg)
    };
    if let Err(error) = checkout_check {
        blocked_reasons.push(FlowBlockedReason::CheckoutUnavailable {
            message: error.to_string(),
        });
    }
    // Path overlaps that the scheduler already placed in different waves are warnings only —
    // they will not run concurrently. Same-wave overlaps remain hard blocks.
    let wave_of: HashMap<String, usize> = execution
        .waves
        .iter()
        .enumerate()
        .flat_map(|(i, wave)| {
            wave.iter()
                .map(move |t| (t.task_id.0.clone(), i))
                .collect::<Vec<_>>()
        })
        .collect();
    for conflict in &execution.conflicts {
        let same_wave = wave_of.get(&conflict.task_a.0) == wave_of.get(&conflict.task_b.0);
        if same_wave {
            blocked_reasons.push(FlowBlockedReason::OverlappingPathClaims {
                task_a: conflict.task_a.0.clone(),
                task_b: conflict.task_b.0.clone(),
                paths: conflict.paths.clone(),
            });
        }
    }
    let ade = if routed {
        // A reviewed profile selects its own harness. A detected legacy ADE
        // must never appear as the agent requested by this reviewed plan.
        FlowAdeSummary {
            requested: None,
            available: false,
            installed: Vec::new(),
            command: None,
        }
    } else {
        summarize_ade(input.ade_id.as_deref())
    };
    if !ade.available && !routed {
        blocked_reasons.push(FlowBlockedReason::AdeUnavailable {
            ade_id: input.ade_id.clone().unwrap_or_else(|| "any".into()),
        });
    }
    let mut warnings: Vec<FlowWarning> = execution
        .warnings
        .iter()
        .map(|message| FlowWarning {
            code: "scheduler".into(),
            message: message.clone(),
        })
        .collect();
    warnings.extend(execution.conflicts.iter().map(|conflict| {
        let same_wave = wave_of.get(&conflict.task_a.0) == wave_of.get(&conflict.task_b.0);
        FlowWarning {
            code: if same_wave {
                "path_claim_overlap".into()
            } else {
                "path_claim_staged".into()
            },
            message: format!(
                "{} and {} overlap on {}{}",
                conflict.task_a.0,
                conflict.task_b.0,
                conflict.paths.join(", "),
                if same_wave {
                    String::new()
                } else {
                    " — scheduled in separate stages".into()
                }
            ),
        }
    }));
    let tasks = planned
        .tasks
        .iter()
        .map(|task| FlowPlanTask {
            id: task.id.0.clone(),
            agent: task.agent.clone(),
            prompt: planned
                .task_prompts
                .get(&task.id.0)
                .cloned()
                .unwrap_or_default(),
            paths: task.paths.clone(),
            dependencies: task.depends_on.clone(),
            root: task.root.clone(),
            verify: task.verify.clone(),
        })
        .collect();
    let waves = execution
        .waves
        .iter()
        .map(|wave| wave.iter().map(|task| task.task_id.0.clone()).collect())
        .collect();
    let status = if blocked_reasons.is_empty() {
        FlowStatus::Ready
    } else {
        FlowStatus::Blocked
    };
    let now = Utc::now().to_rfc3339();
    let backend = format!("{:?}", cfg.execution_backend).to_ascii_lowercase();
    let mut plan = FlowPlan {
        draft_id: input.id.clone(),
        domain_id: repo.to_string_lossy().into_owned(),
        project_id: input.project_id.clone(),
        status,
        tasks,
        waves,
        max_workers: cfg.max_agents,
        permission_profile: cfg.permission_profile.as_str().into(),
        isolation_mode: cfg.isolation.as_str().into(),
        isolation_backend_intent: pytxo_runner::effective_isolation_mode(&cfg).as_str().into(),
        execution_backend: backend,
        ade,
        warnings,
        blocked_reasons,
        estimated_tokens: None,
        estimated_cost_usd: None,
        previewed_at: now.clone(),
        routing: None,
    };
    if let Some(builder) = routed_builder {
        if plan.status != FlowStatus::Ready {
            if let Some(message) = plan.blocked_reasons.iter().find_map(|reason| match reason {
                FlowBlockedReason::CheckoutUnavailable { message } => Some(message),
                _ => None,
            }) {
                bail!("experimental routed Flow checkout unavailable: {message}");
            }
            bail!("experimental routed Flow requires a ready plan");
        }
        if plan.permission_profile != PermissionProfile::Orbit.as_str()
            || cfg.requested_permission_profile != Some(PermissionProfile::Orbit)
            || !(plan.max_workers == 1 || routed_parallel_siblings(&plan))
        {
            bail!("experimental routed Flow requires Orbit and the exact reviewed worker count");
        }
        if plan
            .tasks
            .iter()
            .any(|task| cfg.resolve_profile_for_agent(&task.agent) != PermissionProfile::Orbit)
        {
            bail!("experimental routed Flow requires Orbit for every task");
        }
        validate_routed_local_wave_shape(&plan)?;
        for task in &plan.tasks {
            validate_final_routed_checks(&task.verify)?;
        }
        let run_id = RunId::new();
        let digest = routed_plan_digest(&plan)?;
        let mission = builder(&plan, &run_id, &digest)?;
        if mission.policy.advisor_recipient.as_deref() == Some(HOSTED_RECIPIENT) {
            plan.status = FlowStatus::ReviewOnly;
            plan.warnings.push(FlowWarning {
                code: "hosted_shadow_review_only".into(),
                message: "Hosted Shadow packet review only. This plan cannot be dispatched until the hosted account, consent, and send controller is qualified.".into(),
            });
        } else if mission.profiles.iter().any(|registered| {
            registered.profile.harness_id == "claude"
                && registered.profile.adapter_digest
                    == Digest::of_bytes(crate::routed_claude::CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes())
        }) {
            plan.warnings.push(FlowWarning {
                code: "claude_subscription_readiness_usage".into(),
                message: "Dispatch runs live Claude subscription auth and model-readiness probes before the routed attempt. These may consume account quota even if no candidate is produced.".into(),
            });
        }
        plan.routing = Some(review_routed_mission(&plan, &run_id, &digest, &mission)?);
        // Stage before publishing the Catalog preview. If that publish
        // loses a concurrent dispatch, the private record is an inert orphan.
        let stage =
            PytxoStore::open(&cfg.db_path_at(&repo))?.stage_routing_mission(&input.id, &mission)?;
        if stage != staged_routing_ref(&plan)? {
            bail!("reviewed routing stage differs from current Flow review");
        }
    }
    let previous_created = catalog
        .get_flow_draft(&input.id)?
        .map(|draft| draft.created_at)
        .unwrap_or_else(|| now.clone());
    let preview_record = FlowDraftRecord {
        id: input.id,
        title: input.title,
        mission_text: input.mission_text,
        source: input.source.as_str().into(),
        domain_id: Some(plan.domain_id.clone()),
        project_id: input.project_id,
        status: plan.status.as_str().into(),
        plan_json: Some(serde_json::to_string(&plan)?),
        dispatched_run_id: None,
        created_at: previous_created,
        updated_at: now,
    };
    if !catalog.upsert_flow_preview(&preview_record)? {
        bail!("Flow draft is already dispatching or dispatched");
    }
    Ok(plan)
}

/// Persist prompt edits made during mandatory plan review without allowing Desktop to mutate
/// execution structure or policy. Dispatch subsequently reloads this exact reviewed snapshot.
pub fn save_reviewed_flow_plan(catalog: &Catalog, reviewed: FlowPlan) -> anyhow::Result<FlowPlan> {
    let draft = catalog
        .get_flow_draft(&reviewed.draft_id)?
        .with_context(|| format!("Flow draft not found: {}", reviewed.draft_id))?;
    if !matches!(draft.status.as_str(), "ready" | "review_only") {
        bail!("Flow plan review requires a persisted reviewable preview");
    }
    let expected_plan_json = draft
        .plan_json
        .as_deref()
        .context("Flow plan review requires a persisted ready preview")?;
    let mut persisted: FlowPlan =
        serde_json::from_str(expected_plan_json).context("invalid persisted Flow preview")?;

    let top_level_changed = reviewed.draft_id != persisted.draft_id
        || reviewed.domain_id != persisted.domain_id
        || reviewed.project_id != persisted.project_id
        || reviewed.status != persisted.status
        || reviewed.waves != persisted.waves
        || reviewed.max_workers != persisted.max_workers
        || reviewed.permission_profile != persisted.permission_profile
        || reviewed.isolation_mode != persisted.isolation_mode
        || reviewed.isolation_backend_intent != persisted.isolation_backend_intent
        || reviewed.execution_backend != persisted.execution_backend
        || reviewed.ade != persisted.ade
        || reviewed.warnings != persisted.warnings
        || reviewed.blocked_reasons != persisted.blocked_reasons
        || reviewed.estimated_tokens != persisted.estimated_tokens
        || reviewed.estimated_cost_usd != persisted.estimated_cost_usd
        || reviewed.previewed_at != persisted.previewed_at
        || reviewed.routing != persisted.routing
        || reviewed.tasks.len() != persisted.tasks.len();
    if top_level_changed {
        bail!("Flow plan structure changed; generate a new preview");
    }

    let reviewed_tasks: HashMap<_, _> = reviewed
        .tasks
        .into_iter()
        .map(|task| (task.id.clone(), task))
        .collect();
    if reviewed_tasks.len() != persisted.tasks.len() {
        bail!("Flow plan structure changed; generate a new preview");
    }
    for task in &mut persisted.tasks {
        let edited = reviewed_tasks
            .get(&task.id)
            .context("Flow plan structure changed; generate a new preview")?;
        if edited.agent != task.agent
            || edited.paths != task.paths
            || edited.dependencies != task.dependencies
            || edited.root != task.root
            || edited.verify != task.verify
        {
            bail!("Flow plan structure changed; generate a new preview");
        }
        if edited.prompt.trim().is_empty() {
            bail!("Flow task prompts must not be empty");
        }
        if persisted.routing.is_some() && edited.prompt != task.prompt {
            bail!("routed Flow prompt changed; generate a new experimental preview");
        }
        task.prompt = edited.prompt.clone();
    }

    let reviewed_plan_json = serde_json::to_string(&persisted)?;
    if !catalog.replace_ready_flow_plan(
        &persisted.draft_id,
        expected_plan_json,
        &reviewed_plan_json,
    )? {
        bail!("Flow preview changed or dispatch started; review the latest plan");
    }
    Ok(persisted)
}

/// Revalidate and dispatch a persisted ready preview through the standard `dispatch_run` path.
pub fn dispatch_flow(catalog: &Catalog, draft_id: &str) -> anyhow::Result<String> {
    dispatch_flow_scoped(catalog, draft_id, false, false, None)
}

/// Recheck Desktop admission using the config snapshot passed to execution.
pub fn dispatch_desktop_beta_flow(catalog: &Catalog, draft_id: &str) -> anyhow::Result<String> {
    dispatch_flow_scoped(catalog, draft_id, true, false, None)
}

/// A registry entry is only a stale controller marker when its exact native
/// worker/checker owner has a retained Job-zero settlement. Unknown entries
/// continue to block recovery, even if their PID is no longer present.
pub(crate) fn registry_matches_settled_routed_owners(
    registry: &ProcessRegistryFile,
    store: &PytxoStore,
    history: &RoutingHistory,
    scope: &RoutingScope,
    repo: &Path,
) -> pytxo_core::Result<bool> {
    let mut seen = BTreeSet::new();
    for entry in registry.for_run(&scope.run_id.0) {
        if entry.repo_root != repo.to_string_lossy()
            || entry.pid == 0
            || entry.start_identity.is_none()
            || !entry.branch.is_empty()
            || !seen.insert(&entry.agent_key)
        {
            return Ok(false);
        }
        let mut matched = false;
        for attempt in &history.attempts {
            if attempt.scope != *scope
                || !attempt.state.is_terminal()
                || !attempt.ownership_released
            {
                return Ok(false);
            }
            let worker_key = format!("{}:{}", scope.run_id.0, attempt.attempt_id.0);
            if entry.agent_key == worker_key {
                let Some(owner) = store.launch_ownership(&attempt.attempt_id)? else {
                    return Ok(false);
                };
                matched = owner.phase == LaunchOwnershipPhase::Settled
                    && owner.request.scope == *scope
                    && owner.request.task_id == attempt.task_id
                    && owner.pid == Some(entry.pid)
                    && owner.start_identity == entry.start_identity
                    && owner.settlement_blob.is_some();
                break;
            }
            let Some(task) = history
                .tasks
                .iter()
                .find(|task| task.registration.contract.task_id == attempt.task_id)
            else {
                return Ok(false);
            };
            for (index, check) in task.registration.contract.checks.iter().enumerate() {
                let Ok(ordinal) = u32::try_from(index + 1) else {
                    return Ok(false);
                };
                if entry.agent_key
                    != format!(
                        "{}:{}:checker-{ordinal}",
                        scope.run_id.0, attempt.attempt_id.0
                    )
                {
                    continue;
                }
                let Some(owner) = store.checker_ownership(&attempt.attempt_id, ordinal)? else {
                    return Ok(false);
                };
                matched = owner.phase == CheckerOwnershipPhase::Settled
                    && owner.request.scope == *scope
                    && owner.request.task_id == attempt.task_id
                    && owner.request.check_id == check.id
                    && owner.request.recipe_digest == check.recipe_digest
                    && owner.pid == Some(entry.pid)
                    && owner.start_identity == entry.start_identity
                    && owner.settlement_blob.is_some();
                break;
            }
            if matched {
                break;
            }
        }
        if !matched {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Caller may recover a terminal routed run only after Core and Catalog show
/// no unresolved ownership. The active-run gate serializes the registry and
/// marker cleanup with normal Stop/dispatch settlement.
#[allow(clippy::too_many_arguments)]
fn clear_terminal_routed_active_run_if_safe(
    catalog: &Catalog,
    data_dir: &Path,
    repo: &Path,
    store: &PytxoStore,
    scope: &RoutingScope,
    expected_history: &RoutingHistory,
    allow_registered_failed_review: bool,
    expected_owner: &RoutedFlowDispatchOwner,
) -> anyhow::Result<bool> {
    let active_path = data_dir.join("active_run.json");
    let _gate = ActiveRunGate::acquire(&active_path)?;
    let run_id = &scope.run_id.0;
    if !store.get_run_status(run_id)?.is_some_and(|row| {
        matches!(
            row.0.as_str(),
            "completed" | "failed" | "cancelled" | "failed_startup"
        )
    }) || store.routing_history(scope)?.as_ref() != Some(expected_history)
        || store
            .unresolved_routing_attempts()?
            .iter()
            .any(|attempt| attempt.scope == *scope)
        || (!allow_registered_failed_review
            && store
                .unreconciled_registered_routing_scopes()?
                .contains(scope))
        || store.unresolved_capacity_intent_scopes()?.contains(scope)
        || !store
            .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))?
            .is_empty()
        || catalog
            .unresolved_capacity_reservations()?
            .iter()
            .any(|reservation| {
                reservation.domain_id == scope.domain_id.0 && reservation.run_id == *run_id
            })
        || read_active_run_state(&active_path)?.is_some_and(|active| {
            active.run_id != *run_id
                || active.repo_root != repo.to_string_lossy()
                || active.supervisor_pid != expected_owner.controller_pid
                || active.supervisor_start_identity.as_deref()
                    != Some(expected_owner.controller_start_identity.as_str())
        })
    {
        return Ok(false);
    }
    let path = registry_path(data_dir);
    let registry = ProcessRegistryFile::load(&path)?;
    if !registry_matches_settled_routed_owners(&registry, store, expected_history, scope, repo)? {
        return Ok(false);
    }
    if !registry.for_run(run_id).is_empty() {
        let expected_entries = serde_json::to_value(registry.for_run(run_id))?;
        ProcessRegistryFile::update(&path, |current| {
            let current_entries = serde_json::to_value(current.for_run(run_id))
                .map_err(|error| PytxoError::Runner(error.to_string()))?;
            if current_entries != expected_entries {
                return Err(PytxoError::Runner(
                    "routed registry changed during quiescent recovery".into(),
                ));
            }
            current.remove_run(run_id);
            Ok(())
        })?;
    }
    crate::clear_active_run_unlocked(&active_path, run_id)?;
    Ok(true)
}

/// A dead controller may close one-use owners only while their durable phase
/// proves that native creation was never authorized. This does not resume a
/// task, publish Review, or infer Job-zero from a PID.
#[allow(clippy::too_many_arguments)]
fn reconcile_abandoned_prepared_attempts(
    catalog: &Catalog,
    plan: &FlowPlan,
    owner: &RoutedFlowDispatchOwner,
    repo: &Path,
    data_dir: &Path,
    store_path: &Path,
    expected_file_identity: &str,
    store_file_guard: &pytxo_runner::FileIdentityGuard,
) -> anyhow::Result<bool> {
    let single_task = plan.tasks.len() == 1
        && plan.max_workers == 1
        && plan.waves.as_slice() == [vec![plan.tasks[0].id.clone()]]
        && plan.tasks[0].dependencies.is_empty();
    if !(routed_parallel_siblings(plan) || single_task)
        || pytxo_runner::process_matches(owner.controller_pid, &owner.controller_start_identity)?
    {
        return Ok(false);
    }
    let Some(review) = plan.routing.as_ref() else {
        return Ok(false);
    };
    let scope = RoutingScope {
        domain_id: review.authorization.domain_id.clone(),
        run_id: review.authorization.run_id.clone(),
    };
    let active_path = data_dir.join("active_run.json");
    let _gate = ActiveRunGate::acquire(&active_path)?;
    let Some(active) = read_active_run_state(&active_path)? else {
        return Ok(false);
    };
    if active.run_id != scope.run_id.0
        || active.repo_root != repo.to_string_lossy()
        || active.supervisor_pid != owner.controller_pid
        || active.supervisor_start_identity.as_deref()
            != Some(owner.controller_start_identity.as_str())
        || catalog
            .routed_flow_dispatch_owner(&plan.draft_id, &scope.run_id.0)?
            .as_ref()
            != Some(owner)
        || store_file_guard.identity() != expected_file_identity
        || pytxo_runner::file_identity(store_path)? != expected_file_identity
    {
        return Ok(false);
    }
    let store = PytxoStore::open_existing_read_write(store_path)?;
    let Some(run) = store.get_run(&scope.run_id.0)? else {
        return Ok(false);
    };
    if !matches!(run.status.as_str(), "starting" | "cancelled")
        || run.repo_root != repo.to_string_lossy()
        || run.permission_profile.as_deref() != Some("orbit")
    {
        return Ok(false);
    }
    let Some(history) = store.routing_history(&scope)? else {
        return Ok(false);
    };
    // A passed or failed sibling may have a stale registry row after its
    // owned Job settles. The shared validator still requires an exact
    // terminal, released attempt and matching Job-zero owner for every row.
    let mut registered_history = history.clone();
    registered_history
        .attempts
        .retain(|attempt| attempt.state.is_terminal() && attempt.ownership_released);
    let ordinary_attempts = history.attempts.len() <= plan.tasks.len()
        && history.attempts.iter().all(|attempt| {
            attempt.scope == scope && attempt.ordinal == 1 && attempt.predecessor.is_none()
        });
    let prepared_claude_repair = if single_task
        && history.tasks.len() == 1
        && history.attempts.len() == 2
        && history.mission.authorization.limits.max_attempts == 2
        && history.mission.policy.mode == RoutingMode::Rules
        && history.mission.policy.version == "claude-proposal-rules-repair-v1"
    {
        let task = &history.tasks[0];
        let second = task.current_attempt.as_ref().and_then(|id| {
            history
                .attempts
                .iter()
                .find(|attempt| attempt.attempt_id == *id)
        });
        if let Some(second) = second {
            let prior = second.predecessor.as_ref().and_then(|id| {
                history
                    .attempts
                    .iter()
                    .find(|attempt| attempt.attempt_id == *id)
            });
            if let Some(prior) = prior {
                let prior_safe = settled_actionable_claude_check_predecessor_is_safe(
                    &store, catalog, &scope, prior,
                )?;
                second.scope == scope
                    && second.task_id == task.registration.contract.task_id
                    && second.ordinal == 2
                    && second.selected.profile.id == history.mission.policy.strong.profile_id
                    && second.dependencies.is_empty()
                    && prior.task_id == second.task_id
                    && prior.selected.profile.id == history.mission.policy.everyday.profile_id
                    && task.next_ordinal == 3
                    && task.winner.is_none()
                    && prior_safe
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    if canonical_digest(&history.mission, 1)? != review.mission_digest
        || history.tasks.len() != plan.tasks.len()
        || history.attempts.is_empty()
        || !(ordinary_attempts || prepared_claude_repair)
        || !registry_matches_settled_routed_owners(
            &ProcessRegistryFile::load(&registry_path(data_dir))?,
            &store,
            &registered_history,
            &scope,
            repo,
        )?
    {
        return Ok(false);
    }
    let mut recover = Vec::new();
    let mut prepared_count = 0;
    for planned in &plan.tasks {
        let Some(task) = history
            .tasks
            .iter()
            .find(|task| task.registration.contract.task_id.0 == planned.id)
        else {
            return Ok(false);
        };
        if !task.registration.contract.dependencies.is_empty() {
            return Ok(false);
        }
        let Some(attempt) = task.current_attempt.as_ref().and_then(|id| {
            history.attempts.iter().find(|attempt| {
                attempt.attempt_id == *id && attempt.task_id == task.registration.contract.task_id
            })
        }) else {
            // Admission is sequential even for a parallel wave. An absent
            // second task has no launch authority or capacity release proof;
            // the downstream all-ownership scans must still be empty before
            // the run can be closed.
            if task.current_attempt.is_some()
                || task.winner.is_some()
                || task.next_ordinal != 1
                || !matches!(
                    task.state,
                    TaskRoutingState::Ready | TaskRoutingState::Cancelled
                )
            {
                return Ok(false);
            }
            continue;
        };
        if task.current_attempt.as_ref() != Some(&attempt.attempt_id) {
            return Ok(false);
        }
        if task.winner.is_some() {
            if task.state != TaskRoutingState::Succeeded
                || task.winner.as_ref().is_none_or(|winner| {
                    winner.winning_attempt_id != attempt.attempt_id
                        || attempt.receipts.sealed_output.as_ref() != Some(&winner.output_digest)
                        || attempt.receipts.checks.as_ref()
                            != Some(&winner.verification_receipt_digest)
                })
                || attempt.state != AttemptState::Passed
                || !attempt.ownership_released
            {
                return Ok(false);
            }
            continue;
        }
        // This bounded path can preserve a sibling that already failed and
        // released its native Job before the controller died. The other
        // sibling still needs positive no-create proof below. A failed
        // checker or unsettled usage remains recovery-owned.
        if attempt.state == AttemptState::Failed && attempt.ownership_released {
            if task.state != TaskRoutingState::Cancelled
                || !history.cancelled
                || !settled_failed_fixture_sibling_is_safe(&store, catalog, &scope, attempt)?
            {
                return Ok(false);
            }
            continue;
        }
        let Some(launch) = store.launch_ownership(&attempt.attempt_id)? else {
            return Ok(false);
        };
        if launch.request.scope != scope
            || launch.request.task_id != attempt.task_id
            || launch.request.attempt_id != attempt.attempt_id
            || launch.request.reservation_id != attempt.capacity_reservation
            || !attempt.owned_launch_required
            || attempt.owned_checker_count != 0
            || attempt.receipts.process_identity.is_some()
            || !attempt.dependencies.is_empty()
            || !matches!(
                launch.phase,
                LaunchOwnershipPhase::Prepared | LaunchOwnershipPhase::ClosedNoLaunch
            )
            || launch.create_event_id.is_some()
            || launch.job_name.is_some()
            || launch.pid.is_some()
            || !matches!(
                attempt.state,
                AttemptState::Preparing | AttemptState::Launching | AttemptState::FailedNoLaunch
            )
            || (attempt.state == AttemptState::FailedNoLaunch) != attempt.ownership_released
        {
            return Ok(false);
        }
        if launch.phase == LaunchOwnershipPhase::Prepared {
            prepared_count += 1;
        }
        recover.push(attempt.clone());
    }
    if recover.is_empty() {
        return Ok(false);
    }
    let owned_targets = store.owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))?;
    if owned_targets.len() != prepared_count
        || owned_targets.iter().any(|target| {
            target.scope != scope
                || target.phase != OwnedJobStopPhase::Prepared
                || !matches!(target.kind, OwnedJobStopKind::Worker)
                || target.job_name.is_some()
                || target.launch_nonce.is_some()
                || target.pid.is_some()
                || target.start_identity.is_some()
                || !recover.iter().any(|attempt| {
                    attempt.attempt_id == target.attempt_id && attempt.task_id == target.task_id
                })
        })
    {
        return Ok(false);
    }
    if !history.cancelled {
        store.cancel_routing_mission(
            &scope,
            "controller.recovery.prepared-no-create.v1",
            history.cancel_epoch,
            u64::try_from(Utc::now().timestamp_millis())?,
        )?;
    }
    let cancelled = store
        .routing_history(&scope)?
        .context("prepared sibling mission disappeared after cancellation")?;
    for attempt in &recover {
        settle_abandoned_prepared_attempt(&store, catalog, &cancelled, attempt, data_dir)?;
    }
    Ok(true)
}

fn settled_failed_fixture_sibling_is_safe(
    store: &PytxoStore,
    catalog: &Catalog,
    scope: &RoutingScope,
    attempt: &RoutedAttemptRecord,
) -> anyhow::Result<bool> {
    if attempt.owned_checker_count != 0
        || attempt.receipts.sealed_output.is_some()
        || attempt.receipts.checks.is_some()
        || attempt.receipts.no_worker_created.is_some()
    {
        return Ok(false);
    }
    let Some(launch) = store.launch_ownership(&attempt.attempt_id)? else {
        return Ok(false);
    };
    let Some(intent) = store.capacity_intent(&attempt.capacity_reservation)? else {
        return Ok(false);
    };
    let Some(reservation) = catalog.capacity_reservation(&attempt.capacity_reservation)? else {
        return Ok(false);
    };
    let Some(release) = intent.admitted_quiescent_release.as_ref() else {
        return Ok(false);
    };
    let Some(evidence) = intent.expected_release.as_ref() else {
        return Ok(false);
    };
    let Some(receipt_ref) = launch.settlement_blob.as_ref() else {
        return Ok(false);
    };
    if launch.phase != LaunchOwnershipPhase::Settled
        || launch.request.scope != *scope
        || launch.request.task_id != attempt.task_id
        || launch.request.attempt_id != attempt.attempt_id
        || launch.request.reservation_id != attempt.capacity_reservation
        || attempt.receipts.quiescence.as_ref() != Some(&receipt_ref.digest)
        || intent.request.scope != *scope
        || intent.request.task_id != attempt.task_id
        || intent.request.reservation.reservation_id != attempt.capacity_reservation
        || intent.request.reservation.attempt_id != attempt.attempt_id.0
        || intent.phase != CapacityIntentPhase::Closed
        || intent.close_event_id.as_deref()
            != Some(format!("{}:capacity-closed", attempt.attempt_id.0).as_str())
        || release.launch_token != launch.request.launch_token
        || release.worker_receipt_ref != *receipt_ref
        || !release.checker_receipts.is_empty()
        || release.worker_receipt_claim.scope != *scope
        || release.worker_receipt_claim.task_id != attempt.task_id
        || release.worker_receipt_claim.attempt_id != attempt.attempt_id
        || release.worker_receipt_claim.reservation_id != attempt.capacity_reservation
        || release.worker_receipt_claim.kind != PrivateArtifactKind::ControllerReceipt
        || release.worker_receipt_claim.artifact_id
            != format!("{}:native-job-zero", attempt.attempt_id.0)
        || release.worker_receipt_claim.event_id
            != format!("{}:native-job-zero:observed", attempt.attempt_id.0)
        || evidence.kind != CapacityReleaseEvidenceKind::QuiescenceReconciled
        || evidence.receipt_id != release.worker_receipt_claim.event_id
        || evidence.evidence_digest != canonical_digest(release, 1)?.0
        || reservation.reservation_id != attempt.capacity_reservation
        || reservation.domain_id != scope.domain_id.0
        || reservation.run_id != scope.run_id.0
        || reservation.attempt_id != attempt.attempt_id.0
        || reservation.state != CapacityReservationState::Released
        || reservation.launch_token.as_deref() != Some(launch.request.launch_token.as_str())
        || reservation.release_evidence.as_ref() != Some(evidence)
    {
        return Ok(false);
    }
    let receipt = store.read_controller_receipt(&release.worker_receipt_claim, receipt_ref)?;
    if receipt.source != ReceiptSource::OwnedJobObservation
        || receipt.observed_at_ms != evidence.observed_at_ms
        || !matches!(
            receipt.observation,
            ControllerObservation::NativeJobZero {
                ref job_name,
                ref launch_nonce,
                active_processes: 0,
            } if Some(job_name) == launch.job_name.as_ref()
                && Some(launch_nonce) == launch.launch_nonce.as_ref()
        )
    {
        return Ok(false);
    }
    let expected_usage = canonical_digest(
        &(
            1_u32,
            "local-fixture-no-provider-charge",
            scope,
            &attempt.attempt_id,
            &attempt.selected.observation.executable,
        ),
        1,
    )?;
    Ok(matches!(
        &attempt.usage,
        RoutedUsage::Known { nano_usd: 0, receipt } if receipt == &expected_usage
    ))
}

/// A prepared strong repair may be closed after controller death only when
/// its predecessor is an independently settled Claude check failure. The
/// failed bytes are never imported into the second attempt.
fn settled_actionable_claude_check_predecessor_is_safe(
    store: &PytxoStore,
    catalog: &Catalog,
    scope: &RoutingScope,
    prior: &RoutedAttemptRecord,
) -> anyhow::Result<bool> {
    let Some(failure) = prior.failure.as_ref() else {
        return Ok(false);
    };
    let (Some(quiescence), Some(sealed)) = (
        prior.receipts.quiescence.as_ref(),
        prior.receipts.sealed_output.as_ref(),
    ) else {
        return Ok(false);
    };
    if prior.scope != *scope
        || prior.ordinal != 1
        || prior.predecessor.is_some()
        || prior.state != AttemptState::Failed
        || !prior.ownership_released
        || prior.owned_checker_count != 1
        || prior.receipts.checks.is_some()
        || prior.selected.profile.harness_id != "claude"
        || prior.selected.binding.billing_mode != BillingSourceMode::Subscription
        || !matches!(&prior.usage, RoutedUsage::Unknown { .. })
        || failure.attempt_id != prior.attempt_id
        || failure.ordinal != 1
        || failure.state != AttemptState::Failed
        || failure.failure_class != pytxo_core::routing::AttemptFailureClass::Check
    {
        return Ok(false);
    }
    let (Some(launch), Some(checker), Some(intent), Some(reservation)) = (
        store.launch_ownership(&prior.attempt_id)?,
        store.checker_ownership(&prior.attempt_id, 1)?,
        store.capacity_intent(&prior.capacity_reservation)?,
        catalog.capacity_reservation(&prior.capacity_reservation)?,
    ) else {
        return Ok(false);
    };
    let (Some(worker_ref), Some(check_ref), Some(release), Some(evidence)) = (
        launch.settlement_blob.as_ref(),
        checker.settlement_blob.as_ref(),
        intent.admitted_quiescent_release.as_ref(),
        intent.expected_release.as_ref(),
    ) else {
        return Ok(false);
    };
    let checker_claim = crate::routed_checker::checker_claim(&checker.request);
    if launch.phase != LaunchOwnershipPhase::Settled
        || launch.request.scope != *scope
        || launch.request.task_id != prior.task_id
        || launch.request.attempt_id != prior.attempt_id
        || launch.request.reservation_id != prior.capacity_reservation
        || worker_ref.digest != *quiescence
        || checker.phase != CheckerOwnershipPhase::Settled
        || checker.passed != Some(false)
        || checker.request.scope != *scope
        || checker.request.task_id != prior.task_id
        || checker.request.attempt_id != prior.attempt_id
        || checker.request.reservation_id != prior.capacity_reservation
        || checker.request.ordinal != 1
        || checker.request.sealed_view.digest != *sealed
        || intent.phase != CapacityIntentPhase::Closed
        || intent.request.scope != *scope
        || intent.request.task_id != prior.task_id
        || intent.request.reservation.attempt_id != prior.attempt_id.0
        || intent.request.reservation.reservation_id != prior.capacity_reservation
        || release.launch_token != launch.request.launch_token
        || release.worker_receipt_ref != *worker_ref
        || release.checker_receipts.len() != 1
        || release.checker_receipts[0].ordinal != 1
        || release.checker_receipts[0].receipt_claim != checker_claim
        || release.checker_receipts[0].receipt_ref != *check_ref
        || evidence.kind != CapacityReleaseEvidenceKind::QuiescenceReconciled
        || evidence.evidence_digest != canonical_digest(release, 1)?.0
        || reservation.state != CapacityReservationState::Released
        || reservation.domain_id != scope.domain_id.0
        || reservation.run_id != scope.run_id.0
        || reservation.attempt_id != prior.attempt_id.0
        || reservation.release_evidence.as_ref() != Some(evidence)
    {
        return Ok(false);
    }
    let receipt = store.read_controller_receipt(&checker_claim, check_ref)?;
    let exit = match receipt.observation {
        ControllerObservation::NativeCheckerResult {
            check_id,
            ordinal: 1,
            job_name,
            launch_nonce,
            active_processes: 0,
            exit_code,
            payload_exit_code: Some(payload_exit_code),
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
        } if check_id == checker.request.check_id
            && checker.job_name.as_deref() == Some(job_name.as_str())
            && checker.launch_nonce.as_deref() == Some(launch_nonce.as_str())
            && exit_code > 0
            && u32::try_from(exit_code).ok() == Some(payload_exit_code)
            && sealed_view_before == checker.request.sealed_view
            && sealed_view_after == checker.request.sealed_view =>
        {
            u32::try_from(exit_code)?
        }
        _ => return Ok(false),
    };
    Ok(receipt.source == ReceiptSource::OwnedJobObservation
        && receipt.scope == *scope
        && receipt.task_id == prior.task_id
        && receipt.attempt_id == prior.attempt_id
        && receipt.reservation_id == prior.capacity_reservation
        && failure.actionable_evidence_digest.as_ref()
            == Some(&canonical_digest(
                &(
                    1_u32,
                    "claude-owned-frozen-check-failed",
                    scope,
                    &prior.attempt_id,
                    quiescence,
                    sealed,
                    &check_ref.digest,
                    Some(exit),
                ),
                1,
            )?))
}

/// A prepared owner with a trusted no-create receipt made no provider call.
/// This is the only subscription case where zero can be known without a
/// provider usage receipt. A launched Claude attempt is settled as Unknown.
fn prepared_no_create_usage(
    harness_id: &str,
    billing_mode: BillingSourceMode,
    scope: &RoutingScope,
    attempt_id: &pytxo_core::routing::AttemptId,
    executable: &ExecutableIdentity,
) -> anyhow::Result<RoutedUsage> {
    let source = match (harness_id, billing_mode) {
        ("pytxo-local-fixture-v1", BillingSourceMode::Local) => "local-fixture-no-provider-charge",
        #[cfg(feature = "routed-test-faults")]
        ("pytxo-local-fixture-powershell-v1", BillingSourceMode::Local) => {
            "local-fixture-no-provider-charge"
        }
        ("claude", BillingSourceMode::Subscription) => "claude-subscription-no-owned-create",
        _ => bail!("prepared attempt has no supported no-create billing source"),
    };
    Ok(RoutedUsage::Known {
        nano_usd: 0,
        receipt: canonical_digest(&(1_u32, source, scope, attempt_id, executable), 1)?,
    })
}

fn settled_no_create_attempt_is_safe(
    store: &PytxoStore,
    catalog: &Catalog,
    history: &RoutingHistory,
    attempt: &RoutedAttemptRecord,
) -> anyhow::Result<bool> {
    if !history.cancelled
        || attempt.state != AttemptState::FailedNoLaunch
        || !attempt.ownership_released
        || attempt.receipts.process_identity.is_some()
        || attempt.receipts.quiescence.is_some()
        || attempt.owned_checker_count != 0
        || attempt.usage
            != prepared_no_create_usage(
                &attempt.selected.profile.harness_id,
                attempt.selected.binding.billing_mode,
                &attempt.scope,
                &attempt.attempt_id,
                &attempt.selected.observation.executable,
            )?
    {
        return Ok(false);
    }
    let Some(launch) = store.launch_ownership(&attempt.attempt_id)? else {
        return Ok(false);
    };
    let Some(intent) = store.capacity_intent(&attempt.capacity_reservation)? else {
        return Ok(false);
    };
    let Some(reservation) = catalog.capacity_reservation(&attempt.capacity_reservation)? else {
        return Ok(false);
    };
    let Some(reference) = launch.settlement_blob.as_ref() else {
        return Ok(false);
    };
    let claim = PrivateArtifactClaim {
        scope: attempt.scope.clone(),
        task_id: attempt.task_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        reservation_id: attempt.capacity_reservation.clone(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: format!("{}:native-no-create", attempt.attempt_id.0),
        event_id: format!("{}:native-no-create:observed", attempt.attempt_id.0),
    };
    let receipt = store.read_controller_receipt(&claim, reference)?;
    let Some(evidence) = intent.expected_release.as_ref() else {
        return Ok(false);
    };
    let Some(release) = intent.admitted_no_launch_release.as_ref() else {
        return Ok(false);
    };
    let expected_evidence = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::KnownUnused,
        receipt_id: claim.event_id.clone(),
        evidence_digest: reference.digest.0.clone(),
        observed_at_ms: receipt.observed_at_ms,
    };
    Ok(launch.phase == LaunchOwnershipPhase::ClosedNoLaunch
        && launch.request.scope == attempt.scope
        && launch.request.task_id == attempt.task_id
        && launch.request.attempt_id == attempt.attempt_id
        && launch.request.reservation_id == attempt.capacity_reservation
        && launch.create_event_id.is_none()
        && launch.job_name.is_none()
        && launch.pid.is_none()
        && attempt.receipts.no_worker_created.as_ref() == Some(&reference.digest)
        && receipt.source == ReceiptSource::TrustedController
        && receipt.scope == attempt.scope
        && receipt.task_id == attempt.task_id
        && receipt.attempt_id == attempt.attempt_id
        && receipt.reservation_id == attempt.capacity_reservation
        && matches!(
            &receipt.observation,
            ControllerObservation::AdmittedNoLaunch { cancellation_event_id }
                if history.events.iter().any(|entry| {
                    entry.event_id == *cancellation_event_id
                        && matches!(entry.event, RoutingControlEvent::Cancelled { now_ms, .. }
                            if now_ms == receipt.observed_at_ms)
                })
        )
        && intent.phase == CapacityIntentPhase::Closed
        && intent.request.scope == attempt.scope
        && intent.request.task_id == attempt.task_id
        && intent.request.reservation.reservation_id == attempt.capacity_reservation
        && intent.request.reservation.attempt_id == attempt.attempt_id.0
        && intent.close_event_id.as_deref()
            == Some(format!("{}:capacity-closed", attempt.attempt_id.0).as_str())
        && release.launch_token == launch.request.launch_token
        && release.receipt_claim == claim
        && release.receipt_ref == *reference
        && evidence == &expected_evidence
        && reservation.state == CapacityReservationState::Released
        && reservation.launch_token.as_deref() == Some(launch.request.launch_token.as_str())
        && reservation.domain_id == attempt.scope.domain_id.0
        && reservation.run_id == attempt.scope.run_id.0
        && reservation.attempt_id == attempt.attempt_id.0
        && reservation.release_evidence.as_ref() == Some(&expected_evidence))
}

fn settle_abandoned_prepared_attempt(
    store: &PytxoStore,
    catalog: &Catalog,
    cancelled: &RoutingHistory,
    attempt: &RoutedAttemptRecord,
    data_dir: &Path,
) -> anyhow::Result<()> {
    let scope = &attempt.scope;
    let launch = store
        .launch_ownership(&attempt.attempt_id)?
        .context("prepared sibling launch owner disappeared")?;
    let admission = cancelled
        .events
        .iter()
        .find_map(|event| match &event.event {
            RoutingControlEvent::Admitted(admission)
                if admission.attempt_id == attempt.attempt_id =>
            {
                Some(admission.as_ref())
            }
            _ => None,
        })
        .context("prepared sibling admission journal event disappeared")?;
    if admission.scope != *scope
        || admission.task_id != attempt.task_id
        || admission.agent_id != attempt.agent_id
        || admission.capacity_reservation != attempt.capacity_reservation
        || launch.request.scope != *scope
        || launch.request.task_id != attempt.task_id
        || launch.request.reservation_id != attempt.capacity_reservation
    {
        bail!("prepared sibling journal and launch identity differ");
    }
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: attempt.task_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        reservation_id: attempt.capacity_reservation.clone(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: format!("{}:native-no-create", attempt.attempt_id.0),
        event_id: format!("{}:native-no-create:observed", attempt.attempt_id.0),
    };
    let (receipt_ref, receipt) = if launch.phase == LaunchOwnershipPhase::Prepared {
        let (event_id, at_ms) = cancelled
            .events
            .iter()
            .find_map(|entry| match &entry.event {
                RoutingControlEvent::Cancelled { now_ms, .. } => {
                    Some((entry.event_id.clone(), *now_ms))
                }
                _ => None,
            })
            .context("prepared sibling has no durable cancellation event")?;
        let receipt = ControllerReceiptEnvelope {
            schema_version: 1,
            scope: scope.clone(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            reservation_id: attempt.capacity_reservation.clone(),
            observed_at_ms: at_ms,
            evidence_id: claim.event_id.clone(),
            source: ReceiptSource::TrustedController,
            observation: ControllerObservation::AdmittedNoLaunch {
                cancellation_event_id: event_id,
            },
        };
        let reference = store.put_controller_receipt(&claim, &receipt)?;
        store.close_prepared_launch_without_worker(&LaunchSettlement {
            scope: scope.clone(),
            attempt_id: attempt.attempt_id.clone(),
            event_id: format!("{}:native-no-create-closed", attempt.attempt_id.0),
            receipt_claim: claim.clone(),
            receipt_ref: reference.clone(),
        })?;
        (reference, receipt)
    } else {
        let reference = launch
            .settlement_blob
            .context("closed no-create owner lost its retained receipt")?;
        let receipt = store.read_controller_receipt(&claim, &reference)?;
        (reference, receipt)
    };
    if receipt.source != ReceiptSource::TrustedController
        || !matches!(
            &receipt.observation,
            ControllerObservation::AdmittedNoLaunch { cancellation_event_id }
                if cancelled.events.iter().any(|entry| {
                    entry.event_id == *cancellation_event_id
                        && matches!(entry.event, RoutingControlEvent::Cancelled { now_ms, .. }
                            if now_ms == receipt.observed_at_ms)
                })
        )
    {
        bail!("prepared sibling no-create receipt lacks exact cancellation witness");
    }
    let history = store
        .routing_history(scope)?
        .context("prepared sibling history disappeared")?;
    let current = history
        .attempts
        .iter()
        .find(|row| row.attempt_id == attempt.attempt_id)
        .context("prepared sibling attempt disappeared")?;
    if !current.state.is_terminal() {
        let task = history
            .tasks
            .iter()
            .find(|task| task.registration.contract.task_id == attempt.task_id)
            .context("prepared sibling task disappeared")?;
        let mut facts: RoutingFacts = admission.facts.clone();
        facts.now_ms = u64::try_from(Utc::now().timestamp_millis())?.max(current.updated_at_ms);
        store.transition_routing_attempt(&TransitionRoutingAttempt {
            scope: scope.clone(),
            event_id: format!(
                "{}:{}:failed_no_launch",
                attempt.attempt_id.0, current.revision
            ),
            attempt_id: attempt.attempt_id.clone(),
            expected_attempt_revision: current.revision,
            expected_task_revision: task.revision,
            to: AttemptState::FailedNoLaunch,
            facts,
            receipts: RoutingReceipts {
                no_worker_created: Some(receipt_ref.digest.clone()),
                ..Default::default()
            },
            failure: None,
        })?;
    } else if current.state != AttemptState::FailedNoLaunch
        || !current.ownership_released
        || current.receipts.no_worker_created.as_ref() != Some(&receipt_ref.digest)
    {
        bail!("prepared sibling terminal state differs from no-create receipt");
    }
    let release = AdmittedNoLaunchRelease {
        launch_token: launch.request.launch_token.clone(),
        receipt_claim: claim.clone(),
        receipt_ref: receipt_ref.clone(),
    };
    let evidence = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::KnownUnused,
        receipt_id: claim.event_id,
        evidence_digest: receipt_ref.digest.0,
        observed_at_ms: receipt.observed_at_ms,
    };
    store.bind_admitted_no_launch_release_proof(
        catalog,
        &attempt.capacity_reservation,
        &format!("{}:release-proof", attempt.attempt_id.0),
        &evidence,
        &release,
    )?;
    let intent = store
        .capacity_intent(&attempt.capacity_reservation)?
        .context("prepared sibling capacity intent disappeared")?;
    if intent.phase == CapacityIntentPhase::Closed {
        let reservation = catalog
            .capacity_reservation(&attempt.capacity_reservation)?
            .context("prepared sibling Catalog reservation disappeared")?;
        if intent.expected_release.as_ref() != Some(&evidence)
            || intent.close_event_id.as_deref()
                != Some(format!("{}:capacity-closed", attempt.attempt_id.0).as_str())
            || reservation.state != CapacityReservationState::Released
            || reservation.launch_token.as_deref() != Some(launch.request.launch_token.as_str())
            || reservation.release_evidence.as_ref() != Some(&evidence)
        {
            bail!("prepared sibling closed capacity differs from retained release proof");
        }
    } else {
        store.release_admitted_no_launch_capacity(catalog, &attempt.capacity_reservation)?;
        store.close_released_capacity_intent(
            catalog,
            &attempt.capacity_reservation,
            &format!("{}:capacity-closed", attempt.attempt_id.0),
        )?;
    }
    let current = store
        .routing_history(scope)?
        .context("prepared sibling history disappeared after release")?
        .attempts
        .into_iter()
        .find(|row| row.attempt_id == attempt.attempt_id)
        .context("prepared sibling attempt disappeared after release")?;
    let expected_usage = prepared_no_create_usage(
        &current.selected.profile.harness_id,
        current.selected.binding.billing_mode,
        scope,
        &attempt.attempt_id,
        &attempt.selected.observation.executable,
    )?;
    match &current.usage {
        recorded if recorded == &expected_usage => {}
        RoutedUsage::Unreported => {
            store.settle_routing_usage(
                scope,
                &format!("{}:usage", attempt.attempt_id.0),
                &attempt.attempt_id,
                current.revision,
                &expected_usage,
                u64::try_from(Utc::now().timestamp_millis())?.max(current.updated_at_ms),
            )?;
        }
        _ => bail!("prepared attempt usage differs from its proven no-create receipt"),
    }
    if let Some(agent) = store.get_agent(&attempt.agent_id)? {
        if agent.run_id != scope.run_id.0 || agent.task_id != attempt.task_id.0 {
            bail!("prepared sibling agent ledger identity changed");
        }
    } else {
        store.insert_agent_with_root(
            &attempt.agent_id,
            &scope.run_id.0,
            &attempt.task_id.0,
            1,
            None,
            &current.selected.profile.harness_id,
            None,
        )?;
    }
    let stopped = ProcessRegistryFile::load(&registry_path(data_dir))?
        .cancelled_runs
        .contains(&scope.run_id.0)
        || cancelled.events.iter().any(|entry| {
            entry.event_id == "controller.stop.cancel.v1"
                && matches!(entry.event, RoutingControlEvent::Cancelled { .. })
        });
    store.finish_agent(
        &attempt.agent_id,
        None,
        if stopped { "cancelled" } else { "failed" },
    )?;
    Ok(())
}

/// A dead controller cannot resume a DAG under new authority. Settled work is
/// fenced; when every task passed, Review may use only retained winners.
#[allow(clippy::too_many_arguments)]
fn reconcile_abandoned_registered_waves(
    catalog: &Catalog,
    draft_id: &str,
    plan: &FlowPlan,
    owner: &RoutedFlowDispatchOwner,
    repo: &Path,
    data_dir: &Path,
    store_path: &Path,
    expected_file_identity: &str,
    store_file_guard: &pytxo_runner::FileIdentityGuard,
) -> anyhow::Result<bool> {
    if validate_routed_local_wave_shape(plan).is_err() {
        return Ok(false);
    }
    let Some(review) = plan.routing.as_ref() else {
        return Ok(false);
    };
    let scope = RoutingScope {
        domain_id: review.authorization.domain_id.clone(),
        run_id: review.authorization.run_id.clone(),
    };
    let run_id = &scope.run_id.0;
    let active_path = data_dir.join("active_run.json");
    let gate = ActiveRunGate::acquire(&active_path)?;
    if store_file_guard.identity() != expected_file_identity
        || pytxo_runner::file_identity(store_path)? != expected_file_identity
        || read_active_run_state(&active_path)?.is_some_and(|active| active.run_id != *run_id)
    {
        return Ok(false);
    }
    let store = PytxoStore::open_existing_read_write(store_path)?;
    let Some(run) = store.get_run(run_id)? else {
        return Ok(false);
    };
    if run.status != "starting"
        || run.repo_root != repo.to_string_lossy()
        || run.permission_profile.as_deref() != Some("orbit")
        || pytxo_runner::file_identity(store_path)? != expected_file_identity
    {
        return Ok(false);
    }
    let Some(history) = store.routing_history(&scope)? else {
        return Ok(false);
    };
    let passed_count = history
        .tasks
        .iter()
        .filter(|task| task.winner.is_some())
        .count();
    if canonical_digest(&history.mission, 1)? != review.mission_digest
        || history.tasks.len() != plan.tasks.len()
        || history.attempts.is_empty()
        || history.attempts.len() > review.authorization.limits.max_attempts as usize
        || passed_count > plan.tasks.len()
    {
        return Ok(false);
    }
    let parallel = routed_parallel_siblings(plan);
    let mut previous_winner: Option<pytxo_core::routing::VerifiedDependencyOutput> = None;
    for (index, planned) in plan.tasks.iter().enumerate() {
        let Some(task) = history
            .tasks
            .iter()
            .find(|row| row.registration.contract.task_id.0 == planned.id)
        else {
            return Ok(false);
        };
        if task.registration.contract.dependencies
            != planned
                .dependencies
                .iter()
                .cloned()
                .map(pytxo_core::TaskId)
                .collect::<Vec<_>>()
        {
            return Ok(false);
        }
        if (parallel && task.winner.is_none()) || (!parallel && index >= passed_count) {
            let settled_repair = task.current_attempt.as_ref().is_some_and(|id| {
                history.attempts.iter().any(|attempt| {
                    attempt.attempt_id == *id
                        && attempt.task_id == task.registration.contract.task_id
                        && attempt.ordinal == 1
                        && (attempt.state == pytxo_core::routing::AttemptState::Failed)
                        && attempt.ownership_released
                        && task.state == TaskRoutingState::Ready
                        && task.next_ordinal == 2
                })
            });
            let settled_terminal_sibling = parallel
                && task.current_attempt.as_ref().is_some_and(|id| {
                    history.attempts.iter().any(|attempt| {
                        attempt.attempt_id == *id
                            && attempt.task_id == task.registration.contract.task_id
                            && attempt.ordinal == 1
                            && attempt.state.is_terminal()
                            && attempt.ownership_released
                            && history.cancelled
                    })
                });
            let settled_terminal_no_create = if plan.tasks.len() == 1
                && task.state == TaskRoutingState::Cancelled
            {
                task.current_attempt
                    .as_ref()
                    .and_then(|id| {
                        history.attempts.iter().find(|attempt| {
                            attempt.attempt_id == *id
                                && attempt.task_id == task.registration.contract.task_id
                        })
                    })
                    .map(|attempt| -> anyhow::Result<bool> {
                        if attempt.ordinal == 2 {
                            let Some(prior) = attempt.predecessor.as_ref().and_then(|id| {
                                history.attempts.iter().find(|row| row.attempt_id == *id)
                            }) else {
                                return Ok(false);
                            };
                            if history.mission.policy.version != "claude-proposal-rules-repair-v1"
                                || !settled_actionable_claude_check_predecessor_is_safe(
                                    &store, catalog, &scope, prior,
                                )?
                            {
                                return Ok(false);
                            }
                        } else if attempt.ordinal != 1 || attempt.predecessor.is_some() {
                            return Ok(false);
                        }
                        settled_no_create_attempt_is_safe(&store, catalog, &history, attempt)
                    })
                    .transpose()?
                    .unwrap_or(false)
            } else {
                false
            };
            if task.winner.is_some()
                || (task.current_attempt.is_some()
                    && !settled_repair
                    && !settled_terminal_sibling
                    && !settled_terminal_no_create)
            {
                return Ok(false);
            }
            continue;
        }
        let Some(winner) = task.winner.as_ref() else {
            return Ok(false);
        };
        let Some(attempt) = history
            .attempts
            .iter()
            .find(|attempt| attempt.attempt_id == winner.winning_attempt_id)
        else {
            return Ok(false);
        };
        if attempt.ordinal == 2 {
            let Some(prior) = history.attempts.iter().find(|prior| {
                Some(&prior.attempt_id) == attempt.predecessor.as_ref()
                    && prior.task_id == attempt.task_id
            }) else {
                return Ok(false);
            };
            if prior.ordinal != 1
                || prior.state != pytxo_core::routing::AttemptState::Failed
                || !prior.ownership_released
                || !prior.failure.as_ref().is_some_and(|failure| {
                    matches!(
                        failure.failure_class,
                        pytxo_core::routing::AttemptFailureClass::Implementation
                            | pytxo_core::routing::AttemptFailureClass::Check
                    ) && failure.actionable_evidence_digest.is_some()
                })
            {
                return Ok(false);
            }
        }
        if task.state != TaskRoutingState::Succeeded
            || task.current_attempt.as_ref() != Some(&attempt.attempt_id)
            || winner.winning_attempt_id != attempt.attempt_id
            || winner.task_id != task.registration.contract.task_id
            || attempt.task_id != winner.task_id
            || attempt.state != pytxo_core::routing::AttemptState::Passed
            || !attempt.ownership_released
            || matches!(attempt.usage, pytxo_store::routing::RoutedUsage::Unreported)
            || attempt.receipts.sealed_output.as_ref() != Some(&winner.output_digest)
            || attempt.receipts.checks.as_ref() != Some(&winner.verification_receipt_digest)
            || attempt.dependencies
                != if parallel {
                    vec![]
                } else {
                    previous_winner
                        .as_ref()
                        .map(|winner| vec![winner.clone()])
                        .unwrap_or_default()
                }
        {
            return Ok(false);
        }
        if !parallel {
            previous_winner = Some(winner.clone());
        }
    }
    if store
        .unresolved_routing_attempts()?
        .iter()
        .any(|attempt| attempt.scope == scope)
        || store.unresolved_capacity_intent_scopes()?.contains(&scope)
        || !store
            .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))?
            .is_empty()
        || catalog
            .unresolved_capacity_reservations()?
            .iter()
            .any(|reservation| {
                reservation.domain_id == scope.domain_id.0 && reservation.run_id == *run_id
            })
        || !registry_matches_settled_routed_owners(
            &ProcessRegistryFile::load(&registry_path(data_dir))?,
            &store,
            &history,
            &scope,
            repo,
        )?
    {
        return Ok(false);
    }
    let stop_requested = catalog.routed_flow_stop_requested(draft_id, run_id)? == Some(true);
    let stopped = stop_requested
        || ProcessRegistryFile::load(&registry_path(data_dir))?
            .cancelled_runs
            .contains(run_id)
        || history
            .events
            .iter()
            .any(|entry| entry.event_id == "controller.stop.cancel.v1");
    let preserve_winners = passed_count == plan.tasks.len() && !stopped && !history.cancelled;
    if !preserve_winners && !history.cancelled {
        let now_ms = u64::try_from(Utc::now().timestamp_millis())?;
        store.cancel_routing_mission(
            &scope,
            "controller.recovery.settled-prefix.v1",
            history.cancel_epoch,
            now_ms,
        )?;
    }
    if preserve_winners {
        let contract = store
            .get_run_contract(run_id)?
            .context("settled routed Review has no contract")?;
        if contract.apply_status == "pending" && !store.begin_run_preparation(run_id)? {
            return Ok(false);
        }
        if matches!(contract.apply_status.as_str(), "pending" | "preparing") {
            store.fail_run_preparation(
                run_id,
                &crate::run_apply_error(
                    "routed_review_failed",
                    "controller died after all routed attempts passed, before Review publication",
                    false,
                    None,
                ),
            )?;
        } else if contract.apply_status != "review_failed" {
            return Ok(false);
        }
    }
    let status = if stopped { "cancelled" } else { "failed" };
    if !store.finish_run_if_status(run_id, "starting", status)? {
        return Ok(false);
    }
    drop(gate);
    let history = store
        .routing_history(&scope)?
        .context("fenced between-wave mission disappeared")?;
    if !clear_terminal_routed_active_run_if_safe(
        catalog,
        data_dir,
        repo,
        &store,
        &scope,
        &history,
        preserve_winners,
        owner,
    )? {
        return Ok(false);
    }
    if pytxo_runner::file_identity(store_path)? != expected_file_identity {
        return Ok(false);
    }
    if stopped && stop_requested {
        catalog.mark_routed_flow_stopped_after_quiescence(draft_id, run_id)?;
    } else {
        catalog.mark_routed_flow_recovered_dispatched(draft_id, run_id)?;
    }
    Ok(true)
}

/// Settle an ambiguous routed startup only from the reviewed run's durable
/// terminal state, cancelled mission (if registration committed), and empty
/// process/attempt/capacity ownership. This never reopens dispatch authority.
pub fn reconcile_routed_flow_startup(catalog: &Catalog, draft_id: &str) -> anyhow::Result<bool> {
    let draft = catalog
        .get_flow_draft(draft_id)?
        .with_context(|| format!("Flow draft not found: {draft_id}"))?;
    if !matches!(draft.status.as_str(), "recovery_required" | "dispatching") {
        return Ok(false);
    }
    let run_id = draft
        .dispatched_run_id
        .as_deref()
        .context("recovering routed Flow has no reviewed RunId")?;
    let domain_id = draft
        .domain_id
        .as_deref()
        .context("recovering routed Flow has no execution domain")?;
    let plan: FlowPlan = serde_json::from_str(
        draft
            .plan_json
            .as_deref()
            .context("recovering routed Flow has no reviewed plan")?,
    )?;
    let review = plan
        .routing
        .as_ref()
        .context("recovering Flow has no routed review")?;
    if plan.draft_id != draft_id
        || plan.domain_id != domain_id
        || review.authorization.run_id.0 != run_id
        || review.authorization.domain_id.as_str() != domain_id
    {
        bail!("recovering routed Flow no longer matches the reviewed run");
    }
    let Some(owner) = catalog.routed_flow_dispatch_owner(draft_id, run_id)? else {
        // Older Catalog versions did not record an owner or original Store.
        return Ok(false);
    };
    if catalog.routed_flow_startup_may_have_started(draft_id, run_id)? == Some(false) {
        // The startup call was never entered. Only the original controller's
        // confirmed death permits this no-run settlement.
        if pytxo_runner::process_matches(owner.controller_pid, &owner.controller_start_identity)? {
            return Ok(false);
        }
        if catalog.routed_flow_stop_requested(draft_id, run_id)? == Some(true) {
            catalog.mark_routed_flow_stopped_after_quiescence(draft_id, run_id)?;
        } else if draft.status == "dispatching" {
            catalog.mark_routed_flow_startup_failed(draft_id, run_id)?;
        } else {
            catalog.mark_routed_flow_recovered_failed(draft_id, run_id)?;
        }
        return Ok(true);
    }
    let repo = resolve_repo_root(Some(Path::new(domain_id)))?;
    if pytxo_core::DomainId::from_repo_root(&repo)?.as_str() != domain_id {
        bail!("recovering routed Flow execution domain changed");
    }
    let store_path = std::path::PathBuf::from(&owner.store_db_path);
    if !store_path.is_absolute() || store_path.file_name() != Some(std::ffi::OsStr::new("pytxo.db"))
    {
        return Ok(false);
    }
    let Some(data_dir) = store_path.parent() else {
        return Ok(false);
    };
    let data_dir = data_dir.to_path_buf();
    let Some(expected_file_identity) = owner.store_db_file_identity.as_deref() else {
        // v3/v4 claims only retained a path. They cannot prove that the file
        // now at that path is the one the original controller opened.
        return Ok(false);
    };
    let store_file_guard = match pytxo_runner::FileIdentityGuard::acquire(&store_path) {
        Ok(guard) => guard,
        // Recovery is advisory to the preserved claim. If this platform
        // cannot exclude path replacement, retain recovery_required.
        Err(_) => return Ok(false),
    };
    if store_file_guard.identity() != expected_file_identity {
        return Ok(false);
    }
    // Close only exact no-create sibling owners before the normal settled-run
    // paths inspect unresolved ownership. A killed controller is never resumed.
    reconcile_abandoned_prepared_attempts(
        catalog,
        &plan,
        &owner,
        &repo,
        &data_dir,
        &store_path,
        expected_file_identity,
        &store_file_guard,
    )?;
    let store = PytxoStore::open_existing_read_only(&store_path)?;
    let run = match store.get_run(run_id)? {
        Some(run) if run.status != "starting" => run,
        _ => {
            drop(store);
            if pytxo_runner::process_matches(
                owner.controller_pid,
                &owner.controller_start_identity,
            )? {
                return Ok(false);
            }
            if draft.status == "dispatching" {
                return Ok(false);
            }
            if reconcile_abandoned_registered_waves(
                catalog,
                draft_id,
                &plan,
                &owner,
                &repo,
                &data_dir,
                &store_path,
                expected_file_identity,
                &store_file_guard,
            )? {
                return Ok(true);
            }
            if reconcile_abandoned_registered_no_attempt(
                catalog,
                draft_id,
                &plan,
                &owner,
                &repo,
                &data_dir,
                &store_path,
                expected_file_identity,
                &store_file_guard,
            )? {
                return Ok(true);
            }
            return reconcile_abandoned_routed_startup(
                catalog,
                draft_id,
                run_id,
                domain_id,
                owner.controller_pid,
                &owner.controller_start_identity,
                &repo,
                &data_dir,
                &store_path,
                expected_file_identity,
                &store_file_guard,
            );
        }
    };
    if run.repo_root != repo.to_string_lossy()
        || !matches!(
            run.status.as_str(),
            "failed_startup" | "cancelled" | "failed" | "completed"
        )
    {
        return Ok(false);
    }
    let scope = pytxo_store::routing::RoutingScope {
        domain_id: review.authorization.domain_id.clone(),
        run_id: review.authorization.run_id.clone(),
    };
    let history = store.routing_history(&scope)?;
    let failed_after_passed_review = if run.status == "failed" {
        store.get_run_contract(run_id)?.is_some_and(|contract| {
            matches!(
                contract.apply_status.as_str(),
                "review_failed" | "discarded"
            ) && contract.prepared_manifest.is_none()
                && contract.prepared_digest.is_none()
                && contract.apply_manifest_json.is_none()
                && contract
                    .last_apply_error
                    .as_ref()
                    .is_some_and(|error| error.code == "routed_review_failed")
        }) && pytxo_runner::apply_attempt_ids(&data_dir, run_id)?.is_empty()
    } else {
        false
    };
    if matches!(run.status.as_str(), "failed" | "completed") {
        let Some(history) = history.as_ref() else {
            return Ok(false);
        };
        if canonical_digest(&history.mission, 1)? != review.mission_digest
            || history.attempts.is_empty()
            || history
                .attempts
                .iter()
                .any(|attempt| !attempt.state.is_terminal())
            || (run.status == "failed" && history.cancelled == failed_after_passed_review)
            || (run.status == "completed" && history.cancelled)
        {
            return Ok(false);
        }
        if run.status == "completed" || failed_after_passed_review {
            if history.tasks.len() != plan.tasks.len()
                || history.attempts.len() < plan.tasks.len()
                || history.attempts.len() > review.authorization.limits.max_attempts as usize
                || validate_routed_local_wave_shape(&plan).is_err()
            {
                return Ok(false);
            }
            let parallel = routed_parallel_siblings(&plan);
            let mut previous_winner: Option<pytxo_core::routing::VerifiedDependencyOutput> = None;
            for planned in &plan.tasks {
                let Some(task) = history
                    .tasks
                    .iter()
                    .find(|row| row.registration.contract.task_id.0 == planned.id)
                else {
                    return Ok(false);
                };
                let Some(winner) = task.winner.as_ref() else {
                    return Ok(false);
                };
                let Some(attempt) = history
                    .attempts
                    .iter()
                    .find(|row| row.attempt_id == winner.winning_attempt_id)
                else {
                    return Ok(false);
                };
                if attempt.ordinal == 2 {
                    let Some(prior) = history.attempts.iter().find(|prior| {
                        Some(&prior.attempt_id) == attempt.predecessor.as_ref()
                            && prior.task_id == attempt.task_id
                    }) else {
                        return Ok(false);
                    };
                    if prior.ordinal != 1
                        || prior.state != pytxo_core::routing::AttemptState::Failed
                        || !prior.ownership_released
                        || !prior.failure.as_ref().is_some_and(|failure| {
                            matches!(
                                failure.failure_class,
                                pytxo_core::routing::AttemptFailureClass::Implementation
                                    | pytxo_core::routing::AttemptFailureClass::Check
                            ) && failure.actionable_evidence_digest.is_some()
                        })
                    {
                        return Ok(false);
                    }
                }
                if task.state != TaskRoutingState::Succeeded
                    || task.registration.contract.dependencies
                        != planned
                            .dependencies
                            .iter()
                            .cloned()
                            .map(pytxo_core::TaskId)
                            .collect::<Vec<_>>()
                    || winner.task_id != task.registration.contract.task_id
                    || attempt.task_id != winner.task_id
                    || attempt.state != pytxo_core::routing::AttemptState::Passed
                    || !attempt.ownership_released
                    || attempt.receipts.sealed_output.as_ref() != Some(&winner.output_digest)
                    || attempt.receipts.checks.as_ref() != Some(&winner.verification_receipt_digest)
                    || if parallel {
                        !attempt.dependencies.is_empty()
                    } else {
                        previous_winner
                            .as_ref()
                            .map(|winner| vec![winner.clone()])
                            .unwrap_or_default()
                            != attempt.dependencies
                    }
                {
                    return Ok(false);
                }
                if !parallel {
                    previous_winner = Some(winner.clone());
                }
            }
        } else {
            let won = history
                .tasks
                .iter()
                .filter(|task| task.winner.is_some())
                .collect::<Vec<_>>();
            let parallel = routed_parallel_siblings(&plan);
            if won.len() > if parallel { 2 } else { 1 }
                || won.iter().any(|task| {
                    plan.tasks.len() != 2
                        || (!parallel && task.registration.contract.task_id.0 != plan.tasks[0].id)
                        || (parallel
                            && !plan.tasks.iter().any(|planned| {
                                planned.id == task.registration.contract.task_id.0
                                    && planned.dependencies.is_empty()
                            }))
                        || task.state != TaskRoutingState::Succeeded
                        || !history.attempts.iter().any(|attempt| {
                            task.winner.as_ref().is_some_and(|winner| {
                                winner.winning_attempt_id == attempt.attempt_id
                                    && attempt.state == pytxo_core::routing::AttemptState::Passed
                                    && attempt.ownership_released
                                    && attempt.receipts.sealed_output.as_ref()
                                        == Some(&winner.output_digest)
                                    && attempt.receipts.checks.as_ref()
                                        == Some(&winner.verification_receipt_digest)
                            })
                        })
                })
            {
                return Ok(false);
            }
        }
    } else if history.as_ref().is_some_and(|history| !history.cancelled) {
        return Ok(false);
    }
    if pytxo_runner::file_identity(&store_path)? != expected_file_identity {
        return Ok(false);
    }
    let cleared = if let Some(history) = history.as_ref() {
        clear_terminal_routed_active_run_if_safe(
            catalog,
            &data_dir,
            &repo,
            &store,
            &scope,
            history,
            failed_after_passed_review,
            &owner,
        )?
    } else {
        let active_path = data_dir.join("active_run.json");
        clear_stopped_active_run_if_safe(&active_path, &data_dir, &store, run_id)?
    };
    if !cleared {
        return Ok(false);
    }
    if run.status == "cancelled"
        && catalog.routed_flow_stop_requested(draft_id, run_id)? == Some(true)
    {
        catalog.mark_routed_flow_stopped_after_quiescence(draft_id, run_id)?;
    } else if matches!(run.status.as_str(), "failed" | "completed") {
        catalog.mark_routed_flow_recovered_dispatched(draft_id, run_id)?;
    } else if draft.status == "dispatching" {
        catalog.mark_routed_flow_startup_failed(draft_id, run_id)?;
    } else {
        catalog.mark_routed_flow_recovered_failed(draft_id, run_id)?;
    }
    Ok(true)
}

/// Registration committed, but the dead controller never reached a routing
/// decision or attempt. Fence the reviewed mission before settling its Run.
/// A cancellation-only journal permits replay after a crash between writes.
#[allow(clippy::too_many_arguments)]
fn reconcile_abandoned_registered_no_attempt(
    catalog: &Catalog,
    draft_id: &str,
    plan: &FlowPlan,
    owner: &RoutedFlowDispatchOwner,
    repo: &Path,
    data_dir: &Path,
    store_path: &Path,
    expected_file_identity: &str,
    store_file_guard: &pytxo_runner::FileIdentityGuard,
) -> anyhow::Result<bool> {
    if validate_routed_local_wave_shape(plan).is_err() {
        return Ok(false);
    }
    let Some(review) = plan.routing.as_ref() else {
        return Ok(false);
    };
    let scope = RoutingScope {
        domain_id: review.authorization.domain_id.clone(),
        run_id: review.authorization.run_id.clone(),
    };
    let run_id = &scope.run_id.0;
    let active_path = data_dir.join("active_run.json");
    let _gate = ActiveRunGate::acquire(&active_path)?;
    let marker = read_active_run_state(&active_path)?;
    if marker.as_ref().is_some_and(|active| {
        active.run_id != *run_id
            || active.repo_root != repo.to_string_lossy()
            || active.supervisor_pid != owner.controller_pid
            || active.supervisor_start_identity.as_deref()
                != Some(owner.controller_start_identity.as_str())
    }) || pytxo_runner::process_matches(owner.controller_pid, &owner.controller_start_identity)?
        || store_file_guard.identity() != expected_file_identity
        || pytxo_runner::file_identity(store_path)? != expected_file_identity
    {
        return Ok(false);
    }
    let store = PytxoStore::open_existing_read_write(store_path)?;
    let Some(run) = store.get_run(run_id)? else {
        return Ok(false);
    };
    let Some(history) = store.routing_history(&scope)? else {
        return Ok(false);
    };
    let advisor_requests = store.routing_advisor_requests_for_recovery(&scope)?;
    let recoverable_hosted_request = match advisor_requests.as_slice() {
        [] => None,
        [request] if plan.tasks.len() == 1 && history.tasks.len() == 1 => {
            let contract = &history.tasks[0].registration.contract;
            let expected_packet = reviewed_task_advisor_packet(contract)?.digest();
            if history.mission.policy.mode != RoutingMode::Shadow
                || history.mission.policy.advisor_recipient.as_deref() != Some(HOSTED_RECIPIENT)
                || history.mission.authorization.live_advice_authorized
                || request.scope != scope
                || request.task_id != contract.task_id
                || request.ordinal != 1
                || request.recipient_identity.as_deref() != Some(HOSTED_RECIPIENT)
                || request.scope_digest != history.mission.policy.disclosure_scope_digest
                || request.policy_digest != history.mission.authorization.policy_digest
                || request.packet_digest != expected_packet
                || request.consent_revision != history.mission.authorization.consent_revision
                || request.task_revision != contract.revision
                || request.task_state_revision != 1
                || request.authorization_revision != history.mission.authorization.revision
                || request.cancel_epoch != history.mission.authorization.cancel_epoch
            {
                return Ok(false);
            }
            Some(request)
        }
        _ => return Ok(false),
    };
    let mission_digest = canonical_digest(&history.mission, 1)?;
    let registered = history.events.first().is_some_and(|entry| {
        entry.event_id == "registered"
            && matches!(&entry.event, RoutingControlEvent::Registered { digest } if *digest == mission_digest)
    });
    let hosted_shadow_single = plan.tasks.len() == 1
        && history.tasks.len() == 1
        && history.mission.policy.mode == RoutingMode::Shadow
        && history.mission.policy.advisor_recipient.as_deref() == Some(HOSTED_RECIPIENT);
    let qualified_count = if hosted_shadow_single {
        history
            .events
            .iter()
            .skip(1)
            .take_while(|entry| matches!(entry.event, RoutingControlEvent::Qualified { .. }))
            .count()
    } else {
        0
    };
    if qualified_count > history.mission.profiles.len()
        || (recoverable_hosted_request.is_some()
            && qualified_count != history.mission.profiles.len())
    {
        return Ok(false);
    }
    let qualified_prefix = history
        .events
        .get(1..1 + qualified_count)
        .is_some_and(|events| {
            events.iter().enumerate().all(|(ordinal, entry)| {
                entry.event_id
                    == format!(
                        "{}:{}:fixture-qualification:{ordinal}",
                        scope.run_id.0, history.tasks[0].registration.contract.task_id.0
                    )
                    && matches!(&entry.event, RoutingControlEvent::Qualified { qualification }
                        if qualification.permission_profile == PermissionProfile::Orbit
                            && qualification.receipt_digest.is_valid()
                            && qualification.launch_fingerprint.is_valid()
                            && qualification.tool_probe_passed
                            && qualification.cancellation_probe_passed
                            && qualification.quiescence_probe_passed)
            })
        });
    let recovery_event_id = "controller.recovery.registered-no-attempt.v1";
    let replaying_cancel = history.events.len() == 2 + qualified_count
        && history
            .events
            .get(1 + qualified_count)
            .is_some_and(|entry| {
                entry.event_id == recovery_event_id
                    && matches!(&entry.event, RoutingControlEvent::Cancelled { expected_epoch, .. }
                    if *expected_epoch == history.mission.authorization.cancel_epoch)
            });
    if run.status != "starting"
        || run.repo_root != repo.to_string_lossy()
        || run.permission_profile.as_deref() != Some("orbit")
        || history.mission.authorization.domain_id != scope.domain_id
        || history.mission.authorization.run_id != scope.run_id
        || mission_digest != review.mission_digest
        || !registered
        || !qualified_prefix
        || !(history.events.len() == 1 + qualified_count || replaying_cancel)
        || history.cancelled != replaying_cancel
        || !history.attempts.is_empty()
        || history.tasks.len() != plan.tasks.len()
        || history.tasks.iter().any(|task| {
            let planned = plan
                .tasks
                .iter()
                .find(|planned| planned.id == task.registration.contract.task_id.0);
            !planned.is_some_and(|planned| {
                task.registration.contract.dependencies
                    == planned
                        .dependencies
                        .iter()
                        .cloned()
                        .map(TaskId)
                        .collect::<Vec<_>>()
            }) || task.current_attempt.is_some()
                || task.winner.is_some()
                || task.last_decision.is_some()
                || task.next_ordinal != 1
                || task.revision != if replaying_cancel { 2 } else { 1 }
                || task.state
                    != if replaying_cancel {
                        TaskRoutingState::Cancelled
                    } else if task.registration.contract.dependencies.is_empty() {
                        TaskRoutingState::Ready
                    } else {
                        TaskRoutingState::WaitingDependencies
                    }
        })
        || store
            .unresolved_routing_attempts()?
            .iter()
            .any(|attempt| attempt.scope == scope)
        || store.unresolved_capacity_intent_scopes()?.contains(&scope)
        || !store
            .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))?
            .is_empty()
        || !store.list_agents_for_run(run_id)?.is_empty()
        || !ProcessRegistryFile::load(&registry_path(data_dir))?
            .for_run(run_id)
            .is_empty()
        || catalog
            .unresolved_capacity_reservations()?
            .iter()
            .any(|reservation| {
                reservation.domain_id == scope.domain_id.0 && reservation.run_id == *run_id
            })
        || !pytxo_runner::apply_attempt_ids(data_dir, run_id)?.is_empty()
        || store.get_run_contract(run_id)?.is_some_and(|contract| {
            contract.apply_status != "pending"
                || contract.prepared_manifest.is_some()
                || contract.prepared_digest.is_some()
                || contract.prepared_at.is_some()
                || contract.apply_manifest_json.is_some()
                || contract.applied_at.is_some()
                || contract.last_apply_error.is_some()
                || contract.recovery_state.is_some()
        })
        || pytxo_runner::file_identity(store_path)? != expected_file_identity
    {
        return Ok(false);
    }
    if let Some(request) = recoverable_hosted_request {
        let now_ms = u64::try_from(Utc::now().timestamp_millis())?.max(request.updated_at_ms);
        match request.phase {
            AdvisorSendPhase::Prepared => {
                store.abandon_prepared_routing_advisor_request(
                    &scope,
                    &request.request_id,
                    now_ms,
                )?;
            }
            AdvisorSendPhase::SendingMayHaveHappened => {
                store.settle_routing_advisor_request(&scope, &request.request_id, None, now_ms)?;
            }
            AdvisorSendPhase::Completed
            | AdvisorSendPhase::Uncertain
            | AdvisorSendPhase::LateReceipt => {}
        }
    }
    if !replaying_cancel {
        let now_ms = u64::try_from(Utc::now().timestamp_millis())?;
        store.cancel_routing_mission(&scope, recovery_event_id, history.cancel_epoch, now_ms)?;
    }
    let stop_requested = catalog.routed_flow_stop_requested(draft_id, run_id)? == Some(true);
    let status = if stop_requested {
        "cancelled"
    } else {
        "failed_startup"
    };
    if !store.finish_run_if_status(run_id, "starting", status)? {
        return Ok(false);
    }
    if pytxo_runner::file_identity(store_path)? != expected_file_identity {
        return Ok(false);
    }
    if marker.is_some() {
        crate::clear_active_run_unlocked(&active_path, run_id)?;
    }
    if stop_requested {
        catalog.mark_routed_flow_stopped_after_quiescence(draft_id, run_id)?;
    } else {
        catalog.mark_routed_flow_recovered_failed(draft_id, run_id)?;
    }
    Ok(true)
}

/// The controller died after its pre-start bit, but before mission
/// registration. An absent marker or its exact dead owner may be cleared only
/// with the original physical Store and an ownership-negative snapshot.
#[allow(clippy::too_many_arguments)]
fn reconcile_abandoned_routed_startup(
    catalog: &Catalog,
    draft_id: &str,
    run_id: &str,
    domain_id: &str,
    controller_pid: u32,
    controller_start_identity: &str,
    repo: &Path,
    data_dir: &Path,
    store_path: &Path,
    expected_file_identity: &str,
    store_file_guard: &pytxo_runner::FileIdentityGuard,
) -> anyhow::Result<bool> {
    let active_path = data_dir.join("active_run.json");
    let _gate = ActiveRunGate::acquire(&active_path)?;
    let marker = read_active_run_state(&active_path)?;
    if marker.as_ref().is_some_and(|marker| {
        marker.run_id != run_id
            || marker.repo_root != repo.to_string_lossy()
            || marker.supervisor_pid != controller_pid
            || marker.supervisor_start_identity.as_deref() != Some(controller_start_identity)
    }) || pytxo_runner::process_matches(controller_pid, controller_start_identity)?
        || store_file_guard.identity() != expected_file_identity
        || pytxo_runner::file_identity(store_path)? != expected_file_identity
    {
        return Ok(false);
    }
    let store = PytxoStore::open_existing_read_write(store_path)?;
    if pytxo_runner::file_identity(store_path)? != expected_file_identity {
        return Ok(false);
    }
    let run = store.get_run(run_id)?;
    if run.as_ref().is_some_and(|run| {
        run.status != "starting"
            || run.repo_root != repo.to_string_lossy()
            || run.permission_profile.as_deref() != Some("orbit")
    }) {
        return Ok(false);
    }
    let scope = pytxo_store::routing::RoutingScope {
        domain_id: pytxo_core::DomainId(domain_id.to_owned()),
        run_id: RunId(run_id.to_owned()),
    };
    if store.routing_history(&scope)?.is_some()
        || store
            .unreconciled_registered_routing_scopes()?
            .contains(&scope)
        || store
            .unresolved_routing_attempts()?
            .iter()
            .any(|attempt| attempt.scope == scope)
        || store.unresolved_capacity_intent_scopes()?.contains(&scope)
        || !store
            .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))?
            .is_empty()
        || !ProcessRegistryFile::load(&registry_path(data_dir))?
            .for_run(run_id)
            .is_empty()
        || catalog
            .unresolved_capacity_reservations()?
            .iter()
            .any(|reservation| reservation.domain_id == domain_id && reservation.run_id == run_id)
        || pytxo_runner::file_identity(store_path)? != expected_file_identity
    {
        return Ok(false);
    }
    let stop_requested = catalog.routed_flow_stop_requested(draft_id, run_id)? == Some(true);
    let terminal_status = if stop_requested {
        "cancelled"
    } else {
        "failed_startup"
    };
    if run.is_some() && !store.finish_run_if_status(run_id, "starting", terminal_status)? {
        return Ok(false);
    }
    if pytxo_runner::file_identity(store_path)? != expected_file_identity {
        return Ok(false);
    }
    if marker.is_some() {
        crate::clear_active_run_unlocked(&active_path, run_id)?;
    }
    if stop_requested {
        catalog.mark_routed_flow_stopped_after_quiescence(draft_id, run_id)?;
    } else {
        catalog.mark_routed_flow_recovered_failed(draft_id, run_id)?;
    }
    Ok(true)
}

/// History refresh is a controller entry point for exact startup recovery.
/// Any unresolved row remains visible as recovery_required.
pub fn list_flow_drafts_with_routed_recovery(
    catalog: &Catalog,
) -> anyhow::Result<Vec<FlowDraftRecord>> {
    for draft in catalog.list_flow_drafts()? {
        let mut needs_recovery = draft.status == "recovery_required";
        if draft.status == "dispatching" {
            if let Some(run_id) = draft.dispatched_run_id.as_deref() {
                let observe_owner = (|| -> anyhow::Result<bool> {
                    if let Some(owner) = catalog.routed_flow_dispatch_owner(&draft.id, run_id)? {
                        // A terminal run may be awaiting only the Catalog
                        // acknowledgement while this controller is still live.
                        needs_recovery = true;
                        if !pytxo_runner::process_matches(
                            owner.controller_pid,
                            &owner.controller_start_identity,
                        )? {
                            return Ok(
                                catalog.mark_routed_flow_owner_lost(&draft.id, run_id, &owner)?
                            );
                        }
                    }
                    Ok(false)
                })();
                match observe_owner {
                    Ok(transitioned) => needs_recovery |= transitioned,
                    Err(error) => {
                        tracing::warn!(draft_id = %draft.id, %error, "routed Flow owner observation deferred");
                    }
                }
            }
        }
        if needs_recovery {
            if let Err(error) = reconcile_routed_flow_startup(catalog, &draft.id) {
                tracing::warn!(draft_id = %draft.id, %error, "routed Flow startup recovery deferred");
            }
        }
    }
    Ok(catalog.list_flow_drafts()?)
}

fn dispatch_flow_scoped(
    catalog: &Catalog,
    draft_id: &str,
    desktop_beta: bool,
    experimental_routed: bool,
    hosted_client: Option<Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>>,
) -> anyhow::Result<String> {
    let _upgrade_guard = pytxo_core::UpgradeGuard::work()?;
    let draft = catalog
        .get_flow_draft(draft_id)?
        .with_context(|| format!("Flow draft not found: {draft_id}"))?;
    let hosted_experiment = hosted_client.is_some() && experimental_routed && !desktop_beta;
    if draft.status == FlowStatus::ReviewOnly.as_str() && !hosted_experiment {
        bail!("review-only hosted Shadow Flow cannot be dispatched");
    }
    let reviewed_status = if hosted_experiment {
        FlowStatus::ReviewOnly
    } else {
        FlowStatus::Ready
    };
    if draft.status != reviewed_status.as_str() || draft.plan_json.is_none() {
        bail!("Flow dispatch requires a persisted ready preview");
    }
    let expected_plan_json = draft.plan_json.as_deref().unwrap();
    let plan: FlowPlan =
        serde_json::from_str(expected_plan_json).context("invalid persisted Flow preview")?;
    if plan.status != reviewed_status || !plan.blocked_reasons.is_empty() {
        bail!("Flow dispatch requires a persisted ready preview");
    }
    match (experimental_routed, plan.routing.as_ref()) {
        (false, Some(_)) => {
            bail!("routed Flow requires the experimental routed dispatch entrypoint")
        }
        (true, None) => bail!("experimental routed dispatch requires a routed review"),
        _ => {}
    }
    if draft.domain_id.as_deref() != Some(plan.domain_id.as_str())
        || draft.project_id != plan.project_id
    {
        bail!("Flow execution domain changed; generate a new preview");
    }
    let repo = resolve_repo_root(Some(Path::new(&plan.domain_id)))?;
    let cfg = load_config_for_repo(None, &repo)?;
    let entitlements = crate::entitlements::effective_entitlements(&cfg)
        .map_err(|error| anyhow::anyhow!(error))?;
    let (mut cfg, _) = apply_flow_permission_ceiling(&cfg, entitlements.permission_ceiling);
    if experimental_routed && cfg.requested_permission_profile != Some(PermissionProfile::Orbit) {
        bail!("routed Flow requested permission profile changed; generate a new preview");
    }
    if plan.max_workers == 0 || plan.max_workers > cfg.max_agents {
        bail!(
            "Flow worker limit is missing or exceeds current configuration; generate a new preview"
        );
    }
    // An increased config limit cannot widen an already reviewed mission. A lower
    // limit requires a new preview; task count never becomes worker authority.
    cfg.max_agents = plan.max_workers;
    if cfg.permission_profile.as_str() != plan.permission_profile {
        bail!("Flow permission profile changed; generate a new preview");
    }
    if cfg.isolation.as_str() != plan.isolation_mode
        || pytxo_runner::effective_isolation_mode(&cfg).as_str() != plan.isolation_backend_intent
        || format!("{:?}", cfg.execution_backend).to_ascii_lowercase() != plan.execution_backend
    {
        bail!("Flow execution policy changed; generate a new preview");
    }
    if experimental_routed && crate::routed_fixture::fixture_opted_in() {
        crate::hypervisor::require_routed_worktree_isolation(&cfg)?;
    }
    let tasks = runtime_tasks(&plan);
    if experimental_routed
        && tasks
            .iter()
            .any(|task| cfg.resolve_profile_for_agent(&task.agent) != PermissionProfile::Orbit)
    {
        bail!("experimental routed Flow requires Orbit for every task");
    }
    if desktop_beta {
        let blockers = desktop_beta_blockers(&cfg, &tasks, plan.ade.requested.as_deref());
        if !blockers.is_empty() {
            bail!(
                "Desktop Beta cannot start this plan: {}",
                blockers
                    .iter()
                    .filter_map(|reason| match reason {
                        FlowBlockedReason::PermissionViolation { message } =>
                            Some(message.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
    validate_dispatch_task_plan(&tasks, &cfg, &plan.waves, plan.project_id.as_deref())?;
    if let Some(review) = &plan.routing {
        validate_routed_local_wave_shape(&plan)?;
        if plan.permission_profile != PermissionProfile::Orbit.as_str()
            || !(plan.max_workers == 1 || routed_parallel_siblings(&plan))
            || review.authorization.domain_id.as_str() != plan.domain_id
            || review.authorization.plan_digest != routed_plan_digest(&plan)?
            || review.authorization.permission_profile.as_str() != plan.permission_profile
            || review.authorization.limits.max_workers != u32::try_from(plan.max_workers)?
            || review.authorization.live_advice_authorized
        {
            bail!("routed Flow review no longer matches the dispatch scope");
        }
    }
    // Recheck after preview, before claiming the draft or reserving a run. The
    // executor still checks again, including edits racing this read-only check.
    if experimental_routed {
        observe_experimental_routed_git_base(&repo)?;
    } else {
        validate_flow_checkout(&repo, &tasks, &cfg)?;
    }
    let ade = if experimental_routed {
        None
    } else {
        let ade_id = plan
            .ade
            .requested
            .as_deref()
            .context("Flow dispatch requires a selected ADE; generate a new preview")?;
        let ade = resolve_ade(ade_id).context("selected Flow ADE is not registered")?;
        if !ade_can_dispatch(ade) {
            bail!("selected Flow ADE is detection-only until its permission model is mapped: {ade_id}");
        }
        if !ade_on_path(ade) {
            bail!("selected Flow ADE is unavailable: {ade_id}");
        }
        Some(ade)
    };
    ensure_repo_trusted(&repo)?;
    #[cfg(windows)]
    let routed_store_guard = if experimental_routed {
        crate::routed_claude::reject_reparse_components(&cfg.db_path_at(&repo))
            .context("routed Store path must not contain a reparse point")?;
        // Keep the physical Store pinned from stage validation through startup
        // settlement. Otherwise a different database can receive the run
        // after its predecessor's identity was stored in the Catalog claim.
        Some(pytxo_runner::FileIdentityGuard::acquire(
            &cfg.db_path_at(&repo),
        )?)
    } else {
        None
    };
    let routed_store_db_path = if experimental_routed {
        let db_path = cfg.db_path_at(&repo);
        #[cfg(windows)]
        let original_file_identity = routed_store_guard
            .as_ref()
            .expect("routed Store guard acquired")
            .identity()
            .to_owned();
        #[cfg(not(windows))]
        let original_file_identity = pytxo_runner::file_identity(&db_path)?;
        let store = PytxoStore::open(&db_path)?;
        let mission = load_reviewed_staged_mission(&store, &plan)?;
        if hosted_experiment {
            if mission.policy.mode != RoutingMode::Shadow
                || mission.policy.advisor_recipient.as_deref() != Some(HOSTED_RECIPIENT)
                || mission.tasks.len() != 1
                || mission.authorization.limits.max_attempts != 1
            {
                bail!("hosted Shadow experiment differs from the reviewed one-task policy");
            }
            let packet = preview_reviewed_hosted_advisor_packet(catalog, draft_id)?;
            let consent =
                read_experimental_hosted_advisor_local_consent(catalog, &packet.domain_id)?;
            if !consent.enabled || !consent.current_scope {
                bail!("hosted Shadow experiment has no current local consent");
            }
            let grant = catalog
                .hosted_grant(&packet.domain_id)?
                .context("hosted Shadow experiment has no Link grant")?;
            let client = hosted_client
                .as_ref()
                .context("hosted client disappeared")?;
            let reviewed = HostedAdvisorConsentReview {
                domain_id: packet.domain_id.clone(),
                recipient_identity: packet.recipient_identity.clone(),
                draft_id: draft_id.into(),
                consent_revision: packet.reviewed_consent_revision,
                scope_digest: packet.scope_digest.0.clone(),
                packet_digest: packet.packet_digest.0.clone(),
                request_digest: packet.request_digest.0.clone(),
                store_db_file_identity: packet.store_db_file_identity.clone(),
            };
            if client.recipient_identity() != HOSTED_RECIPIENT
                || client.account_id() != grant.intent.account_id
                || client.link_origin() != grant.intent.link_origin
                || client.workspace_id() != grant.intent.workspace_id
                || Some(client.grant_revision()) != grant.remote_revision
            {
                bail!("hosted Shadow client differs from the confirmed account grant");
            }
            catalog.confirmed_hosted_grant_for_send(&grant.intent, &reviewed)?;
        } else if mission.policy.advisor_recipient.is_some() {
            bail!("hosted advisor controller is not available for routed dispatch");
        }
        if crate::routed_fixture::claude_proposal_opted_in()
            && mission
                .profiles
                .iter()
                .any(|registered| registered.profile.harness_id == "claude")
        {
            #[cfg(windows)]
            crate::routed_fixture::validate_claude_probe_location()?;
            #[cfg(not(windows))]
            bail!("Claude proposal route needs Windows owned Job");
        }
        let canonical_path = std::fs::canonicalize(&db_path)?;
        if pytxo_runner::file_identity(&canonical_path)? != original_file_identity {
            bail!("routed Flow Store file changed while checking its private stage");
        }
        let canonical_data_dir = std::fs::canonicalize(repo.join(&cfg.data_dir))?;
        verify_routed_store_gate_directory(&canonical_path, &canonical_data_dir)?;
        Some((canonical_path, original_file_identity))
    } else {
        None
    };
    let prompts: HashMap<String, String> = plan
        .tasks
        .iter()
        .map(|task| (task.id.clone(), task.prompt.clone()))
        .collect();
    let routed_owner = if plan.routing.is_some() {
        let controller_pid = std::process::id();
        let controller_start_identity = pytxo_runner::process_start_identity(controller_pid)?
            .context("routed Flow controller identity is unavailable")?;
        let (store_db_path, store_db_file_identity) = routed_store_db_path
            .as_ref()
            .context("routed Flow Store locator is unavailable")?;
        let store_db_path = store_db_path
            .to_str()
            .context("routed Flow Store locator is not UTF-8")?;
        Some(pytxo_store::RoutedFlowDispatchOwner {
            controller_pid,
            controller_start_identity,
            store_db_path: store_db_path.to_owned(),
            store_db_file_identity: Some(store_db_file_identity.clone()),
        })
    } else {
        None
    };
    let claimed = if let Some(review) = &plan.routing {
        if hosted_experiment {
            catalog.claim_review_only_hosted_shadow_dispatch(
                draft_id,
                expected_plan_json,
                &review.authorization.run_id.0,
                routed_owner.as_ref().expect("routed owner captured"),
            )?
        } else {
            catalog.claim_routed_flow_dispatch(
                draft_id,
                expected_plan_json,
                &review.authorization.run_id.0,
                routed_owner.as_ref().expect("routed owner captured"),
            )?
        }
    } else {
        catalog.claim_flow_dispatch(draft_id, expected_plan_json)?
    };
    if !claimed {
        bail!("Flow draft is already dispatching or dispatched");
    }
    if let Some(review) = &plan.routing {
        if !catalog.mark_routed_flow_startup_may_have_started(
            draft_id,
            &review.authorization.run_id.0,
            routed_owner.as_ref().expect("routed owner captured"),
        )? {
            bail!("routed Flow startup claim changed before reservation");
        }
        let dispatch = crate::default_hypervisor().dispatch_routed_with_config_snapshot(
            catalog,
            draft_id,
            expected_plan_json,
            &plan,
            &repo,
            cfg,
            hosted_client,
        );
        match dispatch {
            Ok((_, run_id)) => {
                #[cfg(feature = "routed-test-faults")]
                crate::hypervisor::pause_routed_fault_stage(
                    "PYTXO_TEST_ROUTED_PAUSE_AFTER_TERMINAL_BEFORE_ACK",
                )?;
                #[cfg(feature = "routed-test-faults")]
                let terminal_ack = if std::env::var_os("PYTXO_TEST_ROUTED_FAIL_FLOW_ACK").is_some()
                {
                    Err(pytxo_core::PytxoError::Store(
                        "injected routed terminal Catalog acknowledgement failure".into(),
                    ))
                } else {
                    catalog.mark_flow_dispatched(draft_id, &run_id.0)
                };
                #[cfg(not(feature = "routed-test-faults"))]
                let terminal_ack = catalog.mark_flow_dispatched(draft_id, &run_id.0);
                if let Err(error) = terminal_ack {
                    if reconcile_routed_flow_startup(catalog, draft_id)? {
                        let settled = catalog
                            .get_flow_draft(draft_id)?
                            .context("reconciled routed Flow draft disappeared")?;
                        if settled.dispatched_run_id.as_deref() != Some(run_id.0.as_str()) {
                            bail!("reconciled routed Flow no longer matches the reviewed RunId");
                        }
                        if settled.status == FlowStatus::Cancelled.as_str() {
                            bail!("reviewed routed run {} was cancelled by Stop", run_id.0);
                        }
                        if settled.status == FlowStatus::Dispatched.as_str() {
                            return Ok(run_id.0);
                        }
                        bail!("reconciled routed Flow has no terminal acknowledgement");
                    }
                    if let Err(recovery_error) =
                        catalog.mark_routed_flow_recovery_required(draft_id, &run_id.0)
                    {
                        tracing::warn!(draft_id, %recovery_error, "routed Flow terminal claim recovery deferred");
                    }
                    return Err(error.into());
                }
                return Ok(run_id.0);
            }
            Err(error) => {
                let run_id = &plan
                    .routing
                    .as_ref()
                    .expect("routed dispatch review checked")
                    .authorization
                    .run_id
                    .0;
                if error.is::<crate::routed_fixture::RoutedPreflightStopped>() {
                    catalog.mark_routed_flow_preflight_stopped(draft_id, run_id)?;
                } else if error.is::<crate::hypervisor::RoutedStartupStopped>() {
                    catalog.mark_routed_flow_stopped_after_quiescence(draft_id, run_id)?;
                } else if error.is::<crate::hypervisor::RoutedStartupSettled>() {
                    catalog.mark_routed_flow_startup_failed(draft_id, run_id)?;
                } else {
                    catalog.mark_routed_flow_recovery_required(draft_id, run_id)?;
                    if let Err(recovery_error) = reconcile_routed_flow_startup(catalog, draft_id) {
                        tracing::warn!(draft_id, %recovery_error, "routed Flow startup recovery deferred");
                    }
                }
                return Err(error);
            }
        }
    }
    let ade = ade.expect("legacy dispatch checked an ADE before claim");
    let dispatch = dispatch_run_with_config_snapshot(
        RunOptions {
            // Keep the reviewed concurrency limit. Task count is mission scope,
            // not permission to widen the execution waves at dispatch.
            agents: cfg.max_agents,
            cmd: ade.default_cmd.into(),
            config: None,
            dry_run: false,
            keep_worktrees: true,
            repo: Some(repo),
            execution: None,
            project: None,
            tasks: Some(tasks),
            task_cmd_template: Some(ade_prompt_command(ade.default_cmd)),
            task_prompts: Some(prompts),
        },
        cfg,
    );
    let (_, run_id) = match dispatch {
        Ok(result) => result,
        Err(error) => {
            catalog.mark_flow_dispatch_failed(draft_id)?;
            return Err(error);
        }
    };
    catalog.mark_flow_dispatched(draft_id, &run_id)?;
    Ok(run_id)
}

fn verify_routed_store_gate_directory(
    store_path: &Path,
    launch_data_dir: &Path,
) -> anyhow::Result<()> {
    if store_path.parent() != Some(launch_data_dir) {
        bail!("routed Store and native launch gate resolve to different directories");
    }
    Ok(())
}

fn summarize_ade(requested: Option<&str>) -> FlowAdeSummary {
    let available_specs: Vec<_> = all_ade_clis()
        .iter()
        .filter(|spec| ade_can_dispatch(spec) && ade_on_path(spec))
        .collect();
    let installed = available_specs
        .iter()
        .map(|spec| spec.id.to_string())
        .collect();
    let selected = requested.and_then(resolve_ade).or_else(|| {
        requested
            .is_none()
            .then(|| available_specs.first().copied())
            .flatten()
    });
    FlowAdeSummary {
        requested: selected
            .map(|spec| spec.id.to_string())
            .or_else(|| requested.map(str::to_string)),
        available: selected.is_some_and(|spec| ade_can_dispatch(spec) && ade_on_path(spec)),
        installed,
        command: selected.map(|spec| spec.default_cmd.to_string()),
    }
}

/// Build a static ADE adapter command. Mission text is supplied through the child
/// environment. Windows Codex uses its stdin contract; other Windows adapters retain
/// their existing native-argument behavior, whose quoting depends on the installed CLI.
fn ade_prompt_command(default_cmd: &str) -> String {
    if cfg!(windows) {
        let invocation = default_cmd
            .split_whitespace()
            .map(|part| format!("'{}'", part.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(" ");
        let script = if let Some(arguments) = default_cmd.strip_prefix("codex exec") {
            // Codex documents `exec -` as full-prompt stdin. Passing prompt text
            // through PowerShell -> npm's .cmd shim reparses quotes/newlines.
            let arguments = format!("/d /s /c \"\"%PYTXO_ADE_EXECUTABLE%\" exec{arguments} -\"")
                .replace('\'', "''");
            format!(
                r#"$ErrorActionPreference='Stop'
$ade=(Get-Command 'codex' -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
$start=New-Object System.Diagnostics.ProcessStartInfo
$start.FileName=$env:ComSpec
$start.Arguments='{arguments}'
$start.WorkingDirectory=(Get-Location).ProviderPath
$start.UseShellExecute=$false
$start.RedirectStandardInput=$true
$start.EnvironmentVariables['PYTXO_ADE_EXECUTABLE']=$ade
[Console]::InputEncoding=[System.Text.UTF8Encoding]::new($false)
$child=[System.Diagnostics.Process]::Start($start)
try {{
  $bytes=[System.Text.Encoding]::UTF8.GetBytes([string]$env:PYTXO_TASK_PROMPT)
  $child.StandardInput.BaseStream.Write($bytes,0,$bytes.Length)
  $child.StandardInput.Close()
  $child.WaitForExit()
  exit $child.ExitCode
}} finally {{ $child.Dispose() }}"#
            )
        } else {
            format!("& {invocation} $env:PYTXO_TASK_PROMPT")
        };
        let utf16le = script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, utf16le);
        format!("powershell -NoProfile -NonInteractive -EncodedCommand {encoded}")
    } else {
        format!("{default_cmd} \"$PYTXO_TASK_PROMPT\"")
    }
}

fn validate_task_claims(tasks: &[Task], _project_id: Option<&str>) -> Vec<FlowBlockedReason> {
    let mut blocked = Vec::new();
    for task in tasks {
        if task.root.is_some() {
            blocked.push(FlowBlockedReason::InvalidRoot {
                message: format!(
                    "task {} selects a labeled project root; Flow previews are limited to one execution domain",
                    task.id.0
                ),
            });
        }
        for claim in &task.paths {
            let path = Path::new(claim);
            if path.is_absolute()
                || path.components().any(|part| {
                    matches!(
                        part,
                        Component::ParentDir | Component::Prefix(_) | Component::RootDir
                    )
                })
            {
                blocked.push(FlowBlockedReason::InvalidPath {
                    task_id: task.id.0.clone(),
                    path: claim.clone(),
                });
            }
        }
    }
    blocked
}

fn validate_flow_checkout(repo: &Path, tasks: &[Task], cfg: &PytxoConfig) -> anyhow::Result<()> {
    crate::assert_git_ready(repo)?;
    // Match the executor's reviewed repository Apply boundary. Read-only and
    // direct-write runs do not acquire a new clean-checkout requirement here.
    let profiles: Vec<_> = tasks
        .iter()
        .map(|task| cfg.resolve_profile_for_agent(&task.agent))
        .collect();
    if !tasks.iter().any(|task| task.root.is_some())
        && !profiles.contains(&PermissionProfile::DeepSpace)
        && !profiles.contains(&PermissionProfile::Supernova)
    {
        crate::assert_clean_primary_checkout(repo)?;
    }
    Ok(())
}

fn desktop_beta_blockers(
    cfg: &PytxoConfig,
    tasks: &[Task],
    ade_id: Option<&str>,
) -> Vec<FlowBlockedReason> {
    let mut messages = Vec::new();
    if ade_id != Some("codex") {
        messages.push("Desktop Beta runs Codex. Select Codex and build a new plan.");
    }
    if cfg.max_agents != 1 {
        messages.push("Desktop Beta runs one worker. Build a new plan with one worker.");
    }
    if cfg.execution_backend != pytxo_core::ExecutionBackend::Pty {
        messages.push("Desktop Beta requires local PTY execution. Set execution_backend = \"pty\" in this repository's pytxo.toml and build a new plan.");
    }
    if cfg.permission_profile != PermissionProfile::Orbit
        || tasks
            .iter()
            .any(|task| cfg.resolve_profile_for_agent(&task.agent) != PermissionProfile::Orbit)
    {
        messages.push("Desktop Beta requires Orbit for the repository and every worker. Review the repository's permission settings and agent overrides, then build a new plan. Existing settings have not been changed.");
    }
    messages
        .into_iter()
        .map(|message| FlowBlockedReason::PermissionViolation {
            message: message.into(),
        })
        .collect()
}

fn validate_permission_scope(tasks: &[Task], cfg: &PytxoConfig) -> Vec<FlowBlockedReason> {
    tasks
        .iter()
        .filter_map(|task| {
            let task_profile = cfg.resolve_profile_for_agent(&task.agent);
            (profile_rank(task_profile) > profile_rank(cfg.permission_profile)).then(|| {
                FlowBlockedReason::PermissionViolation {
                    message: format!(
                        "task {} requests {} above execution-domain profile {}",
                        task.id.0,
                        task_profile.as_str(),
                        cfg.permission_profile.as_str()
                    ),
                }
            })
        })
        .collect()
}

fn validate_dispatch_task_plan(
    tasks: &[Task],
    cfg: &PytxoConfig,
    expected_waves: &[Vec<String>],
    project_id: Option<&str>,
) -> anyhow::Result<()> {
    let mut blockers = validate_task_claims(tasks, project_id);
    blockers.extend(validate_permission_scope(tasks, cfg));
    let execution = plan_tasks(tasks, cfg)?;
    let runtime_waves: Vec<Vec<String>> = execution
        .waves
        .iter()
        .map(|wave| wave.iter().map(|task| task.task_id.0.clone()).collect())
        .collect();
    if runtime_waves != expected_waves {
        bail!("Flow scheduler result changed; generate a new preview");
    }
    let wave_of: HashMap<&str, usize> = runtime_waves
        .iter()
        .enumerate()
        .flat_map(|(wave, tasks)| tasks.iter().map(move |task| (task.as_str(), wave)))
        .collect();
    let concurrent_overlap = execution.conflicts.iter().any(|conflict| {
        wave_of.get(conflict.task_a.0.as_str()) == wave_of.get(conflict.task_b.0.as_str())
    });
    if !blockers.is_empty() || concurrent_overlap {
        bail!("Flow path claims no longer pass validation; generate a new preview");
    }
    Ok(())
}

fn apply_flow_permission_ceiling(
    cfg: &PytxoConfig,
    ceiling: Option<PermissionProfile>,
) -> (PytxoConfig, Option<FlowBlockedReason>) {
    let mut effective = cfg.clone();
    let Some(ceiling) = ceiling else {
        return (effective, None);
    };
    let configured = cfg.permission_profile;
    effective.permission_profile =
        crate::entitlements::apply_permission_ceiling(configured, ceiling);
    let blocker = (effective.permission_profile != configured).then(|| {
        FlowBlockedReason::PermissionViolation {
            message: format!(
                "execution-domain profile {} exceeds organization ceiling {}",
                configured.as_str(),
                ceiling.as_str()
            ),
        }
    });
    (effective, blocker)
}

fn profile_rank(profile: PermissionProfile) -> u8 {
    match profile {
        PermissionProfile::DeepSpace => 0,
        PermissionProfile::Orbit => 1,
        PermissionProfile::Galaxy => 2,
        PermissionProfile::Supernova => 3,
    }
}

pub(crate) fn runtime_tasks(plan: &FlowPlan) -> Vec<Task> {
    plan.tasks
        .iter()
        .map(|task| Task {
            id: TaskId(task.id.clone()),
            agent: task.agent.clone(),
            paths: task.paths.clone(),
            depends_on: task.dependencies.clone(),
            root: task.root.clone(),
            signal_fidelity: None,
            verify: task.verify.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routed_store_locator_must_share_native_launch_gate_directory() {
        let data_dir = Path::new("/pytxo/data");
        assert!(
            verify_routed_store_gate_directory(Path::new("/pytxo/data/pytxo.db"), data_dir,)
                .is_ok()
        );
        assert!(verify_routed_store_gate_directory(
            Path::new("/other/location/pytxo.db"),
            data_dir,
        )
        .is_err());
    }

    #[test]
    fn prepared_no_create_usage_is_zero_only_for_supported_owned_sources() {
        let scope = RoutingScope {
            domain_id: DomainId("domain".into()),
            run_id: RunId("run".into()),
        };
        let attempt = pytxo_core::routing::AttemptId("attempt".into());
        let executable = ExecutableIdentity {
            path: "claude.exe".into(),
            version: "pin".into(),
            digest: Digest::of_bytes(b"cli"),
        };
        let fixture = prepared_no_create_usage(
            "pytxo-local-fixture-v1",
            BillingSourceMode::Local,
            &scope,
            &attempt,
            &executable,
        )
        .unwrap();
        let subscription = prepared_no_create_usage(
            "claude",
            BillingSourceMode::Subscription,
            &scope,
            &attempt,
            &executable,
        )
        .unwrap();
        assert!(matches!(fixture, RoutedUsage::Known { nano_usd: 0, .. }));
        assert!(matches!(
            subscription,
            RoutedUsage::Known { nano_usd: 0, .. }
        ));
        assert_ne!(fixture, subscription);
        assert!(prepared_no_create_usage(
            "claude",
            BillingSourceMode::Api,
            &scope,
            &attempt,
            &executable,
        )
        .is_err());
    }

    #[test]
    #[cfg(windows)]
    fn drive_relative_and_root_relative_claims_are_not_local_paths() {
        let claims = ["C:src\\foo.rs", "\\pytxo\\src\\foo.rs"];
        for claim in claims {
            let task = Task {
                id: TaskId("worker".into()),
                agent: "builder".into(),
                paths: vec![claim.into()],
                depends_on: vec![],
                root: None,
                signal_fidelity: None,
                verify: vec![],
            };
            assert!(
                validate_task_claims(&[task], None)
                    .iter()
                    .any(|reason| matches!(reason, FlowBlockedReason::InvalidPath { .. })),
                "claim should be rejected: {claim}"
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn routed_review_rejects_case_aliased_parallel_claims() {
        let task = |id: &str, path: &str| FlowPlanTask {
            id: id.into(),
            agent: "builder".into(),
            prompt: "fixture".into(),
            paths: vec![path.into()],
            dependencies: vec![],
            root: None,
            verify: vec!["git status --porcelain".into()],
        };
        let mut plan = FlowPlan {
            draft_id: "draft".into(),
            domain_id: "domain".into(),
            project_id: None,
            status: FlowStatus::Ready,
            tasks: vec![task("a", "src/Foo.rs"), task("b", "src/foo.rs")],
            waves: vec![vec!["a".into(), "b".into()]],
            max_workers: 2,
            permission_profile: "orbit".into(),
            isolation_mode: "sandbox".into(),
            isolation_backend_intent: "copy_on_write".into(),
            execution_backend: "pty".into(),
            ade: FlowAdeSummary {
                requested: Some("codex".into()),
                available: true,
                installed: vec!["codex".into()],
                command: None,
            },
            warnings: vec![],
            blocked_reasons: vec![],
            estimated_tokens: None,
            estimated_cost_usd: None,
            previewed_at: String::new(),
            routing: None,
        };
        assert!(!routed_parallel_siblings(&plan));
        assert!(validate_routed_local_wave_shape(&plan).is_err());
        plan.tasks[0].paths[0] = "src/Dir./file.rs".into();
        plan.tasks[1].paths[0] = "src/dir/file.rs".into();
        assert!(!routed_parallel_siblings(&plan));
        assert!(validate_routed_local_wave_shape(&plan).is_err());
    }

    fn reviewed_claude_shape() -> (FlowPlan, RoutingMission) {
        use pytxo_core::routing::{BillingSourceId, ModelIdentityLevel};

        let mut mission: RoutingMission = serde_json::from_str(include_str!(
            "../../pytxo-store/tests/fixtures/routing_pre_check_recipes_v8_registration.json"
        ))
        .unwrap();
        let task = &mut mission.tasks[0].contract;
        task.goal = "Update result.txt with the reviewed change".into();
        task.claim_roots = vec!["result.txt".into()];
        task.dependencies.clear();
        task.required_capabilities = BTreeSet::from(["edit".into()]);
        task.required_egress =
            BTreeSet::from([crate::routed_claude::CLAUDE_SUBSCRIPTION_ENDPOINT.into()]);
        task.required_resources = BTreeSet::from(["account-claude".into()]);
        task.skill_tool_bundle_digest =
            Digest::of_bytes(crate::routed_claude::CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes());
        task.required_target = None;
        task.strong_only = false;
        task.cross_component_requirement = Some(false);
        task.context_complete = true;
        for (registered, model) in mission.profiles.iter_mut().zip(["haiku", "sonnet"]) {
            let profile = &mut registered.profile;
            profile.harness_id = "claude".into();
            profile.adapter_contract_version = "1".into();
            profile.adapter_digest =
                Digest::of_bytes(crate::routed_claude::CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes());
            profile.skill_tool_bundle_digest = task.skill_tool_bundle_digest.clone();
            profile.backend = ExecutionBackend::Subprocess;
            profile.capabilities = BTreeSet::from(["read".into(), "edit".into()]);
            profile.requested_model.provider = "anthropic".into();
            profile.requested_model.model = model.into();
            profile.requested_model.reasoning = None;
            profile.requested_model.revision = None;
            let binding = &mut registered.binding;
            binding.profile_digest = profile.digest().unwrap();
            binding.credential_reference = None;
            binding.auth_owner = "Claude".into();
            binding.billing_source_id = BillingSourceId("personal-subscription".into());
            binding.billing_mode = BillingSourceMode::Subscription;
            binding.endpoint_identity = crate::routed_claude::CLAUDE_SUBSCRIPTION_ENDPOINT.into();
            binding.trust_class = "vendor".into();
            binding.capacity_pool_ids = task.required_resources.clone();
        }
        mission.authorization.permission_profile = PermissionProfile::Orbit;
        mission.authorization.allowed_billing_sources =
            BTreeSet::from([BillingSourceId("personal-subscription".into())]);
        mission.authorization.allowed_billing_modes =
            BTreeSet::from([BillingSourceMode::Subscription]);
        mission.authorization.allowed_egress = task.required_egress.clone();
        mission.authorization.minimum_model_identity = ModelIdentityLevel::Requested;
        mission.authorization.limits.max_workers = 1;
        mission.authorization.limits.max_attempts = 1;
        mission.authorization.live_advice_authorized = false;
        mission.policy.mode = RoutingMode::Rules;
        mission.policy.version = "claude-proposal-rules-v1".into();
        mission.policy.advisor_recipient = None;
        mission.authorization.policy_digest = mission.policy.digest().unwrap();
        let planned = FlowPlanTask {
            id: task.task_id.0.clone(),
            agent: "default".into(),
            prompt: task.goal.clone(),
            paths: task.claim_roots.clone(),
            dependencies: vec![],
            root: None,
            verify: vec!["if exist result.txt (exit /b 0) else (exit /b 1)".into()],
        };
        let plan = FlowPlan {
            draft_id: "claude-review".into(),
            domain_id: "domain".into(),
            project_id: None,
            status: FlowStatus::Ready,
            waves: vec![vec![planned.id.clone()]],
            tasks: vec![planned],
            max_workers: 1,
            permission_profile: "orbit".into(),
            isolation_mode: "worktree".into(),
            isolation_backend_intent: "worktree".into(),
            execution_backend: "subprocess".into(),
            ade: FlowAdeSummary {
                requested: None,
                available: false,
                installed: vec![],
                command: None,
            },
            warnings: vec![],
            blocked_reasons: vec![],
            estimated_tokens: None,
            estimated_cost_usd: None,
            previewed_at: String::new(),
            routing: None,
        };
        (plan, mission)
    }

    #[test]
    fn one_task_claude_shape_accepts_only_a_shared_subscription_pair() {
        let (plan, mission) = reviewed_claude_shape();
        validate_one_task_claude_route_shape(&plan, &mission).unwrap();

        let mut changed = mission.clone();
        changed.profiles[1].binding.billing_mode = BillingSourceMode::Api;
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.profiles[1].binding.billing_source_id =
            pytxo_core::routing::BillingSourceId("other-account".into());
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.profiles[1].profile.requested_model.model = "opus".into();
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.profiles[0].profile.adapter_digest = Digest::of_bytes(b"old-adapter");
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.profiles[0].profile.adapter_digest =
            Digest::of_bytes(crate::routed_claude::CLAUDE_ADAPTER_ID.as_bytes());
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.tasks[0].contract.claim_roots = vec!["../outside.txt".into()];
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.tasks[0].contract.dependencies = vec![TaskId("prior".into())];
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.authorization.limits.max_attempts = 2;
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.authorization.limits.spend_guarantee = pytxo_core::routing::SpendGuarantee::Hard;
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.tasks[0]
            .contract
            .required_egress
            .insert("external-tool".into());
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.policy.mode = RoutingMode::Live;
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed = mission.clone();
        changed.policy.advisor_recipient = Some("hosted".into());
        assert!(validate_one_task_claude_route_shape(&plan, &changed).is_err());
        let mut changed_plan = plan.clone();
        changed_plan.isolation_backend_intent = "copy_on_write".into();
        assert!(validate_one_task_claude_route_shape(&changed_plan, &mission).is_err());
        changed_plan = plan.clone();
        changed_plan.max_workers = 2;
        assert!(validate_one_task_claude_route_shape(&changed_plan, &mission).is_err());
    }

    #[test]
    fn routed_review_rejects_claude_shape_before_stage_or_launch() {
        let (plan, mut mission) = reviewed_claude_shape();
        mission.profiles[1].binding.billing_mode = BillingSourceMode::Api;
        let error = review_routed_mission(
            &plan,
            &mission.authorization.run_id,
            &mission.authorization.plan_digest,
            &mission,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("Claude route profile or account binding"));

        let (plan, mut mission) = reviewed_claude_shape();
        mission.profiles[0].profile.harness_id = "renamed".into();
        mission.profiles[0].binding.auth_owner = "renamed".into();
        mission.profiles[0].binding.endpoint_identity = "renamed".into();
        let error = review_routed_mission(
            &plan,
            &mission.authorization.run_id,
            &mission.authorization.plan_digest,
            &mission,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("Claude route profile or account binding"));
    }

    #[test]
    fn workspace_scope_binds_the_projection_implementation() {
        let projection_source = include_str!("advisor_projection.rs").replace("\r\n", "\n");
        let source_digest = Digest::of_bytes(projection_source.as_bytes());
        let identity = reviewed_routing_advisor_identity();
        assert!(identity.contains(&format!(":projection:{}:recipient:", source_digest.0)));
        assert_eq!(
            reviewed_routing_advisor_disclosure_scope_digest(),
            Digest::of_bytes(identity.as_bytes())
        );
    }

    #[test]
    fn advisor_packet_withholds_arbitrary_goal_text_even_without_secret_markers() {
        const COARSE: &str = "Classify reviewed repository task using coarse facts";
        for private in [
            "Fix login with password hunter2",
            "Use token=opaque in review",
            "Investigate api key exposure",
            "Check a multiline task\nwith private context",
            "Render `source code` safely",
            "Send Alice Smith salary 90000 to Bob",
            "Update C:\\private\\account.json and README.md",
        ] {
            assert_eq!(redacted_reviewed_task_description(private), COARSE);
        }
    }

    #[test]
    fn advisor_wire_bytes_do_not_change_with_freeform_goal_text() {
        let mut mission: RoutingMission = serde_json::from_str(include_str!(
            "../../pytxo-store/tests/fixtures/routing_pre_check_recipes_v8_registration.json"
        ))
        .unwrap();
        let task = &mut mission.tasks[0].contract;
        let original = reviewed_task_advisor_packet(task)
            .unwrap()
            .request_body()
            .unwrap();
        task.goal = "Send Alice Smith salary 90000 to Bob".into();
        let sensitive = reviewed_task_advisor_packet(task)
            .unwrap()
            .request_body()
            .unwrap();
        assert_eq!(original, sensitive);
        let outbound = String::from_utf8(sensitive).unwrap();
        assert!(!outbound.contains("Alice"));
        assert!(!outbound.contains("salary"));
    }

    #[test]
    fn desktop_beta_checks_each_worker_profile_and_selected_adapter() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pytxo.toml");
        std::fs::write(
            &path,
            "max_agents = 1\n[[agent]]\nname = \"builder\"\npermission_profile = \"deep_space\"\n",
        )
        .unwrap();
        let cfg = PytxoConfig::load(&path).unwrap();
        let tasks = [Task {
            id: TaskId("one".into()),
            agent: "builder".into(),
            paths: vec!["src/lib.rs".into()],
            depends_on: vec![],
            root: None,
            signal_fidelity: None,
            verify: vec!["cargo test".into()],
        }];
        assert_eq!(cfg.permission_profile, PermissionProfile::Orbit);
        assert!(desktop_beta_blockers(&cfg, &tasks, Some("codex")).iter().any(|reason|
            matches!(reason, FlowBlockedReason::PermissionViolation { message } if message.contains("every worker"))));
        for adapter in [None, Some("claude"), Some("gemini")] {
            assert!(desktop_beta_blockers(&cfg, &[], adapter).iter().any(|reason|
                matches!(reason, FlowBlockedReason::PermissionViolation { message } if message.contains("runs Codex"))));
        }
    }

    #[cfg(windows)]
    async fn fail_codex_transport_fixture(
        phase: &str,
        diagnostics: String,
        data_dir: &Path,
        run_id: &str,
        running: &mut tokio::task::JoinHandle<
            pytxo_core::Result<Vec<pytxo_runner::AgentRunResult>>,
        >,
    ) -> ! {
        use std::time::Duration;

        // Report the stalled phase before cleanup can change its evidence.
        eprintln!("{phase}: {diagnostics}");
        let data_dir = data_dir.to_path_buf();
        let run_id = run_id.to_owned();
        let cleanup = tokio::time::timeout(
            Duration::from_secs(15),
            tokio::task::spawn_blocking(move || pytxo_runner::stop_run(&data_dir, &run_id, true)),
        )
        .await;
        let settlement = tokio::time::timeout(Duration::from_secs(10), &mut *running).await;
        if settlement.is_err() {
            running.abort();
        }
        panic!("{phase}: {diagnostics}; owned cleanup: {cleanup:?}; settlement: {settlement:?}");
    }

    #[cfg(windows)]
    async fn codex_transport_fixture(
        backend: pytxo_core::ExecutionBackend,
        exit_code: i32,
        stop: bool,
        legacy: bool,
    ) -> (serde_json::Value, Vec<pytxo_runner::AgentRunResult>) {
        use pytxo_core::{
            BillingMode, ExecutionPlan, FidelityTier, IsolationMode, RunId, ScheduledTask,
        };
        use pytxo_runner::{execute_plan, ProcessRegistry, RunContext, SwarmRegistry};
        use std::process::Command;
        use std::sync::{Arc, Mutex};
        use std::time::{Duration, Instant};

        // Hosted Windows must start Git worktrees, PowerShell, Node and ConPTY.
        // This is a transport contract test, not a startup latency benchmark.
        const LIFECYCLE_DEADLINE: Duration = Duration::from_secs(60);
        // The Stop probe exits naturally after 30s: settlement must beat that.
        const STOP_SETTLEMENT_DEADLINE: Duration = Duration::from_secs(10);

        let fixture = tempfile::tempdir().unwrap();
        let repo = fixture.path().join("repo");
        let tools = fixture.path().join("fake tools");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::create_dir_all(&tools).unwrap();
        std::fs::write(
            tools.join("codex.cmd"),
            "@echo off\r\n@node \"%~dp0probe.cjs\" %*\r\n",
        )
        .unwrap();
        std::fs::write(
            tools.join("opencode.cmd"),
            "@echo off\r\n@node \"%~dp0probe.cjs\" %*\r\n",
        )
        .unwrap();
        std::fs::write(
            tools.join("probe.cjs"),
            r#"
const fs = require('fs');
const args = process.argv.slice(2);
function record(bytes) {
  fs.writeFileSync('observed.json', JSON.stringify({args, received: bytes.toString('utf8'),
    expected: process.env.PYTXO_TASK_PROMPT, bytes: Array.from(bytes), pid: process.pid}));
  if (process.env.PROBE_STOP === '1') setTimeout(() => process.exit(0), 30000);
  else process.exit(Number(process.env.PROBE_EXIT));
}
if (args.at(-1) === '-') {
  const chunks = [];
  process.stdin.on('data', chunk => chunks.push(chunk));
  process.stdin.on('end', () => record(Buffer.concat(chunks)));
} else record(Buffer.from(args.at(-1) || '', 'utf8'));
"#,
        )
        .unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "pytxo@test.local"],
            vec!["config", "user.name", "Pytxo Test"],
        ] {
            assert!(Command::new("git")
                .args(args)
                .current_dir(&repo)
                .output()
                .unwrap()
                .status
                .success());
        }
        std::fs::write(repo.join("README.md"), "fixture\n").unwrap();
        for args in [vec!["add", "."], vec!["commit", "-m", "fixture"]] {
            assert!(Command::new("git")
                .args(args)
                .current_dir(&repo)
                .output()
                .unwrap()
                .status
                .success());
        }
        let default_cmd = if legacy {
            "opencode run"
        } else {
            "codex exec --sandbox workspace-write"
        };
        let command = ade_prompt_command(default_cmd);
        let encoded = command.split_whitespace().last().unwrap();
        let bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded).unwrap();
        let units = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes(*pair))
            .collect::<Vec<_>>();
        let script = String::from_utf16(&units).unwrap();
        // Restrict executable discovery to the local shim, without changing parent PATH.
        let script = format!("$env:PATH='{};' + $env:PATH; $env:PROBE_EXIT='{exit_code}'; $env:PROBE_STOP='{}'; {script}",
            tools.to_string_lossy().replace('\'', "''"), if stop { "1" } else { "0" });
        let bytes = script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        let command = format!(
            "powershell -NoProfile -NonInteractive -EncodedCommand {}",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
        );
        let original = if legacy {
            "Update the owned file."
        } else {
            "Update \"quoted risk\".\n\nKeep CRLF:\r\nUnicode: 日本語. Literal %PATH% !NAME! & | < > ^ ` $() and trailing slash \\"
        };
        let run_id = RunId::new();
        let data_dir = repo.join(".pytxo/data");
        let expected_workspace = repo
            .join(".pytxo/worktrees")
            .join(&run_id.0)
            .join("agent-0");
        let (domain_id, model_router, managed_transport, token_estimator) =
            RunContext::default_metering(&repo);
        let started = Instant::now();
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured_events = Arc::clone(&events);
        let ctx = RunContext {
            run_id: run_id.clone(),
            repo_root: repo.clone(),
            worktree_base: repo.join(".pytxo/worktrees"),
            data_dir: data_dir.clone(),
            cmd: default_cmd.into(),
            task_cmd_template: Some(command),
            task_prompts: HashMap::from([("task".into(), original.into())]),
            keep_worktrees: true,
            on_event: Some(Arc::new(move |_, kind, payload| {
                captured_events.lock().unwrap().push((
                    started.elapsed(),
                    kind.to_owned(),
                    payload.to_owned(),
                ));
            })),
            signal_core: false,
            signal_fidelity: FidelityTier::Low,
            isolation_mode: IsolationMode::Worktree,
            permission_profile: PermissionProfile::Orbit,
            agent_profiles: HashMap::new(),
            route_agents: vec![],
            billing_mode: BillingMode::Byok,
            domain_id,
            model_router,
            managed_transport,
            usage_meter: None,
            token_estimator,
            execution_backend: backend,
            pty_rows: 24,
            pty_cols: 120,
            hitl: None,
            hitl_manual_flush: false,
            agent_paths: HashMap::new(),
            agent_fidelity: HashMap::new(),
            roots: HashMap::new(),
            readonly_context_roots: vec![],
            subprocess_stdin: false,
            cloud_dispatcher: None,
            context_cache: None,
            cloud_cache_enabled: false,
            cloud_fallback_local: false,
            mcp_hub: None,
            mcp_hub_enabled: false,
            mcp_allowlist: vec![],
            sparse_exclude: vec![],
        };
        let plan = ExecutionPlan {
            waves: vec![vec![ScheduledTask {
                task_id: TaskId("task".into()),
                agent: "fixture".into(),
                paths: vec!["observed.json".into()],
                depends_on: vec![],
                wave: 0,
                root: None,
                signal_fidelity: None,
                verify: vec!["echo verification-ok".into()],
            }]],
            conflicts: vec![],
            max_agents: 1,
            warnings: vec![],
        };
        let diagnostics = || {
            format!(
                "backend={backend:?}, legacy={legacy}, exit_code={exit_code}, stop={stop}, elapsed={:?}, observed={:?}, events={:?}",
                started.elapsed(),
                std::fs::read_to_string(expected_workspace.join("observed.json")),
                events.lock().unwrap(),
            )
        };
        let mut running = tokio::spawn(async move {
            execute_plan(
                &ctx,
                &plan,
                &ProcessRegistry::default(),
                &SwarmRegistry::new(),
            )
            .await
        });
        let mut child_identity = None;
        if stop {
            let record = tokio::time::timeout(LIFECYCLE_DEADLINE, async {
                loop {
                    if let Ok(bytes) = std::fs::read(expected_workspace.join("observed.json")) {
                        if let Ok(record) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                            break record;
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await;
            let record = match record {
                Ok(record) => record,
                Err(_) => {
                    fail_codex_transport_fixture(
                        "local shim readiness deadline exceeded",
                        diagnostics(),
                        &data_dir,
                        &run_id.0,
                        &mut running,
                    )
                    .await
                }
            };
            let pid = record["pid"].as_u64().unwrap() as u32;
            child_identity = Some((
                pid,
                pytxo_runner::process_start_identity(pid).unwrap().unwrap(),
            ));
            let stop_data_dir = data_dir.clone();
            let stop_run_id = run_id.0.clone();
            let stop_result = tokio::time::timeout(
                STOP_SETTLEMENT_DEADLINE,
                tokio::task::spawn_blocking(move || {
                    pytxo_runner::stop_run(&stop_data_dir, &stop_run_id, true)
                }),
            )
            .await;
            match stop_result {
                Ok(Ok(Ok(_))) => {}
                other => {
                    fail_codex_transport_fixture(
                        "local adapter Stop call deadline exceeded or failed",
                        format!("{}; stop result: {other:?}", diagnostics()),
                        &data_dir,
                        &run_id.0,
                        &mut running,
                    )
                    .await
                }
            }
        }
        let deadline = if stop {
            STOP_SETTLEMENT_DEADLINE
        } else {
            LIFECYCLE_DEADLINE
        };
        let results = match tokio::time::timeout(deadline, &mut running).await {
            Ok(results) => results.unwrap().unwrap(),
            Err(_) => {
                fail_codex_transport_fixture(
                    if stop {
                        "local adapter post-Stop settlement deadline exceeded"
                    } else {
                        "local adapter lifecycle deadline exceeded"
                    },
                    diagnostics(),
                    &data_dir,
                    &run_id.0,
                    &mut running,
                )
                .await
            }
        };
        if let Some((pid, identity)) = child_identity {
            assert!(
                !pytxo_runner::process_matches(pid, &identity).unwrap(),
                "owned Node child survived Stop"
            );
        }
        let observed = serde_json::from_slice(
            &std::fs::read(expected_workspace.join("observed.json"))
                .unwrap_or_else(|error| panic!("missing shim output: {error}; {results:?}")),
        )
        .unwrap();
        eprintln!(
            "transport fixture completed: backend={backend:?}, legacy={legacy}, exit_code={exit_code}, stop={stop}, elapsed={:?}",
            started.elapsed(),
        );
        (observed, results)
    }

    #[cfg(windows)]
    async fn assert_codex_prompt_transport(backend: pytxo_core::ExecutionBackend) {
        let (observed, results) = codex_transport_fixture(backend, 0, false, false).await;
        assert!(results[0].outcome.is_success(), "{results:?}");
        assert_eq!(
            observed["received"], observed["expected"],
            "actual shim argv/stdin lost prompt bytes: {observed}"
        );
        assert_eq!(
            observed["args"],
            serde_json::json!(["exec", "--sandbox", "workspace-write", "-"])
        );
        let expected = observed["expected"].as_str().unwrap().as_bytes();
        assert_eq!(observed["bytes"], serde_json::json!(expected));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn codex_prompt_transport_preserves_pty_bytes() {
        assert_codex_prompt_transport(pytxo_core::ExecutionBackend::Pty).await;
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn codex_prompt_transport_preserves_subprocess_bytes() {
        assert_codex_prompt_transport(pytxo_core::ExecutionBackend::Subprocess).await;
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn codex_prompt_transport_preserves_native_failure() {
        let (_, results) =
            codex_transport_fixture(pytxo_core::ExecutionBackend::Pty, 23, false, false).await;
        assert_eq!(results[0].exit_code, Some(23));
        assert!(!results[0].outcome.is_success());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn codex_prompt_transport_stop_reaps_owned_child() {
        for backend in [
            pytxo_core::ExecutionBackend::Pty,
            pytxo_core::ExecutionBackend::Subprocess,
        ] {
            let (_, results) = codex_transport_fixture(backend, 0, true, false).await;
            assert!(!results[0].outcome.is_success(), "{results:?}");
        }
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn codex_prompt_transport_legacy_adapter_receives_generated_guidance() {
        for backend in [
            pytxo_core::ExecutionBackend::Pty,
            pytxo_core::ExecutionBackend::Subprocess,
        ] {
            let (observed, results) = codex_transport_fixture(backend, 0, false, true).await;
            assert!(results[0].outcome.is_success(), "{results:?}");
            assert_eq!(
                observed["received"], observed["expected"],
                "generated guidance broke existing adapter: {observed}"
            );
            assert_eq!(observed["args"].as_array().unwrap().len(), 2);
            assert_eq!(observed["args"][0], "run");
        }
    }

    #[test]
    fn organization_ceiling_blocks_flow_escalation() {
        let mut cfg = PytxoConfig {
            permission_profile: PermissionProfile::Supernova,
            ..PytxoConfig::default()
        };
        let (effective, blocker) =
            apply_flow_permission_ceiling(&cfg, Some(PermissionProfile::Orbit));
        assert_eq!(effective.permission_profile, PermissionProfile::Orbit);
        assert!(matches!(
            blocker,
            Some(FlowBlockedReason::PermissionViolation { .. })
        ));

        cfg.permission_profile = PermissionProfile::DeepSpace;
        let (effective, blocker) =
            apply_flow_permission_ceiling(&cfg, Some(PermissionProfile::Orbit));
        assert_eq!(effective.permission_profile, PermissionProfile::DeepSpace);
        assert!(blocker.is_none());
    }

    #[test]
    fn ade_adapter_never_interpolates_prompt_text() {
        let command = ade_prompt_command("cursor-agent");
        assert!(!command.contains("{prompt}"));
        assert!(!command.contains("&& touch injected"));
        if cfg!(windows) {
            let encoded = command
                .split_whitespace()
                .last()
                .expect("encoded command payload");
            let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
                .expect("valid PowerShell base64");
            let units = bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect::<Vec<_>>();
            let script = String::from_utf16(&units).expect("valid UTF-16LE PowerShell");
            assert_eq!(script, "& 'cursor-agent' $env:PYTXO_TASK_PROMPT");
        } else {
            assert!(command.contains("PYTXO_TASK_PROMPT"));
        }
    }

    #[test]
    fn staged_overlaps_pass_dispatch_revalidation_but_wave_drift_does_not() {
        let cfg = PytxoConfig::default();
        let tasks = vec![
            Task {
                id: TaskId("one".into()),
                agent: "a".into(),
                paths: vec!["src/shared.rs".into()],
                depends_on: vec![],
                root: None,
                signal_fidelity: None,
                verify: vec!["cargo test".into()],
            },
            Task {
                id: TaskId("two".into()),
                agent: "b".into(),
                paths: vec!["src/shared.rs".into()],
                depends_on: vec![],
                root: None,
                signal_fidelity: None,
                verify: vec!["cargo test".into()],
            },
        ];
        let execution = plan_tasks(&tasks, &cfg).unwrap();
        let waves: Vec<Vec<String>> = execution
            .waves
            .iter()
            .map(|wave| wave.iter().map(|task| task.task_id.0.clone()).collect())
            .collect();
        assert_eq!(waves.len(), 2, "overlapping claims must be staged");
        validate_dispatch_task_plan(&tasks, &cfg, &waves, None).unwrap();

        let stale = vec![vec!["one".into(), "two".into()]];
        let error = validate_dispatch_task_plan(&tasks, &cfg, &stale, None).unwrap_err();
        assert!(error.to_string().contains("scheduler result changed"));
    }
}
