use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader, Read};
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use pytxo_core::{
    overlay_upper_cloud_delta, root_scoped_claim, AgentId, BillingMode, ByteHeuristicEstimator,
    ChildLaunchEnv, CloudDispatcher, ConfigModelRouter, ContextCache, DomainId, ExecRequest,
    ExecutionBackend, ExecutionPlan, FidelityTier, IsolationCtx, IsolationMode, ManagedTransport,
    ModelRoute, ModelRouter, NetworkPolicy, PermissionEngine, PermissionProfile, PytxoError,
    RaceShield, Result, RunId, ScheduledTask, StartSandboxRequest, TaskId, TokenEstimator,
    UsageKey, UsageMeter, WorkspaceHandle,
};

use crate::context::{extend_context_with_readonly_roots, prepare_agent_context_for_root};
use crate::failure::implicated_paths;
use crate::git::remove_worktree;
use crate::process::{ChildRecord, ProcessRegistry};
use crate::process_registry_file::{registry_path, ProcessEntry, ProcessRegistryFile};
use crate::race::SwarmRegistry;

pub type EventCallback = Arc<dyn Fn(&str, &str, &str) + Send + Sync>;

/// Execution-domain context for checking an assembled candidate with the same
/// permission gates, cancellation registry, and process supervision as task checks.
#[derive(Clone)]
pub struct CandidateCheckContext {
    pub cwd: PathBuf,
    pub run_id: String,
    pub agent_key: String,
    pub repo_root: PathBuf,
    pub data_dir: PathBuf,
    pub profile: PermissionProfile,
    pub domain_id: DomainId,
    pub execution_backend: ExecutionBackend,
    pub workspace_isolated: bool,
    pub hitl: Option<crate::hitl::HitlQueue>,
    pub swarm: SwarmRegistry,
    pub on_event: Option<EventCallback>,
}

/// Blocking check; async callers must use `spawn_blocking`.
pub fn run_candidate_check(
    ctx: &CandidateCheckContext,
    command: &str,
) -> Result<crate::VerificationEnforcementReceipt> {
    let ctx = ctx.clone();
    let command = command.to_owned();
    {
        let receipt = crate::enforcement::verification_enforcement_receipt(
            ctx.profile,
            &ctx.domain_id,
            ctx.execution_backend,
            ctx.workspace_isolated,
            VERIFY_TIMEOUT,
            VERIFY_OUTPUT_LIMIT_BYTES,
        )?;
        let lifecycle = VerificationLifecycle {
            persist: ProcessPersist {
                run_id: ctx.run_id,
                repo_root: ctx.repo_root.to_string_lossy().into_owned(),
                data_dir: ctx.data_dir,
                branch: String::new(),
            },
            swarm: ctx.swarm,
        };
        run_verify_commands(
            &ctx.cwd,
            &[command],
            ctx.on_event.as_ref(),
            &ctx.agent_key,
            ctx.profile,
            &ctx.domain_id,
            ctx.execution_backend,
            ctx.workspace_isolated,
            ctx.hitl.as_ref(),
            Some(&lifecycle),
        )?;
        Ok(receipt)
    }
}

/// One modular-project root's execution surface ([[ADR-0011-modular-project-manifest]]).
#[derive(Clone, Debug)]
pub struct RootExec {
    pub repo_root: PathBuf,
    pub worktree_base: PathBuf,
    pub read_only: bool,
    /// Manifest-requested profile before folder/org trust ceilings.
    pub requested_permission_profile: PermissionProfile,
    pub permission_profile: PermissionProfile,
}

#[derive(Clone)]
pub struct RunContext {
    pub run_id: RunId,
    pub repo_root: PathBuf,
    pub worktree_base: PathBuf,
    pub data_dir: PathBuf,
    pub cmd: String,
    /// When set, expands non-prompt metadata via [`resolve_cmd_for_task`]. Prompt text is passed
    /// separately in `PYTXO_TASK_PROMPT` and cannot be interpolated into shell syntax.
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
    /// When true, `commit_workspace` skips the HITL queue (Deck/CLI Approve merge
    /// is itself the human act). Automated run flushes must leave this false.
    pub hitl_manual_flush: bool,
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
    pub worktree_path: Option<PathBuf>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub outcome: AgentRunOutcome,
    /// Modular project root label this agent ran under ([[ADR-0011-modular-project-manifest]]).
    pub root_id: Option<String>,
}

/// Terminal task outcome. Process completion alone is not dependency success:
/// verification must also pass before a workspace can be composed downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentRunOutcome {
    Succeeded,
    ProcessFailed,
    VerificationFailed,
    BlockedByDependency { task_ids: Vec<String> },
}

impl AgentRunOutcome {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Succeeded)
    }

    pub fn ledger_status(&self) -> &'static str {
        match self {
            Self::Succeeded => "completed",
            Self::ProcessFailed => "failed",
            Self::VerificationFailed => "verify_failed",
            Self::BlockedByDependency { .. } => "blocked_by_dependency",
        }
    }
}

pub struct SingleResult {
    pub worktree_path: PathBuf,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub pid: Option<u32>,
}

#[derive(Clone, Debug)]
struct DependencyOutput {
    task_id: String,
    workspace_path: PathBuf,
    paths: Vec<String>,
    root_id: Option<String>,
}

struct CloudSandboxGuard {
    dispatcher: Arc<dyn CloudDispatcher>,
    sandbox_id: String,
}

impl Drop for CloudSandboxGuard {
    fn drop(&mut self) {
        let _ = self.dispatcher.teardown(&self.sandbox_id);
    }
}

#[derive(Clone, Debug)]
enum DependencyChange {
    Copy(PathBuf),
    Delete,
}

