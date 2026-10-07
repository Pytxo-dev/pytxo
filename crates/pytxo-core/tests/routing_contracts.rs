use pytxo_core::routing::*;
use pytxo_core::{DomainId, ExecutionBackend, PermissionProfile, RunId, TaskId};
use std::collections::BTreeSet;

type Mutation<T> = Box<dyn Fn(&mut T)>;

fn digest(label: &str) -> Digest {
    Digest::of_bytes(label.as_bytes())
}
fn set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|v| (*v).into()).collect()
}

fn candidate(name: &str) -> ProfileCandidate {
    let profile = ExecutionProfile {
        schema_version: 1,
        canonicalization_version: 1,
        id: ProfileId(name.into()),
        revision: 1,
        harness_id: name.into(),
        adapter_contract_version: "1".into(),
        adapter_digest: digest("adapter"),
        requested_model: ModelIdentity {
            provider: "vendor".into(),
            model: name.into(),
            reasoning: Some("medium".into()),
            revision: None,
        },
        skill_tool_bundle_digest: digest("tools"),
        backend: ExecutionBackend::Pty,
        capabilities: set(&["edit", "check"]),
    };
    let binding = ProfileBinding {
        schema_version: 1,
        canonicalization_version: 1,
        id: BindingId(name.into()),
        revision: 1,
        profile_digest: profile.digest().unwrap(),
        credential_reference: Some("keychain:worker".into()),
        auth_owner: "local-user".into(),
        billing_source_id: BillingSourceId("subscription".into()),
        billing_mode: BillingSourceMode::Subscription,
        endpoint_identity: "vendor-cli".into(),
        trust_class: "vendor".into(),
        capacity_pool_ids: set(&["host-worker", "account"]),
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
        arguments_digest: digest("argv-template"),
        environment_policy_digest: digest("environment-policy"),
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
    let observation = ProfileObservation {
        schema_version: 1,
        profile_digest: profile.digest().unwrap(),
        binding_digest: binding.digest().unwrap(),
        executable: ExecutableIdentity {
            path: format!("C:/tools/{name}.exe"),
            version: "1.0".into(),
            digest: digest("exe"),
        },
        launch: Some(launch.clone()),
        observed_at_ms: 100,
        expires_at_ms: 200,
        auth_status: Readiness::Ready,
        dispatch_supported: true,
        qualification: Some(AdapterQualification {
            receipt_digest: digest("qualified"),
            launch_fingerprint: qualification_fingerprint(
                &profile,
                &binding,
                &ExecutableIdentity {
                    path: format!("C:/tools/{name}.exe"),
                    version: "1.0".into(),
                    digest: digest("exe"),
                },
                &launch,
            )
            .unwrap(),
            permission_profile: PermissionProfile::Orbit,
            tool_probe_passed: true,
            cancellation_probe_passed: true,
            quiescence_probe_passed: true,
            capabilities: set(&["edit", "check"]),
            allowed_egress: BTreeSet::new(),
            capacity_pool_ids: set(&["host-worker", "account"]),
        }),
        requested_model: profile.requested_model.clone(),
        reported_model: Some(profile.requested_model.clone()),
        model_identity_level: ModelIdentityLevel::HarnessReported,
        metering_support: MeteringSupport::Unknown,
        hard_spend_limit_verified: false,
        capacity_ready: true,
    };
    ProfileCandidate {
        profile,
        binding,
        observation,
    }
}

fn fixture() -> (RoutingSnapshot, Vec<ProfileCandidate>, RoutingPolicy) {
    let candidates = vec![candidate("everyday"), candidate("strong")];
    let policy = RoutingPolicy {
        schema_version: 1,
        version: "rules-1".into(),
        mode: RoutingMode::Rules,
        everyday: candidates[0].target(),
        strong: candidates[1].target(),
        evaluated_manifest_digest: None,
        everyday_threshold_ppm: 800_000,
        unclear_ceiling_ppm: 100_000,
        advice_model: "pinned".into(),
        advice_template: "rubric-1".into(),
        disclosure_scope_digest: None,
        advisor_recipient: None,
    };
    let task = TaskContract {
        schema_version: 1,
        canonicalization_version: 1,
        task_id: TaskId("task".into()),
        revision: 1,
        plan_digest: digest("plan"),
        base: BaseSnapshot {
            repository_identity: "repo".into(),
            git_revision: "abc".into(),
            snapshot_digest: digest("base"),
        },
        goal: "Format one file".into(),
        constraints: vec!["Keep behavior".into()],
        claim_roots: vec!["src".into()],
        dependencies: vec![],
        task_kind: Some(TaskKind::Formatting),
        task_kind_evidence: Some(digest("reviewed-kind")),
        required_capabilities: set(&["edit", "check"]),
        checks: vec![CheckRecipe {
            id: CheckId("check".into()),
            recipe_digest: digest("check"),
        }],
        required_resources: set(&["host-worker"]),
        skill_tool_bundle_digest: digest("tools"),
        permission_profile: PermissionProfile::Orbit,
        required_egress: BTreeSet::new(),
        required_target: None,
        strong_only: false,
        cross_component_requirement: Some(false),
        context_complete: true,
        repeatable_symptom_supplied: None,
        specific_cause_hypothesis_supplied: None,
    };
    let authorization = MissionAuthorization {
        schema_version: 1,
        domain_id: DomainId("domain".into()),
        run_id: RunId("run".into()),
        plan_id: PlanId("plan".into()),
        plan_digest: task.plan_digest.clone(),
        revision: 1,
        cancel_epoch: 0,
        allowed_task_digests: BTreeSet::from([task.digest().unwrap()]),
        allowed_profiles: candidates
            .iter()
            .map(|c| ApprovedProfile {
                target: c.target(),
                profile_digest: c.profile.digest().unwrap(),
                binding_digest: c.binding.digest().unwrap(),
            })
            .collect(),
        allowed_billing_sources: BTreeSet::from([BillingSourceId("subscription".into())]),
        allowed_billing_modes: BTreeSet::from([BillingSourceMode::Subscription]),
        permission_profile: PermissionProfile::Orbit,
        allowed_egress: BTreeSet::new(),
        minimum_model_identity: ModelIdentityLevel::HarnessReported,
        limits: MissionLimits {
            deadline_ms: 1000,
            max_workers: 1,
            max_attempts: 2,
            max_spend_nano_usd: Some(100),
            spend_guarantee: SpendGuarantee::RiskBounded,
        },
        policy_digest: policy.digest().unwrap(),
        live_advice_authorized: false,
        consent_revision: 1,
    };
    let snapshot = RoutingSnapshot {
        domain_id: authorization.domain_id.clone(),
        run_id: authorization.run_id.clone(),
        plan_id: authorization.plan_id.clone(),
        task,
        authorization,
        task_revision: 1,
        task_state_revision: 1,
        authorization_revision: 1,
        cancel_epoch: 0,
        next_ordinal: 1,
        previous_attempt: None,
        resolved_dependencies: vec![],
        qualification_digests: candidates
            .iter()
            .map(|c| {
                c.observation
                    .qualification
                    .as_ref()
                    .unwrap()
                    .digest()
                    .unwrap()
            })
            .collect(),
        now_ms: 150,
        cancelled: false,
        scope_valid: true,
        dependencies_ready: true,
        ownership_resolved: true,
        active_workers: 0,
        admitted_attempts: 0,
        spent_and_reserved_nano_usd: Some(0),
        manual_target: None,
        packet_digest: Some(digest("packet")),
        advice_request_id: Some(AdviceRequestId("request".into())),
    };
    (snapshot, candidates, policy)
}

#[test]
fn optional_diagnosis_cues_preserve_old_task_bytes_and_bind_reviewed_values() {
    let task = fixture().0.task;
    let old_digest = task.digest().unwrap();
    let serialized = serde_json::to_value(&task).unwrap();
    assert!(serialized.get("repeatable_symptom_supplied").is_none());
    assert!(serialized
        .get("specific_cause_hypothesis_supplied")
        .is_none());
    let decoded: TaskContract = serde_json::from_value(serialized).unwrap();
    assert_eq!(decoded, task);
    assert_eq!(decoded.digest().unwrap(), old_digest);

    let mut reviewed = decoded;
    reviewed.task_kind = Some(TaskKind::Diagnosis);
    reviewed.repeatable_symptom_supplied = Some(true);
    reviewed.specific_cause_hypothesis_supplied = Some(false);
    let changed = serde_json::to_value(&reviewed).unwrap();
    assert_eq!(changed["repeatable_symptom_supplied"], true);
    assert_eq!(changed["specific_cause_hypothesis_supplied"], false);
    assert_ne!(reviewed.digest().unwrap(), old_digest);
    assert_eq!(
        serde_json::from_value::<TaskContract>(changed).unwrap(),
        reviewed
    );
}

fn catalog(s: &RoutingSnapshot, c: &[ProfileCandidate]) -> EligibleCatalog {
    build_eligible_catalog(s, c)
}
fn refresh_task(s: &mut RoutingSnapshot) {
    s.authorization.allowed_task_digests = BTreeSet::from([s.task.digest().unwrap()]);
}
fn refresh_policy(s: &mut RoutingSnapshot, p: &RoutingPolicy) {
    s.authorization.policy_digest = p.digest().unwrap();
}

#[test]
fn disclosure_scope_is_optional_for_old_rules_policies_but_binds_new_review() {
    let (_, _, mut policy) = fixture();
    let legacy_digest = policy.digest().unwrap();
    let legacy_wire = serde_json::to_value(&policy).unwrap();
    assert!(legacy_wire.get("disclosure_scope_digest").is_none());
    assert!(legacy_wire.get("advisor_recipient").is_none());
    assert_eq!(
        serde_json::from_value::<RoutingPolicy>(legacy_wire)
            .unwrap()
            .digest()
            .unwrap(),
        legacy_digest
    );

    let scope = digest("reviewed-recipient-template-v1");
    policy.disclosure_scope_digest = Some(scope.clone());
    assert_ne!(policy.digest().unwrap(), legacy_digest);
    let scoped_wire = serde_json::to_value(&policy).unwrap();
    assert_eq!(scoped_wire["disclosure_scope_digest"], scope.0);
    assert_eq!(
        serde_json::from_value::<RoutingPolicy>(scoped_wire).unwrap(),
        policy
    );

    let scoped_digest = policy.digest().unwrap();
    policy.advisor_recipient = Some("pytxo-hosted-routing/typesafe-systemone/v1".into());
    assert_ne!(policy.digest().unwrap(), scoped_digest);
    let hosted_wire = serde_json::to_value(&policy).unwrap();
    assert_eq!(
        hosted_wire["advisor_recipient"],
        "pytxo-hosted-routing/typesafe-systemone/v1"
    );
    assert_eq!(
        serde_json::from_value::<RoutingPolicy>(hosted_wire).unwrap(),
        policy
    );
}

#[test]
fn positive_route_uses_qualified_complete_tuple_and_rules_strong_default() {
    let (mut s, c, p) = fixture();
    let cat = catalog(&s, &c);
    assert_eq!(cat.eligible.len(), 2);
    assert_eq!(
        select_route(&s, &cat, &p, None).selection,
        RouteSelection::Selected(c[0].target())
    );
    s.task.task_kind = None;
    refresh_task(&mut s);
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Selected(c[1].target())
    );
}

