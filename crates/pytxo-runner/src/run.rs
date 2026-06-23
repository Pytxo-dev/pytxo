use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::thread;

use pytxo_core::{
    root_scoped_claim, AgentId, BillingMode, ByteHeuristicEstimator, ChildLaunchEnv,
    CloudDispatcher, ConfigModelRouter, ContextCache, overlay_upper_cloud_delta, DomainId,
    ExecRequest, ExecutionBackend, ExecutionPlan, FidelityTier, IsolationCtx,
    IsolationMode, ManagedTransport, ModelRoute, ModelRouter, NetworkPolicy, PermissionEngine,
    PermissionProfile, PytxoError, RaceShield, Result, RunId, ScheduledTask, StartSandboxRequest,
    TaskId, TokenEstimator, UsageKey, UsageMeter, WorkspaceHandle,
};

use crate::context::{extend_context_with_readonly_roots, prepare_agent_context_for_root};
use crate::failure::implicated_paths;
use crate::git::remove_worktree;
use crate::process::{ChildRecord, ProcessRegistry};
use crate::process_registry_file::{registry_path, ProcessEntry, ProcessRegistryFile};
use crate::race::SwarmRegistry;

pub type EventCallback = Arc<dyn Fn(&str, &str, &str) + Send + Sync>;

/// One modular-project root's execution surface ([[ADR-0011-modular-project-manifest]]).
#[derive(Clone, Debug)]
pub struct RootExec {
    pub repo_root: PathBuf,
    pub worktree_base: PathBuf,
    pub read_only: bool,
    pub permission_profile: PermissionProfile,
}

#[derive(Clone)]
pub struct RunContext {
    pub run_id: RunId,
    pub repo_root: PathBuf,
    pub worktree_base: PathBuf,
    pub data_dir: PathBuf,
    pub cmd: String,
    /// When set, expands per task via [`resolve_cmd_for_task`].
    pub task_cmd_template: Option<String>,
    pub task_prompts: HashMap<String, String>,
    pub keep_worktrees: bool,
    pub on_event: Option<EventCallback>,
    pub signal_core: bool,
    pub signal_fidelity: FidelityTier,
    pub isolation_mode: IsolationMode,
    /// Run-level default; per-agent overrides in `agent_profiles`.
    pub permission_profile: PermissionProfile,
    pub agent_profiles: HashMap<String, PermissionProfile>,
    /// Per-agent model/provider/cli_adapter from `pytxo.toml` ([[ADR-0014]]).
    pub route_agents: Vec<pytxo_core::AgentSpec>,
    pub billing_mode: BillingMode,
    pub domain_id: DomainId,
    pub model_router: Arc<dyn ModelRouter>,
    pub managed_transport: ManagedTransport,
    pub usage_meter: Option<Arc<dyn UsageMeter>>,
    pub token_estimator: Arc<dyn TokenEstimator>,
    pub execution_backend: ExecutionBackend,
    pub pty_rows: u16,
    pub pty_cols: u16,
    /// Galaxy HITL queue ([[permission-profile-engine]]). When set, flushes that
    /// `flush_requires_approval()` block until a human approves via the queue.
    pub hitl: Option<crate::hitl::HitlQueue>,
    /// Per-agent extra context paths (`[[agent]].paths`), merged with task paths
    /// when materializing context ([[signal-core]]).
    pub agent_paths: HashMap<String, Vec<String>>,
    /// Per-agent Signal Core fidelity overrides (`[[agent]].signal_fidelity`),
    /// applied below any per-task override and the permission cap ([[closed-loop-fidelity]]).
    pub agent_fidelity: HashMap<String, FidelityTier>,
    /// Modular project roots by label for unified multi-root runs
    /// ([[ADR-0011-modular-project-manifest]]). Empty = single-root run on `repo_root`.
    pub roots: HashMap<String, RootExec>,
    /// Read-only project roots whose files are scaffolded into context but never
    /// receive worktrees or flushes (cross-root protos).
    pub readonly_context_roots: Vec<(String, PathBuf)>,
    /// When true and using the subprocess backend, drain the Race Shield stdin queue
    /// into the child after spawn ([[race-shield]]).
    pub subprocess_stdin: bool,
    /// Remote cloud sandbox dispatcher ([[cloud-sandbox-service]]).
    pub cloud_dispatcher: Option<Arc<dyn CloudDispatcher>>,
    /// Pro context cache client (read-through / write-through).
    pub context_cache: Option<Arc<dyn ContextCache>>,
    pub cloud_cache_enabled: bool,
    /// Fall back to local PTY when cloud ping or exec fails.
    pub cloud_fallback_local: bool,
    /// MCP hub v2 child session registry ([[mcp-router]]).
    pub mcp_hub: Option<Arc<crate::mcp_hub::McpHub>>,
    pub mcp_hub_enabled: bool,
    pub mcp_allowlist: Vec<String>,
    /// `[blast].sparse_exclude` from config (overlay sparse copy / cloud sync).
    pub sparse_exclude: Vec<String>,
}