pub async fn execute_plan(
    ctx: &RunContext,
    plan: &ExecutionPlan,
    registry: &ProcessRegistry,
    swarm: &SwarmRegistry,
) -> Result<Vec<AgentRunResult>> {
    let mut all_results = Vec::new();
    let mut completed: HashMap<String, DependencyOutput> = HashMap::new();
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
            let blocked_by = task
                .depends_on
                .iter()
                .filter(|dependency_id| !completed.contains_key(*dependency_id))
                .cloned()
                .collect::<Vec<_>>();
            if !blocked_by.is_empty() {
                all_results.push(AgentRunResult {
                    agent_id,
                    task_id: task.task_id.0,
                    wave: task.wave,
                    worktree_path: None,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: format!(
                        "blocked by unsuccessful dependency: {}",
                        blocked_by.join(", ")
                    ),
                    outcome: AgentRunOutcome::BlockedByDependency {
                        task_ids: blocked_by,
                    },
                    root_id: task.root,
                });
                continue;
            }
            let dependencies = task
                .depends_on
                .iter()
                .filter_map(|dependency_id| completed.get(dependency_id).cloned())
                .collect::<Vec<_>>();
            set.spawn(async move {
                let failure_agent_id = agent_id.clone();
                let failure_task_id = task.task_id.0.clone();
                let failure_wave = task.wave;
                let failure_root = task.root.clone();
                let agent_key = format!("{}:{}", ctx.run_id, agent_id);
                let cleanup_ctx = ctx.clone();
                let cleanup_registry = registry.clone();
                let cleanup_swarm = swarm.clone();
                let worker = tokio::spawn(async move {
                    run_one_agent(&ctx, &task, &agent_id, &registry, &swarm, &dependencies).await
                });
                match worker.await {
                    Ok(Ok(result)) => result,
                    Ok(Err(error)) => failed_agent_result(
                        &cleanup_ctx,
                        &cleanup_registry,
                        &cleanup_swarm,
                        &agent_key,
                        failure_agent_id,
                        failure_task_id,
                        failure_wave,
                        failure_root,
                        format!("agent lifecycle failed: {error}"),
                    ),
                    Err(error) => failed_agent_result(
                        &cleanup_ctx,
                        &cleanup_registry,
                        &cleanup_swarm,
                        &agent_key,
                        failure_agent_id,
                        failure_task_id,
                        failure_wave,
                        failure_root,
                        format!("agent lifecycle task aborted: {error}"),
                    ),
                }
            });
        }

        let mut wave_results = Vec::new();
        let mut join_errors = Vec::new();
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(result) => wave_results.push(result),
                Err(error) => join_errors.push(error.to_string()),
            }
        }
        if !join_errors.is_empty() {
            return Err(PytxoError::Runner(format!(
                "agent supervisor task failed after draining its wave: {}",
                join_errors.join("; ")
            )));
        }
        for result in wave_results {
            let task = wave
                .iter()
                .find(|task| task.task_id.0 == result.task_id)
                .ok_or_else(|| {
                    PytxoError::Runner(format!(
                        "completed task {} was not present in its execution wave",
                        result.task_id
                    ))
                })?;
            if result.outcome.is_success() {
                let workspace_path = result.worktree_path.clone().ok_or_else(|| {
                    PytxoError::Runner(format!(
                        "successful task {} has no workspace output",
                        result.task_id
                    ))
                })?;
                completed.insert(
                    result.task_id.clone(),
                    DependencyOutput {
                        task_id: result.task_id.clone(),
                        workspace_path,
                        paths: task.paths.clone(),
                        root_id: result.root_id.clone(),
                    },
                );
            }
            all_results.push(result);
        }
    }

    ProcessRegistryFile::update(&registry_path(&ctx.data_dir), |registry| {
        registry.remove_run(&ctx.run_id.0);
        Ok(())
    })?;

    Ok(all_results)
}

#[allow(clippy::too_many_arguments)]
fn failed_agent_result(
    ctx: &RunContext,
    registry: &ProcessRegistry,
    swarm: &SwarmRegistry,
    agent_key: &str,
    agent_id: AgentId,
    task_id: String,
    wave: u32,
    root_id: Option<String>,
    message: String,
) -> AgentRunResult {
    swarm.release(agent_key);
    if let Some(hub) = &ctx.mcp_hub {
        hub.deregister(agent_key);
    }
    if let Some(callback) = ctx.on_event.as_ref() {
        callback(agent_key, "agent-lifecycle-failed", &message);
    }
    let worktree_path = registry
        .list()
        .into_iter()
        .find(|record| record.run_id == ctx.run_id && record.agent_id == agent_id)
        .map(|record| record.worktree_path);
    AgentRunResult {
        agent_id,
        task_id,
        wave,
        worktree_path,
        exit_code: None,
        stdout: String::new(),
        stderr: message,
        outcome: AgentRunOutcome::ProcessFailed,
        root_id,
    }
}

fn compose_dependency_outputs(
    base: &Path,
    destination: &Path,
    dependencies: &[DependencyOutput],
    sparse_exclude: &[String],
) -> Result<usize> {
    let mut changes = BTreeMap::<PathBuf, DependencyChange>::new();
    for dependency in dependencies {
        collect_claimed_changes(
            base,
            &dependency.workspace_path,
            &dependency.paths,
            sparse_exclude,
            &mut changes,
        )?;
    }

    for (relative, change) in &changes {
        let target = destination.join(relative);
        match change {
            DependencyChange::Delete => {
                if target.is_file() {
                    std::fs::remove_file(&target).map_err(|error| {
                        PytxoError::Runner(format!(
                            "remove inherited dependency file {}: {error}",
                            target.display()
                        ))
                    })?;
                }
            }
            DependencyChange::Copy(source) => {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).map_err(|error| {
                        PytxoError::Runner(format!(
                            "create dependency output directory {}: {error}",
                            parent.display()
                        ))
                    })?;
                }
                std::fs::copy(source, &target).map_err(|error| {
                    PytxoError::Runner(format!(
                        "compose dependency output {} -> {}: {error}",
                        source.display(),
                        target.display()
                    ))
                })?;
            }
        }
    }
    Ok(changes.len())
}

fn collect_claimed_changes(
    base: &Path,
    workspace: &Path,
    claims: &[String],
    sparse_exclude: &[String],
    changes: &mut BTreeMap<PathBuf, DependencyChange>,
) -> Result<()> {
    for claim in claims {
        let relative = Path::new(claim);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(PytxoError::Runner(format!(
                "unsafe dependency claim: {claim}"
            )));
        }

        if claim.contains('*') || claim.contains('?') || claim.contains('[') {
            let workspace_pattern = workspace
                .join(relative)
                .to_string_lossy()
                .replace('\\', "/");
            let entries = glob::glob(&workspace_pattern)
                .map_err(|error| PytxoError::Runner(format!("dependency glob {claim}: {error}")))?;
            for entry in entries {
                let path = entry.map_err(|error| {
                    PytxoError::Runner(format!("dependency glob entry {claim}: {error}"))
                })?;
                collect_changed_path(base, workspace, &path, sparse_exclude, changes)?;
            }

            let base_pattern = base.join(relative).to_string_lossy().replace('\\', "/");
            let base_entries = glob::glob(&base_pattern)
                .map_err(|error| PytxoError::Runner(format!("dependency glob {claim}: {error}")))?;
            for entry in base_entries {
                let path = entry.map_err(|error| {
                    PytxoError::Runner(format!("dependency base glob entry {claim}: {error}"))
                })?;
                collect_missing_files(base, workspace, &path, sparse_exclude, changes)?;
            }
            continue;
        }

        let source = workspace.join(relative);
        if source.exists() {
            collect_changed_path(base, workspace, &source, sparse_exclude, changes)?;
            let base_claim = base.join(relative);
            if base_claim.exists() {
                collect_missing_files(base, workspace, &base_claim, sparse_exclude, changes)?;
            }
        } else if base.join(relative).is_file()
            && !dependency_path_ignored(relative, sparse_exclude)
        {
            changes.insert(relative.to_path_buf(), DependencyChange::Delete);
        }
    }
    Ok(())
}

