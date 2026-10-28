//! `pytxo` / `pytxo check`: find the agent branches and the test command,
//! check the branches together, print the verdict. Also `pytxo demo`,
//! `pytxo hook` and `pytxo receipts`.

use std::fs::OpenOptions;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use clap::Args;
use pytxo_land::detect::{self, Plan};
use pytxo_land::{land, render_markdown_with, render_text, verdict, LandOptions, PerInput};
use serde_json::json;

#[derive(Args, Clone, Debug, Default)]
pub struct CheckArgs {
    /// Branches or commits to check, in merge order (default: the agent branches found)
    pub branches: Vec<String>,
    /// Target branch (default: PYTXO_INTO or git config pytxo.into, else the
    /// checked-out branch when you name branches, else origin/HEAD, main, master or trunk)
    #[arg(long)]
    pub into: Option<String>,
    /// Base commit (default: the target's tip)
    #[arg(long)]
    pub base: Option<String>,
    /// Test command, repeatable (default: found in the target, e.g. npm test)
    #[arg(long = "check", short = 'c')]
    pub checks: Vec<String>,
    /// Test each branch alone first, even when the combination passes
    #[arg(long)]
    pub per_input: bool,
    /// Land a ready result: fast-forward or compare-and-swap the target
    #[arg(long)]
    pub apply: bool,
    /// Block when two branches changed the same function or type
    #[arg(long)]
    pub strict: bool,
    /// Timeout for each test run, in seconds
    #[arg(long, default_value = "900")]
    pub timeout: u64,
    /// Show what was found, every test run with its error lines, the receipt's path and a ref to the combined commit; with --markdown, the receipt's id and digest
    #[arg(long, short = 'v')]
    pub verbose: bool,
    /// Print the verdict, what was found and the receipt as JSON
    #[arg(long)]
    pub json: bool,
    /// Pull-request comment format
    #[arg(long, conflicts_with = "json")]
    pub markdown: bool,
    /// Repository (default: the current directory)
    #[arg(long)]
    pub repo: Option<PathBuf>,
}

/// The Pytxo 1.x commands only a build with the default `full` feature has.
#[cfg_attr(feature = "full", allow(dead_code))]
pub const FULL_ONLY: [&str; 22] = [
    "accounts",
    "agents",
    "apply",
    "doctor",
    "domains",
    "fleet",
    "hitl",
    "init",
    "login",
    "logout",
    "logs",
    "mission",
    "models",
    "project",
    "providers",
    "review",
    "run",
    "shell",
    "status",
    "stop",
    "trust",
    "whoami",
];

/// The error for `arg` when it names a Pytxo 1.x command this build leaves
/// out and no branch or commit: `pytxo shell` would look for a branch.
#[cfg(not(feature = "full"))]
pub fn left_out(arg: &str, cwd: &Path) -> Option<String> {
    (FULL_ONLY.contains(&arg) && pytxo_land::Git::user(cwd).commit_of(arg).is_err())
        .then(|| not_in_this_build(arg))
}

/// The same error for a command line the parser stops at or shows help for:
/// `help shell`, `shell --help`, `receipts share` and `bench --submit` too.
#[cfg(not(feature = "full"))]
pub fn left_out_command(args: &[String], cwd: &Path) -> Option<String> {
    let words: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| !a.starts_with('-'))
        .collect();
    let words = match words.as_slice() {
        ["help", rest @ ..] => rest,
        all => all,
    };
    match words {
        ["receipts", rest @ ..] if rest.contains(&"share") => {
            Some(not_in_this_build("receipts share"))
        }
        ["bench", ..] if args.iter().any(|a| a == "--submit") => {
            Some(not_in_this_build("bench --submit"))
        }
        [first, ..] => left_out(first, cwd),
        [] => None,
    }
}

#[cfg(not(feature = "full"))]
fn not_in_this_build(what: &str) -> String {
    format!("{what} is a Pytxo 1.x command that is not in this build; a source build with default features has it")
}

