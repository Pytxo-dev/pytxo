//! `--fix`: hands a branch a landing left out, with its fix prompt, to the
//! agent that wrote it. The agent works in a scratch worktree of the branch
//! with the target merged in, or the branch its prompt says to rebase onto,
//! and what it changes there, committed, becomes the branch's new tip.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::detect::AgentBranch;
use crate::{checks, Git, OwnedWorktree, RunDir};

/// What sending a branch to its agent did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sent {
    /// The agent's change, committed: the branch's new tip.
    Fixed(String),
    /// The branch is as it was, and what the agent did instead.
    Unchanged(String),
}

/// The command for `branch`'s agent, which reads the fix prompt on stdin:
/// `git config pytxo.agent.<prefix>`, else `pytxo.agent`, else Claude Code
/// for `claude/*` and Codex for `codex/*`.
pub fn agent_command(git: &Git, branch: &str) -> Option<String> {
    let prefix = branch.split_once('/').map(|(p, _)| p);
    let set = |key: &str| {
        git.run(&["config", "--get", key])
            .ok()
            .filter(|c| !c.is_empty())
    };
    prefix
        .and_then(|p| set(&format!("pytxo.agent.{p}")))
        .or_else(|| set("pytxo.agent"))
        .or_else(|| {
            Some(
                match prefix? {
                    // Edits only, no shell: Pytxo commits what it changes.
                    "claude" => "claude -p --permission-mode acceptEdits",
                    // Writes inside the worktree only.
                    "codex" => "codex exec --sandbox workspace-write -",
                    _ => return None,
                }
                .to_string(),
            )
        })
}

/// Sends `branch` and its fix `prompt` to the agent `command` in a scratch
/// worktree of the branch with `onto` merged in (the target, or the branch the
/// prompt says to rebase onto), the agent's output in `log`. Once the agent
/// exits 0, what it changed there, committed, becomes the branch's tip, and
/// where the branch is checked out its files move too.
pub fn send(
    repo: &Path,
    branch: &AgentBranch,
    onto: &str,
    prompt: &str,
    command: &str,
    log: &Path,
    timeout: Duration,
) -> Result<Sent> {
    let git = Git::user(repo);
    let name = &branch.reference;
    let tag = pytxo_core::new_receipt_id()
        .trim_start_matches("rcpt_")
        .to_lowercase();
    let run_dir = RunDir::new(&tag)?;
    let tree = OwnedWorktree::add(&git, &run_dir.0, &branch.commit, &tag)?;
    // The user's hooks and identity: these commits are theirs.
    let wt = Git::user(&tree.path);
    // The fix goes on top of `onto`, so that is what the agent changes.
    // Named for the branch: the scratch worktree's HEAD is detached.
    let title = format!("Merge branch '{onto}' into {name}");
    let merge = wt.output(&["merge", "--no-edit", "--no-ff", "-m", &title, onto])?;
    let conflicts = wt.lines(&["diff", "--name-only", "--diff-filter=U"])?;
    anyhow::ensure!(
        merge.status.success() || !conflicts.is_empty(),
        "git could not merge {onto} into it: {}",
        String::from_utf8_lossy(&merge.stderr).trim()
    );
    let before = wt.commit_of("HEAD")?;
    let resolve = match conflicts.as_slice() {
        [] => String::new(),
        files => format!(", which left conflicts in {} to resolve", files.join(", ")),
    };
    let prompt = format!(
        "{prompt}\n\nThis directory has {name} with {onto} merged in{resolve}, so skip the rebase and change the files here. Pytxo commits what you change to {name} and checks it again."
    );
    let (code, timed_out) = checks::run_agent(command, &tree.path, &prompt, log, timeout)
        .with_context(|| format!("starting {command}"))?;
    match code {
        _ if timed_out => {
            let after = timeout.as_secs();
            return Ok(Sent::Unchanged(format!("timed out after {after}s")));
        }
        Some(0) => {}
        Some(code) => return Ok(Sent::Unchanged(format!("exited with code {code}"))),
        None => return Ok(Sent::Unchanged("was stopped".into())),
    }
    wt.run(&["add", "-A"])?;
    let check = wt.output(&["diff", "--cached", "--check"])?;
    let mut marked: Vec<String> = String::from_utf8_lossy(&check.stdout)
        .lines()
        .filter(|l| l.ends_with("leftover conflict marker"))
        .filter_map(|l| l.split_once(':').map(|(path, _)| path.to_string()))
        .collect();
    marked.dedup();
    if !marked.is_empty() {
        let files = marked.join(", ");
        return Ok(Sent::Unchanged(format!("left conflict markers in {files}")));
    }
    if wt.ok(&["rev-parse", "-q", "--verify", "MERGE_HEAD"]) {
        wt.run(&["commit", "-q", "--no-edit"])?;
    } else if !wt.ok(&["diff", "--cached", "--quiet"]) {
        let message = format!("Fix {name} on top of {onto}");
        wt.run(&["commit", "-q", "-m", &message])?;
    }
    let head = wt.commit_of("HEAD")?;
    if head == before {
        return Ok(Sent::Unchanged("changed nothing".into()));
    }
    let moved = match &branch.worktree {
        Some(dir) => {
            let there = Git::user(dir);
            if there.commit_of("HEAD")? == branch.commit {
                there.run(&["reset", "-q", "--keep", &head])
            } else {
                let dir = dir.display();
                Err(anyhow::anyhow!(
                    "{name} moved in {dir} while its agent worked"
                ))
            }
        }
        None => {
            let reference = format!("refs/heads/{name}");
            git.run(&["update-ref", &reference, &head, &branch.commit])
        }
    };
    moved.with_context(|| format!("the fix is commit {head}"))?;
    Ok(Sent::Fixed(head))
}
