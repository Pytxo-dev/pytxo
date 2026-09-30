//! Experimental shadow-only advisor controller. Store and Core decide whether
//! a send is eligible and whether its result can be observed. No caller here
//! may select a worker, authorize spending, or launch an attempt.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::{Duration, Instant};

use anyhow::Result;
use pytxo_core::routing::{
    AdviceEnvelope, AdviceRequestId, AdviceStatus, AdviceUsageStatus, Digest, RouteReason,
    RouteSelection, RoutingMode,
};
use pytxo_core::TaskId;
use pytxo_planner::advisor::{
    Advisor, AdvisorContext, AdvisorOutcome, AdvisorPacket, AdvisorRequest, DisclosurePermit,
    MODEL_ID,
};
use pytxo_store::routing::{
    ObserveRoutingDecision, RoutingFacts, RoutingMission, RoutingScope, TaskRoutingState,
};
use pytxo_store::PytxoStore;

const ADVISOR_DEADLINE: Duration = Duration::from_secs(2);

#[cfg(feature = "routed-test-faults")]
pub(crate) fn mock_fixture_advisor() -> Option<Arc<dyn Advisor>> {
    use pytxo_planner::advisor::{
        AdvisorTransport, FixedRequest, JevAdvisor, TransportFuture, UnavailableReason,
    };

    struct FixedMockTransport;
    impl AdvisorTransport for FixedMockTransport {
        fn recipient_identity(&self) -> &str {
            crate::flow::REVIEWED_ADVISOR_RECIPIENT
        }

        fn send<'a>(&'a self, request: FixedRequest<'a>) -> TransportFuture<'a> {
            Box::pin(async move {
                let body: serde_json::Value = serde_json::from_slice(request.body())
                    .map_err(|_| UnavailableReason::InvalidPacket)?;
                let Some(goal) = body["state"]["goal"].as_str() else {
                    return Err(UnavailableReason::InvalidPacket);
                };
                if goal != "Classify reviewed repository task using coarse facts" {
                    return Err(UnavailableReason::InvalidPacket);
                }
                Ok(br#"{"model":"jev-1.13.0","answers":{"execution_demand_v2":{"type":"choice","choice":"everyday_fit","probabilities":{"everyday_fit":0.9,"strong_needed":0.08,"unclear":0.02},"confidence":0.9}},"usage":{"input_tokens":10,"output_tokens":1}}"#.to_vec())
            })
        }
    }

    std::env::var_os("PYTXO_TEST_ROUTED_SHADOW_MOCK_ADVICE")
        .map(|_| Arc::new(JevAdvisor::with_transport(FixedMockTransport)) as Arc<dyn Advisor>)
}

/// Revocation after a response but before observation degrades to a freshly
/// checked rules observation. A changed task/cancel state still fails closed.
pub(crate) fn observe_with_rules_fallback(
    store: &PytxoStore,
    mut request: ObserveRoutingDecision,
) -> Result<ObserveRoutingDecision> {
    match store.observe_routing_decision(&request) {
        Ok(_) => return Ok(request),
        Err(error) if request.advice_json.is_none() => return Err(error.into()),
        Err(error) => {
            let Some(history) = store.routing_history(&request.scope)? else {
                return Err(error.into());
            };
            if history.mission.policy.mode != RoutingMode::Shadow
                || history.mission.policy.advice_model != MODEL_ID
            {
                return Err(error.into());
            }
        }
    }
    request.advice_json = None;
    request.decision =
        store.preview_routing_decision(&request.scope, &request.task_id, &request.facts, None)?;
    let snapshot = store.routing_snapshot(&request.scope, &request.task_id, &request.facts)?;
    request.expected_cancel_epoch = snapshot.cancel_epoch;
    request.expected_next_ordinal = snapshot.next_ordinal;
    store.observe_routing_decision(&request)?;
    Ok(request)
}

/// Returns only a current, journal-backed shadow observation. `None` means
/// execute the deterministic rules preview. The trusted caller projects the
/// reviewed task; worker output never becomes an advisor packet.
pub(crate) fn observe_shadow_once(
    store: &PytxoStore,
    mission: &RoutingMission,
    scope: &RoutingScope,
    task_id: &TaskId,
    facts: &RoutingFacts,
    packet: AdvisorPacket,
    advisor: Arc<dyn Advisor>,
) -> Result<Option<(RoutingFacts, String)>> {
    observe_shadow_once_with_start_delay(
        store,
        mission,
        scope,
        task_id,
        facts,
        packet,
        advisor,
        Duration::ZERO,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "test-only scheduling delay is explicit"
)]
fn observe_shadow_once_with_start_delay(
    store: &PytxoStore,
    mission: &RoutingMission,
    scope: &RoutingScope,
    task_id: &TaskId,
    facts: &RoutingFacts,
    packet: AdvisorPacket,
    advisor: Arc<dyn Advisor>,
    worker_start_delay: Duration,
) -> Result<Option<(RoutingFacts, String)>> {
    if mission.policy.mode != RoutingMode::Shadow
        || mission.policy.advice_model != MODEL_ID
        || mission.policy.advisor_recipient.is_some()
        || advisor.recipient_identity() != crate::flow::REVIEWED_ADVISOR_RECIPIENT
        || !packet.is_complete()
    {
        return Ok(None);
    }
    let Some(history) = store.routing_history(scope)? else {
        return Ok(None);
    };
    if history.mission != *mission {
        return Ok(None);
    }
    let Some(registered) = history
        .mission
        .tasks
        .iter()
        .find(|entry| &entry.contract.task_id == task_id)
    else {
        return Ok(None);
    };
    let Ok(expected_packet) = crate::flow::reviewed_task_advisor_packet(&registered.contract)
    else {
        return Ok(None);
    };
    if packet != expected_packet {
        return Ok(None);
    }
    let Ok(request_identity) =
        crate::flow::reviewed_routing_advisor_identity_for_task(&registered.contract)
    else {
        return Ok(None);
    };
    if mission.policy.advice_template != request_identity
        || mission.policy.disclosure_scope_digest
            != Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest())
    {
        return Ok(None);
    }
    let consent = store.routing_advisor_consent(&scope.domain_id)?;
    if !consent.enabled
        || consent.revision != mission.authorization.consent_revision
        || consent.scope_digest != mission.policy.disclosure_scope_digest
    {
        return Ok(None);
    }
    let initial = store.routing_snapshot(scope, task_id, facts)?;
    let mut bounded = facts.clone();
    bounded.packet_digest = Some(packet.digest());
    let identity = format!(
        "{}/{}/{}/{}",
        scope.domain_id.0, scope.run_id.0, task_id.0, initial.next_ordinal
    );
    bounded.advice_request_id = Some(AdviceRequestId(format!(
        "jev:{}",
        Digest::of_bytes(identity.as_bytes()).0
    )));
    let rules = store.preview_routing_decision(scope, task_id, &bounded, None)?;
    if rules.reason != RouteReason::StrongDefault
        || rules.selection != RouteSelection::Selected(mission.policy.strong.clone())
    {
        return Ok(None);
    }
    let catalog_digest = store.routing_catalog_digest(scope, task_id, &bounded)?;
    let Ok(prepared) = store.prepare_routing_advisor_request(scope, task_id, &bounded) else {
        return Ok(None);
    };
    let Ok(true) = store.mark_routing_advisor_send_may_have_happened(
        scope,
        &prepared.request_id,
        &bounded,
        bounded.now_ms,
    ) else {
        return Ok(None);
    };
    // A committed revocation before this read suppresses the call. The
    // may-send mark has already committed: revocation before the worker's
    // eventual transport poll can still race this read. A hosted transport
    // must resolve that window before claiming revocation prevents disclosure.
    let current_consent = store.routing_advisor_consent(&scope.domain_id)?;
    if !current_consent.enabled
        || current_consent.revision != prepared.consent_revision
        || current_consent.scope_digest != mission.policy.disclosure_scope_digest
    {
        store.settle_routing_advisor_request(scope, &prepared.request_id, None, bounded.now_ms)?;
        return Ok(None);
    }

    let context = AdvisorContext {
        request_id: prepared.request_id.0.clone(),
        task_revision: prepared.task_revision,
        task_state_revision: prepared.task_state_revision,
        authorization_revision: prepared.authorization_revision,
        cancel_epoch: prepared.cancel_epoch,
        consent_revision: prepared.consent_revision,
        policy_digest: prepared.policy_digest.clone(),
        catalog_digest: catalog_digest.clone(),
    };
    let request = AdvisorRequest::new(packet, context);
    let permit = DisclosurePermit::new(
        &prepared.request_id.0,
        prepared.consent_revision,
        prepared.packet_digest.clone(),
        crate::flow::REVIEWED_ADVISOR_RECIPIENT,
    );
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker_cancelled = Arc::clone(&cancelled);
    let (sender, receiver) = mpsc::sync_channel(1);
    let started = Instant::now();
    let deadline = started + ADVISOR_DEADLINE;
    let worker = std::thread::Builder::new()
        .name("pytxo-jev-shadow".into())
        .spawn(move || {
            if !worker_start_delay.is_zero() {
                std::thread::sleep(worker_start_delay);
            }
            if worker_cancelled.load(Ordering::Acquire) || Instant::now() >= deadline {
                return;
            }
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let Ok(runtime) = runtime else { return };
            if worker_cancelled.load(Ordering::Acquire) || Instant::now() >= deadline {
                return;
            }
            let outcome = runtime.block_on(async {
                if worker_cancelled.load(Ordering::Acquire) || Instant::now() >= deadline {
                    return None;
                }
                tokio::time::timeout_at(
                    tokio::time::Instant::from_std(deadline),
                    advisor.advise(&request, &permit),
                )
                .await
                .ok()
            });
            let _ = sender.send(outcome);
        });
    if worker.is_err() {
        store.settle_routing_advisor_request(scope, &prepared.request_id, None, bounded.now_ms)?;
        return Ok(None);
    }
    let outcome = receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()));
    cancelled.store(true, Ordering::Release);
    let within_deadline = Instant::now() < deadline;
    let received_at_ms = bounded
        .now_ms
        .saturating_add(started.elapsed().as_millis() as u64);
    let Ok(Some(AdvisorOutcome::Observed {
        observation,
        elapsed_ms,
    })) = outcome
    else {
        store.settle_routing_advisor_request(scope, &prepared.request_id, None, received_at_ms)?;
        return Ok(None);
    };
    if !within_deadline || observation.model_id != MODEL_ID {
        store.settle_routing_advisor_request(scope, &prepared.request_id, None, received_at_ms)?;
        return Ok(None);
    }
    let envelope = AdviceEnvelope {
        schema_version: 1,
        request_id: prepared.request_id.clone(),
        packet_digest: prepared.packet_digest.clone(),
        policy_version: mission.policy.version.clone(),
        template_version: mission.policy.advice_template.clone(),
        model_id: observation.model_id,
        task_revision: prepared.task_revision,
        task_state_revision: prepared.task_state_revision,
        authorization_revision: prepared.authorization_revision,
        cancel_epoch: prepared.cancel_epoch,
        consent_revision: prepared.consent_revision,
        received_at_ms,
        expires_at_ms: received_at_ms.saturating_add(60_000),
        packet_complete: true,
        choice: observation.choice,
        distribution: observation.distribution,
        usage_status: AdviceUsageStatus::Unknown,
        usage_receipt_id: None,
        elapsed_ms,
    };
    let raw = serde_json::to_string(&envelope)?;
    store.settle_routing_advisor_request(
        scope,
        &prepared.request_id,
        Some(Digest::of_bytes(raw.as_bytes())),
        received_at_ms,
    )?;
    let current_consent = store.routing_advisor_consent(&scope.domain_id)?;
    let history = store.routing_history(scope)?;
    if !current_consent.enabled
        || current_consent.revision != prepared.consent_revision
        || current_consent.scope_digest != mission.policy.disclosure_scope_digest
        || history.as_ref().is_none_or(|history| {
            history.cancelled
                || history.cancel_epoch != prepared.cancel_epoch
                || history.mission != *mission
                || !history.tasks.iter().any(|task| {
                    task.registration.contract.task_id == *task_id
                        && task.state == TaskRoutingState::Ready
                        && task.revision == prepared.task_state_revision
                        && task.next_ordinal == prepared.ordinal
                })
        })
    {
        return Ok(None);
    }
    bounded.now_ms = received_at_ms;
    let fresh_catalog_digest = store.routing_catalog_digest(scope, task_id, &bounded)?;
    if fresh_catalog_digest != catalog_digest
        || store
            .preview_routing_decision(scope, task_id, &bounded, Some(&raw))?
            .advice_status
            != AdviceStatus::ShadowRecorded
    {
        return Ok(None);
    }
    Ok(Some((bounded, raw)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use pytxo_core::routing::{
        qualification_fingerprint, AdapterQualification, AdviceChoice, ExecutableIdentity,
        LaunchContract, LaunchTransport, MeteringSupport, ModelIdentityLevel, ProfileObservation,
        Readiness, StdinDelivery, TaskKind,
    };
    use pytxo_core::{DomainId, PermissionProfile, RunId};
    use pytxo_planner::advisor::{
        AdvisorTransport, FixedRequest, JevAdvisor, TransportFuture, UnavailableReason,
    };
    use pytxo_store::routing::{AdvisorSendPhase, ObserveRoutingDecision, RoutingMission};

    fn digest(value: &str) -> Digest {
        Digest::of_bytes(value.as_bytes())
    }

    fn setup() -> (
        tempfile::TempDir,
        PytxoStore,
        RoutingMission,
        RoutingScope,
        RoutingFacts,
    ) {
        setup_with_mode(true, false)
    }

    fn setup_with_request_identity(
        valid: bool,
    ) -> (
        tempfile::TempDir,
        PytxoStore,
        RoutingMission,
        RoutingScope,
        RoutingFacts,
    ) {
        setup_with_mode(valid, false)
    }

    fn setup_with_mode(
        valid: bool,
        hosted: bool,
    ) -> (
        tempfile::TempDir,
        PytxoStore,
        RoutingMission,
        RoutingScope,
        RoutingFacts,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
        db.insert_run("run", "repo").unwrap();
        let mut mission: RoutingMission = serde_json::from_str(include_str!(
            "../../pytxo-store/tests/fixtures/routing_pre_check_recipes_v8_registration.json"
        ))
        .unwrap();
        mission.policy.mode = RoutingMode::Shadow;
        mission.policy.advice_model = MODEL_ID.into();
        mission.tasks[0].contract.task_kind = Some(if hosted {
            TaskKind::Diagnosis
        } else {
            TaskKind::Other
        });
        if hosted {
            mission.tasks[0].contract.context_complete = true;
            mission.tasks[0].contract.cross_component_requirement = Some(false);
            mission.policy.advisor_recipient =
                Some(pytxo_planner::advisor::HOSTED_RECIPIENT.into());
            mission.policy.advice_template =
                crate::flow::reviewed_hosted_routing_advisor_identity_for_task(
                    &mission.tasks[0].contract,
                )
                .unwrap();
            mission.policy.disclosure_scope_digest =
                Some(pytxo_planner::advisor::hosted_scope_digest());
        } else {
            mission.policy.advice_template =
                crate::flow::reviewed_routing_advisor_identity_for_task(&mission.tasks[0].contract)
                    .unwrap();
            mission.policy.disclosure_scope_digest =
                Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest());
        }
        if !valid {
            mission.policy.advice_template.push_str(":stale-request");
        }
        mission.authorization.policy_digest = mission.policy.digest().unwrap();
        mission.authorization.allowed_task_digests =
            [mission.tasks[0].contract.digest().unwrap()].into();
        let scope = RoutingScope {
            domain_id: DomainId("domain".into()),
            run_id: RunId("run".into()),
        };
        let observations = mission
            .profiles
            .iter()
            .map(|entry| {
                let executable = ExecutableIdentity {
                    path: format!("C:/tools/{}.exe", entry.profile.id.0),
                    version: "1.0".into(),
                    digest: digest("executable"),
                };
                let launch = LaunchContract {
                    schema_version: 1,
                    transport: LaunchTransport::HostPty,
                    host: Some(ExecutableIdentity {
                        path: "C:/tools/pytxo-attempt-host.exe".into(),
                        version: "1.0".into(),
                        digest: digest("host"),
                    }),
                    dependencies: vec![],
                    arguments_digest: digest("argv"),
                    environment_policy_digest: digest("environment"),
                    working_directory_policy: "reviewed_attempt_worktree_v1".into(),
                    stdin_delivery: StdinDelivery::PrivateHostPipe,
                    private_stdin_digest: Some(digest("private-stdin")),
                    output_protocol: "pytxo-attempt-host/1".into(),
                    argument_lowering: "rust-std-command-windows-structured-argv/v1".into(),
                    barrier_timeout_ms: Some(5_000),
                    execution_timeout_ms: 30_000,
                    settlement_timeout_ms: 5_000,
                    output_limit_bytes: 4096,
                };
                ProfileObservation {
                    schema_version: 1,
                    profile_digest: entry.profile.digest().unwrap(),
                    binding_digest: entry.binding.digest().unwrap(),
                    executable: executable.clone(),
                    launch: Some(launch.clone()),
                    observed_at_ms: 100,
                    expires_at_ms: 10_000,
                    auth_status: Readiness::Ready,
                    dispatch_supported: true,
                    qualification: Some(AdapterQualification {
                        receipt_digest: digest("qualification"),
                        launch_fingerprint: qualification_fingerprint(
                            &entry.profile,
                            &entry.binding,
                            &executable,
                            &launch,
                        )
                        .unwrap(),
                        permission_profile: PermissionProfile::Orbit,
                        tool_probe_passed: true,
                        cancellation_probe_passed: true,
                        quiescence_probe_passed: true,
                        capabilities: BTreeSet::from(["edit".into(), "check".into()]),
                        allowed_egress: BTreeSet::new(),
                        capacity_pool_ids: BTreeSet::from(["host-worker".into(), "account".into()]),
                    }),
                    requested_model: entry.profile.requested_model.clone(),
                    reported_model: Some(entry.profile.requested_model.clone()),
                    model_identity_level: ModelIdentityLevel::HarnessReported,
                    metering_support: MeteringSupport::Unknown,
                    hard_spend_limit_verified: false,
                    capacity_ready: true,
                }
            })
            .collect::<Vec<_>>();
        let facts = RoutingFacts {
            now_ms: 150,
            observed_at_ms: 100,
            expires_at_ms: 10_000,
            base: mission.tasks[0].contract.base.clone(),
            plan_digest: mission.authorization.plan_digest.clone(),
            permission_profile: PermissionProfile::Orbit,
            observations,
            manual_target: None,
            packet_digest: None,
            advice_request_id: None,
        };
        db.register_routing_mission(&mission).unwrap();
        for (i, observation) in facts.observations.iter().enumerate() {
            db.register_routing_qualification(
                &scope,
                &format!("qualification-{i}"),
                observation.qualification.as_ref().unwrap(),
            )
            .unwrap();
        }
        (temp, db, mission, scope, facts)
    }

    fn packet(mission: &RoutingMission) -> AdvisorPacket {
        crate::flow::reviewed_task_advisor_packet(&mission.tasks[0].contract).unwrap()
    }

    #[cfg(windows)]
    #[test]
    fn hosted_shadow_fake_client_records_one_bound_observation_but_keeps_rules_target() {
        use crate::routed_hosted_shadow::{
            observe_hosted_shadow_once, observe_hosted_shadow_once_with_review, HostedClientFuture,
            HostedEvaluationRequest, HostedShadowClient,
        };
        use pytxo_store::{
            Catalog, HostedAdvisorConsentFence, HostedAdvisorConsentReview, HostedGrantIntent,
            HostedRemoteGrantReceipt,
        };

        struct FakeClient {
            account_id: String,
            link_origin: String,
            workspace_id: String,
            grant_revision: u64,
            sends: Arc<AtomicUsize>,
            revoke_during_send: Option<(PathBuf, HostedAdvisorConsentFence)>,
        }
        impl HostedShadowClient for FakeClient {
            fn recipient_identity(&self) -> &str {
                pytxo_planner::advisor::HOSTED_RECIPIENT
            }
            fn account_id(&self) -> &str {
                &self.account_id
            }
            fn link_origin(&self) -> &str {
                &self.link_origin
            }
            fn workspace_id(&self) -> &str {
                &self.workspace_id
            }
            fn grant_revision(&self) -> u64 {
                self.grant_revision
            }
            fn evaluate<'a>(
                &'a self,
                request: &'a HostedEvaluationRequest,
            ) -> HostedClientFuture<'a> {
                Box::pin(async move {
                    let value: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
                    assert_eq!(value["request_id"], request.request_id);
                    assert_eq!(value["packet"]["schema_version"], 1);
                    self.sends.fetch_add(1, Ordering::SeqCst);
                    if let Some((path, fence)) = &self.revoke_during_send {
                        Catalog::open(path)
                            .unwrap()
                            .advance_hosted_advisor_consent_fence(fence)
                            .unwrap();
                    }
                    Ok(serde_json::to_vec(&serde_json::json!({
                        "schema_version": 1,
                        "evaluation_id": request.request_id,
                        "request_id": request.request_id,
                        "question_set_version": pytxo_planner::advisor::TEMPLATE_VERSION,
                        "model_id": MODEL_ID,
                        "choice": "everyday_fit",
                        "distribution": {"everyday_fit": 0.9, "strong_needed": 0.08, "unclear": 0.02},
                        "usage_status": "known",
                        "usage_receipt_id": "r_0123456789abcdef0123456789abcdef",
                        "input_tokens": 10,
                        "output_tokens": 1,
                        "cost_nano_usd": 420,
                        "packet_digest": request.packet_digest.0,
                    })).unwrap())
                })
            }
        }

        for revoke_during_send in [false, true] {
            let (temp, db, mission, scope, facts) = setup_with_mode(true, true);
            let recipient = pytxo_planner::advisor::HOSTED_RECIPIENT;
            let store_path = temp.path().join("store.db");
            let store_identity = pytxo_runner::file_identity(&store_path).unwrap();
            let catalog = Catalog::open(&temp.path().join("catalog.db")).unwrap();
            catalog
                .upsert_domain(
                    &scope.domain_id.0,
                    &scope.domain_id.0,
                    store_path.to_str().unwrap(),
                    None,
                )
                .unwrap();
            catalog
                .bind_routing_advisor_consent_store(
                    &scope.domain_id.0,
                    store_path.to_str().unwrap(),
                    &store_identity,
                )
                .unwrap();
            let workspace_id = catalog
                .ensure_hosted_workspace_id(
                    &scope.domain_id.0,
                    store_path.to_str().unwrap(),
                    &store_identity,
                )
                .unwrap();
            let packet = packet(&mission);
            let review = HostedAdvisorConsentReview {
                domain_id: scope.domain_id.0.clone(),
                recipient_identity: recipient.into(),
                draft_id: "reviewed-hosted".into(),
                consent_revision: 1,
                scope_digest: pytxo_planner::advisor::hosted_scope_digest().0,
                packet_digest: packet.digest().0,
                request_digest: Digest::of_bytes(&packet.request_body().unwrap()).0,
                store_db_file_identity: store_identity.clone(),
            };
            let reviewed_packet = crate::flow::ReviewedHostedAdvisorPacketPreview {
                domain_id: scope.domain_id.0.clone(),
                store_db_file_identity: store_identity.clone(),
                run_id: scope.run_id.0.clone(),
                task_id: mission.tasks[0].contract.task_id.0.clone(),
                reviewed_consent_revision: 1,
                recipient_identity: recipient.into(),
                scope_digest: pytxo_planner::advisor::hosted_scope_digest(),
                packet_digest: packet.digest(),
                request_digest: Digest::of_bytes(&packet.request_body().unwrap()),
                wire_schema_version: 1,
                decision_kind: "initial_demand".into(),
                question_set_version: pytxo_planner::advisor::TEMPLATE_VERSION.into(),
                packet_body: serde_json::to_vec(&packet).unwrap(),
                recordable_shadow_context: true,
            };
            catalog
                .record_hosted_advisor_consent_review(&review)
                .unwrap();
            catalog
                .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
                    domain_id: scope.domain_id.0.clone(),
                    recipient_identity: recipient.into(),
                    consent_revision: 1,
                    enabled: true,
                    store_db_file_identity: store_identity.clone(),
                })
                .unwrap();
            db.set_routing_hosted_advisor_consent(
                &scope.domain_id,
                recipient,
                0,
                true,
                Some(pytxo_planner::advisor::hosted_scope_digest()),
                145,
            )
            .unwrap();
            let intent = HostedGrantIntent {
                domain_id: scope.domain_id.0.clone(),
                workspace_id: workspace_id.clone(),
                account_id: "user_test".into(),
                link_origin: "https://link.pytxo.com".into(),
                recipient_identity: recipient.into(),
                scope_digest: review.scope_digest.clone(),
                store_db_file_identity: store_identity,
                consent_revision: 1,
            };
            let sends = Arc::new(AtomicUsize::new(0));
            let client: Arc<dyn HostedShadowClient> = Arc::new(FakeClient {
                account_id: intent.account_id.clone(),
                link_origin: intent.link_origin.clone(),
                workspace_id: intent.workspace_id.clone(),
                grant_revision: 1,
                sends: Arc::clone(&sends),
                revoke_during_send: revoke_during_send.then(|| {
                    (
                        temp.path().join("catalog.db"),
                        HostedAdvisorConsentFence {
                            domain_id: scope.domain_id.0.clone(),
                            recipient_identity: recipient.into(),
                            consent_revision: 2,
                            enabled: false,
                            store_db_file_identity: intent.store_db_file_identity.clone(),
                        },
                    )
                }),
            });
            let task_id = TaskId("task0".into());
            assert!(observe_hosted_shadow_once_with_review(
                &catalog,
                &review.draft_id,
                &scope,
                &task_id,
                &facts,
                Arc::clone(&client),
                || Some(reviewed_packet.clone()),
            )
            .unwrap()
            .is_none());
            assert_eq!(sends.load(Ordering::SeqCst), 0);
            catalog.begin_hosted_grant(&intent).unwrap();
            catalog
                .confirm_hosted_grant(
                    &intent,
                    &HostedRemoteGrantReceipt {
                        workspace_id: intent.workspace_id.clone(),
                        account_id: intent.account_id.clone(),
                        link_origin: intent.link_origin.clone(),
                        recipient_identity: intent.recipient_identity.clone(),
                        scope_digest: intent.scope_digest.clone(),
                        revision: 1,
                        enabled: true,
                    },
                )
                .unwrap();
            // The production entry must reject these synthetic rows: no persisted
            // reviewed Flow exists, even though the grant and Store match.
            assert!(observe_hosted_shadow_once(
                &catalog,
                &review.draft_id,
                &scope,
                &task_id,
                &facts,
                Arc::clone(&client),
            )
            .unwrap()
            .is_none());
            assert_eq!(sends.load(Ordering::SeqCst), 0);
            let observed = observe_hosted_shadow_once_with_review(
                &catalog,
                &review.draft_id,
                &scope,
                &task_id,
                &facts,
                Arc::clone(&client),
                || Some(reviewed_packet.clone()),
            )
            .unwrap();
            assert_eq!(sends.load(Ordering::SeqCst), 1);
            if revoke_during_send {
                assert!(
                    observed.is_none(),
                    "revoked consent admitted a late Shadow reply"
                );
                assert!(!db
                    .routing_history(&scope)
                    .unwrap()
                    .unwrap()
                    .events
                    .iter()
                    .any(|event| matches!(
                        event.event,
                        pytxo_store::routing::RoutingControlEvent::DecisionObserved(_)
                    )));
                let phase: String = rusqlite::Connection::open(&store_path)
                    .unwrap()
                    .query_row(
                        "SELECT phase FROM routing_advisor_requests LIMIT 1",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert_eq!(
                    phase, "completed",
                    "the may-billed reply must remain journaled"
                );
                assert!(observe_hosted_shadow_once_with_review(
                    &catalog,
                    &review.draft_id,
                    &scope,
                    &task_id,
                    &facts,
                    client,
                    || Some(reviewed_packet.clone()),
                )
                .unwrap()
                .is_none());
                assert_eq!(sends.load(Ordering::SeqCst), 1);
                continue;
            }
            let (observed_facts, raw) = observed.unwrap();
            let rules = db
                .preview_routing_decision(&scope, &task_id, &observed_facts, None)
                .unwrap();
            let shadow = db
                .preview_routing_decision(&scope, &task_id, &observed_facts, Some(&raw))
                .unwrap();
            assert_eq!(shadow.selection, rules.selection);
            assert_eq!(shadow.advice_status, AdviceStatus::ShadowRecorded);
            let snapshot = db
                .routing_snapshot(&scope, &task_id, &observed_facts)
                .unwrap();
            db.observe_routing_decision(&ObserveRoutingDecision {
                scope: scope.clone(),
                event_id: "hosted-shadow-observed".into(),
                task_id: task_id.clone(),
                expected_cancel_epoch: snapshot.cancel_epoch,
                expected_next_ordinal: snapshot.next_ordinal,
                facts: observed_facts,
                decision: shadow,
                advice_json: Some(raw),
            })
            .unwrap();
            assert!(db
                .routing_history(&scope)
                .unwrap()
                .unwrap()
                .events
                .iter()
                .any(|event| matches!(
                    &event.event,
                    pytxo_store::routing::RoutingControlEvent::DecisionObserved(observed)
                        if observed.shadow_choice == Some(AdviceChoice::EverydayFit)
                )));
            assert!(observe_hosted_shadow_once_with_review(
                &catalog,
                &review.draft_id,
                &scope,
                &task_id,
                &facts,
                client,
                || Some(reviewed_packet.clone()),
            )
            .unwrap()
            .is_none());
            assert_eq!(sends.load(Ordering::SeqCst), 1);
        }
    }

    struct CountingTransport {
        sends: Arc<AtomicUsize>,
        recipient: &'static str,
        revoke_path: Option<PathBuf>,
        fail: bool,
        delay: Duration,
    }

    impl AdvisorTransport for CountingTransport {
        fn recipient_identity(&self) -> &str {
            self.recipient
        }

        fn send<'a>(&'a self, request: FixedRequest<'a>) -> TransportFuture<'a> {
            Box::pin(async move {
                let body: serde_json::Value = serde_json::from_slice(request.body()).unwrap();
                assert_eq!(
                    body["state"]["goal"],
                    "Classify reviewed repository task using coarse facts"
                );
                self.sends.fetch_add(1, Ordering::SeqCst);
                if !self.delay.is_zero() {
                    tokio::time::sleep(self.delay).await;
                }
                if let Some(path) = &self.revoke_path {
                    let second = PytxoStore::open(path).unwrap();
                    second
                        .set_routing_advisor_consent(
                            &DomainId("domain".into()),
                            1,
                            false,
                            None,
                            100,
                        )
                        .unwrap();
                }
                if self.fail {
                    return Err(UnavailableReason::Network);
                }
                Ok(br#"{"model":"jev-1.13.0","answers":{"execution_demand_v2":{"type":"choice","choice":"everyday_fit","probabilities":{"everyday_fit":0.9,"strong_needed":0.08,"unclear":0.02},"confidence":0.9}},"usage":{"input_tokens":10,"output_tokens":1}}"#.to_vec())
            })
        }
    }

    fn advisor(sends: &Arc<AtomicUsize>, revoke_path: Option<PathBuf>) -> Arc<dyn Advisor> {
        Arc::new(JevAdvisor::with_transport(CountingTransport {
            sends: Arc::clone(sends),
            recipient: crate::flow::REVIEWED_ADVISOR_RECIPIENT,
            revoke_path,
            fail: false,
            delay: Duration::ZERO,
        }))
    }

    fn journal_request_id() -> AdviceRequestId {
        AdviceRequestId(format!("jev:{}", digest("domain/run/task0/1").0))
    }

    #[test]
    fn shadow_rejects_valid_packet_not_projected_from_registered_task() {
        let (_temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let sends = Arc::new(AtomicUsize::new(0));
        let forged = AdvisorPacket::new(
            "reviewed local fixture change",
            ["single_local_change", "reviewed_check"],
            "everyday",
            "strong",
        )
        .unwrap();
        assert!(forged.is_complete());
        assert!(observe_shadow_once(
            &db,
            &mission,
            &scope,
            &TaskId("task0".into()),
            &facts,
            forged,
            advisor(&sends, None),
        )
        .unwrap()
        .is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 0);
        assert!(db
            .routing_advisor_request(&scope, &journal_request_id())
            .unwrap()
            .is_none());
    }

    #[test]
    fn shadow_rejects_stale_request_identity_before_transport_or_journal() {
        let (_temp, db, mission, scope, facts) = setup_with_request_identity(false);
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let sends = Arc::new(AtomicUsize::new(0));
        assert!(observe_shadow_once(
            &db,
            &mission,
            &scope,
            &TaskId("task0".into()),
            &facts,
            packet(&mission),
            advisor(&sends, None),
        )
        .unwrap()
        .is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 0);
        assert!(db
            .routing_advisor_request(&scope, &journal_request_id())
            .unwrap()
            .is_none());
    }

    #[test]
    fn shadow_rejects_unreviewed_recipient_before_transport_or_journal() {
        let (_temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let sends = Arc::new(AtomicUsize::new(0));
        let direct: Arc<dyn Advisor> = Arc::new(JevAdvisor::with_transport(CountingTransport {
            sends: Arc::clone(&sends),
            recipient: "https://api.typesafe.ai/v1/systemone",
            revoke_path: None,
            fail: false,
            delay: Duration::ZERO,
        }));
        assert!(observe_shadow_once(
            &db,
            &mission,
            &scope,
            &TaskId("task0".into()),
            &facts,
            packet(&mission),
            direct,
        )
        .unwrap()
        .is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 0);
        assert!(db
            .routing_advisor_request(&scope, &journal_request_id())
            .unwrap()
            .is_none());
    }

    #[test]
    fn shadow_rejects_consent_for_a_different_disclosure_scope() {
        let (_temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(Digest::of_bytes(b"other-recipient-or-template")),
            145,
        )
        .unwrap();
        let sends = Arc::new(AtomicUsize::new(0));
        assert!(observe_shadow_once(
            &db,
            &mission,
            &scope,
            &TaskId("task0".into()),
            &facts,
            packet(&mission),
            advisor(&sends, None),
        )
        .unwrap()
        .is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 0);
        assert!(db
            .routing_advisor_request(&scope, &journal_request_id())
            .unwrap()
            .is_none());
    }

    #[test]
    fn shadow_send_requires_consent_is_one_use_and_preserves_rules_target() {
        let (temp, db, mission, scope, facts) = setup();
        let task = TaskId("task0".into());
        let sends = Arc::new(AtomicUsize::new(0));
        assert!(observe_shadow_once(
            &db,
            &mission,
            &scope,
            &task,
            &facts,
            packet(&mission),
            advisor(&sends, None)
        )
        .unwrap()
        .is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 0);
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let (shadow_facts, raw) = observe_shadow_once(
            &db,
            &mission,
            &scope,
            &task,
            &facts,
            packet(&mission),
            advisor(&sends, None),
        )
        .unwrap()
        .unwrap();
        assert_eq!(sends.load(Ordering::SeqCst), 1);
        let rules = db
            .preview_routing_decision(&scope, &task, &shadow_facts, None)
            .unwrap();
        let shadow = db
            .preview_routing_decision(&scope, &task, &shadow_facts, Some(&raw))
            .unwrap();
        assert_eq!(shadow.selection, rules.selection);
        assert_eq!(shadow.advice_status, AdviceStatus::ShadowRecorded);
        let snapshot = db.routing_snapshot(&scope, &task, &shadow_facts).unwrap();
        let observed = db
            .observe_routing_decision(&ObserveRoutingDecision {
                scope: scope.clone(),
                event_id: "mock-jev-observation".into(),
                task_id: task.clone(),
                expected_cancel_epoch: snapshot.cancel_epoch,
                expected_next_ordinal: snapshot.next_ordinal,
                facts: shadow_facts,
                decision: shadow,
                advice_json: Some(raw),
            })
            .unwrap();
        assert_eq!(observed.shadow_choice, Some(AdviceChoice::EverydayFit));
        drop(db);
        let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
        assert!(observe_shadow_once(
            &reopened,
            &mission,
            &scope,
            &task,
            &facts,
            packet(&mission),
            advisor(&sends, None)
        )
        .unwrap()
        .is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn revocation_during_mock_transport_discards_shadow_reply_after_clock_rollback() {
        let (temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let task = TaskId("task0".into());
        let sends = Arc::new(AtomicUsize::new(0));
        let observed = observe_shadow_once(
            &db,
            &mission,
            &scope,
            &task,
            &facts,
            packet(&mission),
            advisor(&sends, Some(temp.path().join("store.db"))),
        )
        .unwrap();
        assert!(observed.is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 1);
        assert!(
            !db.routing_advisor_consent(&scope.domain_id)
                .unwrap()
                .enabled
        );
        assert_eq!(
            db.preview_routing_decision(&scope, &task, &facts, None)
                .unwrap()
                .selection,
            RouteSelection::Selected(mission.policy.strong)
        );
    }

    #[test]
    fn revocation_before_observation_records_only_fresh_rules() {
        let (_temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let task = TaskId("task0".into());
        let sends = Arc::new(AtomicUsize::new(0));
        let (shadow_facts, raw) = observe_shadow_once(
            &db,
            &mission,
            &scope,
            &task,
            &facts,
            packet(&mission),
            advisor(&sends, None),
        )
        .unwrap()
        .unwrap();
        let snapshot = db.routing_snapshot(&scope, &task, &shadow_facts).unwrap();
        let shadow = db
            .preview_routing_decision(&scope, &task, &shadow_facts, Some(&raw))
            .unwrap();
        db.set_routing_advisor_consent(&scope.domain_id, 1, false, None, 151)
            .unwrap();
        let observed_request = observe_with_rules_fallback(
            &db,
            ObserveRoutingDecision {
                scope: scope.clone(),
                event_id: "revoked-before-observation".into(),
                task_id: task.clone(),
                expected_cancel_epoch: snapshot.cancel_epoch,
                expected_next_ordinal: snapshot.next_ordinal,
                facts: shadow_facts,
                decision: shadow,
                advice_json: Some(raw),
            },
        )
        .unwrap();
        assert!(observed_request.advice_json.is_none());
        assert_eq!(
            observed_request.decision.advice_status,
            AdviceStatus::NotUsed
        );
        assert_eq!(
            observed_request.decision.selection,
            RouteSelection::Selected(mission.policy.strong)
        );
        let history = db.routing_history(&scope).unwrap().unwrap();
        assert!(history.events.iter().any(|event| matches!(
            &event.event,
            pytxo_store::routing::RoutingControlEvent::DecisionObserved(observed)
                if observed.shadow_choice.is_none()
        )));
    }

    #[test]
    fn unavailable_transport_records_uncertain_usage_without_retry() {
        let (_temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let sends = Arc::new(AtomicUsize::new(0));
        let advisor: Arc<dyn Advisor> = Arc::new(JevAdvisor::with_transport(CountingTransport {
            sends: Arc::clone(&sends),
            recipient: crate::flow::REVIEWED_ADVISOR_RECIPIENT,
            revoke_path: None,
            fail: true,
            delay: Duration::ZERO,
        }));
        for _ in 0..2 {
            assert!(observe_shadow_once(
                &db,
                &mission,
                &scope,
                &TaskId("task0".into()),
                &facts,
                packet(&mission),
                Arc::clone(&advisor),
            )
            .unwrap()
            .is_none());
        }
        assert_eq!(sends.load(Ordering::SeqCst), 1);
        assert_eq!(
            db.routing_advisor_request(&scope, &journal_request_id())
                .unwrap()
                .unwrap()
                .phase,
            AdvisorSendPhase::Uncertain
        );
    }

    #[test]
    fn concurrent_store_connections_make_only_one_mock_send() {
        let (temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let sends = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let handles = (0..2)
            .map(|_| {
                let path = temp.path().join("store.db");
                let mission = mission.clone();
                let scope = scope.clone();
                let facts = facts.clone();
                let sends = Arc::clone(&sends);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    let connection = PytxoStore::open(&path).unwrap();
                    barrier.wait();
                    observe_shadow_once(
                        &connection,
                        &mission,
                        &scope,
                        &TaskId("task0".into()),
                        &facts,
                        packet(&mission),
                        advisor(&sends, None),
                    )
                    .unwrap()
                    .is_some()
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        assert_eq!(
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .filter(|sent| *sent)
                .count(),
            1
        );
        assert_eq!(sends.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn deadline_falls_back_and_does_not_retry_after_reopen() {
        let (temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let sends = Arc::new(AtomicUsize::new(0));
        let advisor: Arc<dyn Advisor> = Arc::new(JevAdvisor::with_transport(CountingTransport {
            sends: Arc::clone(&sends),
            recipient: crate::flow::REVIEWED_ADVISOR_RECIPIENT,
            revoke_path: None,
            fail: false,
            delay: Duration::from_secs(3),
        }));
        assert!(observe_shadow_once(
            &db,
            &mission,
            &scope,
            &TaskId("task0".into()),
            &facts,
            packet(&mission),
            Arc::clone(&advisor),
        )
        .unwrap()
        .is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 1);
        drop(db);
        let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
        assert_eq!(
            reopened
                .routing_advisor_request(&scope, &journal_request_id())
                .unwrap()
                .unwrap()
                .phase,
            AdvisorSendPhase::Uncertain
        );
        assert!(observe_shadow_once(
            &reopened,
            &mission,
            &scope,
            &TaskId("task0".into()),
            &facts,
            packet(&mission),
            advisor,
        )
        .unwrap()
        .is_none());
        assert_eq!(sends.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn delayed_worker_start_cannot_send_after_caller_deadline() {
        let (_temp, db, mission, scope, facts) = setup();
        db.set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(crate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
            145,
        )
        .unwrap();
        let sends = Arc::new(AtomicUsize::new(0));
        let result = observe_shadow_once_with_start_delay(
            &db,
            &mission,
            &scope,
            &TaskId("task0".into()),
            &facts,
            packet(&mission),
            advisor(&sends, None),
            ADVISOR_DEADLINE + Duration::from_millis(100),
        )
        .unwrap();
        assert!(result.is_none());
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(sends.load(Ordering::SeqCst), 0);
        assert_eq!(
            db.routing_advisor_request(&scope, &journal_request_id())
                .unwrap()
                .unwrap()
                .phase,
            AdvisorSendPhase::Uncertain
        );
    }
}