#[test]
fn hosted_capability_qualification_is_stable_but_attempt_launch_stays_prompt_exact() {
    let (snapshot, mut candidates, _) = fixture();
    let selected = &candidates[0];
    let target = selected.target();
    let probe_launch = selected.observation.launch.clone().unwrap();
    let mut attempt_launch = probe_launch.clone();
    attempt_launch.private_stdin_digest = Some(digest("different private task prompt"));
    let qualified = selected
        .observation
        .qualification
        .as_ref()
        .unwrap()
        .launch_fingerprint
        .clone();
    assert_eq!(
        qualified,
        qualification_fingerprint(
            &selected.profile,
            &selected.binding,
            &selected.observation.executable,
            &attempt_launch,
        )
        .unwrap()
    );
    assert_ne!(
        launch_fingerprint(
            &selected.profile,
            &selected.binding,
            &selected.observation.executable,
            &probe_launch,
        )
        .unwrap(),
        launch_fingerprint(
            &selected.profile,
            &selected.binding,
            &selected.observation.executable,
            &attempt_launch,
        )
        .unwrap()
    );
    candidates[0].observation.launch = Some(attempt_launch.clone());
    assert_eq!(catalog(&snapshot, &candidates).eligible.len(), 2);

    // Only the private payload may vary. A probe for different argv or a
    // missing attempt-specific payload still fails eligibility.
    attempt_launch.arguments_digest = digest("unqualified argv");
    candidates[0].observation.launch = Some(attempt_launch);
    assert!(catalog(&snapshot, &candidates)
        .exclusions
        .iter()
        .any(|row| {
            row.target == target && row.reasons.contains(&ExclusionReason::FingerprintMismatch)
        }));
    candidates[0]
        .observation
        .launch
        .as_mut()
        .unwrap()
        .private_stdin_digest = None;
    assert!(catalog(&snapshot, &candidates)
        .exclusions
        .iter()
        .any(|row| {
            row.target == target && row.reasons.contains(&ExclusionReason::InvalidContract)
        }));
}

