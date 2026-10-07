//! Prepared hosted Shadow control path. This has no production caller or HTTP
//! implementation yet: reviewed hosted Flow remains review-only, and the
//! Proxy's paid-send gate remains closed. Its persisted-review lookup accepts
//! only an undispatched Flow, so a future active-Run caller must first add an
//! exact claimed-Flow/Run owner validator. An injected client permits offline
//! proof of the local authority and one-send sequence.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use pytxo_core::routing::{
    AdviceEnvelope, AdviceRequestId, AdviceStatus, AdviceUsageStatus, Digest, RouteReason,
    RouteSelection, RoutingMode,
};
use pytxo_core::TaskId;
use pytxo_planner::advisor::{
    build_hosted_evaluation_request, hosted_scope_digest, parse_hosted_evaluation_receipt,
    HostedEvaluationReceipt, UnavailableReason, HOSTED_RECIPIENT, MODEL_ID,
};
use pytxo_store::routing::{RoutingFacts, RoutingScope, TaskRoutingState};
use pytxo_store::{Catalog, HostedAdvisorConsentReview, HostedGrantState, PytxoStore};

const ADVISOR_DEADLINE: Duration = Duration::from_secs(2);

pub struct HostedEvaluationRequest {
    pub body: Vec<u8>,
    pub request_id: String,
    pub packet_digest: Digest,
    pub workspace_id: String,
    pub grant_revision: u64,
}

pub type HostedClientFuture<'a> =
    Pin<Box<dyn Future<Output = std::result::Result<Vec<u8>, UnavailableReason>> + Send + 'a>>;

/// A future network implementation must obtain a Link routing token for this
/// exact account, workspace and grant revision. The controller never receives
/// or forwards a worker credential.
pub trait HostedShadowClient: Send + Sync {
    fn recipient_identity(&self) -> &str;
    fn account_id(&self) -> &str;
    fn link_origin(&self) -> &str;
    fn workspace_id(&self) -> &str;
    fn grant_revision(&self) -> u64;
    fn evaluate<'a>(&'a self, request: &'a HostedEvaluationRequest) -> HostedClientFuture<'a>;
}

fn uuid_v7_at(now_ms: u64) -> Option<AdviceRequestId> {
    if now_ms >= (1u64 << 48) {
        return None;
    }
    let mut bytes = uuid::Uuid::new_v4().into_bytes();
    bytes[..6].copy_from_slice(&now_ms.to_be_bytes()[2..]);
    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Some(AdviceRequestId(uuid::Uuid::from_bytes(bytes).to_string()))
}

fn response_with_deadline(
    client: Arc<dyn HostedShadowClient>,
    request: HostedEvaluationRequest,
) -> Option<(Vec<u8>, u64)> {
    let started = Instant::now();
    let deadline = started + ADVISOR_DEADLINE;
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker_cancelled = Arc::clone(&cancelled);
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("pytxo-hosted-shadow".into())
        .spawn(move || {
            if worker_cancelled.load(Ordering::Acquire) || Instant::now() >= deadline {
                return;
            }
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            let result = runtime.block_on(async {
                if worker_cancelled.load(Ordering::Acquire) || Instant::now() >= deadline {
                    return None;
                }
                tokio::time::timeout_at(
                    tokio::time::Instant::from_std(deadline),
                    client.evaluate(&request),
                )
                .await
                .ok()
            });
            let _ = sender.send(result);
        })
        .ok()?;
    let response = receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()));
    cancelled.store(true, Ordering::Release);
    if Instant::now() >= deadline {
        return None;
    }
    match response {
        Ok(Some(Ok(bytes))) => Some((bytes, started.elapsed().as_millis() as u64)),
        _ => None,
    }
}

