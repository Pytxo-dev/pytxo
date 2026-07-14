use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use pytxo_core::{DomainId, ProjectManifest, PytxoConfig, RunId};
use pytxo_runner::{HitlQueue, McpHub, ProcessRegistry, SwarmRegistry};
use tracing::error;

use crate::{
    ensure_repo_trusted, execute_run_body, load_config, plan_tasks, resolve_repo_root,
    resolve_run_tasks, RunOptions,
};

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
        let data_dir = repo_root.join(&cfg.data_dir);
        let db_path = cfg.db_path_at(repo_root);
        let hitl = {
            let audit_db = db_path.clone();
            HitlQueue::with_persistence(&data_dir).with_wal_audit(Arc::new(move |id, approved| {
                if let Ok(store) = pytxo_store::PytxoStore::open(&audit_db) {
                    let payload = serde_json::json!({
                        "request_id": id,
                        "decision": if approved { "approved" } else { "denied" },
                    })
                    .to_string();
                    let _ = store.append_event("hitl-resolver", "hitl-resolve", &payload);
                }
            }))
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
    if candidate.starts_with(&temp_dir) || dunce_lossy(candidate).starts_with(&dunce_lossy(&temp_dir)) {
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
    p.to_string_lossy().trim_start_matches(r"\\?\").to_ascii_lowercase()
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
        anyhow::bail!("cannot forget domain with {} active run(s)", row.active_runs);
    }
    if row.hitl_pending > 0 {
        anyhow::bail!(
            "cannot forget domain with {} pending approval(s)",
            row.hitl_pending
        );
    }
    let cat = pytxo_store::Catalog::open_default().map_err(|e| anyhow::anyhow!(e))?;
    cat.delete_domain(domain_id).map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

static DEFAULT_HYPERVISOR: OnceLock<HypervisorRegistry> = OnceLock::new();

pub fn default_hypervisor() -> &'static HypervisorRegistry {
    DEFAULT_HYPERVISOR.get_or_init(HypervisorRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    /// `dispatch_with_config_snapshot` best-effort-registers the domain in the
    /// real `~/.pytxo/hypervisor.db` catalog via `register_in_catalog`. Point
    /// `PYTXO_HOME` at a throwaway directory once per test process so this
    /// tempdir repo never pollutes the developer's actual catalog.
    fn isolate_pytxo_home() {
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
}
