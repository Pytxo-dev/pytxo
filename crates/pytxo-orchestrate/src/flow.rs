//! Text-first Flow planning for one execution domain.
//!
//! Flow is an orchestration facade: the selected repository is canonicalized into a single
//! execution domain, while planner, scheduler, permission, Blast Shield, Race Shield, and ADE
//! checks remain in their owning layers. Desktop presents telemetry and intent; it never
//! enforces policy.

use std::collections::HashMap;
use std::path::{Component, Path};

use anyhow::{bail, Context};
use chrono::Utc;
use pytxo_core::{
    ade_on_path, all_ade_clis, resolve_ade, PermissionProfile, PytxoConfig, Task, TaskId,
};
use pytxo_planner::{MissionPlanner, MissionSpec, PlannerContext, SignalBackedPlanner};
use pytxo_store::{Catalog, FlowDraftRecord};
use serde::{Deserialize, Serialize};

use crate::{
    dispatch_run, ensure_repo_trusted, load_config_for_repo, plan_tasks, resolve_repo_root,
    RunOptions,
};

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
    Blocked,
    Dispatching,
    Dispatched,
    Failed,
}

impl FlowStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Transcribing => "transcribing",
            Self::Planning => "planning",
            Self::Ready => "ready",
            Self::Blocked => "blocked",
            Self::Dispatching => "dispatching",
            Self::Dispatched => "dispatched",
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlowPlanTask {
    pub id: String,
    pub agent: String,
    pub prompt: String,
    pub paths: Vec<String>,
    pub dependencies: Vec<String>,
    pub root: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlowWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FlowBlockedReason {
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
}

/// Produce and persist the mandatory dry-run preview for an explicit Flow request.
pub fn preview_flow(catalog: &Catalog, input: FlowDraftInput) -> anyhow::Result<FlowPlan> {
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

    // Explicit Flow requests intentionally invoke the existing public implementation directly;
    // the shell's hidden PYTXO_PLANNER gate does not apply to a user-requested preview.
    let mission = MissionSpec {
        text: input.mission_text.clone(),
    };
    let planner = SignalBackedPlanner;
    let planned = planner.decompose(
        &mission,
        &PlannerContext {
            repo: &repo,
            config: &cfg,
        },
    )?;
    let execution = plan_tasks(&planned.tasks, &cfg)?;

    let mut blocked_reasons = validate_task_claims(&planned.tasks, input.project_id.as_deref());
    blocked_reasons.extend(validate_permission_scope(&planned.tasks, &cfg));
    blocked_reasons.extend(execution.conflicts.iter().map(|conflict| {
        FlowBlockedReason::OverlappingPathClaims {
            task_a: conflict.task_a.0.clone(),
            task_b: conflict.task_b.0.clone(),
            paths: conflict.paths.clone(),
        }
    }));
    let ade = summarize_ade(input.ade_id.as_deref());
    if !ade.available {
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
    warnings.extend(execution.conflicts.iter().map(|conflict| FlowWarning {
        code: "path_claim_overlap".into(),
        message: format!(
            "{} and {} overlap on {}",
            conflict.task_a.0,
            conflict.task_b.0,
            conflict.paths.join(", ")
        ),
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
    let plan = FlowPlan {
        draft_id: input.id.clone(),
        domain_id: repo.to_string_lossy().into_owned(),
        project_id: input.project_id.clone(),
        status,
        tasks,
        waves,
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
    };
    let previous_created = catalog
        .get_flow_draft(&input.id)?
        .map(|draft| draft.created_at)
        .unwrap_or_else(|| now.clone());
    catalog.upsert_flow_draft(&FlowDraftRecord {
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
    })?;
    Ok(plan)
}

/// Revalidate and dispatch a persisted ready preview through the standard `dispatch_run` path.
pub fn dispatch_flow(catalog: &Catalog, draft_id: &str) -> anyhow::Result<String> {
    let draft = catalog
        .get_flow_draft(draft_id)?
        .with_context(|| format!("Flow draft not found: {draft_id}"))?;
    if draft.status != FlowStatus::Ready.as_str() || draft.plan_json.is_none() {
        bail!("Flow dispatch requires a persisted ready preview");
    }
    let plan: FlowPlan = serde_json::from_str(draft.plan_json.as_deref().unwrap())
        .context("invalid persisted Flow preview")?;
    if plan.status != FlowStatus::Ready || !plan.blocked_reasons.is_empty() {
        bail!("Flow dispatch requires a persisted ready preview");
    }
    if draft.domain_id.as_deref() != Some(plan.domain_id.as_str())
        || draft.project_id != plan.project_id
    {
        bail!("Flow execution domain changed; generate a new preview");
    }
    let repo = resolve_repo_root(Some(Path::new(&plan.domain_id)))?;
    let cfg = load_config_for_repo(None, &repo)?;
    if cfg.permission_profile.as_str() != plan.permission_profile {
        bail!("Flow permission profile changed; generate a new preview");
    }
    if cfg.isolation.as_str() != plan.isolation_mode
        || pytxo_runner::effective_isolation_mode(&cfg).as_str() != plan.isolation_backend_intent
        || format!("{:?}", cfg.execution_backend).to_ascii_lowercase() != plan.execution_backend
    {
        bail!("Flow execution policy changed; generate a new preview");
    }
    let tasks = runtime_tasks(&plan);
    let mut blockers = validate_task_claims(&tasks, plan.project_id.as_deref());
    blockers.extend(validate_permission_scope(&tasks, &cfg));
    if !blockers.is_empty() || !pytxo_scheduler::find_conflicts(&tasks).is_empty() {
        bail!("Flow path claims no longer pass validation; generate a new preview");
    }
    let ade_id = plan
        .ade
        .requested
        .as_deref()
        .context("Flow dispatch requires a selected ADE; generate a new preview")?;
    let ade = resolve_ade(ade_id).context("selected Flow ADE is not registered")?;
    if !ade_on_path(ade) {
        bail!("selected Flow ADE is unavailable: {ade_id}");
    }
    ensure_repo_trusted(&repo)?;
    let prompts: HashMap<String, String> = plan
        .tasks
        .iter()
        .map(|task| (task.id.clone(), task.prompt.clone()))
        .collect();
    let (_, run_id) = dispatch_run(RunOptions {
        agents: tasks.len(),
        cmd: ade.default_cmd.into(),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo),
        execution: None,
        project: None,
        tasks: Some(tasks),
        task_cmd_template: Some(format!("{} {{prompt}}", ade.default_cmd)),
        task_prompts: Some(prompts),
    })?;
    catalog.mark_flow_dispatched(draft_id, &run_id)?;
    Ok(run_id)
}

fn summarize_ade(requested: Option<&str>) -> FlowAdeSummary {
    let available_specs: Vec<_> = all_ade_clis()
        .iter()
        .filter(|spec| ade_on_path(spec))
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
        available: selected.is_some_and(ade_on_path),
        installed,
        command: selected.map(|spec| spec.default_cmd.to_string()),
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
                || path
                    .components()
                    .any(|part| matches!(part, Component::ParentDir))
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

fn profile_rank(profile: PermissionProfile) -> u8 {
    match profile {
        PermissionProfile::DeepSpace => 0,
        PermissionProfile::Orbit => 1,
        PermissionProfile::Galaxy => 2,
        PermissionProfile::Supernova => 3,
    }
}

fn runtime_tasks(plan: &FlowPlan) -> Vec<Task> {
    plan.tasks
        .iter()
        .map(|task| Task {
            id: TaskId(task.id.clone()),
            agent: task.agent.clone(),
            paths: task.paths.clone(),
            depends_on: task.dependencies.clone(),
            root: task.root.clone(),
            signal_fidelity: None,
        })
        .collect()
}
