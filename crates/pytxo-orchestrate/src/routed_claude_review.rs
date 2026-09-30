//! Trusted, narrow review builder for the experimental Claude proposal route.
//! The Desktop/CLI caller supplies task intent; it cannot supply profiles or
//! billing authority as serialized UI data.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::bail;
use pytxo_core::routing::{
    ApprovedProfile, BillingSourceMode, BindingId, Digest, ExecutionProfile, MissionAuthorization,
    MissionLimits, ModelIdentity, ModelIdentityLevel, PlanId, ProfileBinding, ProfileId,
    RouteTarget, RoutingMode, RoutingPolicy, SpendGuarantee, TaskContract, TaskKind,
};
use pytxo_core::{DomainId, ExecutionBackend, PermissionProfile, RunId, TaskId};
use pytxo_planner::advisor::{hosted_scope_digest, HOSTED_RECIPIENT, MODEL_ID};
use pytxo_store::routing::{
    FrozenCheckRecipeV1, RegisteredProfile, RegisteredTask, RoutingMission,
};
use pytxo_store::{Catalog, PytxoStore};
use serde::{Deserialize, Serialize};

use crate::flow::{
    freeze_experimental_routed_checks, observe_experimental_routed_git_base,
    preview_experimental_routed_flow, reviewed_hosted_routing_advisor_identity_for_task,
    FlowDraftInput, FlowPlan,
};
use crate::routed_claude::{
    claude_subscription_account_source_id, CLAUDE_PROPOSAL_ADAPTER_ID,
    CLAUDE_PROPOSAL_TOOL_BUNDLE_ID, CLAUDE_SUBSCRIPTION_ENDPOINT,
};

/// Explicit experiment only. The account directory and capacity pool become
/// immutable reviewed binding facts; readiness is observed again at dispatch.
pub fn preview_experimental_claude_proposal_flow(
    catalog: &Catalog,
    input: FlowDraftInput,
    account_home: &Path,
    capacity_pool_id: &str,
) -> anyhow::Result<FlowPlan> {
    preview_experimental_claude_proposal_flow_with_facts(
        catalog,
        input,
        account_home,
        capacity_pool_id,
        ReviewedDemandFacts::default(),
    )
}

/// Caller-declared, coarse execution-demand facts. They become reviewed only
/// with the exact Flow plan; a filename cannot establish complete context.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedDemandFacts {
    pub task_kind: TaskKind,
    pub context_complete: bool,
    pub cross_component_requirement: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeatable_symptom_supplied: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specific_cause_hypothesis_supplied: Option<bool>,
}

impl Default for ReviewedDemandFacts {
    fn default() -> Self {
        Self {
            task_kind: TaskKind::Other,
            context_complete: false,
            cross_component_requirement: None,
            repeatable_symptom_supplied: None,
            specific_cause_hypothesis_supplied: None,
        }
    }
}

pub fn preview_experimental_claude_proposal_flow_with_facts(
    catalog: &Catalog,
    input: FlowDraftInput,
    account_home: &Path,
    capacity_pool_id: &str,
    facts: ReviewedDemandFacts,
) -> anyhow::Result<FlowPlan> {
    preview_claude_proposal_flow(catalog, input, account_home, capacity_pool_id, false, facts)
}

/// This prepares a hosted-recipient Shadow review only. Dispatch remains
/// fail-closed until the account/grant/send controller is qualified.
pub fn preview_experimental_claude_hosted_shadow_flow(
    catalog: &Catalog,
    input: FlowDraftInput,
    account_home: &Path,
    capacity_pool_id: &str,
) -> anyhow::Result<FlowPlan> {
    preview_experimental_claude_hosted_shadow_flow_with_facts(
        catalog,
        input,
        account_home,
        capacity_pool_id,
        ReviewedDemandFacts::default(),
    )
}

pub fn preview_experimental_claude_hosted_shadow_flow_with_facts(
    catalog: &Catalog,
    input: FlowDraftInput,
    account_home: &Path,
    capacity_pool_id: &str,
    facts: ReviewedDemandFacts,
) -> anyhow::Result<FlowPlan> {
    if std::env::var("PYTXO_EXPERIMENTAL_ROUTED_HOSTED_SHADOW_REVIEW").as_deref() != Ok("1") {
        bail!("experimental hosted Shadow review is disabled");
    }
    preview_claude_proposal_flow(catalog, input, account_home, capacity_pool_id, true, facts)
}

