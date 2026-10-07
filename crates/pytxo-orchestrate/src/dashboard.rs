use std::path::{Path, PathBuf};

use pytxo_store::{Catalog, PytxoStore};
use serde::Serialize;

use crate::doctor::DoctorReport;
use crate::flow::FlowPlan;
use crate::{
    event_lines, list_catalog_domains_enriched, list_hitl_pending, load_config, resolve_repo_root,
    CatalogEntryStatus, RunStatusJson,
};

#[derive(Clone, Debug, Serialize)]
pub struct DashboardSnapshot {
    pub version: String,
    pub repo_root: String,
    pub doctor: Option<DoctorReport>,
    pub runs: Vec<RunStatusJson>,
    pub domains: Vec<CatalogEntryStatus>,
    pub hitl_pending: Vec<pytxo_runner::HitlRequest>,
}

fn dashboard_snapshot_inner(
    repo: Option<PathBuf>,
    run_limit: usize,
    include_doctor: bool,
) -> anyhow::Result<DashboardSnapshot> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let doctor = if include_doctor {
        Some(crate::run_doctor(Some(&repo_root))?)
    } else {
        None
    };
    let cfg = load_config(None, &repo_root)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo_root))?;
    // Moved, deleted and temporary-directory domains are not projects the operator opened.
    let domains: Vec<_> = list_catalog_domains_enriched()
        .unwrap_or_default()
        .into_iter()
        .filter(|domain| domain.is_available && !domain.is_temporary)
        .collect();
    let mut run_json = if domains.len() > 1 {
        aggregate_runs_across_domains(&domains, run_limit)?
    } else {
        runs_for_store(&store, run_limit, &cfg)?
    };
    let drafts = Catalog::open_default()
        .and_then(|catalog| catalog.list_flow_drafts())
        .unwrap_or_default();
    for run in &mut run_json {
        run.title = drafts
            .iter()
            .find(|draft| draft.dispatched_run_id.as_deref() == Some(run.id.as_str()))
            .map(|draft| draft.title.clone());
    }
    let hitl_pending = list_hitl_pending(Some(repo_root.clone())).unwrap_or_default();
    Ok(DashboardSnapshot {
        version,
        repo_root: repo_root.display().to_string(),
        doctor,
        runs: run_json,
        domains,
        hitl_pending,
    })
}

fn runs_for_store(
    store: &PytxoStore,
    run_limit: usize,
    cfg: &pytxo_core::PytxoConfig,
) -> anyhow::Result<Vec<RunStatusJson>> {
    let runs = store.list_runs(run_limit)?;
    let mut run_json = Vec::new();
    for run in &runs {
        run_json.push(crate::run_status_json(run, store, cfg)?);
    }
    Ok(run_json)
}

fn aggregate_runs_across_domains(
    domains: &[CatalogEntryStatus],
    run_limit: usize,
) -> anyhow::Result<Vec<RunStatusJson>> {
    let per_domain = (run_limit / domains.len().max(1)).max(1);
    let mut all = Vec::new();
    for d in domains {
        let repo = Path::new(&d.repo_root);
        if let Ok(cfg) = crate::load_config(None, repo) {
            if let Ok(store) = PytxoStore::open(Path::new(&d.db_path)) {
                all.extend(runs_for_store(&store, per_domain, &cfg)?);
            }
        }
    }
    all.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    all.truncate(run_limit);
    Ok(all)
}

/// Read-only aggregate for the terminal dashboard (`pytxo-tui`), including doctor checks.
pub fn dashboard_snapshot(
    repo: Option<PathBuf>,
    run_limit: usize,
) -> anyhow::Result<DashboardSnapshot> {
    dashboard_snapshot_inner(repo, run_limit, true)
}

/// Lightweight board refresh — SQLite runs, domains, and HITL only (no PTY doctor smoke).
pub fn dashboard_snapshot_light(
    repo: Option<PathBuf>,
    run_limit: usize,
) -> anyhow::Result<DashboardSnapshot> {
    dashboard_snapshot_inner(repo, run_limit, false)
}

/// One reviewed task in a terminal fleet view: its CLI, recorded state and
/// the recent output of its worker. A task whose worker has not started yet
/// has no `agent_id` and reports `queued`.
#[derive(Clone, Debug, Serialize)]
pub struct WorkerPane {
    pub task_id: String,
    pub wave: usize,
    pub agent_id: Option<String>,
    pub status: String,
    pub exit_code: Option<i32>,
    pub launcher: Option<String>,
    pub paths: Vec<String>,
    pub recent: Vec<String>,
}

/// Panes for every task of a run: the reviewed plan supplies tasks, paths and
/// CLIs; recorded workers supply state and their last `lines` output lines.
pub fn worker_panes(run: &RunStatusJson, lines: usize) -> anyhow::Result<Vec<WorkerPane>> {
    let repo = Path::new(&run.repo_root);
    let cfg = load_config(None, repo)?;
    let store = PytxoStore::open(&cfg.db_path_at(repo))?;
    let plan = Catalog::open_default()
        .ok()
        .and_then(|catalog| catalog.list_flow_drafts().ok())
        .and_then(|drafts| {
            drafts
                .into_iter()
                .find(|draft| draft.dispatched_run_id.as_deref() == Some(run.id.as_str()))
        })
        .and_then(|draft| serde_json::from_str::<FlowPlan>(draft.plan_json.as_deref()?).ok());
    let tasks: Vec<(String, usize, Vec<String>, Option<String>)> = match &plan {
        Some(plan) => plan
            .tasks
            .iter()
            .map(|task| {
                let wave = plan.waves.iter().position(|wave| wave.contains(&task.id));
                let cli = task.ade_id.as_ref().or(plan.ade.requested.as_ref());
                let launcher = cli.and_then(|id| {
                    pytxo_core::all_ade_clis()
                        .iter()
                        .find(|spec| spec.id == id)
                        .map(|spec| spec.display_name.to_string())
                });
                (
                    task.id.clone(),
                    wave.unwrap_or(0),
                    task.paths.clone(),
                    launcher,
                )
            })
            .collect(),
        None => run
            .agents
            .iter()
            .map(|agent| {
                (
                    agent.task_id.clone(),
                    agent.wave.max(0) as usize,
                    Vec::new(),
                    None,
                )
            })
            .collect(),
    };
    tasks
        .into_iter()
        .map(|(task_id, wave, paths, planned_cli)| {
            let agent = run.agents.iter().find(|agent| agent.task_id == task_id);
            let mut recent = match agent {
                Some(agent) => event_lines(&store.list_events(&agent.id, 400)?),
                None => Vec::new(),
            };
            recent.drain(..recent.len().saturating_sub(lines));
            Ok(WorkerPane {
                wave,
                agent_id: agent.map(|agent| agent.id.clone()),
                status: agent.map_or_else(|| "queued".into(), |agent| agent.status.clone()),
                exit_code: agent.and_then(|agent| agent.exit_code),
                launcher: agent
                    .and_then(|agent| agent.launcher.clone())
                    .or(planned_cli),
                task_id,
                paths,
                recent,
            })
        })
        .collect()
}