/// Runs the check and returns the exit code: 0 ready or nothing to check,
/// 2 blocked, 3 stale.
pub fn run(args: CheckArgs) -> Result<i32> {
    let cwd = match &args.repo {
        Some(repo) => repo.clone(),
        None => std::env::current_dir()?,
    };
    #[cfg(not(feature = "full"))]
    if let Some(line) = args.branches.iter().find_map(|b| left_out(b, &cwd)) {
        anyhow::bail!(line);
    }
    let plan = match detect::plan(&cwd, &args.branches, args.into.as_deref(), &args.checks) {
        // Outside a repository the demo still works.
        Err(e) if e.to_string() == detect::NOT_A_REPOSITORY => anyhow::bail!(
            "not a git repository: run {0} inside one, or see it work: {0} demo",
            run_as()
        ),
        plan => plan?,
    };
    Ok(check(&plan, &args)?.0)
}

/// The check `plan` describes: its exit code and its verdict's kind, `None`
/// when there was nothing to check.
fn check(plan: &Plan, args: &CheckArgs) -> Result<(i32, Option<&'static str>)> {
    if plan.branches.is_empty() {
        nothing_to_check(plan, args.json)?;
        return Ok((0, None));
    }
    if !args.json {
        eprintln!("{}", progress(plan));
    }
    let outcome = land(&options(plan, args))?;
    let verdict = verdict(&outcome, &again());
    let notes = notes(plan);
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "decision": outcome.decision(),
                "exit_code": outcome.exit_code(),
                "verdict": verdict,
                "notes": uncommitted_notes(plan).into_iter().chain(notes).collect::<Vec<_>>(),
                "target": plan.target,
                "branches": branches_json(plan),
                "checks": plan.checks,
                "check_source": plan.check_source,
                "insights": outcome.insights,
                "combined_ref": outcome.combined_ref,
                "receipt_path": outcome.receipt_path,
                "receipt": outcome.receipt,
            }))?
        );
    } else if args.markdown {
        print!("{}", render_markdown_with(&outcome, args.verbose));
    } else {
        if args.verbose {
            print!("{}\n{}\n", found(plan), render_text(&outcome));
        }
        print!("{verdict}{}", note_lines(&notes));
    }
    Ok((outcome.exit_code(), Some(verdict.kind)))
}

fn options(plan: &Plan, args: &CheckArgs) -> LandOptions {
    let mut opts = LandOptions::new(
        plan.root.clone(),
        plan.branches.iter().map(|b| b.input()).collect(),
    );
    opts.into = Some(plan.target.clone());
    opts.base = args.base.clone();
    opts.checks = plan.checks.clone();
    opts.per_input = if args.per_input {
        PerInput::Always
    } else {
        PerInput::OnFailure
    };
    opts.apply = args.apply;
    opts.strict = args.strict;
    opts.keep_ref = args.verbose;
    opts.check_timeout = Duration::from_secs(args.timeout.max(1));
    opts
}

/// The `timeout` the Claude Code and Codex hook configs give `pytxo hook`.
pub const HOOK_TIMEOUT_SECS: u64 = 900;

/// `pytxo hook`: the check from a Claude Code or Codex `Stop` hook. Reads the
/// hook's JSON on stdin and answers with JSON on stdout, or with nothing when
/// there is nothing new to say. It never fails the agent's turn, and answers
/// before the hook's `timeout` (seconds) runs out.
pub fn hook(block: bool, timeout: u64) -> Result<i32> {
    let started = Instant::now();
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let event: serde_json::Value = serde_json::from_str(&input).unwrap_or_default();
    let timeout = Duration::from_secs(timeout.max(1));
    let reply = match hook_reply(&event, block, started, timeout) {
        Ok(None) => return Ok(0),
        Ok(Some(reply)) => reply,
        Err(e) => {
            json!({ "systemMessage": format!("Pytxo could not check the agent branches: {e:#}") })
        }
    };
    println!("{reply}");
    Ok(0)
}

