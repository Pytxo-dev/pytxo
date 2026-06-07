#[cfg(feature = "overlay-fuse")]
use std::path::Path;

#[cfg(feature = "overlay-fuse")]
use pytxo_core::PytxoError;
use pytxo_core::{IsolationBackend, IsolationCtx, IsolationMode, Result, WorkspaceHandle};

use crate::git::{branch_name, create_worktree, remove_worktree, worktree_path};

#[cfg(feature = "overlay-fuse")]
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| PytxoError::Runner(format!("overlay mkdir: {e}")))?;
    for entry in
        std::fs::read_dir(src).map_err(|e| PytxoError::Runner(format!("overlay readdir: {e}")))?
    {
        let entry = entry.map_err(|e| PytxoError::Runner(format!("overlay entry: {e}")))?;
        let name = entry.file_name();
        let from = entry.path();
        let to = dst.join(&name);
        if entry
            .file_type()
            .map_err(|e| PytxoError::Runner(e.to_string()))?
            .is_dir()
        {
            copy_dir_recursive(&from, &to)?;
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
    copy_dir_recursive(&ctx.repo_root, &upper)?;
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

/// Blast Shield north-star backend — sparse overlay (FUSE / ProjFS). Not yet mounted.
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
                // POC: no git merge; caller may copy upper layer manually.
                return Ok(());
            }
        }
        self.fallback.flush(ctx, handle)
    }
}

pub fn isolation_for_mode(mode: IsolationMode) -> Box<dyn IsolationBackend> {
    match mode {
        IsolationMode::Worktree => Box::new(WorktreeIsolation),
        IsolationMode::Overlay => Box::new(OverlayIsolation::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        std::fs::write(repo.join("README.md"), "x\n").unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(&wt).unwrap();
        let ctx = IsolationCtx {
            run_id: RunId("run1".into()),
            agent_id: AgentId("agent-0".into()),
            repo_root: repo.clone(),
            worktree_base: wt,
        };
        let handle = OverlayIsolation::new().prepare(&ctx).unwrap();
        assert_eq!(handle.backend, IsolationMode::Overlay);
        assert!(handle.branch.is_empty());
        assert!(handle
            .cwd
            .to_string_lossy()
            .contains("overlay-run1-agent-0"));
        assert!(handle.cwd.join("README.md").exists());
    }
}
