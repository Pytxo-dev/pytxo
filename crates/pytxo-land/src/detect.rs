//! What `pytxo check` combines when nobody says: the target branch, the agent
//! branches and the project's test command.
//!
//! Detection only reads refs, worktrees and the target's tree. It never
//! checks out, fetches or runs anything.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

use crate::{checks, native_path, supports_merge_tree, Git, LandInput};

/// Stands for every branch checked out in a linked worktree.
pub const WORKTREES: &str = "worktrees";

/// Branch prefixes coding agents use for the branches they create, `agent/`
/// (Pytxo's own convention), and linked worktrees. `*` matches across `/`.
pub const DEFAULT_PATTERNS: [&str; 8] = [
    "claude/*", "codex/*", "copilot/*", "cursor/*", "devin/*", "jules/*", "agent/*", WORKTREES,
];

/// Detected branches without a commit in this many days are left out.
pub const DEFAULT_MAX_AGE_DAYS: u64 = 14;

/// One branch, or a detached worktree's commit, to combine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentBranch {
    /// Passed to the gate and shown: `codex/fix-tax`, `origin/copilot/x`,
    /// or a commit for a detached worktree.
    pub reference: String,
    /// Names a detached worktree's commit.
    pub label: Option<String>,
    pub commit: String,
    /// The worktree that has it checked out.
    pub worktree: Option<PathBuf>,
    /// That worktree has changes nobody committed, which no check can see.
    pub uncommitted: bool,
}

impl AgentBranch {
    pub fn input(&self) -> LandInput {
        LandInput {
            label: self.label.clone(),
            reference: self.reference.clone(),
        }
    }

    pub fn name(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.reference)
    }
}

/// Everything a check needs before it runs.
#[derive(Clone, Debug)]
pub struct Plan {
    /// Root of the worktree the command ran in.
    pub root: PathBuf,
    pub target: String,
    pub branches: Vec<AgentBranch>,
    /// The branches came from the patterns, not from the command line.
    pub detected: bool,
    pub patterns: Vec<String>,
    /// Matching branches left out for having no recent commit.
    pub old: usize,
    pub max_age_days: u64,
    pub checks: Vec<String>,
    /// Where the checks came from: a project file, `--check` or a setting.
    pub check_source: Option<String>,
}

impl Plan {
    /// Changes whenever the target, a branch tip or the checks change.
    pub fn fingerprint(&self) -> String {
        let git = Git::user(&self.root);
        let mut parts = vec![format!(
            "{} {}",
            self.target,
            git.commit_of(&format!("refs/heads/{}", self.target))
                .unwrap_or_default()
        )];
        parts.extend(
            self.branches
                .iter()
                .map(|b| format!("{} {}", b.name(), b.commit)),
        );
        parts.extend(self.checks.iter().cloned());
        parts.join("\n")
    }
}

/// Works out the target, the branches (`given`, else detected) and the checks
/// (`checks`, else configured, else detected from the target's tree).
pub fn plan(cwd: &Path, given: &[String], into: Option<&str>, checks: &[String]) -> Result<Plan> {
    let root = native_path(
        &Git::user(cwd)
            .run(&["rev-parse", "--show-toplevel"])
            .context("not inside a git repository")?,
    );
    let git = Git::user(&root);
    let target = target_branch(&git, into)?;
    let patterns = patterns(&git);
    let max_age_days = setting(&git, "PYTXO_DAYS", "pytxo.days")
        .first()
        .and_then(|d| d.trim().parse().ok())
        .unwrap_or(DEFAULT_MAX_AGE_DAYS);
    let (branches, old) = if given.is_empty() {
        detect(&git, &target, &patterns, max_age_days)?
    } else {
        (expand(&git, given)?, 0)
    };
    let (checks, check_source) = if !checks.is_empty() {
        (checks.to_vec(), Some("--check".to_string()))
    } else {
        let configured = setting(&git, "PYTXO_CHECK", "pytxo.check");
        if !configured.is_empty() {
            let from = if std::env::var_os("PYTXO_CHECK").is_some() {
                "PYTXO_CHECK"
            } else {
                "git config pytxo.check"
            };
            (configured, Some(from.to_string()))
        } else if let Some((command, file)) = test_command(&git, &format!("refs/heads/{target}")) {
            (vec![command], Some(file))
        } else {
            (Vec::new(), None)
        }
    };
    Ok(Plan {
        root,
        target,
        branches,
        detected: given.is_empty(),
        patterns,
        old,
        max_age_days,
        checks,
        check_source,
    })
}

