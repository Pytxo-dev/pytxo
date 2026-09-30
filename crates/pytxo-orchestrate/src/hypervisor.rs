use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::Context;
use pytxo_core::{DomainId, PermissionProfile, ProjectManifest, PytxoConfig, RunId};
use pytxo_runner::{
    permission_enforcement_receipt, HitlQueue, McpHub, ProcessRegistry, SwarmRegistry,
};
use tracing::error;

use crate::flow::{load_reviewed_staged_mission, runtime_tasks, FlowPlan};
use crate::{
    ensure_repo_trusted, execute_run_body, load_config, plan_tasks, reserve_active_run,
    resolve_repo_root, resolve_run_tasks, settle_reserved_run_after_error,
    settle_reserved_run_after_stop, RunEnforcementEnvelope, RunOptions,
};

pub(crate) fn require_routed_worktree_isolation(cfg: &PytxoConfig) -> anyhow::Result<()> {
    if pytxo_runner::effective_isolation_mode(cfg) != pytxo_core::IsolationMode::Worktree {
        anyhow::bail!("routed local fixture requires actual worktree isolation");
    }
    Ok(())
}

pub(crate) fn routed_review_authority(
    plan: &FlowPlan,
    domain: &DomainState,
    cfg: &PytxoConfig,
) -> anyhow::Result<(pytxo_core::ExecutionPlan, RunEnforcementEnvelope)> {
    let tasks = runtime_tasks(plan);
    let execution = plan_tasks(&tasks, cfg)?;
    let actual_waves: Vec<Vec<String>> = execution
        .waves
        .iter()
        .map(|wave| wave.iter().map(|task| task.task_id.0.clone()).collect())
        .collect();
    if actual_waves != plan.waves
        || cfg.permission_profile != PermissionProfile::Orbit
        || tasks
            .iter()
            .any(|task| cfg.resolve_profile_for_agent(&task.agent) != PermissionProfile::Orbit)
    {
        anyhow::bail!("routed review contract differs from reviewed Orbit execution plan");
    }
    require_routed_worktree_isolation(cfg)?;
    let receipt = permission_enforcement_receipt(
        PermissionProfile::Orbit,
        PermissionProfile::Orbit,
        &domain.id,
        pytxo_runner::effective_isolation_mode(cfg),
        &cfg.blast.sparse_exclude,
    )?;
    let agents = execution
        .waves
        .iter()
        .flatten()
        .enumerate()
        .map(|(index, _)| (format!("agent-{index}"), receipt.clone()))
        .collect::<BTreeMap<_, _>>();
    let enforcement = RunEnforcementEnvelope {
        run: receipt,
        agents,
    };
    Ok((execution, enforcement))
}

fn bind_routed_review_contract(
    store: &pytxo_store::PytxoStore,
    plan: &FlowPlan,
    mission: &pytxo_store::routing::RoutingMission,
    domain: &DomainState,
    cfg: &PytxoConfig,
) -> anyhow::Result<()> {
    let (execution, enforcement) = routed_review_authority(plan, domain, cfg)?;
    let base = mission
        .tasks
        .first()
        .context("routed review contract has no task base")?
        .contract
        .base
        .git_revision
        .as_str();
    store.insert_run_contract_once(
        &mission.authorization.run_id.0,
        base,
        &serde_json::to_string(&execution)?,
        &serde_json::to_string(&enforcement)?,
    )?;
    Ok(())
}

/// Per-repo execution slice: isolated swarm registry and in-process PID tracking.
pub struct DomainState {
    pub id: DomainId,
    pub repo_root: PathBuf,
    pub data_dir: PathBuf,
    pub swarm: SwarmRegistry,
    pub process_registry: ProcessRegistry,
    /// Galaxy HITL approval queue scoped to this domain ([[race-shield]]).
    pub hitl: HitlQueue,
    /// MCP hub v2 child session registry ([[mcp-router]]).
    pub mcp_hub: McpHub,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct DomainSummary {
    pub domain_id: String,
    pub repo_root: String,
}

#[derive(Debug)]
pub(crate) struct RoutedStartupSettled(pub String);

impl std::fmt::Display for RoutedStartupSettled {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "routed execution unavailable for reviewed run {}",
            self.0
        )
    }
}

impl std::error::Error for RoutedStartupSettled {}

