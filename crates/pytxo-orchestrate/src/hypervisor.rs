use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use pytxo_core::{DomainId, ProjectManifest, PytxoConfig, RunId};
use pytxo_runner::{HitlQueue, McpHub, ProcessRegistry, SwarmRegistry};
use pytxo_scheduler::build_plan;
use tracing::error;

use crate::{execute_run_body, load_config, resolve_repo_root, synthetic_tasks, RunOptions};

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
            )
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
        let id = DomainId::from_repo_root(repo_root).map_err(|e| anyhow::anyhow!(e))?;
        let mut guard = self
            .domains
            .lock()
            .map_err(|_| anyhow::anyhow!("hypervisor lock poisoned"))?;
        if let Some(existing) = guard.get(&id) {
            return Ok(Arc::clone(existing));
        }
        fs::create_dir_all(repo_root.join(&cfg.worktree_dir))?;
        fs::create_dir_all(repo_root.join(&cfg.data_dir))?;
        let state = Arc::new(DomainState {
            id: id.clone(),
            repo_root: repo_root.to_path_buf(),
            data_dir: repo_root.join(&cfg.data_dir),
            swarm: SwarmRegistry::new(),
            process_registry: ProcessRegistry::default(),
            hitl: HitlQueue::new(),
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

    pub async fn run_blocking(&self, opts: RunOptions) -> anyhow::Result<RunId> {
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
        let mut cfg = load_config(opts.config.as_deref(), &repo_root)?;
        if opts.agents > 0 {
            cfg.max_agents = opts.agents;
        }
        let domain = self.ensure_domain(&repo_root, &cfg)?;
        let domain_id = domain.id.clone();
        let run_id = RunId::new();
        let run_id_for_task = run_id.clone();

        if opts.dry_run {
            let _ = domain;
            let tasks = if cfg.task.is_empty() {
                synthetic_tasks(cfg.max_agents)
            } else {
                cfg.tasks()
            };
            let _plan = build_plan(&tasks, cfg.max_agents, cfg.dag_explicit_deps)
                .map_err(|e| anyhow::anyhow!(e))?;
            return Ok((domain_id, run_id));
        }

        tokio::spawn(async move {
            if let Err(e) = execute_run_body(domain, opts, cfg, Some(run_id_for_task.clone())).await
            {
                error!("dispatch run {} failed: {e:#}", run_id_for_task.0);
            }
        });

        Ok((domain_id, run_id))
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

static DEFAULT_HYPERVISOR: OnceLock<HypervisorRegistry> = OnceLock::new();

pub fn default_hypervisor() -> &'static HypervisorRegistry {
    DEFAULT_HYPERVISOR.get_or_init(HypervisorRegistry::new)
}