#[test]
fn routing_v1_rejects_ambiguous_roles_and_a_third_manual_or_required_target() {
    let (mut snapshot, mut candidates, mut policy) = fixture();
    policy.strong = policy.everyday.clone();
    refresh_policy(&mut snapshot, &policy);
    assert_eq!(
        select_route(&snapshot, &catalog(&snapshot, &candidates), &policy, None).selection,
        RouteSelection::Blocked(RouteBlocker::InvalidPolicy)
    );

    let (_, _, mut policy) = fixture();
    policy.strong.profile_id = policy.everyday.profile_id.clone();
    refresh_policy(&mut snapshot, &policy);
    assert_eq!(
        select_route(&snapshot, &catalog(&snapshot, &candidates), &policy, None).selection,
        RouteSelection::Blocked(RouteBlocker::InvalidPolicy)
    );

    let (_, _, policy) = fixture();
    refresh_policy(&mut snapshot, &policy);
    let third = candidate("third");
    snapshot
        .authorization
        .allowed_profiles
        .push(ApprovedProfile {
            target: third.target(),
            profile_digest: third.profile.digest().unwrap(),
            binding_digest: third.binding.digest().unwrap(),
        });
    snapshot.qualification_digests.insert(
        third
            .observation
            .qualification
            .as_ref()
            .unwrap()
            .digest()
            .unwrap(),
    );
    candidates.push(third);
    let third_target = candidates[2].target();
    snapshot.manual_target = Some(third_target.clone());
    assert_eq!(catalog(&snapshot, &candidates).eligible.len(), 3);
    assert_eq!(
        select_route(&snapshot, &catalog(&snapshot, &candidates), &policy, None).selection,
        RouteSelection::Blocked(RouteBlocker::ConflictingRequirement)
    );
    snapshot.manual_target = None;
    snapshot.task.required_target = Some(third_target);
    refresh_task(&mut snapshot);
    assert_eq!(
        select_route(&snapshot, &catalog(&snapshot, &candidates), &policy, None).selection,
        RouteSelection::Blocked(RouteBlocker::ConflictingRequirement)
    );
}

#[test]
fn disabled_is_default_and_manual_or_required_routes_never_substitute() {
    let (mut s, mut c, mut p) = fixture();
    assert_eq!(RoutingMode::default(), RoutingMode::Disabled);
    p.mode = RoutingMode::Disabled;
    refresh_policy(&mut s, &p);
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(RouteBlocker::Disabled)
    );
    p.mode = RoutingMode::Rules;
    refresh_policy(&mut s, &p);
    s.manual_target = Some(c[0].target());
    c[0].observation.auth_status = Readiness::Unavailable;
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(RouteBlocker::PreferredUnavailable)
    );
    s.manual_target = None;
    s.task.required_target = Some(c[0].target());
    refresh_task(&mut s);
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(RouteBlocker::PreferredUnavailable)
    );
}

#[test]
fn each_missing_tuple_fact_has_a_typed_exclusion() {
    let (s, c, _) = fixture();
    let cases: Vec<(ExclusionReason, Mutation<ProfileCandidate>)> = vec![
        (
            ExclusionReason::UnsupportedHarness,
            Box::new(|c| c.observation.dispatch_supported = false),
        ),
        (
            ExclusionReason::StaleObservation,
            Box::new(|c| c.observation.expires_at_ms = 150),
        ),
        (
            ExclusionReason::UnqualifiedAdapter,
            Box::new(|c| c.observation.qualification = None),
        ),
        (
            ExclusionReason::ModelMismatch,
            Box::new(|c| c.observation.reported_model.as_mut().unwrap().model = "other".into()),
        ),
        (
            ExclusionReason::InsufficientModelEvidence,
            Box::new(|c| c.observation.model_identity_level = ModelIdentityLevel::Requested),
        ),
        (
            ExclusionReason::BindingNotReady,
            Box::new(|c| c.observation.auth_status = Readiness::Unknown),
        ),
        (
            ExclusionReason::FingerprintMismatch,
            Box::new(|c| c.observation.executable.digest = digest("changed")),
        ),
        (
            ExclusionReason::CapabilityMismatch,
            Box::new(|c| {
                c.profile.capabilities.remove("check");
            }),
        ),
        (
            ExclusionReason::PermissionMismatch,
            Box::new(|c| {
                c.observation
                    .qualification
                    .as_mut()
                    .unwrap()
                    .permission_profile = PermissionProfile::Supernova
            }),
        ),
        (
            ExclusionReason::CapacityUnavailable,
            Box::new(|c| c.observation.capacity_ready = false),
        ),
    ];
    for (reason, change) in cases {
        let mut one = c[0].clone();
        change(&mut one);
        let cat = catalog(&s, &[one]);
        assert!(cat.eligible.is_empty(), "{reason:?}");
        assert!(
            cat.exclusions[0].reasons.contains(&reason),
            "{reason:?}: {:?}",
            cat.exclusions
        );
    }
}

#[test]
fn credentials_and_changed_recipes_do_not_supply_approval() {
    let (mut s, mut c, _) = fixture();
    c[0].binding.auth_owner = "another-account".into();
    assert!(catalog(&s, &c).exclusions[0]
        .reasons
        .contains(&ExclusionReason::UnapprovedBinding));
    c[1].profile.skill_tool_bundle_digest = digest("changed-tools");
    assert!(catalog(&s, &c).eligible.is_empty());
    s.authorization.allowed_profiles.clear();
    assert!(catalog(&s, &c).eligible.is_empty());
}

