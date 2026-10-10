//! `pytxo land`: the landing gate for agent-written branches.
//!
//! Inputs are commits produced by any agent tool. The gate records which
//! files and symbols each input changed, merges them in a Pytxo-owned
//! worktree at the target's tip, runs the user's checks on the combined
//! result (and optionally on each input alone), decides, and writes a
//! receipt. Shadow mode changes nothing in the repository. `--apply` moves
//! the target only by compare-and-swap (`update-ref <new> <old>`) or a
//! fast-forward of a checked-out branch whose HEAD is still the base, so a
//! target that moved during the gate is refused, never overwritten.

pub mod bench;
pub mod checks;
mod git;
mod overlap;
mod render;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use pytxo_core::{
    new_receipt_id, receipts_dir, Receipt, ReceiptCheck, ReceiptInput, ReceiptPredicate,
    ReceiptSharedFile, ReceiptSymbolCollision,
};

pub use git::Git;
pub use render::{render_markdown, render_text};

pub const DEFAULT_CHECK_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// `label:ref` or `ref`. Git refnames cannot contain `:`, so the split is unambiguous.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LandInput {
    pub label: Option<String>,
    pub reference: String,
}

impl LandInput {
    pub fn parse(arg: &str) -> Self {
        if let Some((label, reference)) = arg.split_once(':') {
            let label_ok = !label.is_empty()
                && label
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
            if label_ok && !reference.is_empty() {
                return Self {
                    label: Some(label.to_string()),
                    reference: reference.to_string(),
                };
            }
        }
        Self {
            label: None,
            reference: arg.to_string(),
        }
    }

    fn name(&self) -> &str {
        &self.reference
    }
}

#[derive(Clone, Debug)]
pub struct LandOptions {
    pub repo: PathBuf,
    pub inputs: Vec<LandInput>,
    /// Defaults to the tip of the target.
    pub base: Option<String>,
    /// Target branch; defaults to the branch checked out in `repo`.
    pub into: Option<String>,
    pub checks: Vec<String>,
    pub per_input: bool,
    pub apply: bool,
    /// Treat symbol collisions as blocking.
    pub strict: bool,
    pub check_timeout: Duration,
    /// Override the receipts directory (tests, benchmarks).
    pub receipts_dir: Option<PathBuf>,
    /// Keep `refs/pytxo/land/<id>` pointing at the combined commit.
    pub keep_ref: bool,
}