/// `--into`, else the default branch, else the checked-out branch. A hook
/// runs inside an agent's worktree, where the checked-out branch is the
/// agent's own, so the default branch comes first.
pub fn target_branch(git: &Git, into: Option<&str>) -> Result<String> {
    if let Some(t) = into {
        return Ok(t.to_string());
    }
    let exists = |b: &str| git.ok(&["show-ref", "--verify", "--quiet", &format!("refs/heads/{b}")]);
    let remote_head = git
        .run(&["symbolic-ref", "--quiet", "--short", "refs/remotes/origin/HEAD"])
        .ok()
        .and_then(|h| h.strip_prefix("origin/").map(str::to_string));
    if let Some(b) = remote_head
        .into_iter()
        .chain(["main", "master", "trunk"].map(String::from))
        .find(|b| exists(b))
    {
        return Ok(b);
    }
    git.run(&["symbolic-ref", "--quiet", "--short", "HEAD"])
        .context("no target branch: pass --into <branch>")
}

/// `PYTXO_BRANCHES`, else `git config pytxo.branches`, else the defaults.
/// Values split on spaces and commas; a trailing `/` means `/*`.
pub fn patterns(git: &Git) -> Vec<String> {
    let configured = setting(git, "PYTXO_BRANCHES", "pytxo.branches");
    let raw: Vec<String> = if configured.is_empty() {
        DEFAULT_PATTERNS.map(String::from).to_vec()
    } else {
        configured
            .iter()
            .flat_map(|v| v.split(|c: char| c == ',' || c.is_whitespace()))
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect()
    };
    raw.into_iter()
        .map(|p| if p.ends_with('/') { format!("{p}*") } else { p })
        .collect()
}

/// An environment variable, else every value of a git config key.
fn setting(git: &Git, env: &str, key: &str) -> Vec<String> {
    if let Ok(v) = std::env::var(env) {
        if !v.trim().is_empty() {
            return vec![v.trim().to_string()];
        }
    }
    git.lines(&["config", "--get-all", key]).unwrap_or_default()
}

/// `*` matches any run of characters, slashes included; `?` matches one.
fn glob(pattern: &str, text: &str) -> bool {
    let (p, t): (Vec<char>, Vec<char>) = (pattern.chars().collect(), text.chars().collect());
    let (mut pi, mut ti) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some((pi, ti));
            pi += 1;
        } else if let Some((sp, st)) = star {
            pi = sp + 1;
            ti = st + 1;
            star = Some((sp, st + 1));
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

struct Worktree {
    path: PathBuf,
    head: String,
    branch: Option<String>,
    linked: bool,
}

/// `git worktree list --porcelain`, minus bare and missing (prunable) entries.
fn worktrees(git: &Git) -> Vec<Worktree> {
    let Ok(text) = git.run(&["worktree", "list", "--porcelain"]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, block) in text.replace("\r\n", "\n").split("\n\n").enumerate() {
        let (mut path, mut head, mut branch, mut gone) = (None, String::new(), None, false);
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(native_path(p));
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                head = h.to_string();
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                branch = Some(b.to_string());
            } else if line == "bare" || line.starts_with("prunable") {
                gone = true;
            }
        }
        if let (Some(path), false) = (path, gone) {
            out.push(Worktree {
                path,
                head,
                branch,
                linked: i > 0,
            });
        }
    }
    out
}

struct Candidate {
    reference: String,
    label: Option<String>,
    commit: String,
    time: u64,
    worktree: Option<PathBuf>,
}

