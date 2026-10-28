//! Terminal and pull-request (Markdown) summaries of a landing outcome.

use std::fmt::Write as _;

use pytxo_core::{ReceiptCheck, ReceiptPredicate};
use serde::Serialize;

use crate::{short, LandOutcome};

fn seconds(ms: u64) -> String {
    format!("{:.1}s", ms as f64 / 1000.0)
}

fn input_name(reference: &str, agent: Option<&str>) -> String {
    agent.map_or_else(|| reference.to_string(), |a| format!("{a}:{reference}"))
}

/// A check's scope as shown; the re-run of a combined check is run 2.
fn scope_label(c: &ReceiptCheck) -> String {
    if c.rerun {
        format!("{}, run 2", c.scope)
    } else {
        c.scope.clone()
    }
}

/// A failed combined run whose re-run passed.
fn passed_when_rerun(p: &ReceiptPredicate, c: &ReceiptCheck) -> bool {
    !c.passed
        && !c.rerun
        && c.scope == "combined"
        && p.checks
            .iter()
            .any(|r| r.rerun && r.passed && r.command == c.command)
}

/// Inputs that conflicted and so are absent from the tree the combined checks
/// ran on; without this a "combined pass" reads as covering every input.
fn left_out_of_combined(outcome: &LandOutcome) -> Option<String> {
    let p = &outcome.receipt.predicate;
    if !p.checks.iter().any(|c| c.scope == "combined") {
        return None;
    }
    let names: Vec<String> = p
        .inputs
        .iter()
        .filter(|i| i.status == "conflict")
        .map(|i| input_name(&i.reference, i.agent.as_deref()))
        .collect();
    (!names.is_empty()).then(|| names.join(", "))
}

/// The lines of a failed check's output most likely to say why it failed:
/// error-looking lines first, else the last few.
fn why_lines(tail: &[String]) -> Vec<String> {
    const MARKERS: [&str; 7] = [
        "error",
        "fail",
        "not recognized",
        "not found",
        "panicked",
        "traceback",
        "assert",
    ];
    let trimmed = || tail.iter().map(|l| l.trim()).filter(|l| !l.is_empty());
    let mut lines: Vec<&str> = trimmed()
        .filter(|l| {
            let l = l.to_lowercase();
            MARKERS.iter().any(|m| l.contains(m))
        })
        .take(3)
        .collect();
    if lines.is_empty() {
        lines = trimmed().collect();
        lines.drain(..lines.len().saturating_sub(3));
    }
    lines
        .into_iter()
        .map(|l| l.chars().take(160).collect())
        .collect()
}

/// What to do about a decision that did not land; `None` when nothing is needed.
fn next_step(outcome: &LandOutcome) -> Option<&'static str> {
    let p = &outcome.receipt.predicate;
    match p.decision.as_str() {
        "blocked" if p.inputs.iter().any(|i| i.status == "conflict") => Some(
            "merge or rebase the conflicting branch onto the ones before it, resolve the conflict, then run the same command again",
        ),
        "blocked"
            if p.checks
                .iter()
                .filter(|c| !c.passed)
                .all(|c| passed_when_rerun(p, c))
                && p.checks.iter().any(|c| !c.passed) =>
        {
            Some("the combined check is flaky: make it reliable, or run the same command again")
        }
        "blocked" if p.checks.iter().any(|c| !c.passed) => Some(
            "fix the failure on one of the branches (or fix the check), then run the same command again",
        ),
        "blocked" => Some("review the shared symbols, or run again without --strict"),
        "stale" => Some("the target moved: run the same command again on the new tip"),
        _ => None,
    }
}

