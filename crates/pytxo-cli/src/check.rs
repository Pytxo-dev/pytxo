//! `pytxo` / `pytxo check`: find the agent branches and the test command,
//! check the branches together, print the verdict. Also `pytxo demo`,
//! `pytxo hook` and `pytxo receipts`.

use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

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
    /// Show what was found, every test run, the receipt and its digest
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

/// Runs the check and returns the exit code: 0 ready or nothing to check,
/// 2 blocked, 3 stale.
pub fn run(args: CheckArgs) -> Result<i32> {
    let cwd = match &args.repo {
        Some(repo) => repo.clone(),
        None => std::env::current_dir()?,
    };
    let plan = detect::plan(&cwd, &args.branches, args.into.as_deref(), &args.checks)?;
    if plan.branches.is_empty() {
        nothing_to_check(&plan, args.json)?;
        return Ok(0);
    }
    if !args.json {
        eprintln!("{}", progress(&plan));
    }
    let outcome = land(&options(&plan, &args))?;
    let verdict = verdict(&outcome, &format!("run {} again", rerun_command()));
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "decision": outcome.decision(),
                "exit_code": outcome.exit_code(),
                "verdict": verdict,
                "target": plan.target,
                "branches": branches_json(&plan),
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
            print!("{}\n{}\n", found(&plan), render_text(&outcome));
        }
        print!("{verdict}");
    }
    Ok(outcome.exit_code())
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
    // Not a repository, or one agent alone: nothing to combine.
    let Ok(plan) = detect::plan(&cwd, &[], None, &[]) else {
        return Ok(None);
    };
    if plan.branches.len() < 2 {
        return Ok(None);
    }
    // Same branches, same tips, same tests: the last verdict still stands.
    let memo = pytxo_land::repo_receipts_dir(&plan.root)?.with_file_name("hook-last");
    let fingerprint = plan.fingerprint();
    if std::fs::read_to_string(&memo).is_ok_and(|last| last == fingerprint) {
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
    // agent tool kills the hook: a minute, or half of a short timeout.
    opts.deadline = Some(started + timeout - Duration::from_secs(60).min(timeout / 2));
    let outcome = land(&opts)?;
    let _ = std::fs::write(&memo, &fingerprint);
    if outcome.out_of_time {
        return Ok(Some(json!({ "systemMessage": format!(
            "Pytxo could not finish checking the agent branches before the hook's time ran out. Run {} in a terminal to check them.",
            run_as()
        ) })));
    }
    let verdict = verdict(&outcome, "Pytxo checks again when an agent stops");
    let text = format!("Pytxo checked the agent branches together.\n{verdict}");
    Ok(Some(if block && outcome.exit_code() == 2 {
        json!({ "decision": "block", "reason": text })
    } else {
        json!({ "systemMessage": text })
    }))
}

/// `pytxo demo`: the check on a throwaway repository where two agent
/// branches pass their tests alone and fail together.
pub fn demo(keep: bool) -> Result<i32> {
    eprintln!("Building a throwaway repo where two agents changed the same shop code...");
    let dir = pytxo_land::bench::demo_repo()?;
    for (name, change) in pytxo_land::bench::DEMO_BRANCHES {
        eprintln!("  {name:<20} {change}");
    }
    let code = run(CheckArgs {
        repo: Some(dir.path().to_path_buf()),
        timeout: 120,
        ..CheckArgs::default()
    })?;
    let run_as = run_as();
    if keep {
        eprintln!("\nThe demo repo is at {}", dir.keep().display());
    } else {
        eprintln!(
            "\nThat repo was temporary and is gone. Now run {run_as} in a repo where agents work."
        );
    }
    // The demo worked when it caught the break.
    Ok(if code == 2 { 0 } else { 1.max(code) })
}

fn nothing_to_check(plan: &Plan, json: bool) -> Result<()> {
    let skipped = match plan.old {
        0 => String::new(),
        n => format!(
            " {n} older than {} days left out (PYTXO_DAYS=0 includes them).",
            plan.max_age_days
        ),
    };
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
            }))?
        );
    } else {
        println!(
            "No agent branches to check against {} (looked for {}).{skipped} See it work: {} demo",
            plan.target,
            looked_for(&plan.patterns),
            run_as()
        );
    }
    Ok(())
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
    match names.as_slice() {
        [init @ .., last] if !init.is_empty() => format!("{} and {last}", init.join(", ")),
        _ => names.join(""),
    }
}

/// One line on stderr before the tests run, so a long test run is not silence.
fn progress(plan: &Plan) -> String {
    let names: Vec<&str> = plan.branches.iter().map(|b| b.name()).collect();
    let names = match names.as_slice() {
        [one] => one.to_string(),
        [init @ .., last] if names.len() <= 4 => format!("{} and {last}", init.join(", ")),
        _ => format!("{} branches", names.len()),
    };
    let tests = match (plan.checks.as_slice(), &plan.check_source) {
        ([], _) => " (no test command found)".to_string(),
        (checks, Some(source)) => format!(" with {} (from {source})", checks.join(" && ")),
        (checks, None) => format!(" with {}", checks.join(" && ")),
    };
    let mut line = format!("Checking {names} against {}{tests}...", plan.target);
    for b in plan.branches.iter().filter(|b| b.uncommitted) {
        line.push_str(&format!(
            "\n  {} has uncommitted changes; only its commits are checked.",
            b.name()
        ));
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
            "  ({} older than {} days left out)\n",
            plan.old, plan.max_age_days
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
            json!({
                "name": b.name(),
                "reference": b.reference,
                "commit": b.commit,
                "worktree": b.worktree,
                "uncommitted": b.uncommitted,
            })
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

/// The command line that got here, to tell the user what to run again.
fn rerun_command() -> String {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .map(|a| {
            if a.contains(' ') {
                format!("\"{a}\"")
            } else {
                a
            }
        })
        .collect();
    match args.first().map(String::as_str) {
        // The demo repo is gone by then; point at the real thing.
        None | Some("demo") => run_as(),
        Some(_) => format!("{} {}", run_as(), args.join(" ")),
    }
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

    /// The hook answers inside the timeout its configs declare.
    #[test]
    fn hook_configs_declare_the_timeout_the_hook_answers_inside() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../integrations");
        for config in ["claude-code/hooks/hooks.json", "codex/hooks.json"] {
            let text = std::fs::read_to_string(root.join(config)).unwrap();
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            let hook = &v["hooks"]["Stop"][0]["hooks"][0];
            assert_eq!(hook["timeout"], HOOK_TIMEOUT_SECS, "{config}");
        }
    }
}