fn detect(
    git: &Git,
    target: &str,
    patterns: &[String],
    max_age_days: u64,
) -> Result<(Vec<AgentBranch>, usize)> {
    let worktrees = worktrees(git);
    let use_worktrees = patterns.iter().any(|p| p == WORKTREES);
    let matches = |name: &str| {
        patterns
            .iter()
            .any(|p| p != WORKTREES && glob(p, name))
    };
    let checked_out: BTreeMap<&str, &Worktree> = worktrees
        .iter()
        .filter_map(|w| Some((w.branch.as_deref()?, w)))
        .collect();

    let refs = git.lines(&[
        "for-each-ref",
        "--format=%(refname)%09%(objectname)%09%(committerdate:unix)",
        "refs/heads",
        "refs/remotes",
    ])?;
    let refs: Vec<(&str, &str, u64)> = refs
        .iter()
        .filter_map(|line| {
            let mut f = line.split('\t');
            Some((f.next()?, f.next()?, f.next()?.parse().unwrap_or(0)))
        })
        .collect();
    let local: BTreeSet<&str> = refs
        .iter()
        .filter_map(|(r, _, _)| r.strip_prefix("refs/heads/"))
        .collect();

    let mut candidates = Vec::new();
    for &(refname, commit, time) in &refs {
        let Some(name) = refname.strip_prefix("refs/heads/") else {
            continue;
        };
        let in_linked = checked_out.get(name).is_some_and(|w| w.linked);
        if name == target || !(matches(name) || (use_worktrees && in_linked)) {
            continue;
        }
        candidates.push(Candidate {
            reference: name.to_string(),
            label: None,
            commit: commit.to_string(),
            time,
            worktree: checked_out.get(name).map(|w| w.path.clone()),
        });
    }
    // Remote branches nobody has locally (cloud agents push those); when
    // several remotes carry one, origin's copy wins.
    let mut remote: Vec<(&str, &str, &str, u64)> = refs
        .iter()
        .filter_map(|&(refname, commit, time)| {
            let (remote, name) = refname.strip_prefix("refs/remotes/")?.split_once('/')?;
            Some((remote, name, commit, time))
        })
        .filter(|&(_, name, _, _)| {
            name != "HEAD" && name != target && !local.contains(name) && matches(name)
        })
        .collect();
    remote.sort_by_key(|&(remote, name, _, _)| (name, remote != "origin"));
    remote.dedup_by_key(|&mut (_, name, _, _)| name);
    for (remote, name, commit, time) in remote {
        candidates.push(Candidate {
            reference: format!("{remote}/{name}"),
            label: None,
            commit: commit.to_string(),
            time,
            worktree: None,
        });
    }
    if use_worktrees {
        for w in worktrees.iter().filter(|w| w.linked && w.branch.is_none()) {
            let dir = w
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "worktree".into());
            let label: String = dir
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || "._-".contains(c) { c } else { '-' })
                .collect();
            candidates.push(Candidate {
                reference: w.head.clone(),
                label: Some(label),
                commit: w.head.clone(),
                time: git
                    .run(&["log", "-1", "--format=%ct", &w.head])
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(0),
                worktree: Some(w.path.clone()),
            });
        }
    }

    let target_ref = format!("refs/heads/{target}");
    let target_tree = git.run(&["rev-parse", &format!("{target_ref}^{{tree}}")])?;
    let in_memory = supports_merge_tree(git);
    let cutoff = (max_age_days > 0).then(|| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        now.saturating_sub(max_age_days * 86_400)
    });
    let mut commits = BTreeSet::new();
    let mut old = 0;
    let mut out = Vec::new();
    for c in candidates {
        if !commits.insert(c.commit.clone())
            || git.ok(&["merge-base", "--is-ancestor", &c.commit, &target_ref])
        {
            continue;
        }
        if cutoff.is_some_and(|cut| c.time < cut) {
            old += 1;
            continue;
        }
        // Squash-merged branches are not ancestors but add nothing.
        if in_memory && adds_nothing(git, &target_ref, &c.commit, &target_tree) {
            continue;
        }
        let uncommitted = c.worktree.as_deref().is_some_and(has_uncommitted);
        out.push(AgentBranch {
            reference: c.reference,
            label: c.label,
            commit: c.commit,
            worktree: c.worktree,
            uncommitted,
        });
    }
    Ok((out, old))
}

fn adds_nothing(git: &Git, target_ref: &str, commit: &str, target_tree: &str) -> bool {
    git.output(&["merge-tree", "--write-tree", "--no-messages", target_ref, commit])
        .is_ok_and(|o| {
            o.status.success()
                && String::from_utf8_lossy(&o.stdout).lines().next().map(str::trim)
                    == Some(target_tree)
        })
}

fn has_uncommitted(worktree: &Path) -> bool {
    Git::user(worktree)
        .run(&["status", "--porcelain"])
        .is_ok_and(|s| !s.is_empty())
}

/// Branches named on the command line; `*` patterns match local branches.
fn expand(git: &Git, given: &[String]) -> Result<Vec<AgentBranch>> {
    let worktrees = worktrees(git);
    let mut out = Vec::new();
    for arg in given {
        let parsed = LandInput::parse(arg);
        let references = if parsed.reference.contains(['*', '?', '[']) {
            let pattern = format!("refs/heads/{}", parsed.reference);
            let names = git.lines(&["for-each-ref", "--format=%(refname:short)", &pattern])?;
            if names.is_empty() {
                bail!("no branches match {}", parsed.reference);
            }
            names
        } else {
            vec![parsed.reference.clone()]
        };
        for reference in references {
            let commit = git
                .commit_of(&reference)
                .with_context(|| format!("unknown branch or commit: {reference}"))?;
            let worktree = worktrees
                .iter()
                .find(|w| w.branch.as_deref() == Some(reference.as_str()))
                .map(|w| w.path.clone());
            out.push(AgentBranch {
                uncommitted: worktree.as_deref().is_some_and(has_uncommitted),
                reference,
                label: parsed.label.clone(),
                commit,
                worktree,
            });
        }
    }
    Ok(out)
}

