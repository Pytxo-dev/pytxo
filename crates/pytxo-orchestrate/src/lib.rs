use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fs2::FileExt;
use pytxo_core::{
    canonical_repo_root, DomainId, ExecutionPlan, FidelityTier, PermissionEngine,
    PermissionProfile, PytxoConfig, PytxoError, RunApplyError, RunId, SignalCore, Task, TaskId,
    TokenWallet, UsageMeter,
};
use pytxo_runner::{
    apply_attempt_ids, apply_prepared_review_under_lease, execute_plan, load_review_package,
    permission_enforcement_receipt, prepare_review_package, reconcile_apply_journals_under_lease,
    registry_path, require_candidate_verification, run_candidate_check, stop_run,
    AgentWorkspaceInput, CandidateCheckContext, CandidateVerification,
    ExecutionDomainMutationLease, PermissionEnforcementReceipt, ProcessRegistryFile,
    RecoveryOutcome, RunApplyManifest, RunContext,
};
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
pub use doctor::{run_doctor, run_doctor_with_tier, DoctorCheck, DoctorReport, DoctorTier};
pub use entitlements::{
    effective_entitlements, fetch_link_wallet_balance, invalidate_entitlements_cache,
    runtime_session_token, set_runtime_session_token, EntitlementStatus,
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
    default_hypervisor, forget_catalog_domain, list_catalog_domains, list_catalog_domains_enriched,
    CatalogEntryStatus, DomainState, DomainSummary, HypervisorRegistry,
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
    #[serde(default)]
    supervisor_pid: u32,
    #[serde(default)]
    supervisor_start_identity: Option<String>,
}

struct RunFinalizer {
    db_path: PathBuf,
    active_path: PathBuf,
    run_id: String,
    settled: bool,
}

impl RunFinalizer {
    fn new(db_path: PathBuf, active_path: PathBuf, run_id: String) -> Self {
        Self {
            db_path,
            active_path,
            run_id,
            settled: false,
        }
    }

    fn settle(&mut self, status: &str) -> anyhow::Result<()> {
        let store = PytxoStore::open(&self.db_path)?;
        let _ = store.finish_run_if_status(&self.run_id, "starting", status)?
            || store.finish_run_if_running(&self.run_id, status)?;
        clear_active_run_if_matches(&self.active_path, &self.run_id)?;
        tracing::info!(run_id = %self.run_id, terminal_status = status, "run settled");
        self.settled = true;
        Ok(())
    }
}

impl Drop for RunFinalizer {
    fn drop(&mut self) {
        if !self.settled {
            if let Ok(store) = PytxoStore::open(&self.db_path) {
                let startup_failed = store
                    .finish_run_if_status(&self.run_id, "starting", "failed_startup")
                    .unwrap_or(false);
                if !startup_failed {
                    let _ = store.finish_run_if_running(&self.run_id, "failed");
                }
            }
            let _ = clear_active_run_if_matches(&self.active_path, &self.run_id);
            tracing::error!(run_id = %self.run_id, terminal_status = "failed", "run settled during error unwind");
        }
    }
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

pub fn doctor(repo: Option<PathBuf>, json: bool, quick: bool) -> anyhow::Result<()> {
    let tier = if quick {
        DoctorTier::Quick
    } else {
        DoctorTier::Full
    };
    let report = run_doctor_with_tier(repo.as_deref(), tier)?;
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
    let effective_profile = live_agent_effective_profile(&repo_root, &cfg, agent_key)?;
    if let Ok(text) = std::str::from_utf8(data) {
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            pytxo_runner::gate_spawn_command(
                Some(&domain.hitl),
                effective_profile,
                agent_key,
                trimmed,
            )
            .map_err(|e| anyhow::anyhow!(e))?;
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

fn live_agent_effective_profile(
    repo_root: &Path,
    cfg: &PytxoConfig,
    agent_key: &str,
) -> anyhow::Result<PermissionProfile> {
    let (run_id, agent_id) = parse_agent_key(agent_key);
    if run_id.is_empty() || agent_id.is_empty() {
        anyhow::bail!("invalid live agent key: {agent_key}");
    }
    let store = PytxoStore::open(&cfg.db_path_at(repo_root))?;
    let contract = store
        .get_run_contract(&run_id)?
        .ok_or_else(|| anyhow::anyhow!("live agent run has no enforcement contract: {run_id}"))?;
    let envelope: RunEnforcementEnvelope =
        serde_json::from_str(contract.enforcement_json.as_deref().ok_or_else(|| {
            anyhow::anyhow!("live agent run has no enforcement receipt: {run_id}")
        })?)
        .map_err(|error| anyhow::anyhow!("invalid live agent enforcement receipt: {error}"))?;

    let receipt = envelope.agents.get(&agent_id).ok_or_else(|| {
        anyhow::anyhow!("live agent has no effective enforcement receipt: {agent_id}")
    })?;
    PermissionProfile::parse(&receipt.effective_profile)
        .ok_or_else(|| anyhow::anyhow!("invalid live agent effective permission profile"))
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
    let effective_profile = live_agent_effective_profile(&repo_root, &cfg, agent_key)?;
    pytxo_runner::gate_mcp_proxy(
        Some(&domain.hitl),
        effective_profile,
        agent_key,
        method,
        &params,
    )
    .map_err(|e| anyhow::anyhow!(e))?;
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
    domain.hitl.try_resolve(request_id, approve)
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
    if agent.exit_code != Some(0) || agent.status != "completed" {
        anyhow::bail!(
            "cannot apply changes: agent {agent_id} status={} exit_code={:?} (verifies must pass)",
            agent.status,
            agent.exit_code
        );
    }
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RunEnforcementEnvelope {
    run: PermissionEnforcementReceipt,
    agents: std::collections::BTreeMap<String, PermissionEnforcementReceipt>,
}

/// Original task authority is keyed by runtime actor ordinal, matching the
/// runner's flattened plan order. Configured agent names may collide with those
/// keys and must never be used as a fallback at the candidate/Apply boundary.
fn original_task_profile(
    enforcement: &RunEnforcementEnvelope,
    actor_index: usize,
) -> anyhow::Result<PermissionProfile> {
    let actor_id = pytxo_core::AgentId::new(actor_index).0;
    let receipt = enforcement.agents.get(&actor_id).ok_or_else(|| {
        anyhow::anyhow!("original task has no runtime enforcement receipt: {actor_id}")
    })?;
    // Both identities came from the saved envelope; preserve its path spelling.
    if enforcement.run.execution_domain.is_empty()
        || receipt.execution_domain != enforcement.run.execution_domain
    {
        anyhow::bail!(
            "original task enforcement receipt belongs to another execution domain: {actor_id}"
        );
    }
    let requested = PermissionProfile::parse(&receipt.requested_profile)
        .ok_or_else(|| anyhow::anyhow!("invalid original requested profile: {actor_id}"))?;
    let effective = PermissionProfile::parse(&receipt.effective_profile)
        .ok_or_else(|| anyhow::anyhow!("invalid original effective profile: {actor_id}"))?;
    if effective.capped_at(requested) != effective {
        anyhow::bail!("original effective profile exceeds its requested authority: {actor_id}");
    }
    Ok(effective)
}

/// Apply one completed run as one reviewed filesystem transaction.
///
/// Scope for v1.1: one execution domain under Orbit or Galaxy. DeepSpace is
/// intentionally non-flushable, Supernova already writes directly to the host,
/// and multi-root runs are rejected instead of pretending to be atomic.
pub fn apply_run_changes(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    run_id: &str,
) -> anyhow::Result<RunApplyManifest> {
    apply_run_changes_with(config, repo, run_id, apply_prepared_review_under_lease)
}

/// Reconcile one reviewed Apply journal and persist its authoritative run-contract state.
///
/// Scope: one execution domain and one repository root. This preserves the reviewed
/// Apply permission boundary established by the run contract; it does not make
/// DeepSpace flushable or alter Supernova's host-direct behavior.
pub fn reconcile_run_recovery(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    run_id: &str,
) -> anyhow::Result<RecoveryOutcome> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo_root)?;
    let data_dir = repo_root.join(&cfg.data_dir);
    let mutation_lease = ExecutionDomainMutationLease::try_acquire(&data_dir)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo_root))?;
    let contract = store
        .get_run_contract(run_id)?
        .ok_or_else(|| anyhow::anyhow!("run has no persisted review/apply contract: {run_id}"))?;
    if !matches!(
        contract.apply_status.as_str(),
        "applying" | "recovery_required"
    ) {
        anyhow::bail!(
            "run recovery cannot be reconciled from status {}",
            contract.apply_status
        );
    }

    let journal_outcome =
        reconcile_apply_journals_under_lease(&repo_root, &data_dir, run_id, &mutation_lease)?;
    let journal_missing = matches!(journal_outcome, RecoveryOutcome::NothingToDo);
    let outcome = if journal_missing {
        RecoveryOutcome::RecoveryRequired {
            attempt_id: contract
                .last_apply_error
                .as_ref()
                .and_then(|error| error.attempt_id.clone()),
        }
    } else {
        journal_outcome
    };
    match &outcome {
        RecoveryOutcome::RolledBack { attempt_id } => store.finish_run_recovery_error(
            run_id,
            "ready",
            &run_apply_error(
                "interrupted_apply",
                "interrupted Apply was rolled back",
                true,
                Some(attempt_id.clone()),
            ),
            Some("rolled_back"),
        )?,
        RecoveryOutcome::RecoveryRequired { attempt_id } => {
            store.finish_run_recovery_error(
                run_id,
                "recovery_required",
                &run_apply_error(
                    if journal_missing {
                        "recovery_journal_missing"
                    } else {
                        "recovery_unprovable"
                    },
                    if journal_missing {
                        "no durable Apply journal was found; recovery cannot be proven"
                    } else {
                        "interrupted Apply could not be proven restored"
                    },
                    false,
                    attempt_id.clone(),
                ),
                Some("unprovable"),
            )?;
        }
        RecoveryOutcome::Committed(manifest) => {
            let audit = serde_json::to_string(manifest)?;
            store.finish_run_recovery_apply(run_id, &audit)?;
        }
        RecoveryOutcome::NothingToDo => unreachable!("NothingToDo is normalized above"),
    }
    Ok(outcome)
}