#[derive(Debug)]
pub(crate) struct RoutedStartupStopped(pub String);

impl std::fmt::Display for RoutedStartupStopped {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "reviewed routed run {} was stopped before registration",
            self.0
        )
    }
}

impl std::error::Error for RoutedStartupStopped {}

#[cfg(feature = "routed-test-faults")]
pub(crate) fn pause_routed_fault_stage(name: &str) -> anyhow::Result<()> {
    let Some(root) = std::env::var_os(name).map(std::path::PathBuf::from) else {
        return Ok(());
    };
    if !root.is_absolute() || !root.is_dir() {
        anyhow::bail!("routed test pause root is invalid");
    }
    std::fs::write(root.join("entered"), b"")?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !root.join("release").exists() {
        if std::time::Instant::now() >= deadline {
            anyhow::bail!("routed test pause timed out");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}

pub struct HypervisorRegistry {
    domains: Mutex<HashMap<DomainId, Arc<DomainState>>>,
}

impl Default for HypervisorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HypervisorRegistry {
    pub fn new() -> Self {
        Self {
            domains: Mutex::new(HashMap::new()),
        }
    }

    /// Ensure the primary domain for a modular project and register all roots
    /// in the catalog ([[ADR-0011-modular-project-manifest]]).
    pub fn ensure_project(
        &self,
        manifest: &ProjectManifest,
        cfg: &PytxoConfig,
    ) -> anyhow::Result<Arc<DomainState>> {
        let primary = manifest
            .primary_repo_root()
            .map_err(|e| anyhow::anyhow!(e))?;
        let domain = self.ensure_domain(&primary, cfg)?;
        if let Err(e) = pytxo_store::Catalog::open_default().and_then(|cat| {
            cat.upsert_domain(
                domain.id.as_str(),
                &primary.to_string_lossy(),
                &cfg.db_path_at(&primary).to_string_lossy(),
                Some(&manifest.project.id),
            )?;
            for root in &manifest.roots {
                let label = root.effective_label();
                cat.upsert_project_root(
                    &manifest.project.id,
                    &label,
                    root.path.to_string_lossy().as_ref(),
                    root.read_only,
                    root.primary,
                    root.permission_profile.map(|p| p.as_str()),
                )?;
            }
            Ok(())
        }) {
            tracing::debug!("project catalog upsert skipped: {e}");
        }
        Ok(domain)
    }

    pub fn ensure_domain(
        &self,
        repo_root: &Path,
        cfg: &PytxoConfig,
    ) -> anyhow::Result<Arc<DomainState>> {
        // Keep the cached path spelling aligned with resolve_repo_root. On
        // Windows, canonicalize alone yields a verbatim \\?\ path; a later
        // routed dispatch strips that prefix before checking active ownership.
        let repo_root = pytxo_core::canonical_repo_root(repo_root)?;
        let id = DomainId::from_repo_root(&repo_root).map_err(|e| anyhow::anyhow!(e))?;
        let mut guard = self
            .domains
            .lock()
            .map_err(|_| anyhow::anyhow!("hypervisor lock poisoned"))?;
        if let Some(existing) = guard.get(&id) {
            return Ok(Arc::clone(existing));
        }
        fs::create_dir_all(repo_root.join(&cfg.worktree_dir))?;
        fs::create_dir_all(repo_root.join(&cfg.data_dir))?;
        let data_dir = repo_root.join(&cfg.data_dir);
        let db_path = cfg.db_path_at(&repo_root);
        let hitl = {
            let audit_db = db_path.clone();
            HitlQueue::with_persistence(&data_dir).with_wal_audit(Arc::new(
                move |request, approved| {
                    let store = pytxo_store::PytxoStore::open(&audit_db)?;
                    let (run_id, agent_id) = request
                        .agent_key
                        .split_once(':')
                        .map(|(run, agent)| (Some(run), Some(agent)))
                        .unwrap_or((None, None));
                    let payload = serde_json::json!({
                        "request_id": request.id,
                        "agent_key": request.agent_key,
                        "run_id": run_id,
                        "agent_id": agent_id,
                        "action": request.action,
                        "decision": if approved { "approved" } else { "denied" },
                    })
                    .to_string();
                    store.append_approval_event(
                        &request.agent_key,
                        &request.id,
                        "hitl-resolve",
                        &payload,
                    )?;
                    Ok(())
                },
            ))
        };
        let state = Arc::new(DomainState {
            id: id.clone(),
            repo_root: repo_root.to_path_buf(),
            data_dir,
            swarm: SwarmRegistry::new(),
            process_registry: ProcessRegistry::default(),
            hitl,
            mcp_hub: McpHub::new(),
        });
        register_in_catalog(&state, cfg);
        guard.insert(id, Arc::clone(&state));
        Ok(state)
    }

    pub fn list_domains(&self) -> Vec<DomainSummary> {
        let guard = self.domains.lock().expect("hypervisor lock");
        guard
            .values()
            .map(|d| DomainSummary {
                domain_id: d.id.as_str().to_string(),
                repo_root: d.repo_root.to_string_lossy().into_owned(),
            })
            .collect()
    }

    /// HITL pending count when this domain is loaded in-process.
    pub fn domain_hitl_pending(&self, domain_id: &str) -> Option<usize> {
        let guard = self.domains.lock().ok()?;
        let state = guard.values().find(|d| d.id.as_str() == domain_id)?;
        Some(state.hitl.pending().len())
    }

    pub fn domain_state(&self, domain_id: &str) -> Option<Arc<DomainState>> {
        let guard = self.domains.lock().ok()?;
        guard
            .values()
            .find(|d| d.id.as_str() == domain_id)
            .map(Arc::clone)
    }

    pub async fn run_blocking(&self, opts: RunOptions) -> anyhow::Result<RunId> {
        let _upgrade_guard = pytxo_core::UpgradeGuard::work()?;
        let repo_root = resolve_repo_root(opts.repo.as_deref())?;
        let mut cfg = load_config(opts.config.as_deref(), &repo_root)?;
        if opts.agents > 0 {
            cfg.max_agents = opts.agents;
        }
        let domain = self.ensure_domain(&repo_root, &cfg)?;
        execute_run_body(domain, opts, cfg, None).await
    }

    pub fn dispatch(&self, opts: RunOptions) -> anyhow::Result<(DomainId, RunId)> {
        let repo_root = resolve_repo_root(opts.repo.as_deref())?;
        let cfg = load_config(opts.config.as_deref(), &repo_root)?;
        self.dispatch_with_config_snapshot(opts, cfg)
    }

    /// Dispatch using a configuration that was already validated by an orchestration facade.
    ///
    /// Flow uses this path so policy cannot change between preview revalidation and execution.
    pub(crate) fn dispatch_with_config_snapshot(
        &self,
        opts: RunOptions,
        mut cfg: PytxoConfig,
    ) -> anyhow::Result<(DomainId, RunId)> {
        let upgrade_guard = pytxo_core::UpgradeGuard::work()?;
        let repo_root = resolve_repo_root(opts.repo.as_deref())?;
        if opts.agents > 0 {
            cfg.max_agents = opts.agents;
        }
        let domain = self.ensure_domain(&repo_root, &cfg)?;
        let domain_id = domain.id.clone();
        let run_id = RunId::new();
        let run_id_for_task = run_id.clone();

        if opts.dry_run {
            let _ = domain;
            let tasks = resolve_run_tasks(&cfg, opts.agents, opts.tasks.clone());
            let _plan = plan_tasks(&tasks, &cfg)?;
            return Ok((domain_id, run_id));
        }

        ensure_repo_trusted(&repo_root)?;

        let runtime = tokio::runtime::Handle::try_current()
            .context("dispatch requires an active Tokio runtime")?;
        reserve_active_run(&domain, &cfg, &run_id)?;
        let error_domain = Arc::clone(&domain);
        let error_cfg = cfg.clone();
        runtime.spawn(async move {
            let _upgrade_guard = upgrade_guard;
            let worker = tokio::spawn(execute_run_body(
                domain,
                opts,
                cfg,
                Some(run_id_for_task.clone()),
            ));
            match worker.await {
                Ok(Ok(_)) => {}
                Ok(Err(error)) => {
                    settle_reserved_run_after_error(&error_domain, &error_cfg, &run_id_for_task);
                    error!("dispatch run {} failed: {error:#}", run_id_for_task.0);
                }
                Err(error) => {
                    settle_reserved_run_after_error(&error_domain, &error_cfg, &run_id_for_task);
                    error!("dispatch run {} task aborted: {error}", run_id_for_task.0);
                }
            }
        });

        Ok((domain_id, run_id))
    }

    /// Experimental reviewed identity path. A separately opted-in local fixture
    /// runs through the owned routed supervisor; all other routes retain the
    /// existing no-worker fail-closed behavior.
    #[expect(
        clippy::too_many_arguments,
        reason = "keep the exact Flow claim, config snapshot, and injected hosted client explicit"
    )]
    pub(crate) fn dispatch_routed_with_config_snapshot(
        &self,
        catalog: &pytxo_store::Catalog,
        draft_id: &str,
        expected_plan_json: &str,
        plan: &FlowPlan,
        repo_root: &Path,
        cfg: PytxoConfig,
        hosted_client: Option<Arc<dyn crate::routed_hosted_shadow::HostedShadowClient>>,
    ) -> anyhow::Result<(DomainId, RunId)> {
        let _upgrade_guard = pytxo_core::UpgradeGuard::work()?;
        let repo_root = resolve_repo_root(Some(repo_root))?;
        let domain = self.ensure_domain(&repo_root, &cfg)?;
        if crate::routed_fixture::fixture_opted_in()
            || crate::routed_fixture::claude_proposal_opted_in()
        {
            routed_review_authority(plan, &domain, &cfg)?;
        }
        let review = plan
            .routing
            .as_ref()
            .context("experimental routed dispatch requires a routed review")?;
        if review.authorization.domain_id != domain.id {
            anyhow::bail!("reviewed routed execution domain changed");
        }
        let run_id = review.authorization.run_id.clone();
        ensure_repo_trusted(&repo_root)?;
        // Qualification probes are bounded pre-run observations in a private
        // OS temp directory. A routed Run is not claimed until they finish, so
        // no unregistered probe child can outlive a Stop-cleared run marker.
        let preflight_mission = if crate::routed_fixture::fixture_opted_in()
            || crate::routed_fixture::claude_proposal_opted_in()
        {
            let store = pytxo_store::PytxoStore::open(&cfg.db_path_at(&repo_root))?;
            Some(load_reviewed_staged_mission(&store, plan)?)
        } else {
            None
        };
        let claude_route = crate::routed_fixture::claude_proposal_opted_in()
            && preflight_mission.as_ref().is_some_and(|mission| {
                mission.profiles.len() == 2
                    && mission
                        .profiles
                        .iter()
                        .all(|profile| profile.profile.harness_id == "claude")
            });
        let fixture_route = crate::routed_fixture::fixture_opted_in() && !claude_route;
        #[cfg(feature = "routed-test-faults")]
        pause_routed_fault_stage("PYTXO_TEST_ROUTED_PAUSE_BEFORE_PROBES")?;
        let prequalified = if fixture_route || claude_route {
            let claimed = catalog
                .get_flow_draft(draft_id)?
                .context("reviewed Flow draft disappeared before local qualification")?;
            if claimed.status != "dispatching"
                || claimed.plan_json.as_deref() != Some(expected_plan_json)
                || claimed.dispatched_run_id.as_deref() != Some(run_id.0.as_str())
            {
                anyhow::bail!("reviewed Flow claim changed before local qualification");
            }
            let preflight = preflight_mission
                .as_ref()
                .context("reviewed routed preflight disappeared")?;
            if claude_route {
                crate::flow::validate_one_task_claude_route_shape(plan, preflight)?;
            }
            #[cfg(windows)]
            let claude_context = if claude_route {
                Some(crate::routed_fixture::observe_claude_proposal_probe_context()?)
            } else {
                None
            };
            preflight
                .tasks
                .iter()
                .map(|task| {
                    preflight
                        .profiles
                        .iter()
                        .map(|profile| {
                            if claude_route {
                                #[cfg(windows)]
                                {
                                    let target = if profile.profile.id
                                        == preflight.policy.everyday.profile_id
                                    {
                                        &preflight.policy.everyday
                                    } else {
                                        &preflight.policy.strong
                                    };
                                    crate::routed_fixture::qualify_claude_proposal(
                                        profile,
                                        target,
                                        catalog,
                                        claude_context
                                            .as_ref()
                                            .context("Claude pins are absent")?,
                                        draft_id,
                                        &run_id.0,
                                    )
                                }
                                #[cfg(not(windows))]
                                anyhow::bail!("Claude proposal route needs Windows owned Job")
                            } else {
                                let arguments = crate::routed_fixture::fixture_worker_arguments(
                                    &task.contract.goal,
                                    &profile.profile.id.0,
                                )?;
                                crate::routed_fixture::qualify_local_fixture(
                                    profile,
                                    catalog,
                                    &std::env::temp_dir(),
                                    &arguments,
                                    draft_id,
                                    &run_id.0,
                                )
                            }
                        })
                        .collect::<anyhow::Result<Vec<_>>>()
                })
                .collect::<anyhow::Result<Vec<_>>>()?
        } else {
            vec![]
        };
        if crate::routed_fixture::routed_stop_requested(catalog, draft_id, &run_id.0)? {
            return Err(crate::routed_fixture::RoutedPreflightStopped.into());
        }
        // Flow has already won the single-use dispatch CAS before this claim.
        reserve_active_run(&domain, &cfg, &run_id)?;
        #[cfg(feature = "routed-test-faults")]
        pause_routed_fault_stage("PYTXO_TEST_ROUTED_PAUSE_AFTER_RESERVE")?;
        // Stop and registration share the same cross-process gate. Otherwise
        // Stop could clear an unregistered marker just before we commit a live
        // routing mission that it never saw.
        let active_path = repo_root.join(&cfg.data_dir).join("active_run.json");
        let registration_gate = crate::ActiveRunGate::acquire(&active_path)?;
        let still_owned = crate::read_active_run_state(&active_path)?
            .is_some_and(|active| active.run_id == run_id.0);
        let registry_cancelled = pytxo_runner::ProcessRegistryFile::load(
            &pytxo_runner::registry_path(&repo_root.join(&cfg.data_dir)),
        )?
        .cancelled_runs
        .contains(&run_id.0);
        let stop_requested =
            crate::routed_fixture::routed_stop_requested(catalog, draft_id, &run_id.0)?;
        if !still_owned || registry_cancelled || stop_requested {
            drop(registration_gate);
            if stop_requested {
                settle_reserved_run_after_stop(&domain, &cfg, &run_id)
                    .context("routed Stop needs startup recovery")?;
                return Err(RoutedStartupStopped(run_id.0).into());
            }
            settle_reserved_run_after_error(&domain, &cfg, &run_id);
            return Err(RoutedStartupSettled(run_id.0).into());
        }
        let registration = (|| -> anyhow::Result<pytxo_store::routing::RoutingMission> {
            // This exact Catalog snapshot, not an orphan private stage, remains
            // the only reviewed Flow authority after the dispatch claim.
            let claimed = catalog
                .get_flow_draft(draft_id)?
                .context("reviewed routed Flow draft disappeared after dispatch claim")?;
            if claimed.status != "dispatching"
                || claimed.plan_json.as_deref() != Some(expected_plan_json)
                || claimed.domain_id.as_deref() != Some(domain.id.as_str())
                || claimed.project_id != plan.project_id
                || claimed.dispatched_run_id.as_deref() != Some(run_id.0.as_str())
                || plan.draft_id != draft_id
                || serde_json::to_string(plan)? != expected_plan_json
            {
                anyhow::bail!("reviewed routed Flow changed after dispatch claim");
            }
            let store = pytxo_store::PytxoStore::open(&cfg.db_path_at(&repo_root))?;
            let mission = load_reviewed_staged_mission(&store, plan)?;
            store.register_routing_mission(&mission)?;
            Ok(mission)
        })();
        drop(registration_gate);
        // An error after reservation may represent an ambiguous SQLite write.
        // Keep the starting run and active marker for recovery rather than
        // clearing ownership around a possibly admit-eligible mission.
        let mission = registration
            .context("routed mission registration failed; active run retained for recovery")?;
        let store = pytxo_store::PytxoStore::open(&cfg.db_path_at(&repo_root))?;
        if fixture_route || claude_route {
            if mission.tasks.len() != prequalified.len()
                || prequalified
                    .iter()
                    .any(|wave| mission.profiles.len() != wave.len())
            {
                anyhow::bail!("reviewed local qualification changed after run claim");
            }
            bind_routed_review_contract(&store, plan, &mission, &domain, &cfg)?;
            let data_dir = repo_root.join(&cfg.data_dir);
            #[cfg(feature = "routed-test-faults")]
            let advisor = crate::routed_advisor::mock_fixture_advisor();
            #[cfg(not(feature = "routed-test-faults"))]
            let advisor = None;
            if claude_route {
                crate::routed_supervisor::run_one_claude_proposal(
                    &store,
                    catalog,
                    &mission,
                    plan,
                    &cfg,
                    &repo_root,
                    &data_dir,
                    prequalified
                        .into_iter()
                        .next()
                        .context("Claude route has no wave")?,
                    hosted_client,
                )
                .context("routed Claude proposal requires exact owned recovery after failure")?;
            } else {
                crate::routed_supervisor::run_one_local_fixture(
                    &store,
                    catalog,
                    &mission,
                    plan,
                    &cfg,
                    &repo_root,
                    &data_dir,
                    prequalified,
                    advisor,
                    hosted_client,
                )
                .context("routed local fixture requires exact owned recovery after failure")?;
            }
            return Ok((domain.id.clone(), run_id));
        }
        let run = store
            .get_run(&run_id.0)?
            .context("reviewed routed run disappeared before cancellation")?;
        let started_ms = u64::try_from(run.started_at.timestamp_millis())?;
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: domain.id.clone(),
            run_id: run_id.clone(),
        };
        store
            .cancel_routing_mission(
                &scope,
                &format!("routed-unavailable:{}", run_id.0),
                mission.authorization.cancel_epoch,
                started_ms,
            )
            .context("routing cancellation failed; active run retained for recovery")?;
        // No attempt, worker, check or Apply authority was created. The
        // registered mission is durably cancelled before releasing ownership.
        settle_reserved_run_after_error(&domain, &cfg, &run_id);
        let status = store.get_run_status(&run_id.0)?.map(|(status, _)| status);
        let marker = repo_root.join(&cfg.data_dir).join("active_run.json");
        if status.as_deref() != Some("failed_startup") || marker.exists() {
            anyhow::bail!(
                "routed execution unavailable; reviewed run {} requires startup recovery (status={status:?}, active_marker={})",
                run_id.0,
                marker.exists(),
            );
        }
        Err(RoutedStartupSettled(run_id.0).into())
    }
}