pub fn render_text(outcome: &LandOutcome) -> String {
    let p = &outcome.receipt.predicate;
    let mut s = String::new();
    let _ = writeln!(s, "pytxo land  {}  {}", p.id, p.mode);
    let _ = writeln!(
        s,
        "target {} @ {} · {} input{} · {} file{} changed",
        p.target.as_deref().unwrap_or("HEAD"),
        short(&p.base),
        p.inputs.len(),
        if p.inputs.len() == 1 { "" } else { "s" },
        p.files_changed,
        if p.files_changed == 1 { "" } else { "s" },
    );
    let _ = writeln!(s, "\ninputs");
    for i in &p.inputs {
        let detail = match i.status.as_str() {
            "conflict" if !i.conflict_files.is_empty() => {
                format!("CONFLICT ({})", i.conflict_files.join(", "))
            }
            "conflict" => "CONFLICT".into(),
            other => other.to_string(),
        };
        let _ = writeln!(
            s,
            "  {:<32} {:>3} file{}  {}",
            input_name(&i.reference, i.agent.as_deref()),
            i.files.len(),
            if i.files.len() == 1 { " " } else { "s" },
            detail
        );
    }
    if !p.shared_files.is_empty() {
        let _ = writeln!(s, "\nshared files");
        for f in &p.shared_files {
            let note = if f.symbol_collisions.is_empty() {
                "no shared symbol".to_string()
            } else {
                format!(
                    "same symbol: {}",
                    f.symbol_collisions
                        .iter()
                        .map(|c| c.symbol.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            let _ = writeln!(s, "  {}  [{}]  {}", f.path, f.inputs.join(", "), note);
        }
    }
    if !p.checks.is_empty() {
        let _ = writeln!(s, "\nchecks");
        for c in &p.checks {
            let verdict = if c.passed {
                "pass".to_string()
            } else if c.timed_out {
                "TIMEOUT".to_string()
            } else {
                format!(
                    "FAIL exit {}",
                    c.exit_code.map_or("?".into(), |e| e.to_string())
                )
            };
            let _ = writeln!(
                s,
                "  {:<28} {:<14} {}  {}",
                scope_label(c),
                verdict,
                seconds(c.duration_ms),
                c.command
            );
            if !c.passed {
                for line in why_lines(&c.output_tail) {
                    let _ = writeln!(s, "      {line}");
                }
            }
        }
        if let Some(names) = left_out_of_combined(outcome) {
            let _ = writeln!(s, "  combined leaves out {names} (conflict)");
        }
    }
    for insight in &outcome.insights {
        let _ = writeln!(s, "\n! {insight}");
    }
    let _ = writeln!(s, "\ndecision  {}", p.decision.to_uppercase());
    for r in &p.reasons {
        let _ = writeln!(s, "  - {r}");
    }
    if let Some(next) = next_step(outcome) {
        let _ = writeln!(s, "next      {next}");
    }
    if p.checks.iter().any(|c| !c.passed) {
        let _ = writeln!(s, "output    pytxo receipts show {}", p.id);
    }
    if let Some(r) = &outcome.combined_ref {
        let _ = writeln!(s, "inspect   git log --oneline --graph {r}");
    }
    let _ = writeln!(s, "receipt   {}", outcome.receipt_path.display());
    if p.mode == "shadow" && p.decision == "ready" && p.files_changed > 0 {
        let _ = writeln!(s, "land it   rerun with --apply");
    }
    s
}

/// A decision in three plain lines: what happened, why, and what to do next.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Verdict {
    /// ready, no_tests, nothing, landed, conflict, together, alone, failed,
    /// timeout, flaky, collision, refused or stale.
    pub kind: &'static str,
    /// The word a person reads first: READY, BLOCKED, FLAKY, STALE...
    pub label: &'static str,
    pub what: String,
    pub why: String,
    pub next: String,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<9}{}", self.label, self.what)?;
        writeln!(f, "{:<9}{}", "why", self.why)?;
        writeln!(f, "{:<9}{}", "next", self.next)
    }
}

/// A branch as a person names it: the label of a detached worktree's
/// commit, otherwise the branch.
fn plain_name(i: &pytxo_core::ReceiptInput) -> String {
    let is_commit = i.reference.len() >= 7 && i.reference.chars().all(|c| c.is_ascii_hexdigit());
    match &i.agent {
        Some(label) if is_commit => label.clone(),
        _ => i.reference.clone(),
    }
}

/// "a", "a and b", "a, b and c", or "4 branches".
fn and_list(names: &[String]) -> String {
    match names {
        [] => "no branches".into(),
        [one] => one.clone(),
        [init @ .., last] if names.len() <= 3 => format!("{} and {last}", init.join(", ")),
        _ => format!("{} branches", names.len()),
    }
}

/// The decision in plain words. `again` finishes "fix it, then ...", for
/// example "run npx pytxo again" or "push again".
pub fn verdict(outcome: &LandOutcome, again: &str) -> Verdict {
    verdict_for(outcome, again, false)
}