impl LandOptions {
    pub fn new(repo: impl Into<PathBuf>, inputs: Vec<LandInput>) -> Self {
        Self {
            repo: repo.into(),
            inputs,
            base: None,
            into: None,
            checks: Vec::new(),
            per_input: false,
            apply: false,
            strict: false,
            check_timeout: DEFAULT_CHECK_TIMEOUT,
            receipts_dir: None,
            keep_ref: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LandOutcome {
    pub receipt: Receipt,
    pub receipt_path: PathBuf,
    pub combined_ref: Option<String>,
    /// Plain-language observations worth leading with.
    pub insights: Vec<String>,
}

impl LandOutcome {
    pub fn decision(&self) -> &str {
        &self.receipt.predicate.decision
    }

    /// 0 ready/landed, 2 blocked, 3 stale.
    pub fn exit_code(&self) -> i32 {
        match self.decision() {
            "ready" | "landed" => 0,
            "stale" => 3,
            _ => 2,
        }
    }
}

struct ResolvedInput {
    input: LandInput,
    commit: String,
    merge_base: String,
    files: Vec<String>,
}

/// A detached worktree removed on drop, success or failure.
struct OwnedWorktree {
    repo: Git,
    path: PathBuf,
}

impl OwnedWorktree {
    fn add(repo: &Git, at: &str, tag: &str) -> Result<Self> {
        let path = std::env::temp_dir().join(format!("pytxo-land-{tag}"));
        if path.exists() {
            std::fs::remove_dir_all(&path).ok();
        }
        // Checkout runs no hooks: the owned config points hooksPath nowhere.
        Git::owned(repo.cwd(), None)
            .run(&[
                "worktree",
                "add",
                "--detach",
                "--force",
                &path.to_string_lossy(),
                at,
            ])
            .context("creating the landing worktree")?;
        Ok(Self {
            repo: repo.clone(),
            path,
        })
    }
}

impl Drop for OwnedWorktree {
    fn drop(&mut self) {
        let _ = self.repo.output(&[
            "worktree",
            "remove",
            "--force",
            &self.path.to_string_lossy(),
        ]);
        if self.path.exists() {
            let _ = std::fs::remove_dir_all(&self.path);
            let _ = self.repo.output(&["worktree", "prune"]);
        }
    }
}

pub fn land(opts: &LandOptions) -> Result<LandOutcome> {
    let started = Instant::now();
    if opts.inputs.is_empty() {
        bail!("name at least one branch or commit to land");
    }
    let root = native_path(
        &Git::user(&opts.repo)
            .run(&["rev-parse", "--show-toplevel"])
            .context("not inside a git repository")?,
    );
    let repo = Git::user(&root);
    let common_dir = {
        let raw = native_path(&repo.run(&["rev-parse", "--git-common-dir"])?);
        if raw.is_absolute() {
            raw
        } else {
            root.join(raw)
        }
    };

    let current_branch = repo
        .run(&["symbolic-ref", "--quiet", "--short", "HEAD"])
        .ok();
    let target = opts.into.clone().or_else(|| current_branch.clone());
    if opts.apply && target.is_none() {
        bail!("--apply needs a branch: pass --into <branch> or check out a branch");
    }
    if let Some(t) = &target {
        repo.commit_of(&format!("refs/heads/{t}"))
            .with_context(|| format!("target branch does not exist: {t}"))?;
    }
    let target_ref = target.as_ref().map(|t| format!("refs/heads/{t}"));
    let base_rev = opts
        .base
        .clone()
        .or_else(|| target_ref.clone())
        .unwrap_or_else(|| "HEAD".into());
    let base = repo.commit_of(&base_rev)?;
    if opts.apply {
        let tip = repo.commit_of(target_ref.as_deref().unwrap_or("HEAD"))?;
        if tip != base {
            bail!("--apply lands onto the current tip of the target; drop --base or point it at that tip");
        }
    }

    let mut resolved = Vec::new();
    for input in &opts.inputs {
        let commit = repo
            .commit_of(&input.reference)
            .with_context(|| format!("unknown input: {}", input.reference))?;
        let merge_base = repo
            .run(&["merge-base", &base, &commit])
            .with_context(|| format!("{} shares no history with the target", input.reference))?;
        let files = repo.lines(&["diff", "--name-only", "--no-renames", &merge_base, &commit])?;
        resolved.push(ResolvedInput {
            input: input.clone(),
            commit,
            merge_base,
            files,
        });
    }

    // Shared files and the symbols more than one input changed in them.
    let mut owners: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, r) in resolved.iter().enumerate() {
        for f in &r.files {
            owners.entry(f.as_str()).or_default().push(i);
        }
    }
    let mut shared_files = Vec::new();
    for (path, idxs) in owners.iter().filter(|(_, v)| v.len() > 1) {
        let mut by_symbol: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        for &i in idxs {
            let r = &resolved[i];
            for sym in overlap::touched_symbols(&repo, &r.merge_base, &r.commit, path) {
                by_symbol.entry(sym).or_default().insert(i);
            }
        }
        shared_files.push(ReceiptSharedFile {
            path: path.to_string(),
            inputs: idxs
                .iter()
                .map(|&i| resolved[i].input.name().to_string())
                .collect(),
            symbol_collisions: by_symbol
                .into_iter()
                .filter(|(_, who)| who.len() > 1)
                .map(|(symbol, who)| ReceiptSymbolCollision {
                    symbol,
                    inputs: who
                        .iter()
                        .map(|&i| resolved[i].input.name().to_string())
                        .collect(),
                })
                .collect(),
        });
    }

    let identity = repo
        .run(&["config", "user.email"])
        .ok()
        .filter(|e| !e.is_empty())
        .map_or(Some(("Pytxo Land", "land@pytxo.invalid")), |_| None);
    let id = new_receipt_id();
    let tag = id.trim_start_matches("rcpt_").to_lowercase();

    // Combine.
    let combined_tree = OwnedWorktree::add(&repo, &base, &tag)?;
    let wt = Git::owned(&combined_tree.path, identity);
    let mut receipt_inputs = Vec::new();
    let mut reasons = Vec::new();
    for r in &resolved {
        let mut status = "merged";
        let mut conflict_files = Vec::new();
        if r.files.is_empty() {
            status = "empty";
        } else {
            let message = format!("pytxo land: merge {}", r.input.reference);
            let out = wt.output(&["merge", "--no-ff", "--no-edit", "-m", &message, &r.commit])?;
            if !out.status.success() {
                status = "conflict";
                conflict_files = wt
                    .lines(&["diff", "--name-only", "--diff-filter=U"])
                    .unwrap_or_default();
                if !wt.ok(&["merge", "--abort"]) {
                    wt.run(&["reset", "--hard", "HEAD"])?;
                }
                reasons.push(format!(
                    "{} conflicts with the inputs before it{}",
                    r.input.reference,
                    if conflict_files.is_empty() {
                        String::new()
                    } else {
                        format!(" in {}", conflict_files.join(", "))
                    }
                ));
            }
        }
        receipt_inputs.push(ReceiptInput {
            reference: r.input.reference.clone(),
            agent: r.input.label.clone(),
            commit: r.commit.clone(),
            status: status.into(),
            files: r.files.clone(),
            conflict_files,
        });
    }
    let combined = wt.run(&["rev-parse", "HEAD"])?;

    // Checks: each input alone first (optional), then the combination.
    let mut checks: Vec<ReceiptCheck> = Vec::new();
    if opts.per_input && !opts.checks.is_empty() {
        let scratch = OwnedWorktree::add(&repo, &base, &format!("{tag}-solo"))?;
        let sg = Git::owned(&scratch.path, identity);
        for r in resolved.iter().filter(|r| !r.files.is_empty()) {
            sg.run(&["reset", "--hard", &base])?;
            sg.run(&["clean", "-fdq"])?;
            let scope = format!("input:{}", r.input.reference);
            let out = sg.output(&[
                "merge",
                "--no-ff",
                "--no-edit",
                "-m",
                "pytxo land: solo",
                &r.commit,
            ])?;
            if !out.status.success() {
                let _ = sg.output(&["merge", "--abort"]);
                checks.push(ReceiptCheck {
                    scope,
                    command: "(merge onto target)".into(),
                    exit_code: None,
                    passed: false,
                    duration_ms: 0,
                    timed_out: false,
                    output_tail: vec!["input does not merge cleanly onto the target".into()],
                });
                continue;
            }
            for command in &opts.checks {
                checks.push(checks::run_check(
                    &scope,
                    command,
                    &scratch.path,
                    opts.check_timeout,
                ));
            }
        }
    }
    for command in &opts.checks {
        let check = checks::run_check("combined", command, &combined_tree.path, opts.check_timeout);
        if !check.passed {
            reasons.push(if check.timed_out {
                format!("combined check timed out: {command}")
            } else {
                format!(
                    "combined check failed: {command} (exit {})",
                    check.exit_code.map_or("?".into(), |c| c.to_string())
                )
            });
        }
        checks.push(check);
    }
    drop(combined_tree);

    let collisions: Vec<String> = shared_files
        .iter()
        .flat_map(|f| {
            f.symbol_collisions
                .iter()
                .map(move |c| format!("{} in {} ({})", c.symbol, f.path, c.inputs.join(", ")))
        })
        .collect();
    if opts.strict {
        reasons.extend(collisions.iter().map(|c| format!("symbol collision: {c}")));
    }

    let files_changed = if combined == base {
        0
    } else {
        repo.lines(&["diff", "--name-only", &base, &combined])?
            .len()
    };

    let combined_ref = if opts.keep_ref && combined != base {
        let name = format!("refs/pytxo/land/{id}");
        repo.run(&["update-ref", &name, &combined])?;
        Some(name)
    } else {
        None
    };

    // Did the target move while we were checking?
    let target_moved = opts.base.is_none()
        && target_ref
            .as_deref()
            .is_some_and(|r| repo.commit_of(r).map(|now| now != base).unwrap_or(true));

    let mut decision = if target_moved {
        reasons.push(format!(
            "{} moved while the gate ran; the result describes an old tip",
            target.as_deref().unwrap_or("target")
        ));
        "stale"
    } else if !reasons.is_empty() {
        "blocked"
    } else {
        "ready"
    };

    if opts.apply && decision == "ready" && files_changed > 0 {
        let t = target.as_deref().expect("checked above");
        decision = match carry_out(&repo, t, &base, &combined, &id)? {
            Landing::Landed => "landed",
            Landing::Stale(why) => {
                reasons.push(why);
                "stale"
            }
            Landing::Refused(why) => {
                reasons.push(why);
                "blocked"
            }
        };
    }

    let mut insights = Vec::new();
    let solo: Vec<&ReceiptCheck> = checks.iter().filter(|c| c.scope != "combined").collect();
    let combined_failed = checks.iter().any(|c| c.scope == "combined" && !c.passed);
    if combined_failed && !solo.is_empty() && solo.iter().all(|c| c.passed) {
        insights.push("Every input passed its checks alone. The combined change failed.".into());
    }
    if !collisions.is_empty() {
        insights.push(format!(
            "{} symbol{} changed by more than one input: {}",
            collisions.len(),
            if collisions.len() == 1 { "" } else { "s" },
            collisions.join("; ")
        ));
    }
    if opts.checks.is_empty() && files_changed > 0 {
        insights.push("No checks ran. Pass --check \"<command>\" to verify the combined change before landing.".into());
    }
    if files_changed == 0 && decision == "ready" {
        insights.push("Nothing to land: the inputs add no changes to the target.".into());
    }

    let mode = if opts.apply { "apply" } else { "shadow" };
    let result_commit = (combined != base).then(|| combined.clone());
    let predicate = ReceiptPredicate {
        id: id.clone(),
        kind: "land".into(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        pytxo_version: env!("CARGO_PKG_VERSION").into(),
        decision: decision.into(),
        mode: mode.into(),
        target: target.clone(),
        base: base.clone(),
        result: result_commit.clone(),
        inputs: receipt_inputs,
        files_changed,
        shared_files,
        checks,
        reasons,
        duration_ms: started.elapsed().as_millis() as u64,
        content_digest: String::new(),
    };
    let subject = format!("git:{}", target.as_deref().unwrap_or("HEAD"));
    let receipt = Receipt::new(predicate, &subject, result_commit.as_deref());
    let dir = opts
        .receipts_dir
        .clone()
        .unwrap_or_else(|| receipts_dir(&common_dir));
    let receipt_path = receipt.write_to(&dir).context("writing the receipt")?;

    Ok(LandOutcome {
        receipt,
        receipt_path,
        combined_ref,
        insights,
    })
}

enum Landing {
    Landed,
    Stale(String),
    Refused(String),
}

fn carry_out(repo: &Git, target: &str, base: &str, combined: &str, id: &str) -> Result<Landing> {
    let refname = format!("refs/heads/{target}");
    if let Some(path) = checked_out_at(repo, &refname)? {
        let there = Git::user(&path);
        let head = there.commit_of("HEAD")?;
        if head != base {
            return Ok(Landing::Stale(format!(
                "{target} moved to {} before landing",
                short(&head)
            )));
        }
        let out = there.output(&["merge", "--ff-only", "--no-edit", combined])?;
        if !out.status.success() {
            let why = String::from_utf8_lossy(&out.stderr);
            let first = why
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("fast-forward refused");
            return Ok(Landing::Refused(format!(
                "could not fast-forward {target} in {}: {}",
                path.display(),
                first.trim()
            )));
        }
        return Ok(Landing::Landed);
    }
    let out = repo.output(&[
        "update-ref",
        "-m",
        &format!("pytxo land {id}"),
        &refname,
        combined,
        base,
    ])?;
    if out.status.success() {
        Ok(Landing::Landed)
    } else {
        Ok(Landing::Stale(format!(
            "{target} moved before landing; nothing was changed"
        )))
    }
}

/// The worktree that has `refname` checked out, if any.
fn checked_out_at(repo: &Git, refname: &str) -> Result<Option<PathBuf>> {
    let listing = repo.run(&["worktree", "list", "--porcelain"])?;
    let mut current: Option<PathBuf> = None;
    for line in listing.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            current = Some(PathBuf::from(p));
        } else if line.strip_prefix("branch ") == Some(refname) {
            return Ok(current);
        }
    }
    Ok(None)
}

/// Git prints `/` separators even on Windows; keep displayed paths native.
fn native_path(raw: &str) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(raw.replace('/', "\\"))
    } else {
        PathBuf::from(raw)
    }
}

pub(crate) fn short(sha: &str) -> &str {
    &sha[..sha.len().min(9)]
}

/// Receipts directory for a repository (`.git/pytxo/receipts`).
pub fn repo_receipts_dir(repo: &Path) -> Result<PathBuf> {
    let git = Git::user(repo);
    let root = native_path(&git.run(&["rev-parse", "--show-toplevel"])?);
    let raw = native_path(&Git::user(&root).run(&["rev-parse", "--git-common-dir"])?);
    let common = if raw.is_absolute() {
        raw
    } else {
        root.join(raw)
    };
    Ok(receipts_dir(&common))
}

#[cfg(test)]
mod tests;
