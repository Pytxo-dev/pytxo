//! Hypervisor fleet DAG orchestration ([[hypervisor-fleet-dag]], [[ADR-0015-hypervisor-fleet-dag]]).

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use pytxo_core::{canonical_repo_root, DomainId, FleetManifest, FleetPlan, RunId};
use pytxo_store::{Catalog, FleetNodeRecord, FleetRunRecord, PytxoStore};
use serde::Serialize;
use tokio::time::sleep;

use crate::hypervisor::default_hypervisor;
use crate::{ensure_repo_trusted, load_config, RunOptions};

const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(3600);
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Inputs for `pytxo fleet run`.
pub struct FleetRunOptions {
    pub manifest: Option<PathBuf>,
    pub fleet_id: Option<String>,
    pub dry_run: bool,
    pub wait_timeout: Duration,
    /// When true, continue later waves after a node failure (ADR-0015).
    pub continue_on_error: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FleetRunResult {
    pub fleet_run_id: String,
    pub fleet_id: String,
    pub status: String,
    pub nodes: Vec<FleetNodeResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FleetNodeResult {
    pub node_id: String,
    pub domain_id: String,
    pub domain_run_id: Option<String>,
    pub wave: u32,
    pub status: String,
}

fn discover_manifest(manifest: Option<&Path>, id: Option<&str>) -> anyhow::Result<FleetManifest> {
    let path = FleetManifest::discover(manifest, id)
        .ok_or_else(|| anyhow::anyhow!("no fleet manifest found (pass --manifest or --id)"))?;
    FleetManifest::load(&path).map_err(|e| anyhow::anyhow!(e))
}

pub fn fleet_plan_from_manifest(
    manifest: Option<PathBuf>,
    fleet_id: Option<String>,
) -> anyhow::Result<FleetPlan> {
    let m = discover_manifest(manifest.as_deref(), fleet_id.as_deref())?;
    m.plan().map_err(|e| anyhow::anyhow!(e))
}

pub fn fleet_dry_run_json(
    manifest: Option<PathBuf>,
    fleet_id: Option<String>,
) -> anyhow::Result<String> {
    let plan = fleet_plan_from_manifest(manifest, fleet_id)?;
    let mut waves = Vec::new();
    for w in &plan.waves {
        let mut nodes = Vec::new();
        for n in &w.nodes {
            let canon = canonical_repo_root(&n.repo).map_err(|e| anyhow::anyhow!(e))?;
            let domain_id = DomainId::from_repo_root(&canon).map_err(|e| anyhow::anyhow!(e))?;
            nodes.push(serde_json::json!({
                "id": n.id,
                "repo": canon.to_string_lossy(),
                "domain_id": domain_id.as_str(),
                "cmd": n.cmd,
                "agents": n.agents,
                "depends_on": n.depends_on,
            }));
        }
        waves.push(serde_json::json!({
            "wave": w.wave,
            "nodes": nodes,
        }));
    }
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "fleet_id": plan.fleet_id,
        "waves": waves,
    }))?)
}

/// Poll a domain's `pytxo.db` until the run reaches a terminal status.
pub async fn wait_for_domain_run(
    db_path: &Path,
    run_id: &str,
    timeout: Duration,
) -> anyhow::Result<String> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let status = {
            let store = PytxoStore::open(db_path)?;
            match store.get_run_status(run_id)? {
                Some((s, _)) => s,
                None => "pending".to_string(),
            }
        };
        if status == "pending" {
            if tokio::time::Instant::now() >= deadline {
                anyhow::bail!("timeout waiting for run {run_id} to appear in {}", db_path.display());
            }
            sleep(POLL_INTERVAL).await;
            continue;
        }
        if PytxoStore::is_terminal_run_status(&status) {
            return Ok(status);
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("timeout waiting for run {run_id} (last status: {status})");
        }
        sleep(POLL_INTERVAL).await;
    }
}

