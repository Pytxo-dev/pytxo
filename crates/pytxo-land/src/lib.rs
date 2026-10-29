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
mod cause;
pub mod checks;
pub mod detect;
mod git;
mod overlap;
mod render;

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, Once, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use pytxo_core::{
    new_receipt_id, receipts_dir, Receipt, ReceiptCheck, ReceiptInput, ReceiptPredicate,
    ReceiptSharedFile, ReceiptSymbolCollision,
};

pub use cause::{Cause, Use};
pub use git::Git;
pub use render::{
    fix_prompt, render_markdown, render_markdown_with, render_text, verdict, NamedCause, Reader,
    Verdict, INSTALL_HINT,
};

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

    /// As the verdict names it: the label of a detached worktree's commit,
    /// otherwise the reference.
    fn name(&self) -> &str {
        render::plain_name(&self.reference, self.label.as_deref())
    }
}

/// When each input is also checked alone, beside the combined change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PerInput {
    /// Only the combined change is checked.
    #[default]
    Never,
    /// Each input alone first, then the combination (`--per-input`).
    Always,
    /// The combination first; each input alone only after it fails, which
    /// tells "fails together" from "fails alone" without paying for it on a pass.
    OnFailure,
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
    pub per_input: PerInput,
    pub apply: bool,
    /// Treat symbol collisions as blocking.
    pub strict: bool,
    pub check_timeout: Duration,
    /// No test run starts after this, and the one running then is cut short
    /// (`pytxo hook`, which must answer before the agent tool kills it).
    pub deadline: Option<Instant>,
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
            per_input: PerInput::Never,
            apply: false,
            strict: false,
            check_timeout: DEFAULT_CHECK_TIMEOUT,
            deadline: None,
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
    /// The deadline stopped the checks before they finished, so the decision
    /// says nothing about the inputs.
    pub out_of_time: bool,
    /// The target has submodules, which the checks' worktrees leave out.
    pub submodules: bool,
    /// The conflicting inputs that conflict with the target itself, not only
    /// with the inputs merged before them.
    pub target_conflicts: Vec<String>,
    /// When every merged input passed alone and the combined change failed
    /// twice: a name one input renamed or removed that another still uses.
    /// A reading of the runs, so not in the receipt.
    pub cause: Option<Cause>,
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

/// Scratch worktrees are `<temp>/pytxo-<tag>/pytxo-land-<tag>` or
/// `...-<tag>-solo`, where the tag is the receipt id without `rcpt_`, in
/// lower case.
const SCRATCH_PREFIX: &str = "pytxo-land-";

/// One of Pytxo's own scratch worktrees: this run's, a concurrent run's, or
/// one a killed run left behind. Matched by the name `OwnedWorktree::add`
/// gives it, not by its parent: the temp directory differs between a terminal
/// and a hook, and between its short (8.3), long and symlinked spellings.
pub(crate) fn is_scratch_worktree(path: &Path) -> bool {
    scratch_tag(path).is_some()
}

/// The tag of a scratch worktree's run, without `-solo`.
fn scratch_tag(path: &Path) -> Option<&str> {
    let name = path.file_name()?.to_str()?.strip_prefix(SCRATCH_PREFIX)?;
    // `20261028t120005z_ab12cd34`, as `new_receipt_id` writes it.
    let tag = name.strip_suffix("-solo").unwrap_or(name);
    let ok = tag.len() == 25
        && tag.bytes().enumerate().all(|(i, c)| match i {
            8 => c == b't',
            15 => c == b'z',
            16 => c == b'_',
            17.. => c.is_ascii_hexdigit(),
            _ => c.is_ascii_digit(),
        });
    ok.then_some(tag)
}

impl OwnedWorktree {
    fn add(repo: &Git, dir: &Path, at: &str, tag: &str) -> Result<Self> {
        let path = dir.join(format!("{SCRATCH_PREFIX}{tag}"));
        if path.exists() {
            std::fs::remove_dir_all(&path).ok();
        }
        // Recorded first, so an interrupt during the checkout removes it too;
        // dropped, it is removed even when the checkout fails.
        active().worktrees.push((repo.clone(), path.clone()));
        let tree = Self {
            repo: repo.clone(),
            path,
        };
        let path = tree.path.to_string_lossy().into_owned();
        // Checkout runs no hooks: the owned config points hooksPath nowhere.
        let owned = Git::owned(repo.cwd(), None);
        owned
            .run_stoppable(&["worktree", "add", "--detach", "--force", &path, at])
            .context("creating the landing worktree")?;
        // The pid tells a later run whether this one still goes.
        let reason = format!("{LOCK_REASON}{}", std::process::id());
        let _ = owned.output(&["worktree", "lock", "--reason", &reason, &path]);
        Ok(tree)
    }
}