fn apply_run_changes_with(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    run_id: &str,
    apply: impl FnOnce(
        &Path,
        &Path,
        &pytxo_core::PreparedRunManifest,
        &ExecutionDomainMutationLease,
    ) -> pytxo_core::Result<RunApplyManifest>,
) -> anyhow::Result<RunApplyManifest> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo_root)?;
    let data_dir = repo_root.join(&cfg.data_dir);
    let mutation_lease = ExecutionDomainMutationLease::try_acquire(&data_dir)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo_root))?;
    let run = store
        .get_run(run_id)?
        .ok_or_else(|| anyhow::anyhow!("run not found: {run_id}"))?;
    let mut contract = store
        .get_run_contract(run_id)?
        .ok_or_else(|| anyhow::anyhow!("run has no persisted review/apply contract: {run_id}"))?;
    match reconcile_apply_journals_under_lease(&repo_root, &data_dir, run_id, &mutation_lease)? {
        RecoveryOutcome::RolledBack { attempt_id } if contract.apply_status == "applying" => {
            store.finish_run_apply_error(
                run_id,
                "ready",
                &run_apply_error(
                    "interrupted_apply",
                    "interrupted Apply was rolled back",
                    true,
                    Some(attempt_id),
                ),
                Some("rolled_back"),
            )?;
            contract = store.get_run_contract(run_id)?.expect("contract exists");
        }
        RecoveryOutcome::RecoveryRequired { attempt_id } if contract.apply_status == "applying" => {
            store.finish_run_apply_error(
                run_id,
                "recovery_required",
                &run_apply_error(
                    "recovery_unprovable",
                    "interrupted Apply could not be proven restored",
                    false,
                    attempt_id,
                ),
                Some("unprovable"),
            )?;
            contract = store.get_run_contract(run_id)?.expect("contract exists");
        }
        RecoveryOutcome::Committed(manifest) if contract.apply_status == "applying" => {
            let audit = serde_json::to_string(&manifest)?;
            store.finish_run_apply(run_id, "applied", Some(&audit))?;
            return Ok(manifest);
        }
        RecoveryOutcome::NothingToDo if contract.apply_status == "applying" => {
            store.finish_run_apply_error(
                run_id,
                "recovery_required",
                &run_apply_error(
                    "recovery_journal_missing",
                    "no durable Apply journal was found; recovery cannot be proven",
                    false,
                    contract
                        .last_apply_error
                        .as_ref()
                        .and_then(|error| error.attempt_id.clone()),
                ),
                Some("unprovable"),
            )?;
            contract = store.get_run_contract(run_id)?.expect("contract exists");
        }
        _ => {}
    }
    if contract.apply_status != "ready" {
        anyhow::bail!(
            "run apply is not ready: {run_id} status={}",
            contract.apply_status
        );
    }
    if run.status != "completed" {
        anyhow::bail!("run apply is not ready: {run_id} run_status={}", run.status);
    }
    let stored_root = canonical_repo_root(Path::new(&run.repo_root))?;
    if stored_root != canonical_repo_root(&repo_root)? {
        anyhow::bail!(
            "run belongs to a different execution domain: {}",
            run.repo_root
        );
    }
    let profile = run
        .permission_profile
        .as_deref()
        .and_then(PermissionProfile::parse)
        .ok_or_else(|| anyhow::anyhow!("run has no valid effective permission profile"))?;
    match profile {
        PermissionProfile::Orbit | PermissionProfile::Galaxy => {}
        PermissionProfile::DeepSpace => {
            anyhow::bail!("DeepSpace runs are non-flushable by policy")
        }
        PermissionProfile::Supernova => {
            anyhow::bail!("Supernova runs write directly to the host and have no apply step")
        }
    }

    let plan: ExecutionPlan = serde_json::from_str(
        contract
            .plan_json
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("run contract has no execution plan"))?,
    )?;
    let tasks: Vec<_> = plan.waves.iter().flatten().collect();
    let enforcement: RunEnforcementEnvelope = serde_json::from_str(
        contract
            .enforcement_json
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("run contract has no enforcement receipt"))?,
    )
    .map_err(|error| anyhow::anyhow!("invalid run enforcement receipt: {error}"))?;
    for (actor_index, task) in tasks.iter().enumerate() {
        let task_profile = original_task_profile(&enforcement, actor_index)?;
        if !matches!(
            task_profile,
            PermissionProfile::Orbit | PermissionProfile::Galaxy
        ) {
            anyhow::bail!(
                "task {} uses {} and cannot enter reviewed run Apply",
                task.task_id.0,
                task_profile.as_str()
            );
        }
    }
    if tasks
        .iter()
        .any(|task| task.root.as_deref().is_some_and(|root| !root.is_empty()))
    {
        anyhow::bail!("multi-root run apply is not atomic in v1.1 and was rejected");
    }

    if !store.claim_run_apply(run_id)? {
        anyhow::bail!("run apply is not ready or is already being applied: {run_id}");
    }
    let prepared = match load_review_package(&data_dir, run_id) {
        Ok(manifest) => manifest,
        Err(error) => {
            let error_code = if error.to_string().contains("package version 1") {
                "review_package_upgrade_required"
            } else {
                "review_package_invalid"
            };
            store.finish_run_apply_error(
                run_id,
                "review_failed",
                &run_apply_error(error_code, &error.to_string(), false, None),
                None,
            )?;
            return Err(anyhow::anyhow!(error));
        }
    };
    if contract.prepared_digest.as_deref() != Some(&prepared.package_digest) {
        let error =
            PytxoError::Runner("durable review package does not match the claimed contract".into());
        store.finish_run_apply_error(
            run_id,
            "review_failed",
            &run_apply_error("review_package_mismatch", &error.to_string(), false, None),
            None,
        )?;
        return Err(anyhow::anyhow!(error));
    }
    if !prepared.files.is_empty() {
        if let Err(error) = validate_candidate_recipe(
            &prepared,
            &plan,
            &enforcement,
            &DomainId::from_repo_root(&repo_root)?,
        ) {
            store.finish_run_apply_error(
                run_id,
                "review_failed",
                &run_apply_error(
                    "candidate_verification_required",
                    &error.to_string(),
                    false,
                    None,
                ),
                None,
            )?;
            return Err(anyhow::anyhow!(error));
        }
    }
    let previous_attempts = apply_attempt_ids(&data_dir, run_id)?;
    let result = apply(&repo_root, &data_dir, &prepared, &mutation_lease);
    match result {
        Ok(manifest) => {
            let manifest_json = serde_json::to_string(&manifest)?;
            store.finish_run_apply(run_id, "applied", Some(&manifest_json))?;
            if let Ok(workspaces) = review_workspaces(&store, run_id, &plan) {
                cleanup_preserved_workspaces(&repo_root, run_id, &workspaces);
            }
            Ok(manifest)
        }
        Err(error) => {
            let stale = error.to_string().contains("changed since review")
                || error
                    .to_string()
                    .contains("candidate base inventory drifted");
            let current_attempts = apply_attempt_ids(&data_dir, run_id)?;
            let new_attempts = current_attempts
                .difference(&previous_attempts)
                .cloned()
                .collect::<Vec<_>>();
            let new_attempt = (new_attempts.len() == 1).then(|| new_attempts[0].clone());
            let recovery = (!new_attempts.is_empty())
                .then(|| {
                    reconcile_apply_journals_under_lease(
                        &repo_root,
                        &data_dir,
                        run_id,
                        &mutation_lease,
                    )
                })
                .transpose()?;
            if let Some(RecoveryOutcome::Committed(manifest)) = recovery {
                let audit = serde_json::to_string(&manifest)?;
                store.finish_run_apply(run_id, "applied", Some(&audit))?;
                return Ok(manifest);
            }
            let rollback_confirmed = matches!(
                recovery,
                Some(RecoveryOutcome::RolledBack { .. } | RecoveryOutcome::NothingToDo)
            ) || error.to_string().contains("was rolled back");
            let recovery_required =
                matches!(recovery, Some(RecoveryOutcome::RecoveryRequired { .. }));
            let attempt_id = match &recovery {
                Some(RecoveryOutcome::RolledBack { attempt_id }) => Some(attempt_id.clone()),
                Some(RecoveryOutcome::RecoveryRequired { attempt_id }) => attempt_id.clone(),
                _ => new_attempt,
            };
            let (status, code, recovery_state) =
                classify_apply_failure(stale, recovery_required, rollback_confirmed);
            store.finish_run_apply_error(
                run_id,
                status,
                &run_apply_error(code, &error.to_string(), rollback_confirmed, attempt_id),
                recovery_state,
            )?;
            Err(anyhow::anyhow!(error))
        }
    }
}

fn classify_apply_failure(
    stale: bool,
    recovery_required: bool,
    rollback_confirmed: bool,
) -> (&'static str, &'static str, Option<&'static str>) {
    if recovery_required {
        (
            "recovery_required",
            "apply_recovery_required",
            Some("unprovable"),
        )
    } else if stale {
        ("stale", "source_drift", Some("source_drift"))
    } else if rollback_confirmed {
        ("ready", "apply_failed", Some("rolled_back"))
    } else {
        ("review_failed", "review_package_invalid", None)
    }
}

pub fn refresh_run_review(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    run_id: &str,
) -> anyhow::Result<pytxo_core::PreparedRunManifest> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo_root)?;
    let data_dir = repo_root.join(&cfg.data_dir);
    let _mutation_lease = ExecutionDomainMutationLease::try_acquire(&data_dir)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo_root))?;
    let contract = store
        .get_run_contract(run_id)?
        .ok_or_else(|| anyhow::anyhow!("run has no persisted review contract: {run_id}"))?;
    if !matches!(contract.apply_status.as_str(), "stale" | "review_failed") {
        anyhow::bail!(
            "run review cannot be refreshed from status {}",
            contract.apply_status
        );
    }
    if contract
        .last_apply_error
        .as_ref()
        .is_some_and(|error| error.code == "event_persistence_failed")
    {
        anyhow::bail!(
            "run evidence is incomplete; refresh cannot restore lost events; rerun the mission"
        );
    }
    let run = store
        .get_run(run_id)?
        .ok_or_else(|| anyhow::anyhow!("unknown run: {run_id}"))?;
    if !matches!(run.status.as_str(), "completed" | "failed") {
        anyhow::bail!(
            "run review cannot be refreshed from run status {}",
            run.status
        );
    }
    if canonical_repo_root(Path::new(&run.repo_root))? != canonical_repo_root(&repo_root)? {
        anyhow::bail!("run belongs to a different execution domain");
    }
    let plan: ExecutionPlan = serde_json::from_str(
        contract
            .plan_json
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("run contract has no execution plan"))?,
    )?;
    let workspaces = review_workspaces(&store, run_id, &plan)?;
    let enforcement: RunEnforcementEnvelope = serde_json::from_str(
        contract
            .enforcement_json
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("run has no enforcement receipt"))?,
    )?;
    let domain = default_hypervisor().ensure_domain(&repo_root, &cfg)?;
    if !store.begin_run_preparation(run_id)? {
        anyhow::bail!("run review could not enter preparing state");
    }
    let base_revision = current_head_revision(&repo_root)?;
    let preparation = match load_review_package(&data_dir, run_id) {
        Ok(frozen) => pytxo_runner::refresh_frozen_review_package(
            &repo_root,
            &data_dir,
            &frozen,
            &base_revision,
            &cfg.blast.sparse_exclude,
        ),
        Err(_) => prepare_review_package(
            &repo_root,
            &data_dir,
            run_id,
            &base_revision,
            &workspaces,
            &cfg.blast.sparse_exclude,
        ),
    }
    .map_err(anyhow::Error::from)
    .and_then(|manifest| verify_combined_candidate(&domain, &cfg, &plan, &enforcement, manifest));
    match preparation {
        Ok(manifest) => {
            persist_finished_run_review(&store, run_id, &manifest)?;
            // review_workspaces has required every planned task to have completed
            // successfully. A failed review can now recover after explicit fresh
            // verification, without ever reviving a cancelled or stopped run.
            if run.status == "failed"
                && !store.finish_run_if_status(run_id, "failed", "completed")?
            {
                anyhow::bail!("run changed status while refreshing its review");
            }
            Ok(manifest)
        }
        Err(error) => {
            let apply_error =
                run_apply_error("review_refresh_failed", &error.to_string(), false, None);
            let _ = store.fail_run_preparation(run_id, &apply_error);
            Err(anyhow::anyhow!(error))
        }
    }
}