fn preview_claude_proposal_flow(
    catalog: &Catalog,
    input: FlowDraftInput,
    account_home: &Path,
    capacity_pool_id: &str,
    hosted_shadow: bool,
    facts: ReviewedDemandFacts,
) -> anyhow::Result<FlowPlan> {
    if !cfg!(windows) || !crate::routed_fixture::claude_proposal_opted_in() {
        bail!("experimental Claude proposal routing is not enabled on this Windows host");
    }
    if capacity_pool_id.trim() != capacity_pool_id || capacity_pool_id.is_empty() {
        bail!("Claude subscription capacity pool must be an exact nonempty ID");
    }
    let billing_source = claude_subscription_account_source_id(account_home)?;
    if catalog.capacity_pool_status(capacity_pool_id)?.is_none() {
        bail!("reviewed Claude subscription capacity pool is not configured");
    }
    let pool = capacity_pool_id.to_owned();
    preview_experimental_routed_flow(catalog, input, move |plan, run_id, plan_digest| {
        let advice = if hosted_shadow {
            let repo = Path::new(&plan.domain_id);
            let config = crate::load_config_for_repo(None, repo)?;
            let store = PytxoStore::open(&config.db_path_at(repo))?;
            let consent = store.routing_hosted_advisor_consent(
                &DomainId(plan.domain_id.clone()),
                HOSTED_RECIPIENT,
            )?;
            AdvisorReview::HostedShadow(
                consent
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("hosted consent revision exhausted"))?,
            )
        } else {
            AdvisorReview::Rules
        };
        build_mission(
            plan,
            run_id,
            plan_digest,
            &billing_source,
            &pool,
            advice,
            facts,
        )
    })
}

#[derive(Clone, Copy)]
enum AdvisorReview {
    Rules,
    HostedShadow(u64),
}

fn profile(
    model: &str,
    billing_source: &pytxo_core::routing::BillingSourceId,
    pool: &str,
) -> anyhow::Result<RegisteredProfile> {
    let execution = ExecutionProfile {
        schema_version: 1,
        canonicalization_version: 1,
        id: ProfileId(format!("claude-subscription-{model}")),
        revision: 1,
        harness_id: "claude".into(),
        adapter_contract_version: "1".into(),
        adapter_digest: Digest::of_bytes(CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes()),
        requested_model: ModelIdentity {
            provider: "anthropic".into(),
            model: model.into(),
            reasoning: None,
            revision: None,
        },
        skill_tool_bundle_digest: Digest::of_bytes(CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes()),
        backend: ExecutionBackend::Subprocess,
        capabilities: BTreeSet::from(["read".into(), "edit".into()]),
    };
    let binding = ProfileBinding {
        schema_version: 1,
        canonicalization_version: 1,
        id: BindingId(format!("claude-subscription-{model}-account")),
        revision: 1,
        profile_digest: execution.digest()?,
        credential_reference: None,
        auth_owner: "Claude".into(),
        billing_source_id: billing_source.clone(),
        billing_mode: BillingSourceMode::Subscription,
        endpoint_identity: CLAUDE_SUBSCRIPTION_ENDPOINT.into(),
        trust_class: "vendor".into(),
        capacity_pool_ids: BTreeSet::from([pool.to_owned()]),
    };
    Ok(RegisteredProfile {
        profile: execution,
        binding,
    })
}