impl RunContext {
    pub fn default_metering(
        repo_root: &Path,
    ) -> (
        DomainId,
        Arc<dyn ModelRouter>,
        ManagedTransport,
        Arc<dyn TokenEstimator>,
    ) {
        let domain_id =
            DomainId::from_repo_root(repo_root).unwrap_or_else(|_| DomainId("unknown".to_string()));
        (
            domain_id,
            Arc::new(ConfigModelRouter),
            ManagedTransport::default(),
            Arc::new(ByteHeuristicEstimator),
        )
    }
}

#[derive(Clone, Debug)]
pub struct AgentRunResult {
    pub agent_id: AgentId,
    pub task_id: String,
    pub wave: u32,
    pub worktree_path: PathBuf,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    /// Modular project root label this agent ran under ([[ADR-0011-modular-project-manifest]]).
    pub root_id: Option<String>,
}

pub struct SingleResult {
    pub worktree_path: PathBuf,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub pid: Option<u32>,
}

pub async fn execute_plan(
    ctx: &RunContext,
    plan: &ExecutionPlan,
    registry: &ProcessRegistry,
    swarm: &SwarmRegistry,
) -> Result<Vec<AgentRunResult>> {
    let mut all_results = Vec::new();
    let mut agent_index = 0usize;

    for wave in &plan.waves {
        let mut set = tokio::task::JoinSet::new();
        for task in wave {
            let agent_id = AgentId::new(agent_index);
            agent_index += 1;
            let ctx = ctx.clone();
            let task = task.clone();
            let registry = registry.clone();
            let swarm = swarm.clone();
            set.spawn(
                async move { run_one_agent(&ctx, &task, &agent_id, &registry, &swarm).await },
            );
        }

        while let Some(joined) = set.join_next().await {
            let result = joined.map_err(|e| PytxoError::Runner(format!("join: {e}")))??;
            all_results.push(result);
        }
    }

    let mut proc_file = ProcessRegistryFile::load(&registry_path(&ctx.data_dir))?;
    proc_file.remove_run(&ctx.run_id.0);
    proc_file.save(&registry_path(&ctx.data_dir))?;

    Ok(all_results)
}

