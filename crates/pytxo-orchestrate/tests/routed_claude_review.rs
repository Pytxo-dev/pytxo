#![cfg(windows)]

use std::fs;
use std::path::Path;
use std::process::Command;

use pytxo_core::routing::{BillingSourceMode, RoutingMode, TaskKind};
use pytxo_core::{PermissionProfile, PytxoConfig};
use pytxo_orchestrate::flow::{
    enable_experimental_hosted_advisor_local_consent, preview_reviewed_hosted_advisor_packet,
};
use pytxo_orchestrate::{
    dispatch_experimental_routed_flow, preview_experimental_claude_hosted_shadow_flow,
    preview_experimental_claude_hosted_shadow_flow_with_facts,
    preview_experimental_claude_proposal_flow,
    preview_experimental_claude_proposal_flow_with_facts, save_reviewed_flow_plan, trust_repo,
    FlowDraftInput, FlowSource, FlowStatus, ReviewedDemandFacts,
};
use pytxo_store::capacity::CapacityPoolConfig;
use pytxo_store::routing::StagedRoutingMissionRef;
use pytxo_store::{Catalog, PytxoStore};

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn reviewed_claude_preview_requires_gate_and_stages_subscription_without_a_run() {
    let home = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    let account = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("PYTXO_HOME", home.path().join("home"));
        std::env::set_var("PYTXO_TRUST_STORE", home.path().join("trust.json"));
        std::env::set_var("PYTXO_PLANNER", "signal");
        std::env::remove_var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL");
    }
    fs::create_dir(repo.path().join("src")).unwrap();
    fs::write(repo.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    fs::write(repo.path().join("pytxo.toml"), "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n").unwrap();
    git(repo.path(), &["init", "-q"]);
    git(repo.path(), &["add", "."]);
    git(
        repo.path(),
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    );
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let input = FlowDraftInput {
        id: "reviewed-claude".into(),
        title: "One-file proposal".into(),
        mission_text: "Update src/lib.rs".into(),
        source: FlowSource::Text,
        domain_id: Some(repo.path().to_string_lossy().into_owned()),
        project_id: None,
        ade_id: None,
        ade_ids: Vec::new(),
        task_ades: Default::default(),
        max_workers: Some(1),
        verification_commands: vec!["echo verification-ok".into()],
    };
    let hosted_input = FlowDraftInput {
        id: "reviewed-claude-hosted".into(),
        ..input.clone()
    };
    assert!(preview_experimental_claude_proposal_flow(
        &catalog,
        input.clone(),
        account.path(),
        "account-claude"
    )
    .is_err());
    unsafe {
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL", "1");
    }
    assert!(preview_experimental_claude_proposal_flow(
        &catalog,
        input.clone(),
        account.path(),
        "account-claude"
    )
    .is_err());
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "account-claude".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let plan = preview_experimental_claude_proposal_flow(
        &catalog,
        input,
        account.path(),
        "account-claude",
    )
    .unwrap();
    assert_eq!(plan.status, FlowStatus::Ready);
    assert_eq!(plan.ade.requested, None);
    assert_eq!(plan.ade.command, None);
    assert_eq!(plan.tasks.len(), 1);
    assert_eq!(plan.max_workers, 1);
    let normalized = tempfile::tempdir().unwrap();
    let normalized_repo = normalized.path().join("checkout");
    git(
        repo.path(),
        &[
            "-c",
            "core.autocrlf=true",
            "clone",
            "--quiet",
            repo.path().to_str().unwrap(),
            normalized_repo.to_str().unwrap(),
        ],
    );
    assert_eq!(
        fs::read(normalized_repo.join("src/lib.rs")).unwrap(),
        b"pub fn value() {}\r\n"
    );
    trust_repo(&normalized_repo, PermissionProfile::Orbit).unwrap();
    let normalized_input = FlowDraftInput {
        id: "reviewed-claude-normalized".into(),
        domain_id: Some(normalized_repo.to_string_lossy().into_owned()),
        ..hosted_input.clone()
    };
    let error = preview_experimental_claude_proposal_flow(
        &catalog,
        normalized_input,
        account.path(),
        "account-claude",
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("checkout bytes differ from Git blobs"),
        "{error}"
    );
    let routing = plan.routing.as_ref().unwrap();
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let mission = store
        .load_staged_routing_mission(&StagedRoutingMissionRef {
            domain_id: routing.authorization.domain_id.clone(),
            run_id: routing.authorization.run_id.clone(),
            draft_id: plan.draft_id.clone(),
            plan_digest: routing.authorization.plan_digest.clone(),
            mission_digest: routing.mission_digest.clone(),
        })
        .unwrap();
    assert_eq!(mission.policy.mode, RoutingMode::Rules);
    assert_eq!(mission.profiles[0].profile.requested_model.model, "haiku");
    assert_eq!(mission.profiles[1].profile.requested_model.model, "sonnet");
    assert!(mission
        .profiles
        .iter()
        .all(
            |profile| profile.binding.billing_mode == BillingSourceMode::Subscription
                && profile.binding.credential_reference.is_none()
        ));
    assert_eq!(mission.authorization.limits.max_attempts, 1);
    unsafe { std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_REPAIR", "1") };
    let repair_plan = preview_experimental_claude_proposal_flow_with_facts(
        &catalog,
        FlowDraftInput {
            id: "reviewed-claude-repair".into(),
            ..hosted_input.clone()
        },
        account.path(),
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::LocalTransformation,
            context_complete: true,
            cross_component_requirement: Some(false),
            repeatable_symptom_supplied: None,
            specific_cause_hypothesis_supplied: None,
        },
    )
    .unwrap();
    let repair_review = repair_plan.routing.as_ref().unwrap();
    let repair_mission = store
        .load_staged_routing_mission(&StagedRoutingMissionRef {
            domain_id: repair_review.authorization.domain_id.clone(),
            run_id: repair_review.authorization.run_id.clone(),
            draft_id: repair_plan.draft_id.clone(),
            plan_digest: repair_review.authorization.plan_digest.clone(),
            mission_digest: repair_review.mission_digest.clone(),
        })
        .unwrap();
    assert_eq!(repair_mission.authorization.limits.max_attempts, 2);
    assert_eq!(
        repair_mission.policy.version,
        "claude-proposal-rules-repair-v1"
    );
    let strong_plan = preview_experimental_claude_proposal_flow_with_facts(
        &catalog,
        FlowDraftInput {
            id: "reviewed-claude-strong-no-repair".into(),
            ..hosted_input.clone()
        },
        account.path(),
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::Architecture,
            context_complete: true,
            cross_component_requirement: Some(true),
            repeatable_symptom_supplied: None,
            specific_cause_hypothesis_supplied: None,
        },
    )
    .unwrap();
    let strong_review = strong_plan.routing.as_ref().unwrap();
    let strong_mission = store
        .load_staged_routing_mission(&StagedRoutingMissionRef {
            domain_id: strong_review.authorization.domain_id.clone(),
            run_id: strong_review.authorization.run_id.clone(),
            draft_id: strong_plan.draft_id.clone(),
            plan_digest: strong_review.authorization.plan_digest.clone(),
            mission_digest: strong_review.mission_digest.clone(),
        })
        .unwrap();
    assert_eq!(strong_mission.authorization.limits.max_attempts, 1);
    assert_eq!(strong_mission.policy.version, "claude-proposal-rules-v1");
    unsafe { std::env::remove_var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_REPAIR") };
    assert!(!mission.authorization.live_advice_authorized);
    assert_eq!(mission.tasks[0].contract.task_kind, Some(TaskKind::Other));
    assert!(!mission.tasks[0].contract.context_complete);
    assert_eq!(mission.tasks[0].contract.cross_component_requirement, None);
    assert!(store
        .get_run(&mission.authorization.run_id.0)
        .unwrap()
        .is_none());
    let public = catalog
        .get_flow_draft(&plan.draft_id)
        .unwrap()
        .unwrap()
        .plan_json
        .unwrap();
    assert!(!public.contains("credential_reference"));
    assert!(!public.contains(&account.path().to_string_lossy().to_string()));
    assert!(preview_experimental_claude_hosted_shadow_flow(
        &catalog,
        hosted_input.clone(),
        account.path(),
        "account-claude"
    )
    .is_err());
    unsafe { std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_HOSTED_SHADOW_REVIEW", "1") };
    let hosted_plan = preview_experimental_claude_hosted_shadow_flow(
        &catalog,
        hosted_input.clone(),
        account.path(),
        "account-claude",
    )
    .unwrap();
    assert_eq!(hosted_plan.status, FlowStatus::ReviewOnly);
    assert!(hosted_plan.warnings.iter().any(|warning| {
        warning.code == "hosted_shadow_review_only"
            && warning.message.contains("cannot be dispatched")
    }));
    let hosted_plan = save_reviewed_flow_plan(&catalog, hosted_plan).unwrap();
    let hosted_review = hosted_plan.routing.as_ref().unwrap();
    let hosted_mission = store
        .load_staged_routing_mission(&StagedRoutingMissionRef {
            domain_id: hosted_review.authorization.domain_id.clone(),
            run_id: hosted_review.authorization.run_id.clone(),
            draft_id: hosted_plan.draft_id.clone(),
            plan_digest: hosted_review.authorization.plan_digest.clone(),
            mission_digest: hosted_review.mission_digest.clone(),
        })
        .unwrap();
    assert_eq!(hosted_mission.policy.mode, RoutingMode::Shadow);
    assert_eq!(
        hosted_mission.policy.advisor_recipient.as_deref(),
        Some(pytxo_planner::advisor::HOSTED_RECIPIENT)
    );
    assert_eq!(hosted_review.authorization.consent_revision, 1);
    assert!(!hosted_review.authorization.live_advice_authorized);
    let packet = preview_reviewed_hosted_advisor_packet(&catalog, &hosted_plan.draft_id).unwrap();
    assert_eq!(packet.run_id, hosted_review.authorization.run_id.0);
    assert_eq!(packet.reviewed_consent_revision, 1);
    assert!(!packet.packet_body.is_empty());
    assert!(!packet.recordable_shadow_context);
    assert!(enable_experimental_hosted_advisor_local_consent(
        &catalog,
        &hosted_plan.draft_id,
        &packet,
        0,
    )
    .is_err());
    let default_packet: serde_json::Value = serde_json::from_slice(&packet.packet_body).unwrap();
    assert_eq!(default_packet["features"][0], "task_kind_unspecified");

    let diagnosed_input = FlowDraftInput {
        id: "reviewed-claude-hosted-diagnosis".into(),
        ..hosted_input.clone()
    };
    let diagnosed_plan = preview_experimental_claude_hosted_shadow_flow_with_facts(
        &catalog,
        diagnosed_input,
        account.path(),
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::Diagnosis,
            context_complete: true,
            cross_component_requirement: Some(false),
            repeatable_symptom_supplied: Some(true),
            specific_cause_hypothesis_supplied: Some(false),
        },
    )
    .unwrap();
    let diagnosed_plan = save_reviewed_flow_plan(&catalog, diagnosed_plan).unwrap();
    let diagnosed_packet =
        preview_reviewed_hosted_advisor_packet(&catalog, &diagnosed_plan.draft_id).unwrap();
    assert!(diagnosed_packet.recordable_shadow_context);
    assert_ne!(diagnosed_packet.packet_digest, packet.packet_digest);
    let diagnosis: serde_json::Value =
        serde_json::from_slice(&diagnosed_packet.packet_body).unwrap();
    assert_eq!(diagnosis["features"][0], "task_kind_diagnosis");
    assert_eq!(diagnosis["features"][2], "single_component_claimed");
    assert_eq!(diagnosis["features"][3], "context_claimed_complete");
    assert_eq!(diagnosis["features"][6], "repeatable_symptom_supplied");
    assert_eq!(
        diagnosis["features"][7],
        "specific_cause_hypothesis_not_supplied"
    );
    let private_variation = preview_experimental_claude_hosted_shadow_flow_with_facts(
        &catalog,
        FlowDraftInput {
            id: "reviewed-claude-hosted-diagnosis-private-variation".into(),
            mission_text: "Diagnose src/lib.rs for confidential customer ZK-917".into(),
            verification_commands: vec!["echo alternate-private-check".into()],
            ..hosted_input.clone()
        },
        account.path(),
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::Diagnosis,
            context_complete: true,
            cross_component_requirement: Some(false),
            repeatable_symptom_supplied: Some(true),
            specific_cause_hypothesis_supplied: Some(false),
        },
    )
    .unwrap();
    let private_variation = save_reviewed_flow_plan(&catalog, private_variation).unwrap();
    let private_packet =
        preview_reviewed_hosted_advisor_packet(&catalog, &private_variation.draft_id).unwrap();
    assert_eq!(private_packet.packet_body, diagnosed_packet.packet_body);
    assert_eq!(private_packet.packet_digest, diagnosed_packet.packet_digest);
    let other_diagnosis = preview_experimental_claude_hosted_shadow_flow_with_facts(
        &catalog,
        FlowDraftInput {
            id: "reviewed-claude-hosted-diagnosis-cues-b".into(),
            ..hosted_input.clone()
        },
        account.path(),
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::Diagnosis,
            context_complete: true,
            cross_component_requirement: Some(false),
            repeatable_symptom_supplied: Some(false),
            specific_cause_hypothesis_supplied: Some(true),
        },
    )
    .unwrap();
    let other_diagnosis = save_reviewed_flow_plan(&catalog, other_diagnosis).unwrap();
    let other_packet =
        preview_reviewed_hosted_advisor_packet(&catalog, &other_diagnosis.draft_id).unwrap();
    assert!(other_packet.recordable_shadow_context);
    assert_ne!(other_packet.packet_digest, diagnosed_packet.packet_digest);
    let other: serde_json::Value = serde_json::from_slice(&other_packet.packet_body).unwrap();
    assert_eq!(other["features"][6], "repeatable_symptom_not_supplied");
    assert_eq!(other["features"][7], "specific_cause_hypothesis_supplied");
    let cross_plan = preview_experimental_claude_hosted_shadow_flow_with_facts(
        &catalog,
        FlowDraftInput {
            id: "reviewed-claude-hosted-cross-component".into(),
            ..hosted_input
        },
        account.path(),
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::Diagnosis,
            context_complete: true,
            cross_component_requirement: Some(true),
            repeatable_symptom_supplied: None,
            specific_cause_hypothesis_supplied: None,
        },
    )
    .unwrap();
    let cross_review = cross_plan.routing.as_ref().unwrap();
    let cross_mission = store
        .load_staged_routing_mission(&StagedRoutingMissionRef {
            domain_id: cross_review.authorization.domain_id.clone(),
            run_id: cross_review.authorization.run_id.clone(),
            draft_id: cross_plan.draft_id.clone(),
            plan_digest: cross_review.authorization.plan_digest.clone(),
            mission_digest: cross_review.mission_digest.clone(),
        })
        .unwrap();
    assert!(cross_mission.tasks[0].contract.strong_only);
    assert_eq!(
        cross_mission.tasks[0].contract.cross_component_requirement,
        Some(true)
    );
    let packet_bytes = String::from_utf8(packet.packet_body.clone()).unwrap();
    let diagnosed_bytes = String::from_utf8(diagnosed_packet.packet_body).unwrap();
    let other_bytes = String::from_utf8(other_packet.packet_body).unwrap();
    for private_value in [
        "Update src/lib.rs",
        "src/lib.rs",
        "echo verification-ok",
        &account.path().to_string_lossy(),
    ] {
        assert!(
            !packet_bytes.contains(private_value),
            "hosted packet leaked {private_value}"
        );
        assert!(!diagnosed_bytes.contains(private_value));
        assert!(!other_bytes.contains(private_value));
    }
    assert_eq!(
        packet.scope_digest,
        pytxo_planner::advisor::hosted_scope_digest()
    );
    assert_eq!(
        packet.request_digest,
        hosted_mission
            .policy
            .advice_template
            .rsplit(':')
            .next()
            .map(|digest| pytxo_core::routing::Digest(digest.into()))
            .unwrap()
    );
    assert!(store
        .get_run(&hosted_review.authorization.run_id.0)
        .unwrap()
        .is_none());
    let dispatch_error =
        dispatch_experimental_routed_flow(&catalog, &hosted_plan.draft_id).unwrap_err();
    assert!(
        format!("{dispatch_error:#}")
            .contains("review-only hosted Shadow Flow cannot be dispatched"),
        "unexpected hosted dispatch error: {dispatch_error:#}"
    );
    assert_eq!(
        catalog
            .get_flow_draft(&hosted_plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "review_only"
    );
    unsafe { std::env::remove_var("PYTXO_EXPERIMENTAL_ROUTED_HOSTED_SHADOW_REVIEW") };
}