fn build_mission(
    plan: &FlowPlan,
    run_id: &RunId,
    plan_digest: &Digest,
    billing_source: &pytxo_core::routing::BillingSourceId,
    pool: &str,
    advisor_review: AdvisorReview,
    facts: ReviewedDemandFacts,
) -> anyhow::Result<RoutingMission> {
    let [planned] = plan.tasks.as_slice() else {
        bail!("Claude proposal review requires one task");
    };
    if planned.paths.len() != 1 || !planned.dependencies.is_empty() || planned.verify.len() != 1 {
        bail!("Claude proposal review requires one claimed file and one frozen check");
    }
    let profiles = vec![
        profile("haiku", billing_source, pool)?,
        profile("sonnet", billing_source, pool)?,
    ];
    let targets: Vec<RouteTarget> = profiles
        .iter()
        .map(|entry| RouteTarget {
            profile_id: entry.profile.id.clone(),
            binding_id: entry.binding.id.clone(),
        })
        .collect();
    let mut policy = RoutingPolicy {
        schema_version: 1,
        version: "claude-proposal-rules-v1".into(),
        mode: RoutingMode::Rules,
        everyday: targets[0].clone(),
        strong: targets[1].clone(),
        evaluated_manifest_digest: None,
        everyday_threshold_ppm: 800_000,
        unclear_ceiling_ppm: 100_000,
        advice_model: "unused".into(),
        advice_template: "unused".into(),
        disclosure_scope_digest: None,
        advisor_recipient: None,
    };
    let reviewed_repair = matches!(advisor_review, AdvisorReview::Rules)
        && crate::routed_fixture::claude_repair_opted_in()
        && matches!(
            facts.task_kind,
            TaskKind::Documentation
                | TaskKind::Formatting
                | TaskKind::Rename
                | TaskKind::LocalTransformation
        )
        && facts.context_complete
        && facts.cross_component_requirement == Some(false);
    if reviewed_repair {
        policy.version = "claude-proposal-rules-repair-v1".into();
    }
    let check_recipes = freeze_experimental_routed_checks(&planned.id, &planned.verify)?;
    let checks = check_recipes
        .iter()
        .map(FrozenCheckRecipeV1::reference)
        .collect::<pytxo_core::Result<Vec<_>>>()?;
    let task = RegisteredTask {
        contract: TaskContract {
            schema_version: 1,
            canonicalization_version: 1,
            task_id: TaskId(planned.id.clone()),
            revision: 1,
            plan_digest: plan_digest.clone(),
            base: observe_experimental_routed_git_base(Path::new(&plan.domain_id))?,
            goal: planned.prompt.clone(),
            constraints: vec![],
            claim_roots: planned.paths.clone(),
            dependencies: vec![],
            task_kind: Some(facts.task_kind),
            task_kind_evidence: Some(pytxo_core::routing::canonical_digest(
                &(
                    "declared-one-file-demand-v1",
                    &planned.id,
                    plan_digest,
                    &planned.paths,
                    &planned.verify,
                    facts,
                ),
                1,
            )?),
            required_capabilities: BTreeSet::from(["edit".into()]),
            checks,
            required_resources: BTreeSet::from([pool.to_owned()]),
            skill_tool_bundle_digest: Digest::of_bytes(CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes()),
            permission_profile: PermissionProfile::Orbit,
            required_egress: BTreeSet::from([CLAUDE_SUBSCRIPTION_ENDPOINT.into()]),
            required_target: None,
            strong_only: facts.task_kind == TaskKind::Architecture
                || facts.cross_component_requirement == Some(true),
            cross_component_requirement: facts.cross_component_requirement,
            context_complete: facts.context_complete,
            repeatable_symptom_supplied: (facts.task_kind == TaskKind::Diagnosis)
                .then_some(facts.repeatable_symptom_supplied)
                .flatten(),
            specific_cause_hypothesis_supplied: (facts.task_kind == TaskKind::Diagnosis)
                .then_some(facts.specific_cause_hypothesis_supplied)
                .flatten(),
        },
        attempt_budget_nano_usd: 0,
        check_recipes,
    };
    if matches!(advisor_review, AdvisorReview::HostedShadow(_)) {
        policy.version = "claude-proposal-hosted-shadow-v1".into();
        policy.mode = RoutingMode::Shadow;
        policy.advice_model = MODEL_ID.into();
        policy.advice_template = reviewed_hosted_routing_advisor_identity_for_task(&task.contract)?;
        policy.disclosure_scope_digest = Some(hosted_scope_digest());
        policy.advisor_recipient = Some(HOSTED_RECIPIENT.into());
    }
    let allowed_profiles = profiles
        .iter()
        .zip(&targets)
        .map(|(entry, target)| {
            Ok(ApprovedProfile {
                target: target.clone(),
                profile_digest: entry.profile.digest()?,
                binding_digest: entry.binding.digest()?,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let reviewed_at_ms = u64::try_from(chrono::Utc::now().timestamp_millis())?;
    let deadline_ms = reviewed_at_ms
        .checked_add(60 * 60 * 1_000)
        .ok_or_else(|| anyhow::anyhow!("Claude proposal review deadline overflow"))?;
    let authorization = MissionAuthorization {
        schema_version: 1,
        domain_id: DomainId(plan.domain_id.clone()),
        run_id: run_id.clone(),
        plan_id: PlanId(format!("flow:{}", plan.draft_id)),
        plan_digest: plan_digest.clone(),
        revision: 1,
        cancel_epoch: 0,
        allowed_task_digests: BTreeSet::from([task.contract.digest()?]),
        allowed_profiles,
        allowed_billing_sources: BTreeSet::from([billing_source.clone()]),
        allowed_billing_modes: BTreeSet::from([BillingSourceMode::Subscription]),
        permission_profile: PermissionProfile::Orbit,
        allowed_egress: BTreeSet::from([CLAUDE_SUBSCRIPTION_ENDPOINT.into()]),
        minimum_model_identity: ModelIdentityLevel::Requested,
        limits: MissionLimits {
            deadline_ms,
            max_workers: 1,
            max_attempts: if reviewed_repair { 2 } else { 1 },
            max_spend_nano_usd: None,
            spend_guarantee: SpendGuarantee::RiskBounded,
        },
        policy_digest: policy.digest()?,
        live_advice_authorized: false,
        consent_revision: match advisor_review {
            AdvisorReview::Rules => 1,
            AdvisorReview::HostedShadow(revision) => revision,
        },
    };
    Ok(RoutingMission {
        authorization,
        policy,
        tasks: vec![task],
        profiles,
    })
}