/// The longest run of `c` in `text`.
fn longest_run(text: &str, c: char) -> usize {
    text.split(|x| x != c).map(str::len).max().unwrap_or(0)
}

/// `text` as a Markdown code span nothing in it can close, so test output in
/// a pull-request comment renders no mention, link or HTML.
fn code_span(text: &str) -> String {
    let ticks = "`".repeat(longest_run(text, '`') + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{ticks}{pad}{text}{pad}{ticks}")
}

/// `pr`: for a pull-request comment, where test output shows only as code.
fn verdict_for(outcome: &LandOutcome, again: &str, pr: bool) -> Verdict {
    let p = &outcome.receipt.predicate;
    let target = p.target.as_deref().unwrap_or("HEAD");
    let all: Vec<String> = p.inputs.iter().map(plain_name).collect();
    let names = and_list(&all);
    let several = all.len() > 1;
    let v = |kind, label, what: String, why: String, next: String| Verdict {
        kind,
        label,
        what,
        why,
        next,
    };
    let first_line = |c: &ReceiptCheck| {
        let line = why_lines(&c.output_tail).into_iter().next()?;
        Some(if pr { code_span(&line) } else { line })
    };
    let failed_combined = p
        .checks
        .iter()
        .find(|c| c.scope == "combined" && !c.passed && !c.rerun);

    match p.decision.as_str() {
        "stale" => v(
            "stale",
            "STALE",
            format!("{target} moved while the check ran."),
            format!("the result describes an old tip of {target}, so it does not count."),
            format!("{again}."),
        ),
        "landed" => {
            let how = match p.checks.iter().find(|c| c.scope == "combined") {
                Some(c) => format!("{} passed on the combined change", c.command),
                None => "no tests ran".into(),
            };
            v(
                "landed",
                "LANDED",
                format!("{names} {} on {target} now.", if several { "are" } else { "is" }),
                format!("they merged cleanly and {how}."),
                format!("push {target} when you are ready."),
            )
        }
        "blocked" => {
            if let Some(c) = p.inputs.iter().find(|i| i.status == "conflict") {
                let earlier: Vec<String> = p
                    .inputs
                    .iter()
                    .take_while(|i| i.reference != c.reference)
                    .filter(|i| {
                        c.conflict_files.is_empty()
                            || i.files.iter().any(|f| c.conflict_files.contains(f))
                    })
                    .map(plain_name)
                    .collect();
                let others = if earlier.is_empty() {
                    "the branches before it".to_string()
                } else {
                    and_list(&earlier)
                };
                let files = match c.conflict_files.as_slice() {
                    [] => "the same lines".to_string(),
                    [a] => a.clone(),
                    [a, b] => format!("{a} and {b}"),
                    [a, b, rest @ ..] => format!("{a}, {b} and {} more", rest.len()),
                };
                let name = plain_name(c);
                return v(
                    "conflict",
                    "BLOCKED",
                    format!("{name} conflicts with {others}."),
                    format!("git cannot merge them: {files} changed on both sides."),
                    format!("rebase {name} onto {others} (or ask its agent to), then {again}."),
                );
            }
            if let Some(c) = failed_combined {
                let cmd = &c.command;
                let rerun = p
                    .checks
                    .iter()
                    .find(|r| r.rerun && r.command == c.command);
                if rerun.is_some_and(|r| r.passed) {
                    return v(
                        "flaky",
                        "FLAKY",
                        format!("{cmd} failed on the combined change, then passed when run again."),
                        format!("a flaky test cannot say whether {names} work together."),
                        format!("make that test reliable, then {again}."),
                    );
                }
                if c.timed_out {
                    return v(
                        "timeout",
                        "BLOCKED",
                        format!(
                            "{cmd} did not finish on the combined change ({}).",
                            seconds(c.duration_ms)
                        ),
                        "a test command that waits for input or watches files never exits."
                            .into(),
                        format!("make {cmd} exit on its own, or raise --timeout, then {again}."),
                    );
                }
                let solo: Vec<&ReceiptCheck> = p
                    .checks
                    .iter()
                    .filter(|s| s.scope != "combined" && s.command == c.command)
                    .collect();
                let why = |check: &ReceiptCheck, place: &str| match first_line(check) {
                    Some(line) => format!("{cmd} fails {place}: {line}"),
                    None => format!(
                        "{cmd} exits {} {place}.",
                        check.exit_code.map_or("with an error".into(), |e| e.to_string())
                    ),
                };
                if solo.is_empty() {
                    return v(
                        "failed",
                        "BLOCKED",
                        format!("{names} fail {cmd} when combined."),
                        why(c, "on the combined change"),
                        format!("to see whether each branch passes alone, run it with --per-input. Fix the failure, then {again}."),
                    );
                }
                let failing: Vec<String> = solo
                    .iter()
                    .filter(|s| !s.passed)
                    .filter_map(|s| s.scope.strip_prefix("input:"))
                    .map(|r| {
                        p.inputs
                            .iter()
                            .find(|i| i.reference == r)
                            .map_or_else(|| r.to_string(), plain_name)
                    })
                    .collect();
                if failing.is_empty() {
                    return v(
                        "together",
                        "BLOCKED",
                        format!("{names} pass {cmd} alone but fail together."),
                        why(rerun.unwrap_or(c), "on the combined change"),
                        format!("fix one branch so they agree, then {again}."),
                    );
                }
                let solo_fail = solo.iter().find(|s| !s.passed).copied().unwrap_or(c);
                let install = outcome
                    .insights
                    .iter()
                    .find_map(|i| i.split("for example ").nth(1))
                    .map(|s| s.trim_end_matches('.'));
                if failing.len() == solo.len() {
                    return v(
                        "alone",
                        "BLOCKED",
                        format!("{cmd} fails on every branch alone too, so the combination is not the problem."),
                        why(solo_fail, "on each branch"),
                        match install {
                            Some(fix) => format!("checks run in a fresh worktree, so install first: {fix}. Then {again}."),
                            None => format!("make {cmd} pass on {target}, or pass the right command with --check, then {again}."),
                        },
                    );
                }
                let who = and_list(&failing);
                return v(
                    "alone",
                    "BLOCKED",
                    format!(
                        "{who} {} {cmd} on {}, so the combination fails too.",
                        if failing.len() == 1 { "fails" } else { "fail" },
                        if failing.len() == 1 { "its own" } else { "their own" }
                    ),
                    why(solo_fail, "alone"),
                    format!("fix {who} first, then {again}."),
                );
            }
            if let Some((file, c)) = p
                .shared_files
                .iter()
                .find_map(|f| f.symbol_collisions.first().map(|c| (f, c)))
            {
                return v(
                    "collision",
                    "BLOCKED",
                    format!(
                        "{} both changed {} in {}.",
                        and_list(&c.inputs),
                        c.symbol,
                        file.path
                    ),
                    "--strict blocks when two branches change the same function or type.".into(),
                    format!("check that {} still works for both, or run without --strict.", c.symbol),
                );
            }
            v(
                "refused",
                "BLOCKED",
                format!("{names} could not land on {target}."),
                p.reasons.first().cloned().unwrap_or_default(),
                format!("fix that, then {again}."),
            )
        }
        _ if p.files_changed == 0 => v(
            "nothing",
            "READY",
            format!("{names} add nothing to {target}."),
            format!("every change on {} is already in {target}.", if several { "them" } else { "it" }),
            "nothing to merge.".into(),
        ),
        _ => match p.checks.iter().find(|c| c.scope == "combined") {
            None => v(
                "no_tests",
                "NO TESTS",
                format!("{names} merge cleanly onto {target}, but no tests ran."),
                format!("no test command was found at the root of {target} (package.json, Cargo.toml, go.mod, pyproject.toml...)."),
                "name one with --check \"<command>\", or once for this repo: git config pytxo.check \"<command>\".".into(),
            ),
            Some(c) if several => v(
                "ready",
                "READY",
                format!("{names} pass {} together.", c.command),
                format!(
                    "they merge cleanly onto {target} and {} passed on the combined change in {}.",
                    c.command,
                    seconds(c.duration_ms)
                ),
                "merge them as usual, or land them here with --apply. Nothing in your repository changed.".into(),
            ),
            Some(c) => v(
                "ready",
                "READY",
                format!("{names} passes {} on top of {target}.", c.command),
                format!(
                    "it merges cleanly and {} passed in {}.",
                    c.command,
                    seconds(c.duration_ms)
                ),
                "merge it as usual, or land it here with --apply. Nothing in your repository changed.".into(),
            ),
        },
    }
}

/// The one line every pull-request comment ends with.
pub const INSTALL_HINT: &str = "Add this check to any repo in one line: `uses: Pytxo-dev/pytxo/actions/land@v2` · see it work: `npx pytxo demo`";

/// Pull-request comment: the verdict, the detail folded, the install line.
pub fn render_markdown(outcome: &LandOutcome) -> String {
    render_markdown_with(outcome, false)
}

pub fn render_markdown_with(outcome: &LandOutcome, verbose: bool) -> String {
    let p = &outcome.receipt.predicate;
    let verdict = verdict_for(outcome, "push again", true);
    let mark = match verdict.label {
        "READY" | "LANDED" => "✅",
        "STALE" | "NO TESTS" => "⏸️",
        "FLAKY" => "⚠️",
        _ => "⛔",
    };
    let mut s = String::new();
    let _ = writeln!(
        s,
        "### {mark} {}: {}\n",
        verdict.label,
        verdict.what.trim_end_matches('.')
    );
    // Lower case on purpose: these often start with a command (npm test...).
    let _ = writeln!(s, "**Why:** {}\n", verdict.why);
    let _ = writeln!(s, "**Next:** {}\n", verdict.next);
    let _ = writeln!(s, "<details><summary>Details</summary>\n");
    let _ = writeln!(
        s,
        "Target `{}` at `{}` · {} inputs · {} files changed · {}\n",
        p.target.as_deref().unwrap_or("HEAD"),
        short(&p.base),
        p.inputs.len(),
        p.files_changed,
        seconds(p.duration_ms)
    );
    for insight in &outcome.insights {
        let _ = writeln!(s, "> {insight}\n");
    }
    let _ = writeln!(s, "| Input | Files | Status |\n|---|---:|---|");
    for i in &p.inputs {
        let _ = writeln!(
            s,
            "| `{}` | {} | {} |",
            input_name(&i.reference, i.agent.as_deref()),
            i.files.len(),
            if i.status == "conflict" {
                "**conflict**"
            } else {
                &i.status
            }
        );
    }
    if !p.shared_files.is_empty() {
        let _ = writeln!(s, "\n| Shared file | Inputs | Same symbol |\n|---|---|---|");
        for f in &p.shared_files {
            let same = f
                .symbol_collisions
                .iter()
                .map(|c| format!("`{}`", c.symbol))
                .collect::<Vec<_>>();
            let _ = writeln!(
                s,
                "| `{}` | {} | {} |",
                f.path,
                f.inputs.join(", "),
                if same.is_empty() {
                    "—".into()
                } else {
                    same.join(", ")
                }
            );
        }
    }
    if !p.checks.is_empty() {
        let _ = writeln!(s, "\n| Check | Scope | Result | Time |\n|---|---|---|---:|");
        for c in &p.checks {
            let result = if c.passed {
                "pass".to_string()
            } else if c.timed_out {
                "**timeout**".into()
            } else {
                "**fail**".into()
            };
            let _ = writeln!(
                s,
                "| `{}` | {} | {} | {} |",
                c.command,
                scope_label(c),
                result,
                seconds(c.duration_ms)
            );
        }
        for c in p.checks.iter().filter(|c| !c.passed) {
            let lines = why_lines(&c.output_tail).join("\n");
            if !lines.is_empty() {
                // Longer than any backtick run in the output, so no line closes it.
                let fence = "`".repeat(longest_run(&lines, '`').max(2) + 1);
                let _ = writeln!(
                    s,
                    "\n`{}` ({}):\n{fence}\n{lines}\n{fence}",
                    c.command,
                    scope_label(c),
                );
            }
        }
        if let Some(names) = left_out_of_combined(outcome) {
            let _ = writeln!(s, "\nCombined checks leave out {names}, which conflicted.");
        }
    }
    if !p.reasons.is_empty() {
        let _ = writeln!(s, "\nReasons:");
        for r in &p.reasons {
            let _ = writeln!(s, "- {r}");
        }
    }
    if verbose {
        let _ = writeln!(s, "\nReceipt `{}` · digest `{}`", p.id, p.content_digest);
    }
    let _ = writeln!(s, "\n</details>\n\n<sub>{INSTALL_HINT}</sub>");
    s
}
