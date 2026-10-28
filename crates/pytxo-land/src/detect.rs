//! What `pytxo check` combines when nobody says: the target branch, the agent
//! branches and the project's test command.
//!
//! Detection reads refs, worktrees and trees, and writes no ref. It never
//! checks out, fetches or runs anything.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};

use crate::{checks, is_scratch_worktree, native_path, supports_merge_tree, Git, LandInput};

/// Stands for every branch checked out in a linked worktree.
pub const WORKTREES: &str = "worktrees";

/// Branch prefixes coding agents use for the branches they create, `agent/`
/// (Pytxo's own convention), and linked worktrees. `*` matches across `/`.
pub const DEFAULT_PATTERNS: [&str; 8] = [
    "claude/*",
    "codex/*",
    "copilot/*",
    "cursor/*",
    "devin/*",
    "jules/*",
    "agent/*",
    WORKTREES,
];

/// Detected branches without a commit in this many days are left out.
pub const DEFAULT_MAX_AGE_DAYS: u64 = 14;

/// At most this many detected branches at once, the newest commits, as the
/// Action caps pull requests. Named branches are never capped.
pub const MAX_BRANCHES: usize = 10;

/// What `plan` says outside a repository; the CLI says where to go from there.
pub const NOT_A_REPOSITORY: &str = "not a git repository";

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

    /// As the verdict names it: a detached worktree's label, else the branch.
    pub fn name(&self) -> &str {
        let commit =
            self.reference.len() >= 7 && self.reference.chars().all(|c| c.is_ascii_hexdigit());
        match &self.label {
            Some(label) if commit => label,
            _ => &self.reference,
        }
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
    /// Matching branches left out for sharing no history with the target.
    pub unrelated: usize,
    /// Branches past `MAX_BRANCHES` left out, the oldest commits first.
    pub left_out: usize,
    /// Worktrees whose only work is uncommitted, by name: nothing to check.
    pub uncommitted_only: Vec<String>,
    /// Commits `origin/<target>` has that the target does not.
    pub behind: usize,
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
    let root = toplevel(cwd)?;
    let git = Git::user(&root);
    // Named branches first: the target depends on them.
    let named = if given.is_empty() {
        None
    } else {
        Some(expand(&git, given)?)
    };
    let target = target_branch(&git, into, named.as_deref())?;
    // Patterns and days only matter to detection, so named branches read neither.
    let (patterns, max_age_days, found) = match named {
        Some(branches) => (
            Vec::new(),
            DEFAULT_MAX_AGE_DAYS,
            Detected {
                branches,
                ..Detected::default()
            },
        ),
        None => {
            let patterns = patterns(&git);
            let days = setting(&git, "PYTXO_DAYS", "pytxo.days")
                .and_then(|d| d.parse().ok())
                .unwrap_or(DEFAULT_MAX_AGE_DAYS);
            let found = detect(&git, &target, &patterns, days)?;
            (patterns, days, found)
        }
    };
    let (checks, check_source) = if !checks.is_empty() {
        (checks.to_vec(), Some("--check".to_string()))
    } else {
        let configured = settings(&git, "PYTXO_CHECK", "pytxo.check");
        if !configured.is_empty() {
            let from = if env("PYTXO_CHECK").is_some() {
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
    // Agents that push to GitHub build on origin's copy; the check uses this one.
    let behind = git
        .run(&[
            "rev-list",
            "--count",
            &format!("refs/heads/{target}..refs/remotes/origin/{target}"),
        ])
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    Ok(Plan {
        root,
        target,
        branches: found.branches,
        detected: given.is_empty(),
        patterns,
        old: found.old,
        unrelated: found.unrelated,
        left_out: found.left_out,
        uncommitted_only: found.uncommitted_only,
        behind,
        max_age_days,
        checks,
        check_source,
    })
}

/// The root of the worktree `cwd` is in.
fn toplevel(cwd: &Path) -> Result<PathBuf> {
    if !cwd.is_dir() {
        bail!(NOT_A_REPOSITORY);
    }
    let out = Git::user(cwd)
        .output(&["rev-parse", "--show-toplevel"])
        .map_err(|_| anyhow!("git is not on PATH: install it from https://git-scm.com"))?;
    if !out.status.success() {
        let why = String::from_utf8_lossy(&out.stderr);
        if why.contains("not a git repository") {
            bail!(NOT_A_REPOSITORY);
        }
        bail!("git rev-parse --show-toplevel failed: {}", why.trim());
    }
    Ok(native_path(String::from_utf8_lossy(&out.stdout).trim()))
}

/// The first of: `--into`; `PYTXO_INTO`, else `git config pytxo.into`; with
/// branches `named` on the command line, the checked-out branch unless it is
/// one of them; origin/HEAD, main, master or trunk; the checked-out branch.
/// Without named branches the default branch comes first: a hook runs inside
/// an agent's worktree, where the checked-out branch is the agent's own.
pub fn target_branch(
    git: &Git,
    into: Option<&str>,
    named: Option<&[AgentBranch]>,
) -> Result<String> {
    let exists = |b: &str| {
        git.ok(&[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{b}"),
        ])
    };
    // A branch someone named must exist, or detection fails on it with a git error.
    let named_by = |t: &str, from: &str| {
        if exists(t) {
            Ok(t.to_string())
        } else {
            bail!("target branch does not exist: {t} (from {from})")
        }
    };
    if let Some(t) = into {
        return named_by(t, "--into");
    }
    if let Some(t) = setting(git, "PYTXO_INTO", "pytxo.into") {
        return named_by(
            &t,
            if env("PYTXO_INTO").is_some() {
                "PYTXO_INTO"
            } else {
                "git config pytxo.into"
            },
        );
    }
    let current = git
        .run(&["symbolic-ref", "--quiet", "--short", "HEAD"])
        .ok();
    if let (Some(current), Some(named)) = (&current, named) {
        // `HEAD`, `heads/x` and `x` all name the same branch.
        let mut args = vec!["rev-parse", "--symbolic-full-name"];
        args.extend(named.iter().map(|b| b.reference.as_str()));
        let inputs = git.lines(&args).unwrap_or_default();
        if !inputs.contains(&format!("refs/heads/{current}")) {
            return Ok(current.clone());
        }
    }
    let remote_head = git
        .run(&[
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ])
        .ok()
        .and_then(|h| h.strip_prefix("origin/").map(str::to_string));
    if let Some(b) = remote_head
        .into_iter()
        .chain(["main", "master", "trunk"].map(String::from))
        .find(|b| exists(b))
    {
        return Ok(b);
    }
    current.context("no target branch: pass --into <branch>, or set git config pytxo.into <branch>")
}

/// `PYTXO_BRANCHES`, else `git config pytxo.branches`, else the defaults.
/// Values split on spaces and commas; a trailing `/` means `/*`.
pub fn patterns(git: &Git) -> Vec<String> {
    let configured = settings(git, "PYTXO_BRANCHES", "pytxo.branches");
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

/// A non-blank environment variable, trimmed.
fn env(name: &str) -> Option<String> {
    let v = std::env::var(name).ok()?;
    (!v.trim().is_empty()).then(|| v.trim().to_string())
}

/// An environment variable, else a git config key's last value, as
/// `git config --get` reads it: a repository's beats a global one.
fn setting(git: &Git, var: &str, key: &str) -> Option<String> {
    env(var).or_else(|| {
        git.run(&["config", "--get", key])
            .ok()
            .filter(|v| !v.is_empty())
    })
}

/// An environment variable, else every value of a git config key from the
/// most specific scope that sets it: a repository's pytxo.check replaces a
/// global one instead of running beside it.
fn settings(git: &Git, var: &str, key: &str) -> Vec<String> {
    if let Some(v) = env(var) {
        return vec![v];
    }
    // Git before 2.26 has no --show-scope; there the last file that sets the
    // key stands in for the last scope.
    for show in ["--show-scope", "--show-origin"] {
        let Ok(out) = git.output(&["config", show, "--get-all", key]) else {
            break;
        };
        match out.status.code() {
            Some(0) => return most_specific(&String::from_utf8_lossy(&out.stdout)),
            // Set nowhere.
            Some(1) => break,
            _ => {}
        }
    }
    Vec::new()
}

/// The values of the last scope, or file, out of `git config --get-all`
/// with `--show-scope` (`local\tnpm test`) or `--show-origin`, which list
/// the system's first.
fn most_specific(text: &str) -> Vec<String> {
    let lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let scoped: Vec<(&str, &str)> = lines.filter_map(|l| l.split_once('\t')).collect();
    let last = scoped.last().map(|&(scope, _)| scope);
    scoped
        .into_iter()
        .filter(|&(scope, _)| Some(scope) == last)
        .map(|(_, v)| v.to_string())
        .collect()
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

/// `git worktree list --porcelain`, minus bare and missing (prunable) entries
/// and Pytxo's own scratch worktrees.
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
        let scratch = branch.is_none() && path.as_deref().is_some_and(is_scratch_worktree);
        if let (Some(path), false) = (path, gone || scratch) {
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

/// What detection found, and what it left out.
#[derive(Default)]
struct Detected {
    branches: Vec<AgentBranch>,
    old: usize,
    unrelated: usize,
    left_out: usize,
    uncommitted_only: Vec<String>,
}

fn detect(git: &Git, target: &str, patterns: &[String], max_age_days: u64) -> Result<Detected> {
    let worktrees = worktrees(git);
    let use_worktrees = patterns.iter().any(|p| p == WORKTREES);
    let matches = |name: &str| patterns.iter().any(|p| p != WORKTREES && glob(p, name));
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
        let detached: Vec<&Worktree> = worktrees
            .iter()
            .filter(|w| w.linked && w.branch.is_none())
            .collect();
        let dir = |p: &Path| p.file_name().map(|n| clean(&n.to_string_lossy()));
        let dirs: Vec<String> = detached
            .iter()
            .map(|w| dir(&w.path).unwrap_or_else(|| "worktree".into()))
            .collect();
        let mut taken: BTreeSet<String> = candidates.iter().map(|c| c.reference.clone()).collect();
        for (w, name) in detached.iter().zip(&dirs) {
            // Codex keeps each at ~/.codex/worktrees/<id>/<repo>: one name for
            // all of a repository's, so the parent tells them apart.
            let shared = dirs.iter().filter(|d| *d == name).count() > 1;
            let label = match w.path.parent().and_then(dir) {
                Some(parent) if shared => format!("{parent}/{name}"),
                _ => name.clone(),
            };
            let mut unique = label.clone();
            for n in 2.. {
                if taken.insert(unique.clone()) {
                    break;
                }
                unique = format!("{label}-{n}");
            }
            candidates.push(Candidate {
                reference: w.head.clone(),
                label: Some(unique),
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

    // A branch merged on GitHub is in origin's copy of the target before the
    // local one pulls it, so both count when they differ.
    let tip = |r: &str| refs.iter().find(|f| f.0 == r).map(|f| f.1);
    let mut targets = vec![format!("refs/heads/{target}")];
    // A checked-out branch with no commit yet, in a new repository.
    let Some(local) = tip(&targets[0]) else {
        return Ok(Detected::default());
    };
    let upstream = format!("refs/remotes/origin/{target}");
    if tip(&upstream).is_some_and(|c| c != local) {
        targets.push(upstream);
    }
    let trees = targets
        .iter()
        .map(|t| git.run(&["rev-parse", &format!("{t}^{{tree}}")]))
        .collect::<Result<Vec<_>>>()?;
    let in_memory = supports_merge_tree(git);
    let cutoff = (max_age_days > 0).then(|| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        now.saturating_sub(max_age_days * 86_400)
    });
    let old = |c: &Candidate| cutoff.is_some_and(|cut| c.time < cut);
    let (mut seen, mut merged) = (BTreeSet::new(), BTreeSet::new());
    // An old worktree on a commit a target has can only hold uncommitted
    // work, so it is not old but merged. One walk, for all of them, lists
    // the commits the targets lack.
    let old_worktrees: BTreeSet<&str> = candidates
        .iter()
        .filter(|c| c.worktree.is_some() && old(c))
        .map(|c| c.commit.as_str())
        .collect();
    if !old_worktrees.is_empty() {
        let mut args = vec!["rev-list"];
        args.extend(&old_worktrees);
        args.push("--not");
        args.extend(targets.iter().map(String::as_str));
        if let Ok(lacked) = git.lines(&args) {
            let lacked: BTreeSet<String> = lacked.into_iter().collect();
            for commit in old_worktrees.into_iter().filter(|c| !lacked.contains(*c)) {
                seen.insert(commit.to_string());
                merged.insert(commit.to_string());
            }
        }
    }
    let mut found = Detected::default();
    let mut open = Vec::new();
    for c in candidates {
        // The age check costs no git call, so it goes first, for each name.
        if old(&c) && !(c.worktree.is_some() && merged.contains(&c.commit)) {
            found.old += 1;
            continue;
        }
        if seen.insert(c.commit.clone()) {
            // Nothing to merge onto: a `gh-pages` worktree, an orphan branch.
            let Ok(base) = git.run(&["merge-base", &targets[0], &c.commit]) else {
                found.unrelated += 1;
                continue;
            };
            // Merged, or squash-merged: not an ancestor, but adds nothing.
            let is_merged = base == c.commit
                || targets[1..]
                    .iter()
                    .any(|t| git.ok(&["merge-base", "--is-ancestor", &c.commit, t]))
                || targets.iter().zip(&trees).any(|(t, tree)| {
                    match in_memory.then(|| merged_tree(git, t, &c.commit)).flatten() {
                        Some(merged) => merged == *tree,
                        // A conflict, once the target edits the same lines
                        // again, or an old Git: the patches say.
                        None => in_by_patch_id(git, t, &c.commit),
                    }
                });
            if !is_merged {
                let uncommitted = c.worktree.as_deref().is_some_and(has_uncommitted);
                open.push((
                    c.time,
                    AgentBranch {
                        reference: c.reference,
                        label: c.label,
                        commit: c.commit,
                        worktree: c.worktree,
                        uncommitted,
                    },
                ));
                continue;
            }
            merged.insert(c.commit.clone());
        } else if !merged.contains(&c.commit) {
            // A second name for a commit counts once.
            continue;
        }
        // Nothing to check, unless its worktree holds work nobody committed:
        // an agent at its turn's end, or a Codex worktree.
        if c.worktree.as_deref().is_some_and(has_uncommitted) {
            found.uncommitted_only.push(c.label.unwrap_or(c.reference));
        }
    }
    // The newest commits, in merge order.
    let mut newest: Vec<usize> = (0..open.len()).collect();
    newest.sort_by_key(|&i| std::cmp::Reverse(open[i].0));
    newest.truncate(MAX_BRANCHES);
    found.left_out = open.len() - newest.len();
    found.branches = open
        .into_iter()
        .enumerate()
        .filter(|(i, _)| newest.contains(i))
        .map(|(_, (_, b))| b)
        .collect();
    Ok(found)
}

/// `text` with every character but letters, digits and `._-` made a `-`.
fn clean(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._-".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Every change on `commit` is in `target` by patch id: rebased or
/// cherry-picked commit by commit, or squashed into one. Any Git has it.
fn in_by_patch_id(git: &Git, target: &str, commit: &str) -> bool {
    let all_in = |head: &str| {
        git.lines(&["cherry", target, head]).map(|lines| {
            (
                !lines.is_empty() && lines.iter().all(|l| l.starts_with('-')),
                lines.len(),
            )
        })
    };
    match all_in(commit) {
        Ok((true, _)) => true,
        // One commit is its own squash; more get one, on the merge base.
        Ok((false, n)) if n > 1 => git
            .run(&["merge-base", target, commit])
            .and_then(|base| squash_commit(git, &base, commit))
            .and_then(|squash| all_in(&squash))
            .is_ok_and(|(all, _)| all),
        _ => false,
    }
}

/// `commit`'s tree as one commit on `base`. Its names and dates are fixed,
/// so the same squash is the same object: written once, not on every run.
fn squash_commit(git: &Git, base: &str, commit: &str) -> Result<String> {
    let (name, email, date) = ("Pytxo Land", "land@pytxo.invalid", "2000-01-01T00:00:00Z");
    let tree = format!("{commit}^{{tree}}");
    let out = pytxo_core::background_command("git")
        .args(["-c", "commit.gpgsign=false", "commit-tree"])
        .args([tree.as_str(), "-p", base, "-m", "squash"])
        .current_dir(git.cwd())
        .envs([
            ("GIT_AUTHOR_NAME", name),
            ("GIT_AUTHOR_EMAIL", email),
            ("GIT_AUTHOR_DATE", date),
            ("GIT_COMMITTER_NAME", name),
            ("GIT_COMMITTER_EMAIL", email),
            ("GIT_COMMITTER_DATE", date),
        ])
        .output()?;
    if !out.status.success() {
        bail!(
            "git commit-tree failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The tree of `commit` merged into `target_ref` in memory, `None` on a conflict.
fn merged_tree(git: &Git, target_ref: &str, commit: &str) -> Option<String> {
    let out = git
        .output(&[
            "merge-tree",
            "--write-tree",
            "--no-messages",
            target_ref,
            commit,
        ])
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let tree = stdout.lines().next()?.trim();
    out.status.success().then(|| tree.to_string())
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
        git(
            &d,
            &["merge", "-q", "--no-ff", "-m", "merge", "devin/merged"],
        );
        // Same commit under two names counts once.
        git(&d, &["branch", "agent/copy", "claude/rename"]);

        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(plan.target, "main");
        assert_eq!(
            names(&plan),
            [
                "agent/copy",
                "codex/discount",
                "copilot/fix-tax",
                "cursor/docs"
            ]
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
        git(
            &d,
            &["update-ref", "refs/remotes/origin/copilot/both", &both],
        );
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
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "matt/eager-curie",
                &wt.to_string_lossy(),
            ],
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
    fn pytxo_scratch_worktrees_are_not_agent_branches() {
        // A killed run leaves its scratch worktrees registered, each with a
        // commit nobody merged; a concurrent run's look the same.
        let (dir, d) = repo();
        branch(&d, "claude/a", "a.js");
        let detached = |name: &str| {
            let wt = dir.path().join(name);
            git(
                &d,
                &["worktree", "add", "-q", "--detach", &wt.to_string_lossy()],
            );
            commit_file(&wt, &format!("{name}.txt"), name, "pytxo land: merge");
        };
        detached("pytxo-land-20261028t120005z_ab12cd34");
        detached("pytxo-land-20261028t120005z_ab12cd34-solo");
        // The user's own detached worktree with a similar name still counts.
        detached("pytxo-land-fix");
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["claude/a", "pytxo-land-fix"]);
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
        // Named as the verdict names them: the branch, not the label.
        assert_eq!(names(&plan), ["feature/x", "agent/one", "agent/two"]);
        assert_eq!(plan.branches[0].label.as_deref(), Some("mine"));
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
            &[
                "symbolic-ref",
                "refs/remotes/origin/HEAD",
                "refs/remotes/origin/develop",
            ],
        );
        assert_eq!(
            target_branch(&Git::user(&d), None, None).unwrap(),
            "develop"
        );
        assert_eq!(
            target_branch(&Git::user(&d), Some("main"), None).unwrap(),
            "main"
        );
    }

    /// Every rung of the target order, from the main checkout and from linked
    /// worktrees. `PYTXO_INTO` is tested from the binary: it sets no env here.
    #[test]
    fn the_target_order() {
        let (dir, d) = repo();
        branch(&d, "codex/a", "a.js");
        branch(&d, "codex/b", "b.js");
        git(&d, &["branch", "develop"]);
        let target = |cwd: &Path, given: &[&str], into: Option<&str>| {
            let given: Vec<String> = given.iter().map(|s| s.to_string()).collect();
            plan(cwd, &given, into, &[]).unwrap().target
        };
        // Without names (bare pytxo, hooks): the default branch.
        git(&d, &["checkout", "-q", "develop"]);
        assert_eq!(target(&d, &[], None), "main");
        // Named branches land on the checked-out branch...
        assert_eq!(target(&d, &["codex/a", "codex/b"], None), "develop");
        // ...unless it is one of them, however it is spelled, or HEAD is detached.
        git(&d, &["checkout", "-q", "codex/a"]);
        assert_eq!(target(&d, &["codex/a", "codex/b"], None), "main");
        assert_eq!(target(&d, &["HEAD", "codex/b"], None), "main");
        git(&d, &["checkout", "-q", "--detach", "develop"]);
        assert_eq!(target(&d, &["codex/a"], None), "main");
        // A linked worktree's own checked-out branch.
        let wt = dir.path().join("feature wt");
        let wt_arg = wt.to_string_lossy().into_owned();
        git(
            &d,
            &["worktree", "add", "-q", "-b", "feature", &wt_arg, "main"],
        );
        assert_eq!(target(&wt, &["codex/a"], None), "feature");
        assert_eq!(target(&wt, &[], None), "main");
        let agent = dir.path().join("agent wt");
        git(
            &d,
            &["worktree", "add", "-q", &agent.to_string_lossy(), "codex/b"],
        );
        assert_eq!(target(&agent, &["codex/a", "codex/b"], None), "main");
        // git config pytxo.into beats both; its last value counts.
        git(&d, &["config", "--add", "pytxo.into", "main"]);
        git(&d, &["config", "--add", "pytxo.into", "develop"]);
        assert_eq!(target(&wt, &["codex/a"], None), "develop");
        assert_eq!(target(&agent, &[], None), "develop");
        // --into beats everything.
        assert_eq!(target(&wt, &["codex/a"], Some("main")), "main");
        // A target someone named must exist, and the error says who named it.
        let error = |into: Option<&str>| format!("{:#}", plan(&wt, &[], into, &[]).unwrap_err());
        assert_eq!(
            error(Some("nope")),
            "target branch does not exist: nope (from --into)"
        );
        git(&d, &["config", "--add", "pytxo.into", "gone"]);
        assert_eq!(
            error(None),
            "target branch does not exist: gone (from git config pytxo.into)"
        );

        // No default branch: the checked-out one.
        let lone = tempfile::tempdir().unwrap();
        let l = lone.path();
        git(l, &["init", "-q", "-b", "work"]);
        git(l, &["config", "user.email", "test@pytxo.invalid"]);
        git(l, &["config", "user.name", "Test"]);
        commit_file(l, "README.md", "hi", "base");
        assert_eq!(target(l, &[], None), "work");
    }

    #[test]
    fn branches_merged_upstream_count_as_merged_while_main_lags_behind() {
        let (_dir, d) = repo();
        branch(&d, "codex/merged", "a.js");
        branch(&d, "codex/squashed", "b.js");
        branch(&d, "codex/open", "c.js");
        // GitHub merged one and squash-merged the other; main has not pulled.
        git(&d, &["checkout", "-q", "-b", "upstream", "main"]);
        git(
            &d,
            &["merge", "-q", "--no-ff", "-m", "merge", "codex/merged"],
        );
        git(&d, &["merge", "-q", "--squash", "codex/squashed"]);
        git(&d, &["commit", "-q", "-m", "squash"]);
        git(&d, &["update-ref", "refs/remotes/origin/main", "upstream"]);
        git(&d, &["checkout", "-q", "main"]);
        git(&d, &["branch", "-q", "-D", "upstream"]);
        // The age check runs before any merge check, so an old branch is
        // counted as old, merged or not.
        git(&d, &["checkout", "-q", "-b", "codex/old", "main"]);
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
        git(
            &d,
            &["merge", "-q", "--no-ff", "-m", "merge old", "codex/old"],
        );

        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!((names(&plan), plan.old), (vec!["codex/open"], 1));
    }

    /// Once the target edits the same lines again, a 3-way merge conflicts;
    /// the patches still say the work is in.
    #[test]
    fn squash_and_rebase_merged_branches_stay_skipped_after_the_target_edits_them() {
        let (_dir, d) = repo();
        commit_file(&d, "a.js", "a1\n", "a");
        let two_commits = |name: &str, file: &str| {
            git(&d, &["checkout", "-q", "-b", name, "main"]);
            commit_file(&d, file, "2\n", "one");
            commit_file(&d, file, "3\n", "two");
            git(&d, &["checkout", "-q", "main"]);
        };
        two_commits("codex/squashed", "a.js");
        two_commits("codex/rebased", "r.js");
        branch(&d, "codex/one", "o.js");
        git(&d, &["merge", "-q", "--squash", "codex/squashed"]);
        git(&d, &["commit", "-q", "-m", "squash"]);
        git(&d, &["cherry-pick", "codex/rebased~1", "codex/rebased"]);
        git(&d, &["merge", "-q", "--squash", "codex/one"]);
        git(&d, &["commit", "-q", "-m", "squash one"]);
        commit_file(&d, "a.js", "a4\n", "edit a");
        commit_file(&d, "r.js", "r4\n", "edit r");
        commit_file(&d, "o.js", "o4\n", "edit o");
        branch(&d, "codex/open", "b.js");
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["codex/open"]);
        // The squash is the same commit every time: a second look, a second
        // later, writes no object.
        let objects = || git(&d, &["count-objects"]);
        let before = objects();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        super::plan(&d, &[], None, &[]).unwrap();
        assert_eq!(objects(), before);
    }

    #[test]
    fn worktrees_with_only_uncommitted_work_are_named() {
        let (dir, d) = repo();
        let add = |args: &[&str]| {
            let mut all = vec!["worktree", "add", "-q"];
            all.extend(args);
            git(&d, &all);
        };
        // An agent at its turn's end: a worktree on its branch, nothing committed.
        let one = dir.path().join("one");
        add(&["-b", "claude/one", &one.to_string_lossy()]);
        std::fs::write(one.join("draft.js"), "x").unwrap();
        // A Codex worktree: detached at main, nothing committed.
        let codex = dir.path().join("codex").join("05b5").join("my repo");
        add(&["--detach", &codex.to_string_lossy()]);
        std::fs::write(codex.join("package.json"), "{}").unwrap();
        // Merged and clean: nothing to say.
        add(&[
            "-b",
            "claude/done",
            &dir.path().join("done").to_string_lossy(),
        ]);
        branch(&d, "codex/tax", "tax.js");

        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["codex/tax"]);
        assert_eq!(plan.uncommitted_only, ["claude/one", "my-repo"]);
    }

    /// A target whose last commit is old: worktrees on it, or on origin's
    /// copy of it, have no commit of their own to be old, so their
    /// uncommitted work is still named. Old branches count by name, two names
    /// for one commit as two.
    #[test]
    fn worktrees_on_an_old_target_are_named_and_old_branches_count_by_name() {
        let (dir, d) = repo();
        let old = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
                .current_dir(&d)
                .output()
                .unwrap();
            assert!(out.status.success());
        };
        old(&["commit", "-q", "--amend", "--no-edit"]);
        git(&d, &["checkout", "-q", "-b", "codex/old", "main"]);
        std::fs::write(d.join("old.js"), "old").unwrap();
        git(&d, &["add", "-A"]);
        old(&["commit", "-q", "-m", "old"]);
        git(&d, &["checkout", "-q", "main"]);
        git(&d, &["branch", "agent/old", "codex/old"]);
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert!(plan.branches.is_empty());
        assert_eq!(plan.old, 2);

        // Two agents at their turn's end on main, and one with nothing to say.
        for name in ["claude/one", "claude/two", "claude/done"] {
            let wt = dir.path().join(name.replace('/', "-"));
            git(
                &d,
                &["worktree", "add", "-q", "-b", name, &wt.to_string_lossy()],
            );
            if name != "claude/done" {
                std::fs::write(wt.join("draft.txt"), "draft").unwrap();
            }
        }
        let plan = super::plan(&d, &[], None, &[]).unwrap();
        assert!(plan.branches.is_empty());
        assert_eq!(plan.uncommitted_only, ["claude/one", "claude/two"]);
        assert_eq!(plan.old, 2);

        // And one on origin's copy of main, which main has not pulled yet.
        git(&d, &["checkout", "-q", "-b", "upstream", "main"]);
        std::fs::write(d.join("up.js"), "up").unwrap();
        git(&d, &["add", "-A"]);
        old(&["commit", "-q", "-m", "up"]);
        git(&d, &["update-ref", "refs/remotes/origin/main", "upstream"]);
        git(&d, &["checkout", "-q", "main"]);
        git(&d, &["branch", "-q", "-D", "upstream"]);
        git(&d, &["branch", "claude/ahead", "origin/main"]);
        let wt = dir.path().join("claude-ahead");
        let path = wt.to_string_lossy();
        git(&d, &["worktree", "add", "-q", &path, "claude/ahead"]);
        std::fs::write(wt.join("draft.txt"), "draft").unwrap();
        let plan = super::plan(&d, &[], None, &[]).unwrap();
        let named = ["claude/ahead", "claude/one", "claude/two"];
        assert_eq!(plan.uncommitted_only, named);
        assert_eq!(plan.old, 2);
    }

    #[test]
    fn branches_sharing_no_history_are_left_out_and_counted() {
        let (dir, d) = repo();
        // `git worktree add ../site gh-pages`, on an orphan branch.
        git(&d, &["checkout", "-q", "--orphan", "claude/pages"]);
        git(&d, &["rm", "-rfq", "."]);
        commit_file(&d, "index.html", "hi", "pages");
        git(&d, &["checkout", "-q", "main"]);
        let site = dir.path().join("site");
        git(
            &d,
            &[
                "worktree",
                "add",
                "-q",
                &site.to_string_lossy(),
                "claude/pages",
            ],
        );
        branch(&d, "codex/a", "a.js");
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!((names(&plan), plan.unrelated), (vec!["codex/a"], 1));
    }

    #[test]
    fn detached_worktrees_get_labels_that_tell_them_apart() {
        let (dir, d) = repo();
        let detached = |path: &[&str]| {
            let wt = path.iter().fold(dir.path().to_path_buf(), |p, c| p.join(c));
            git(
                &d,
                &["worktree", "add", "-q", "--detach", &wt.to_string_lossy()],
            );
            commit_file(&wt, &format!("{}.js", path.join("-")), "x", "work");
        };
        // Codex: ~/.codex/worktrees/<id>/<repo>.
        detached(&["worktrees", "05b5", "my repo"]);
        detached(&["worktrees", "9c1d", "my repo"]);
        // Still the same with the parent: a suffix.
        detached(&["a", "x", "wt"]);
        detached(&["b", "x", "wt"]);
        detached(&["solo"]);
        let plan = plan(&d, &[], None, &[]).unwrap();
        let mut labels = names(&plan);
        labels.sort_unstable();
        assert_eq!(
            labels,
            ["05b5/my-repo", "9c1d/my-repo", "solo", "x/wt", "x/wt-2"]
        );
    }

    /// The test command comes from the target's tree, so an agent branch
    /// cannot pick it. What `npm test` runs is still the combined tree's
    /// test script.
    #[test]
    fn a_branch_that_changes_the_test_script_does_not_pick_the_command() {
        let (_dir, d) = repo();
        git(&d, &["checkout", "-q", "-b", "codex/no-tests", "main"]);
        commit_file(&d, "package.json", r#"{"scripts":{}}"#, "no tests");
        git(&d, &["checkout", "-q", "-b", "claude/fast", "main"]);
        commit_file(
            &d,
            "package.json",
            r#"{"scripts":{"test":"exit 0"}}"#,
            "fast",
        );
        git(&d, &["checkout", "-q", "main"]);
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(names(&plan), ["claude/fast", "codex/no-tests"]);
        assert_eq!(plan.checks, ["npm test"]);
        assert_eq!(plan.check_source.as_deref(), Some("package.json"));
    }

    #[test]
    fn at_most_ten_detected_branches_the_newest_and_the_rest_counted() {
        let (_dir, d) = repo();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        for i in 0..12u64 {
            let name = format!("codex/b{i:02}");
            git(&d, &["checkout", "-q", "-b", &name, "main"]);
            std::fs::write(d.join(format!("{i}.js")), "x").unwrap();
            git(&d, &["add", "-A"]);
            // b00 is the newest, b11 the oldest.
            let out = Command::new("git")
                .args(["commit", "-q", "-m", &name])
                .env(
                    "GIT_COMMITTER_DATE",
                    format!("{} +0000", now - 3600 * (i + 1)),
                )
                .current_dir(&d)
                .output()
                .unwrap();
            assert!(out.status.success());
        }
        git(&d, &["checkout", "-q", "main"]);
        let plan = plan(&d, &[], None, &[]).unwrap();
        let kept: Vec<String> = (0..10).map(|i| format!("codex/b{i:02}")).collect();
        assert_eq!(
            (names(&plan), plan.left_out),
            (kept.iter().map(String::as_str).collect(), 2)
        );
        // Named branches are never capped.
        let all = super::plan(&d, &["codex/*".into()], None, &[]).unwrap();
        assert_eq!((all.branches.len(), all.left_out), (12, 0));
    }

    #[test]
    fn a_target_behind_origin_says_by_how_much() {
        let (_dir, d) = repo();
        git(&d, &["checkout", "-q", "-b", "ahead", "main"]);
        commit_file(&d, "a.js", "a", "a");
        commit_file(&d, "b.js", "b", "b");
        git(&d, &["update-ref", "refs/remotes/origin/main", "ahead"]);
        git(&d, &["checkout", "-q", "main"]);
        assert_eq!(plan(&d, &[], None, &[]).unwrap().behind, 2);
    }

    /// Git's rule: a single value's last one counts, and a list comes from
    /// the most specific scope that sets it, so a global pytxo.check or
    /// pytxo.days never runs beside, or beats, the repository's.
    #[test]
    fn settings_follow_git_config_precedence() {
        let (_dir, d) = repo();
        branch(&d, "codex/a", "a.js");
        // Two scopes without touching the user's: local, then worktree.
        git(&d, &["config", "extensions.worktreeConfig", "true"]);
        git(&d, &["config", "--add", "pytxo.check", "exit 1"]);
        git(&d, &["config", "--add", "pytxo.check", "exit 2"]);
        git(&d, &["config", "--add", "pytxo.days", "0"]);
        git(&d, &["config", "--add", "pytxo.days", "3"]);
        let plan = plan(&d, &[], None, &[]).unwrap();
        assert_eq!(plan.checks, ["exit 1", "exit 2"]);
        assert_eq!(plan.max_age_days, 3);
        git(
            &d,
            &["config", "--worktree", "pytxo.check", "git --version"],
        );
        git(&d, &["config", "--worktree", "pytxo.days", "5"]);
        let plan = super::plan(&d, &[], None, &[]).unwrap();
        assert_eq!(plan.checks, ["git --version"]);
        assert_eq!(plan.max_age_days, 5);
    }

    /// Git 2.26 and later say each value's scope; before, the file each one
    /// came from stands in for it, so two values in one file both count.
    #[test]
    fn the_most_specific_scope_wins_with_or_without_scopes() {
        let scoped = "global\texit 1\nlocal\tnpm test\nlocal\tnpm run lint\n";
        assert_eq!(most_specific(scoped), ["npm test", "npm run lint"]);
        let files = "file:C:/Users/me/.gitconfig\texit 1\n\
                     file:.git/config\tnpm test\nfile:.git/config\tnpm run lint\n";
        assert_eq!(most_specific(files), ["npm test", "npm run lint"]);
        assert!(most_specific("").is_empty());
    }
}
