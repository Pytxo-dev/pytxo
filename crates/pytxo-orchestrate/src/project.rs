//! Modular project orchestration ([[modular-projects]], [[ADR-0011-modular-project-manifest]]).
//!
//! Phase 20: `project run` executes one coordinated hypervisor run on the primary
//! root's domain; tasks with `root = "label"` execute against that root's
//! `repo_root` / worktree. Read-only roots can be merged into agent context.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use pytxo_core::{canonical_repo_root, ProjectManifest, ProjectMeta, ProjectRoot, PytxoConfig};
use pytxo_runner::RootExec;
use pytxo_store::PytxoStore;

use crate::{default_hypervisor, ProjectRunContext, RunOptions};

/// Inputs for `pytxo project run`.
pub struct ProjectRunOptions {
    pub manifest: Option<PathBuf>,
    pub project_id: Option<String>,
    pub cmd: String,
    pub agents: usize,
    pub config: Option<PathBuf>,
    pub dry_run: bool,
}

/// Result of a unified project run (one `run_id` across roots).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectRunResult {
    pub run_id: String,
    pub project_id: String,
    pub primary_root: String,
    /// Writable root labels included in this run.
    pub roots: Vec<String>,
}

/// Aggregated run row for `pytxo project status`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectStatusRow {
    pub run_id: String,
    pub status: String,
    pub started_at: String,
    pub agent_count: usize,
    pub root_ids: Vec<String>,
    /// Agent counts keyed by modular root label.
    pub agents_by_root: HashMap<String, usize>,
}

/// User-facing project root details: label, path, read-only, primary, permission profile.
pub type ProjectRootSummary = (String, String, bool, bool, Option<String>);

fn discover_manifest(manifest: Option<&Path>, id: Option<&str>) -> anyhow::Result<ProjectManifest> {
    let cwd = std::env::current_dir()?;
    let path = ProjectManifest::discover(manifest, id, &cwd)
        .ok_or_else(|| anyhow::anyhow!("no project manifest found (pass --manifest or --id)"))?;
    ProjectManifest::load(&path).map_err(|e| anyhow::anyhow!(e))
}

/// Load a project manifest by explicit path or id.
pub fn project_load(
    manifest: Option<PathBuf>,
    id: Option<String>,
) -> anyhow::Result<ProjectManifest> {
    discover_manifest(manifest.as_deref(), id.as_deref())
}

/// List a project's roots as (label, path, read_only, primary, permission_profile).
pub fn project_roots(
    manifest: Option<PathBuf>,
    id: Option<String>,
) -> anyhow::Result<Vec<ProjectRootSummary>> {
    let m = discover_manifest(manifest.as_deref(), id.as_deref())?;
    Ok(m.roots
        .iter()
        .map(|r| {
            (
                r.effective_label(),
                r.path.to_string_lossy().into_owned(),
                r.read_only,
                r.primary,
                r.permission_profile.map(|p| p.as_str().to_string()),
            )
        })
        .collect())
}

/// Create a new project manifest under `~/.pytxo/projects/<id>.toml`.
pub fn project_init(
    id: &str,
    name: Option<String>,
    roots: Vec<PathBuf>,
) -> anyhow::Result<PathBuf> {
    if roots.is_empty() {
        anyhow::bail!("project init requires at least one --add <path>");
    }
    let manifest = ProjectManifest {
        project: ProjectMeta {
            id: id.to_string(),
            name,
        },
        roots: roots
            .into_iter()
            .enumerate()
            .map(|(i, path)| ProjectRoot {
                path,
                label: None,
                primary: i == 0,
                read_only: false,
                permission_profile: None,
            })
            .collect(),
    };
    let path = ProjectManifest::user_manifest_path(id)
        .ok_or_else(|| anyhow::anyhow!("cannot resolve home directory for project manifest"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    write_manifest(&path, &manifest)?;
    Ok(path)
}

/// Add a root to an existing project manifest.
pub fn project_add_root(
    manifest: Option<PathBuf>,
    id: Option<String>,
    path: PathBuf,
    read_only: bool,
) -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let manifest_path = ProjectManifest::discover(manifest.as_deref(), id.as_deref(), &cwd)
        .ok_or_else(|| anyhow::anyhow!("no project manifest found to update"))?;
    let mut m = ProjectManifest::load(&manifest_path).map_err(|e| anyhow::anyhow!(e))?;
    m.roots.push(ProjectRoot {
        path,
        label: None,
        primary: false,
        read_only,
        permission_profile: None,
    });
    write_manifest(&manifest_path, &m)?;
    Ok(manifest_path)
}

/// Remove a root from an existing project manifest by label or path.
pub fn project_remove_root(
    manifest: Option<PathBuf>,
    id: Option<String>,
    label_or_path: &str,
) -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let manifest_path = ProjectManifest::discover(manifest.as_deref(), id.as_deref(), &cwd)
        .ok_or_else(|| anyhow::anyhow!("no project manifest found to update"))?;
    let mut m = ProjectManifest::load(&manifest_path).map_err(|e| anyhow::anyhow!(e))?;
    let before = m.roots.len();
    m.roots.retain(|r| {
        r.effective_label() != label_or_path
            && r.path.to_string_lossy() != label_or_path
            && r.path.as_path() != Path::new(label_or_path)
    });
    if m.roots.len() == before {
        anyhow::bail!("root not found in project manifest: {label_or_path}");
    }
    if !m.roots.iter().any(|r| r.primary) {
        if let Some(first) = m.roots.first_mut() {
            first.primary = true;
        }
    }
    write_manifest(&manifest_path, &m)?;
    Ok(manifest_path)
}

