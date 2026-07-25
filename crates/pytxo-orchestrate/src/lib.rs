use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use pytxo_core::{
    canonical_repo_root, DomainId, ExecutionPlan, FidelityTier, PermissionEngine, PytxoConfig,
    PytxoError, RunId, SignalCore, Task, TaskId, TokenWallet, UsageMeter,
};
use pytxo_runner::{execute_plan, stop_all, stop_run, ProcessRegistry, RunContext};
use pytxo_scheduler::build_plan;
use pytxo_signal::TreeSitterSignalCore;
use pytxo_store::{PytxoStore, SharedStore};
use serde::{Deserialize, Serialize};

pub mod billing;
pub mod cloud;
mod cost;
mod dashboard;
mod doctor;
pub mod entitlements;
mod fleet;
pub mod flow;
mod hypervisor;
mod preflight;
mod project;
mod structural;

pub use dashboard::{dashboard_snapshot, dashboard_snapshot_light, DashboardSnapshot};

pub use cloud::{cloud_clients, cloud_health_url, ping_cloud, CloudClients};

pub use cost::{parse_cost_from_lines, CostEstimate};
pub use doctor::{run_doctor, DoctorCheck, DoctorReport};
pub use entitlements::{
    effective_entitlements, fetch_link_wallet_balance, invalidate_entitlements_cache,
    EntitlementStatus,
};
pub use fleet::{
    fleet_dry_run_json, fleet_init, fleet_plan_from_manifest, fleet_run, fleet_run_status,
    fleet_status, fleet_status_nodes, wait_for_domain_run, FleetRunOptions, FleetRunResult,
    FleetRunStatus,
};
pub use flow::{
    dispatch_flow, preview_flow, save_flow_draft, save_reviewed_flow_plan, FlowAdeSummary,
    FlowBlockedReason, FlowDraftInput, FlowPlan, FlowPlanTask, FlowSource, FlowStatus, FlowWarning,
};
pub use hypervisor::{
    default_hypervisor, forget_catalog_domain, list_catalog_domains,
    list_catalog_domains_enriched, CatalogEntryStatus, DomainState, DomainSummary,
    HypervisorRegistry,
};
pub use preflight::assert_git_ready;
pub use project::{
    list_project_manifests, project_add_root, project_init, project_load, project_remove_root,
    project_roots, project_run, project_status, ProjectRunOptions, ProjectRunResult,
    ProjectStatusRow,
};
pub use pytxo_core::ExecutionBackend;
pub use pytxo_store::CatalogEntry;
pub use structural::{structural_graph, workspace_structural_graph};

#[cfg(feature = "sanitize")]
use pytxo_sanitize::sanitize_line;

pub struct RunOptions {
    pub agents: usize,
    pub cmd: String,
    pub config: Option<PathBuf>,
    pub dry_run: bool,
    pub keep_worktrees: bool,
    pub repo: Option<PathBuf>,
    /// Overrides `pytxo.toml` `execution_backend` when set.
    pub execution: Option<pytxo_core::ExecutionBackend>,
    /// Unified multi-root project run ([[ADR-0011-modular-project-manifest]]).
    pub project: Option<ProjectRunContext>,
    /// Runtime task graph from Hypervisor Shell / planner; overrides config tasks when set.
    pub tasks: Option<Vec<Task>>,
    /// Per-agent command template: `{task_id}`, `{agent}`, `{paths}`, `{wave}`. Task prompts are
    /// supplied separately as `PYTXO_TASK_PROMPT`; raw shell interpolation is forbidden.
    pub task_cmd_template: Option<String>,
    /// Per-task prompt text keyed by task id (used with `task_cmd_template`).
    pub task_prompts: Option<std::collections::HashMap<String, String>>,
}

/// Roots and metadata for a single coordinated project run (Phase 20).
#[derive(Clone, Debug)]
pub struct ProjectRunContext {
    pub project_id: String,
    pub roots: std::collections::HashMap<String, pytxo_runner::RootExec>,
    /// Read-only roots merged into agent context (label, canonical path).
    pub readonly_context_roots: Vec<(String, PathBuf)>,
}

