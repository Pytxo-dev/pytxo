//! macOS FUSE overlay POC ([[sparse-overlay-fs]], Phase 33B).
//!
//! When macFUSE is available, attempts a userspace mount; otherwise callers fall back
//! to the copy-layer path in `blast.rs`.

use std::path::Path;
use std::process::Command;

use pytxo_core::{IsolationMode, PytxoError, Result, WorkspaceHandle};

pub fn fuse_available() -> bool {
    Path::new("/dev/macfuse").exists() || Path::new("/dev/fuse").exists()
}

fn mount_overlay_available() -> bool {
    Command::new("mount")
        .args(["-t", "overlay"])
        .status()
        .map(|s| s.success() || s.code() == Some(1))
        .unwrap_or(false)
}

/// Honest capability label for Deck / doctor telemetry (Phase 46, 55).
pub fn capability_probe() -> &'static str {
    if fuse_available() && mount_overlay_available() {
        "macos-sparse-overlay-v2"
    } else if fuse_available() {
        "macos-fuse-sparse-pending"
    } else {
        "macos-sparse-copy-v2"
    }
}

pub fn prepare_macos_overlay(
    repo_root: &Path,
    worktree_base: &Path,
    run_id: &str,
    agent_id: &str,
    sparse_exclude: &[String],
) -> Result<WorkspaceHandle> {
    let layer = worktree_base.join(format!("overlay-{run_id}-{agent_id}"));
    let upper = layer.join("upper");
    let work = layer.join("work");
    let mnt = layer.join("mnt");
    for p in [&upper, &work, &mnt] {
        std::fs::create_dir_all(p)
            .map_err(|e| PytxoError::Runner(format!("overlay mkdir: {e}")))?;
    }

    if try_overlay_mount_sparse(repo_root, sparse_exclude, &upper, &work, &mnt).is_ok() {
        return Ok(WorkspaceHandle {
            cwd: mnt,
            branch: String::new(),
            backend: IsolationMode::Overlay,
        });
    }

    Err(PytxoError::Runner(
        "macOS overlay mount unavailable; use overlay-fuse copy layer".into(),
    ))
}

pub fn rollback_macos_overlay(handle: &WorkspaceHandle) -> Result<()> {
    let mnt = handle.cwd.clone();
    let _ = Command::new("umount").arg(&mnt).status();
    if let Some(layer) = mnt.parent() {
        std::fs::remove_dir_all(layer).ok();
    }
    Ok(())
}

fn try_overlay_mount_sparse(
    lower: &Path,
    sparse_exclude: &[String],
    upper: &Path,
    work: &Path,
    mnt: &Path,
) -> Result<()> {
    let lowerdir = build_sparse_lowerdir(lower, sparse_exclude);
    let options = format!(
        "lowerdir={lowerdir},upperdir={},workdir={}",
        upper.display(),
        work.display()
    );
    let status = Command::new("mount")
        .args([
            "-t",
            "overlay",
            "overlay",
            "-o",
            &options,
            &mnt.to_string_lossy(),
        ])
        .status()
        .map_err(|e| PytxoError::Runner(format!("overlay mount: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(PytxoError::Runner("overlay mount failed".into()))
    }
}

fn build_sparse_lowerdir(repo_root: &Path, sparse_exclude: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(repo_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if sparse_exclude.iter().any(|e| e == &name) {
                continue;
            }
            parts.push(entry.path().display().to_string());
        }
    }
    if parts.is_empty() {
        repo_root.display().to_string()
    } else {
        parts.join(":")
    }
}
