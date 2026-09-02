//! Hypervisor fleet DAG orchestration ([[hypervisor-fleet-dag]], [[ADR-0015-hypervisor-fleet-dag]]).

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use pytxo_core::{canonical_repo_root, DomainId, FleetManifest, FleetNode, FleetPlan, RunId};
use pytxo_store::{Catalog, FleetNodeRecord, FleetRunRecord, PytxoStore};
use serde::Serialize;
use tokio::task::JoinSet;
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

struct FleetNodeExecution {
    result: FleetNodeResult,
    error: Option<String>,
}

/// Owns the durable fleet catalog lifecycle. If the future is cancelled or an
/// unexpected error escapes, dropping the guard still settles every unfinished
/// node and the fleet run instead of leaving misleading `running` rows behind.
struct FleetCatalogGuard {
    catalog: Catalog,
    fleet_run_id: String,
    settled: bool,
}

impl FleetCatalogGuard {
    fn start(fleet_run_id: &str, fleet_id: &str, plan: &FleetPlan) -> anyhow::Result<Self> {
        let catalog = Catalog::open_default().map_err(|e| anyhow::anyhow!(e))?;
        catalog
            .insert_fleet_run(fleet_run_id, fleet_id, &Utc::now().to_rfc3339())
            .map_err(|e| anyhow::anyhow!(e))?;

        let guard = Self {
            catalog,
            fleet_run_id: fleet_run_id.to_string(),
            settled: false,
        };
        for wave in &plan.waves {
            for node in &wave.nodes {
                guard
                    .catalog
                    .insert_fleet_node(
                        fleet_run_id,
                        &node.id,
                        "",
                        None,
                        wave.wave as i32,
                        "pending",
                    )
                    .map_err(|e| anyhow::anyhow!(e))?;
            }
        }
        Ok(guard)
    }