async fn run_one_agent(
    ctx: &RunContext,
    task: &ScheduledTask,
    agent_id: &AgentId,
    registry: &ProcessRegistry,
    swarm: &SwarmRegistry,
) -> Result<AgentRunResult> {
    let agent_key = format!("{}:{}", ctx.run_id, agent_id);
    let claim_paths: Vec<String> = task
        .paths
        .iter()
        .map(|p| root_scoped_claim(task.root.as_deref(), p))
        .collect();
    swarm.try_claim_paths(&agent_key, &claim_paths).or_else(|e| {
        if std::env::var("PYTXO_DAG_RECOVERY").ok().as_deref() == Some("1") {
            if let Some(cb) = ctx.on_event.as_ref() {
                cb(
                    &agent_key,
                    "dag-recovery",
                    &format!("path-claim stall; synthetic completion injected: {e}"),
                );
            }
            Ok(())
        } else {
            Err(e)
        }
    })?;

    let profile = match task.root.as_deref() {
        Some(label) if !label.is_empty() => ctx
            .roots
            .get(label)
            .map(|r| r.permission_profile)
            .unwrap_or(ctx.permission_profile),
        _ => ctx
            .agent_profiles
            .get(&task.agent)
            .copied()
            .unwrap_or(ctx.permission_profile),
    };
    let engine = PermissionEngine::new(profile);

    // Resolve the execution root for this task's modular-project `root` label;
    // single-root runs (empty map) fall back to the run's primary root.
    let (eff_repo_root, eff_worktree_base) = resolve_task_root(task, ctx)?;

    let isolation = crate::blast::isolation_for_mode(ctx.isolation_mode);
    let iso_ctx = IsolationCtx {
        run_id: ctx.run_id.clone(),
        agent_id: agent_id.clone(),
        repo_root: eff_repo_root.clone(),
        worktree_base: eff_worktree_base.clone(),
        sparse_exclude: ctx.sparse_exclude.clone(),
    };

    let (workspace, used_isolation) = if engine.use_worktree_isolation() {
        let workspace = isolation.prepare(&iso_ctx)?;
        (workspace, true)
    } else {
        // Supernova: run directly in repo root without Blast worktree isolation.
        (
            WorkspaceHandle {
                cwd: eff_repo_root.clone(),
                branch: String::new(),
                backend: ctx.isolation_mode,
            },
            false,
        )
    };
    let wt_path = workspace.cwd.clone();
    let branch = workspace.branch.clone();

    // Materialize context over task paths unioned with the agent's own
    // `[[agent]].paths` ([[signal-core]] context launch contract).
    let mut context_paths = task.paths.clone();
    if let Some(extra) = ctx.agent_paths.get(&task.agent) {
        for p in extra {
            if !context_paths.contains(p) {
                context_paths.push(p.clone());
            }
        }
    }

    // Resolve fidelity: per-task override > per-agent override > global, then cap.
    let requested_fidelity = task
        .signal_fidelity
        .or_else(|| ctx.agent_fidelity.get(&task.agent).copied())
        .unwrap_or(ctx.signal_fidelity);
    let fidelity = engine.max_fidelity(requested_fidelity);
    let route = ctx
        .model_router
        .route(&task.agent, &minimal_config_for_route(ctx));
    let cache_ref = ctx.context_cache.as_deref();
    let mut bundle = prepare_agent_context_for_root(
        &eff_repo_root,
        &ctx.data_dir,
        &ctx.run_id,
        &agent_id.0,
        &context_paths,
        ctx.signal_core,
        fidelity,
        ctx.token_estimator.as_ref(),
        &route.model,
        task.root.as_deref(),
        cache_ref,
        &ctx.domain_id.0,
        ctx.cloud_cache_enabled,
        profile,
    )?;
    if ctx.signal_core && !ctx.readonly_context_roots.is_empty() {
        bundle = extend_context_with_readonly_roots(
            bundle,
            &ctx.data_dir,
            &ctx.run_id,
            &agent_id.0,
            &ctx.readonly_context_roots,
            fidelity,
            ctx.token_estimator.as_ref(),
            &route.model,
        )?;
    }

    if let Some(meter) = &ctx.usage_meter {
        let key = UsageKey {
            run_id: ctx.run_id.clone(),
            agent_id: agent_id.clone(),
            domain_id: ctx.domain_id.clone(),
            task_id: TaskId(task.task_id.0.clone()),
        };
        meter.record_context_arbitrage(&key, &bundle.arbitrage)?;
    }

    if ctx.signal_core && bundle.fallback_count > 0 {
        if let Some(cb) = ctx.on_event.as_ref() {
            cb(
                &agent_key,
                "signal-fallback",
                &format!(
                    "signal core raw fallback for {} of {} scaffolded path(s)",
                    bundle.fallback_count,
                    bundle.arbitrage.len()
                ),
            );
        }
    }

    let context_dir = bundle.context_dir;

    registry.register(ChildRecord {
        run_id: ctx.run_id.clone(),
        agent_id: agent_id.clone(),
        worktree_path: wt_path.clone(),
        branch: branch.clone(),
        pid: None,
    });

    let mut effective_backend = ctx.execution_backend;
    let mut sandbox_id: Option<String> = None;
    if ctx.execution_backend == ExecutionBackend::Cloud {
        if let Some(dispatcher) = ctx.cloud_dispatcher.as_ref() {
            match dispatcher.start_sandbox(&StartSandboxRequest {
                domain_id: ctx.domain_id.0.clone(),
                run_id: ctx.run_id.0.clone(),
                agent_id: agent_id.0.clone(),
                repo_fingerprint: eff_repo_root.to_string_lossy().into_owned(),
            }) {
                Ok(start) => {
                    sandbox_id = Some(start.sandbox_id.clone());
                    if let Ok(files) =
                        pytxo_core::collect_sync_paths(&eff_repo_root, &ctx.sparse_exclude)
                    {
                        if !files.is_empty() {
                            let _ = dispatcher.sync_delta(&start.sandbox_id, &files);
                        }
                    }
                }
                Err(e) if ctx.cloud_fallback_local => {
                    if let Some(cb) = ctx.on_event.as_ref() {
                        cb(&agent_key, "cloud-fallback", &format!("{e}"));
                    }
                    effective_backend = ExecutionBackend::Pty;
                }
                Err(e) => return Err(e),
            }
        } else if ctx.cloud_fallback_local {
            effective_backend = ExecutionBackend::Pty;
        } else {
            return Err(PytxoError::Runner(
                "cloud execution requires cloud_dispatcher".into(),
            ));
        }
    }

    if let (Some(ref sid), Some(dispatcher)) = (&sandbox_id, ctx.cloud_dispatcher.as_ref()) {
        if let Some(delta_result) = overlay_upper_cloud_delta(&eff_repo_root, &wt_path) {
            match delta_result {
                Ok(delta) if !delta.files.is_empty() => {
                    match dispatcher.sync_delta(sid, &delta.files) {
                        Ok(()) => {
                            if let Some(cb) = ctx.on_event.as_ref() {
                                cb(
                                    &agent_key,
                                    "cloud-delta",
                                    &format!(
                                        "synced {} file(s) fingerprint={}",
                                        delta.files.len(),
                                        delta.fingerprint
                                    ),
                                );
                            }
                        }
                        Err(e) if ctx.cloud_fallback_local => {
                            if let Some(cb) = ctx.on_event.as_ref() {
                                cb(&agent_key, "cloud-fallback", &format!("delta sync: {e}"));
                            }
                            effective_backend = ExecutionBackend::Pty;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) if ctx.cloud_fallback_local => {
                    if let Some(cb) = ctx.on_event.as_ref() {
                        cb(&agent_key, "cloud-fallback", &format!("delta: {e}"));
                    }
                    effective_backend = ExecutionBackend::Pty;
                }
                Err(e) => return Err(e),
                _ => {}
            }
        }
    }

    if ctx.mcp_hub_enabled {
        if let Some(hub) = &ctx.mcp_hub {
            if mcp_cmd_allowed(&ctx.cmd, &ctx.mcp_allowlist) {
                if let Ok((session, _handle)) = crate::mcp_hub::spawn_agent_mcp_child()
                    .or_else(|_| crate::mcp_hub::spawn_test_mcp_child())
                {
                    hub.register(&agent_key, session);
                }
            }
        }
    }

    let cmd = resolve_cmd_for_task(ctx, task);
    let net = engine.network();
    if !net.spawn_egress_allowed(&cmd) {
        return Err(PytxoError::Runner(format!(
            "network egress denied for {} profile",
            profile.as_str()
        )));
    }
    if command_implies_egress(&cmd) && !net.egress_allowed("1.1.1.1", 443) {
        return Err(PytxoError::Runner(format!(
            "runtime TCP egress denied for {} profile",
            profile.as_str()
        )));
    }
    if profile == PermissionProfile::DeepSpace {
        if net.egress_allowed("1.1.1.1", 443) {
            return Err(PytxoError::Runner(
                "DeepSpace network policy misconfigured: egress must be blocked".into(),
            ));
        }
        if let Some(cb) = ctx.on_event.as_ref() {
            cb(
                &agent_key,
                "network-isolation",
                &format!("deepspace-v2:{}", crate::network_isolation::isolation_mechanism()),
            );
        }
    }
    crate::hitl_gate::gate_spawn_command(ctx.hitl.as_ref(), profile, &agent_key, &cmd)?;

    let result = tokio::task::spawn_blocking({
        let wt_path = wt_path.clone();
        let cmd = cmd.clone();
        let on_event = ctx.on_event.clone();
        let agent_key = agent_key.clone();
        let run_id = ctx.run_id.0.clone();
        let repo_root = ctx.repo_root.to_string_lossy().to_string();
        let data_dir = ctx.data_dir.clone();
        let branch = branch.clone();
        let context_dir = context_dir.clone();
        let managed_transport = ctx.managed_transport.clone();
        let route = route.clone();
        let swarm = swarm.clone();
        let execution_backend = effective_backend;
        let pty_rows = ctx.pty_rows;
        let pty_cols = ctx.pty_cols;
        let subprocess_stdin = ctx.subprocess_stdin;
        let cloud_dispatcher = ctx.cloud_dispatcher.clone();
        let cloud_fallback_local = ctx.cloud_fallback_local;
        let sandbox_id = sandbox_id.clone();
        move || {
            let out = run_command_streaming(
                &wt_path,
                &cmd,
                on_event.as_ref(),
                &agent_key,
                context_dir.as_deref(),
                profile,
                &managed_transport,
                &route,
                execution_backend,
                pty_rows,
                pty_cols,
                &swarm,
                subprocess_stdin,
                cloud_dispatcher.as_deref(),
                sandbox_id.as_deref(),
                cloud_fallback_local,
                Some(ProcessPersist {
                    run_id,
                    repo_root,
                    data_dir,
                    branch,
                }),
            );
            if let Ok(ref res) = out {
                if let Some(pid) = res.pid {
                    swarm.register_pid(&agent_key, pid);
                }
            }
            out
        }
    })
    .await
    .map_err(|e| PytxoError::Runner(format!("join: {e}")))??;

    let mut result = result;
    if result.exit_code != Some(0)
        && ctx.signal_core
        && fidelity != FidelityTier::High
        && !context_paths.is_empty()
    {
        // Closed-loop v2: escalate only the paths the failure implicates; fall
        // back to the full context surface when the classifier is unsure.
        let combined_output = format!("{}\n{}", result.stdout, result.stderr);
        let implicated = implicated_paths(&combined_output, &context_paths);
        let retry_paths_vec: Vec<String> = if implicated.is_empty() {
            context_paths.clone()
        } else {
            let edited: Vec<(String, String, Option<String>)> = implicated
                .iter()
                .map(|p| (p.clone(), agent_key.clone(), task.root.clone()))
                .collect();
            pytxo_signal::graph_neighbor_paths(&eff_repo_root, &edited, &implicated)
        };
        let retry_paths: &[String] = &retry_paths_vec;

        let high_bundle = prepare_agent_context_for_root(
            &eff_repo_root,
            &ctx.data_dir,
            &ctx.run_id,
            &agent_id.0,
            retry_paths,
            true,
            FidelityTier::High,
            ctx.token_estimator.as_ref(),
            &route.model,
            task.root.as_deref(),
            cache_ref,
            &ctx.domain_id.0,
            ctx.cloud_cache_enabled,
            profile,
        )?;
        if let Some(meter) = &ctx.usage_meter {
            let key = UsageKey {
                run_id: ctx.run_id.clone(),
                agent_id: agent_id.clone(),
                domain_id: ctx.domain_id.clone(),
                task_id: TaskId(task.task_id.0.clone()),
            };
            meter.record_context_arbitrage(&key, &high_bundle.arbitrage)?;
        }
        if let Some(cb) = ctx.on_event.as_ref() {
            cb(
                &agent_key,
                "signal-retry",
                &format!(
                    "closed-loop retry at high fidelity over {} of {} path(s)",
                    retry_paths.len(),
                    context_paths.len()
                ),
            );
        }
        let context_dir = high_bundle.context_dir;
        let execution_backend = effective_backend;
        let pty_rows = ctx.pty_rows;
        let pty_cols = ctx.pty_cols;
        let subprocess_stdin = ctx.subprocess_stdin;
        let cloud_dispatcher = ctx.cloud_dispatcher.clone();
        let cloud_fallback_local = ctx.cloud_fallback_local;
        let sandbox_id = sandbox_id.clone();
        let retry = tokio::task::spawn_blocking({
            let wt_path = wt_path.clone();
            let cmd = ctx.cmd.clone();
            let on_event = ctx.on_event.clone();
            let agent_key = agent_key.clone();
            let run_id = ctx.run_id.0.clone();
            let repo_root = ctx.repo_root.to_string_lossy().to_string();
            let data_dir = ctx.data_dir.clone();
            let branch = branch.clone();
            let managed_transport = ctx.managed_transport.clone();
            let route = route.clone();
            let swarm = swarm.clone();
            move || {
                run_command_streaming(
                    &wt_path,
                    &cmd,
                    on_event.as_ref(),
                    &agent_key,
                    context_dir.as_deref(),
                    profile,
                    &managed_transport,
                    &route,
                    execution_backend,
                    pty_rows,
                    pty_cols,
                    &swarm,
                    subprocess_stdin,
                    cloud_dispatcher.as_deref(),
                    sandbox_id.as_deref(),
                    cloud_fallback_local,
                    Some(ProcessPersist {
                        run_id,
                        repo_root,
                        data_dir,
                        branch,
                    }),
                )
            }
        })
        .await
        .map_err(|e| PytxoError::Runner(format!("join: {e}")))??;
        result = retry;
    }

    swarm.release(&agent_key);

    if let Some(hub) = &ctx.mcp_hub {
        hub.deregister(&agent_key);
    }
    if let (Some(sid), Some(dispatcher)) = (sandbox_id.as_ref(), ctx.cloud_dispatcher.as_ref()) {
        let _ = dispatcher.teardown(sid);
    }

    if used_isolation && !ctx.keep_worktrees {
        let _ = isolation.rollback(&iso_ctx, &workspace);
    }

    Ok(AgentRunResult {
        agent_id: agent_id.clone(),
        task_id: task.task_id.0.clone(),
        wave: task.wave,
        worktree_path: result.worktree_path,
        exit_code: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
        root_id: task.root.clone(),
    })
}

struct ProcessPersist {
    run_id: String,
    repo_root: String,
    data_dir: PathBuf,
    branch: String,
}

fn resolve_task_root(task: &ScheduledTask, ctx: &RunContext) -> Result<(PathBuf, PathBuf)> {
    match task.root.as_deref() {
        Some(label) if !label.is_empty() => {
            let root = ctx.roots.get(label).ok_or_else(|| {
                PytxoError::Runner(format!(
                    "task {} references unknown root label {label:?} (available: {:?})",
                    task.task_id.0,
                    ctx.roots.keys().collect::<Vec<_>>()
                ))
            })?;
            if root.read_only {
                return Err(PytxoError::Runner(format!(
                    "task {} cannot execute on read-only root {label:?}",
                    task.task_id.0,
                )));
            }
            Ok((root.repo_root.clone(), root.worktree_base.clone()))
        }
        _ => Ok((ctx.repo_root.clone(), ctx.worktree_base.clone())),
    }
}

fn minimal_config_for_route(ctx: &RunContext) -> pytxo_core::PytxoConfig {
    pytxo_core::PytxoConfig {
        permission_profile: ctx.permission_profile,
        agent: ctx.route_agents.clone(),
        ..Default::default()
    }
}

fn build_child_env(
    context_dir: Option<&Path>,
    profile: PermissionProfile,
    managed_transport: &ManagedTransport,
    route: &ModelRoute,
) -> ChildLaunchEnv {
    let mut env = ChildLaunchEnv::new();
    if let Some(dir) = context_dir {
        env = env.with_context_dir(dir);
    }
    env.with_managed_and_profile(managed_transport, route, &PermissionEngine::new(profile))
}

#[allow(clippy::too_many_arguments)]
fn run_command_streaming(
    worktree: &Path,
    cmd: &str,
    on_event: Option<&EventCallback>,
    agent_key: &str,
    context_dir: Option<&Path>,
    profile: PermissionProfile,
    managed_transport: &ManagedTransport,
    route: &ModelRoute,
    execution_backend: ExecutionBackend,
    pty_rows: u16,
    pty_cols: u16,
    swarm: &SwarmRegistry,
    subprocess_stdin: bool,
    cloud_dispatcher: Option<&dyn CloudDispatcher>,
    cloud_sandbox_id: Option<&str>,
    cloud_fallback_local: bool,
    persist: Option<ProcessPersist>,
) -> Result<SingleResult> {
    let env = build_child_env(context_dir, profile, managed_transport, route);

    if execution_backend == ExecutionBackend::Cloud {
        if let (Some(dispatcher), Some(sid)) = (cloud_dispatcher, cloud_sandbox_id) {
            match dispatcher.exec(
                sid,
                &ExecRequest {
                    cmd: cmd.to_string(),
                    cwd: Some(worktree.to_string_lossy().into_owned()),
                },
            ) {
                Ok(resp) => {
                    if let Some(cb) = on_event {
                        cb(agent_key, "cloud-exec", &format!("exit={}", resp.exit_code));
                        for line in resp.stdout.lines() {
                            cb(agent_key, "stdout", line);
                        }
                        for line in resp.stderr.lines() {
                            cb(agent_key, "stderr", line);
                        }
                    }
                    return Ok(SingleResult {
                        worktree_path: worktree.to_path_buf(),
                        exit_code: Some(resp.exit_code),
                        stdout: resp.stdout,
                        stderr: resp.stderr,
                        pid: None,
                    });
                }
                Err(e) if cloud_fallback_local => {
                    if let Some(cb) = on_event {
                        cb(agent_key, "cloud-fallback", &format!("exec: {e}"));
                    }
                }
                Err(e) => return Err(e),
            }
        } else if !cloud_fallback_local {
            return Err(PytxoError::Runner(
                "cloud exec missing sandbox_id or dispatcher".into(),
            ));
        }
    }

    if execution_backend == ExecutionBackend::Pty
        || (execution_backend == ExecutionBackend::Cloud && cloud_fallback_local)
    {
        let effective_cmd = if profile == PermissionProfile::DeepSpace {
            crate::network_isolation::wrap_deepspace_shell_cmd(cmd)
        } else {
            cmd.to_string()
        };
        let result = crate::pty::run_pty_session(
            worktree,
            &effective_cmd,
            env,
            pty_rows,
            pty_cols,
            on_event,
            agent_key,
            swarm,
        )?;
        if let Some(p) = persist {
            persist_process(&p, agent_key, worktree, &p.branch, result.pid)?;
        }
        return Ok(result);
    }

    let shell = shell_command();
    let mut command = std::process::Command::new(&shell.0);
    command
        .args(&shell.1)
        .arg(cmd)
        .current_dir(worktree)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if subprocess_stdin {
        command.stdin(Stdio::piped());
    }
    env.apply_command(&mut command);
    if profile == PermissionProfile::DeepSpace {
        crate::network_isolation::isolate_deepspace_network(&mut command)?;
    }
    let mut child = command
        .spawn()
        .map_err(|e| PytxoError::Runner(format!("spawn command: {e}")))?;

    if subprocess_stdin {
        let pending = swarm.drain_stdin(agent_key);
        if !pending.is_empty() {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(&pending);
                let _ = stdin.flush();
            }
        }
    }

    let pid = child.id();
    if let Some(p) = persist {
        persist_process(&p, agent_key, worktree, &p.branch, Some(pid))?;
    }
    let mut stdout_acc = String::new();
    let mut stderr_acc = String::new();

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let on_event_out = on_event.cloned();
    let on_event_err = on_event.cloned();
    let agent_key_out = agent_key.to_string();
    let agent_key_err = agent_key.to_string();

    let out_handle = stdout.map(|out| {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            let mut acc = String::new();
            for line in reader.lines().map_while(|line| line.ok()) {
                if let Some(cb) = &on_event_out {
                    cb(&agent_key_out, "stdout", &line);
                }
                acc.push_str(&line);
                acc.push('\n');
            }
            acc
        })
    });

    let err_handle = stderr.map(|err| {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            let mut acc = String::new();
            for line in reader.lines().map_while(|line| line.ok()) {
                if let Some(cb) = &on_event_err {
                    cb(&agent_key_err, "stderr", &line);
                }
                acc.push_str(&line);
                acc.push('\n');
            }
            acc
        })
    });

    let status = child
        .wait()
        .map_err(|e| PytxoError::Runner(format!("wait: {e}")))?;

    if let Some(h) = out_handle {
        if let Ok(acc) = h.join() {
            stdout_acc = acc;
        }
    }
    if let Some(h) = err_handle {
        if let Ok(acc) = h.join() {
            stderr_acc = acc;
        }
    }

    Ok(SingleResult {
        worktree_path: worktree.to_path_buf(),
        exit_code: status.code(),
        stdout: stdout_acc,
        stderr: stderr_acc,
        pid: Some(pid),
    })
}

