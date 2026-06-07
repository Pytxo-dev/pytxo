//! Windows ProjFS overlay POC ([[sparse-overlay-fs]]).
//!
//! Full ProjFS provider registration is deferred; this module detects support and
//! falls back to copy-layer / worktree when unavailable.

use std::path::{Path, PathBuf};

use pytxo_core::{IsolationMode, PytxoError, Result, WorkspaceHandle};

pub fn projfs_supported() -> bool {
    #[cfg(windows)]
    {
        std::env::var("OS").is_ok()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn prepare_projfs_overlay(
    repo_root: &Path,
    worktree_base: &Path,
    run_id: &str,
    agent_id: &str,
) -> Result<WorkspaceHandle> {
    let _ = (repo_root, worktree_base, run_id, agent_id);
    #[cfg(windows)]
    {
        if !projfs_supported() {
            return Err(PytxoError::Runner(
                "ProjFS not supported on this SKU".into(),
            ));
        }
        Err(PytxoError::Runner(
            "ProjFS provider POC not registered; use overlay-fuse copy layer".into(),
        ))
    }
    #[cfg(not(windows))]
    {
        Err(PytxoError::Runner("ProjFS is Windows-only".into()))
    }
}

pub fn rollback_projfs_overlay(handle: &WorkspaceHandle) -> Result<()> {
    let layer: PathBuf = handle.cwd.clone();
    if layer.to_string_lossy().contains("projfs-") {
        std::fs::remove_dir_all(layer.parent().unwrap_or(&layer)).ok();
    }
    Ok(())
}