#[derive(Serialize, Deserialize)]
struct ActiveRunState {
    run_id: String,
    repo_root: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct StatusJson {
    pub runs: Vec<RunStatusJson>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunStatusJson {
    pub id: String,
    pub status: String,
    pub repo_root: String,
    pub started_at: String,
    pub estimated_tokens_in: Option<i64>,
    pub estimated_tokens_out: Option<i64>,
    pub estimated_cost_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_profile: Option<String>,
    pub isolation_mode: String,
    pub isolation_backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arbitrage_saved_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_balance_microcredits: Option<i64>,
    pub agents: Vec<AgentStatusJson>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentStatusJson {
    pub id: String,
    pub task_id: String,
    pub wave: i32,
    pub status: String,
    pub exit_code: Option<i32>,
}

pub fn init(repo: Option<PathBuf>) -> anyhow::Result<()> {
    let repo = repo.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let cfg = PytxoConfig::default();
    fs::create_dir_all(repo.join(&cfg.worktree_dir))?;
    fs::create_dir_all(repo.join(&cfg.data_dir))?;
    ensure_gitignore(&repo)?;
    Ok(())
}

/// Load `pytxo.toml` from an explicit path or the repo root.
pub fn load_config_for_repo(path: Option<&Path>, repo: &Path) -> anyhow::Result<PytxoConfig> {
    load_config(path, repo)
}

/// Whether the canonical repo root has a trust record ([[ADR-0013]]).
pub fn is_repo_trusted(repo: &Path) -> anyhow::Result<bool> {
    let store = pytxo_core::TrustedDomainStore::open_default().map_err(|e| anyhow::anyhow!(e))?;
    Ok(store.is_trusted(repo))
}

pub fn trust_repo(repo: &Path, profile: pytxo_core::PermissionProfile) -> anyhow::Result<()> {
    let mut store =
        pytxo_core::TrustedDomainStore::open_default().map_err(|e| anyhow::anyhow!(e))?;
    store
        .trust(repo, profile, None)
        .map_err(|e| anyhow::anyhow!(e))
}

pub fn trusted_permission_for(
    repo: &Path,
) -> anyhow::Result<Option<pytxo_core::PermissionProfile>> {
    let store = pytxo_core::TrustedDomainStore::open_default().map_err(|e| anyhow::anyhow!(e))?;
    Ok(store.permission_for(repo))
}

pub(crate) fn ensure_repo_trusted(repo: &Path) -> anyhow::Result<()> {
    if is_repo_trusted(repo)? {
        return Ok(());
    }
    anyhow::bail!(
        "folder is not trusted — run `pytxo` and accept the trust prompt, or use `/trust`"
    )
}

pub fn resolve_repo_root(repo: Option<&Path>) -> anyhow::Result<PathBuf> {
    let repo_root = repo
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    canonical_repo_root(&repo_root).map_err(|e| anyhow::anyhow!(e))
}

pub fn doctor(repo: Option<PathBuf>, json: bool) -> anyhow::Result<()> {
    let report = run_doctor(repo.as_deref())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for c in &report.checks {
            let mark = if c.ok { "ok" } else { "FAIL" };
            println!("[{mark}] {} — {}", c.name, c.detail);
        }
        if report.all_ok() {
            println!("All checks passed.");
        } else {
            println!("Some checks failed. Fix before `pytxo run`.");
        }
    }
    if !report.all_ok() {
        anyhow::bail!("doctor checks failed");
    }
    Ok(())
}

pub async fn run(opts: RunOptions) -> anyhow::Result<RunId> {
    default_hypervisor().run_blocking(opts).await
}

pub fn dispatch(opts: RunOptions) -> anyhow::Result<(DomainId, RunId)> {
    default_hypervisor().dispatch(opts)
}

pub fn list_domains() -> Vec<DomainSummary> {
    default_hypervisor().list_domains()
}

/// Open SQLite telemetry for a domain (`domain_id` is canonical repo root).
pub fn open_store_for_domain(
    domain_id: &str,
    config: Option<PathBuf>,
) -> anyhow::Result<(PytxoConfig, PytxoStore)> {
    let repo = PathBuf::from(domain_id);
    open_store(config, Some(repo))
}

/// Non-blocking multi-project dispatch ([[execution-domains]]).
pub fn dispatch_run(opts: RunOptions) -> anyhow::Result<(String, String)> {
    let (domain, run) = dispatch(opts)?;
    Ok((domain.as_str().to_string(), run.0))
}

/// Dispatch with the exact configuration snapshot already validated by Flow.
pub(crate) fn dispatch_run_with_config_snapshot(
    opts: RunOptions,
    cfg: PytxoConfig,
) -> anyhow::Result<(String, String)> {
    let (domain, run) = default_hypervisor().dispatch_with_config_snapshot(opts, cfg)?;
    Ok((domain.as_str().to_string(), run.0))
}

pub fn enqueue_agent_stdin(
    repo: Option<PathBuf>,
    agent_key: &str,
    data: &[u8],
) -> anyhow::Result<()> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(None, &repo_root)?;
    let domain = default_hypervisor().ensure_domain(&repo_root, &cfg)?;
    ensure_agent_live(&domain.swarm, agent_key)?;
    if cfg.permission_profile == pytxo_core::PermissionProfile::Galaxy {
        if let Ok(text) = std::str::from_utf8(data) {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                pytxo_runner::gate_spawn_command(
                    Some(&domain.hitl),
                    cfg.permission_profile,
                    agent_key,
                    trimmed,
                )
                .map_err(|e| anyhow::anyhow!(e))?;
            }
        }
    }
    domain
        .swarm
        .enqueue_stdin(agent_key, data)
        .map_err(|e| anyhow::anyhow!(e))?;
    audit_mcp_tool(&repo_root, &cfg, agent_key, "pytxo_stdin", "{}")?;
    Ok(())
}

/// Live agent session row for MCP hub v1 ([[mcp-router]]).
#[derive(Clone, Debug, serde::Serialize)]
pub struct LiveAgentSession {
    pub agent_key: String,
    pub run_id: String,
    pub agent_id: String,
    pub paths: Vec<String>,
    pub pid: Option<u32>,
}