fn persist_process(
    p: &ProcessPersist,
    agent_key: &str,
    worktree: &Path,
    branch: &str,
    pid: Option<u32>,
) -> Result<()> {
    let pid = pid.ok_or_else(|| PytxoError::Runner("missing child pid".into()))?;
    let mut proc_file = ProcessRegistryFile::load(&registry_path(&p.data_dir))?;
    proc_file.push(ProcessEntry {
        run_id: p.run_id.clone(),
        repo_root: p.repo_root.clone(),
        agent_key: agent_key.to_string(),
        pid,
        worktree_path: worktree.to_string_lossy().to_string(),
        branch: branch.to_string(),
    });
    proc_file.save(&registry_path(&p.data_dir))?;
    Ok(())
}

/// Resolve the shell command for one scheduled task (Hypervisor Shell templates).
pub fn resolve_cmd_for_task(ctx: &RunContext, task: &pytxo_core::ScheduledTask) -> String {
    if let Some(template) = &ctx.task_cmd_template {
        let prompt = ctx
            .task_prompts
            .get(&task.task_id.0)
            .map(String::as_str)
            .unwrap_or("");
        let paths = task.paths.join(",");
        return template
            .replace("{task_id}", &task.task_id.0)
            .replace("{agent}", &task.agent)
            .replace("{paths}", &paths)
            .replace("{prompt}", prompt)
            .replace("{wave}", &task.wave.to_string());
    }
    ctx.cmd.clone()
}