#[test]
fn qualified_launch_requires_host_transport_and_unchanged_shape() {
    let (snapshot, candidates, _) = fixture();
    for change in [
        "missing",
        "different_host",
        "different_argv",
        "different_environment",
        "different_timeout",
        "different_lowering",
        "missing_private_stdin_digest",
        "direct_private_stdin",
        "wrong_protocol",
    ] {
        let mut candidate = candidates[0].clone();
        match change {
            "missing" => candidate.observation.launch = None,
            "different_host" => {
                candidate
                    .observation
                    .launch
                    .as_mut()
                    .unwrap()
                    .host
                    .as_mut()
                    .unwrap()
                    .digest = digest("different-host");
            }
            "different_argv" => {
                candidate
                    .observation
                    .launch
                    .as_mut()
                    .unwrap()
                    .arguments_digest = digest("different-argv");
            }
            "different_environment" => {
                candidate
                    .observation
                    .launch
                    .as_mut()
                    .unwrap()
                    .environment_policy_digest = digest("different-environment");
            }
            "different_timeout" => {
                candidate
                    .observation
                    .launch
                    .as_mut()
                    .unwrap()
                    .execution_timeout_ms += 1;
            }
            "different_lowering" => {
                candidate
                    .observation
                    .launch
                    .as_mut()
                    .unwrap()
                    .argument_lowering = "unqualified-lowering".into();
            }
            "missing_private_stdin_digest" => {
                candidate
                    .observation
                    .launch
                    .as_mut()
                    .unwrap()
                    .private_stdin_digest = None;
            }
            "direct_private_stdin" => {
                let launch = candidate.observation.launch.as_mut().unwrap();
                launch.transport = LaunchTransport::DirectSubprocess;
                launch.host = None;
            }
            "wrong_protocol" => {
                candidate
                    .observation
                    .launch
                    .as_mut()
                    .unwrap()
                    .output_protocol = "unqualified-protocol".into();
            }
            _ => unreachable!(),
        }
        let catalog = catalog(&snapshot, &[candidate]);
        assert!(catalog.eligible.is_empty(), "{change}");
        assert!(catalog.exclusions[0].reasons.iter().any(|reason| {
            matches!(
                reason,
                ExclusionReason::InvalidContract | ExclusionReason::FingerprintMismatch
            )
        }));
    }
}

#[test]
fn cancellation_scope_prerequisites_ownership_and_budgets_prevent_admission() {
    let (s, c, p) = fixture();
    let cases: Vec<Mutation<RoutingSnapshot>> = vec![
        Box::new(|s| s.cancelled = true),
        Box::new(|s| s.cancel_epoch += 1),
        Box::new(|s| s.scope_valid = false),
        Box::new(|s| s.dependencies_ready = false),
        Box::new(|s| s.ownership_resolved = false),
        Box::new(|s| s.admitted_attempts = 2),
        Box::new(|s| s.now_ms = 1000),
        Box::new(|s| s.spent_and_reserved_nano_usd = Some(100)),
        Box::new(|s| s.task.goal = "changed scope".into()),
    ];
    for change in cases {
        let mut s = s.clone();
        change(&mut s);
        let cat = catalog(&s, &c);
        assert!(cat.eligible.is_empty());
        assert!(matches!(
            select_route(&s, &cat, &p, None).selection,
            RouteSelection::Blocked(_)
        ));
    }
}

#[test]
fn billing_and_hard_spend_guarantees_are_explicit() {
    let (mut s, c, _) = fixture();
    s.authorization.allowed_billing_modes = BTreeSet::from([BillingSourceMode::Api]);
    assert!(catalog(&s, &c).exclusions[0]
        .reasons
        .contains(&ExclusionReason::BillingMismatch));
    s.authorization.allowed_billing_modes = BTreeSet::from([BillingSourceMode::Subscription]);
    s.authorization.limits.spend_guarantee = SpendGuarantee::Hard;
    assert!(catalog(&s, &c).exclusions[0]
        .reasons
        .contains(&ExclusionReason::SpendGuaranteeUnavailable));
}

fn advice(s: &RoutingSnapshot) -> AdviceEnvelope {
    AdviceEnvelope {
        schema_version: 1,
        request_id: s.advice_request_id.clone().unwrap(),
        packet_digest: s.packet_digest.clone().unwrap(),
        policy_version: "rules-1".into(),
        template_version: "rubric-1".into(),
        model_id: "pinned".into(),
        task_revision: s.task_revision,
        task_state_revision: s.task_state_revision,
        authorization_revision: s.authorization_revision,
        cancel_epoch: s.cancel_epoch,
        consent_revision: s.authorization.consent_revision,
        received_at_ms: 140,
        expires_at_ms: 180,
        packet_complete: true,
        choice: AdviceChoice::EverydayFit,
        distribution: AdviceDistribution {
            everyday_fit: 0.9,
            strong_needed: 0.08,
            unclear: 0.02,
        },
        usage_status: AdviceUsageStatus::Unknown,
        usage_receipt_id: None,
        elapsed_ms: 40,
    }
}

#[test]
fn shadow_returns_exact_rules_decision_while_recording_valid_advice() {
    let (mut s, c, mut p) = fixture();
    s.task.task_kind = Some(TaskKind::Other);
    refresh_task(&mut s);
    let a = advice(&s);
    let rules = select_route(&s, &catalog(&s, &c), &p, None);
    p.mode = RoutingMode::Shadow;
    refresh_policy(&mut s, &p);
    let shadow = select_route(&s, &catalog(&s, &c), &p, Some(&a));
    assert_eq!(shadow.selection, rules.selection);
    assert_eq!(shadow.reason, rules.reason);
    assert_eq!(shadow.advice_status, AdviceStatus::ShadowRecorded);
}

#[test]
fn live_advice_only_changes_initial_ambiguous_case_with_evaluated_authorized_policy() {
    let (mut s, c, mut p) = fixture();
    s.task.task_kind = Some(TaskKind::Diagnosis);
    refresh_task(&mut s);
    p.mode = RoutingMode::Live;
    refresh_policy(&mut s, &p);
    let a = advice(&s);
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).selection,
        RouteSelection::Selected(c[1].target())
    );
    p.evaluated_manifest_digest = Some(digest("evaluated"));
    s.authorization.live_advice_authorized = true;
    refresh_policy(&mut s, &p);
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).selection,
        RouteSelection::Selected(c[0].target())
    );
    s.task.strong_only = true;
    refresh_task(&mut s);
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).selection,
        RouteSelection::Selected(c[1].target())
    );
    s.task.strong_only = false;
    refresh_task(&mut s);
    s.next_ordinal = 2;
    assert_ne!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).reason,
        RouteReason::AdviceEveryday
    );
}