fn parse_agent_key(agent_key: &str) -> (String, String) {
    match agent_key.split_once(':') {
        Some((run, agent)) => (run.to_string(), agent.to_string()),
        None => (String::new(), agent_key.to_string()),
    }
}

fn ensure_agent_live(swarm: &pytxo_runner::SwarmRegistry, agent_key: &str) -> anyhow::Result<()> {
    if swarm.list_live().iter().any(|a| a.agent_key == agent_key) {
        return Ok(());
    }
    anyhow::bail!("agent {agent_key} is not live (run may have finished)")
}

/// List agents currently registered in the domain's Race Shield ([[mcp-router]] v1).
pub fn list_live_agents(
    repo: Option<PathBuf>,
    run_id: Option<&str>,
) -> anyhow::Result<Vec<LiveAgentSession>> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(None, &repo_root)?;
    let domain = default_hypervisor().ensure_domain(&repo_root, &cfg)?;
    let mut out: Vec<LiveAgentSession> = domain
        .swarm
        .list_live()
        .into_iter()
        .map(|a| {
            let (run, agent) = parse_agent_key(&a.agent_key);
            LiveAgentSession {
                agent_key: a.agent_key,
                run_id: run,
                agent_id: agent,
                paths: a.paths,
                pid: a.pid,
            }
        })
        .collect();
    if let Some(rid) = run_id {
        out.retain(|s| s.run_id == rid);
    }
    Ok(out)
}

pub fn audit_mcp_tool(
    repo_root: &Path,
    cfg: &PytxoConfig,
    agent_key: &str,
    tool: &str,
    payload: &str,
) -> anyhow::Result<()> {
    let store = PytxoStore::open(&cfg.db_path_at(repo_root))?;
    let body = serde_json::json!({ "tool": tool, "payload": payload }).to_string();
    store.append_event(agent_key, "mcp-tool", &body)?;
    Ok(())
}

/// Forward a JSON-RPC call to a live child MCP session ([[mcp-router]] v2).
pub fn mcp_proxy_call(
    repo: Option<PathBuf>,
    agent_key: &str,
    method: &str,
    params: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(None, &repo_root)?;
    let domain = default_hypervisor().ensure_domain(&repo_root, &cfg)?;
    ensure_agent_live(&domain.swarm, agent_key)?;
    if cfg.permission_profile == pytxo_core::PermissionProfile::Galaxy {
        pytxo_runner::gate_mcp_proxy(
            Some(&domain.hitl),
            cfg.permission_profile,
            agent_key,
            method,
            &params,
        )
        .map_err(|e| anyhow::anyhow!(e))?;
    }
    let result = domain.mcp_hub.proxy_call(agent_key, method, params)?;
    audit_mcp_tool(
        &repo_root,
        &cfg,
        agent_key,
        "pytxo_mcp_proxy",
        &serde_json::json!({ "method": method }).to_string(),
    )?;
    Ok(result)
}

/// Aggregate tools from all live child MCP sessions ([[mcp-router]] v2).
pub fn mcp_tools_list(repo: Option<PathBuf>) -> anyhow::Result<Vec<serde_json::Value>> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(None, &repo_root)?;
    let domain = default_hypervisor().ensure_domain(&repo_root, &cfg)?;
    domain
        .mcp_hub
        .aggregate_tools()
        .map_err(|e| anyhow::anyhow!(e))
}

#[derive(Debug, Clone, Serialize)]
pub struct McpAuditRow {
    pub id: i64,
    pub agent_id: String,
    pub ts: String,
    pub tool: String,
    pub payload: String,
}

/// Export MCP tool audit events for a run from the domain WAL.
pub fn export_mcp_audit(repo: Option<PathBuf>, run_id: &str) -> anyhow::Result<Vec<McpAuditRow>> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(None, &repo_root)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo_root))?;
    let events = store.list_mcp_audit_for_run(run_id)?;
    Ok(events
        .into_iter()
        .map(|e| {
            let tool = serde_json::from_str::<serde_json::Value>(&e.payload)
                .ok()
                .and_then(|v| v.get("tool").and_then(|t| t.as_str()).map(str::to_string))
                .unwrap_or_else(|| "unknown".into());
            McpAuditRow {
                id: e.id,
                agent_id: e.agent_id,
                ts: e.ts.to_rfc3339(),
                tool,
                payload: e.payload,
            }
        })
        .collect())
}

/// Galaxy HITL: list pending approval requests for a domain ([[race-shield]]).
pub fn list_hitl_pending(repo: Option<PathBuf>) -> anyhow::Result<Vec<pytxo_runner::HitlRequest>> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(None, &repo_root)?;
    let domain = default_hypervisor().ensure_domain(&repo_root, &cfg)?;
    Ok(domain.hitl.pending())
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct HitlPendingRow {
    pub domain_id: String,
    pub request: pytxo_runner::HitlRequest,
}

