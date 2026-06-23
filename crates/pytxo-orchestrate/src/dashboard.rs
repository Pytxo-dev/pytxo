use std::path::{Path, PathBuf};

use pytxo_store::PytxoStore;
use serde::Serialize;

use crate::doctor::DoctorReport;
use crate::{
    list_catalog_domains_enriched, list_hitl_pending, load_config, resolve_repo_root,
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
    let domains = list_catalog_domains_enriched().unwrap_or_default();
    let run_json = if domains.len() > 1 {
        aggregate_runs_across_domains(&domains, run_limit)?
    } else {
        runs_for_store(&store, run_limit, &cfg)?
    };
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
