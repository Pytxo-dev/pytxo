use pytxo_core::{
    IsolationBackend, IsolationCtx, IsolationMode, PytxoError, Result, WorkspaceHandle,
};

use crate::git::{branch_name, create_worktree, remove_worktree, worktree_path};

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
        // Worktree writes are already on disk; merge/approve is a future git operation.
        let _ = (ctx, handle);
        Ok(())
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
        // Until FUSE/ProjFS lands, delegate to worktrees so runs stay functional.
        let mut handle = self.fallback.prepare(ctx)?;
        handle.backend = IsolationMode::Overlay;
        Ok(handle)
    }

    fn rollback(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()> {
        self.fallback.rollback(ctx, handle)
    }

    fn flush(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()> {
        Err(PytxoError::Runner(
            "overlay flush requires sparse FS mount (not yet implemented)".into(),
        ))
        .or_else(|_| self.fallback.flush(ctx, handle))
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
}