fn command_implies_egress(cmd: &str) -> bool {
    let lower = cmd.to_ascii_lowercase();
    lower.contains("curl ")
        || lower.contains("wget ")
        || lower.contains("nc ")
        || lower.contains("ncat ")
        || lower.starts_with("ssh ")
}

fn mcp_cmd_allowed(cmd: &str, allowlist: &[String]) -> bool {
    if allowlist.is_empty() {
        return true;
    }
    allowlist.iter().any(|prefix| cmd.contains(prefix))
}

pub(crate) fn shell_command() -> (String, Vec<String>) {
    if cfg!(windows) {
        ("cmd".into(), vec!["/C".into()])
    } else {
        ("sh".into(), vec!["-c".into()])
    }
}

pub fn stop_run(data_dir: &Path, run_id: &str, kill: bool) -> Result<Vec<u32>> {
    use crate::kill::kill_pids;

    let path = registry_path(data_dir);
    let mut file = ProcessRegistryFile::load(&path)?;
    let pids: Vec<u32> = file.for_run(run_id).iter().map(|e| e.pid).collect();
    if kill {
        kill_pids(&pids)?;
    }
    file.remove_run(run_id);
    file.save(&path)?;
    Ok(pids)
}

pub fn stop_all(data_dir: &Path, kill: bool) -> Result<()> {
    use crate::kill::kill_pids;

    let path = registry_path(data_dir);
    let mut file = ProcessRegistryFile::load(&path)?;
    let pids = file.all_pids();
    if kill {
        kill_pids(&pids)?;
    }
    file.clear();
    file.save(&path)?;
    Ok(())
}