impl Drop for OwnedWorktree {
    fn drop(&mut self) {
        remove_worktree(&self.repo, &self.path);
        active().worktrees.retain(|(_, p)| p != &self.path);
    }
}

/// Back to a fresh worktree's files before the next run in it. What a build
/// left in ignored directories goes too: a test command that runs `tsc`'s
/// outDir would run the last branch's tests as well. Dependencies stay, since
/// their tools never run stale output: `node_modules`, which an install brings
/// in line, and Cargo's `target`. Best effort: what it cannot remove, such as a
/// container's root-owned cache or a file a leftover process holds open, stays,
/// and the run goes on to its verdict.
const SCRATCH_CLEAN: [&str; 6] = ["clean", "-ffdxq", "-e", "node_modules", "-e", "target"];

/// `git worktree lock --reason`, then the pid of the run that holds it.
const LOCK_REASON: &str = "pytxo ";

/// Unlocks and removes a scratch worktree, as far as it goes.
fn remove_worktree(repo: &Git, path: &Path) {
    let path_arg = path.to_string_lossy();
    let _ = repo.output(&["worktree", "unlock", &path_arg]);
    if !repo.ok(&["worktree", "remove", "--force", &path_arg]) {
        // Half made, or its directory already gone: what is left, then git's
        // record of it.
        let _ = std::fs::remove_dir_all(path);
        let _ = repo.output(&["worktree", "prune"]);
    }
}

/// Removes the scratch worktrees killed runs left registered in `repo`:
/// those of a run that started a day or more ago, or whose process is gone.
/// A concurrent run's stay.
fn remove_stale_worktrees(repo: &Git) {
    let Ok(listing) = repo.run(&["worktree", "list", "--porcelain"]) else {
        return;
    };
    // ponytail: by age a run counts as dead after a day even if its pid runs
    // (pids are reused), so a run past 24 hours can lose its worktrees to
    // another; record the start time in the lock if runs ever last that long.
    let cutoff = (chrono::Utc::now() - chrono::Duration::hours(24))
        .format("%Y%m%dt%H%M%Sz")
        .to_string();
    for block in listing.replace("\r\n", "\n").split("\n\n") {
        let Some(path) = block
            .lines()
            .find_map(|l| l.strip_prefix("worktree "))
            .map(native_path)
        else {
            continue;
        };
        let Some(tag) = scratch_tag(&path) else {
            continue;
        };
        let pid = block
            .lines()
            .find_map(|l| l.strip_prefix(&format!("locked {LOCK_REASON}")))
            .and_then(|p| p.trim().parse::<u32>().ok());
        // Tags start with the time, so they sort by it.
        if tag[..16] < *cutoff || pid.is_some_and(|p| !checks::alive(p)) {
            remove_worktree(repo, &path);
            // Its run's directory holds the run's logs too.
            let run_dir = path
                .parent()
                .filter(|d| d.ends_with(format!("pytxo-{tag}")));
            if let Some(dir) = run_dir {
                let _ = std::fs::remove_dir_all(dir);
            }
        }
    }
}

/// Where a run's scratch worktrees and check logs go: a directory of the
/// run's own, removed with the run, or by the next run after a kill. Any
/// local user can write to a Unix temp directory, so there only the user can
/// enter it.
struct RunDir(PathBuf);

impl RunDir {
    fn new(tag: &str) -> Result<Self> {
        let dir = long_temp_dir().join(format!("pytxo-{tag}"));
        #[cfg(unix)]
        let made = {
            use std::os::unix::fs::DirBuilderExt;
            std::fs::DirBuilder::new().mode(0o700).create(&dir)
        };
        #[cfg(not(unix))]
        let made = std::fs::create_dir(&dir);
        made.context("creating the run's scratch directory")?;
        active().paths.push(dir.clone());
        Ok(Self(dir))
    }
}

impl Drop for RunDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
        active().paths.retain(|p| p != &self.0);
    }
}