/// Derive all disclosure bytes from the registered private mission, then
/// require the exact physical Store, local consent and confirmed Link grant.
/// An unavailable reply leaves execution on rules; advice never selects a
/// worker here. The only dispatch caller is the explicit, fake-client
/// experiment; production hosted sends remain disabled.
pub(crate) fn observe_hosted_shadow_once(
    catalog: &Catalog,
    draft_id: &str,
    scope: &RoutingScope,
    task_id: &TaskId,
    facts: &RoutingFacts,
    client: Arc<dyn HostedShadowClient>,
) -> Result<Option<(RoutingFacts, String)>> {
    observe_hosted_shadow_once_inner(catalog, draft_id, scope, task_id, facts, client, || {
        crate::flow::validated_claimed_hosted_advisor_packet(catalog, draft_id).ok()
    })
}

/// Offline protocol tests can supply a reviewed packet without making their
/// synthetic Store and Catalog rows look like a persisted Flow. This seam is
/// absent from release builds.
#[cfg(test)]
pub(crate) fn observe_hosted_shadow_once_with_review(
    catalog: &Catalog,
    draft_id: &str,
    scope: &RoutingScope,
    task_id: &TaskId,
    facts: &RoutingFacts,
    client: Arc<dyn HostedShadowClient>,
    reviewed_packet: impl Fn() -> Option<crate::flow::ReviewedHostedAdvisorPacketPreview>,
) -> Result<Option<(RoutingFacts, String)>> {
    observe_hosted_shadow_once_inner(
        catalog,
        draft_id,
        scope,
        task_id,
        facts,
        client,
        reviewed_packet,
    )
}