/// Pending HITL across all catalog domains (loads persisted queues from disk).
pub fn list_hitl_pending_all() -> anyhow::Result<Vec<HitlPendingRow>> {
    use pytxo_runner::HitlQueue;
    use pytxo_store::Catalog;

    let mut out = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for d in default_hypervisor().list_domains() {
        if let Some(state) = default_hypervisor().domain_state(&d.domain_id) {
            for req in state.hitl.pending() {
                seen_ids.insert(req.id.clone());
                out.push(HitlPendingRow {
                    domain_id: d.domain_id.clone(),
                    request: req,
                });
            }
        }
    }

    if let Ok(cat) = Catalog::open_default() {
        for entry in cat.list_domains().unwrap_or_default() {
            let repo = PathBuf::from(&entry.repo_root);
            let cfg = load_config(None, &repo).unwrap_or_default();
            let data_dir = repo.join(&cfg.data_dir);
            let q = HitlQueue::with_persistence(&data_dir);
            for req in q.pending() {
                if seen_ids.insert(req.id.clone()) {
                    out.push(HitlPendingRow {
                        domain_id: entry.domain_id.clone(),
                        request: req,
                    });
                }
            }
        }
    }
    Ok(out)
}

/// Galaxy HITL: approve or deny a pending request. Returns whether it was pending.
pub fn hitl_respond(
    repo: Option<PathBuf>,
    request_id: &str,
    approve: bool,
) -> anyhow::Result<bool> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(None, &repo_root)?;
    let domain = default_hypervisor().ensure_domain(&repo_root, &cfg)?;
    Ok(domain.hitl.resolve(request_id, approve))
}

pub fn commit_workspace_for_agent(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    run_id: &str,
    agent_id: &str,
) -> anyhow::Result<()> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo_root)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo_root))?;
    let agent_key = format!("{run_id}:{agent_id}");
    let agent = store
        .get_agent(&agent_key)?
        .ok_or_else(|| anyhow::anyhow!("agent not found: {agent_key}"))?;
    let worktree = agent
        .worktree_path
        .filter(|p| !p.is_empty())
        .ok_or_else(|| anyhow::anyhow!("no worktree for agent"))?;
    let branch = pytxo_runner::branch_name(run_id, agent_id);
    let handle = pytxo_core::WorkspaceHandle {
        cwd: PathBuf::from(worktree),
        branch,
        backend: cfg.isolation,
    };
    let metering = billing::metering_for_ctx(&cfg, &repo_root, &None);
    let ctx = RunContext {
        run_id: RunId(run_id.to_string()),
        repo_root: repo_root.clone(),
        worktree_base: repo_root.join(&cfg.worktree_dir),
        data_dir: repo_root.join(&cfg.data_dir),
        cmd: String::new(),
        task_cmd_template: None,
        task_prompts: std::collections::HashMap::new(),
        keep_worktrees: true,
        on_event: None,
        signal_core: cfg.signal_core,
        signal_fidelity: cfg.signal_fidelity,
        isolation_mode: pytxo_runner::effective_isolation_mode(&cfg),
        permission_profile: cfg.permission_profile,
        agent_profiles: cfg.agent_profile_map(),
        route_agents: cfg.agent.clone(),
        billing_mode: metering.billing_mode,
        domain_id: metering.domain_id,
        model_router: metering.model_router,
        managed_transport: metering.managed_transport,
        usage_meter: metering.usage_meter,
        token_estimator: metering.token_estimator,
        execution_backend: cfg.execution_backend,
        pty_rows: cfg.pty_rows,
        pty_cols: cfg.pty_cols,
        // Manual commit IS the human approval; no queue gate needed here.
        hitl: None,
        hitl_manual_flush: true,
        agent_paths: std::collections::HashMap::new(),
        agent_fidelity: std::collections::HashMap::new(),
        roots: std::collections::HashMap::new(),
        readonly_context_roots: Vec::new(),
        subprocess_stdin: cfg.subprocess_stdin,
        cloud_dispatcher: None,
        context_cache: None,
        cloud_cache_enabled: false,
        cloud_fallback_local: true,
        mcp_hub: None,
        mcp_hub_enabled: false,
        mcp_allowlist: Vec::new(),
        sparse_exclude: cfg.blast.sparse_exclude.clone(),
    };
    pytxo_runner::commit_workspace(&ctx, &handle, cfg.permission_profile)
        .map_err(|e| anyhow::anyhow!(e))
}

