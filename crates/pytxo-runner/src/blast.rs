use std::path::Path;

use pytxo_core::{IsolationBackend, IsolationCtx, IsolationMode, PytxoError, Result, WorkspaceHandle};

use crate::git::{branch_name, create_worktree, remove_worktree, worktree_path};

fn should_skip_dir(name: &str, sparse_exclude: &[String]) -> bool {
    sparse_exclude.iter().any(|e| e == name)
}

/// Sparse copy-layer: materialize `repo_root` into `dst`, skipping top-level `sparse_exclude` names.
/// Works for git and non-git trees (Phase 69). Kernel FUSE/ProjFS remain preferred when available.
fn copy_dir_recursive(src: &Path, dst: &Path, sparse_exclude: &[String]) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| PytxoError::Runner(format!("overlay mkdir: {e}")))?;
    let source_root = std::fs::canonicalize(src)
        .map_err(|e| PytxoError::Runner(format!("overlay canonical source: {e}")))?;
    let destination_root = std::fs::canonicalize(dst)
        .map_err(|e| PytxoError::Runner(format!("overlay canonical destination: {e}")))?;
    copy_dir_recursive_inner(
        &source_root,
        &destination_root,
        sparse_exclude,
        &destination_root,
    )
}

fn copy_dir_recursive_inner(
    src: &Path,
    dst: &Path,
    sparse_exclude: &[String],
    destination_root: &Path,
) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| PytxoError::Runner(format!("overlay mkdir: {e}")))?;
    for entry in
        std::fs::read_dir(src).map_err(|e| PytxoError::Runner(format!("overlay readdir: {e}")))?
    {
        let entry = entry.map_err(|e| PytxoError::Runner(format!("overlay entry: {e}")))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if should_skip_dir(&name_str, sparse_exclude) {
            continue;
        }
        let from = entry.path();
        if from.starts_with(destination_root) {
            continue;
        }
        let to = dst.join(&name);
        if entry
            .file_type()
            .map_err(|e| PytxoError::Runner(e.to_string()))?
            .is_dir()
        {
            copy_dir_recursive_inner(&from, &to, &[], destination_root)?;
        } else {
            std::fs::copy(&from, &to)
                .map_err(|e| PytxoError::Runner(format!("overlay copy: {e}")))?;
        }
    }
    Ok(())
}

/// Non-git overlay upper: copy-layer under `overlay-{run}-{agent}/upper` (Phase 69 default fallback).
fn prepare_overlay_layer(ctx: &IsolationCtx) -> Result<WorkspaceHandle> {
    let layer = ctx
        .worktree_base
        .join(format!("overlay-{}-{}", ctx.run_id.0, ctx.agent_id.0));
    let upper = layer.join("upper");
    if upper.exists() {
        std::fs::remove_dir_all(&upper).ok();
    }
    copy_dir_recursive(&ctx.repo_root, &upper, &ctx.sparse_exclude)?;
    Ok(WorkspaceHandle {
        cwd: upper,
        branch: String::new(),
        backend: IsolationMode::Overlay,
    })
}

/// Blast Shield MVP backend — git worktree isolation ([[ADR-0005-worktree-isolation-for-mvp]]).
#[derive(Clone, Default)]
pub struct WorktreeIsolation;

impl IsolationBackend for WorktreeIsolation {
    fn mode(&self) -> IsolationMode {
        IsolationMode::Worktree
    }

    fn prepare(&self, ctx: &IsolationCtx) -> Result<WorkspaceHandle> {
        let branch = branch_name(&ctx.run_id.0, &ctx.agent_id.0);
        let cwd = worktree_path(&ctx.worktree_base, &ctx.run_id.0, &ctx.agent_id.0);
        create_worktree(&ctx.repo_root, &cwd, &branch)?;
        Ok(WorkspaceHandle {
            cwd,
            branch,
            backend: IsolationMode::Worktree,
        })
    }