/// Blocking repository verification boundary. Scope: original Orbit/Galaxy
/// task authority, capped by current folder trust, in one execution domain.
fn verify_combined_candidate(
    domain: &hypervisor::DomainState,
    cfg: &PytxoConfig,
    plan: &ExecutionPlan,
    enforcement: &RunEnforcementEnvelope,
    manifest: pytxo_core::PreparedRunManifest,
) -> anyhow::Result<pytxo_core::PreparedRunManifest> {
    if manifest.files.is_empty() {
        return Ok(manifest);
    }
    let candidate = CandidateVerification::prepare(
        &domain.repo_root,
        &domain.data_dir,
        &manifest,
        &cfg.blast.sparse_exclude,
    )?;
    let mut checks = Vec::new();
    let actors = PytxoStore::open(&cfg.db_path_at(&domain.repo_root))?
        .list_agents_for_run(&manifest.run_id)?;
    for (actor_index, task) in plan.waves.iter().flatten().enumerate() {
        if task.root.as_deref().is_some_and(|root| !root.is_empty()) {
            anyhow::bail!("combined candidate verification supports one repository root");
        }
        if task.verify.is_empty() || task.verify.iter().any(|command| command.trim().is_empty()) {
            anyhow::bail!("task {} has no complete verification recipe; configure checks and rerun the mission", task.task_id.0);
        }
        let original_profile = original_task_profile(enforcement, actor_index)?;
        let profile = original_profile.capped_at(cfg.resolve_profile_for_agent(&task.agent));
        if !matches!(
            profile,
            PermissionProfile::Orbit | PermissionProfile::Galaxy
        ) {
            anyhow::bail!(
                "candidate verification cannot run under {}",
                profile.as_str()
            );
        }
        let actor = actors
            .iter()
            .find(|actor| {
                actor.task_id == task.task_id.0
                    && actor.status == "completed"
                    && actor.exit_code == Some(0)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "candidate task has no completed audit actor: {}",
                    task.task_id.0
                )
            })?;
        let context = CandidateCheckContext {
            cwd: candidate.workspace_root().to_path_buf(),
            run_id: manifest.run_id.clone(),
            agent_key: actor.id.clone(),
            repo_root: domain.repo_root.clone(),
            data_dir: domain.data_dir.clone(),
            profile,
            domain_id: domain.id.clone(),
            execution_backend: cfg.execution_backend,
            workspace_isolated: true,
            hitl: Some(domain.hitl.clone()),
            swarm: domain.swarm.clone(),
            on_event: None,
        };
        for command in &task.verify {
            let result = run_candidate_check(&context, command);
            candidate.check_unchanged()?;
            let receipt = result?;
            checks.push(pytxo_core::CandidateCheckEvidence {
                task_id: task.task_id.0.clone(),
                command: command.clone(),
                effective_profile: profile.as_str().into(),
                passed: true,
                enforcement: serde_json::to_value(receipt)?,
            });
        }
    }
    Ok(candidate.finish(checks)?)
}

fn validate_candidate_recipe(
    manifest: &pytxo_core::PreparedRunManifest,
    plan: &ExecutionPlan,
    enforcement: &RunEnforcementEnvelope,
    domain_id: &DomainId,
) -> anyhow::Result<()> {
    let evidence = require_candidate_verification(manifest)?;
    let expected: Vec<_> = plan
        .waves
        .iter()
        .flatten()
        .enumerate()
        .flat_map(|(actor_index, task)| {
            task.verify
                .iter()
                .map(move |command| (actor_index, task, command))
        })
        .collect();
    if expected.len() != evidence.checks.len() {
        anyhow::bail!("candidate verification recipe differs from the approved plan");
    }
    for ((actor_index, task, command), check) in expected.into_iter().zip(&evidence.checks) {
        let original_profile = original_task_profile(enforcement, actor_index)?;
        let check_profile = PermissionProfile::parse(&check.effective_profile)
            .ok_or_else(|| anyhow::anyhow!("invalid candidate check profile"))?;
        if check.task_id != task.task_id.0
            || check.command != *command
            || check_profile.capped_at(original_profile) != check_profile
            || check.enforcement["effective_profile"].as_str()
                != Some(check.effective_profile.as_str())
            || check.enforcement["execution_domain"].as_str() != Some(domain_id.as_str())
        {
            anyhow::bail!(
                "candidate verification identity or authority differs from the approved task {}",
                task.task_id.0
            );
        }
    }
    Ok(())
}

pub fn discard_run_review(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    run_id: &str,
) -> anyhow::Result<()> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo_root)?;
    let store = PytxoStore::open(&cfg.db_path_at(&repo_root))?;
    let contract = store
        .get_run_contract(run_id)?
        .ok_or_else(|| anyhow::anyhow!("run has no persisted review contract: {run_id}"))?;
    let workspaces = contract
        .plan_json
        .as_deref()
        .and_then(|json| serde_json::from_str::<ExecutionPlan>(json).ok())
        .and_then(|plan| review_workspaces(&store, run_id, &plan).ok());
    if !store.discard_run_review(run_id)? {
        anyhow::bail!("run review cannot be discarded from its current state: {run_id}");
    }
    let package = repo_root.join(&cfg.data_dir).join("reviews").join(run_id);
    if package.exists() {
        fs::remove_dir_all(package)?;
    }
    if let Some(workspaces) = workspaces {
        cleanup_preserved_workspaces(&repo_root, run_id, &workspaces);
    }
    Ok(())
}

fn review_workspaces(
    store: &PytxoStore,
    run_id: &str,
    plan: &ExecutionPlan,
) -> anyhow::Result<Vec<AgentWorkspaceInput>> {
    let tasks: std::collections::BTreeMap<_, _> = plan
        .waves
        .iter()
        .flatten()
        .map(|task| (task.task_id.0.as_str(), task))
        .collect();
    let agents = store.list_agents_for_run(run_id)?;
    if agents.len() != tasks.len() {
        anyhow::bail!("run review workspace count does not match its task plan");
    }
    agents
        .into_iter()
        .map(|agent| {
            if agent.status != "completed" || agent.exit_code != Some(0) {
                anyhow::bail!("agent result is not reviewable: {}", agent.id);
            }
            let task = tasks
                .get(agent.task_id.as_str())
                .ok_or_else(|| anyhow::anyhow!("unknown review task: {}", agent.task_id))?;
            Ok(AgentWorkspaceInput {
                agent_id: agent
                    .id
                    .strip_prefix(&format!("{run_id}:"))
                    .unwrap_or(&agent.id)
                    .to_owned(),
                task_id: agent.task_id,
                workspace_path: agent
                    .worktree_path
                    .filter(|path| !path.is_empty())
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("agent has no preserved workspace"))?,
                claims: task.paths.clone(),
                depends_on: task.depends_on.clone(),
            })
        })
        .collect()
}

fn run_apply_error(
    code: &str,
    message: &str,
    rollback_confirmed: bool,
    attempt_id: Option<String>,
) -> RunApplyError {
    RunApplyError {
        at: chrono::Utc::now().to_rfc3339(),
        code: code.to_owned(),
        message: message.to_owned(),
        attempt_id,
        rollback_confirmed,
    }
}

fn persist_finished_run_review(
    store: &PytxoStore,
    run_id: &str,
    manifest: &pytxo_core::PreparedRunManifest,
) -> anyhow::Result<()> {
    match store.finish_run_preparation(run_id, manifest) {
        Ok(()) => Ok(()),
        Err(error) => {
            let original = error.to_string();
            let apply_error = run_apply_error("review_persistence_failed", &original, false, None);
            match store.fail_run_preparation(run_id, &apply_error) {
                Ok(()) => Err(anyhow::anyhow!(original)),
                Err(settle_error) => Err(anyhow::anyhow!(
                    "{original}; additionally failed to settle review contract: {settle_error}"
                )),
            }
        }
    }
}

fn reconcile_domain_apply_journals(
    repo_root: &Path,
    data_dir: &Path,
    store: &PytxoStore,
) -> anyhow::Result<()> {
    let mutation_lease = ExecutionDomainMutationLease::try_acquire(data_dir)?;
    let apply_root = data_dir.join("apply");
    let mut run_ids = store
        .list_run_contract_ids_with_status("applying")?
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    for entry in fs::read_dir(apply_root)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        run_ids.insert(entry.file_name().to_string_lossy().to_string());
    }
    for run_id in run_ids {
        let Some(contract) = store.get_run_contract(&run_id)? else {
            continue;
        };
        if contract.apply_status != "applying" {
            continue;
        }
        match reconcile_apply_journals_under_lease(repo_root, data_dir, &run_id, &mutation_lease)? {
            RecoveryOutcome::RolledBack { attempt_id } => store.finish_run_apply_error(
                &run_id,
                "ready",
                &run_apply_error(
                    "interrupted_apply",
                    "interrupted Apply was rolled back",
                    true,
                    Some(attempt_id),
                ),
                Some("rolled_back"),
            )?,
            RecoveryOutcome::Committed(manifest) => {
                let audit = serde_json::to_string(&manifest)?;
                store.finish_run_apply(&run_id, "applied", Some(&audit))?;
            }
            RecoveryOutcome::RecoveryRequired { attempt_id } => store.finish_run_apply_error(
                &run_id,
                "recovery_required",
                &run_apply_error(
                    "recovery_unprovable",
                    "interrupted Apply could not be proven restored",
                    false,
                    attempt_id,
                ),
                Some("unprovable"),
            )?,
            RecoveryOutcome::NothingToDo => store.finish_run_apply_error(
                &run_id,
                "recovery_required",
                &run_apply_error(
                    "recovery_journal_missing",
                    "no durable Apply journal was found; recovery cannot be proven",
                    false,
                    contract
                        .last_apply_error
                        .as_ref()
                        .and_then(|error| error.attempt_id.clone()),
                ),
                Some("unprovable"),
            )?,
        }
    }
    Ok(())
}

