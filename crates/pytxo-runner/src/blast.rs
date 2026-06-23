#[cfg(feature = "overlay-fuse")]
use std::path::Path;

#[cfg(feature = "overlay-fuse")]
use pytxo_core::PytxoError;
use pytxo_core::{IsolationBackend, IsolationCtx, IsolationMode, Result, WorkspaceHandle};

use crate::git::{branch_name, create_worktree, remove_worktree, worktree_path};

#[cfg(feature = "overlay-fuse")]
fn should_skip_dir(name: &str, sparse_exclude: &[String]) -> bool {
    sparse_exclude.iter().any(|e| e == name)
}

#[cfg(feature = "overlay-fuse")]
fn copy_dir_recursive(src: &Path, dst: &Path, sparse_exclude: &[String]) -> Result<()> {
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
        let to = dst.join(&name);
        if entry
            .file_type()
            .map_err(|e| PytxoError::Runner(e.to_string()))?
            .is_dir()
        {
            copy_dir_recursive(&from, &to, &[])?;
        } else {
            std::fs::copy(&from, &to)
                .map_err(|e| PytxoError::Runner(format!("overlay copy: {e}")))?;
        }
    }
    Ok(())
}

#[cfg(feature = "overlay-fuse")]
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

/// Blast Shield north-star backend — sparse overlay (FUSE / ProjFS).
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
        #[cfg(all(feature = "overlay-projfs", target_os = "windows"))]
        {
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
        #[cfg(feature = "overlay-fuse")]
        {
            return prepare_overlay_layer(ctx);
        }
        #[cfg(not(feature = "overlay-fuse"))]
        {
            let mut handle = self.fallback.prepare(ctx)?;
            handle.backend = IsolationMode::Overlay;
            Ok(handle)
        }
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
        #[cfg(all(feature = "overlay-projfs", target_os = "windows"))]
        {
            if handle.cwd.to_string_lossy().contains("projfs-") {
                return crate::overlay_projfs::rollback_projfs_overlay(handle);
            }
        }
        #[cfg(feature = "overlay-fuse")]
        {
            if handle.branch.is_empty() && handle.cwd.to_string_lossy().contains("overlay-") {
                std::fs::remove_dir_all(handle.cwd.parent().unwrap_or(&handle.cwd)).ok();
                return Ok(());
            }
        }
        self.fallback.rollback(ctx, handle)
    }

    fn flush(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()> {
        #[cfg(feature = "overlay-fuse")]
        {
            if handle.branch.is_empty() {
                return flush_overlay_upper(&ctx.repo_root, &handle.cwd);
            }
        }
        self.fallback.flush(ctx, handle)
    }
}

#[cfg(feature = "overlay-fuse")]
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

/// Human-readable active isolation backend for Deck telemetry (Phase 33).
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
            #[cfg(all(feature = "overlay-projfs", target_os = "windows"))]
            {
                return crate::overlay_projfs::capability_probe().into();
            }
            #[cfg(feature = "overlay-fuse")]
            return format!(
                "overlay-copy-layer (excludes: {})",
                sparse_exclude.join(",")
            );
            #[cfg(not(feature = "overlay-fuse"))]
            return format!(
                "overlay-worktree-fallback (excludes: {})",
                sparse_exclude.join(",")
            );
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
            label.starts_with("overlay-"),
            "expected overlay backend label, got {label}"
        );
        assert!(label.contains("node_modules"));
    }

    #[test]
    fn overlay_mode_reports_overlay() {
        let backend = OverlayIsolation::new();
        assert_eq!(backend.mode(), IsolationMode::Overlay);
    }

    #[cfg(feature = "overlay-fuse")]
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
        assert!(handle
            .cwd
            .to_string_lossy()
            .contains("overlay-run1-agent-0"));
        assert!(handle.cwd.join("README.md").exists());
        assert!(!handle.cwd.join("node_modules").exists());
    }
}