/// Best-effort registration of a domain in the cross-project catalog
/// (`~/.pytxo/hypervisor.db`). Failures are logged, never fatal.
fn register_in_catalog(state: &DomainState, cfg: &PytxoConfig) {
    let db_path = cfg.db_path_at(&state.repo_root);
    if let Err(e) = pytxo_store::Catalog::open_default().and_then(|cat| {
        cat.upsert_domain(
            state.id.as_str(),
            &state.repo_root.to_string_lossy(),
            &db_path.to_string_lossy(),
            None,
        )
    }) {
        tracing::debug!("hypervisor catalog upsert skipped: {e}");
    }
}

/// List all domains the local hypervisor has registered in the catalog.
pub fn list_catalog_domains() -> anyhow::Result<Vec<pytxo_store::CatalogEntry>> {
    let cat = pytxo_store::Catalog::open_default().map_err(|e| anyhow::anyhow!(e))?;
    cat.list_domains().map_err(|e| anyhow::anyhow!(e))
}

/// Catalog row plus per-domain run health (read-only; never merges event streams).
#[derive(Clone, Debug, serde::Serialize)]
pub struct CatalogEntryStatus {
    pub domain_id: String,
    pub repo_root: String,
    pub db_path: String,
    pub project_id: Option<String>,
    pub status: String,
    pub updated_at: String,
    pub active_runs: usize,
    pub latest_run_status: Option<String>,
    pub latest_started_at: Option<String>,
    pub hitl_pending: usize,
    /// `repo_root` still exists on disk. False typically means a moved/deleted
    /// repository, or (before v0.5.0's test isolation fix) a leftover tempdir.
    pub is_available: bool,
    /// `repo_root` resolves inside the OS temp directory — almost always a stale
    /// test artifact rather than a real workspace the operator opened.
    pub is_temporary: bool,
}