#[test]
fn live_advice_cannot_downgrade_cross_component_or_unknown_scope() {
    let (mut snapshot, profiles, mut policy) = fixture();
    snapshot.task.task_kind = Some(TaskKind::Diagnosis);
    policy.mode = RoutingMode::Live;
    policy.evaluated_manifest_digest = Some(digest("evaluated"));
    snapshot.authorization.live_advice_authorized = true;
    refresh_policy(&mut snapshot, &policy);
    for scope in [None, Some(true)] {
        snapshot.task.cross_component_requirement = scope;
        refresh_task(&mut snapshot);
        let advisor = advice(&snapshot);
        let decision = select_route(
            &snapshot,
            &catalog(&snapshot, &profiles),
            &policy,
            Some(&advisor),
        );
        assert_eq!(decision.reason, RouteReason::StrongDefault);
        assert_eq!(
            decision.selection,
            RouteSelection::Selected(profiles[1].target())
        );
    }
    snapshot.task.cross_component_requirement = Some(false);
    refresh_task(&mut snapshot);
    let advisor = advice(&snapshot);
    let decision = select_route(
        &snapshot,
        &catalog(&snapshot, &profiles),
        &policy,
        Some(&advisor),
    );
    assert_eq!(decision.reason, RouteReason::AdviceEveryday);

    for kind in [TaskKind::Other, TaskKind::Architecture] {
        snapshot.task.task_kind = Some(kind);
        refresh_task(&mut snapshot);
        let advisor = advice(&snapshot);
        let decision = select_route(
            &snapshot,
            &catalog(&snapshot, &profiles),
            &policy,
            Some(&advisor),
        );
        assert_eq!(decision.reason, RouteReason::StrongDefault);
    }
}

#[test]
fn undeclared_execution_demand_defaults_to_the_strong_profile() {
    let (mut snapshot, profiles, policy) = fixture();
    snapshot.task.task_kind = Some(TaskKind::Other);
    snapshot.task.context_complete = false;
    snapshot.task.cross_component_requirement = None;
    refresh_task(&mut snapshot);
    let decision = select_route(&snapshot, &catalog(&snapshot, &profiles), &policy, None);
    assert_eq!(decision.reason, RouteReason::StrongDefault);
    assert_eq!(
        decision.selection,
        RouteSelection::Selected(profiles[1].target())
    );
}

#[test]
fn malformed_stale_unknown_or_wrong_context_advice_falls_back() {
    let (mut s, c, mut p) = fixture();
    s.task.task_kind = Some(TaskKind::Other);
    refresh_task(&mut s);
    p.mode = RoutingMode::Live;
    p.evaluated_manifest_digest = Some(digest("evaluated"));
    s.authorization.live_advice_authorized = true;
    refresh_policy(&mut s, &p);
    let cases: Vec<Mutation<AdviceEnvelope>> = vec![
        Box::new(|a| a.model_id = "wrong".into()),
        Box::new(|a| a.expires_at_ms = 150),
        Box::new(|a| a.packet_complete = false),
        Box::new(|a| a.packet_digest = digest("other")),
        Box::new(|a| a.authorization_revision += 1),
        Box::new(|a| a.distribution.everyday_fit = f64::NAN),
        Box::new(|a| a.distribution.unclear = -0.1),
        Box::new(|a| a.distribution.strong_needed = 0.5),
        Box::new(|a| a.choice = AdviceChoice::StrongNeeded),
    ];
    for change in cases {
        let mut a = advice(&s);
        change(&mut a);
        let d = select_route(&s, &catalog(&s, &c), &p, Some(&a));
        assert_eq!(d.selection, RouteSelection::Selected(c[1].target()));
        assert_eq!(d.advice_status, AdviceStatus::InvalidOrStale);
    }
    let mut wire = serde_json::to_value(advice(&s)).unwrap();
    wire["unexpected"] = true.into();
    assert!(serde_json::from_value::<AdviceEnvelope>(wire).is_err());
}

#[test]
fn lifecycle_requires_proof_and_never_relaunches_uncertain_or_terminal_attempts() {
    let empty = AttemptTransitionEvidence::default();
    let mut state = AttemptState::Admitted;
    assert!(state.transition(AttemptState::Preparing, &empty).is_err());
    let proven = AttemptTransitionEvidence {
        inputs_bound: true,
        launch_checks_passed: true,
        process_registered: true,
        no_worker_created: false,
        quiescent: true,
        output_sealed: true,
        checks_passed: true,
        authority_current: true,
        reconciliation_complete: true,
    };
    for next in [
        AttemptState::Preparing,
        AttemptState::Launching,
        AttemptState::Running,
        AttemptState::Sealing,
        AttemptState::Verifying,
        AttemptState::Passed,
    ] {
        state.transition(next, &proven).unwrap();
    }
    for next in [
        AttemptState::Failed,
        AttemptState::Cancelled,
        AttemptState::Launching,
    ] {
        assert!(state.transition(next, &proven).is_err());
    }
    let mut launching = AttemptState::Launching;
    assert!(launching
        .transition(AttemptState::FailedNoLaunch, &empty)
        .is_err());
    launching
        .transition(AttemptState::RecoveryRequired, &empty)
        .unwrap();
    assert!(launching
        .transition(AttemptState::Launching, &proven)
        .is_err());
    assert!(launching.transition(AttemptState::Failed, &empty).is_err());
    launching.transition(AttemptState::Failed, &proven).unwrap();
    assert_eq!(
        AttemptState::parse("failed_no_launch"),
        Some(AttemptState::FailedNoLaunch)
    );
}

#[test]
fn canonical_hashes_sort_keys_reject_floats_and_bind_all_recipe_settings() {
    assert_eq!(
        canonical_digest(&serde_json::json!({"z": null,"a": [2,1]}), 1).unwrap(),
        canonical_digest(&serde_json::json!({"a": [2,1],"z": null}), 1).unwrap()
    );
    assert!(canonical_digest(&serde_json::json!({"a": 1.0}), 1).is_err());
    assert!(canonical_digest(&serde_json::json!({"a": 1}), 2).is_err());
    let c = candidate("everyday");
    let mut changed = c.profile.clone();
    changed.requested_model.reasoning = Some("high".into());
    assert_ne!(c.profile.digest().unwrap(), changed.digest().unwrap());
    assert_eq!(
        serde_json::from_str::<ExecutionProfile>(&serde_json::to_string(&c.profile).unwrap())
            .unwrap(),
        c.profile
    );
}