fn cleanup_preserved_workspaces(
    repo_root: &Path,
    run_id: &str,
    workspaces: &[AgentWorkspaceInput],
) {
    for workspace in workspaces {
        let path = &workspace.workspace_path;
        let cleanup = if path.join(".git").exists() {
            let branch = pytxo_runner::branch_name(run_id, &workspace.agent_id);
            pytxo_runner::remove_worktree(repo_root, path, &branch, true)
        } else if path.exists() {
            fs::remove_dir_all(path).map_err(PytxoError::Io)
        } else {
            Ok(())
        };
        if let Err(error) = cleanup {
            tracing::warn!(
                run_id,
                workspace = %path.display(),
                %error,
                "durable review prepared but workspace cleanup failed"
            );
        }
    }
}

fn current_head_revision(repo_root: &Path) -> anyhow::Result<String> {
    let output = pytxo_core::background_command("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse HEAD failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn assert_clean_primary_checkout(repo_root: &Path) -> anyhow::Result<()> {
    let output = pytxo_core::background_command("git")
        .args([
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            ".",
            ":(exclude).pytxo",
            ":(exclude).pytxo/**",
        ])
        .current_dir(repo_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let dirty = String::from_utf8_lossy(&output.stdout);
    if !dirty.trim().is_empty() {
        let paths = dirty
            .lines()
            .take(8)
            .map(|line| line.get(3..).unwrap_or(line))
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!(
            "uncommitted changes block trustworthy run apply: {paths}. Commit or stash them, then dispatch/review again"
        );
    }
    Ok(())
}

pub(crate) async fn execute_run_body(
    domain: Arc<hypervisor::DomainState>,
    mut opts: RunOptions,
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
    let requested_profile = cfg
        .requested_permission_profile
        .unwrap_or(cfg.permission_profile);
    let requested_agent_profiles = cfg.requested_agent_profile_map();
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
    if let Some(project) = opts.project.as_mut() {
        for root in project.roots.values_mut() {
            if let Some(ceiling) = cfg.permission_ceiling {
                root.permission_profile = root.permission_profile.capped_at(ceiling);
            }
            if let Some(ceiling) = entitlements.permission_ceiling {
                root.permission_profile = root.permission_profile.capped_at(ceiling);
            }
        }
    }
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
        eprintln!("Isolated copies until you approve (Blast).");
        eprintln!(
            "Overlapping paths wait in later stages (Race): {} stage(s), {} conflict pair(s).",
            plan.waves.len(),
            plan.conflicts.len()
        );
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(RunId::new());
    }

    ensure_repo_trusted(&domain.repo_root)?;

    let was_reserved = run_id.is_some();
    let run_id = run_id.unwrap_or_default();
    if !was_reserved {
        reserve_active_run(&domain, &cfg, &run_id)?;
    }
    let db_path = domain.data_dir.join("pytxo.db");
    let mut run_finalizer = RunFinalizer::new(
        db_path.clone(),
        domain.data_dir.join("active_run.json"),
        run_id.0.clone(),
    );
    let domain_id = DomainId::from_repo_root(&domain.repo_root).map_err(|e| anyhow::anyhow!(e))?;
    let isolation_mode = pytxo_runner::effective_isolation_mode(&cfg);
    let agent_profiles = entitlements::effective_agent_profiles(&cfg, &entitlements);
    let mut agent_enforcement = agent_profiles
        .iter()
        .map(|(name, effective)| {
            let requested = requested_agent_profiles
                .get(name)
                .copied()
                .unwrap_or(requested_profile);
            permission_enforcement_receipt(
                requested,
                *effective,
                &domain_id,
                isolation_mode,
                &cfg.blast.sparse_exclude,
            )
            .map(|receipt| (name.clone(), receipt))
        })
        .collect::<pytxo_core::Result<std::collections::BTreeMap<_, _>>>()
        .map_err(|error| anyhow::anyhow!(error))?;
    for (index, task) in plan.waves.iter().flatten().enumerate() {
        let (requested, effective) = task
            .root
            .as_deref()
            .and_then(|label| opts.project.as_ref()?.roots.get(label))
            .map(|root| (root.requested_permission_profile, root.permission_profile))
            .unwrap_or_else(|| {
                (
                    requested_agent_profiles
                        .get(&task.agent)
                        .copied()
                        .unwrap_or(requested_profile),
                    agent_profiles
                        .get(&task.agent)
                        .copied()
                        .unwrap_or(cfg.permission_profile),
                )
            });
        let receipt = permission_enforcement_receipt(
            requested,
            effective,
            &domain_id,
            isolation_mode,
            &cfg.blast.sparse_exclude,
        )
        .map_err(|error| anyhow::anyhow!(error))?;
        agent_enforcement.insert(format!("agent-{index}"), receipt);
    }
    let enforcement = RunEnforcementEnvelope {
        run: permission_enforcement_receipt(
            requested_profile,
            cfg.permission_profile,
            &domain_id,
            isolation_mode,
            &cfg.blast.sparse_exclude,
        )
        .map_err(|error| anyhow::anyhow!(error))?,
        agents: agent_enforcement,
    };
    let task_profiles: Vec<_> = plan
        .waves
        .iter()
        .flatten()
        .map(|task| {
            task.root
                .as_deref()
                .and_then(|label| opts.project.as_ref()?.roots.get(label))
                .map(|root| root.permission_profile)
                .or_else(|| agent_profiles.get(&task.agent).copied())
                .unwrap_or(cfg.permission_profile)
        })
        .collect();
    let apply_status = if plan
        .waves
        .iter()
        .flatten()
        .any(|task| task.root.as_deref().is_some_and(|root| !root.is_empty()))
    {
        "unsupported"
    } else if task_profiles.contains(&PermissionProfile::DeepSpace) {
        "non_flushable"
    } else if task_profiles.contains(&PermissionProfile::Supernova) {
        "not_applicable"
    } else {
        "pending"
    };
    let store_for_events = Arc::new(SharedStore::open(&db_path)?);
    {
        let store = store_for_events
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        reconcile_domain_apply_journals(&domain.repo_root, &domain.data_dir, &store)?;
    }
    let base_revision = if apply_status == "pending" {
        assert_clean_primary_checkout(&domain.repo_root)?;
        Some(current_head_revision(&domain.repo_root)?)
    } else {
        current_head_revision(&domain.repo_root).ok()
    };
    let plan_json = serde_json::to_string(&plan)?;
    let enforcement_json = serde_json::to_string(&enforcement)?;
    {
        let store = store_for_events
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        if !store.mark_run_running(&run_id.0)? {
            anyhow::bail!("run {} is no longer eligible to start", run_id.0);
        }
        store.save_run_contract_with_status(
            &run_id.0,
            base_revision.as_deref(),
            &plan_json,
            &enforcement_json,
            apply_status,
        )?;
        if let Some(proj) = &opts.project {
            store.tag_run_project(&run_id.0, &proj.project_id, None)?;
        }
    }

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
    let lost_events = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let lost_events_for_callback = Arc::clone(&lost_events);
    let event_tasks: std::collections::BTreeMap<_, _> = plan
        .waves
        .iter()
        .flatten()
        .enumerate()
        .map(|(index, task)| {
            (
                format!("{}:{}", run_id, pytxo_core::AgentId::new(index)),
                task.clone(),
            )
        })
        .collect();
    let event_run_id = run_id.0.clone();
    let event_command = opts.cmd.clone();
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
        let persisted = store_for_on_event.lock().is_ok_and(|guard| {
            let write = || -> pytxo_core::Result<()> {
                let task = event_tasks.get(agent_key).ok_or_else(|| {
                    PytxoError::Store("event actor is absent from the execution plan".into())
                })?;
                // Events reference agents in SQLite. Create the actual actor on
                // its first event, before writing evidence, rather than waiting
                // until the entire plan has finished. Unstarted tasks stay absent.
                if guard.get_agent(agent_key)?.is_none() {
                    guard.insert_agent_with_root(
                        agent_key,
                        &event_run_id,
                        &task.task_id.0,
                        task.wave,
                        None,
                        &event_command,
                        task.root.as_deref(),
                    )?;
                }
                guard.append_event(agent_key, kind, &payload)
            };
            write().is_ok()
        });
        if !persisted {
            lost_events_for_callback.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    });

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
        // Review-eligible workspaces must survive until their immutable package is durable.
        keep_worktrees: opts.keep_worktrees || apply_status == "pending",
        on_event: Some(on_event),
        signal_core: cfg.signal_core,
        signal_fidelity: cfg.signal_fidelity,
        isolation_mode,
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
        cloud_fallback_local: cfg.cloud.fallback_local
            && cloud::out_of_band_local_fallback_consent(),
        mcp_hub: Some(Arc::new(domain.mcp_hub.clone())),
        mcp_hub_enabled: cfg.mcp_hub.enabled,
        mcp_allowlist: cfg.mcp_hub.allowlist.clone(),
        sparse_exclude: cfg.blast.sparse_exclude.clone(),
    };

    let results = execute_plan(&ctx, &plan, &domain.process_registry, &domain.swarm).await?;

    // Candidate checks may wait for processes or operator approvals. Keep that
    // entire preparation/ledger section off the async supervisor thread.
    let review_cfg = cfg.clone();
    let review_run_id = run_id.clone();
    let (failed, review_error, cost) = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let cfg = review_cfg;
        let run_id = review_run_id;
        let mut failed = false;
        let mut review_error: Option<anyhow::Error> = None;
        let mut all_lines: Vec<String> = Vec::new();
        let store = store_for_events
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        for result in &results {
            let agent_key = format!("{}:{}", run_id, result.agent_id);
            let worktree_path = result
                .worktree_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned());
            if store.get_agent(&agent_key)?.is_none() {
                store.insert_agent_with_root(
                    &agent_key,
                    &run_id.0,
                    &result.task_id,
                    result.wave,
                    worktree_path.as_deref(),
                    &opts.cmd,
                    result.root_id.as_deref(),
                )?;
            } else {
                store.set_agent_workspace(&agent_key, worktree_path.as_deref())?;
            }
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
            let status = result.outcome.ledger_status();
            if !result.outcome.is_success() {
                failed = true;
            }
            store.finish_agent(&agent_key, result.exit_code, status)?;
        }

        let line_refs: Vec<&str> = all_lines.iter().map(String::as_str).collect();
        let cost = parse_cost_from_lines(&line_refs);
        if cost.tokens_in > 0 || cost.tokens_out > 0 || cost.cost_usd > 0.0 {
            store.update_run_cost(&run_id.0, cost.tokens_in, cost.tokens_out, cost.cost_usd)?;
        }

        let missing_events = lost_events.load(std::sync::atomic::Ordering::Relaxed);
        if missing_events > 0 {
            let message = format!(
                "{missing_events} live event(s) could not be persisted; run evidence is incomplete"
            );
            if let Some(result) = results.first() {
                store.append_event(
                    &format!("{}:{}", run_id, result.agent_id),
                    "evidence-gap",
                    &message,
                )?;
            }
            if apply_status == "pending" && store.begin_run_preparation(&run_id.0)? {
                store.fail_run_preparation(
                    &run_id.0,
                    &run_apply_error("event_persistence_failed", &message, false, None),
                )?;
            }
            review_error = Some(anyhow::anyhow!(message));
        }

        if !failed && review_error.is_none() && apply_status == "pending" {
            let tasks_by_id: std::collections::BTreeMap<_, _> = plan
                .waves
                .iter()
                .flatten()
                .map(|task| (task.task_id.0.as_str(), task))
                .collect();
            let workspaces = results
                .iter()
                .map(|result| {
                    let task = tasks_by_id.get(result.task_id.as_str()).ok_or_else(|| {
                        anyhow::anyhow!(
                            "completed task missing from review plan: {}",
                            result.task_id
                        )
                    })?;
                    Ok(AgentWorkspaceInput {
                        agent_id: result.agent_id.0.clone(),
                        task_id: result.task_id.clone(),
                        workspace_path: result.worktree_path.clone().ok_or_else(|| {
                            anyhow::anyhow!("completed task has no workspace: {}", result.task_id)
                        })?,
                        claims: task.paths.clone(),
                        depends_on: task.depends_on.clone(),
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            if !store.begin_run_preparation(&run_id.0)? {
                review_error = Some(anyhow::anyhow!(
                    "run review contract could not enter preparing state"
                ));
            } else {
                let expected_revision = base_revision.as_deref().unwrap_or_default();
                let preparation = prepare_review_package(
                    &domain.repo_root,
                    &domain.data_dir,
                    &run_id.0,
                    expected_revision,
                    &workspaces,
                    &cfg.blast.sparse_exclude,
                )
                .map_err(anyhow::Error::from)
                .and_then(|manifest| {
                    verify_combined_candidate(&domain, &cfg, &plan, &enforcement, manifest)
                });
                match preparation {
                    Ok(manifest) => {
                        if let Err(error) =
                            persist_finished_run_review(&store, &run_id.0, &manifest)
                        {
                            review_error = Some(error);
                        }
                    }
                    Err(error) => {
                        let apply_error = run_apply_error(
                            "review_preparation_failed",
                            &error.to_string(),
                            false,
                            None,
                        );
                        let _ = store.fail_run_preparation(&run_id.0, &apply_error);
                        review_error = Some(anyhow::anyhow!(error));
                    }
                }
            }
        }

        drop(store);
        Ok((failed, review_error, cost))
    })
    .await
    .map_err(|error| anyhow::anyhow!("candidate preparation worker failed: {error}"))??;

    if let Some(ref mut u) = ultra {
        let totals = u.meter.run_totals(&run_id)?;
        let cost_micro = if totals.cost_micro_usd > 0 {
            totals.cost_micro_usd
        } else {
            (cost.cost_usd * 1_000_000.0) as i64
        };
        billing::settle_ultra_run(u, &run_id, cost_micro)?;
    }

    let run_status = if failed || review_error.is_some() {
        "failed"
    } else {
        "completed"
    };
    run_finalizer.settle(run_status)?;

    if let Some(error) = review_error {
        return Err(error.context("run completed but durable review preparation failed"));
    }

    if failed {
        anyhow::bail!(
            "run {} failed after durable settlement (fail_fast={})",
            run_id.0,
            cfg.fail_fast
        );
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
    let (isolation_mode, isolation_backend) = resolved_isolation_status(cfg);
    Ok(RunStatusJson {
        id: run.id.clone(),
        status: run.status.clone(),
        repo_root: run.repo_root.clone(),
        started_at: run.started_at.to_rfc3339(),
        estimated_tokens_in: run.estimated_tokens_in,
        estimated_tokens_out: run.estimated_tokens_out,
        estimated_cost_usd: run.estimated_cost_usd,
        permission_profile: run.permission_profile.clone(),
        isolation_mode,
        isolation_backend,
        arbitrage_saved_tokens: arbitrage_saved,
        wallet_balance_microcredits: wallet_balance,
        agents,
    })
}

fn resolved_isolation_status(cfg: &PytxoConfig) -> (String, String) {
    let effective = pytxo_runner::effective_isolation_mode(cfg);
    (
        effective.as_str().to_string(),
        pytxo_runner::isolation_backend_label(effective, &cfg.blast.sparse_exclude),
    )
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
    stop_impl(config, repo, all, cleanup_worktrees, None).await
}

/// Stop one explicitly named run only when it is still the active run for the
/// supplied execution domain.
///
/// Desktop uses this guard so a stale supervision snapshot cannot terminate a
/// newer run that started in the same domain after the operator opened a stop
/// confirmation.
pub async fn stop_exact(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    expected_run_id: &str,
    cleanup_worktrees: bool,
) -> anyhow::Result<()> {
    stop_impl(
        config,
        repo,
        false,
        cleanup_worktrees,
        Some(expected_run_id),
    )
    .await
}

async fn stop_impl(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    all: bool,
    cleanup_worktrees: bool,
    expected_run_id: Option<&str>,
) -> anyhow::Result<()> {
    let repo = resolve_repo_root(repo.as_deref())?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let data_dir = repo.join(&cfg.data_dir);

    if all {
        fs::create_dir_all(&data_dir)?;
        let state_path = data_dir.join("active_run.json");
        // Serialize the snapshot and settlement against a new domain dispatch.
        // Even between children, the active run must receive durable cancellation.
        with_active_run_lock(&state_path, || {
            let tracked = ProcessRegistryFile::load(&registry_path(&data_dir))?;
            let mut run_ids: std::collections::BTreeSet<String> = tracked
                .entries
                .iter()
                .map(|entry| entry.run_id.clone())
                .collect();
            let active = match fs::read_to_string(&state_path) {
                Ok(raw) => Some(serde_json::from_str::<ActiveRunState>(&raw)?),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => return Err(error.into()),
            };
            if let Some(state) = &active {
                run_ids.insert(state.run_id.clone());
            }
            if let Ok(domain_id) = DomainId::from_repo_root(&repo) {
                if let Some(domain) = default_hypervisor().domain_state(domain_id.as_str()) {
                    domain.swarm.request_stop_all();
                }
            }
            let store = PytxoStore::open(&cfg.db_path_at(&repo))?;
            for run_id in &run_ids {
                stop_run(&data_dir, run_id, true)?;
                // Do not overwrite a completed/failed outcome racing with Stop.
                let _ = store.finish_run_if_status(run_id, "starting", "cancelled")?
                    || store.finish_run_if_running(run_id, "cancelled")?;
            }
            if cleanup_worktrees {
                for process in &tracked.entries {
                    pytxo_runner::remove_worktree(
                        Path::new(&process.repo_root),
                        Path::new(&process.worktree_path),
                        &process.branch,
                        true,
                    )?;
                }
            }
            if let Some(state) = active {
                clear_active_run_unlocked(&state_path, &state.run_id)?;
            }
            Ok(())
        })?;
        println!("Stopped all tracked runs and processes in this execution domain.");
        return Ok(());
    }

    let state_path = repo.join(&cfg.data_dir).join("active_run.json");
    if !state_path.exists() {
        if let Some(expected) = expected_run_id {
            anyhow::bail!(
                "refusing to stop run {expected}: no run is active in execution domain {}",
                repo.display()
            );
        }
        println!("No active run.");
        return Ok(());
    }
    let raw = fs::read_to_string(&state_path)?;
    let state: ActiveRunState = serde_json::from_str(&raw)?;
    if let Some(expected) = expected_run_id {
        if state.run_id != expected {
            anyhow::bail!(
                "refusing to stop run {expected}: active run in execution domain {} is {}",
                repo.display(),
                state.run_id
            );
        }
    }
    let tracked_processes = ProcessRegistryFile::load(&registry_path(&data_dir))?
        .for_run(&state.run_id)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let pids = stop_run(&data_dir, &state.run_id, true).map_err(|e| anyhow::anyhow!(e))?;
    if let Ok(domain_id) = DomainId::from_repo_root(&repo) {
        if let Some(domain) = default_hypervisor().domain_state(domain_id.as_str()) {
            domain.swarm.request_stop_run(&state.run_id);
        }
    }
    if cleanup_worktrees {
        for process in &tracked_processes {
            pytxo_runner::remove_worktree(
                Path::new(&process.repo_root),
                Path::new(&process.worktree_path),
                &process.branch,
                true,
            )
            .map_err(|e| anyhow::anyhow!(e))?;
        }
    }
    PytxoStore::open(&cfg.db_path_at(&repo))?.finish_run(&state.run_id, "cancelled")?;
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
    let path = repo_root.join(rel_path);
    let engine = PermissionEngine::new(cfg.permission_profile);
    let tier = engine.max_fidelity(fidelity.unwrap_or(cfg.signal_fidelity));
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

    let raw_allowed = engine.max_fidelity(FidelityTier::High) == FidelityTier::High;
    if force_raw && !raw_allowed {
        anyhow::bail!(
            "raw reads denied for {} profile",
            cfg.permission_profile.as_str()
        );
    }
    let effective_tier = engine.max_fidelity(cfg.signal_fidelity);
    let scaffold =
        !raw_allowed || (cfg.signal_core && !force_raw && effective_tier != FidelityTier::High);
    if scaffold {
        return TreeSitterSignalCore
            .read_scaffolded(&path, effective_tier)
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
    cfg.requested_permission_profile = Some(cfg.permission_profile);
    if let Ok(store) = pytxo_core::TrustedDomainStore::open_default() {
        if let Some(ceiling) = store.permission_for(repo) {
            cfg.permission_ceiling = Some(ceiling);
            cfg.permission_profile = cfg.permission_profile.capped_at(ceiling);
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
            verify: vec![],
        })
        .collect()
}

pub(crate) fn reserve_active_run(
    domain: &hypervisor::DomainState,
    cfg: &PytxoConfig,
    run_id: &RunId,
) -> anyhow::Result<()> {
    let db_path = cfg.db_path_at(&domain.repo_root);
    let store = PytxoStore::open(&db_path)?;
    store.insert_starting_run_with_profile(
        &run_id.0,
        &domain.repo_root.to_string_lossy(),
        Some(cfg.permission_profile.as_str()),
    )?;
    if let Err(error) = claim_active_run(cfg, run_id, &domain.repo_root) {
        let _ = store.finish_run_if_status(&run_id.0, "starting", "failed_startup");
        return Err(error);
    }
    Ok(())
}

pub(crate) fn settle_reserved_run_after_error(
    domain: &hypervisor::DomainState,
    cfg: &PytxoConfig,
    run_id: &RunId,
) {
    if let Ok(store) = PytxoStore::open(&cfg.db_path_at(&domain.repo_root)) {
        let startup_failed = store
            .finish_run_if_status(&run_id.0, "starting", "failed_startup")
            .unwrap_or(false);
        if !startup_failed {
            let _ = store.finish_run_if_running(&run_id.0, "failed");
        }
    }
    let _ = clear_active_run_if_matches(
        &domain.repo_root.join(&cfg.data_dir).join("active_run.json"),
        &run_id.0,
    );
}

fn claim_active_run(cfg: &PytxoConfig, run_id: &RunId, repo: &Path) -> anyhow::Result<()> {
    let path = repo.join(&cfg.data_dir).join("active_run.json");
    with_active_run_lock(&path, || {
        if let Some(owner) = reconcile_active_run_unlocked(&path, repo, cfg)? {
            anyhow::bail!(
                "execution domain {} already owns active run {owner}",
                repo.display()
            );
        }

        let supervisor_pid = std::process::id();
        let supervisor_start_identity = pytxo_runner::process_start_identity(supervisor_pid)?
            .ok_or_else(|| anyhow::anyhow!("current supervisor process has no live identity"))?;
        let state = ActiveRunState {
            run_id: run_id.0.clone(),
            repo_root: repo.to_string_lossy().to_string(),
            supervisor_pid,
            supervisor_start_identity: Some(supervisor_start_identity),
        };
        let json = serde_json::to_string_pretty(&state)?;
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        if let Err(error) = file
            .write_all(json.as_bytes())
            .and_then(|()| file.sync_all())
        {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(error.into());
        }
        Ok(())
    })
}

fn reconcile_active_run_unlocked(
    path: &Path,
    repo: &Path,
    cfg: &PytxoConfig,
) -> anyhow::Result<Option<String>> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let state: ActiveRunState = serde_json::from_str(&raw)?;
    let store = PytxoStore::open(&cfg.db_path_at(repo))?;
    let Some((status, _)) = store.get_run_status(&state.run_id)? else {
        clear_active_run_unlocked(path, &state.run_id)?;
        return Ok(None);
    };
    if PytxoStore::is_terminal_run_status(&status) {
        clear_active_run_unlocked(path, &state.run_id)?;
        return Ok(None);
    }
    if !matches!(status.as_str(), "starting" | "running") {
        anyhow::bail!(
            "active run {} has unreconciled ledger status {status}",
            state.run_id
        );
    }

    if let Some(identity) = state.supervisor_start_identity.as_deref() {
        if pytxo_runner::process_matches(state.supervisor_pid, identity)? {
            return Ok(Some(state.run_id));
        }
    }

    let registry = ProcessRegistryFile::load(&registry_path(&repo.join(&cfg.data_dir)))?;
    for process in registry.for_run(&state.run_id) {
        let Some(identity) = process.start_identity.as_deref() else {
            anyhow::bail!(
                "active run {} has a durable process without verifiable identity",
                state.run_id
            );
        };
        if pytxo_runner::process_matches(process.pid, identity)? {
            anyhow::bail!(
                "active run {} lost its supervisor but still owns live process {}",
                state.run_id,
                process.pid
            );
        }
    }

    let terminal = if status == "starting" {
        "failed_startup"
    } else {
        "failed"
    };
    let _ = store.finish_run_if_status(&state.run_id, &status, terminal)?;
    clear_active_run_unlocked(path, &state.run_id)?;
    tracing::warn!(
        run_id = %state.run_id,
        previous_status = %status,
        terminal_status = terminal,
        "reconciled crashed active-run owner"
    );
    Ok(None)
}

fn clear_active_run_if_matches(path: &Path, run_id: &str) -> anyhow::Result<()> {
    with_active_run_lock(path, || clear_active_run_unlocked(path, run_id))
}

fn clear_active_run_unlocked(path: &Path, run_id: &str) -> anyhow::Result<()> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let state: ActiveRunState = serde_json::from_str(&raw)?;
    if state.run_id == run_id {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn with_active_run_lock<T>(
    active_path: &Path,
    operation: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    let file_name = active_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("active_run.json");
    let lock_path = active_path.with_file_name(format!("{file_name}.lock"));
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.lock_exclusive()?;
    operation()
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

#[cfg(test)]
mod tests {
    use super::*;

    struct OriginalCapFixture {
        _temp: tempfile::TempDir,
        domain: Arc<hypervisor::DomainState>,
        cfg: PytxoConfig,
        plan: ExecutionPlan,
        enforcement: RunEnforcementEnvelope,
        manifest: pytxo_core::PreparedRunManifest,
    }

    fn original_cap_fixture() -> OriginalCapFixture {
        hypervisor::tests::isolate_pytxo_home();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        for path in ["a.txt", "b.txt"] {
            fs::write(repo.join(path), "before\n").unwrap();
        }
        // The current grant is Galaxy. A's saved original receipt is still Orbit.
        let cfg = PytxoConfig {
            permission_profile: PermissionProfile::Galaxy,
            execution_backend: pytxo_core::ExecutionBackend::Subprocess,
            ..Default::default()
        };
        let domain = hypervisor::HypervisorRegistry::new()
            .ensure_domain(&repo, &cfg)
            .unwrap();
        let run_id = "original-cap-collision";
        let store = PytxoStore::open(&cfg.db_path_at(&repo)).unwrap();
        store.insert_run(run_id, &repo.to_string_lossy()).unwrap();
        let mut tasks = Vec::new();
        let mut inputs = Vec::new();
        for (index, (task_id, name, claim)) in
            [("B", "agent-1", "b.txt"), ("A", "agent-0", "a.txt")]
                .into_iter()
                .enumerate()
        {
            let workspace = temp.path().join(format!("workspace-{index}"));
            fs::create_dir_all(&workspace).unwrap();
            for path in ["a.txt", "b.txt"] {
                fs::copy(repo.join(path), workspace.join(path)).unwrap();
            }
            fs::write(workspace.join(claim), "after\n").unwrap();
            let actor_id = format!("agent-{index}");
            let actor_key = format!("{run_id}:{actor_id}");
            store
                .insert_agent(
                    &actor_key,
                    run_id,
                    task_id,
                    0,
                    Some(&workspace.to_string_lossy()),
                    "fixture",
                )
                .unwrap();
            store
                .finish_agent(&actor_key, Some(0), "completed")
                .unwrap();
            tasks.push(pytxo_core::ScheduledTask {
                task_id: TaskId(task_id.into()),
                agent: name.into(),
                paths: vec![claim.into()],
                depends_on: vec![],
                wave: 0,
                root: None,
                signal_fidelity: None,
                verify: vec!["echo verification-ok".into()],
            });
            inputs.push(AgentWorkspaceInput {
                agent_id: actor_id,
                task_id: task_id.into(),
                workspace_path: workspace,
                claims: vec![claim.into()],
                depends_on: vec![],
            });
        }
        let plan = ExecutionPlan {
            waves: vec![tasks],
            conflicts: vec![],
            max_agents: 2,
            warnings: vec![],
        };
        let receipt = |profile| {
            permission_enforcement_receipt(
                profile,
                profile,
                &domain.id,
                pytxo_core::IsolationMode::Worktree,
                &[],
            )
            .unwrap()
        };
        let enforcement = RunEnforcementEnvelope {
            run: receipt(PermissionProfile::Galaxy),
            agents: std::collections::BTreeMap::from([
                ("agent-0".into(), receipt(PermissionProfile::Galaxy)), // B
                ("agent-1".into(), receipt(PermissionProfile::Orbit)),  // A
            ]),
        };
        let manifest =
            prepare_review_package(&repo, &domain.data_dir, run_id, "base", &inputs, &[]).unwrap();
        OriginalCapFixture {
            _temp: temp,
            domain,
            cfg,
            plan,
            enforcement,
            manifest,
        }
    }

    #[test]
    fn candidate_runtime_receipt_retains_original_cap_after_current_grant() {
        let fixture = original_cap_fixture();
        let checked = verify_combined_candidate(
            &fixture.domain,
            &fixture.cfg,
            &fixture.plan,
            &fixture.enforcement,
            fixture.manifest,
        )
        .unwrap();
        let checks = &checked.candidate_verification.as_ref().unwrap().checks;
        let a = checks.iter().find(|check| check.task_id == "A").unwrap();
        assert_eq!(
            a.effective_profile, "orbit",
            "later grants must not widen original A authority"
        );
        assert_eq!(a.enforcement["effective_profile"], "orbit");
        assert_eq!(
            checks
                .iter()
                .find(|check| check.task_id == "B")
                .unwrap()
                .effective_profile,
            "galaxy"
        );
        validate_candidate_recipe(
            &checked,
            &fixture.plan,
            &fixture.enforcement,
            &fixture.domain.id,
        )
        .unwrap();
    }

    fn assert_candidate_runtime_receipt_rejected(change: impl Fn(&mut RunEnforcementEnvelope)) {
        let fixture = original_cap_fixture();
        let mut invalid = fixture.enforcement.clone();
        change(&mut invalid);
        let error = verify_combined_candidate(
            &fixture.domain,
            &fixture.cfg,
            &fixture.plan,
            &invalid,
            fixture.manifest,
        )
        .expect_err("verifier accepted an invalid runtime receipt");
        assert!(
            error.to_string().contains("original"),
            "unexpected refusal: {error}"
        );

        let fixture = original_cap_fixture();
        let checked = verify_combined_candidate(
            &fixture.domain,
            &fixture.cfg,
            &fixture.plan,
            &fixture.enforcement,
            fixture.manifest,
        )
        .unwrap();
        let mut invalid = fixture.enforcement.clone();
        change(&mut invalid);
        assert!(
            validate_candidate_recipe(&checked, &fixture.plan, &invalid, &fixture.domain.id)
                .is_err(),
            "recipe accepted an invalid runtime receipt"
        );
    }

    #[test]
    fn candidate_runtime_receipt_missing_actor_fails_closed() {
        assert_candidate_runtime_receipt_rejected(|enforcement| {
            enforcement.agents.remove("agent-1");
        });
    }

    #[test]
    fn candidate_runtime_receipt_wrong_domain_fails_closed() {
        assert_candidate_runtime_receipt_rejected(|enforcement| {
            enforcement
                .agents
                .get_mut("agent-1")
                .unwrap()
                .execution_domain = "another-domain".into();
        });
    }

    #[test]
    fn candidate_runtime_receipt_effective_above_requested_fails_closed() {
        assert_candidate_runtime_receipt_rejected(|enforcement| {
            enforcement
                .agents
                .get_mut("agent-1")
                .unwrap()
                .effective_profile = "galaxy".into();
        });
    }

    #[test]
    fn candidate_runtime_receipt_rejects_check_above_original_task_cap() {
        let fixture = original_cap_fixture();
        let mut checked = verify_combined_candidate(
            &fixture.domain,
            &fixture.cfg,
            &fixture.plan,
            &fixture.enforcement,
            fixture.manifest,
        )
        .unwrap();
        let a = checked
            .candidate_verification
            .as_mut()
            .unwrap()
            .checks
            .iter_mut()
            .find(|check| check.task_id == "A")
            .unwrap();
        a.effective_profile = "galaxy".into();
        a.enforcement["effective_profile"] = "galaxy".into();
        assert!(
            validate_candidate_recipe(
                &checked,
                &fixture.plan,
                &fixture.enforcement,
                &fixture.domain.id
            )
            .is_err(),
            "saved evidence widened A's original Orbit authority"
        );
    }

    #[test]
    fn candidate_approval_requires_a_persisted_origin_actor_and_audit() {
        hypervisor::tests::isolate_pytxo_home();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&workspace).unwrap();
        fs::write(repo.join("owned.txt"), "before\n").unwrap();
        fs::write(workspace.join("owned.txt"), "after\n").unwrap();
        let cfg = PytxoConfig {
            permission_profile: PermissionProfile::Galaxy,
            execution_backend: pytxo_core::ExecutionBackend::Subprocess,
            ..Default::default()
        };
        let domain = hypervisor::HypervisorRegistry::new()
            .ensure_domain(&repo, &cfg)
            .unwrap();
        let run_id = "candidate-approval";
        let actor = format!("{run_id}:agent-0");
        let store = PytxoStore::open(&cfg.db_path_at(&repo)).unwrap();
        store.insert_run(run_id, &repo.to_string_lossy()).unwrap();
        store
            .insert_agent(
                &actor,
                run_id,
                "task",
                0,
                Some(&workspace.to_string_lossy()),
                "fixture",
            )
            .unwrap();
        store.finish_agent(&actor, Some(0), "completed").unwrap();
        let manifest = prepare_review_package(
            &repo,
            &domain.data_dir,
            run_id,
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent-0".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &cfg.blast.sparse_exclude,
        )
        .unwrap();
        // The classifier requests approval, but echo performs no installation.
        let plan = ExecutionPlan {
            waves: vec![vec![pytxo_core::ScheduledTask {
                task_id: TaskId("task".into()),
                agent: "codex".into(),
                paths: vec!["owned.txt".into()],
                depends_on: vec![],
                wave: 0,
                root: None,
                signal_fidelity: None,
                verify: vec!["echo npm install".into()],
            }]],
            conflicts: vec![],
            max_agents: 1,
            warnings: vec![],
        };
        let receipt = permission_enforcement_receipt(
            PermissionProfile::Galaxy,
            PermissionProfile::Galaxy,
            &domain.id,
            pytxo_core::IsolationMode::Worktree,
            &[],
        )
        .unwrap();
        let enforcement = RunEnforcementEnvelope {
            run: receipt.clone(),
            agents: std::collections::BTreeMap::from([("agent-0".into(), receipt)]),
        };
        let db = rusqlite::Connection::open(cfg.db_path_at(&repo)).unwrap();
        db.execute_batch("CREATE TRIGGER reject_approval BEFORE INSERT ON events WHEN NEW.kind = 'hitl-resolve' BEGIN SELECT RAISE(ABORT, 'injected approval write failure'); END;").unwrap();
        let worker_domain = domain.clone();
        let worker = std::thread::spawn(move || {
            verify_combined_candidate(&worker_domain, &cfg, &plan, &enforcement, manifest)
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut exercised_failure = false;
        let mut approvals = 0;
        while !worker.is_finished() && std::time::Instant::now() < deadline {
            for request in domain.hitl.pending() {
                assert_eq!(request.agent_key, actor);
                if !exercised_failure {
                    assert!(domain.hitl.try_resolve(&request.id, true).is_err());
                    assert_eq!(
                        domain.hitl.decision(&request.id),
                        pytxo_runner::HitlDecision::Pending
                    );
                    assert!(store.list_events(&actor, 100).unwrap().is_empty());
                    db.execute_batch("DROP TRIGGER reject_approval;").unwrap();
                    exercised_failure = true;
                }
                assert!(domain.hitl.try_resolve(&request.id, true).unwrap());
                approvals += 1;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        if !worker.is_finished() {
            domain.swarm.request_stop_run(run_id);
        }
        let checked = worker.join().unwrap().unwrap();
        assert!(exercised_failure);
        assert!(approvals > 0);
        assert_eq!(checked.candidate_verification.unwrap().checks.len(), 1);
        assert_eq!(store.list_agents_for_run(run_id).unwrap().len(), 1);
        assert_eq!(
            store
                .list_events(&actor, 100)
                .unwrap()
                .iter()
                .filter(|event| event.kind == "hitl-resolve")
                .count(),
            approvals
        );
    }

    fn deep_space_read_fixture(signal_core: bool) -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(
            repo.join("sample.rs"),
            "pub fn sample() { let raw_secret = \"must-not-egress\"; println!(\"{raw_secret}\"); }\n",
        )
        .unwrap();
        let config = repo.join("pytxo.toml");
        fs::write(
            &config,
            format!(
                "permission_profile = \"deep_space\"\nsignal_core = {signal_core}\nsignal_fidelity = \"high\"\n"
            ),
        )
        .unwrap();
        (temp, repo, config)
    }

    #[test]
    fn deepspace_caps_explicit_scaffold_fidelity_to_low() {
        let (_temp, repo, config) = deep_space_read_fixture(true);

        let result = read_file_scaffolded(
            Some(config),
            Some(repo),
            "sample.rs",
            Some(FidelityTier::High),
        )
        .unwrap();

        assert!(!result.fallback_raw);
        assert!(result.content.contains("pub fn sample"));
        assert!(!result.content.contains("must-not-egress"));
    }

    #[test]
    fn deepspace_rejects_mcp_raw_override() {
        let (_temp, repo, config) = deep_space_read_fixture(true);

        let error = read_file(Some(config), Some(repo), "sample.rs", true)
            .expect_err("DeepSpace must reject raw MCP reads");

        assert!(error
            .to_string()
            .contains("raw reads denied for deep_space"));
    }

    #[test]
    fn deepspace_enforces_low_fidelity_when_signal_core_is_disabled_in_config() {
        let (_temp, repo, config) = deep_space_read_fixture(false);

        let result = read_file(Some(config), Some(repo), "sample.rs", false).unwrap();

        assert!(!result.fallback_raw);
        assert!(!result.content.contains("must-not-egress"));
    }

    #[test]
    fn status_reports_effective_isolation_backend() {
        let cfg = PytxoConfig::default();
        let (mode, backend) = resolved_isolation_status(&cfg);

        assert_eq!(mode, "overlay");
        assert!(
            backend.starts_with("overlay-") || backend.starts_with("projfs-"),
            "expected effective overlay backend, got {backend}"
        );
    }

    #[test]
    fn finalizer_only_clears_its_matching_active_marker() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("active_run.json");
        fs::write(
            &path,
            serde_json::to_vec(&ActiveRunState {
                run_id: "newer-run".into(),
                repo_root: temp.path().display().to_string(),
                supervisor_pid: 0,
                supervisor_start_identity: None,
            })
            .unwrap(),
        )
        .unwrap();

        clear_active_run_if_matches(&path, "older-run").unwrap();
        assert!(path.exists());
        clear_active_run_if_matches(&path, "newer-run").unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn interrupted_apply_is_reconciled_before_checkout_cleanliness_validation() {
        fn git(repo: &Path, args: &[&str]) {
            let status = std::process::Command::new("git")
                .args(args)
                .current_dir(repo)
                .status()
                .unwrap();
            assert!(status.success());
        }

        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-q"]);
        git(&repo, &["config", "user.email", "pytxo@example.invalid"]);
        git(&repo, &["config", "user.name", "Pytxo Test"]);
        fs::write(repo.join(".gitignore"), ".pytxo/\n").unwrap();
        fs::write(repo.join("owned.txt"), "before\n").unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "base"]);

        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join(".gitignore"), ".pytxo/\n").unwrap();
        fs::write(workspace.join("owned.txt"), "after\n").unwrap();
        let data_dir = repo.join(".pytxo/data");
        let manifest = prepare_review_package(
            &repo,
            &data_dir,
            "run-ordering",
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        store
            .insert_run_with_profile("run-ordering", &repo.to_string_lossy(), Some("orbit"))
            .unwrap();
        store
            .save_run_contract("run-ordering", "base", "{}", "{}")
            .unwrap();
        assert!(store.begin_run_preparation("run-ordering").unwrap());
        store
            .finish_run_preparation("run-ordering", &manifest)
            .unwrap();
        store.finish_run("run-ordering", "completed").unwrap();
        assert!(store.claim_run_apply("run-ordering").unwrap());
        pytxo_runner::apply_prepared_review_with_fault(
            &repo,
            &data_dir,
            &manifest,
            Some(pytxo_runner::ApplyFaultPoint::InterruptAfterRename(1)),
        )
        .expect_err("interrupt");
        assert!(assert_clean_primary_checkout(&repo).is_err());

        reconcile_domain_apply_journals(&repo, &data_dir, &store).unwrap();

        assert_clean_primary_checkout(&repo).unwrap();
        let contract = store.get_run_contract("run-ordering").unwrap().unwrap();
        assert_eq!(contract.apply_status, "ready");
        assert!(contract
            .last_apply_error
            .unwrap()
            .attempt_id
            .is_some_and(|attempt| !attempt.is_empty()));
    }

    #[test]
    fn explicit_recovery_reconciliation_persists_rolled_back_contract() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("owned.txt"), "before\n").unwrap();
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("owned.txt"), "after\n").unwrap();
        let run_id = "run-explicit-recovery";
        let data_dir = repo.join(".pytxo/data");
        let manifest = prepare_review_package(
            &repo,
            &data_dir,
            run_id,
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        store
            .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
            .unwrap();
        store.save_run_contract(run_id, "base", "{}", "{}").unwrap();
        assert!(store.begin_run_preparation(run_id).unwrap());
        store.finish_run_preparation(run_id, &manifest).unwrap();
        store.finish_run(run_id, "completed").unwrap();
        assert!(store.claim_run_apply(run_id).unwrap());
        pytxo_runner::apply_prepared_review_with_fault(
            &repo,
            &data_dir,
            &manifest,
            Some(pytxo_runner::ApplyFaultPoint::InterruptAfterRename(1)),
        )
        .expect_err("interrupt");
        store
            .finish_run_apply_error(
                run_id,
                "recovery_required",
                &run_apply_error(
                    "apply_recovery_required",
                    "reconciliation required",
                    false,
                    None,
                ),
                Some("unprovable"),
            )
            .unwrap();
        drop(store);

        let outcome = reconcile_run_recovery(None, Some(repo.clone()), run_id).unwrap();

        assert!(matches!(outcome, RecoveryOutcome::RolledBack { .. }));
        assert_eq!(
            fs::read_to_string(repo.join("owned.txt")).unwrap(),
            "before\n"
        );
        let contract = PytxoStore::open(&data_dir.join("pytxo.db"))
            .unwrap()
            .get_run_contract(run_id)
            .unwrap()
            .unwrap();
        assert_eq!(contract.apply_status, "ready");
        let error = contract.last_apply_error.unwrap();
        assert_eq!(error.code, "interrupted_apply");
        assert!(error.rollback_confirmed);
        assert_eq!(contract.recovery_state.as_deref(), Some("rolled_back"));
    }

    #[test]
    fn explicit_recovery_reconciliation_persists_committed_contract() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("owned.txt"), "before\n").unwrap();
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("owned.txt"), "after\n").unwrap();
        let run_id = "run-explicit-committed-recovery";
        let data_dir = repo.join(".pytxo/data");
        let manifest = prepare_review_package(
            &repo,
            &data_dir,
            run_id,
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        store
            .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
            .unwrap();
        store.save_run_contract(run_id, "base", "{}", "{}").unwrap();
        assert!(store.begin_run_preparation(run_id).unwrap());
        store.finish_run_preparation(run_id, &manifest).unwrap();
        store.finish_run(run_id, "completed").unwrap();
        assert!(store.claim_run_apply(run_id).unwrap());
        let applied = pytxo_runner::apply_prepared_review(&repo, &data_dir, &manifest).unwrap();
        store
            .finish_run_apply_error(
                run_id,
                "recovery_required",
                &run_apply_error(
                    "apply_recovery_required",
                    "commit outcome was not persisted",
                    false,
                    Some(applied.transaction_id.clone()),
                ),
                Some("unprovable"),
            )
            .unwrap();
        drop(store);

        let outcome = reconcile_run_recovery(None, Some(repo.clone()), run_id).unwrap();

        let RecoveryOutcome::Committed(recovered) = outcome else {
            panic!("expected committed recovery");
        };
        assert_eq!(recovered.transaction_id, applied.transaction_id);
        let contract = PytxoStore::open(&data_dir.join("pytxo.db"))
            .unwrap()
            .get_run_contract(run_id)
            .unwrap()
            .unwrap();
        assert_eq!(contract.apply_status, "applied");
        assert!(contract.applied_at.is_some());
        assert_eq!(
            contract
                .apply_manifest_json
                .as_deref()
                .map(serde_json::from_str::<RunApplyManifest>)
                .transpose()
                .unwrap()
                .unwrap()
                .transaction_id,
            applied.transaction_id
        );
        assert!(contract.recovery_state.is_none());
    }

    #[test]
    fn explicit_recovery_without_journal_refreshes_recovery_required_error() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let data_dir = repo.join(".pytxo/data");
        fs::create_dir_all(&data_dir).unwrap();
        let run_id = "run-missing-recovery-journal";
        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        store
            .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
            .unwrap();
        store.save_run_contract(run_id, "base", "{}", "{}").unwrap();
        let manifest = pytxo_core::PreparedRunManifest {
            candidate_verification: None,
            version: 1,
            run_id: run_id.into(),
            base_revision: "base".into(),
            prepared_at: "2026-08-01T00:00:00Z".into(),
            package_digest: "digest".into(),
            summary: Default::default(),
            files: vec![],
        };
        assert!(store.begin_run_preparation(run_id).unwrap());
        store.finish_run_preparation(run_id, &manifest).unwrap();
        store.finish_run(run_id, "completed").unwrap();
        assert!(store.claim_run_apply(run_id).unwrap());
        store
            .finish_run_apply_error(
                run_id,
                "recovery_required",
                &run_apply_error("old_error", "old recovery evidence", false, None),
                Some("unprovable"),
            )
            .unwrap();
        drop(store);

        let outcome = reconcile_run_recovery(None, Some(repo.clone()), run_id).unwrap();

        assert!(matches!(
            outcome,
            RecoveryOutcome::RecoveryRequired { attempt_id: None }
        ));
        let contract = PytxoStore::open(&data_dir.join("pytxo.db"))
            .unwrap()
            .get_run_contract(run_id)
            .unwrap()
            .unwrap();
        assert_eq!(contract.apply_status, "recovery_required");
        assert_eq!(
            contract.last_apply_error.unwrap().code,
            "recovery_journal_missing"
        );
        assert_eq!(contract.recovery_state.as_deref(), Some("unprovable"));
    }

    #[test]
    fn unprovable_recovery_takes_precedence_over_affected_path_drift() {
        let (status, code, recovery_state) = classify_apply_failure(true, true, false);
        assert_eq!(status, "recovery_required");
        assert_eq!(code, "apply_recovery_required");
        assert_eq!(recovery_state, Some("unprovable"));
    }

    #[test]
    fn future_dated_prior_attempt_cannot_hide_a_new_interrupted_apply() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("owned.txt"), "before\n").unwrap();
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("owned.txt"), "after\n").unwrap();
        let run_id = "run-future-prior-attempt";
        let data_dir = repo.join(".pytxo/data");
        let db_path = data_dir.join("pytxo.db");
        let command = if cfg!(windows) {
            "powershell -NoProfile -NonInteractive -Command \"if ((Get-Content -Raw 'owned.txt').Trim() -ne 'after') { exit 1 }\""
        } else {
            "grep -qx 'after' owned.txt"
        };
        let plan = ExecutionPlan {
            waves: vec![vec![pytxo_core::ScheduledTask {
                task_id: TaskId("task".into()),
                agent: "codex".into(),
                paths: vec!["owned.txt".into()],
                depends_on: vec![],
                wave: 0,
                root: None,
                signal_fidelity: None,
                verify: vec![command.into()],
            }]],
            conflicts: vec![],
            max_agents: 1,
            warnings: vec![],
        };
        let receipt = permission_enforcement_receipt(
            PermissionProfile::Orbit,
            PermissionProfile::Orbit,
            &DomainId::from_repo_root(&repo).unwrap(),
            pytxo_core::IsolationMode::Worktree,
            &[],
        )
        .unwrap();
        let enforcement = serde_json::json!({
            "run": receipt,
            "agents": { "codex": receipt, "agent-0": receipt }
        });
        let manifest = prepare_review_package(
            &repo,
            &data_dir,
            run_id,
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        let candidate = CandidateVerification::prepare(&repo, &data_dir, &manifest, &[]).unwrap();
        let check = pytxo_runner::run_candidate_check(
            &pytxo_runner::CandidateCheckContext {
                cwd: candidate.workspace_root().into(),
                run_id: run_id.into(),
                agent_key: format!("{run_id}:candidate"),
                repo_root: repo.clone(),
                data_dir: data_dir.clone(),
                profile: PermissionProfile::Orbit,
                domain_id: DomainId::from_repo_root(&repo).unwrap(),
                execution_backend: pytxo_core::ExecutionBackend::Subprocess,
                workspace_isolated: true,
                hitl: None,
                swarm: pytxo_runner::SwarmRegistry::new(),
                on_event: None,
            },
            command,
        )
        .unwrap();
        let manifest = candidate
            .finish(vec![pytxo_core::CandidateCheckEvidence {
                task_id: "task".into(),
                command: command.into(),
                effective_profile: "orbit".into(),
                passed: true,
                enforcement: serde_json::to_value(check).unwrap(),
            }])
            .unwrap();
        let store = PytxoStore::open(&db_path).unwrap();
        store
            .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
            .unwrap();
        store
            .save_run_contract(
                run_id,
                "base",
                &serde_json::to_string(&plan).unwrap(),
                &serde_json::to_string(&enforcement).unwrap(),
            )
            .unwrap();
        assert!(store.begin_run_preparation(run_id).unwrap());
        store.finish_run_preparation(run_id, &manifest).unwrap();
        store.finish_run(run_id, "completed").unwrap();
        drop(store);

        pytxo_runner::apply_prepared_review_with_fault(
            &repo,
            &data_dir,
            &manifest,
            Some(pytxo_runner::ApplyFaultPoint::InterruptAfterJournalPrepared),
        )
        .expect_err("create prior attempt");
        let prior_attempt = pytxo_runner::apply_attempt_ids(&data_dir, run_id)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let prior_journal = data_dir
            .join("apply")
            .join(run_id)
            .join(&prior_attempt)
            .join("journal.json");
        let mut journal: serde_json::Value =
            serde_json::from_slice(&fs::read(&prior_journal).unwrap()).unwrap();
        journal["created_at"] = "2999-01-01T00:00:00Z".into();
        fs::write(&prior_journal, serde_json::to_vec_pretty(&journal).unwrap()).unwrap();
        assert!(matches!(
            pytxo_runner::reconcile_apply_journals(&repo, &data_dir, run_id).unwrap(),
            RecoveryOutcome::RolledBack { .. }
        ));

        apply_run_changes_with(
            None,
            Some(repo.clone()),
            run_id,
            |repo_root, data_dir, prepared, lease| {
                pytxo_runner::apply_prepared_review_with_fault_under_lease(
                    repo_root,
                    data_dir,
                    prepared,
                    lease,
                    Some(pytxo_runner::ApplyFaultPoint::InterruptAfterRename(1)),
                )
            },
        )
        .expect_err("the new interrupted attempt is reconciled and reported");

        assert_eq!(
            fs::read_to_string(repo.join("owned.txt")).unwrap(),
            "before\n"
        );
        let contract = PytxoStore::open(&db_path)
            .unwrap()
            .get_run_contract(run_id)
            .unwrap()
            .unwrap();
        assert_eq!(contract.apply_status, "ready");
        let error = contract.last_apply_error.unwrap();
        assert_eq!(error.code, "apply_failed");
        assert!(error.rollback_confirmed);
        assert!(error
            .attempt_id
            .is_some_and(|attempt| attempt != prior_attempt));
        assert!(
            refresh_run_review(None, Some(repo), run_id)
                .unwrap_err()
                .to_string()
                .contains("cannot be refreshed from status ready"),
            "a reconciled failed Apply must not be misclassified as refreshable review_failed"
        );
    }
}