pub fn cleanup_worktrees(ctx: &RunContext, registry: &ProcessRegistry) -> Result<()> {
    for record in registry.list() {
        if record.run_id == ctx.run_id {
            let _ = remove_worktree(&ctx.repo_root, &record.worktree_path, &record.branch, true);
        }
    }
    Ok(())
}

/// Default window a Blast flush waits for a human HITL decision before timing out.
const HITL_FLUSH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// Persist approved agent mutations (Blast Shield flush).
///
/// When `ctx.hitl` is set and the profile's `flush_requires_approval()` is true
/// (Orbit/Galaxy), the flush submits an approval request and blocks until a human
/// resolves it via the queue (Deck/CLI `hitl_respond`). The manual
/// `commit_workspace_for_agent` path passes no queue: that call is itself the
/// human approval, so it flushes directly.
pub fn commit_workspace(
    ctx: &RunContext,
    workspace: &WorkspaceHandle,
    profile: PermissionProfile,
) -> Result<()> {
    let engine = PermissionEngine::new(profile);
    if !engine.may_flush() {
        return Err(PytxoError::Runner(
            "flush denied for permission profile".into(),
        ));
    }
    if engine.flush_requires_approval() {
        if let Some(hitl) = ctx.hitl.as_ref() {
            let agent_key = if workspace.branch.is_empty() {
                ctx.run_id.0.clone()
            } else {
                workspace.branch.clone()
            };
            if profile == PermissionProfile::Galaxy
                && crate::hitl_gate::workspace_writes_outside_root(
                    &workspace.cwd,
                    &ctx.repo_root,
                )
            {
                crate::hitl_gate::gate_hitl_action(
                    Some(hitl),
                    profile,
                    &agent_key,
                    "fs.write_outside_root",
                    "flush would write outside repository root",
                )?;
            }
            let id = hitl.submit(
                &agent_key,
                "blast.flush",
                "commit agent workspace to repo root",
            );
            match hitl.wait_blocking(&id, HITL_FLUSH_TIMEOUT) {
                crate::hitl::HitlDecision::Approved => {}
                crate::hitl::HitlDecision::Denied => {
                    return Err(PytxoError::Runner("flush denied by human reviewer".into()));
                }
                crate::hitl::HitlDecision::Pending => {
                    return Err(PytxoError::Runner("flush approval timed out".into()));
                }
            }
        }
    }
    let isolation = crate::blast::isolation_for_mode(ctx.isolation_mode);
    let iso_ctx = IsolationCtx {
        run_id: ctx.run_id.clone(),
        agent_id: AgentId::new(0),
        repo_root: ctx.repo_root.clone(),
        worktree_base: ctx.worktree_base.clone(),
        sparse_exclude: ctx.sparse_exclude.clone(),
    };
    isolation.flush(&iso_ctx, workspace)
}