fn handoff(s: &RoutingSnapshot) -> PortableHandoffManifest {
    PortableHandoffManifest {
        schema_version: 1,
        canonicalization_version: 1,
        manifest_digest: digest("placeholder"),
        origin: HandoffOrigin {
            domain_id: s.domain_id.clone(),
            run_id: s.run_id.clone(),
            task_id: s.task.task_id.clone(),
            attempt_id: AttemptId("attempt-1".into()),
            task_revision: s.task.revision,
            plan_id: s.plan_id.clone(),
            plan_digest: s.task.plan_digest.clone(),
            authorization_revision: s.authorization.revision,
        },
        task_contract_ref: s.task.digest().unwrap(),
        base: s.task.base.clone(),
        dependencies: s.resolved_dependencies.clone(),
        repair_input: Some(UntrustedRepairInput {
            failed_attempt_id: AttemptId("attempt-0".into()),
            task_id: s.task.task_id.clone(),
            scoped_change_manifest: BlobRef {
                digest: digest("repair"),
                byte_length: 6,
            },
            trust: RepairInputTrust::Untrusted,
        }),
        changes: vec![HandoffChange {
            path: "src/file.rs".into(),
            operation: ChangeOperation::Modify,
            preimage_digest: Some(digest("old")),
            result: Some(BlobRef {
                digest: digest("new"),
                byte_length: 3,
            }),
            executable: false,
        }],
        evidence: vec![HandoffEvidence {
            kind: EvidenceKind::Quiescence,
            provenance: EvidenceProvenance::Observed,
            receipt: BlobRef {
                digest: digest("proof"),
                byte_length: 5,
            },
            check_id: None,
        }],
        requirements: HandoffRequirements {
            capabilities: s.task.required_capabilities.clone(),
            skill_tool_bundle_digest: s.task.skill_tool_bundle_digest.clone(),
        },
        notes: vec![ClaimedWorkerNote {
            text: "The check failed.".into(),
            provenance: NoteProvenance::Claimed,
        }],
    }
    .seal()
    .unwrap()
}

#[test]
fn handoff_has_verified_dependencies_separate_from_untrusted_repair_and_checks_blob_integrity() {
    let (s, _, _) = fixture();
    let h = handoff(&s);
    h.validate(&s.task, &h.origin, &s.resolved_dependencies)
        .unwrap();
    let blob = BlobRef {
        digest: digest("new"),
        byte_length: 3,
    };
    blob.verify(b"new").unwrap();
    assert!(blob.verify(b"bad").is_err());
    assert!(blob.verify(b"new!").is_err());
    let mut corrupt = h.clone();
    corrupt.notes[0].text = "tampered".into();
    assert!(corrupt
        .validate(&s.task, &h.origin, &s.resolved_dependencies)
        .is_err());
    let mut other_task = h.clone();
    other_task.repair_input.as_mut().unwrap().task_id = TaskId("other".into());
    other_task = other_task.seal().unwrap();
    assert!(other_task
        .validate(&s.task, &h.origin, &s.resolved_dependencies)
        .is_err());
    let mut self_repair = h.clone();
    self_repair.repair_input.as_mut().unwrap().failed_attempt_id = h.origin.attempt_id.clone();
    self_repair = self_repair.seal().unwrap();
    assert!(self_repair
        .validate(&s.task, &h.origin, &s.resolved_dependencies)
        .is_err());
    let mut fake_dependency = h.clone();
    fake_dependency.dependencies.push(VerifiedDependencyOutput {
        task_id: TaskId("new-dependency".into()),
        winning_attempt_id: AttemptId("fake".into()),
        output_digest: digest("output"),
        verification_receipt_digest: digest("receipt"),
    });
    fake_dependency = fake_dependency.seal().unwrap();
    assert!(fake_dependency
        .validate(&s.task, &h.origin, &s.resolved_dependencies)
        .is_err());
    let wire = serde_json::to_value(&h).unwrap();
    assert_eq!(wire["repair_input"]["trust"], "untrusted");
    assert_eq!(wire["notes"][0]["provenance"], "claimed");
}

#[test]
fn handoff_rejects_unsafe_unclaimed_and_instruction_paths() {
    let (s, _, _) = fixture();
    let h = handoff(&s);
    for path in [
        "../outside",
        "C:/outside",
        "/absolute",
        "\\\\server\\share",
        "src/file:stream",
        "src/CON.txt",
        "src/../outside",
        "other/file",
        "src/.git/config",
        "src/AGENTS.md",
        "src/.pytxo/state",
        "src/.env",
        "src/trailing.",
        "src\\file.rs",
    ] {
        let mut bad = h.clone();
        bad.changes[0].path = path.into();
        bad = bad.seal().unwrap();
        assert!(
            bad.validate(&s.task, &h.origin, &s.resolved_dependencies)
                .is_err(),
            "accepted {path}"
        );
    }
    let mut collision = h.clone();
    let mut duplicate = collision.changes[0].clone();
    duplicate.path = "src/FILE.rs".into();
    collision.changes.push(duplicate);
    collision = collision.seal().unwrap();
    assert!(collision
        .validate(&s.task, &h.origin, &s.resolved_dependencies)
        .is_err());
}

#[test]
fn qualification_receipts_and_capability_egress_resource_scope_come_from_trusted_facts() {
    let (s, c, _) = fixture();
    let mut unknown = c[0].clone();
    unknown
        .observation
        .qualification
        .as_mut()
        .unwrap()
        .receipt_digest = digest("self-attested");
    assert!(catalog(&s, &[unknown]).eligible.is_empty());
    let mut incapable = c[0].clone();
    incapable
        .observation
        .qualification
        .as_mut()
        .unwrap()
        .capabilities
        .clear();
    assert!(catalog(&s, &[incapable]).exclusions[0]
        .reasons
        .contains(&ExclusionReason::CapabilityMismatch));
    let mut unscoped = c[0].clone();
    unscoped
        .observation
        .qualification
        .as_mut()
        .unwrap()
        .capacity_pool_ids
        .clear();
    assert!(catalog(&s, &[unscoped]).exclusions[0]
        .reasons
        .contains(&ExclusionReason::ResourceMismatch));
    let mut s = s;
    s.task.required_egress = set(&["api.example"]);
    s.authorization.allowed_egress = s.task.required_egress.clone();
    refresh_task(&mut s);
    assert!(catalog(&s, &c).exclusions[0]
        .reasons
        .contains(&ExclusionReason::PermissionMismatch));
}

#[test]
fn planned_dependency_contract_stays_stable_when_trusted_winners_arrive() {
    let (mut s, c, p) = fixture();
    s.task.dependencies = vec![TaskId("prior".into())];
    refresh_task(&mut s);
    let approved_task = s.task.digest().unwrap();
    assert!(catalog(&s, &c).eligible.is_empty());
    let winner = VerifiedDependencyOutput {
        task_id: TaskId("prior".into()),
        winning_attempt_id: AttemptId("winner".into()),
        output_digest: digest("frozen-output"),
        verification_receipt_digest: digest("verified-checks"),
    };
    s.resolved_dependencies = vec![winner.clone()];
    assert_eq!(s.task.digest().unwrap(), approved_task);
    assert_eq!(catalog(&s, &c).eligible.len(), 2);
    assert!(matches!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Selected(_)
    ));
    let h = handoff(&s);
    h.validate(&s.task, &h.origin, &s.resolved_dependencies)
        .unwrap();
    let mut mismatch = s.resolved_dependencies.clone();
    mismatch[0].output_digest = digest("different");
    assert!(h.validate(&s.task, &h.origin, &mismatch).is_err());
    s.resolved_dependencies.push(winner);
    assert!(catalog(&s, &c).eligible.is_empty());
    s.resolved_dependencies.truncate(1);
    s.resolved_dependencies[0].task_id = TaskId("unexpected".into());
    assert!(catalog(&s, &c).eligible.is_empty());
}