fn hook_reply(
    event: &serde_json::Value,
    block: bool,
    started: Instant,
    timeout: Duration,
) -> Result<Option<serde_json::Value>> {
    // Claude Code fires Stop while its background agents still run; the last
    // one to finish gets the check.
    let busy = event["background_tasks"].as_array().is_some_and(|tasks| {
        tasks.iter().any(|t| {
            matches!(
                t["type"].as_str(),
                Some("subagent" | "teammate" | "workflow")
            )
        })
    });
    if busy {
        return Ok(None);
    }
    let cwd = match event["cwd"].as_str() {
        Some(cwd) => PathBuf::from(cwd),
        None => std::env::current_dir()?,
    };
    // Not a repository: nothing to combine.
    let Ok(receipts) = pytxo_land::repo_receipts_dir(&cwd) else {
        return Ok(None);
    };
    // Same branches, same tips, same tests: the last verdict still stands.
    let memo = receipts.with_file_name("hook-last");
    let said = |text: &str| std::fs::read_to_string(&memo).is_ok_and(|last| last == text);
    let plan = match detect::plan(&cwd, &[], None, &[]) {
        Ok(plan) => plan,
        // An error every check would hit, such as a pytxo.into naming no
        // branch: said once, not on every stop.
        Err(e) => {
            let error = format!("{e:#}");
            if said(&error) {
                return Ok(None);
            }
            let _ = std::fs::create_dir_all(&receipts);
            let _ = std::fs::write(&memo, &error);
            return Err(e);
        }
    };
    // One agent alone: nothing to combine.
    if plan.branches.len() < 2 {
        // Unless committing the work nobody committed yet makes two: said once.
        if plan.branches.len() + plan.uncommitted_only.len() < 2 {
            return Ok(None);
        }
        let note = uncommitted_only(&plan.uncommitted_only);
        if said(&note) {
            return Ok(None);
        }
        let _ = std::fs::create_dir_all(&receipts);
        let _ = std::fs::write(&memo, &note);
        return Ok(Some(json!({
            "systemMessage": format!("Pytxo did not check the agent branches: {note}")
        })));
    }
    // Parallel sessions stop at the same time: one check per repository.
    let Some(_lock) = HookLock::take(&memo.with_file_name("hook.lock"), timeout)? else {
        return Ok(None);
    };
    let fingerprint = plan.fingerprint();
    if said(&fingerprint) {
        return Ok(None);
    }
    let mut opts = options(
        &plan,
        &CheckArgs {
            timeout: 900,
            ..CheckArgs::default()
        },
    );
    // Room to stop the tests, remove the worktree and answer before the
    // agent tool kills the hook: a minute, or half of a short timeout. A
    // timeout past what the clock can count means no deadline.
    opts.deadline = started.checked_add(timeout - Duration::from_secs(60).min(timeout / 2));
    // A verdict or an error, said once for the same branches, tips and tests.
    let outcome = land(&opts);
    let _ = std::fs::write(&memo, &fingerprint);
    let outcome = outcome?;
    if outcome.out_of_time {
        return Ok(Some(json!({ "systemMessage": format!(
            "Pytxo could not finish checking the agent branches before the hook's time ran out. Run {} in a terminal to check them.",
            run_as()
        ) })));
    }
    let verdict = verdict(&outcome, "Pytxo checks again when an agent stops");
    let notes: Vec<String> = uncommitted_notes(&plan)
        .into_iter()
        .chain(notes(&plan))
        .collect();
    let text = format!(
        "Pytxo checked the agent branches together.\n{verdict}{}",
        note_lines(&notes)
    );
    Ok(Some(if block && outcome.exit_code() == 2 {
        json!({ "decision": "block", "reason": text })
    } else {
        json!({ "systemMessage": text })
    }))
}

/// `hook.lock` in the git common dir, which every linked worktree shares.
/// Dropping it removes it, on every way out of the hook; a lock a killed hook
/// left behind is stale once it is older than the hook timeout.
struct HookLock {
    path: PathBuf,
    token: String,
}

impl HookLock {
    /// `None` while another hook holds a lock younger than `stale_after`.
    fn take(path: &Path, stale_after: Duration) -> Result<Option<Self>> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let token = format!("{} {nanos}", std::process::id());
        for _ in 0..2 {
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(mut file) => {
                    file.write_all(token.as_bytes())?;
                    return Ok(Some(Self {
                        path: path.to_path_buf(),
                        token,
                    }));
                }
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    let age = std::fs::metadata(path)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok());
                    if age.is_some_and(|age| age < stale_after) {
                        return Ok(None);
                    }
                    // ponytail: two hooks taking over one stale lock at the
                    // same moment can both run, as every hook could before.
                    let _ = std::fs::remove_file(path);
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(None)
    }
}

