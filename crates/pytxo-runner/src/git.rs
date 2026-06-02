use std::path::{Path, PathBuf};
use std::process::Command;

use pytxo_core::{strip_extended_path, PytxoError, Result};

pub fn create_worktree(repo_root: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
    if worktree_path.exists() {
        return Err(PytxoError::Runner(format!(
            "worktree path already exists: {}",
            worktree_path.display()
        )));
    }
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    git(
        repo_root,
        &[
            "worktree",
            "add",
            "-b",
            branch,
            &path_to_git(worktree_path),
            "HEAD",
        ],
    )?;

    Ok(())
}

pub fn remove_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch: &str,
    force: bool,
) -> Result<()> {
    if worktree_path.exists() {
        let path = path_to_git(worktree_path);
        if force {
            let _ = git(repo_root, &["worktree", "remove", "--force", &path]);
        } else {
            let _ = git(repo_root, &["worktree", "remove", &path]);
        }
    }
    let _ = git(repo_root, &["branch", "-D", branch]);
    Ok(())
}

pub fn branch_name(run_id: &str, agent_id: &str) -> String {
    format!("pytxo/{run_id}/{agent_id}")
}

pub fn worktree_path(base: &Path, run_id: &str, agent_id: &str) -> PathBuf {
    base.join(run_id).join(agent_id)
}

fn git(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| PytxoError::Runner(format!("spawn git: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PytxoError::Runner(format!(
            "git {} failed: {stderr}",
            args.join(" ")
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn path_to_git(path: &Path) -> String {
    strip_extended_path(path.to_path_buf())
        .to_string_lossy()
        .to_string()
}