    fn record_node(&self, result: &FleetNodeResult) -> anyhow::Result<()> {
        self.catalog
            .insert_fleet_node(
                &self.fleet_run_id,
                &result.node_id,
                &result.domain_id,
                result.domain_run_id.as_deref(),
                result.wave as i32,
                &result.status,
            )
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn settle_unfinished_nodes(&self, status: &str) -> anyhow::Result<()> {
        let nodes = self
            .catalog
            .list_fleet_nodes(&self.fleet_run_id)
            .map_err(|e| anyhow::anyhow!(e))?;
        let mut first_error = None;
        for node in nodes {
            if matches!(
                node.status.as_str(),
                "completed" | "failed" | "cancelled" | "skipped"
            ) {
                continue;
            }
            if let Err(error) = self.catalog.update_fleet_node_status(
                &self.fleet_run_id,
                &node.node_id,
                status,
                node.domain_run_id.as_deref(),
            ) {
                if first_error.is_none() {
                    first_error = Some(anyhow::anyhow!(error));
                }
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn settle(&mut self, fleet_status: &str, unfinished_status: &str) -> anyhow::Result<()> {
        let node_result = self.settle_unfinished_nodes(unfinished_status);
        let fleet_result = self
            .catalog
            .finish_fleet_run(&self.fleet_run_id, fleet_status, &Utc::now().to_rfc3339())
            .map_err(|e| anyhow::anyhow!(e));
        if node_result.is_ok() && fleet_result.is_ok() {
            self.settled = true;
        }
        node_result.and(fleet_result)
    }
}

impl Drop for FleetCatalogGuard {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        let _ = self.settle_unfinished_nodes("failed");
        let _ =
            self.catalog
                .finish_fleet_run(&self.fleet_run_id, "failed", &Utc::now().to_rfc3339());
    }
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
                anyhow::bail!(
                    "timeout waiting for run {run_id} to appear in {}",
                    db_path.display()
                );
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
    let mut catalog = FleetCatalogGuard::start(&fleet_run_id, &fleet_id, &plan)?;
    let mut node_results: Vec<FleetNodeResult> = Vec::new();
    let mut fleet_failed = false;
    let mut failure_details = Vec::new();

    for wave in &plan.waves {
        let mut tasks = JoinSet::new();
        for (index, node) in wave.nodes.iter().cloned().enumerate() {
            let fleet_run_id = fleet_run_id.clone();
            let wait_timeout = opts.wait_timeout;
            let wave_number = wave.wave;
            tasks.spawn(async move {
                (
                    index,
                    run_fleet_node(fleet_run_id, node, wave_number, wait_timeout).await,
                )
            });
        }

        let mut wave_results: Vec<Option<FleetNodeExecution>> =
            (0..wave.nodes.len()).map(|_| None).collect();
        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok((index, execution)) => wave_results[index] = Some(execution),
                Err(error) => {
                    failure_details.push(format!("fleet node task failed to join: {error}"))
                }
            }
        }

        // Join completion order is intentionally discarded. Results and durable
        // writes follow manifest/plan order so repeated fleet runs are stable.
        for (index, execution) in wave_results.into_iter().enumerate() {
            let execution = execution.unwrap_or_else(|| FleetNodeExecution {
                result: FleetNodeResult {
                    node_id: wave.nodes[index].id.clone(),
                    domain_id: String::new(),
                    domain_run_id: None,
                    wave: wave.wave,
                    status: "failed".into(),
                },
                error: Some("fleet node task ended without a result".into()),
            });
            if execution.result.status != "completed" {
                fleet_failed = true;
            }
            if let Some(error) = execution.error {
                failure_details.push(format!("{}: {error}", execution.result.node_id));
            }
            catalog.record_node(&execution.result)?;
            node_results.push(execution.result);
        }

        if fleet_failed && !opts.continue_on_error {
            break;
        }
    }

    let fleet_status = if fleet_failed { "failed" } else { "completed" };
    let unfinished_status = if fleet_failed && !opts.continue_on_error {
        "skipped"
    } else {
        "failed"
    };
    catalog.settle(fleet_status, unfinished_status)?;

    if fleet_failed {
        let detail = if failure_details.is_empty() {
            String::new()
        } else {
            format!(": {}", failure_details.join("; "))
        };
        anyhow::bail!("fleet run {fleet_run_id} failed after durable settlement{detail}");
    }

    Ok(FleetRunResult {
        fleet_run_id,
        fleet_id,
        status: fleet_status.to_string(),
        nodes: node_results,
    })
}

async fn run_fleet_node(
    fleet_run_id: String,
    node: FleetNode,
    wave: u32,
    wait_timeout: Duration,
) -> FleetNodeExecution {
    let mut domain_id = String::new();
    let mut domain_run_id = None;

    let execution = async {
        let canon = canonical_repo_root(&node.repo).map_err(|e| anyhow::anyhow!(e))?;
        ensure_repo_trusted(&canon)?;
        let id = DomainId::from_repo_root(&canon).map_err(|e| anyhow::anyhow!(e))?;
        domain_id = id.as_str().to_string();
        let cfg = load_config(node.config.as_deref(), &canon)?;
        let db_path = cfg.db_path_at(&canon);

        let (_, run_id) = default_hypervisor().dispatch(RunOptions {
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
        })?;
        domain_run_id = Some(run_id.0.clone());

        // This is an intermediate observability update only; the owning fleet
        // task performs the required terminal upsert after every task joins.
        if let Ok(catalog) = Catalog::open_default() {
            let _ = catalog.insert_fleet_node(
                &fleet_run_id,
                &node.id,
                &domain_id,
                domain_run_id.as_deref(),
                wave as i32,
                "running",
            );
        }

        wait_for_domain_run(&db_path, &run_id.0, wait_timeout).await
    }
    .await;

    match execution {
        Ok(status) => {
            let error =
                (status != "completed").then(|| format!("domain run settled with status {status}"));
            FleetNodeExecution {
                result: FleetNodeResult {
                    node_id: node.id,
                    domain_id,
                    domain_run_id,
                    wave,
                    status,
                },
                error,
            }
        }
        Err(error) => FleetNodeExecution {
            result: FleetNodeResult {
                node_id: node.id,
                domain_id,
                domain_run_id,
                wave,
                status: "failed".into(),
            },
            error: Some(format!("{error:#}")),
        },
    }
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
            .map(
                |(i, (repo, cmd, agents, depends_on))| pytxo_core::FleetNode {
                    id: repo
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| format!("node-{i}")),
                    repo,
                    cmd,
                    agents,
                    depends_on,
                    config: None,
                },
            )
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

pub fn fleet_status(fleet_id: Option<&str>, limit: usize) -> anyhow::Result<Vec<FleetRunRecord>> {
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