fn write_manifest(path: &Path, manifest: &ProjectManifest) -> anyhow::Result<()> {
    let toml = toml::to_string_pretty(manifest)
        .map_err(|e| anyhow::anyhow!("serialize project manifest: {e}"))?;
    std::fs::write(path, toml)?;
    Ok(())
}

fn build_roots_map(
    manifest: &ProjectManifest,
    cfg: &PytxoConfig,
) -> anyhow::Result<HashMap<String, RootExec>> {
    let mut roots = HashMap::new();
    let trust_store = pytxo_core::TrustedDomainStore::open_default()
        .map_err(|error| anyhow::anyhow!("cannot read folder trust records: {error}"))?;
    for r in &manifest.roots {
        let canon = canonical_repo_root(&r.path).map_err(|e| anyhow::anyhow!(e))?;
        let trust_ceiling = trust_store.permission_for(&canon).ok_or_else(|| {
            anyhow::anyhow!(
                "project root '{}' is not trusted; trust {} before running this project",
                r.effective_label(),
                canon.display()
            )
        })?;
        let requested = r.permission_profile.unwrap_or(
            cfg.requested_permission_profile
                .unwrap_or(cfg.permission_profile),
        );
        let mut profile = requested;
        if let Some(ceiling) = cfg.permission_ceiling {
            profile = profile.capped_at(ceiling);
        }
        profile = profile.capped_at(trust_ceiling);
        roots.insert(
            r.effective_label(),
            RootExec {
                repo_root: canon.clone(),
                worktree_base: canon.join(&cfg.worktree_dir),
                read_only: r.read_only,
                requested_permission_profile: requested,
                permission_profile: profile,
            },
        );
    }
    Ok(roots)
}

fn sync_project_store(manifest: &ProjectManifest) -> anyhow::Result<()> {
    let Some(db_path) = ProjectManifest::project_db_path(&manifest.project.id) else {
        return Ok(());
    };
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let store = pytxo_store::ProjectStore::open(&db_path)?;
    store.sync_roots(manifest)?;
    Ok(())
}