impl Drop for HookLock {
    fn drop(&mut self) {
        // Only its own: a hook that outlived its timeout leaves its successor's.
        if std::fs::read_to_string(&self.path).is_ok_and(|t| t == self.token) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// `pytxo demo`: the check on a throwaway repository where two agent
/// branches pass their tests alone and fail together.
pub fn demo(keep: bool) -> Result<i32> {
    eprintln!("Building a throwaway repo where two agents changed the same shop code...");
    let dir = pytxo_land::bench::demo_repo()?;
    for (name, change) in pytxo_land::bench::DEMO_BRANCHES {
        eprintln!("  {name:<20} {change}");
    }
    // Its own branches, target and test, so no PYTXO_* variable and no
    // pytxo.* setting of the user's reaches it.
    let git = pytxo_land::Git::user(dir.path());
    let (test, file) = detect::test_command(&git, "refs/heads/main")
        .context("the demo repo declares no test command")?;
    let branches = pytxo_land::bench::DEMO_BRANCHES.map(|(name, _)| name.to_string());
    let mut plan = detect::plan(dir.path(), &branches, Some("main"), &[test])?;
    plan.check_source = Some(file);
    let kind = check(
        &plan,
        &CheckArgs {
            timeout: 120,
            ..CheckArgs::default()
        },
    )?
    .1;
    if keep {
        eprintln!("\nThe demo repo is at {}", dir.keep().display());
    }
    // The demo worked when it caught the break it was built for.
    if kind != Some("together") {
        anyhow::bail!(
            "the demo should show two branches that pass alone and fail together, and it did not"
        );
    }
    if !keep {
        eprintln!(
            "\nThat repo was temporary and is gone. Now run {} in a repo where agents work.",
            run_as()
        );
    }
    Ok(0)
}

fn nothing_to_check(plan: &Plan, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "decision": "nothing_to_check",
                "exit_code": 0,
                "target": plan.target,
                "branches": [],
                "patterns": plan.patterns,
                "old": plan.old,
                "unrelated": plan.unrelated,
                "uncommitted_only": plan.uncommitted_only,
            }))?
        );
        return Ok(());
    }
    let mut line = format!(
        "No agent branches to check against {} (looked for {}).",
        plan.target,
        looked_for(&plan.patterns)
    );
    if !plan.uncommitted_only.is_empty() {
        line.push_str(&format!(" {}", uncommitted_only(&plan.uncommitted_only)));
    }
    if plan.unrelated > 0 {
        line.push_str(&format!(
            " {} sharing no history with {} left out.",
            plan.unrelated, plan.target
        ));
    }
    if plan.old > 0 {
        line.push_str(&format!(
            // Not "unmerged": the age test runs first, so merged ones count too.
            " {} with no commit in {} days not looked at (PYTXO_DAYS=0 looks at them).",
            plan.old, plan.max_age_days
        ));
    }
    println!(
        "{line} If your agents push to GitHub, run git fetch first. See it work: {} demo",
        run_as()
    );
    Ok(())
}

/// "a", "a and b", "a, b and c".
fn and_list(names: &[&str]) -> String {
    match names {
        [init @ .., last] if !init.is_empty() => format!("{} and {last}", init.join(", ")),
        _ => names.join(""),
    }
}

/// "claude/*, codex/* and linked worktrees".
fn looked_for(patterns: &[String]) -> String {
    let names: Vec<&str> = patterns
        .iter()
        .map(|p| {
            if p == detect::WORKTREES {
                "linked worktrees"
            } else {
                p
            }
        })
        .collect();
    and_list(&names)
}

/// Worktrees with work only in uncommitted changes, which no check can see.
fn uncommitted_only(names: &[String]) -> String {
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    match names.as_slice() {
        [one] => format!("{one} has only uncommitted changes; commit them to check them."),
        _ => format!(
            "{} have only uncommitted changes; commit them to check them together.",
            and_list(&names)
        ),
    }
}

