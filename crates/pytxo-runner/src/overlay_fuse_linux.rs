//! Linux kernel overlay mount POC ([[sparse-overlay-fs]]).
//!
//! Uses the kernel `overlay` filesystem when available; falls back to copy-layer.

use std::path::Path;
use std::process::Command;

use pytxo_core::{IsolationMode, PytxoError, Result, WorkspaceHandle};

pub fn fuse_available() -> bool {
    Path::new("/dev/fuse").exists()
}

pub fn prepare_kernel_overlay(
    repo_root: &Path,
    worktree_base: &Path,
    run_id: &str,
    agent_id: &str,
) -> Result<WorkspaceHandle> {
    let layer = worktree_base.join(format!("overlay-{run_id}-{agent_id}"));
    let upper = layer.join("upper");
    let work = layer.join("work");
    let mnt = layer.join("mnt");
    for p in [&upper, &work, &mnt] {
        std::fs::create_dir_all(p).map_err(|e| PytxoError::Runner(format!("overlay mkdir: {e}")))?;
    }

    if try_overlay_mount(repo_root, &upper, &work, &mnt).is_ok() {
        return Ok(WorkspaceHandle {
            cwd: mnt,
            branch: String::new(),
            backend: IsolationMode::Overlay,
        });
    }

    Err(PytxoError::Runner(
        "kernel overlay mount unavailable; use overlay-fuse copy layer".into(),
    ))
}

pub fn rollback_kernel_overlay(handle: &WorkspaceHandle) -> Result<()> {
    let mnt = handle.cwd.clone();
    let _ = Command::new("umount").arg(&mnt).status();
    if let Some(layer) = mnt.parent() {
        std::fs::remove_dir_all(layer).ok();
    }
    Ok(())
}

fn try_overlay_mount(lower: &Path, upper: &Path, work: &Path, mnt: &Path) -> Result<()> {
    let options = format!(
        "lowerdir={},upperdir={},workdir={}",
        lower.display(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires root/CAP_SYS_ADMIN and overlay fs"]
    fn kernel_overlay_smoke() {
        if !fuse_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(repo.join("README.md"), "x\n").unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(&wt).unwrap();
        let handle = prepare_kernel_overlay(&repo, &wt, "run1", "agent-0").unwrap();
        assert!(handle.cwd.join("README.md").exists());
        rollback_kernel_overlay(&handle).unwrap();
    }
}