fn build_project_run_context(
    manifest: &ProjectManifest,
    cfg: &PytxoConfig,
) -> anyhow::Result<ProjectRunContext> {
    let roots = build_roots_map(manifest, cfg)?;
    let readonly_context_roots: Vec<(String, PathBuf)> = manifest
        .roots
        .iter()
        .filter(|r| r.read_only)
        .map(|r| {
            let canon = canonical_repo_root(&r.path).map_err(|e| anyhow::anyhow!(e))?;
            Ok((r.effective_label(), canon))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(ProjectRunContext {
        project_id: manifest.project.id.clone(),
        roots,
        readonly_context_roots,
    })
}

/// Run the task plan once on the primary domain; tasks use `root` labels for other roots.
pub async fn project_run(opts: ProjectRunOptions) -> anyhow::Result<Vec<ProjectRunResult>> {
    let manifest = discover_manifest(opts.manifest.as_deref(), opts.project_id.as_deref())?;
    let project_id = manifest.project.id.clone();
    let primary = manifest.primary_root();
    let primary_canon = manifest
        .primary_repo_root()
        .map_err(|e| anyhow::anyhow!(e))?;

    let cfg = crate::load_config(opts.config.as_deref(), &primary_canon)?;
    let project_ctx = build_project_run_context(&manifest, &cfg)?;

    let writable_labels: Vec<String> = manifest
        .roots
        .iter()
        .filter(|r| !r.read_only)
        .map(|r| r.effective_label())
        .collect();
    if writable_labels.is_empty() {
        anyhow::bail!("project '{project_id}' has no writable roots to run");
    }

    let domain = default_hypervisor().ensure_project(&manifest, &cfg)?;
    let _ = sync_project_store(&manifest);
    let run_id = crate::execute_run_body(
        domain,
        RunOptions {
            agents: opts.agents,
            cmd: opts.cmd.clone(),
            config: opts.config.clone(),
            dry_run: opts.dry_run,
            keep_worktrees: false,
            repo: Some(primary.path.clone()),
            execution: None,
            project: Some(project_ctx),
            tasks: None,
            task_cmd_template: None,
            task_prompts: None,
        },
        cfg,
        None,
    )
    .await?;

    Ok(vec![ProjectRunResult {
        run_id: run_id.0,
        project_id,
        primary_root: primary_canon.to_string_lossy().into_owned(),
        roots: writable_labels,
    }])
}

/// Recent runs for a project across the primary domain WAL.
pub fn project_status(
    manifest: Option<PathBuf>,
    id: Option<String>,
    limit: usize,
) -> anyhow::Result<Vec<ProjectStatusRow>> {
    let manifest = discover_manifest(manifest.as_deref(), id.as_deref())?;
    let project_id = manifest.project.id.clone();
    let primary_canon = manifest
        .primary_repo_root()
        .map_err(|e| anyhow::anyhow!(e))?;
    let cfg = crate::load_config(None, &primary_canon).unwrap_or_default();
    let store = PytxoStore::open(&cfg.db_path_at(&primary_canon))?;
    let runs = store.list_runs(limit)?;

    let mut rows = Vec::new();
    for run in runs {
        if store.run_project(&run.id)?.as_deref() != Some(project_id.as_str()) {
            continue;
        }
        let agents = store.list_agents_for_run(&run.id)?;
        let mut root_ids: Vec<String> = agents.iter().filter_map(|a| a.root_id.clone()).collect();
        root_ids.sort();
        root_ids.dedup();
        let mut agents_by_root: HashMap<String, usize> = HashMap::new();
        for a in &agents {
            let label = a
                .root_id
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "primary".into());
            *agents_by_root.entry(label).or_insert(0) += 1;
        }
        rows.push(ProjectStatusRow {
            run_id: run.id,
            status: run.status,
            started_at: run.started_at.to_rfc3339(),
            agent_count: agents.len(),
            root_ids,
            agents_by_root,
        });
    }
    Ok(rows)
}

fn projects_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|h| PathBuf::from(h).join(".pytxo").join("projects"))
}

/// List project manifest paths from `~/.pytxo/projects/*.toml`.
pub fn list_project_manifests() -> anyhow::Result<Vec<(String, PathBuf)>> {
    let Some(dir) = projects_dir() else {
        return Ok(Vec::new());
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        if let Ok(m) = ProjectManifest::load(&path) {
            out.push((m.project.id, path));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pytxo_core::{PermissionProfile, TrustedDomainStore};

    #[test]
    fn project_root_profile_cannot_exceed_folder_trust_ceiling() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("root");
        std::fs::create_dir(&repo).unwrap();
        let trust_path = dir.path().join("trusted-domains.json");
        TrustedDomainStore::open(&trust_path)
            .unwrap()
            .trust(&repo, PermissionProfile::Orbit, None)
            .unwrap();
        // SAFETY: this test owns the temporary trust store path.
        unsafe { std::env::set_var("PYTXO_TRUST_STORE", &trust_path) };

        let manifest = ProjectManifest {
            project: ProjectMeta {
                id: "ceiling-fixture".into(),
                name: None,
            },
            roots: vec![ProjectRoot {
                path: repo,
                label: Some("root".into()),
                primary: true,
                read_only: false,
                permission_profile: Some(PermissionProfile::Supernova),
            }],
        };

        let roots = build_roots_map(&manifest, &PytxoConfig::default()).unwrap();
        assert_eq!(roots["root"].permission_profile, PermissionProfile::Orbit);
    }
}