/// The temp directory, on Windows spelled in full where that is safe: checks
/// whose working directory reads `C:\Users\RUNNER~1\...` see it unlike its
/// real path.
fn long_temp_dir() -> PathBuf {
    let temp = std::env::temp_dir();
    #[cfg(windows)]
    if let Some(full) = std::fs::canonicalize(&temp)
        .ok()
        .and_then(|full| plain_drive_path(&full.to_string_lossy()))
    {
        return full;
    }
    temp
}

/// `\\?\C:\...` without the prefix, when the rest is printable ASCII with no
/// space; otherwise the short spelling serves better. A space or a non-ASCII
/// letter breaks tools that use the working directory unquoted (make, npm
/// scripts), cmd.exe refuses a UNC working directory (`\\?\UNC\...`), and a
/// volume GUID path (`\\?\Volume{...}\`) is relative without its prefix.
#[cfg(any(windows, test))]
fn plain_drive_path(canonical: &str) -> Option<PathBuf> {
    let path = canonical.strip_prefix(r"\\?\")?;
    let drive = matches!(path.as_bytes(), [d, b':', b'\\', ..] if d.is_ascii_alphabetic());
    (drive && path.bytes().all(|b| b.is_ascii_graphic())).then(|| PathBuf::from(path))
}

/// What a run has going that Ctrl+C or a kill must not leave behind.
pub(crate) struct Active {
    /// Check processes; on Unix each leads its own process group.
    pub(crate) checks: Vec<u32>,
    worktrees: Vec<(Git, PathBuf)>,
    /// Check logs and the run's directory.
    pub(crate) paths: Vec<PathBuf>,
}

static ACTIVE: Mutex<Active> = Mutex::new(Active {
    checks: Vec::new(),
    worktrees: Vec::new(),
    paths: Vec::new(),
});

/// The running checks and scratch worktrees. The first call installs the
/// Ctrl+C, SIGTERM and SIGHUP handler that clears them away.
pub(crate) fn active() -> MutexGuard<'static, Active> {
    static HANDLER: Once = Once::new();
    HANDLER.call_once(|| {
        // A signal ignored when pytxo started stays ignored, as `nohup pytxo`
        // and a script's `pytxo &` ask. Only for the binary itself: npx and
        // npm start it with every signal back to its default.
        #[cfg(unix)]
        let ignored: Vec<libc::c_int> = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP]
            .into_iter()
            .filter(|&signal| {
                // SAFETY: all zeroes is a valid sigaction, and without a new
                // action sigaction only reads the current one into it.
                let mut old: libc::sigaction = unsafe { std::mem::zeroed() };
                let read = unsafe { libc::sigaction(signal, std::ptr::null(), &mut old) } == 0;
                read && old.sa_sigaction == libc::SIG_IGN
            })
            .collect();
        // Replaces any handler the program set, except one it set through
        // ctrlc first, which stays.
        let _ = ctrlc::set_handler(interrupted);
        #[cfg(unix)]
        for signal in ignored {
            // SAFETY: ignoring a signal runs no code of ours.
            unsafe { libc::signal(signal, libc::SIG_IGN) };
        }
    });
    ACTIVE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Installs the interrupt handler now, so an interrupt during the first
/// steps (finding branches, building the demo) exits with 130 as well.
pub fn watch_interrupts() {
    drop(active());
}

/// Set by the first Ctrl+C, SIGTERM or SIGHUP.
static STOPPING: AtomicBool = AtomicBool::new(false);

/// The first signal says so and clears up on a thread of its own, since ctrlc
/// handles the next signal only once this returns; the second exits at once.
fn interrupted() {
    if STOPPING.swap(true, Ordering::SeqCst) {
        std::process::exit(130);
    }
    let _ = writeln!(
        std::io::stderr(),
        "pytxo is stopping and removing its scratch checkout. Press Ctrl+C again to quit at once."
    );
    if std::thread::Builder::new()
        .spawn(clean_up_and_exit)
        .is_err()
    {
        clean_up_and_exit();
    }
}