pub(crate) async fn execute_run_body(
    domain: Arc<hypervisor::DomainState>,
    opts: RunOptions,
    mut cfg: PytxoConfig,
    run_id: Option<RunId>,
) -> anyhow::Result<RunId> {
    assert_git_ready(&domain.repo_root)?;

    if opts.agents > 0 {
        cfg.max_agents = opts.agents;
    }
    if let Some(exec) = opts.execution {
        cfg.execution_backend = exec;
    }
    let entitlements =
        entitlements::effective_entitlements(&cfg).map_err(|e| anyhow::anyhow!("{e}"))?;
    if cfg.max_agents > entitlements.max_agents {
        anyhow::bail!(
            "max_agents {} exceeds tier limit {} ({})",
            cfg.max_agents,
            entitlements.max_agents,
            entitlements.tier
        );
    }
    cfg.tier_max_agents = entitlements.max_agents;
    let effective_profile = entitlements::effective_permission_profile(&cfg, &entitlements);
    cfg.permission_profile = effective_profile;
    if entitlements.cloud_enabled {
        cfg.cloud.enabled = true;
        if cfg.execution_backend == pytxo_core::ExecutionBackend::Pty {
            cfg.execution_backend = pytxo_core::ExecutionBackend::Cloud;
        }
    }
    if matches!(entitlements.tier.as_str(), "pro" | "max" | "ultra") {
        cfg.cloud.cache_enabled = true;
    }
    if cfg.execution_backend == pytxo_core::ExecutionBackend::Cloud && !entitlements.cloud_enabled {
        anyhow::bail!(
            "cloud execution requires Max or Ultra tier (current: {})",
            entitlements.tier
        );
    }

    let tasks = resolve_run_tasks(&cfg, opts.agents, opts.tasks.clone());

    let plan = plan_tasks(&tasks, &cfg)?;

    if opts.dry_run {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(RunId::new());
    }

    ensure_repo_trusted(&domain.repo_root)?;

    let run_id = run_id.unwrap_or_default();
    let db_path = domain.data_dir.join("pytxo.db");
    let store_for_events = Arc::new(SharedStore::open(&db_path)?);
    {
        let store = store_for_events
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        store.insert_run_with_profile(
            &run_id.0,
            &domain.repo_root.to_string_lossy(),
            Some(cfg.permission_profile.as_str()),
        )?;
        if let Some(proj) = &opts.project {
            store.tag_run_project(&run_id.0, &proj.project_id, None)?;
        }
    }

    let domain_id = DomainId::from_repo_root(&domain.repo_root).map_err(|e| anyhow::anyhow!(e))?;
    let mut ultra =
        billing::setup_ultra_billing(Arc::clone(&store_for_events), &cfg, domain_id.clone())?;
    if let Some(u) = ultra.as_mut() {
        u.hybrid.on_run_start(&u.domain_id, &run_id)?;
        let reserve =
            billing::estimate_reserve_microcredits(&cfg, plan.waves.iter().map(|w| w.len()).sum());
        let res = u.hybrid.wallet.reserve(&u.domain_id, reserve, &run_id)?;
        u.reservation = Some(res);
    }

    let sanitize = cfg.sanitize;
    let store_for_on_event = Arc::clone(&store_for_events);
    let on_event = Arc::new(move |agent_key: &str, kind: &str, line: &str| {
        let payload = if sanitize {
            #[cfg(feature = "sanitize")]
            {
                sanitize_line(line)
            }
            #[cfg(not(feature = "sanitize"))]
            {
                line.to_string()
            }
        } else {
            line.to_string()
        };
        if let Ok(guard) = store_for_on_event.lock() {
            let _ = guard.append_event(agent_key, kind, &payload);
        }
    });

    let agent_profiles = entitlements::effective_agent_profiles(&cfg, &entitlements);
    let metering = billing::metering_for_ctx(&cfg, &domain.repo_root, &ultra);
    let cloud = cloud::cloud_clients(&cfg);
    let ctx = RunContext {
        run_id: run_id.clone(),
        repo_root: domain.repo_root.clone(),
        worktree_base: domain.repo_root.join(&cfg.worktree_dir),
        data_dir: domain.data_dir.clone(),
        cmd: opts.cmd.clone(),
        task_cmd_template: opts.task_cmd_template.clone(),
        task_prompts: opts.task_prompts.clone().unwrap_or_default(),
        keep_worktrees: opts.keep_worktrees,
        on_event: Some(on_event),
        signal_core: cfg.signal_core,
        signal_fidelity: cfg.signal_fidelity,
        isolation_mode: pytxo_runner::effective_isolation_mode(&cfg),
        permission_profile: cfg.permission_profile,
        agent_profiles,
        route_agents: cfg.agent.clone(),
        billing_mode: metering.billing_mode,
        domain_id: metering.domain_id,
        model_router: metering.model_router,
        managed_transport: metering.managed_transport,
        usage_meter: metering.usage_meter,
        token_estimator: metering.token_estimator,
        execution_backend: cfg.execution_backend,
        pty_rows: cfg.pty_rows,
        pty_cols: cfg.pty_cols,
        hitl: Some(domain.hitl.clone()),
        hitl_manual_flush: false,
        agent_paths: cfg
            .agent
            .iter()
            .filter(|a| !a.paths.is_empty())
            .map(|a| (a.name.clone(), a.paths.clone()))
            .collect(),
        agent_fidelity: cfg
            .agent
            .iter()
            .filter_map(|a| a.signal_fidelity.map(|f| (a.name.clone(), f)))
            .collect(),
        roots: opts
            .project
            .as_ref()
            .map(|p| p.roots.clone())
            .unwrap_or_default(),
        readonly_context_roots: opts
            .project
            .as_ref()
            .map(|p| p.readonly_context_roots.clone())
            .unwrap_or_default(),
        subprocess_stdin: cfg.subprocess_stdin,
        cloud_dispatcher: Some(cloud.dispatcher),
        context_cache: Some(cloud.cache),
        cloud_cache_enabled: cfg.cloud.cache_enabled,
        cloud_fallback_local: cfg.cloud.fallback_local,
        mcp_hub: Some(Arc::new(domain.mcp_hub.clone())),
        mcp_hub_enabled: cfg.mcp_hub.enabled,
        mcp_allowlist: cfg.mcp_hub.allowlist.clone(),
        sparse_exclude: cfg.blast.sparse_exclude.clone(),
    };

    save_active_run(&cfg, &run_id, &domain.repo_root).map_err(|e| anyhow::anyhow!(e))?;

    let results = execute_plan(&ctx, &plan, &domain.process_registry, &domain.swarm).await?;

    let mut failed = false;
    let mut all_lines: Vec<String> = Vec::new();
    let store = store_for_events
        .lock()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    for result in &results {
        let agent_key = format!("{}:{}", run_id, result.agent_id);
        store.insert_agent_with_root(
            &agent_key,
            &run_id.0,
            &result.task_id,
            result.wave,
            Some(&result.worktree_path.to_string_lossy()),
            &opts.cmd,
            result.root_id.as_deref(),
        )?;
        if !result.stdout.is_empty() {
            let payload = maybe_sanitize(&result.stdout, sanitize);
            store.append_event(&agent_key, "stdout", &payload)?;
            all_lines.extend(result.stdout.lines().map(String::from));
        }
        if !result.stderr.is_empty() {
            let payload = maybe_sanitize(&result.stderr, sanitize);
            store.append_event(&agent_key, "stderr", &payload)?;
            all_lines.extend(result.stderr.lines().map(String::from));
        }
        let exit = result.exit_code.unwrap_or(-1);
        let status = if exit == 0 { "completed" } else { "failed" };
        if exit != 0 {
            failed = true;
        }
        store.finish_agent(&agent_key, result.exit_code, status)?;
    }

    let line_refs: Vec<&str> = all_lines.iter().map(String::as_str).collect();
    let cost = parse_cost_from_lines(&line_refs);
    if cost.tokens_in > 0 || cost.tokens_out > 0 || cost.cost_usd > 0.0 {
        store.update_run_cost(&run_id.0, cost.tokens_in, cost.tokens_out, cost.cost_usd)?;
    }

    drop(store);

    if let Some(ref mut u) = ultra {
        let totals = u.meter.run_totals(&run_id)?;
        let cost_micro = if totals.cost_micro_usd > 0 {
            totals.cost_micro_usd
        } else {
            (cost.cost_usd * 1_000_000.0) as i64
        };
        billing::settle_ultra_run(u, &run_id, cost_micro)?;
    }

    let run_status = if failed { "failed" } else { "completed" };
    store_for_events
        .lock()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .finish_run(&run_id.0, run_status)?;
    let state_path = domain.data_dir.join("active_run.json");
    let _ = fs::remove_file(state_path);

    if failed && cfg.fail_fast {
        anyhow::bail!("one or more agents failed (fail_fast=true)");
    }

    Ok(run_id)
}