impl From<pytxo_store::CatalogEntry> for CatalogEntryStatus {
    fn from(e: pytxo_store::CatalogEntry) -> Self {
        let is_available = Path::new(&e.repo_root).exists();
        let is_temporary = is_temporary_path(&e.repo_root);
        Self {
            domain_id: e.domain_id,
            repo_root: e.repo_root,
            db_path: e.db_path,
            project_id: e.project_id,
            status: e.status,
            updated_at: e.updated_at,
            active_runs: 0,
            latest_run_status: None,
            latest_started_at: None,
            hitl_pending: 0,
            is_available,
            is_temporary,
        }
    }
}

/// Heuristic: does `path` resolve inside the OS temp directory? Catches
/// `tempfile`-crate test fixtures regardless of their random prefix.
fn is_temporary_path(path: &str) -> bool {
    let candidate = Path::new(path);
    let temp_dir = std::env::temp_dir();
    if candidate.starts_with(&temp_dir)
        || dunce_lossy(candidate).starts_with(&dunce_lossy(&temp_dir))
    {
        return true;
    }
    // `std::env::temp_dir()` returns the raw TEMP/TMP value, which Windows
    // commonly sets to an 8.3 short path (e.g. `MATTBA~1`) even though
    // `ensure_domain` canonicalizes `repo_root` to its long form before
    // storing it. Canonicalize the (always-present) temp root itself to
    // resolve that mismatch; `repo_root` may point at an already-deleted
    // tempdir, so only the temp root — never `candidate` — is canonicalized.
    match std::fs::canonicalize(&temp_dir) {
        Ok(canonical) => {
            candidate.starts_with(&canonical)
                || dunce_lossy(candidate).starts_with(&dunce_lossy(&canonical))
        }
        Err(_) => false,
    }
}