pub async fn fleet_run(opts: FleetRunOptions) -> anyhow::Result<FleetRunResult> {
    let manifest = discover_manifest(opts.manifest.as_deref(), opts.fleet_id.as_deref())?;
    let plan = manifest.plan().map_err(|e| anyhow::anyhow!(e))?;
    let fleet_id = manifest.fleet.id.clone();

    if opts.dry_run {
        return Ok(FleetRunResult {
            fleet_run_id: RunId::new().0,
            fleet_id,
            status: "dry-run".into(),
            nodes: plan
                .waves
                .iter()
                .flat_map(|w| {
                    w.nodes.iter().map(|n| FleetNodeResult {
                        node_id: n.id.clone(),
                        domain_id: String::new(),
                        domain_run_id: None,
                        wave: w.wave,
                        status: "planned".into(),
                    })
                })
                .collect(),
        });
    }

    let fleet_run_id = RunId::new().0;
    let started_at = Utc::now().to_rfc3339();
    if let Ok(cat) = Catalog::open_default() {
        let _ = cat.insert_fleet_run(&fleet_run_id, &fleet_id, &started_at);
    }

    let hv = default_hypervisor();
    let mut node_results: Vec<FleetNodeResult> = Vec::new();
    let mut fleet_failed = false;

    for wave in &plan.waves {
        let mut wave_dispatches: Vec<(String, String, String, u32)> = Vec::new();

        for node in &wave.nodes {
            let canon = canonical_repo_root(&node.repo)?;
            ensure_repo_trusted(&canon)?;
            let domain_id = DomainId::from_repo_root(&canon)?;
            let _cfg = load_config(node.config.as_deref(), &canon)?;

            if let Ok(cat) = Catalog::open_default() {
                let _ = cat.insert_fleet_node(
                    &fleet_run_id,
                    &node.id,
                    domain_id.as_str(),
                    None,
                    wave.wave as i32,
                    "dispatching",
                );
            }

            let domain_run_id = hv
                .run_blocking(RunOptions {
                    agents: node.agents,
                    cmd: node.cmd.clone(),
                    config: node.config.clone(),
                    dry_run: false,
                    keep_worktrees: false,
                    repo: Some(node.repo.clone()),
                    execution: None,
                    project: None,
                    tasks: None,
                    task_cmd_template: None,
                    task_prompts: None,
                })
                .await?;

            if let Ok(cat) = Catalog::open_default() {
                let _ = cat.update_fleet_node_status(
                    &fleet_run_id,
                    &node.id,
                    "running",
                    Some(&domain_run_id.0),
                );
            }

            wave_dispatches.push((
                node.id.clone(),
                domain_id.as_str().to_string(),
                domain_run_id.0,
                wave.wave,
            ));
        }

        for (node_id, domain_id, domain_run_id, wave_num) in wave_dispatches {
            let db_path = {
                let canon = manifest
                    .node_by_id(&node_id)
                    .ok_or_else(|| anyhow::anyhow!("node {node_id} missing from manifest"))?;
                let canon_root = canonical_repo_root(&canon.repo)?;
                let cfg = load_config(canon.config.as_deref(), &canon_root)?;
                cfg.db_path_at(&canon_root)
            };

            let status = {
                let store = PytxoStore::open(&db_path)?;
                store
                    .get_run_status(&domain_run_id)?
                    .map(|(s, _)| s)
                    .unwrap_or_else(|| "unknown".into())
            };
            if status == "failed" {
                fleet_failed = true;
            }

            if let Ok(cat) = Catalog::open_default() {
                let _ = cat.update_fleet_node_status(
                    &fleet_run_id,
                    &node_id,
                    &status,
                    Some(&domain_run_id),
                );
            }

            node_results.push(FleetNodeResult {
                node_id,
                domain_id,
                domain_run_id: Some(domain_run_id),
                wave: wave_num,
                status: status.clone(),
            });

            if status == "failed" && !opts.continue_on_error {
                break;
            }
        }

        if fleet_failed && !opts.continue_on_error {
            break;
        }
    }

    let fleet_status = if fleet_failed { "failed" } else { "completed" };
    let finished_at = Utc::now().to_rfc3339();
    if let Ok(cat) = Catalog::open_default() {
        let _ = cat.finish_fleet_run(&fleet_run_id, fleet_status, &finished_at);
    }

    if fleet_failed && !opts.continue_on_error {
        anyhow::bail!("fleet run {fleet_run_id} failed");
    }

    Ok(FleetRunResult {
        fleet_run_id,
        fleet_id,
        status: fleet_status.to_string(),
        nodes: node_results,
    })
}

pub fn fleet_init(
    id: &str,
    name: Option<String>,
    nodes: Vec<(PathBuf, String, usize, Vec<String>)>,
) -> anyhow::Result<PathBuf> {
    if nodes.is_empty() {
        anyhow::bail!("fleet init requires at least one --add <repo> --cmd <command>");
    }
    let manifest = FleetManifest {
        fleet: pytxo_core::FleetMeta {
            id: id.to_string(),
            name,
        },
        node: nodes
            .into_iter()
            .enumerate()
            .map(|(i, (repo, cmd, agents, depends_on))| pytxo_core::FleetNode {
                id: repo
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| format!("node-{i}")),
                repo,
                cmd,
                agents,
                depends_on,
                config: None,
            })
            .collect(),
    };
    manifest.plan().map_err(|e| anyhow::anyhow!(e))?;
    let path = FleetManifest::user_manifest_path(id)
        .ok_or_else(|| anyhow::anyhow!("cannot resolve home directory for fleet manifest"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let toml = toml::to_string_pretty(&manifest)
        .map_err(|e| anyhow::anyhow!("serialize fleet manifest: {e}"))?;
    std::fs::write(&path, toml)?;
    Ok(path)
}

pub fn fleet_run_status(fleet_run_id: &str) -> anyhow::Result<FleetRunStatus> {
    let cat = Catalog::open_default().map_err(|e| anyhow::anyhow!(e))?;
    let runs = cat
        .list_fleet_runs(None, 100)
        .map_err(|e| anyhow::anyhow!(e))?;
    let run = runs
        .into_iter()
        .find(|r| r.id == fleet_run_id)
        .ok_or_else(|| anyhow::anyhow!("fleet run not found: {fleet_run_id}"))?;
    let nodes = fleet_status_nodes(fleet_run_id)?;
    Ok(FleetRunStatus { run, nodes })
}

#[derive(Debug, Clone, Serialize)]
pub struct FleetRunStatus {
    pub run: FleetRunRecord,
    pub nodes: Vec<FleetNodeRecord>,
}

pub fn fleet_status(
    fleet_id: Option<&str>,
    limit: usize,
) -> anyhow::Result<Vec<FleetRunRecord>> {
    let cat = Catalog::open_default().map_err(|e| anyhow::anyhow!(e))?;
    cat.list_fleet_runs(fleet_id, limit)
        .map_err(|e| anyhow::anyhow!(e))
}

pub fn fleet_status_nodes(fleet_run_id: &str) -> anyhow::Result<Vec<FleetNodeRecord>> {
    let cat = Catalog::open_default().map_err(|e| anyhow::anyhow!(e))?;
    cat.list_fleet_nodes(fleet_run_id)
        .map_err(|e| anyhow::anyhow!(e))
}

impl Default for FleetRunOptions {
    fn default() -> Self {
        Self {
            manifest: None,
            fleet_id: None,
            dry_run: false,
            wait_timeout: DEFAULT_WAIT_TIMEOUT,
            continue_on_error: false,
        }
    }
}