pub fn resolve_run_tasks(
    cfg: &PytxoConfig,
    agents: usize,
    runtime: Option<Vec<Task>>,
) -> Vec<Task> {
    if let Some(tasks) = runtime {
        return tasks;
    }
    if cfg.task.is_empty() {
        let n = if agents > 0 { agents } else { cfg.max_agents };
        synthetic_tasks(n)
    } else {
        cfg.tasks()
    }
}

pub fn plan_tasks(tasks: &[Task], cfg: &PytxoConfig) -> anyhow::Result<ExecutionPlan> {
    let mut plan =
        build_plan(tasks, cfg.max_agents, cfg.dag_explicit_deps).map_err(|e| anyhow::anyhow!(e))?;
    for c in pytxo_scheduler::find_cross_root_conflicts(tasks) {
        let msg = format!(
            "cross-root path overlap: {} vs {} ({})",
            c.task_a.0,
            c.task_b.0,
            c.paths.join(", ")
        );
        if !plan.warnings.contains(&msg) {
            plan.warnings.push(msg);
        }
    }
    Ok(plan)
}

pub fn dry_run_with_tasks(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    agents: usize,
    runtime_tasks: Option<Vec<Task>>,
) -> anyhow::Result<String> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let mut cfg = load_config(config.as_deref(), &repo_root)?;
    if agents > 0 {
        cfg.max_agents = agents;
    }
    let tasks = resolve_run_tasks(&cfg, agents, runtime_tasks);
    let plan = plan_tasks(&tasks, &cfg)?;
    Ok(serde_json::to_string_pretty(&plan)?)
}

pub fn dry_run_json(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    agents: usize,
) -> anyhow::Result<String> {
    dry_run_with_tasks(config, repo, agents, None)
}

