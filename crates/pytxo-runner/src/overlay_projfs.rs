//! Windows Blast overlay path ([[sparse-overlay-fs]], Phase 33C / 69).
//!
//! # Production status (honest)
//!
//! Full Windows **Projected File System** provider registration (virtualized lowerdir
//! with on-demand hydration) remains the north-star path. Until that lands, this module
//! ships a **production-grade sparse copy-layer**:
//!
//! - Upper lives under `projfs-{run}-{agent}/upper`
//! - Top-level `sparse_exclude` dirs (e.g. `node_modules`, `target`) are skipped
//! - Works for **git and non-git** trees (no `git worktree` dependency)
//! - Flush copies the upper back onto `repo_root` via `OverlayIsolation::flush`
//!
//! Capability label: `projfs-sparse-copy-v2`. Doctor / Deck telemetry use this string so
//! operators know they are on the copy-layer interim, not a kernel ProjFS mount.

use std::path::{Path, PathBuf};

use pytxo_core::{IsolationMode, PytxoError, Result, WorkspaceHandle};

/// True on Windows hosts (copy-layer path is always usable; full ProjFS provider TBD).
pub fn projfs_supported() -> bool {
    #[cfg(windows)]
    {
        true
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Honest capability label for Deck / doctor telemetry (Phase 46, 55, 69).
pub fn capability_probe() -> &'static str {
    #[cfg(windows)]
    {
        // Sparse copy-layer v2: skips `sparse_exclude` dirs (e.g. node_modules).
        // Not a kernel ProjFS virtualization provider — see module docs.
        "projfs-sparse-copy-v2"
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
            "ProjFS overlay path not supported on this platform".into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Integration: sparse copy must not materialize excluded top-level dirs.
    #[test]
    fn sparse_copy_excludes_node_modules_from_upper() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(repo.join("node_modules/pkg")).unwrap();
        std::fs::write(repo.join("README.md"), "hi\n").unwrap();
        std::fs::write(repo.join("node_modules/pkg/x.js"), "x").unwrap();
        let upper = tmp.path().join("upper");
        copy_dir_skip_sparse(&repo, &upper, &["node_modules".into()]).unwrap();
        assert!(upper.join("README.md").exists());
        assert!(!upper.join("node_modules").exists());
    }

    #[test]
    fn capability_probe_reports_sparse_copy_v2_on_windows() {
        #[cfg(windows)]
        assert_eq!(capability_probe(), "projfs-sparse-copy-v2");
        #[cfg(not(windows))]
        assert_eq!(capability_probe(), "projfs-non-windows");
    }

    #[cfg(windows)]
    #[test]
    fn prepare_projfs_overlay_skips_node_modules() {
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

    #[cfg(windows)]
    #[test]
    fn prepare_projfs_works_without_git() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("plain");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(repo.join("app.ts"), "export {}\n").unwrap();
        let wt = tmp.path().join("wt");
        let handle =
            prepare_projfs_overlay(&repo, &wt, "run2", "agent-1", &[]).unwrap();
        assert!(handle.branch.is_empty());
        assert!(handle.cwd.join("app.ts").exists());
    }
}