fn observe_hosted_shadow_once_inner(
    catalog: &Catalog,
    draft_id: &str,
    scope: &RoutingScope,
    task_id: &TaskId,
    facts: &RoutingFacts,
    client: Arc<dyn HostedShadowClient>,
    reviewed_packet: impl Fn() -> Option<crate::flow::ReviewedHostedAdvisorPacketPreview>,
) -> Result<Option<(RoutingFacts, String)>> {
    if !cfg!(windows) {
        return Ok(None);
    }
    let Some(locator) = catalog.routing_advisor_consent_store(&scope.domain_id.0)? else {
        return Ok(None);
    };
    let store_path = PathBuf::from(&locator.store_db_path);
    if !store_path.is_absolute()
        || pytxo_runner::file_identity(&store_path)? != locator.store_db_file_identity
    {
        return Ok(None);
    }
    let guard = pytxo_runner::FileIdentityGuard::acquire(&store_path)?;
    if guard.identity() != locator.store_db_file_identity {
        return Ok(None);
    }
    let store = PytxoStore::open_existing_read_write(&store_path)?;
    let Some(history) = store.routing_history(scope)? else {
        return Ok(None);
    };
    let mission = &history.mission;
    let Some(task) = mission
        .tasks
        .iter()
        .find(|task| &task.contract.task_id == task_id)
    else {
        return Ok(None);
    };
    if mission.tasks.len() != 1
        || mission.authorization.domain_id != scope.domain_id
        || mission.authorization.run_id != scope.run_id
        || mission.authorization.live_advice_authorized
        || mission.policy.mode != RoutingMode::Shadow
        || mission.policy.advisor_recipient.as_deref() != Some(HOSTED_RECIPIENT)
        || mission.policy.advice_model != MODEL_ID
        || mission.policy.disclosure_scope_digest != Some(hosted_scope_digest())
        || !task.contract.has_recordable_advice_context()
    {
        return Ok(None);
    }
    let packet = crate::flow::reviewed_task_advisor_packet(&task.contract)?;
    let packet_body = serde_json::to_vec(&packet)?;
    let packet_digest = packet.digest();
    let request_digest = Digest::of_bytes(
        &packet
            .request_body()
            .map_err(|_| anyhow::anyhow!("invalid hosted advisor packet"))?,
    );
    if mission.policy.advice_template
        != crate::flow::reviewed_hosted_routing_advisor_identity_for_task(&task.contract)?
    {
        return Ok(None);
    }
    let expected_review = HostedAdvisorConsentReview {
        domain_id: scope.domain_id.0.clone(),
        recipient_identity: HOSTED_RECIPIENT.into(),
        draft_id: draft_id.into(),
        consent_revision: mission.authorization.consent_revision,
        scope_digest: hosted_scope_digest().0,
        packet_digest: packet_digest.0.clone(),
        request_digest: request_digest.0,
        store_db_file_identity: locator.store_db_file_identity.clone(),
    };
    let review_is_current = || {
        reviewed_packet().is_some_and(|reviewed| {
            reviewed.domain_id == expected_review.domain_id
                && reviewed.run_id == scope.run_id.0
                && reviewed.task_id == task_id.0
                && reviewed.reviewed_consent_revision == expected_review.consent_revision
                && reviewed.recipient_identity == expected_review.recipient_identity
                && reviewed.scope_digest.0 == expected_review.scope_digest
                && reviewed.packet_digest.0 == expected_review.packet_digest
                && reviewed.request_digest.0 == expected_review.request_digest
                && reviewed.store_db_file_identity == expected_review.store_db_file_identity
                && reviewed.packet_body == packet_body
                && reviewed.wire_schema_version == 1
                && reviewed.decision_kind == "initial_demand"
                && reviewed.question_set_version == pytxo_planner::advisor::TEMPLATE_VERSION
                && reviewed.recordable_shadow_context
        })
    };
    if !review_is_current() {
        return Ok(None);
    }
    let Some(binding) = catalog.hosted_grant(&scope.domain_id.0)? else {
        return Ok(None);
    };
    if binding.state != HostedGrantState::Enabled
        || client.recipient_identity() != HOSTED_RECIPIENT
        || binding.intent.account_id != client.account_id()
        || binding.intent.link_origin != client.link_origin()
        || binding.intent.workspace_id != client.workspace_id()
        || binding.remote_revision != Some(client.grant_revision())
        || catalog
            .confirmed_hosted_grant_for_send(&binding.intent, &expected_review)
            .is_err()
    {
        return Ok(None);
    }
    let consent = store.routing_hosted_advisor_consent(&scope.domain_id, HOSTED_RECIPIENT)?;
    if !consent.enabled
        || consent.revision != mission.authorization.consent_revision
        || consent.scope_digest != mission.policy.disclosure_scope_digest
    {
        return Ok(None);
    }
    let mut bounded = facts.clone();
    bounded.packet_digest = Some(packet_digest.clone());
    bounded.advice_request_id = uuid_v7_at(facts.now_ms);
    let Some(request_id) = bounded.advice_request_id.clone() else {
        return Ok(None);
    };
    let wire = build_hosted_evaluation_request(&request_id.0, &packet)
        .map_err(|_| anyhow::anyhow!("invalid hosted evaluation wire"))?;
    let rules = store.preview_routing_decision(scope, task_id, &bounded, None)?;
    if rules.reason != RouteReason::StrongDefault
        || rules.selection != RouteSelection::Selected(mission.policy.strong.clone())
    {
        return Ok(None);
    }
    let catalog_digest = store.routing_catalog_digest(scope, task_id, &bounded)?;
    let Ok(prepared) =
        store.prepare_routing_hosted_advisor_request(scope, task_id, &bounded, HOSTED_RECIPIENT)
    else {
        return Ok(None);
    };
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_PREPARE").is_some() {
        panic!("injected hosted Shadow controller death after durable prepare");
    }
    let Ok(true) = store.mark_routing_advisor_send_may_have_happened(
        scope,
        &request_id,
        &bounded,
        bounded.now_ms,
    ) else {
        return Ok(None);
    };
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_MAY_SEND").is_some() {
        panic!("injected hosted Shadow controller death after durable may-send");
    }
    if catalog
        .confirmed_hosted_grant_for_send(&binding.intent, &expected_review)
        .is_err()
        || !review_is_current()
        || pytxo_runner::file_identity(&store_path)? != locator.store_db_file_identity
        || {
            let current =
                store.routing_hosted_advisor_consent(&scope.domain_id, HOSTED_RECIPIENT)?;
            !current.enabled
                || current.revision != prepared.consent_revision
                || current.scope_digest != mission.policy.disclosure_scope_digest
        }
    {
        store.settle_routing_advisor_request(scope, &request_id, None, bounded.now_ms)?;
        return Ok(None);
    }
    let request = HostedEvaluationRequest {
        body: wire,
        request_id: request_id.0.clone(),
        packet_digest: packet_digest.clone(),
        workspace_id: binding.intent.workspace_id.clone(),
        grant_revision: binding
            .remote_revision
            .ok_or_else(|| anyhow::anyhow!("missing confirmed grant revision"))?,
    };
    let result = response_with_deadline(client, request);
    let received_at_ms = bounded.now_ms.saturating_add(
        result
            .as_ref()
            .map_or(ADVISOR_DEADLINE.as_millis() as u64, |(_, elapsed)| *elapsed),
    );
    let Some((reply, elapsed_ms)) = result else {
        store.settle_routing_advisor_request(scope, &request_id, None, received_at_ms)?;
        return Ok(None);
    };
    let Ok(receipt) = parse_hosted_evaluation_receipt(&reply, &request_id.0, &packet_digest) else {
        store.settle_routing_advisor_request(scope, &request_id, None, received_at_ms)?;
        return Ok(None);
    };
    let raw = hosted_advice_json(&prepared, mission, &receipt, received_at_ms, elapsed_ms)?;
    store.settle_routing_advisor_request(
        scope,
        &request_id,
        Some(Digest::of_bytes(raw.as_bytes())),
        received_at_ms,
    )?;
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_COMPLETED").is_some() {
        panic!("injected hosted Shadow controller death after durable completion");
    }
    if catalog
        .confirmed_hosted_grant_for_send(&binding.intent, &expected_review)
        .is_err()
        || !review_is_current()
        || pytxo_runner::file_identity(&store_path)? != locator.store_db_file_identity
    {
        return Ok(None);
    }
    let current = store.routing_hosted_advisor_consent(&scope.domain_id, HOSTED_RECIPIENT)?;
    let Some(latest) = store.routing_history(scope)? else {
        return Ok(None);
    };
    if !current.enabled
        || current.revision != prepared.consent_revision
        || current.scope_digest != mission.policy.disclosure_scope_digest
        || latest.cancelled
        || latest.cancel_epoch != prepared.cancel_epoch
        || latest.mission != *mission
        || !latest.tasks.iter().any(|task| {
            task.registration.contract.task_id == *task_id
                && task.state == TaskRoutingState::Ready
                && task.revision == prepared.task_state_revision
                && task.next_ordinal == prepared.ordinal
        })
    {
        return Ok(None);
    }
    bounded.now_ms = received_at_ms;
    if store.routing_catalog_digest(scope, task_id, &bounded)? != catalog_digest
        || store
            .preview_routing_decision(scope, task_id, &bounded, Some(&raw))?
            .advice_status
            != AdviceStatus::ShadowRecorded
    {
        return Ok(None);
    }
    Ok(Some((bounded, raw)))
}