pub fn run_status_json(
    run: &pytxo_store::RunRecord,
    store: &PytxoStore,
    cfg: &PytxoConfig,
) -> anyhow::Result<RunStatusJson> {
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
    Ok(RunStatusJson {
        id: run.id.clone(),
        status: run.status.clone(),
        repo_root: run.repo_root.clone(),
        started_at: run.started_at.to_rfc3339(),
        estimated_tokens_in: run.estimated_tokens_in,
        estimated_tokens_out: run.estimated_tokens_out,
        estimated_cost_usd: run.estimated_cost_usd,
        permission_profile: run.permission_profile.clone(),
        isolation_mode: cfg.isolation.as_str().to_string(),
        isolation_backend: pytxo_runner::isolation_backend_label(
            cfg.isolation,
            &cfg.blast.sparse_exclude,
        ),
        arbitrage_saved_tokens: arbitrage_saved,
        wallet_balance_microcredits: wallet_balance,
        agents,
    })
}

pub fn status_json(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    limit: usize,
) -> anyhow::Result<StatusJson> {
    let repo = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo))?;
    let runs = store.list_runs(limit)?;
    let mut out = Vec::new();
    for run in &runs {
        out.push(run_status_json(run, &store, &cfg)?);
    }
    Ok(StatusJson { runs: out })
}

pub fn status(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let repo = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo))?;
    let runs = store.list_runs(limit)?;
    if json {
        let mut out = Vec::new();
        for run in &runs {
            out.push(run_status_json(run, &store, &cfg)?);
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&StatusJson { runs: out })?
        );
        return Ok(());
    }
    if runs.is_empty() {
        println!("No runs recorded.");
        return Ok(());
    }
    for run in runs {
        println!(
            "run {}  status={}  repo={}  started={}  cost_usd={:?}",
            run.id, run.status, run.repo_root, run.started_at, run.estimated_cost_usd
        );
        for agent in store.list_agents_for_run(&run.id)? {
            println!(
                "  agent {}  task={}  wave={}  status={}  exit={:?}",
                agent.id, agent.task_id, agent.wave, agent.status, agent.exit_code
            );
        }
    }
    Ok(())
}

pub fn logs(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    agent: &str,
    tail: usize,
) -> anyhow::Result<Vec<String>> {
    let repo = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo))?;
    let events = store.list_events(agent, tail)?;
    Ok(events
        .into_iter()
        .map(|ev| format!("[{}] {}: {}", ev.ts, ev.kind, ev.payload.trim_end()))
        .collect())
}

pub async fn stop(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    all: bool,
    cleanup_worktrees: bool,
) -> anyhow::Result<()> {
    let repo = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let data_dir = repo.join(&cfg.data_dir);

    if all {
        stop_all(&data_dir, true).map_err(|e| anyhow::anyhow!(e))?;
        let state_path = repo.join(&cfg.data_dir).join("active_run.json");
        if state_path.exists() {
            fs::remove_file(state_path)?;
        }
        println!("Stopped all tracked processes.");
        return Ok(());
    }

    let state_path = repo.join(&cfg.data_dir).join("active_run.json");
    if !state_path.exists() {
        println!("No active run.");
        return Ok(());
    }
    let raw = fs::read_to_string(&state_path)?;
    let state: ActiveRunState = serde_json::from_str(&raw)?;
    let pids = stop_run(&data_dir, &state.run_id, true).map_err(|e| anyhow::anyhow!(e))?;
    if cleanup_worktrees {
        let repo_path = PathBuf::from(&state.repo_root);
        let metering = billing::metering_for_ctx(&cfg, &repo_path, &None);
        let ctx = RunContext {
            run_id: RunId(state.run_id.clone()),
            repo_root: repo_path.clone(),
            worktree_base: repo_path.join(&cfg.worktree_dir),
            data_dir: data_dir.clone(),
            cmd: String::new(),
            task_cmd_template: None,
            task_prompts: std::collections::HashMap::new(),
            keep_worktrees: false,
            on_event: None,
            signal_core: cfg.signal_core,
            signal_fidelity: cfg.signal_fidelity,
            isolation_mode: pytxo_runner::effective_isolation_mode(&cfg),
            permission_profile: cfg.permission_profile,
            agent_profiles: cfg.agent_profile_map(),
            route_agents: cfg.agent.clone(),
            billing_mode: metering.billing_mode,
            domain_id: metering.domain_id,
            model_router: metering.model_router,
            managed_transport: metering.managed_transport,
            usage_meter: metering.usage_meter,
            token_estimator: metering.token_estimator,
            execution_backend: cfg.execution_backend,
            pty_rows: cfg.pty_rows,
            pty_cols: cfg.pty_cols,
            hitl: None,
            hitl_manual_flush: false,
            agent_paths: std::collections::HashMap::new(),
            agent_fidelity: std::collections::HashMap::new(),
            roots: std::collections::HashMap::new(),
            readonly_context_roots: Vec::new(),
            subprocess_stdin: cfg.subprocess_stdin,
            cloud_dispatcher: None,
            context_cache: None,
            cloud_cache_enabled: false,
            cloud_fallback_local: true,
            mcp_hub: None,
            mcp_hub_enabled: false,
            mcp_allowlist: Vec::new(),
            sparse_exclude: cfg.blast.sparse_exclude.clone(),
        };
        let registry = ProcessRegistry::default();
        pytxo_runner::cleanup_worktrees(&ctx, &registry).map_err(|e| anyhow::anyhow!(e))?;
    }
    fs::remove_file(&state_path)?;
    println!(
        "Stopped run {} (killed {} process(es))",
        state.run_id,
        pids.len()
    );
    Ok(())
}