#[test]
fn reviewed_task_kind_absence_defaults_strong_and_tampering_blocks() {
    let (mut s, c, p) = fixture();
    s.task.task_kind_evidence = None;
    refresh_task(&mut s);
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Selected(c[1].target())
    );
    s.task.task_kind_evidence = Some(digest("unapproved-kind"));
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(RouteBlocker::ScopeDrift)
    );
}

#[test]
fn capacity_wait_is_bounded_and_catalog_cannot_be_reused_after_snapshot_change() {
    let (mut s, mut c, p) = fixture();
    c[0].observation.capacity_ready = false;
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::WaitForCapacity {
            target: c[0].target(),
            max_wait_ms: 850
        }
    );
    let cat = catalog(&s, &c);
    s.task_revision += 1;
    assert!(!matches!(
        select_route(&s, &cat, &p, None).selection,
        RouteSelection::Selected(_)
    ));
    s.task_revision -= 1;
    s.now_ms += 1;
    assert_eq!(
        select_route(&s, &cat, &p, None).selection,
        RouteSelection::Blocked(RouteBlocker::StaleCatalog)
    );
}

#[test]
fn live_threshold_uses_exact_integer_ppm_boundary_and_unclear_is_strict() {
    let (mut s, c, mut p) = fixture();
    s.task.task_kind = Some(TaskKind::Diagnosis);
    refresh_task(&mut s);
    p.mode = RoutingMode::Live;
    p.evaluated_manifest_digest = Some(digest("evaluated"));
    s.authorization.live_advice_authorized = true;
    refresh_policy(&mut s, &p);
    let mut a = advice(&s);
    a.distribution = AdviceDistribution {
        everyday_fit: 0.8,
        strong_needed: 0.15,
        unclear: 0.05,
    };
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).reason,
        RouteReason::AdviceEveryday
    );
    a.distribution.everyday_fit = 0.7999999;
    a.distribution.strong_needed = 0.1500001;
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).reason,
        RouteReason::StrongDefault
    );
    a.distribution = AdviceDistribution {
        everyday_fit: 0.8,
        strong_needed: 0.1,
        unclear: 0.1,
    };
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).reason,
        RouteReason::StrongDefault
    );
}

#[test]
fn admitted_input_binding_failure_records_no_launch_only_with_positive_evidence() {
    let mut state = AttemptState::Admitted;
    assert!(state
        .transition(
            AttemptState::FailedNoLaunch,
            &AttemptTransitionEvidence::default()
        )
        .is_err());
    state
        .transition(
            AttemptState::FailedNoLaunch,
            &AttemptTransitionEvidence {
                no_worker_created: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(state.is_terminal());
}

#[test]
fn manual_pin_cannot_override_reviewed_required_or_strong_only_task() {
    let (mut s, c, p) = fixture();
    s.manual_target = Some(c[0].target());
    s.task.required_target = Some(c[1].target());
    refresh_task(&mut s);
    assert!(matches!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(_)
    ));
    s.task.required_target = None;
    s.task.strong_only = true;
    refresh_task(&mut s);
    assert!(matches!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(_)
    ));
}

#[test]
fn second_attempt_requires_actionable_first_failure_and_routes_only_strong_repair() {
    let (mut s, c, p) = fixture();
    s.next_ordinal = 2;
    s.admitted_attempts = 1;
    assert!(matches!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(_)
    ));
    s.previous_attempt = Some(RepairEvidence {
        attempt_id: AttemptId("first".into()),
        ordinal: 1,
        state: AttemptState::Failed,
        failure_class: AttemptFailureClass::Check,
        actionable_evidence_digest: Some(digest("failure-evidence")),
    });
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Selected(c[1].target())
    );
    for class in [
        AttemptFailureClass::Authentication,
        AttemptFailureClass::Environment,
        AttemptFailureClass::Quota,
        AttemptFailureClass::Cancellation,
        AttemptFailureClass::UnsupportedCapability,
        AttemptFailureClass::Unknown,
    ] {
        s.previous_attempt.as_mut().unwrap().failure_class = class;
        assert!(
            matches!(
                select_route(&s, &catalog(&s, &c), &p, None).selection,
                RouteSelection::Blocked(_)
            ),
            "{class:?}"
        );
    }
    s.previous_attempt.as_mut().unwrap().failure_class = AttemptFailureClass::Implementation;
    s.previous_attempt.as_mut().unwrap().state = AttemptState::RecoveryRequired;
    assert!(matches!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(_)
    ));
    s.previous_attempt.as_mut().unwrap().state = AttemptState::Failed;
    s.previous_attempt
        .as_mut()
        .unwrap()
        .actionable_evidence_digest = None;
    assert!(matches!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(_)
    ));
}

#[test]
fn live_never_downgrades_a_strong_or_unclear_answer_with_permissive_thresholds() {
    let (mut s, c, mut p) = fixture();
    s.task.task_kind = Some(TaskKind::Diagnosis);
    refresh_task(&mut s);
    p.mode = RoutingMode::Live;
    p.evaluated_manifest_digest = Some(digest("evaluated"));
    p.everyday_threshold_ppm = 200_000;
    p.unclear_ceiling_ppm = 800_000;
    s.authorization.live_advice_authorized = true;
    refresh_policy(&mut s, &p);
    let mut a = advice(&s);
    a.choice = AdviceChoice::StrongNeeded;
    a.distribution = AdviceDistribution {
        everyday_fit: 0.3,
        strong_needed: 0.6,
        unclear: 0.1,
    };
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).selection,
        RouteSelection::Selected(c[1].target())
    );
    a.choice = AdviceChoice::Unclear;
    a.distribution = AdviceDistribution {
        everyday_fit: 0.3,
        strong_needed: 0.1,
        unclear: 0.6,
    };
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, Some(&a)).selection,
        RouteSelection::Selected(c[1].target())
    );
}