/// Stops the checks, removes the scratch worktrees and exits with 130. The
/// lock is never released, so nothing new starts meanwhile.
fn clean_up_and_exit() {
    let active = ACTIVE.lock().unwrap_or_else(PoisonError::into_inner);
    for &pid in &active.checks {
        checks::kill_tree(pid);
    }
    for (repo, path) in &active.worktrees {
        remove_worktree(repo, path);
    }
    for path in &active.paths {
        let _ = std::fs::remove_dir_all(path).or_else(|_| std::fs::remove_file(path));
    }
    std::process::exit(130);
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

    let current_branch = repo.current_branch();
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
            .branch_commit(&input.reference)
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
    remove_stale_worktrees(&repo);
    let run_dir = RunDir::new(&tag)?;
    let target_name = target.as_deref().unwrap_or("HEAD");

    // Combine. Git >= 2.38 merges in memory (`merge-tree --write-tree` plus
    // `commit-tree`): no checkout, no hooks, no index. Older Git falls back to
    // merging in a Pytxo-owned worktree.
    let in_memory = supports_merge_tree(&repo);
    let owned = Git::owned(&root, identity);
    let mut receipt_inputs: Vec<ReceiptInput> = Vec::new();
    let mut reasons = Vec::new();
    let mut target_conflicts = Vec::new();
    let mut combined = base.clone();
    let fallback_tree = if in_memory {
        None
    } else {
        Some(OwnedWorktree::add(&repo, &run_dir.0, &base, &tag)?)
    };
    for r in &resolved {
        let mut status = "merged";
        let mut conflict_files = Vec::new();
        if r.files.is_empty() {
            status = "empty";
        } else {
            let message = format!("pytxo land: merge {}", r.input.reference);
            let merge = |onto: &str| match &fallback_tree {
                None => merge_in_memory(&owned, onto, &r.commit, &message),
                Some(tree) => {
                    let wt = Git::owned(&tree.path, identity);
                    wt.run(&["reset", "--hard", onto])?;
                    merge_in_worktree(&wt, &r.commit, &message)
                }
            };
            let merged = merge(&combined)?;
            match merged {
                Ok(commit) => combined = commit,
                Err(files) => {
                    status = "conflict";
                    conflict_files = files;
                    // Merges start from the target, so an input that does
                    // not merge onto it alone either conflicts with the
                    // target; any other, with the inputs before it.
                    let alone = combined == base || merge(&base)?.is_err();
                    if alone {
                        target_conflicts.push(r.input.reference.clone());
                    }
                    reasons.push(format!(
                        "{} conflicts with {}{}",
                        r.input.reference,
                        if alone {
                            target_name
                        } else {
                            "the inputs before it"
                        },
                        if conflict_files.is_empty() {
                            String::new()
                        } else {
                            format!(" in {}", conflict_files.join(", "))
                        }
                    ));
                }
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
    drop(fallback_tree);

    // Checks: the combination, and each input alone when asked. Only checks
    // need files on disk, so only checks create worktrees.
    let mut checks: Vec<ReceiptCheck> = Vec::new();
    // Past the deadline nothing more starts; a run gets at most what is left.
    let out_of_time = std::cell::Cell::new(false);
    let late = || {
        let late = opts.deadline.is_some_and(|d| Instant::now() >= d);
        out_of_time.set(out_of_time.get() || late);
        late
    };
    let run_check = |scope: &str, command: &str, dir: &Path| {
        if late() {
            return None;
        }
        let left = opts
            .deadline
            .map(|d| d.saturating_duration_since(Instant::now()));
        let timeout = left.map_or(opts.check_timeout, |l| l.min(opts.check_timeout));
        let check = checks::run_check(scope, command, dir, &run_dir.0, timeout);
        if check.timed_out && timeout < opts.check_timeout {
            out_of_time.set(true);
        }
        Some(check)
    };
    let run_solo = |checks: &mut Vec<ReceiptCheck>| -> Result<()> {
        let scratch = OwnedWorktree::add(&repo, &run_dir.0, &base, &format!("{tag}-solo"))?;
        let sg = Git::owned(&scratch.path, identity);
        for r in resolved.iter().filter(|r| !r.files.is_empty()) {
            if late() {
                return Ok(());
            }
            let scope = format!("input:{}", r.input.reference);
            let solo = if in_memory {
                merge_in_memory(&owned, &base, &r.commit, "pytxo land: solo")?
            } else {
                sg.run(&["reset", "--hard", &base])?;
                merge_in_worktree(&sg, &r.commit, "pytxo land: solo")?
            };
            let Ok(solo) = solo else {
                checks.push(ReceiptCheck {
                    scope,
                    command: "(merge onto target)".into(),
                    exit_code: None,
                    passed: false,
                    duration_ms: 0,
                    timed_out: false,
                    output_tail: vec!["input does not merge cleanly onto the target".into()],
                    rerun: false,
                });
                continue;
            };
            sg.run(&["checkout", "--detach", "--force", &solo])?;
            let _ = sg.run(&SCRATCH_CLEAN);
            for command in &opts.checks {
                let Some(check) = run_check(&scope, command, &scratch.path) else {
                    return Ok(());
                };
                checks.push(check);
            }
        }
        Ok(())
    };
    if opts.per_input == PerInput::Always && !opts.checks.is_empty() {
        run_solo(&mut checks)?;
    }
    let combined_tree = if opts.checks.is_empty() {
        None
    } else {
        Some(OwnedWorktree::add(&repo, &run_dir.0, &combined, &tag)?)
    };
    let dir = combined_tree
        .as_ref()
        .map(|t| t.path.clone())
        .unwrap_or_default();
    // A run that times out is the answer: nothing runs after it, since each
    // further run could take the whole timeout to say the same.
    let mut first_runs: Vec<ReceiptCheck> = Vec::new();
    for command in &opts.checks {
        let Some(check) = run_check("combined", command, &dir) else {
            break;
        };
        let timed_out = check.timed_out;
        first_runs.push(check);
        if timed_out {
            break;
        }
    }
    let timed_out = first_runs.iter().any(|c| c.timed_out);
    if opts.per_input == PerInput::OnFailure
        && !timed_out
        && first_runs.iter().any(|c| !c.passed)
        && !late()
    {
        run_solo(&mut checks)?;
    }
    // "Every input passed alone, the combination failed" is the claim this
    // gate leads with, so one flaky run must not make it: a failed check that
    // every input passed alone runs once more. Not a timeout, which would take
    // as long again to say the same.
    let again: Vec<bool> = opts
        .checks
        .iter()
        .zip(&first_runs)
        .map(|(command, check)| {
            let solo: Vec<&ReceiptCheck> = checks
                .iter()
                .filter(|c| c.scope != "combined" && &c.command == command)
                .collect();
            !check.passed && !check.timed_out && !solo.is_empty() && solo.iter().all(|c| c.passed)
        })
        .collect();
    // From the combined commit's files, as the first run started, and after the
    // checks before it, whose output it may build on. Those run again only to
    // set it up, so the receipt keeps their first runs.
    let mut reruns: Vec<Option<ReceiptCheck>> = Vec::new();
    if let Some(last) = again.iter().rposition(|&a| a) {
        let cg = Git::owned(&dir, identity);
        cg.run(&["checkout", "--detach", "--force", &combined])?;
        let _ = cg.run(&SCRATCH_CLEAN);
        for (command, &needed) in opts.checks.iter().zip(&again).take(last + 1) {
            let Some(check) = run_check("combined", command, &dir) else {
                break;
            };
            reruns.push(needed.then_some(ReceiptCheck {
                rerun: true,
                ..check
            }));
        }
    }
    for (i, (command, check)) in opts.checks.iter().zip(first_runs).enumerate() {
        let rerun = reruns.get_mut(i).and_then(Option::take);
        let how = |c: &ReceiptCheck| {
            if c.timed_out {
                "timed out".to_string()
            } else {
                format!("exit {}", c.exit_code.map_or("?".into(), |e| e.to_string()))
            }
        };
        match &rerun {
            Some(again) if again.passed => reasons.push(format!(
                "combined check flaky: {command} failed ({}), then passed when run again",
                how(&check)
            )),
            Some(again) => reasons.push(format!(
                "combined check failed twice: {command} ({}, then {})",
                how(&check),
                how(again)
            )),
            None if check.timed_out => reasons.push(format!("combined check timed out: {command}")),
            None if !check.passed => reasons.push(format!(
                "combined check failed: {command} ({})",
                how(&check)
            )),
            None => {}
        }
        checks.push(check);
        checks.extend(rerun);
    }
    // Why they fail together, when they do: each merged input passed alone,
    // the combined change failed again when run again, neither run timed out
    // and no input conflicts, the "together" verdict. From the runs of the
    // command that verdict speaks of, the first whose combined run failed and
    // did not pass when run again (`failed_combined` in render.rs): a later
    // command that fails together, after one a branch fails alone, has no
    // cause, so nothing reads or runs more for a verdict that does not show
    // it. Never past the deadline, so it cannot hold up the answer. Read
    // before the target run, while the combined worktree is still here.
    let speaks_of = opts.checks.iter().zip(&again).find(|(command, _)| {
        let runs = || {
            checks
                .iter()
                .filter(|c| c.scope == "combined" && &c.command == *command)
        };
        runs().any(|c| !c.rerun && !c.passed) && !runs().any(|c| c.rerun && c.passed)
    });
    let together = speaks_of
        .filter(|&(command, &alone_ok)| {
            alone_ok
                && checks
                    .iter()
                    .any(|c| c.rerun && !c.passed && !c.timed_out && &c.command == command)
        })
        .map(|(command, _)| command);
    let in_time = !out_of_time.get() && !opts.deadline.is_some_and(|d| Instant::now() >= d);
    let cause = match together {
        Some(command) if in_time && receipt_inputs.iter().all(|i| i.status != "conflict") => {
            let merged: Vec<cause::Merged> = resolved
                .iter()
                .zip(&receipt_inputs)
                .enumerate()
                .filter(|(_, (_, i))| i.status == "merged")
                .map(|(index, (r, _))| cause::Merged {
                    index,
                    merge_base: &r.merge_base,
                    commit: &r.commit,
                })
                .collect();
            let failing: Vec<&[String]> = checks
                .iter()
                .filter(|c| c.scope == "combined" && !c.passed && &c.command == command)
                .map(|c| c.output_tail.as_slice())
                .collect();
            cause::find(&repo, &merged, &combined, &failing)
        }
        _ => None,
    };
    // A check every input fails alone blames them only if the target passes
    // it: run it there once, in the combined worktree, which is done with.
    // Not after a timeout, which would take as long again.
    let on_target: Vec<&String> = opts
        .checks
        .iter()
        .filter(|command| {
            let runs = || checks.iter().filter(|c| &c.command == *command);
            let alone: Vec<&ReceiptCheck> =
                runs().filter(|c| c.scope.starts_with("input:")).collect();
            runs().any(|c| c.scope == "combined" && !c.passed)
                && !runs().any(|c| c.timed_out)
                && !alone.is_empty()
                && alone.iter().all(|c| !c.passed)
        })
        .collect();
    if let (Some(tree), false) = (&combined_tree, on_target.is_empty() || late()) {
        let tg = Git::owned(&tree.path, identity);
        tg.run(&["checkout", "--detach", "--force", &base])?;
        let _ = tg.run(&SCRATCH_CLEAN);
        for command in on_target {
            let Some(check) = run_check("target", command, &tree.path) else {
                break;
            };
            checks.push(check);
        }
    }
    drop(combined_tree);
    drop(run_dir);
    let out_of_time = out_of_time.get();
    if out_of_time {
        reasons.push("out of time: the deadline stopped the checks before they finished".into());
    }

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
    let solo: Vec<&ReceiptCheck> = checks
        .iter()
        .filter(|c| c.scope.starts_with("input:"))
        .collect();
    // Only a combined check that failed its re-run too blames the combination.
    let failed_twice = checks.iter().any(|c| c.rerun && !c.passed);
    if failed_twice && solo.iter().all(|c| c.passed) {
        insights.push(
            "Every input passed its checks alone. The combined change failed both times it ran."
                .into(),
        );
    }
    for c in checks.iter().filter(|c| c.rerun && c.passed) {
        insights.push(format!(
            "`{}` failed on the combined change, then passed when run again: the check is flaky, so this run does not show that the combination is at fault.",
            c.command
        ));
    }
    if !collisions.is_empty() {
        insights.push(format!(
            "{} symbol{} changed by more than one input: {}",
            collisions.len(),
            if collisions.len() == 1 { "" } else { "s" },
            collisions.join("; ")
        ));
    }
    // Suggested from the project files for the user to pass; never run unasked.
    let suggestion = checks::suggest_check(&root);
    if opts.checks.is_empty() && files_changed > 0 {
        insights.push(format!(
            "No checks ran. Pass --check \"{}\" to verify the combined change before landing.",
            suggestion.as_deref().unwrap_or("<command>")
        ));
    }
    // A check that fails everywhere, or a Node check that never installs, is
    // most likely failing for want of a set-up worktree, not because of the inputs.
    // A pass alone or on the target shows the set-up works, so no install hint.
    for command in &opts.checks {
        let of_command = |c: &&ReceiptCheck| &c.command == command;
        if !checks
            .iter()
            .filter(of_command)
            .rfind(|c| c.scope == "combined")
            .is_some_and(|c| !c.passed)
        {
            continue;
        }
        let solo: Vec<&ReceiptCheck> = checks
            .iter()
            .filter(of_command)
            .filter(|c| c.scope.starts_with("input:"))
            .collect();
        // A run that timed out on the target says nothing about it.
        let on_target = checks
            .iter()
            .filter(of_command)
            .find(|c| c.scope == "target" && !c.timed_out);
        let everywhere = !solo.is_empty()
            && solo.iter().all(|c| !c.passed)
            && !on_target.is_some_and(|c| c.passed);
        let set_up = solo.iter().chain(&on_target).any(|c| c.passed);
        let install_hint = suggestion
            .as_deref()
            .filter(|s| checks::installs(s) && !checks::installs(command) && !set_up);
        let mut insight = String::new();
        if everywhere {
            insight.push_str(&match on_target {
                Some(_) => format!("`{command}` failed on every input alone and on {target_name} too, so the check itself may be what fails, not the combination."),
                None => format!("`{command}` failed on every input alone too, so the check itself may be what fails, not the combination."),
            });
        }
        if let Some(s) = install_hint {
            if !insight.is_empty() {
                insight.push(' ');
            }
            insight.push_str(&format!(
                "Checks run in a fresh worktree where dependencies are not installed: install them in the check, for example --check \"{s}\"."
            ));
        }
        if !insight.is_empty() {
            insights.push(insight);
        }
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
    let submodules = repo.ok(&["cat-file", "-e", &format!("{base}:.gitmodules")]);

    Ok(LandOutcome {
        receipt,
        receipt_path,
        combined_ref,
        insights,
        out_of_time,
        submodules,
        target_conflicts,
        cause,
    })
}

/// `git merge-tree --write-tree` arrived in Git 2.38.
pub(crate) fn supports_merge_tree(repo: &Git) -> bool {
    if std::env::var_os("PYTXO_LAND_WORKTREE_MERGE").is_some() {
        return false;
    }
    let Ok(version) = repo.run(&["--version"]) else {
        return false;
    };
    let mut parts = version
        .split_whitespace()
        .nth(2)
        .unwrap_or("")
        .split('.')
        .map(|p| p.parse::<u32>().unwrap_or(0));
    let (major, minor) = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));
    (major, minor) >= (2, 38)
}

/// Merge `commit` into `onto` without a worktree. `Ok(new commit)` or
/// `Err(conflicted paths)`.
fn merge_in_memory(
    git: &Git,
    onto: &str,
    commit: &str,
    message: &str,
) -> Result<std::result::Result<String, Vec<String>>> {
    let out = git.output(&[
        "merge-tree",
        "--write-tree",
        "--name-only",
        "--no-messages",
        onto,
        commit,
    ])?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let tree = lines.next().unwrap_or_default().to_string();
    match out.status.code() {
        Some(0) => Ok(Ok(git.run(&[
            "commit-tree",
            &tree,
            "-p",
            onto,
            "-p",
            commit,
            "-m",
            message,
        ])?)),
        Some(1) => {
            let mut files: Vec<String> = lines.map(str::to_string).collect();
            files.dedup();
            Ok(Err(files))
        }
        _ => bail!(
            "git merge-tree failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ),
    }
}

/// Fallback for Git < 2.38: merge in a Pytxo-owned worktree (hooks off).
fn merge_in_worktree(
    wt: &Git,
    commit: &str,
    message: &str,
) -> Result<std::result::Result<String, Vec<String>>> {
    let out = wt.output(&["merge", "--no-ff", "--no-edit", "-m", message, commit])?;
    if out.status.success() {
        return Ok(Ok(wt.run(&["rev-parse", "HEAD"])?));
    }
    let files = wt
        .lines(&["diff", "--name-only", "--diff-filter=U"])
        .unwrap_or_default();
    if !wt.ok(&["merge", "--abort"]) {
        wt.run(&["reset", "--hard", "HEAD"])?;
    }
    Ok(Err(files))
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
pub(crate) fn native_path(raw: &str) -> PathBuf {
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