/// The test command the project at `rev` declares, and the file that says so.
pub fn test_command(git: &Git, rev: &str) -> Option<(String, String)> {
    let names: BTreeSet<String> = git
        .lines(&["ls-tree", "--name-only", rev])
        .ok()?
        .into_iter()
        .collect();
    checks::detect_check(&names, |file| git.show(rev, file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn commit_file(d: &Path, path: &str, body: &str, message: &str) {
        let p = d.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", message]);
    }

    /// A repository whose path has a space in it, like `C:\Users\Jo Smith`.
    fn repo() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path().join("my repo");
        std::fs::create_dir_all(&d).unwrap();
        git(&d, &["init", "-q", "-b", "main"]);
        git(&d, &["config", "user.email", "test@pytxo.invalid"]);
        git(&d, &["config", "user.name", "Test"]);
        git(&d, &["config", "core.autocrlf", "false"]);
        commit_file(
            &d,
            "package.json",
            r#"{"scripts":{"test":"node test.js"}}"#,
            "base",
        );
        (dir, d)
    }

    fn branch(d: &Path, name: &str, file: &str) {
        git(d, &["checkout", "-q", "-b", name, "main"]);
        commit_file(d, file, name, name);
        git(d, &["checkout", "-q", "main"]);
    }

    fn names(plan: &Plan) -> Vec<&str> {
        plan.branches.iter().map(AgentBranch::name).collect()
    }

    #[test]
    fn globs_match_across_slashes() {
        assert!(glob("codex/*", "codex/fix-tax"));
        assert!(glob("codex/*", "codex/a/b"));
        assert!(!glob("codex/*", "codex"));
        assert!(!glob("codex/*", "my-codex/x"));
        assert!(glob("*/wip", "matt/wip"));
        assert!(glob("team-?/*", "team-a/x"));
        assert!(glob("main", "main"));
        assert!(!glob("main", "mainline"));
    }

    #[test]
    fn finds_agent_branches_by_prefix_and_skips_the_rest() {
        let (_dir, d) = repo();
        branch(&d, "claude/rename", "a.js");
        branch(&d, "codex/discount", "b.js");
        branch(&d, "copilot/fix-tax", "c.js");
        branch(&d, "cursor/docs", "d.md");
        branch(&d, "feature/mine", "e.js");
        // Already merged into main: nothing to combine.
        branch(&d, "devin/merged", "f.js");
        git(&d, &["merge", "-q", "--no-ff", "-m", "merge", "devin/merged"]);
        // Same commit under two names counts once.
        git(&d, &["branch", "agent/copy", "claude/rename"]);

        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(plan.target, "main");
        assert_eq!(
            names(&plan),
            ["agent/copy", "codex/discount", "copilot/fix-tax", "cursor/docs"]
        );
        assert!(plan.detected);
        assert_eq!(plan.checks, ["npm test"]);
        assert_eq!(plan.check_source.as_deref(), Some("package.json"));
    }

    #[test]
    fn squash_merged_branches_add_nothing_and_are_skipped() {
        let (_dir, d) = repo();
        branch(&d, "codex/squashed", "a.js");
        git(&d, &["merge", "-q", "--squash", "codex/squashed"]);
        git(&d, &["commit", "-q", "-m", "squash"]);
        branch(&d, "codex/open", "b.js");
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["codex/open"]);
    }

    #[test]
    fn remote_branches_count_when_there_is_no_local_one() {
        let (_dir, d) = repo();
        branch(&d, "codex/cloud", "a.js");
        branch(&d, "copilot/both", "b.js");
        let sha = git(&d, &["rev-parse", "codex/cloud"]);
        git(&d, &["update-ref", "refs/remotes/origin/codex/cloud", &sha]);
        let both = git(&d, &["rev-parse", "copilot/both"]);
        git(&d, &["update-ref", "refs/remotes/origin/copilot/both", &both]);
        git(&d, &["branch", "-D", "-q", "codex/cloud"]);
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["copilot/both", "origin/codex/cloud"]);
    }

    #[test]
    fn linked_worktrees_count_and_the_target_is_the_default_branch() {
        let (dir, d) = repo();
        // Claude Code Desktop-style worktree with a custom branch name.
        let wt = dir.path().join("work trees").join("eager-curie");
        git(
            &d,
            &["worktree", "add", "-q", "-b", "matt/eager-curie", &wt.to_string_lossy()],
        );
        commit_file(&wt, "src/feature.js", "x", "feature");
        // An agent that has not committed yet.
        std::fs::write(wt.join("notes.txt"), "draft").unwrap();
        branch(&d, "codex/tax", "tax.js");

        // Run from inside the agent's worktree, on the agent's branch.
        let plan = plan(&wt, &[], None, &[]).unwrap();
        assert_eq!(plan.target, "main");
        assert_eq!(names(&plan), ["codex/tax", "matt/eager-curie"]);
        let mine = &plan.branches[1];
        assert!(mine.uncommitted);
        assert_eq!(
            mine.worktree.as_deref().map(|p| p.canonicalize().unwrap()),
            Some(wt.canonicalize().unwrap())
        );
        assert!(!plan.branches[0].uncommitted);

        // Without the `worktrees` pattern only prefixes count.
        git(&d, &["config", "pytxo.branches", "codex/"]);
        let plan = super::plan(&wt, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["codex/tax"]);
    }

    #[test]
    fn configured_patterns_days_and_checks_win() {
        let (_dir, d) = repo();
        branch(&d, "team/a", "a.js");
        branch(&d, "codex/b", "b.js");
        git(&d, &["config", "pytxo.branches", "team/*"]);
        git(&d, &["config", "pytxo.check", "node test.js --fast"]);
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["team/a"]);
        assert_eq!(plan.checks, ["node test.js --fast"]);
        assert_eq!(plan.check_source.as_deref(), Some("git config pytxo.check"));

        // --check beats configuration.
        let plan = super::plan(&d, &[], None, &["make test".into()]).unwrap();
        assert_eq!(plan.checks, ["make test"]);

        // A branch whose last commit is old is left out and counted.
        git(&d, &["checkout", "-q", "-b", "team/old", "main"]);
        std::fs::write(d.join("old.js"), "old").unwrap();
        git(&d, &["add", "-A"]);
        let out = Command::new("git")
            .args(["commit", "-q", "-m", "old"])
            .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
            .current_dir(&d)
            .output()
            .unwrap();
        assert!(out.status.success());
        git(&d, &["checkout", "-q", "main"]);
        let plan = super::plan(&d, &[], None, &[]).unwrap();
        assert_eq!((names(&plan), plan.old), (vec!["team/a"], 1));
        git(&d, &["config", "pytxo.days", "0"]);
        let plan = super::plan(&d, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["team/a", "team/old"]);
    }

    #[test]
    fn given_branches_are_used_as_they_are() {
        let (_dir, d) = repo();
        branch(&d, "feature/x", "x.js");
        branch(&d, "agent/one", "1.js");
        branch(&d, "agent/two", "2.js");
        let plan = plan(&d, &["mine:feature/x".into(), "agent/*".into()], None, &[]).unwrap();
        assert!(!plan.detected);
        assert_eq!(names(&plan), ["mine", "agent/one", "agent/two"]);
        assert!(super::plan(&d, &["nope/*".into()], None, &[]).is_err());
        assert!(super::plan(&d, &["missing".into()], None, &[]).is_err());
    }

    #[test]
    fn no_agent_branches_and_no_tests() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        git(d, &["init", "-q", "-b", "trunk"]);
        git(d, &["config", "user.email", "test@pytxo.invalid"]);
        git(d, &["config", "user.name", "Test"]);
        commit_file(d, "README.md", "hi", "base");
        let plan = plan(d, &[], None, &[]).unwrap();
        assert_eq!(plan.target, "trunk");
        assert!(plan.branches.is_empty());
        assert!(plan.checks.is_empty() && plan.check_source.is_none());
    }

    #[test]
    fn origin_head_names_the_target() {
        let (_dir, d) = repo();
        git(&d, &["branch", "develop"]);
        let sha = git(&d, &["rev-parse", "develop"]);
        git(&d, &["update-ref", "refs/remotes/origin/develop", &sha]);
        git(
            &d,
            &["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/develop"],
        );
        assert_eq!(target_branch(&Git::user(&d), None).unwrap(), "develop");
        assert_eq!(target_branch(&Git::user(&d), Some("main")).unwrap(), "main");
    }
}