#[test]
fn review_fix_state_revision_advances_without_changing_the_approved_contract() {
    let (mut s, c, p) = fixture();
    let approved_digest = s.task.digest().unwrap();
    let old_catalog = catalog(&s, &c);
    s.task_state_revision = 2;
    let fresh_catalog = catalog(&s, &c);
    assert_eq!(fresh_catalog.eligible.len(), 2);
    let decision = select_route(&s, &fresh_catalog, &p, None);
    assert_eq!(decision.selection, RouteSelection::Selected(c[0].target()));
    assert_eq!(decision.task_state_revision, 2);
    assert_eq!(s.task.revision, 1);
    assert_eq!(s.task_revision, 1);
    assert_eq!(s.task.digest().unwrap(), approved_digest);
    assert_eq!(
        select_route(&s, &old_catalog, &p, None).selection,
        RouteSelection::Blocked(RouteBlocker::StaleCatalog)
    );
}

#[test]
fn review_fix_advice_binds_current_task_state_revision_independently_of_contract_revision() {
    let (mut s, c, mut p) = fixture();
    s.task.task_kind = Some(TaskKind::Diagnosis);
    refresh_task(&mut s);
    p.mode = RoutingMode::Live;
    p.evaluated_manifest_digest = Some(digest("evaluated"));
    s.authorization.live_advice_authorized = true;
    refresh_policy(&mut s, &p);
    let mut a = advice(&s);
    s.task_state_revision += 1;
    let stale = select_route(&s, &catalog(&s, &c), &p, Some(&a));
    assert_eq!(stale.advice_status, AdviceStatus::InvalidOrStale);
    assert_eq!(stale.selection, RouteSelection::Selected(c[1].target()));
    a.task_state_revision = s.task_state_revision;
    let fresh = select_route(&s, &catalog(&s, &c), &p, Some(&a));
    assert_eq!(fresh.advice_status, AdviceStatus::Applied);
    assert_eq!(fresh.task_state_revision, s.task_state_revision);
    assert_eq!(fresh.selection, RouteSelection::Selected(c[0].target()));
}

fn repair_snapshot() -> (RoutingSnapshot, Vec<ProfileCandidate>, RoutingPolicy) {
    let (mut s, c, p) = fixture();
    s.next_ordinal = 2;
    s.admitted_attempts = 1;
    s.previous_attempt = Some(RepairEvidence {
        attempt_id: AttemptId("first".into()),
        ordinal: 1,
        state: AttemptState::Failed,
        failure_class: AttemptFailureClass::Check,
        actionable_evidence_digest: Some(digest("failure")),
    });
    (s, c, p)
}

#[test]
fn review_fix_second_ordinal_cannot_hide_prior_admission_under_one_attempt_cap() {
    let (mut s, c, p) = repair_snapshot();
    s.admitted_attempts = 0;
    s.authorization.limits.max_attempts = 1;
    assert!(catalog(&s, &c).eligible.is_empty());
    assert!(matches!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(_)
    ));
}

#[test]
fn review_fix_second_ordinal_requires_the_prior_slot_even_with_remaining_budget() {
    let (mut s, c, p) = repair_snapshot();
    s.admitted_attempts = 0;
    assert!(catalog(&s, &c).eligible.is_empty());
    assert!(matches!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(_)
    ));
}

#[test]
fn review_fix_mission_admission_count_may_exceed_this_tasks_prior_attempts() {
    let (mut s, c, p) = repair_snapshot();
    s.admitted_attempts = 5;
    s.authorization.limits.max_attempts = 6;
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Selected(c[1].target())
    );
    s.authorization.limits.max_attempts = 5;
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(RouteBlocker::BudgetExhausted)
    );
}

#[test]
fn review_fix_contradictory_launch_evidence_cannot_record_no_launch_or_advance_lifecycle() {
    let contradictory = AttemptTransitionEvidence {
        inputs_bound: true,
        launch_checks_passed: true,
        process_registered: true,
        no_worker_created: true,
        quiescent: true,
        output_sealed: true,
        checks_passed: true,
        authority_current: true,
        reconciliation_complete: true,
    };
    for (from, to) in [
        (AttemptState::Admitted, AttemptState::FailedNoLaunch),
        (AttemptState::Preparing, AttemptState::FailedNoLaunch),
        (AttemptState::Launching, AttemptState::FailedNoLaunch),
        (AttemptState::RecoveryRequired, AttemptState::FailedNoLaunch),
        (AttemptState::Launching, AttemptState::Running),
        (AttemptState::Preparing, AttemptState::Launching),
        (AttemptState::Admitted, AttemptState::Cancelled),
        (AttemptState::Launching, AttemptState::Cancelled),
    ] {
        let mut state = from;
        assert!(
            state.transition(to, &contradictory).is_err(),
            "accepted contradictory {from:?} -> {to:?}"
        );
        assert_eq!(state, from);
    }
    let no_launch = AttemptTransitionEvidence {
        no_worker_created: true,
        reconciliation_complete: true,
        ..Default::default()
    };
    for mut state in [
        AttemptState::Admitted,
        AttemptState::Preparing,
        AttemptState::Launching,
        AttemptState::RecoveryRequired,
    ] {
        state
            .transition(AttemptState::FailedNoLaunch, &no_launch)
            .unwrap();
    }
}

fn assert_invalid_check_contract(checks: Vec<CheckRecipe>) {
    let (mut s, c, p) = fixture();
    s.task.checks = checks;
    refresh_task(&mut s);
    assert!(catalog(&s, &c).eligible.is_empty());
    assert_eq!(
        select_route(&s, &catalog(&s, &c), &p, None).selection,
        RouteSelection::Blocked(RouteBlocker::InvalidContract)
    );
}

#[test]
fn review_fix_mandatory_check_id_cannot_be_empty_or_whitespace() {
    for id in ["", "   "] {
        assert_invalid_check_contract(vec![CheckRecipe {
            id: CheckId(id.into()),
            recipe_digest: digest("valid"),
        }]);
    }
}

#[test]
fn review_fix_mandatory_check_recipe_digest_must_be_valid() {
    assert_invalid_check_contract(vec![CheckRecipe {
        id: CheckId("check".into()),
        recipe_digest: Digest("bad".into()),
    }]);
}

#[test]
fn review_fix_mandatory_check_ids_must_be_unique_even_when_recipe_digests_differ() {
    assert_invalid_check_contract(vec![
        CheckRecipe {
            id: CheckId("check".into()),
            recipe_digest: digest("first"),
        },
        CheckRecipe {
            id: CheckId("check".into()),
            recipe_digest: digest("second"),
        },
    ]);
}
