//! Windows ProjFS overlay POC ([[sparse-overlay-fs]], Phase 33C).
//!
//! Registers a copy-layer upper under `projfs-{run}-{agent}`; full ProjFS provider
//! registration remains a north-star path.

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

/// Honest capability label for Deck / doctor telemetry (Phase 46).
pub fn capability_probe() -> &'static str {
    #[cfg(windows)]
    {
        if projfs_supported() {
            // Full ProjFS provider registration is not shipped; copy-layer POC only.
            "projfs-copy-layer-poc"
        } else {
            "projfs-unavailable"
        }
    }
    #[cfg(not(windows))]
    {
        "projfs-non-windows"
    }
}

pub fn prepare_projfs_overlay(
    repo_root: &Path,
    worktree_base: &Path,
    run_id: &str,
    agent_id: &str,
    sparse_exclude: &[String],
) -> Result<WorkspaceHandle> {
    if !projfs_supported() {
        return Err(PytxoError::Runner(
            "ProjFS not supported on this platform".into(),
        ));
    }
    let layer = worktree_base.join(format!("projfs-{run_id}-{agent_id}"));
    let upper = layer.join("upper");
    if upper.exists() {
        std::fs::remove_dir_all(&upper).ok();
    }
    copy_dir_skip_sparse(repo_root, &upper, sparse_exclude)?;
    Ok(WorkspaceHandle {
        cwd: upper,
        branch: String::new(),
        backend: IsolationMode::Overlay,
    })
}

pub fn rollback_projfs_overlay(handle: &WorkspaceHandle) -> Result<()> {
    let layer: PathBuf = handle.cwd.clone();
    if layer.to_string_lossy().contains("projfs-") {
        std::fs::remove_dir_all(layer.parent().unwrap_or(&layer)).ok();
    }
    Ok(())
}

fn copy_dir_skip_sparse(src: &Path, dst: &Path, sparse_exclude: &[String]) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| PytxoError::Runner(format!("projfs mkdir: {e}")))?;
    for entry in
        std::fs::read_dir(src).map_err(|e| PytxoError::Runner(format!("projfs readdir: {e}")))?
    {
        let entry = entry.map_err(|e| PytxoError::Runner(format!("projfs entry: {e}")))?;
        let name = entry.file_name();
        if sparse_exclude
            .iter()
            .any(|e| e == name.to_string_lossy().as_ref())
        {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        if entry
            .file_type()
            .map_err(|e| PytxoError::Runner(e.to_string()))?
            .is_dir()
        {
            copy_dir_skip_sparse(&from, &to, &[])?;
        } else {
            std::fs::copy(&from, &to)
                .map_err(|e| PytxoError::Runner(format!("projfs copy: {e}")))?;
        }
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn projfs_copy_layer_skips_node_modules() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(repo.join("node_modules/pkg")).unwrap();
        std::fs::write(repo.join("README.md"), "hi\n").unwrap();
        std::fs::write(repo.join("node_modules/pkg/x.js"), "x").unwrap();
        let wt = tmp.path().join("wt");
        let handle = prepare_projfs_overlay(
            &repo,
            &wt,
            "run1",
            "agent-0",
            &["node_modules".into()],
        )
        .unwrap();
        assert!(handle.cwd.join("README.md").exists());
        assert!(!handle.cwd.join("node_modules").exists());
    }
}