pub fn open_store(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
) -> anyhow::Result<(PytxoConfig, PytxoStore)> {
    let repo = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo))?;
    Ok((cfg, store))
}

/// Signal Core read hook — scaffold a repo-relative or absolute file before MCP/PTY egress.
pub fn read_file_scaffolded(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    rel_path: &str,
    fidelity: Option<FidelityTier>,
) -> anyhow::Result<pytxo_core::ScaffoldResult> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo_root)?;
    let tier = fidelity.unwrap_or(cfg.signal_fidelity);
    let path = repo_root.join(rel_path);
    let engine = PermissionEngine::new(cfg.permission_profile);
    if !engine.may_read(&repo_root, &path, &repo_root) {
        anyhow::bail!(
            "read denied for {} profile (path outside agent cwd)",
            cfg.permission_profile.as_str()
        );
    }
    TreeSitterSignalCore
        .read_scaffolded(&path, tier)
        .map_err(|e| anyhow::anyhow!(e))
}

/// Default MCP read path — routes through Signal Core when `signal_core = true` and fidelity is not high.
pub fn read_file(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    rel_path: &str,
    force_raw: bool,
) -> anyhow::Result<pytxo_core::ScaffoldResult> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo_root)?;
    let path = repo_root.join(rel_path);
    let engine = PermissionEngine::new(cfg.permission_profile);
    if !engine.may_read(&repo_root, &path, &repo_root) {
        anyhow::bail!(
            "read denied for {} profile (path outside agent cwd)",
            cfg.permission_profile.as_str()
        );
    }

    let scaffold = cfg.signal_core && !force_raw && cfg.signal_fidelity != FidelityTier::High;
    if scaffold {
        return TreeSitterSignalCore
            .read_scaffolded(&path, cfg.signal_fidelity)
            .map_err(|e| anyhow::anyhow!(e));
    }

    let source = std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!(e))?;
    let bytes = source.len();
    let content = maybe_sanitize(&source, cfg.sanitize);
    Ok(pytxo_core::ScaffoldResult {
        path: path.display().to_string(),
        content,
        stats: pytxo_core::ScaffoldStats {
            original_bytes: bytes,
            scaffolded_bytes: bytes,
            language: None,
            token_reduction_pct: 0.0,
        },
        fallback_raw: true,
    })
}

fn maybe_sanitize(s: &str, enabled: bool) -> String {
    if !enabled {
        return s.to_string();
    }
    #[cfg(feature = "sanitize")]
    {
        sanitize_line(s)
    }
    #[cfg(not(feature = "sanitize"))]
    {
        s.to_string()
    }
}

pub(crate) fn load_config(path: Option<&Path>, repo: &Path) -> anyhow::Result<PytxoConfig> {
    let path = path.map(|p| p.to_path_buf()).or_else(|| {
        let candidate = repo.join("pytxo.toml");
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    });
    let mut cfg = if let Some(path) = path {
        PytxoConfig::load(&path).map_err(|e| anyhow::anyhow!(e))?
    } else {
        PytxoConfig::default()
    };
    if let Ok(store) = pytxo_core::TrustedDomainStore::open_default() {
        if let Some(profile) = store.permission_for(repo) {
            cfg.permission_profile = profile;
        }
    }
    Ok(cfg)
}

pub(crate) fn synthetic_tasks(count: usize) -> Vec<Task> {
    (0..count)
        .map(|i| Task {
            id: TaskId(format!("synthetic-{i}")),
            agent: "default".into(),
            paths: vec![format!("src/agent-{i}.ts")],
            depends_on: Vec::new(),
            root: None,
            signal_fidelity: None,
        })
        .collect()
}

fn save_active_run(cfg: &PytxoConfig, run_id: &RunId, repo: &Path) -> Result<(), PytxoError> {
    let state = ActiveRunState {
        run_id: run_id.0.clone(),
        repo_root: repo.to_string_lossy().to_string(),
    };
    let json =
        serde_json::to_string_pretty(&state).map_err(|e| PytxoError::Other(e.to_string()))?;
    let path = repo.join(&cfg.data_dir).join("active_run.json");
    fs::write(path, json).map_err(PytxoError::Io)?;
    Ok(())
}

fn ensure_gitignore(repo: &Path) -> anyhow::Result<()> {
    let gi = repo.join(".gitignore");
    let marker = ".pytxo/";
    if gi.exists() {
        let content = fs::read_to_string(&gi)?;
        if content.contains(marker) {
            return Ok(());
        }
        fs::write(gi, format!("{content}\n{marker}\n"))?;
    } else {
        fs::write(gi, format!("{marker}\n"))?;
    }
    Ok(())
}