/// Best-effort case/prefix-insensitive comparison for Windows short-path and
/// `\\?\` long-path prefix differences between `temp_dir()` and stored paths.
fn dunce_lossy(p: &Path) -> String {
    p.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_ascii_lowercase()
}

/// Enriched catalog for hypervisor dashboard ([[execution-domains]] Phase 3).
pub fn list_catalog_domains_enriched() -> anyhow::Result<Vec<CatalogEntryStatus>> {
    let entries = list_catalog_domains()?;
    let hv = default_hypervisor();
    let in_memory_domains: std::collections::HashMap<String, usize> = hv
        .list_domains()
        .into_iter()
        .filter_map(|d| {
            hv.domain_hitl_pending(&d.domain_id)
                .map(|n| (d.domain_id, n))
        })
        .collect();

    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let mut row = CatalogEntryStatus::from(entry);
        if let Ok(store) = pytxo_store::PytxoStore::open(std::path::Path::new(&row.db_path)) {
            if let Ok(summary) = store.domain_run_summary() {
                row.active_runs = summary.active_runs;
                row.latest_run_status = summary.latest_run_status;
                row.latest_started_at = summary.latest_started_at;
            }
        }
        row.hitl_pending = in_memory_domains.get(&row.domain_id).copied().unwrap_or(0);
        out.push(row);
    }
    Ok(out)
}