fn collect_changed_path(
    base: &Path,
    workspace: &Path,
    path: &Path,
    sparse_exclude: &[String],
    changes: &mut BTreeMap<PathBuf, DependencyChange>,
) -> Result<()> {
    let relative = path.strip_prefix(workspace).map_err(|error| {
        PytxoError::Runner(format!(
            "dependency output {} escaped workspace {}: {error}",
            path.display(),
            workspace.display()
        ))
    })?;
    if dependency_path_ignored(relative, sparse_exclude) {
        return Ok(());
    }

    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        PytxoError::Runner(format!(
            "read dependency output metadata {}: {error}",
            path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in std::fs::read_dir(path).map_err(|error| {
            PytxoError::Runner(format!(
                "read dependency output directory {}: {error}",
                path.display()
            ))
        })? {
            let entry = entry.map_err(|error| {
                PytxoError::Runner(format!("read dependency output entry: {error}"))
            })?;
            collect_changed_path(base, workspace, &entry.path(), sparse_exclude, changes)?;
        }
        return Ok(());
    }
    if !metadata.is_file() {
        return Ok(());
    }

    let changed = match (std::fs::read(path), std::fs::read(base.join(relative))) {
        (Ok(candidate), Ok(original)) => candidate != original,
        (Ok(_), Err(error)) if error.kind() == std::io::ErrorKind::NotFound => true,
        (Err(error), _) => {
            return Err(PytxoError::Runner(format!(
                "read dependency output {}: {error}",
                path.display()
            )))
        }
        (_, Err(error)) => {
            return Err(PytxoError::Runner(format!(
                "read dependency base {}: {error}",
                base.join(relative).display()
            )))
        }
    };
    if changed {
        changes.insert(
            relative.to_path_buf(),
            DependencyChange::Copy(path.to_path_buf()),
        );
    }
    Ok(())
}

fn collect_missing_files(
    base: &Path,
    workspace: &Path,
    path: &Path,
    sparse_exclude: &[String],
    changes: &mut BTreeMap<PathBuf, DependencyChange>,
) -> Result<()> {
    let relative = path.strip_prefix(base).map_err(|error| {
        PytxoError::Runner(format!(
            "dependency base path {} escaped repository {}: {error}",
            path.display(),
            base.display()
        ))
    })?;
    if dependency_path_ignored(relative, sparse_exclude) {
        return Ok(());
    }

    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        PytxoError::Runner(format!(
            "read dependency base metadata {}: {error}",
            path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in std::fs::read_dir(path).map_err(|error| {
            PytxoError::Runner(format!(
                "read dependency base directory {}: {error}",
                path.display()
            ))
        })? {
            let entry = entry.map_err(|error| {
                PytxoError::Runner(format!("read dependency base entry: {error}"))
            })?;
            collect_missing_files(base, workspace, &entry.path(), sparse_exclude, changes)?;
        }
    } else if metadata.is_file() && !workspace.join(relative).is_file() {
        changes.insert(relative.to_path_buf(), DependencyChange::Delete);
    }
    Ok(())
}

fn dependency_path_ignored(relative: &Path, sparse_exclude: &[String]) -> bool {
    let key = relative.to_string_lossy().replace('\\', "/");
    if key == ".git" || key.starts_with(".git/") || key == ".pytxo" || key.starts_with(".pytxo/") {
        return true;
    }
    sparse_exclude.iter().any(|excluded| {
        let excluded = excluded.trim_matches(|character| character == '/' || character == '\\');
        !excluded.is_empty() && (key == excluded || key.starts_with(&format!("{excluded}/")))
    })
}

async fn run_one_agent(
    ctx: &RunContext,
    task: &ScheduledTask,
    agent_id: &AgentId,
    registry: &ProcessRegistry,
    swarm: &SwarmRegistry,
    dependencies: &[DependencyOutput],
) -> Result<AgentRunResult> {
    let agent_key = format!("{}:{}", ctx.run_id, agent_id);
    let claim_paths: Vec<String> = task
        .paths
        .iter()
        .map(|p| root_scoped_claim(task.root.as_deref(), p))
        .collect();
    swarm
        .try_claim_paths(&agent_key, &claim_paths)
        .or_else(|e| {
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

    // Establish the live audit actor before any command can request approval.
    if let Some(callback) = ctx.on_event.as_ref() {
        callback(&agent_key, "agent-start", &task.task_id.0);
    }
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

    let applicable_dependencies = dependencies
        .iter()
        .filter(|dependency| dependency.root_id.as_deref() == task.root.as_deref())
        .cloned()
        .collect::<Vec<_>>();
    if !applicable_dependencies.is_empty() {
        let composed = compose_dependency_outputs(
            &eff_repo_root,
            &wt_path,
            &applicable_dependencies,
            &ctx.sparse_exclude,
        )?;
        if let Some(cb) = ctx.on_event.as_ref() {
            cb(
                &agent_key,
                "dependency-context",
                &format!(
                    "composed {composed} changed file(s) from {}",
                    applicable_dependencies
                        .iter()
                        .map(|dependency| dependency.task_id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }
    }

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
    let mut sandbox_guard: Option<CloudSandboxGuard> = None;
    if ctx.execution_backend == ExecutionBackend::Cloud {
        if let Some(dispatcher) = ctx.cloud_dispatcher.as_ref() {
            match dispatcher.start_sandbox(&StartSandboxRequest {
                domain_id: ctx.domain_id.0.clone(),
                run_id: ctx.run_id.0.clone(),
                agent_id: agent_id.0.clone(),
                repo_fingerprint: pytxo_core::content_hash(
                    eff_repo_root.to_string_lossy().as_bytes(),
                ),
            }) {
                Ok(start) => {
                    let sync_result = (|| {
                        let files =
                            pytxo_core::collect_sync_paths(&eff_repo_root, &ctx.sparse_exclude)?;
                        if files.is_empty() {
                            return Ok(());
                        }
                        let manifest = pytxo_core::cloud_sync_manifest(&files)?;
                        if let Some(cb) = ctx.on_event.as_ref() {
                            let detail = serde_json::to_string(&manifest).map_err(|e| {
                                PytxoError::Other(format!("cloud sync manifest: {e}"))
                            })?;
                            cb(&agent_key, "cloud-sync-manifest", &detail);
                        }
                        dispatcher.sync_delta(&start.sandbox_id, &files)
                    })();
                    match sync_result {
                        Ok(()) => {
                            sandbox_id = Some(start.sandbox_id.clone());
                            sandbox_guard = Some(CloudSandboxGuard {
                                dispatcher: Arc::clone(dispatcher),
                                sandbox_id: start.sandbox_id,
                            });
                        }
                        Err(e) if cloud_fallback_allowed(ctx.cloud_fallback_local, &e) => {
                            let _ = dispatcher.teardown(&start.sandbox_id);
                            if let Some(cb) = ctx.on_event.as_ref() {
                                cb(&agent_key, "cloud-upload-denied", &format!("{e}"));
                            }
                            effective_backend = ExecutionBackend::Pty;
                        }
                        Err(e) => {
                            let _ = dispatcher.teardown(&start.sandbox_id);
                            return Err(e);
                        }
                    }
                }
                Err(e) if cloud_fallback_allowed(ctx.cloud_fallback_local, &e) => {
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
                    let manifest = pytxo_core::cloud_sync_manifest(&delta.files)?;
                    if let Some(cb) = ctx.on_event.as_ref() {
                        let detail = serde_json::to_string(&manifest)
                            .map_err(|e| PytxoError::Other(format!("cloud delta manifest: {e}")))?;
                        cb(&agent_key, "cloud-sync-manifest", &detail);
                    }
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
                        Err(e) if cloud_fallback_allowed(ctx.cloud_fallback_local, &e) => {
                            if let Some(cb) = ctx.on_event.as_ref() {
                                cb(&agent_key, "cloud-fallback", &format!("delta sync: {e}"));
                            }
                            effective_backend = ExecutionBackend::Pty;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) if cloud_fallback_allowed(ctx.cloud_fallback_local, &e) => {
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

    let cmd = resolve_cmd_for_task(ctx, task)?;
    let task_prompt = ctx.task_prompts.get(&task.task_id.0).cloned();
    let net = engine.network();
    if !net.spawn_egress_allowed(&cmd) {
        return Err(PytxoError::Runner(format!(
            "network egress denied for {} profile",
            profile.as_str()
        )));
    }
    // Galaxy: public egress is HITL-gated (not hard-denied) so reviewers can allow one-shot fetches.
    if command_implies_egress(&cmd) && !net.egress_allowed("1.1.1.1", 443) {
        if profile == PermissionProfile::Galaxy {
            crate::hitl_gate::gate_hitl_action(
                ctx.hitl.as_ref(),
                profile,
                &agent_key,
                "net.egress",
                "runtime TCP egress to public internet",
            )?;
        } else {
            return Err(PytxoError::Runner(format!(
                "runtime TCP egress denied for {} profile",
                profile.as_str()
            )));
        }
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
                &format!(
                    "deepspace-v2:{}",
                    crate::network_isolation::isolation_mechanism()
                ),
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
        let task_prompt = task_prompt.clone();
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
                task_prompt.as_deref(),
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
            let cmd = cmd.clone();
            let task_prompt = task_prompt.clone();
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
                    task_prompt.as_deref(),
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

    let mut exit_code = result.exit_code;
    let mut stderr = result.stderr;
    let mut verification_failed = false;
    if exit_code == Some(0) && !task.verify.is_empty() {
        let verify_ctx = ctx.clone();
        let verify_cwd = wt_path.clone();
        let verify_commands = task.verify.clone();
        let verify_agent_key = agent_key.clone();
        let verify_lifecycle = VerificationLifecycle {
            persist: ProcessPersist {
                run_id: ctx.run_id.0.clone(),
                repo_root: eff_repo_root.to_string_lossy().into_owned(),
                data_dir: ctx.data_dir.clone(),
                branch: branch.clone(),
            },
            swarm: swarm.clone(),
        };
        let verification = tokio::task::spawn_blocking(move || {
            run_verify_commands(
                &verify_cwd,
                &verify_commands,
                verify_ctx.on_event.as_ref(),
                &verify_agent_key,
                profile,
                &verify_ctx.domain_id,
                effective_backend,
                used_isolation,
                verify_ctx.hitl.as_ref(),
                Some(&verify_lifecycle),
            )
        })
        .await
        .map_err(|error| PytxoError::Runner(format!("verification join: {error}")))?;
        match verification {
            Ok(()) => {
                if let Some(cb) = ctx.on_event.as_ref() {
                    cb(
                        &agent_key,
                        "verify-ok",
                        &format!("{} check(s) passed", task.verify.len()),
                    );
                }
            }
            Err(err) => {
                if let Some(cb) = ctx.on_event.as_ref() {
                    cb(&agent_key, "verify-failed", &err.to_string());
                }
                stderr = format!("{stderr}\nverify failed: {err}");
                exit_code = Some(1);
                verification_failed = true;
            }
        }
    }

    // Verification is part of the actor lifecycle: retain the Race claim and any
    // execution sandbox until its bounded result has been recorded.
    swarm.release(&agent_key);
    if let Some(hub) = &ctx.mcp_hub {
        hub.deregister(&agent_key);
    }
    drop(sandbox_guard);

    if used_isolation && !ctx.keep_worktrees && exit_code == Some(0) {
        let _ = isolation.rollback(&iso_ctx, &workspace);
    }

    Ok(AgentRunResult {
        agent_id: agent_id.clone(),
        task_id: task.task_id.0.clone(),
        wave: task.wave,
        worktree_path: Some(result.worktree_path),
        exit_code,
        stdout: result.stdout,
        stderr,
        outcome: if verification_failed {
            AgentRunOutcome::VerificationFailed
        } else if exit_code == Some(0) {
            AgentRunOutcome::Succeeded
        } else {
            AgentRunOutcome::ProcessFailed
        },
        root_id: task.root.clone(),
    })
}

#[allow(clippy::too_many_arguments)]
fn run_verify_commands(
    cwd: &Path,
    commands: &[String],
    on_event: Option<&EventCallback>,
    agent_key: &str,
    profile: PermissionProfile,
    domain_id: &DomainId,
    execution_backend: ExecutionBackend,
    workspace_isolated: bool,
    hitl: Option<&crate::hitl::HitlQueue>,
    lifecycle: Option<&VerificationLifecycle>,
) -> Result<()> {
    run_verify_commands_with_limits(
        cwd,
        commands,
        on_event,
        agent_key,
        profile,
        domain_id,
        execution_backend,
        workspace_isolated,
        hitl,
        VERIFY_TIMEOUT,
        VERIFY_OUTPUT_LIMIT_BYTES,
        lifecycle,
    )
}

const VERIFY_TIMEOUT: Duration = Duration::from_secs(120);
const VERIFY_OUTPUT_LIMIT_BYTES: usize = 256 * 1024;

#[allow(clippy::too_many_arguments)]
fn run_verify_commands_with_limits(
    cwd: &Path,
    commands: &[String],
    on_event: Option<&EventCallback>,
    agent_key: &str,
    profile: PermissionProfile,
    domain_id: &DomainId,
    execution_backend: ExecutionBackend,
    workspace_isolated: bool,
    hitl: Option<&crate::hitl::HitlQueue>,
    timeout: Duration,
    output_limit_bytes: usize,
    lifecycle: Option<&VerificationLifecycle>,
) -> Result<()> {
    let receipt = crate::enforcement::verification_enforcement_receipt(
        profile,
        domain_id,
        execution_backend,
        workspace_isolated,
        timeout,
        output_limit_bytes,
    )?;
    if let Some(cb) = on_event {
        let evidence = serde_json::to_string(&receipt)
            .map_err(|e| PytxoError::Runner(format!("serialize verification receipt: {e}")))?;
        cb(agent_key, "verify-boundary", &evidence);
    }

    if execution_backend == ExecutionBackend::Cloud {
        return Err(PytxoError::Runner(
            "verification refused: cloud execution has no cancellable verifier contract".into(),
        ));
    }

    let engine = PermissionEngine::new(profile);
    let net = engine.network();
    for cmd in commands {
        if let Some(lifecycle) = lifecycle {
            lifecycle.ensure_not_cancelled(agent_key)?;
        }
        let cmd = cmd.trim();
        if cmd.is_empty() {
            continue;
        }
        if !net.spawn_egress_allowed(cmd) {
            return Err(PytxoError::Runner(format!(
                "verification network egress denied for {} profile",
                profile.as_str()
            )));
        }
        if command_implies_egress(cmd) && !net.egress_allowed("1.1.1.1", 443) {
            if profile == PermissionProfile::Galaxy {
                crate::hitl_gate::gate_hitl_action_cancellable(
                    hitl,
                    profile,
                    agent_key,
                    "verify.net.egress",
                    "verification TCP egress to public internet",
                    || match lifecycle {
                        Some(lifecycle) => lifecycle.ensure_not_cancelled(agent_key),
                        None => Ok(()),
                    },
                )?;
            } else {
                return Err(PytxoError::Runner(format!(
                    "verification runtime TCP egress denied for {} profile",
                    profile.as_str()
                )));
            }
        }
        if let Some((action, reason)) = crate::hitl_gate::classify_risky_command(cmd) {
            crate::hitl_gate::gate_hitl_action_cancellable(
                hitl,
                profile,
                agent_key,
                action,
                reason,
                || match lifecycle {
                    Some(lifecycle) => lifecycle.ensure_not_cancelled(agent_key),
                    None => Ok(()),
                },
            )?;
        }
        if let Some(lifecycle) = lifecycle {
            lifecycle.ensure_not_cancelled(agent_key)?;
        }
        if let Some(cb) = on_event {
            cb(agent_key, "verify", cmd);
        }
        let shell = shell_command();
        let mut command = std::process::Command::new(&shell.0);
        command
            .args(&shell.1)
            .arg(cmd)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut env = ChildLaunchEnv::new();
        env.set("PYTXO_VERIFICATION_ACTOR", "1");
        env.set("PYTXO_PERMISSION_PROFILE", profile.as_str());
        env.set("PYTXO_EXECUTION_DOMAIN", &domain_id.0);
        env.apply_command(&mut command);
        if profile == PermissionProfile::DeepSpace {
            crate::network_isolation::isolate_deepspace_network(&mut command)?;
        }
        configure_verifier_process_group(&mut command);

        let mut child = command
            .spawn()
            .map_err(|e| PytxoError::Runner(format!("verify spawn `{cmd}`: {e}")))?;
        if let Some(lifecycle) = lifecycle {
            if let Err(error) = lifecycle.persist_child(&mut child, agent_key, cwd) {
                terminate_verifier_tree(&mut child);
                let _ = child.wait();
                return Err(error);
            }
            lifecycle.swarm.register_pid(agent_key, child.id());
        }
        let stdout_handle = child
            .stdout
            .take()
            .map(|pipe| spawn_bounded_reader(pipe, output_limit_bytes));
        let stderr_handle = child
            .stderr
            .take()
            .map(|pipe| spawn_bounded_reader(pipe, output_limit_bytes));
        let started = Instant::now();
        let output_status = loop {
            if let Some(lifecycle) = lifecycle {
                if let Err(error) = lifecycle.ensure_not_cancelled(agent_key) {
                    terminate_verifier_tree(&mut child);
                    let _ = child.wait();
                    break Err(error);
                }
            }
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => {}
                Err(error) => {
                    terminate_verifier_tree(&mut child);
                    let _ = child.wait();
                    break Err(PytxoError::Runner(format!("verify wait `{cmd}`: {error}")));
                }
            }
            if started.elapsed() >= timeout {
                terminate_verifier_tree(&mut child);
                let _ = child.wait();
                break Err(PytxoError::Runner(format!(
                    "verify `{cmd}` timed out after {} ms",
                    timeout.as_millis()
                )));
            }
            thread::sleep(Duration::from_millis(10));
        };
        let output_deadline = started + timeout;
        let captured = (|| {
            let stdout = join_bounded_reader(stdout_handle, output_deadline, lifecycle, agent_key)?;
            let stderr = join_bounded_reader(stderr_handle, output_deadline, lifecycle, agent_key)?;
            Ok::<_, PytxoError>((stdout, stderr))
        })();
        if captured.is_err() {
            terminate_verifier_tree(&mut child);
        }
        if let Some(lifecycle) = lifecycle {
            remove_persisted_process(&lifecycle.persist, agent_key)?;
            lifecycle.ensure_not_cancelled(agent_key)?;
        }
        let output = output_status?;
        let (stdout, stderr) = captured?;
        if !output.success() {
            let code = output.code().unwrap_or(-1);
            return Err(PytxoError::Runner(format!(
                "verify `{cmd}` failed (exit {code}): {}{}",
                stderr.text,
                if stderr.truncated {
                    "\n[verification stderr truncated]"
                } else {
                    ""
                }
            )));
        }
        if let Some(cb) = on_event {
            if !stdout.text.is_empty() {
                cb(agent_key, "verify-stdout", &stdout.text);
            }
            if !stderr.text.is_empty() {
                cb(agent_key, "verify-stderr", &stderr.text);
            }
            if stdout.truncated || stderr.truncated {
                cb(
                    agent_key,
                    "verify-output-truncated",
                    &format!("max_bytes_per_stream={output_limit_bytes}"),
                );
            }
        }
    }
    Ok(())
}

struct BoundedOutput {
    text: String,
    truncated: bool,
}

fn spawn_bounded_reader(
    mut pipe: impl Read + Send + 'static,
    limit: usize,
) -> std::sync::mpsc::Receiver<Result<BoundedOutput>> {
    let (sender, receiver) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let mut captured = Vec::with_capacity(limit.min(8192));
        let mut buffer = [0_u8; 8192];
        let mut truncated = false;
        loop {
            let read = match pipe.read(&mut buffer) {
                Ok(0) => break,
                Err(error) => {
                    let _ = sender.send(Err(PytxoError::Io(error)));
                    return;
                }
                Ok(read) => read,
            };
            let remaining = limit.saturating_sub(captured.len());
            let keep = remaining.min(read);
            captured.extend_from_slice(&buffer[..keep]);
            truncated |= keep < read;
        }
        let _ = sender.send(Ok(BoundedOutput {
            text: String::from_utf8_lossy(&captured).into_owned(),
            truncated,
        }));
    });
    receiver
}

fn join_bounded_reader(
    receiver: Option<std::sync::mpsc::Receiver<Result<BoundedOutput>>>,
    deadline: Instant,
    lifecycle: Option<&VerificationLifecycle>,
    agent_key: &str,
) -> Result<BoundedOutput> {
    let Some(receiver) = receiver else {
        return Ok(BoundedOutput {
            text: String::new(),
            truncated: false,
        });
    };
    loop {
        if let Some(lifecycle) = lifecycle {
            lifecycle.ensure_not_cancelled(agent_key)?;
        }
        match receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(output) => return output,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err(PytxoError::Runner(
                    "verification output capture failed".into(),
                ));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) if Instant::now() >= deadline => {
                return Err(PytxoError::Runner(
                    "verification output capture timed out; a descendant may still hold its pipe"
                        .into(),
                ));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

fn configure_verifier_process_group(command: &mut std::process::Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
}

fn terminate_verifier_tree(child: &mut std::process::Child) {
    let pid = child.id();
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(unix)]
    {
        let group = format!("-{pid}");
        let _ = std::process::Command::new("kill")
            .args(["-TERM", "--", &group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        thread::sleep(Duration::from_millis(50));
        let _ = std::process::Command::new("kill")
            .args(["-KILL", "--", &group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

struct ProcessPersist {
    run_id: String,
    repo_root: String,
    data_dir: PathBuf,
    branch: String,
}

struct VerificationLifecycle {
    persist: ProcessPersist,
    swarm: SwarmRegistry,
}

impl VerificationLifecycle {
    fn persist_child(
        &self,
        child: &mut std::process::Child,
        agent_key: &str,
        cwd: &Path,
    ) -> Result<()> {
        let pid = child.id();
        let identity = crate::kill::process_start_identity(pid)?;
        // A trivial verifier may exit before registration. An observed exit is
        // sufficient evidence; never turn a fast successful check into a failure.
        if identity.is_none() {
            if child.try_wait().map_err(PytxoError::Io)?.is_some() {
                return self.ensure_not_cancelled(agent_key);
            }
            return Err(PytxoError::Runner(
                "live verifier has no process identity".into(),
            ));
        }
        ProcessRegistryFile::update(&registry_path(&self.persist.data_dir), |registry| {
            if registry.cancelled_runs.contains(&self.persist.run_id) {
                return Err(PytxoError::Runner("verification cancelled by Stop".into()));
            }
            registry.push(ProcessEntry {
                run_id: self.persist.run_id.clone(),
                repo_root: self.persist.repo_root.clone(),
                agent_key: agent_key.to_string(),
                pid,
                start_identity: identity,
                worktree_path: cwd.to_string_lossy().into_owned(),
                branch: self.persist.branch.clone(),
            });
            Ok(())
        })
    }

    fn ensure_not_cancelled(&self, agent_key: &str) -> Result<()> {
        let registry = ProcessRegistryFile::load(&registry_path(&self.persist.data_dir))?;
        if self.swarm.stop_requested(agent_key)
            || registry.cancelled_runs.contains(&self.persist.run_id)
        {
            return Err(PytxoError::Runner("verification cancelled by Stop".into()));
        }
        Ok(())
    }
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
    task_prompt: Option<&str>,
    cloud_dispatcher: Option<&dyn CloudDispatcher>,
    cloud_sandbox_id: Option<&str>,
    cloud_fallback_local: bool,
    persist: Option<ProcessPersist>,
) -> Result<SingleResult> {
    let mut env = build_child_env(context_dir, profile, managed_transport, route);
    if let Some(prompt) = task_prompt {
        env.set("PYTXO_TASK_PROMPT", prompt);
    }

    if execution_backend == ExecutionBackend::Cloud {
        if let (Some(dispatcher), Some(sid)) = (cloud_dispatcher, cloud_sandbox_id) {
            match dispatcher.exec(sid, &cloud_exec_request(cmd, env.vars().clone())) {
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
                Err(e) if cloud_fallback_allowed(cloud_fallback_local, &e) => {
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
        let persist_spawn = |pid| {
            if let Some(persist) = persist.as_ref() {
                persist_process(persist, agent_key, worktree, &persist.branch, Some(pid))?;
            }
            Ok(())
        };
        let result = crate::pty::run_pty_session_with_spawn(
            worktree,
            &effective_cmd,
            env,
            pty_rows,
            pty_cols,
            on_event,
            agent_key,
            swarm,
            Some(&persist_spawn),
        )?;
        if let Some(persist) = persist.as_ref() {
            remove_persisted_process(persist, agent_key)?;
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
    if let Some(persist) = persist.as_ref() {
        if let Err(error) =
            persist_process(persist, agent_key, worktree, &persist.branch, Some(pid))
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
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

    if let Some(persist) = persist.as_ref() {
        remove_persisted_process(persist, agent_key)?;
    }

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

fn cloud_exec_request(cmd: &str, env: HashMap<String, String>) -> ExecRequest {
    ExecRequest {
        cmd: cmd.to_string(),
        // The sandbox sync API roots repository content here. A client-local
        // absolute worktree path does not exist in the remote container.
        cwd: Some("/workspace".into()),
        env,
    }
}

fn cloud_fallback_allowed(enabled: bool, error: &PytxoError) -> bool {
    enabled && !error.is_cloud_policy_denial()
}

fn persist_process(
    p: &ProcessPersist,
    agent_key: &str,
    worktree: &Path,
    branch: &str,
    pid: Option<u32>,
) -> Result<()> {
    let pid = pid.ok_or_else(|| PytxoError::Runner("missing child pid".into()))?;
    let start_identity = crate::kill::process_start_identity(pid)?.ok_or_else(|| {
        PytxoError::Runner(format!(
            "child pid {pid} exited before identity was durable"
        ))
    })?;
    ProcessRegistryFile::update(&registry_path(&p.data_dir), |registry| {
        if registry.cancelled_runs.contains(&p.run_id) {
            return Err(PytxoError::Runner(format!(
                "run {} cancelled by Stop",
                p.run_id
            )));
        }
        registry.push(ProcessEntry {
            run_id: p.run_id.clone(),
            repo_root: p.repo_root.clone(),
            agent_key: agent_key.to_string(),
            pid,
            start_identity: Some(start_identity),
            worktree_path: worktree.to_string_lossy().to_string(),
            branch: branch.to_string(),
        });
        Ok(())
    })
}

fn remove_persisted_process(p: &ProcessPersist, agent_key: &str) -> Result<()> {
    ProcessRegistryFile::update(&registry_path(&p.data_dir), |registry| {
        registry.remove_agent(agent_key);
        Ok(())
    })
}

/// Resolve the shell command for one scheduled task (Hypervisor Shell templates).
pub fn resolve_cmd_for_task(ctx: &RunContext, task: &pytxo_core::ScheduledTask) -> Result<String> {
    if let Some(template) = &ctx.task_cmd_template {
        if template.contains("{prompt}") {
            return Err(PytxoError::Runner(
                "raw {prompt} shell interpolation is forbidden; use PYTXO_TASK_PROMPT".into(),
            ));
        }
        let paths = task.paths.join(",");
        return Ok(template
            .replace("{task_id}", &task.task_id.0)
            .replace("{agent}", &task.agent)
            .replace("{paths}", &paths)
            .replace("{wave}", &task.wave.to_string()));
    }
    Ok(ctx.cmd.clone())
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
    let path = registry_path(data_dir);
    ProcessRegistryFile::update(&path, |registry| {
        if kill
            && !registry
                .cancelled_runs
                .iter()
                .any(|cancelled| cancelled == run_id)
        {
            registry.cancelled_runs.push(run_id.to_string());
        }
        let entries: Vec<ProcessEntry> = registry
            .for_run(run_id)
            .iter()
            .map(|entry| (*entry).clone())
            .collect();
        if kill {
            for entry in &entries {
                if let Err(error) = stop_registry_entry(entry) {
                    // Persist cancellation even when termination cannot be confirmed.
                    // Keep every process entry so a later Stop can reconcile it.
                    return Ok(Err(error));
                }
            }
        }
        registry.remove_run(run_id);
        Ok(Ok(entries.into_iter().map(|entry| entry.pid).collect()))
    })?
}

pub fn stop_all(data_dir: &Path, kill: bool) -> Result<()> {
    let path = registry_path(data_dir);
    ProcessRegistryFile::update(&path, |registry| {
        if kill {
            for entry in &registry.entries {
                if !registry.cancelled_runs.contains(&entry.run_id) {
                    registry.cancelled_runs.push(entry.run_id.clone());
                }
            }
        }
        let entries = registry.entries.clone();
        if kill {
            for entry in &entries {
                if let Err(error) = stop_registry_entry(entry) {
                    return Ok(Err(error));
                }
            }
        }
        registry.clear();
        Ok(Ok(()))
    })?
}

fn stop_registry_entry(entry: &ProcessEntry) -> Result<()> {
    match (
        entry.start_identity.as_deref(),
        crate::kill::process_start_identity(entry.pid)?,
    ) {
        (_, None) => Ok(()),
        (Some(expected), Some(actual)) if expected == actual => {
            crate::kill::kill_process_tree(entry.pid, expected)
        }
        (Some(_), Some(_)) => Err(PytxoError::Runner(format!(
            "refusing to stop pid {}: the PID now belongs to a different process",
            entry.pid
        ))),
        (None, Some(_)) => Err(PytxoError::Runner(format!(
            "refusing to stop pid {}: legacy registry entry has no process start identity",
            entry.pid
        ))),
    }
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
        if ctx.hitl_manual_flush {
            // Deck/CLI Approve merge is the human act; flush without queue.
        } else if let Some(hitl) = ctx.hitl.as_ref() {
            let agent_key = if workspace.branch.is_empty() {
                ctx.run_id.0.clone()
            } else {
                workspace.branch.clone()
            };
            if profile == PermissionProfile::Galaxy
                && crate::hitl_gate::workspace_writes_outside_root(&workspace.cwd, &ctx.repo_root)
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
        } else {
            return Err(PytxoError::Runner(
                "flush requires HITL queue (set hitl_manual_flush for Deck/CLI approve path)"
                    .into(),
            ));
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

#[cfg(test)]
mod dependency_output_tests {
    use super::*;

    fn write(path: &Path, contents: &str) {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        std::fs::write(path, contents).expect("write fixture");
    }

    #[test]
    fn composes_changed_and_deleted_claimed_files_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let base = temp.path().join("base");
        let upstream = temp.path().join("upstream");
        let downstream = temp.path().join("downstream");

        for root in [&base, &upstream, &downstream] {
            write(&root.join("src/value.txt"), "base");
            write(&root.join("src/removed.txt"), "remove me");
            write(&root.join("README.md"), "unchanged");
        }
        write(&upstream.join("src/value.txt"), "upstream");
        std::fs::remove_file(upstream.join("src/removed.txt")).expect("remove upstream fixture");
        write(&upstream.join(".pytxo/private.txt"), "never compose");

        let count = compose_dependency_outputs(
            &base,
            &downstream,
            &[DependencyOutput {
                task_id: "implementation".into(),
                workspace_path: upstream,
                paths: vec!["src".into()],
                root_id: None,
            }],
            &[],
        )
        .expect("compose dependency outputs");

        assert_eq!(count, 2);
        assert_eq!(
            std::fs::read_to_string(downstream.join("src/value.txt")).unwrap(),
            "upstream"
        );
        assert!(!downstream.join("src/removed.txt").exists());
        assert_eq!(
            std::fs::read_to_string(downstream.join("README.md")).unwrap(),
            "unchanged"
        );
        assert!(!downstream.join(".pytxo/private.txt").exists());
    }

    #[test]
    fn rejects_parent_directory_claims() {
        let temp = tempfile::tempdir().expect("tempdir");
        let output = DependencyOutput {
            task_id: "unsafe".into(),
            workspace_path: temp.path().join("upstream"),
            paths: vec!["../outside.txt".into()],
            root_id: None,
        };

        let error =
            compose_dependency_outputs(temp.path(), temp.path(), &[output], &[]).unwrap_err();
        assert!(error.to_string().contains("unsafe dependency claim"));
    }

    #[test]
    fn cloud_exec_is_rooted_in_the_remote_workspace() {
        let request = cloud_exec_request("pwd", HashMap::new());
        assert_eq!(request.cwd.as_deref(), Some("/workspace"));
    }

    #[test]
    fn cloud_policy_denials_never_fall_back_to_local_execution() {
        let policy = PytxoError::CloudPolicy("secret detected".into());
        let transport = PytxoError::Other("connection refused".into());
        assert!(!cloud_fallback_allowed(true, &policy));
        assert!(cloud_fallback_allowed(true, &transport));
        assert!(!cloud_fallback_allowed(false, &transport));
    }
}

#[cfg(test)]
mod verification_boundary_tests {
    use std::sync::Mutex;

    use super::*;

    fn verify_command_for_absent_secret() -> String {
        if cfg!(windows) {
            "if defined PYTXO_VERIFY_SENTINEL_SECRET (exit /b 9) else (exit /b 0)".into()
        } else {
            "test -z \"$PYTXO_VERIFY_SENTINEL_SECRET\"".into()
        }
    }

    fn blocking_command() -> String {
        if cfg!(windows) {
            "ping -n 6 127.0.0.1 >NUL".into()
        } else {
            "sleep 5".into()
        }
    }

    fn lifecycle(data_dir: &Path) -> VerificationLifecycle {
        VerificationLifecycle {
            persist: ProcessPersist {
                run_id: "run".into(),
                repo_root: data_dir.to_string_lossy().into_owned(),
                data_dir: data_dir.to_path_buf(),
                branch: String::new(),
            },
            swarm: SwarmRegistry::new(),
        }
    }

    #[test]
    fn output_capture_deadline_includes_a_pipe_that_never_closes() {
        struct DelayedPipe;
        impl Read for DelayedPipe {
            fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
                thread::sleep(Duration::from_millis(500));
                Ok(0)
            }
        }
        let started = Instant::now();
        let reader = spawn_bounded_reader(DelayedPipe, 1024);
        let result = join_bounded_reader(
            Some(reader),
            started + Duration::from_millis(50),
            None,
            "run:agent-0",
        );
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("output capture timed out"));
        assert!(started.elapsed() < Duration::from_millis(400));
    }

    #[test]
    fn failed_stop_preserves_cancellation_and_process_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let path = registry_path(temp.path());
        ProcessRegistryFile::update(&path, |registry| {
            registry.push(ProcessEntry {
                run_id: "run".into(),
                repo_root: String::new(),
                agent_key: "run:agent-0".into(),
                pid: std::process::id(),
                start_identity: Some("intentionally-stale-identity".into()),
                worktree_path: String::new(),
                branch: String::new(),
            });
            Ok(())
        })
        .unwrap();
        stop_run(temp.path(), "run", true).expect_err("stale identity must refuse termination");
        let registry = ProcessRegistryFile::load(&path).unwrap();
        assert_eq!(registry.cancelled_runs, vec!["run"]);
        assert_eq!(
            registry.entries.len(),
            1,
            "retain failed termination evidence"
        );
        ProcessRegistryFile::update(&path, |registry| {
            registry.cancelled_runs.clear();
            Ok(())
        })
        .unwrap();
        stop_all(temp.path(), true).expect_err("Stop all must also refuse stale identities");
        let registry = ProcessRegistryFile::load(&path).unwrap();
        assert_eq!(registry.cancelled_runs, vec!["run"]);
        assert_eq!(registry.entries.len(), 1);
    }

    #[test]
    fn candidate_adapter_returns_boundary_and_obeys_durable_stop() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = CandidateCheckContext {
            cwd: temp.path().to_path_buf(),
            run_id: "candidate-run".into(),
            agent_key: "candidate-run:verify".into(),
            repo_root: temp.path().to_path_buf(),
            data_dir: temp.path().to_path_buf(),
            profile: PermissionProfile::Orbit,
            domain_id: DomainId("candidate-domain".into()),
            execution_backend: ExecutionBackend::Subprocess,
            workspace_isolated: true,
            hitl: None,
            swarm: SwarmRegistry::new(),
            on_event: None,
        };
        let observed = run_candidate_check(&ctx, "echo candidate verified").unwrap();
        let expected = crate::verification_enforcement_receipt(
            ctx.profile,
            &ctx.domain_id,
            ctx.execution_backend,
            true,
            VERIFY_TIMEOUT,
            VERIFY_OUTPUT_LIMIT_BYTES,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(observed).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        stop_run(temp.path(), &ctx.run_id, true).unwrap();
        assert!(run_candidate_check(&ctx, "echo escaped > after-stop.txt")
            .unwrap_err()
            .to_string()
            .contains("cancelled by Stop"));
        assert!(!temp.path().join("after-stop.txt").exists());
    }

    #[test]
    fn durable_stop_cancels_verification_waiting_for_galaxy_approval() {
        let temp = tempfile::tempdir().unwrap();
        let data_dir = temp.path().to_path_buf();
        let worker_dir = data_dir.clone();
        let hitl = crate::HitlQueue::new();
        let worker_hitl = hitl.clone();
        let worker = thread::spawn(move || {
            run_verify_commands_with_limits(
                &worker_dir,
                &["echo chmod > after-stop.txt".into()],
                None,
                "run:agent-0",
                PermissionProfile::Galaxy,
                &DomainId("approval-domain".into()),
                ExecutionBackend::Subprocess,
                true,
                Some(&worker_hitl),
                Duration::from_secs(2),
                4096,
                Some(&lifecycle(&worker_dir)),
            )
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        while hitl.pending().is_empty() {
            assert!(Instant::now() < deadline, "approval was never requested");
            thread::sleep(Duration::from_millis(10));
        }
        stop_run(&data_dir, "run", true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !worker.is_finished() {
            assert!(
                Instant::now() < deadline,
                "Stop did not interrupt approval wait"
            );
            thread::sleep(Duration::from_millis(10));
        }
        assert!(worker
            .join()
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("cancelled by Stop"));
        assert!(hitl.pending().is_empty());
        assert!(!data_dir.join("after-stop.txt").exists());
    }

    #[test]
    fn successful_fast_verification_retains_its_observed_exit() {
        let temp = tempfile::tempdir().unwrap();
        let lifecycle = lifecycle(temp.path());
        for _ in 0..20 {
            run_verify_commands_with_limits(
                temp.path(),
                &["echo verified".into()],
                None,
                "run:agent-0",
                PermissionProfile::Orbit,
                &DomainId("fast-domain".into()),
                ExecutionBackend::Subprocess,
                true,
                None,
                Duration::from_secs(2),
                4096,
                Some(&lifecycle),
            )
            .expect("short lived verifiers remain successful");
        }
        assert!(ProcessRegistryFile::load(&registry_path(temp.path()))
            .unwrap()
            .entries
            .is_empty());
    }

    #[test]
    fn durable_stop_terminates_verification_and_prevents_later_commands() {
        let temp = tempfile::tempdir().expect("tempdir");
        let data_dir = temp.path().to_path_buf();
        let worker_dir = data_dir.clone();
        let worker = thread::spawn(move || {
            let lifecycle = lifecycle(&worker_dir);
            run_verify_commands_with_limits(
                &worker_dir,
                &[
                    blocking_command(),
                    "echo must-not-run > after-stop.txt".into(),
                ],
                None,
                "run:agent-0",
                PermissionProfile::Orbit,
                &DomainId("stop-domain".into()),
                ExecutionBackend::Subprocess,
                true,
                None,
                Duration::from_secs(10),
                4096,
                Some(&lifecycle),
            )
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        let entry = loop {
            let registry = ProcessRegistryFile::load(&registry_path(&data_dir)).unwrap();
            if let Some(entry) = registry.entries.first() {
                break entry.clone();
            }
            assert!(
                Instant::now() < deadline,
                "verifier PID was never persisted"
            );
            thread::sleep(Duration::from_millis(10));
        };
        let identity = entry.start_identity.as_deref().expect("durable identity");
        assert!(crate::kill::process_matches(entry.pid, identity).unwrap());
        stop_run(&data_dir, "run", true).expect("stop verifier using durable registry");
        let error = worker
            .join()
            .unwrap()
            .expect_err("stopped verifier cannot succeed");
        assert!(error.to_string().contains("cancelled by Stop"));
        assert!(!crate::kill::process_matches(entry.pid, identity).unwrap());
        assert!(!data_dir.join("after-stop.txt").exists());
        assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
            .unwrap()
            .entries
            .is_empty());
    }

    #[test]
    fn durable_stop_before_verification_prevents_spawn() {
        let temp = tempfile::tempdir().expect("tempdir");
        stop_run(temp.path(), "run", true).unwrap();
        let lifecycle = lifecycle(temp.path());
        let error = run_verify_commands_with_limits(
            temp.path(),
            &["echo must-not-run > after-stop.txt".into()],
            None,
            "run:agent-0",
            PermissionProfile::Orbit,
            &DomainId("stop-domain".into()),
            ExecutionBackend::Subprocess,
            true,
            None,
            Duration::from_secs(2),
            4096,
            Some(&lifecycle),
        )
        .expect_err("prior durable Stop prevents verifier spawn");
        assert!(error.to_string().contains("cancelled by Stop"));
        assert!(!temp.path().join("after-stop.txt").exists());
    }

    #[test]
    fn verification_times_out_and_terminates_the_child() {
        let temp = tempfile::tempdir().expect("tempdir");
        let started = Instant::now();
        let error = run_verify_commands_with_limits(
            temp.path(),
            &[blocking_command()],
            None,
            "run:agent-0",
            PermissionProfile::Orbit,
            &DomainId("timeout-domain".into()),
            ExecutionBackend::Subprocess,
            true,
            None,
            Duration::from_millis(100),
            4096,
            None,
        )
        .expect_err("blocking verifier must time out");

        assert!(error.to_string().contains("timed out after 100 ms"));
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "timeout did not bound verifier execution"
        );
    }

    #[test]
    fn verification_strips_parent_secrets_and_emits_boundary_receipt() {
        const SENTINEL: &str = "PYTXO_VERIFY_SENTINEL_SECRET";
        std::env::set_var(SENTINEL, "must-not-leak");
        let events = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
        let event_sink = Arc::clone(&events);
        let callback: EventCallback = Arc::new(move |_agent, kind, body| {
            event_sink
                .lock()
                .expect("event lock")
                .push((kind.into(), body.into()));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let result = run_verify_commands_with_limits(
            temp.path(),
            &[verify_command_for_absent_secret()],
            Some(&callback),
            "run:agent-0",
            PermissionProfile::Orbit,
            &DomainId("boundary-domain".into()),
            ExecutionBackend::Subprocess,
            true,
            None,
            Duration::from_secs(2),
            4096,
            None,
        );
        std::env::remove_var(SENTINEL);
        result.expect("filtered verifier should not observe parent secret");

        let events = events.lock().expect("event lock");
        let receipt_json = events
            .iter()
            .find_map(|(kind, body)| (kind == "verify-boundary").then_some(body))
            .expect("verification boundary receipt event");
        let receipt: crate::enforcement::VerificationEnforcementReceipt =
            serde_json::from_str(receipt_json).expect("parse receipt");
        assert_eq!(receipt.actor, "verification");
        assert_eq!(receipt.effective_profile, "orbit");
        assert_eq!(receipt.execution_domain, "boundary-domain");
        assert_eq!(receipt.environment.status, "enforced");
        assert_eq!(receipt.network.status, "advisory");
        assert_eq!(receipt.timeout.status, "enforced");
        assert_eq!(receipt.output_capture.status, "enforced");
    }

    #[test]
    fn cloud_verification_fails_closed_after_recording_boundary() {
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let event_sink = Arc::clone(&events);
        let callback: EventCallback = Arc::new(move |_agent, kind, body| {
            if kind == "verify-boundary" {
                event_sink.lock().expect("event lock").push(body.into());
            }
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let marker = temp.path().join("must-not-exist");
        let command = if cfg!(windows) {
            format!("echo unsafe>{}", marker.display())
        } else {
            format!("touch {}", marker.display())
        };
        let error = run_verify_commands_with_limits(
            temp.path(),
            &[command],
            Some(&callback),
            "run:agent-0",
            PermissionProfile::Orbit,
            &DomainId("cloud-domain".into()),
            ExecutionBackend::Cloud,
            true,
            None,
            Duration::from_secs(2),
            4096,
            None,
        )
        .expect_err("cloud verifier must fail closed without a cancellable remote contract");

        assert!(error
            .to_string()
            .contains("no cancellable verifier contract"));
        assert!(!marker.exists(), "verifier escaped to the host");
        let receipt: crate::enforcement::VerificationEnforcementReceipt = serde_json::from_str(
            events
                .lock()
                .expect("event lock")
                .first()
                .expect("boundary receipt"),
        )
        .expect("parse receipt");
        assert_eq!(receipt.execution_backend, "remote-unsupported-fail-closed");
    }

    #[test]
    fn orbit_verification_rejects_known_egress_before_spawn() {
        let temp = tempfile::tempdir().expect("tempdir");
        let error = run_verify_commands_with_limits(
            temp.path(),
            &["curl https://example.com".into()],
            None,
            "run:agent-0",
            PermissionProfile::Orbit,
            &DomainId("network-domain".into()),
            ExecutionBackend::Subprocess,
            true,
            None,
            Duration::from_secs(2),
            4096,
            None,
        )
        .expect_err("Orbit verifier egress must be policy-gated");

        assert!(error
            .to_string()
            .contains("verification network egress denied for orbit profile"));
    }
}