    fn rollback(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()> {
        remove_worktree(&ctx.repo_root, &handle.cwd, &handle.branch, true)
    }

    fn flush(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()> {
        crate::git::merge_agent_branch(&ctx.repo_root, &handle.branch)
    }
}

/// Blast Shield north-star backend — sparse overlay (FUSE / ProjFS / copy-layer).
///
/// Preference order (Phase 69):
/// 1. Kernel FUSE (Linux) / macFUSE / Windows ProjFS sparse copy when feature + probe OK
/// 2. Always-on sparse copy-layer (git **or** non-git trees)
/// 3. Git worktree only if copy-layer fails (should be rare)
#[derive(Clone, Default)]
pub struct OverlayIsolation {
    fallback: WorktreeIsolation,
}

impl OverlayIsolation {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IsolationBackend for OverlayIsolation {
    fn mode(&self) -> IsolationMode {
        IsolationMode::Overlay
    }

    fn prepare(&self, ctx: &IsolationCtx) -> Result<WorkspaceHandle> {
        #[cfg(all(feature = "overlay-fuse-kernel", target_os = "linux"))]
        {
            if crate::overlay_fuse_linux::fuse_available() {
                if let Ok(handle) = crate::overlay_fuse_linux::prepare_kernel_overlay(
                    &ctx.repo_root,
                    &ctx.worktree_base,
                    &ctx.run_id.0,
                    &ctx.agent_id.0,
                    &ctx.sparse_exclude,
                ) {
                    return Ok(handle);
                }
            }
        }
        #[cfg(all(feature = "overlay-fuse-macos", target_os = "macos"))]
        {
            if crate::overlay_fuse_macos::fuse_available() {
                if let Ok(handle) = crate::overlay_fuse_macos::prepare_macos_overlay(
                    &ctx.repo_root,
                    &ctx.worktree_base,
                    &ctx.run_id.0,
                    &ctx.agent_id.0,
                    &ctx.sparse_exclude,
                ) {
                    return Ok(handle);
                }
            }
        }
        #[cfg(target_os = "windows")]
        {
            // Production interim: sparse copy-layer labeled projfs-* (full ProjFS provider = north star).
            if crate::overlay_projfs::projfs_supported() {
                if let Ok(handle) = crate::overlay_projfs::prepare_projfs_overlay(
                    &ctx.repo_root,
                    &ctx.worktree_base,
                    &ctx.run_id.0,
                    &ctx.agent_id.0,
                    &ctx.sparse_exclude,
                ) {
                    return Ok(handle);
                }
            }
        }
        // Always-on copy-layer: works for non-git directories (Phase 69).
        if let Ok(handle) = prepare_overlay_layer(ctx) {
            return Ok(handle);
        }
        let mut handle = self.fallback.prepare(ctx)?;
        handle.backend = IsolationMode::Overlay;
        Ok(handle)
    }

    fn rollback(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()> {
        #[cfg(all(feature = "overlay-fuse-kernel", target_os = "linux"))]
        {
            if handle.cwd.to_string_lossy().contains("/mnt") {
                return crate::overlay_fuse_linux::rollback_kernel_overlay(handle);
            }
        }
        #[cfg(all(feature = "overlay-fuse-macos", target_os = "macos"))]
        {
            if handle.cwd.to_string_lossy().contains("/mnt") {
                return crate::overlay_fuse_macos::rollback_macos_overlay(handle);
            }
        }
        #[cfg(target_os = "windows")]
        {
            if handle.cwd.to_string_lossy().contains("projfs-") {
                return crate::overlay_projfs::rollback_projfs_overlay(handle);
            }
        }
        if handle.branch.is_empty() && handle.cwd.to_string_lossy().contains("overlay-") {
            std::fs::remove_dir_all(handle.cwd.parent().unwrap_or(&handle.cwd)).ok();
            return Ok(());
        }
        self.fallback.rollback(ctx, handle)
    }

    fn flush(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()> {
        if handle.branch.is_empty() {
            return flush_overlay_upper(&ctx.repo_root, &handle.cwd);
        }
        self.fallback.flush(ctx, handle)
    }
}

fn flush_overlay_upper(repo_root: &Path, upper: &Path) -> Result<()> {
    fn walk_upper(src: &Path, dst: &Path) -> Result<()> {
        for entry in std::fs::read_dir(src)
            .map_err(|e| PytxoError::Runner(format!("overlay flush readdir: {e}")))?
        {
            let entry = entry.map_err(|e| PytxoError::Runner(format!("overlay flush entry: {e}")))?;
            let name = entry.file_name();
            let from = entry.path();
            let to = dst.join(&name);
            if entry
                .file_type()
                .map_err(|e| PytxoError::Runner(e.to_string()))?
                .is_dir()
            {
                std::fs::create_dir_all(&to)
                    .map_err(|e| PytxoError::Runner(format!("overlay flush mkdir: {e}")))?;
                walk_upper(&from, &to)?;
            } else {
                std::fs::copy(&from, &to)
                    .map_err(|e| PytxoError::Runner(format!("overlay flush copy: {e}")))?;
            }
        }
        Ok(())
    }
    walk_upper(upper, repo_root)
}

pub fn isolation_for_mode(mode: IsolationMode) -> Box<dyn IsolationBackend> {
    match mode {
        IsolationMode::Worktree => Box::new(WorktreeIsolation),
        IsolationMode::Overlay => Box::new(OverlayIsolation::new()),
    }
}

/// Resolve isolation mode for a run (Phase 62/69: prefer overlay when doctor probe passes).
pub fn effective_isolation_mode(cfg: &pytxo_core::PytxoConfig) -> IsolationMode {
    if cfg.isolation == IsolationMode::Overlay {
        return IsolationMode::Overlay;
    }
    if cfg.blast.prefer_kernel_overlay && crate::doctor_overlay_probe().is_ok() {
        return IsolationMode::Overlay;
    }
    IsolationMode::Worktree
}

/// Human-readable active isolation backend for Deck telemetry (Phase 33/69).
pub fn isolation_backend_label(mode: IsolationMode, sparse_exclude: &[String]) -> String {
    match mode {
        IsolationMode::Worktree => "worktree".into(),
        IsolationMode::Overlay => {
            #[cfg(all(feature = "overlay-fuse-kernel", target_os = "linux"))]
            if crate::overlay_fuse_linux::fuse_available() {
                return "overlay-kernel-fuse".into();
            }
            #[cfg(all(feature = "overlay-fuse-macos", target_os = "macos"))]
            {
                return crate::overlay_fuse_macos::capability_probe().into();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = sparse_exclude;
                return crate::overlay_projfs::capability_probe().into();
            }
            #[cfg(not(target_os = "windows"))]
            {
                format!(
                    "overlay-copy-layer (excludes: {})",
                    sparse_exclude.join(",")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolation_backend_label_worktree() {
        assert_eq!(
            isolation_backend_label(IsolationMode::Worktree, &[]),
            "worktree"
        );
    }

    #[test]
    fn isolation_backend_label_overlay_reports_sparse_exclude() {
        let label = isolation_backend_label(
            IsolationMode::Overlay,
            &["node_modules".into(), "target".into()],
        );
        assert!(
            label.starts_with("overlay-") || label.starts_with("projfs-"),
            "expected overlay backend label, got {label}"
        );
    }

    #[test]
    fn overlay_mode_reports_overlay() {
        let backend = OverlayIsolation::new();
        assert_eq!(backend.mode(), IsolationMode::Overlay);
    }

    #[test]
    fn prefer_kernel_overlay_defaults_true() {
        let cfg = pytxo_core::PytxoConfig::default();
        assert!(cfg.blast.prefer_kernel_overlay);
    }

    #[test]
    fn effective_isolation_upgrades_when_probe_ok() {
        let cfg = pytxo_core::PytxoConfig::default();
        assert!(cfg.blast.prefer_kernel_overlay);
        // Copy-layer probe always succeeds (Phase 69); effective mode must be Overlay.
        assert!(crate::doctor_overlay_probe().is_ok());
        assert_eq!(
            effective_isolation_mode(&cfg),
            IsolationMode::Overlay
        );
    }

    #[test]
    fn overlay_layer_works_on_non_git_tree() {
        use pytxo_core::{AgentId, RunId};
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(repo.join("node_modules")).unwrap();
        std::fs::write(repo.join("README.md"), "x\n").unwrap();
        std::fs::write(repo.join("node_modules").join("pkg.js"), "x").unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(&wt).unwrap();
        let ctx = IsolationCtx {
            run_id: RunId("run1".into()),
            agent_id: AgentId("agent-0".into()),
            repo_root: repo.clone(),
            worktree_base: wt,
            sparse_exclude: vec!["node_modules".into()],
        };
        let handle = OverlayIsolation::new().prepare(&ctx).unwrap();
        assert_eq!(handle.backend, IsolationMode::Overlay);
        assert!(handle.branch.is_empty());
        assert!(handle.cwd.join("README.md").exists());
        assert!(!handle.cwd.join("node_modules").exists());
    }

    #[test]
    fn overlay_layer_not_git_worktree() {
        use pytxo_core::{AgentId, RunId};
        use std::process::Command;
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        assert!(Command::new("git")
            .args(["init"])
            .current_dir(&repo)
            .status()
            .unwrap()
            .success());
        std::fs::create_dir_all(repo.join("node_modules")).unwrap();
        std::fs::write(repo.join("README.md"), "x\n").unwrap();
        std::fs::write(repo.join("node_modules").join("pkg.js"), "x").unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(&wt).unwrap();
        let ctx = IsolationCtx {
            run_id: RunId("run1".into()),
            agent_id: AgentId("agent-0".into()),
            repo_root: repo.clone(),
            worktree_base: wt,
            sparse_exclude: vec!["node_modules".into()],
        };
        let handle = OverlayIsolation::new().prepare(&ctx).unwrap();
        assert_eq!(handle.backend, IsolationMode::Overlay);
        assert!(handle.branch.is_empty());
        assert!(handle.cwd.join("README.md").exists());
        assert!(!handle.cwd.join("node_modules").exists());
    }

    #[test]
    fn copy_layer_works_with_overlay_base_inside_repo() {
        use pytxo_core::{AgentId, RunId};
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::create_dir_all(repo.join(".pytxo")).unwrap();
        std::fs::write(repo.join("README.md"), "x\n").unwrap();
        std::fs::write(repo.join(".pytxo/keep.toml"), "keep = true\n").unwrap();
        let ctx = IsolationCtx {
            run_id: RunId("run2".into()),
            agent_id: AgentId("agent-1".into()),
            repo_root: repo.clone(),
            worktree_base: repo.join(".pytxo/worktrees"),
            sparse_exclude: Vec::new(),
        };

        let handle = prepare_overlay_layer(&ctx).unwrap();

        assert!(handle.cwd.join("README.md").exists());
        assert!(handle.cwd.join(".pytxo/keep.toml").exists());
    }

    #[cfg(windows)]
    #[test]
    fn copy_layer_normalizes_mixed_case_destination() {
        use pytxo_core::{AgentId, RunId};
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(repo.join("README.md"), "x\n").unwrap();
        let mut alias = repo.to_string_lossy().into_owned();
        if alias.as_bytes().get(1) == Some(&b':') {
            alias.replace_range(0..1, &alias[0..1].to_ascii_lowercase());
        }
        let alias = std::path::PathBuf::from(alias);
        let ctx = IsolationCtx {
            run_id: RunId("run-case".into()),
            agent_id: AgentId("agent-case".into()),
            repo_root: repo,
            worktree_base: alias.join(".pytxo/worktrees"),
            sparse_exclude: Vec::new(),
        };

        let handle = prepare_overlay_layer(&ctx).unwrap();

        assert!(handle.cwd.join("README.md").exists());
    }
}