/// Work the check cannot see: the terminal says it before the tests run.
fn uncommitted_notes(plan: &Plan) -> Vec<String> {
    let mut notes: Vec<String> = plan
        .branches
        .iter()
        .filter(|b| b.uncommitted)
        .map(|b| {
            format!(
                "{} has uncommitted changes; only its commits are checked.",
                b.name()
            )
        })
        .collect();
    if !plan.uncommitted_only.is_empty() {
        notes.push(uncommitted_only(&plan.uncommitted_only));
    }
    notes
}

/// What the verdict leaves unsaid about what it checked against.
fn notes(plan: &Plan) -> Vec<String> {
    let mut notes = Vec::new();
    if plan.behind > 0 {
        notes.push(format!(
            "{0} is {1} commit{2} behind origin/{0}; pull it to check against {3} too.",
            plan.target,
            plan.behind,
            if plan.behind == 1 { "" } else { "s" },
            if plan.behind == 1 { "that" } else { "those" }
        ));
    }
    if plan.left_out > 0 {
        notes.push(format!(
            "{} older agent branch{} left out (at most {} at once).",
            plan.left_out,
            if plan.left_out == 1 { "" } else { "es" },
            detect::MAX_BRANCHES
        ));
    }
    notes
}

/// Notes as lines under the verdict.
fn note_lines(notes: &[String]) -> String {
    notes
        .iter()
        .map(|n| format!("{:<9}{n}\n", "note"))
        .collect()
}

/// One line on stderr before the tests run, so a long test run is not silence.
fn progress(plan: &Plan) -> String {
    let names: Vec<&str> = plan.branches.iter().map(|b| b.name()).collect();
    let names = match names.as_slice() {
        [one] => one.to_string(),
        // As many as the verdict names.
        all if all.len() <= 4 => and_list(all),
        _ => format!("{} branches", names.len()),
    };
    let tests = match (plan.checks.as_slice(), &plan.check_source) {
        ([], _) => " (no test command found)".to_string(),
        (checks, Some(source)) => format!(" with {} (from {source})", checks.join(" && ")),
        (checks, None) => format!(" with {}", checks.join(" && ")),
    };
    let mut line = format!("Checking {names} against {}{tests}...", plan.target);
    for note in uncommitted_notes(plan) {
        line.push_str(&format!("\n  {note}"));
    }
    line
}

/// What detection found, for `--verbose`.
fn found(plan: &Plan) -> String {
    let mut s = format!(
        "target    {}\nbranches  {}\n",
        plan.target,
        if plan.detected {
            format!("found by {}", looked_for(&plan.patterns))
        } else {
            "given on the command line".into()
        }
    );
    for b in &plan.branches {
        s.push_str(&format!(
            "  {:<28} {}",
            b.name(),
            &b.commit[..12.min(b.commit.len())]
        ));
        if let Some(w) = &b.worktree {
            s.push_str(&format!("  worktree {}", w.display()));
        }
        s.push('\n');
    }
    if plan.old > 0 {
        s.push_str(&format!(
            "  ({} with no commit in {} days not looked at)\n",
            plan.old, plan.max_age_days
        ));
    }
    if plan.unrelated > 0 {
        s.push_str(&format!(
            "  ({} sharing no history with {} left out)\n",
            plan.unrelated, plan.target
        ));
    }
    if plan.left_out > 0 {
        s.push_str(&format!(
            "  ({} older left out: at most {} at once)\n",
            plan.left_out,
            detect::MAX_BRANCHES
        ));
    }
    s.push_str(&format!(
        "tests     {}{}\n",
        if plan.checks.is_empty() {
            "none found".to_string()
        } else {
            plan.checks.join(" && ")
        },
        plan.check_source
            .as_deref()
            .map(|s| format!(" (from {s})"))
            .unwrap_or_default()
    ));
    s
}

fn branches_json(plan: &Plan) -> serde_json::Value {
    plan.branches
        .iter()
        .map(|b| {
            let mut branch = json!({
                "name": b.name(),
                "reference": b.reference,
                "commit": b.commit,
                "worktree": b.worktree,
                "uncommitted": b.uncommitted,
            });
            // `claude:claude/a` on the command line, or a detached worktree's.
            if let Some(label) = &b.label {
                branch["label"] = json!(label);
            }
            branch
        })
        .collect()
}