/// Remove a domain from the global catalog only (`~/.pytxo/hypervisor.db`).
/// Never touches the repository on disk or the domain's own per-repo store.
/// Refuses when the domain has active runs or pending HITL approvals so an
/// operator cannot accidentally hide work still in flight.
pub fn forget_catalog_domain(domain_id: &str) -> anyhow::Result<()> {
    let rows = list_catalog_domains_enriched()?;
    let row = rows
        .iter()
        .find(|r| r.domain_id == domain_id)
        .ok_or_else(|| anyhow::anyhow!("domain not found in catalog: {domain_id}"))?;
    if row.active_runs > 0 {
        anyhow::bail!(
            "cannot forget domain with {} active run(s)",
            row.active_runs
        );
    }
    if row.hitl_pending > 0 {
        anyhow::bail!(
            "cannot forget domain with {} pending approval(s)",
            row.hitl_pending
        );
    }
    let cat = pytxo_store::Catalog::open_default().map_err(|e| anyhow::anyhow!(e))?;
    cat.delete_domain(domain_id)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

static DEFAULT_HYPERVISOR: OnceLock<HypervisorRegistry> = OnceLock::new();

pub fn default_hypervisor() -> &'static HypervisorRegistry {
    DEFAULT_HYPERVISOR.get_or_init(HypervisorRegistry::new)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::OnceLock;

    /// `dispatch_with_config_snapshot` best-effort-registers the domain in the
    /// real `~/.pytxo/hypervisor.db` catalog via `register_in_catalog`. Point
    /// `PYTXO_HOME` at a throwaway directory once per test process so this
    /// tempdir repo never pollutes the developer's actual catalog.
    pub(crate) fn isolate_pytxo_home() {
        static HOME: OnceLock<()> = OnceLock::new();
        HOME.get_or_init(|| {
            let dir = tempfile::tempdir().expect("pytxo home tempdir");
            let path = dir.path().to_path_buf();
            std::mem::forget(dir);
            // SAFETY: test-only; isolates the hypervisor catalog from the developer's ~/.pytxo store.
            unsafe { std::env::set_var("PYTXO_HOME", &path) };
        });
    }

    #[test]
    fn snapshot_dispatch_does_not_reload_changed_config() {
        isolate_pytxo_home();
        let repo = tempfile::tempdir().unwrap();
        let cfg = PytxoConfig::default();
        fs::write(repo.path().join("pytxo.toml"), "this is not valid toml").unwrap();
        let opts = RunOptions {
            agents: 1,
            cmd: "unused".into(),
            config: None,
            dry_run: true,
            keep_worktrees: false,
            repo: Some(repo.path().to_path_buf()),
            execution: None,
            project: None,
            tasks: None,
            task_cmd_template: None,
            task_prompts: None,
        };

        HypervisorRegistry::new()
            .dispatch_with_config_snapshot(opts, cfg)
            .expect("validated snapshot must be used without reloading pytxo.toml");
    }

    #[cfg(windows)]
    #[test]
    fn ensure_domain_normalizes_verbatim_path_before_caching() {
        isolate_pytxo_home();
        let repo = tempfile::tempdir().unwrap();
        let verbatim = fs::canonicalize(repo.path()).unwrap();
        let ordinary = pytxo_core::canonical_repo_root(repo.path()).unwrap();
        assert_ne!(verbatim, ordinary);

        let registry = HypervisorRegistry::new();
        let cfg = PytxoConfig::default();
        let first = registry.ensure_domain(&verbatim, &cfg).unwrap();
        let second = registry.ensure_domain(&ordinary, &cfg).unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.repo_root, ordinary);

        let run_id = RunId("normalized-domain-owner".into());
        crate::reserve_active_run(&first, &cfg, &run_id).unwrap();
        let marker: crate::ActiveRunState =
            serde_json::from_slice(&fs::read(first.data_dir.join("active_run.json")).unwrap())
                .unwrap();
        assert_eq!(marker.run_id, run_id.0);
        assert_eq!(marker.repo_root, ordinary.to_string_lossy());
        assert_eq!(marker.supervisor_pid, std::process::id());
        assert!(marker.supervisor_start_identity.is_some());
    }

    #[test]
    fn is_temporary_path_detects_tempdir_even_through_short_path_form() {
        // On Windows, `std::env::temp_dir()` returns the raw TEMP/TMP value,
        // which is frequently an 8.3 short path (e.g. `MATTBA~1`), while
        // `ensure_domain` stores a canonicalized (long-form) `repo_root`.
        // Round-tripping through `canonicalize` reproduces that mismatch
        // regardless of how this specific machine's temp path is configured.
        let dir = tempfile::tempdir().unwrap();
        let stored = std::fs::canonicalize(dir.path()).unwrap_or_else(|_| dir.path().to_path_buf());
        assert!(
            is_temporary_path(&stored.to_string_lossy()),
            "canonicalized tempdir path {stored:?} must be detected as temporary"
        );
    }

    #[test]
    fn is_temporary_path_rejects_paths_outside_the_temp_root() {
        let workspace = env!("CARGO_MANIFEST_DIR");
        assert!(!is_temporary_path(workspace));
    }

    #[test]
    fn routed_fixture_rejects_an_overlay_receipt_for_a_worktree_launch() {
        let mut cfg = PytxoConfig::default();
        cfg.blast.prefer_kernel_overlay = false;
        require_routed_worktree_isolation(&cfg).unwrap();
        cfg.isolation = pytxo_core::IsolationMode::Overlay;
        assert!(require_routed_worktree_isolation(&cfg).is_err());
    }
}