fn hosted_advice_json(
    prepared: &pytxo_store::routing::RoutingAdvisorRequest,
    mission: &pytxo_store::routing::RoutingMission,
    receipt: &HostedEvaluationReceipt,
    received_at_ms: u64,
    elapsed_ms: u64,
) -> Result<String> {
    if receipt.usage_receipt_id().is_empty() {
        bail!("hosted usage receipt is absent");
    }
    Ok(serde_json::to_string(&AdviceEnvelope {
        schema_version: 1,
        request_id: prepared.request_id.clone(),
        packet_digest: prepared.packet_digest.clone(),
        policy_version: mission.policy.version.clone(),
        template_version: mission.policy.advice_template.clone(),
        model_id: MODEL_ID.into(),
        task_revision: prepared.task_revision,
        task_state_revision: prepared.task_state_revision,
        authorization_revision: prepared.authorization_revision,
        cancel_epoch: prepared.cancel_epoch,
        consent_revision: prepared.consent_revision,
        received_at_ms,
        expires_at_ms: received_at_ms.saturating_add(60_000),
        packet_complete: true,
        choice: receipt.choice(),
        distribution: receipt.distribution().clone(),
        usage_status: AdviceUsageStatus::Known,
        usage_receipt_id: Some(receipt.usage_receipt_id().into()),
        elapsed_ms,
    })?)
}