/// How the user started Pytxo: the npm launcher sets `PYTXO_RUN_AS`
/// (`npx pytxo` or `pytxo`).
fn run_as() -> String {
    std::env::var("PYTXO_RUN_AS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "pytxo".into())
}

/// What to run again: the command line that got here, when it can be pasted.
fn again() -> String {
    rerun_command(&std::env::args().skip(1).collect::<Vec<_>>()).map_or_else(
        || "run the same command again".into(),
        |c| format!("run {c} again"),
    )
}

/// `args` as one line to paste, `None` when a quote or a `$` in one would
/// change what the shell runs.
fn rerun_command(args: &[String]) -> Option<String> {
    let mut line = run_as();
    // The demo repo is gone by then; point at the real thing.
    if args.first().is_none_or(|a| a == "demo") {
        return Some(line);
    }
    for a in args {
        if a.contains(['"', '\'', '`', '$']) {
            return None;
        }
        let plain = !a.is_empty()
            && a.chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_./:=@,+".contains(c));
        line.push_str(&if plain {
            format!(" {a}")
        } else {
            format!(" \"{a}\"")
        });
    }
    Some(line)
}

pub fn receipts_dir(repo: Option<PathBuf>) -> Result<PathBuf> {
    let repo = repo.unwrap_or(std::env::current_dir()?);
    pytxo_land::repo_receipts_dir(&repo)
}

pub fn receipts_list(repo: Option<PathBuf>, json: bool) -> Result<()> {
    let dir = receipts_dir(repo)?;
    let receipts = pytxo_core::list_receipts(&dir);
    if json {
        let rows: Vec<_> = receipts.iter().map(|(_, r)| &r.predicate).collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    if receipts.is_empty() {
        println!("No receipts yet. Every check writes one.");
        return Ok(());
    }
    for (_, r) in receipts {
        let p = &r.predicate;
        println!(
            "{}  {:<5}  {:<7}  {:<7}  {} input(s), {} file(s)  {}",
            p.id,
            p.kind,
            p.decision,
            p.mode,
            p.inputs.len(),
            p.files_changed,
            p.created_at
        );
    }
    Ok(())
}

pub fn receipts_show(repo: Option<PathBuf>, id: &str) -> Result<()> {
    let dir = receipts_dir(repo)?;
    let (path, receipt) = pytxo_core::find_receipt(&dir, id)
        .with_context(|| format!("no single receipt matches {id}"))?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    eprintln!(
        "{}  digest {}",
        path.display(),
        if receipt.digest_matches() {
            "matches"
        } else {
            "DOES NOT MATCH (edited after writing)"
        }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rerun_hint_pastes_or_says_run_it_again() {
        let line = |args: &[&str]| {
            rerun_command(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
                .map(|c| c.replacen(&run_as(), "pytxo", 1))
        };
        assert_eq!(line(&[]).as_deref(), Some("pytxo"));
        assert_eq!(line(&["demo"]).as_deref(), Some("pytxo"));
        assert_eq!(
            line(&["-c", "npm test", "agent/*", "--into", "main"]).as_deref(),
            Some(r#"pytxo -c "npm test" "agent/*" --into main"#)
        );
        // A quote or a `$` in one changes what the shell runs.
        assert_eq!(line(&["-c", r#"echo "hi""#]), None);
        assert_eq!(line(&["-c", "echo 'hi'"]), None);
        assert_eq!(line(&["-c", "echo $HOME"]), None);
    }

    /// The hook answers inside the timeout its configs declare.
    #[test]
    fn hook_configs_declare_the_timeout_the_hook_answers_inside() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../integrations");
        for config in ["claude-code/hooks/hooks.json", "codex/hooks.json"] {
            let text = std::fs::read_to_string(root.join(config)).unwrap();
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            let hook = &v["hooks"]["Stop"][0]["hooks"][0];
            assert_eq!(hook["timeout"], HOOK_TIMEOUT_SECS, "{config}");
            // The command says it too, so whoever lowers one sees the other.
            let command = hook["command"].as_str().unwrap();
            assert!(
                command.ends_with(&format!(" hook --timeout {HOOK_TIMEOUT_SECS}")),
                "{config}: {command}"
            );
        }
    }
}
