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
use pytxo_planner::{MissionSpec, PlannerContext};
use pytxo_store::{Catalog, FlowDraftRecord};
use serde::{Deserialize, Serialize};

use crate::{
    dispatch_run_with_config_snapshot, ensure_repo_trusted, load_config_for_repo, plan_tasks,
    resolve_repo_root, RunOptions,
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
    let (cfg, ceiling_blocker) =
        apply_flow_permission_ceiling(&cfg, entitlements.permission_ceiling);

    // Explicit Flow / mission requests use the mission planner (ADR-0031).
    let mission = MissionSpec {
        text: input.mission_text.clone(),
    };
    let planner = pytxo_planner::default_mission_planner(&cfg);
    let planned = planner.decompose(
        &mission,
        &PlannerContext {
            repo: &repo,
            config: &cfg,
        },
    )?;
    let execution = plan_tasks(&planned.tasks, &cfg)?;

    let mut blocked_reasons = validate_task_claims(&planned.tasks, input.project_id.as_deref());
    blocked_reasons.extend(ceiling_blocker);
    blocked_reasons.extend(validate_permission_scope(&planned.tasks, &cfg));
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
    if draft.status != FlowStatus::Ready.as_str() {
        bail!("Flow plan review requires a persisted ready preview");
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
        {
            bail!("Flow plan structure changed; generate a new preview");
        }
        if edited.prompt.trim().is_empty() {
            bail!("Flow task prompts must not be empty");
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
    let draft = catalog
        .get_flow_draft(draft_id)?
        .with_context(|| format!("Flow draft not found: {draft_id}"))?;
    if draft.status != FlowStatus::Ready.as_str() || draft.plan_json.is_none() {
        bail!("Flow dispatch requires a persisted ready preview");
    }
    let expected_plan_json = draft.plan_json.as_deref().unwrap();
    let plan: FlowPlan =
        serde_json::from_str(expected_plan_json).context("invalid persisted Flow preview")?;
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
    let entitlements = crate::entitlements::effective_entitlements(&cfg)
        .map_err(|error| anyhow::anyhow!(error))?;
    let (cfg, _) = apply_flow_permission_ceiling(&cfg, entitlements.permission_ceiling);
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
    if !catalog.claim_flow_dispatch(draft_id, expected_plan_json)? {
        bail!("Flow draft is already dispatching or dispatched");
    }
    let dispatch = dispatch_run_with_config_snapshot(
        RunOptions {
            agents: tasks.len(),
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

/// Build a static ADE adapter command. Mission text is supplied separately through the child
/// environment, so reviewed prompts are never parsed as shell syntax.
fn ade_prompt_command(default_cmd: &str) -> String {
    if cfg!(windows) {
        let invocation = default_cmd
            .split_whitespace()
            .map(|part| format!("'{}'", part.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(" ");
        let script = format!("& {invocation} $env:PYTXO_TASK_PROMPT");
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
            verify: task.verify.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect::<Vec<_>>();
            let script = String::from_utf16(&units).expect("valid UTF-16LE PowerShell");
            assert_eq!(script, "& 'cursor-agent' $env:PYTXO_TASK_PROMPT");
        } else {
            assert!(command.contains("PYTXO_TASK_PROMPT"));
        }
    }
}
