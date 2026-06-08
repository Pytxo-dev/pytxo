use std::path::{Path, PathBuf};

use pytxo_core::DomainId;
use pytxo_store::PytxoStore;
use serde::Serialize;

use crate::doctor::DoctorReport;
use crate::{
    list_catalog_domains, list_hitl_pending, load_config, resolve_repo_root, AgentStatusJson,
    CatalogEntry, RunStatusJson,
};

#[derive(Clone, Debug, Serialize)]
pub struct DashboardSnapshot {
    pub version: String,
    pub repo_root: String,
    pub doctor: Option<DoctorReport>,
    pub runs: Vec<RunStatusJson>,
    pub domains: Vec<CatalogEntry>,
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
    let runs = store.list_runs(run_limit)?;
    let mut run_json = Vec::new();
    for run in &runs {
        let agents: Vec<AgentStatusJson> = store
            .list_agents_for_run(&run.id)?
            .into_iter()
            .map(|a| AgentStatusJson {
                id: a.id,
                task_id: a.task_id,
                wave: a.wave,
                status: a.status,
                exit_code: a.exit_code,
            })
            .collect();
        let arbitrage_saved = store.arbitrage_saved_tokens_for_run(&run.id).ok();
        let wallet_balance = DomainId::from_repo_root(Path::new(&run.repo_root))
            .ok()
            .and_then(|d| store.wallet_balance_microcredits(&d).ok());
        run_json.push(RunStatusJson {
            id: run.id.clone(),
            status: run.status.clone(),
            repo_root: run.repo_root.clone(),
            started_at: run.started_at.to_rfc3339(),
            estimated_tokens_in: run.estimated_tokens_in,
            estimated_tokens_out: run.estimated_tokens_out,
            estimated_cost_usd: run.estimated_cost_usd,
            arbitrage_saved_tokens: arbitrage_saved,
            wallet_balance_microcredits: wallet_balance,
            agents,
        });
    }
    let domains = list_catalog_domains()?;
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
