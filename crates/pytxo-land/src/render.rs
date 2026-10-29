//! Terminal and pull-request (Markdown) summaries of a landing outcome.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use pytxo_core::{ReceiptCheck, ReceiptPredicate};
use serde::Serialize;

use crate::{short, Cause, LandOutcome, Use};

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

/// Whether `command` passed on the target with only the two inputs a cause
/// names (scope `pair`), and with every other merged input (`rest`); `None`
/// for a run that did not happen or did not finish.
fn culprits(p: &ReceiptPredicate, command: &str) -> (Option<bool>, Option<bool>) {
    let ran = |scope: &str| {
        p.checks
            .iter()
            .find(|c| c.scope == scope && c.command == command && !c.timed_out)
            .map(|c| c.passed)
    };
    (ran("pair"), ran("rest"))
}

/// Positions, in command-line order, of the inputs a BLOCKED verdict blames:
/// one that does not merge, one whose own check fails alone, and every input
/// of a combination that fails although each passes alone, or only the two
/// `cause` names once they fail it on their own and the others pass it
/// without them. Every input when it blames none of those (say the combined
/// run timed out before any input ran alone), so the GitHub Action stays
/// conservative. Empty for any other label.
fn implicated(p: &ReceiptPredicate, cause: Option<&Cause>, label: &str) -> Vec<usize> {
    if label != "BLOCKED" {
        return Vec::new();
    }
    let all = 0..p.inputs.len();
    let alone = |n: usize| {
        let scope = format!("input:{}", p.inputs[n].reference);
        p.checks.iter().filter(move |c| c.scope == scope)
    };
    let mut blamed: BTreeSet<usize> = all
        .clone()
        .filter(|&n| p.inputs[n].status == "conflict" || alone(n).any(|c| !c.passed))
        .collect();
    let merged: Vec<usize> = all
        .clone()
        .filter(|&n| p.inputs[n].status == "merged")
        .collect();
    let together = p
        .checks
        .iter()
        .filter(|c| c.scope == "combined" && !c.passed && !c.rerun && !passed_when_rerun(p, c))
        .filter(|c| {
            merged.iter().all(|&n| {
                let runs: Vec<&ReceiptCheck> =
                    alone(n).filter(|r| r.command == c.command).collect();
                !runs.is_empty() && runs.iter().all(|r| r.passed)
            })
        });
    for c in together {
        match cause {
            Some(cause) if culprits(p, &c.command) == (Some(false), Some(true)) => {
                blamed.extend([cause.by, cause.user]);
            }
            _ => blamed.extend(&merged),
        }
    }
    if blamed.is_empty() {
        all.collect()
    } else {
        blamed.into_iter().collect()
    }
}

/// The last line of a pull-request comment, for the GitHub Action:
/// `<!-- pytxo-verdict {"label":"BLOCKED","implicated":[0,2]} -->`.
fn verdict_tag(p: &ReceiptPredicate, cause: Option<&Cause>, label: &str) -> String {
    #[derive(Serialize)]
    struct Tag<'a> {
        label: &'a str,
        implicated: Vec<usize>,
    }
    let tag = Tag {
        label,
        implicated: implicated(p, cause, label),
    };
    format!(
        "<!-- pytxo-verdict {} -->",
        serde_json::to_string(&tag).expect("plain data")
    )
}

/// Inputs that conflicted and so are absent from the tree the combined checks
/// ran on; without this a "combined pass" reads as covering every input.
fn left_out_of_combined(outcome: &LandOutcome) -> Vec<String> {
    let p = &outcome.receipt.predicate;
    if !p.checks.iter().any(|c| c.scope == "combined") {
        return Vec::new();
    }
    p.inputs
        .iter()
        .filter(|i| i.status == "conflict")
        .map(|i| input_name(&i.reference, i.agent.as_deref()))
        .collect()
}

/// The lines of a failed check's output most likely to say why it failed,
/// the likeliest first, else the last few.
fn why_lines(tail: &[String]) -> Vec<String> {
    let trimmed = || tail.iter().map(|l| l.trim()).filter(|l| !l.is_empty());
    let mut ranked: Vec<(u8, &str)> = trimmed().filter_map(|l| Some((rank(l)?, l))).collect();
    // Stable: lines that rank alike keep the output's order.
    ranked.sort_by_key(|&(rank, _)| rank);
    let mut lines: Vec<&str> = ranked.into_iter().map(|(_, l)| l).take(3).collect();
    if lines.is_empty() {
        lines = trimmed().collect();
        lines.drain(..lines.len().saturating_sub(3));
    }
    // Go's `FAIL\t<package>\t0.2s` keeps its tabs, which a terminal widens.
    lines
        .into_iter()
        .map(|l| l.replace('\t', " ").chars().take(160).collect())
        .collect()
}

/// How well `line`, trimmed, says why a check failed, 0 best; `None` when it
/// does not. An error or exception name says the most, then an assertion or
/// a place with what is wrong there, then a test file or a Go test that
/// failed, then any line that says error, then fail. A test runner's count,
/// such as node:test's `ℹ fail 1` printed before the error itself, says
/// nothing.
pub(crate) fn rank(line: &str) -> Option<u8> {
    if names_an_error(line) {
        return Some(0);
    }
    let lower = line.to_lowercase();
    if lower.contains("traceback") || lower.contains("assert") || says_where(line) {
        return Some(1);
    }
    if fails_a_file(line) || line.starts_with("--- FAIL: ") {
        return Some(2);
    }
    if counts_failures(&lower) {
        return None;
    }
    if lower.contains("error") {
        return Some(3);
    }
    ["fail", "not recognized", "not found", "panicked"]
        .iter()
        .any(|m| lower.contains(m))
        .then_some(4)
}

/// `TypeError: ...`, `Error: ...`, Node's `TypeError [Error]: ...` and
/// `AssertionError [ERR_ASSERTION]: ...`, `java.io.IOException: ...`,
/// rustc's `error[E0425]: ...` or a Rust panic.
fn names_an_error(line: &str) -> bool {
    if line.contains("error[") || line.contains("panicked at") {
        return true;
    }
    ["Error", "Exception"].iter().any(|name| {
        line.match_indices(name).any(|(i, _)| {
            let rest = &line[i + name.len()..];
            let rest = match rest.strip_prefix(" [") {
                Some(coded) => coded.split_once(']').map_or("", |(_, r)| r),
                None => rest,
            };
            rest.starts_with(':')
        })
    })
}

/// `FAIL src/cart.test.js`, as Jest and Vitest name a test file that failed;
/// not Go's `FAIL\texample.com/shop\t0.2s`, which names a package.
fn fails_a_file(line: &str) -> bool {
    line.strip_prefix("FAIL").is_some_and(|rest| {
        rest.starts_with(char::is_whitespace)
            && rest.split_whitespace().next().is_some_and(names_a_file)
    })
}

/// A place and what is wrong there, as Go's tests and compilers, pytest,
/// mypy, eslint and gcc print it (`total_test.go:7: Total = 700, want 7`,
/// `.\discount.go:3:40: undefined: Price`), or tsc (`src/a.ts:3:5 - error
/// TS2304: ...`). Not a stack frame or a place with nothing after it.
fn says_where(line: &str) -> bool {
    let Some((word, rest)) = line.split_once(char::is_whitespace) else {
        return false;
    };
    let (place, message) = match word.strip_suffix(':') {
        Some(place) => (place, rest),
        None => match rest.strip_prefix("- ") {
            Some(message) => (word, message),
            None => return false,
        },
    };
    let number = |n: &str| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit());
    // `file:line`, then maybe `:column`.
    let Some((file, _)) = place.rsplit_once(':').filter(|(_, n)| number(n)) else {
        return false;
    };
    let file = file
        .rsplit_once(':')
        .filter(|(_, n)| number(n))
        .map_or(file, |(file, _)| file);
    !message.trim().is_empty() && names_a_file(file)
}

/// Whether `word` ends in a file name: after its last `/` or `\`, a name with
/// an extension of letters, a trailing `:` aside. `example.com/shop` is a Go
/// package, and so is `gopkg.in/yaml.v3`.
fn names_a_file(word: &str) -> bool {
    let name = word.rsplit(['/', '\\']).next().unwrap_or(word);
    name.trim_end_matches(':')
        .rsplit_once('.')
        .is_some_and(|(stem, ext)| {
            !stem.is_empty() && !ext.is_empty() && ext.bytes().all(|b| b.is_ascii_alphabetic())
        })
}

/// A count of failures: `ℹ fail 1`, `# fail 1`, `Tests: 1 failed`, `1 failed`
/// or `test result: FAILED. 1 passed; 1 failed`.
fn counts_failures(lower: &str) -> bool {
    let number = |w: &str| w.bytes().all(|b| b.is_ascii_digit());
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    words.windows(2).any(|pair| {
        (number(pair[0]) && pair[1].starts_with("fail"))
            || (pair[0].starts_with("fail") && number(pair[1]))
    })
}

/// What to do about a decision that did not land; `None` when nothing is needed.
fn next_step(outcome: &LandOutcome) -> Option<&'static str> {
    let p = &outcome.receipt.predicate;
    match p.decision.as_str() {
        "blocked" if p.inputs.iter().any(|i| i.status == "conflict") => Some(
            "merge or rebase the conflicting branch onto the target and the branches before it, resolve the conflict, then run the same command again",
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
        let names = left_out_of_combined(outcome);
        if !names.is_empty() {
            let _ = writeln!(s, "  combined leaves out {} (conflict)", names.join(", "));
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
    /// For "together": the name one branch renamed or removed that another
    /// still uses.
    pub cause: Option<NamedCause>,
    /// For the agent on the branch that has to change, once the failing
    /// output confirms the cause.
    pub fix_prompt: Option<String>,
    /// A line under the three, among the notes: a cause the failing output
    /// does not confirm, so the why line stays the error.
    #[serde(skip)]
    pub note: Option<String>,
}

/// A cause as `--json` gives it: the branches by name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NamedCause {
    pub by: String,
    pub user: String,
    pub name: String,
    pub renamed_to: Option<String>,
    pub uses: Vec<Use>,
    pub confirmed: bool,
}

/// Whose agent reads a fix prompt: the one on the branch that has to change,
/// the one on the branch that renamed or removed the name, or anyone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reader {
    User,
    By,
    Anyone,
}

/// The fix prompt for `reader`; `None` unless the verdict names a confirmed
/// cause.
pub fn fix_prompt(outcome: &LandOutcome, reader: Reader) -> Option<String> {
    verdict_for(outcome, "", false, reader).fix_prompt
}

/// What to paste into the agent of the branch that has to change: what
/// broke, where, what to do, and the end of the failing output.
fn prompt(
    by: &str,
    user: &str,
    cause: &Cause,
    command: &str,
    tail: &[String],
    reader: Reader,
) -> String {
    let x = &cause.name;
    let at: Vec<String> = cause
        .uses
        .iter()
        .take(3)
        .map(|u| format!("{}:{}", u.path, u.line))
        .collect();
    let at = at.join(", ");
    let (merged, renamer, still, fix) = match reader {
        Reader::User => (
            format!("{by} and your branch {user}"),
            by.to_string(),
            format!("Your branch {user} still uses"),
            "Fix it on your branch".to_string(),
        ),
        Reader::By => (
            format!("your branch {by} and {user}"),
            format!("Your branch {by}"),
            format!("{user} still uses"),
            format!("The fix belongs on {user}"),
        ),
        Reader::Anyone => (
            format!("{by} and {user}"),
            by.to_string(),
            format!("{user} still uses"),
            format!("Fix it on {user}"),
        ),
    };
    let mut s = format!(
        "Pytxo merged {merged} and ran {command}: each passes alone, together they fail.\n"
    );
    let _ = match &cause.renamed_to {
        Some(y) => writeln!(
            s,
            "{renamer} renamed {x} to {y}. {still} {x} at {at}.\n{fix}: rebase it onto {by}, replace {x} with {y}, and run {command} until it passes. {}",
            match reader {
                Reader::By => format!("The other way out is to keep {x} on your branch as an alias of {y}."),
                _ => format!("Leave {by}'s changes as they are."),
            }
        ),
        None => writeln!(
            s,
            "{renamer} removed {x}. {still} {x} at {at}.\n{fix}: rebase it onto {by}, stop using {x}, and run {command} until it passes. {}",
            match reader {
                Reader::User => format!("If your branch needs {x}, ask for it back on {by} instead."),
                Reader::By => format!("The other way out is to keep {x} on your branch."),
                Reader::Anyone => format!("If {user} needs {x}, ask for it back on {by} instead."),
            }
        ),
    };
    if !tail.is_empty() {
        let last = &tail[tail.len().saturating_sub(15)..];
        let _ = write!(s, "Failing output:\n{}", last.join("\n"));
    }
    s.truncate(s.trim_end().len());
    s
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
pub(crate) fn plain_name<'a>(reference: &'a str, label: Option<&'a str>) -> &'a str {
    let is_commit = reference.len() >= 7 && reference.chars().all(|c| c.is_ascii_hexdigit());
    match label {
        Some(label) if is_commit => label,
        _ => reference,
    }
}

/// "a", "a and b", up to "a, b, c and d" as the progress line has it, then
/// "5 branches".
fn and_list(names: &[String]) -> String {
    match names {
        [] => "no branches".into(),
        [one] => one.clone(),
        [init @ .., last] if names.len() <= 4 => format!("{} and {last}", init.join(", ")),
        _ => format!("{} branches", names.len()),
    }
}

/// The decision in plain words. `again` finishes "fix it, then ...", for
/// example "run npx pytxo again" or "push again".
pub fn verdict(outcome: &LandOutcome, again: &str) -> Verdict {
    verdict_for(outcome, again, false, Reader::Anyone)
}

/// The longest run of `c` in `text`.
fn longest_run(text: &str, c: char) -> usize {
    text.split(|x| x != c).map(str::len).max().unwrap_or(0)
}

/// `text` as a Markdown code span nothing in it can close, so test output in
/// a pull-request comment renders no mention, link or HTML. A line break
/// would end the span, so it reads as a space.
fn code_span(text: &str) -> String {
    let text = &text.replace(['\r', '\n'], " ");
    let ticks = "`".repeat(longest_run(text, '`') + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{ticks}{pad}{text}{pad}{ticks}")
}

/// A Markdown table cell nothing in `text` can end or format.
fn cell(text: &str) -> String {
    code_span(text).replace('|', "\\|")
}

/// `pr`: for a pull-request comment, where test output, branch names and
/// paths, which a fork controls, show only as code. `reader`: whom the fix
/// prompt speaks to.
fn verdict_for(outcome: &LandOutcome, again: &str, pr: bool, reader: Reader) -> Verdict {
    let p = &outcome.receipt.predicate;
    let target = p.target.as_deref().unwrap_or("HEAD");
    let shown = |text: &str| {
        if pr {
            code_span(text)
        } else {
            text.to_string()
        }
    };
    let branch = |i: &pytxo_core::ReceiptInput| shown(plain_name(&i.reference, i.agent.as_deref()));
    let all: Vec<String> = p.inputs.iter().map(branch).collect();
    let names = and_list(&all);
    let several = all.len() > 1;
    let v = |kind, label, what: String, why: String, next: String| Verdict {
        kind,
        label,
        what,
        why,
        next,
        cause: None,
        fix_prompt: None,
        note: None,
    };
    let first_line = |c: &ReceiptCheck| {
        let line = why_lines(&c.output_tail).into_iter().next()?;
        Some(if pr { code_span(&line) } else { line })
    };
    // A check that fails again speaks over one that passed when run again.
    let failing = || {
        p.checks
            .iter()
            .filter(|c| c.scope == "combined" && !c.passed && !c.rerun)
    };
    let failed_combined = failing()
        .find(|c| !passed_when_rerun(p, c))
        .or_else(|| failing().next());

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
                let before: Vec<&pytxo_core::ReceiptInput> = p
                    .inputs
                    .iter()
                    .take_while(|i| i.reference != c.reference)
                    .filter(|i| i.status == "merged")
                    .collect();
                let touched: Vec<String> = before
                    .iter()
                    .filter(|i| i.files.iter().any(|f| c.conflict_files.contains(f)))
                    .map(|i| branch(i))
                    .collect();
                // Merges start from the target: a branch that does not merge
                // onto it alone either conflicts with the target; any other,
                // with the branches before it that changed the same files.
                let others = if before.is_empty() || outcome.target_conflicts.contains(&c.reference)
                {
                    shown(target)
                } else if touched.is_empty() {
                    and_list(&before.iter().map(|i| branch(i)).collect::<Vec<_>>())
                } else {
                    and_list(&touched)
                };
                let files = match c.conflict_files.as_slice() {
                    [] => "the same lines".to_string(),
                    [a] => shown(a),
                    [a, b] => format!("{} and {}", shown(a), shown(b)),
                    [a, b, rest @ ..] => {
                        format!("{}, {} and {} more", shown(a), shown(b), rest.len())
                    }
                };
                let name = branch(c);
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
                        format!(
                            "make {cmd} exit on its own, or raise {}, then {again}.",
                            if pr {
                                "the Action's `timeout:` input"
                            } else {
                                "--timeout"
                            }
                        ),
                    );
                }
                let solo: Vec<&ReceiptCheck> = p
                    .checks
                    .iter()
                    .filter(|s| s.scope.starts_with("input:") && s.command == c.command)
                    .collect();
                // A run that timed out on the target says nothing about it.
                let on_target = p
                    .checks
                    .iter()
                    .find(|s| s.scope == "target" && s.command == c.command && !s.timed_out);
                let why = |check: &ReceiptCheck, place: &str| match first_line(check) {
                    Some(line) => format!("{cmd} fails {place}: {line}"),
                    None => format!(
                        "{cmd} exits {} {place}.",
                        check.exit_code.map_or("with an error".into(), |e| e.to_string())
                    ),
                };
                // `--check "<command that installs first>"`, from the insights.
                let install = outcome
                    .insights
                    .iter()
                    .find_map(|i| i.split("for example ").nth(1))
                    .map(|s| s.trim_end_matches('.'));
                let install_first = |fix: &str| {
                    if pr {
                        format!(
                            "checks run in a fresh worktree, so install first: set the Action's `check:` input to `{}`. Then {again}.",
                            fix.trim_start_matches("--check ").trim_matches('"')
                        )
                    } else {
                        format!("checks run in a fresh worktree, so install first: {fix}. Then {again}.")
                    }
                };
                // For a failure the set-up may explain; not when a branch or
                // the target passes.
                let submodules = |next: String| {
                    if !outcome.submodules {
                        return next;
                    }
                    let fix = format!("git submodule update --init --recursive && {cmd}");
                    let how = if pr {
                        format!("set the Action's `check:` input to {}", code_span(&fix))
                    } else {
                        format!("pass --check \"{fix}\"")
                    };
                    format!("{next} Checks run in a fresh worktree without submodules: to include them, {how}.")
                };
                if solo.is_empty() {
                    return v(
                        "failed",
                        "BLOCKED",
                        format!("{names} fail {cmd} when combined."),
                        why(c, "on the combined change"),
                        submodules(match install {
                            Some(fix) if pr => install_first(fix),
                            _ if pr => format!("fix the failure, then {again}."),
                            _ => format!("to see whether each branch passes alone, run it with --per-input. Fix the failure, then {again}."),
                        }),
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
                            .map_or_else(|| shown(r), branch)
                    })
                    .collect();
                if failing.is_empty() {
                    let run = rerun.unwrap_or(c);
                    let verdict = v(
                        "together",
                        "BLOCKED",
                        format!("{names} pass {cmd} alone but fail together."),
                        why(run, "on the combined change"),
                        format!("fix one branch so they agree, then {again}."),
                    );
                    let named = |n: usize| {
                        let i = p.inputs.get(n)?;
                        Some(plain_name(&i.reference, i.agent.as_deref()).to_string())
                    };
                    let Some((cause, by, user)) = outcome
                        .cause
                        .as_ref()
                        .and_then(|c| Some((c, named(c.by)?, named(c.user)?)))
                    else {
                        return verdict;
                    };
                    let (b, u, x) = (shown(&by), shown(&user), shown(&cause.name));
                    // The two in the order they were given, as `names` has them.
                    let pair = if cause.by < cause.user {
                        format!("{b} and {u}")
                    } else {
                        format!("{u} and {b}")
                    };
                    let (pair_ran, rest_ran) = culprits(p, cmd);
                    // They pass without the others, so the cause guessed
                    // wrong: the verdict as without one, and what they did.
                    if pair_ran == Some(true) {
                        return Verdict {
                            note: Some(format!(
                                "{pair} pass {cmd} together without the other branches."
                            )),
                            ..verdict
                        };
                    }
                    let at = cause.uses.first().map_or_else(String::new, |w| {
                        format!(" ({})", shown(&format!("{}:{}", w.path, w.line)))
                    });
                    let mut verdict = Verdict {
                        cause: Some(NamedCause {
                            by: by.clone(),
                            user: user.clone(),
                            name: cause.name.clone(),
                            renamed_to: cause.renamed_to.clone(),
                            uses: cause.uses.clone(),
                            confirmed: cause.confirmed,
                        }),
                        ..verdict
                    };
                    // Unconfirmed, it is a hint under the error, not the why.
                    match (cause.renamed_to.as_deref().map(shown), cause.confirmed) {
                        (Some(y), true) => {
                            verdict.why = format!("{b} renamed {x} to {y}; {u} still uses {x}{at}.");
                            verdict.next = format!("rebase {u} onto {b} and use {y}, then {again}.");
                        }
                        (None, true) => {
                            verdict.why = format!("{b} removed {x}; {u} still uses it{at}.");
                            verdict.next = format!(
                                "rebase {u} onto {b} and stop using {x}, or keep {x} on {b}, then {again}."
                            );
                        }
                        (Some(y), false) => {
                            verdict.note = Some(format!("{u} uses {x}, which {b} renamed to {y}{at}."));
                        }
                        (None, false) => {
                            verdict.note = Some(format!("{u} uses {x}, which {b} removed{at}."));
                        }
                    }
                    if cause.confirmed {
                        verdict.fix_prompt =
                            Some(prompt(&by, &user, cause, cmd, &run.output_tail, reader));
                    }
                    // They fail it on their own: the what line names them,
                    // and says how the others did without them.
                    if pair_ran == Some(false) {
                        let others: Vec<String> = p
                            .inputs
                            .iter()
                            .enumerate()
                            .filter(|&(n, i)| {
                                i.status == "merged" && n != cause.by && n != cause.user
                            })
                            .map(|(_, i)| branch(i))
                            .collect();
                        let (who, one) = match others.as_slice() {
                            [other] => (other.clone(), true),
                            _ => (format!("the other {} branches", others.len()), false),
                        };
                        verdict.what = match rest_ran {
                            Some(true) => format!(
                                "{pair} fail together; {who} {} without them.",
                                if one { "passes" } else { "pass" }
                            ),
                            Some(false) => format!(
                                "{pair} fail together; {who} {} without them too.",
                                if one { "fails" } else { "fail" }
                            ),
                            None => format!("{pair} fail together."),
                        };
                    }
                    return verdict;
                }
                let solo_fail = solo.iter().find(|s| !s.passed).copied().unwrap_or(c);
                // Every branch failing alone blames them only if the target
                // passes; then they each fail on their own, below.
                if failing.len() == solo.len() && !on_target.is_some_and(|t| t.passed) {
                    let (what, reason) = match on_target {
                        Some(t) => (
                            format!("{cmd} fails on {target} as well, so the check or the environment is broken, not the branches."),
                            why(t, &format!("on {target}")),
                        ),
                        None => (
                            format!("{cmd} fails on every branch alone too, so the combination is not the problem."),
                            why(solo_fail, "on each branch"),
                        ),
                    };
                    return v(
                        "alone",
                        "BLOCKED",
                        what,
                        reason,
                        submodules(match install {
                            Some(fix) => install_first(fix),
                            None if pr => format!("make {cmd} pass on {target}, or set the Action's `check:` input to the right command, then {again}."),
                            None => format!("make {cmd} pass on {target}, or pass the right command with --check, then {again}."),
                        }),
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
                let inputs: Vec<String> = c.inputs.iter().map(|n| shown(n)).collect();
                return v(
                    "collision",
                    "BLOCKED",
                    format!(
                        "{} both changed {} in {}.",
                        and_list(&inputs),
                        shown(&c.symbol),
                        shown(&file.path)
                    ),
                    "--strict blocks when two branches change the same function or type.".into(),
                    format!(
                        "check that {} still works for both, or run without --strict.",
                        shown(&c.symbol)
                    ),
                );
            }
            v(
                "refused",
                "BLOCKED",
                format!("{names} could not land on {target}."),
                p.reasons.first().map(|r| shown(r)).unwrap_or_default(),
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
                format!(
                    "{names} {} cleanly onto {target}, but no tests ran.",
                    if several { "merge" } else { "merges" }
                ),
                format!("no test command was found at the root of {target} (package.json, Cargo.toml, go.mod, pyproject.toml...)."),
                if pr {
                    "set the Action's `check:` input to your test command.".into()
                } else {
                    "name one with --check \"<command>\", or once for this repo: git config pytxo.check \"<command>\".".into()
                },
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
                format!(
                    "merge them as usual{}. Nothing in your repository changed.",
                    if pr { "" } else { ", or land them here with --apply" }
                ),
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
                format!(
                    "merge it as usual{}. Nothing in your repository changed.",
                    if pr { "" } else { ", or land it here with --apply" }
                ),
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
    let verdict = verdict_for(outcome, "push again", true, Reader::Anyone);
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
    if let Some(note) = &verdict.note {
        let _ = writeln!(s, "**Note:** {note}\n");
    }
    if let (Some(prompt), Some(cause)) = (&verdict.fix_prompt, &verdict.cause) {
        // The prompt holds branch names and test output: only in a fence,
        // longer than any backtick run in it. The name in the summary is
        // HTML, where `<code>` keeps it from mentioning anyone.
        let fence = "`".repeat(longest_run(prompt, '`').max(2) + 1);
        let user = cause
            .user
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('@', "&#64;");
        let _ = writeln!(
            s,
            "<details><summary>Fix prompt for <code>{user}</code></summary>\n\n{fence}text\n{prompt}\n{fence}\n\n</details>\n"
        );
    }
    let _ = writeln!(s, "<details><summary>Details</summary>\n");
    let _ = writeln!(
        s,
        "Target `{}` at `{}` · {} input{} · {} file{} changed · {}\n",
        p.target.as_deref().unwrap_or("HEAD"),
        short(&p.base),
        p.inputs.len(),
        if p.inputs.len() == 1 { "" } else { "s" },
        p.files_changed,
        if p.files_changed == 1 { "" } else { "s" },
        seconds(p.duration_ms)
    );
    // No insights: they give terminal advice (--check), name branches and
    // paths as plain text, and say again what the verdict and tables say.
    let _ = writeln!(s, "| Input | Files | Status |\n|---|---:|---|");
    for i in &p.inputs {
        let _ = writeln!(
            s,
            "| {} | {} | {} |",
            cell(&input_name(&i.reference, i.agent.as_deref())),
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
                .map(|c| cell(&c.symbol))
                .collect::<Vec<_>>();
            let inputs = f.inputs.iter().map(|n| cell(n)).collect::<Vec<_>>();
            let _ = writeln!(
                s,
                "| {} | {} | {} |",
                cell(&f.path),
                inputs.join(", "),
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
                "| {} | {} | {} | {} |",
                cell(&c.command),
                cell(&scope_label(c)),
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
                    "\n{} ({}):\n{fence}\n{lines}\n{fence}",
                    code_span(&c.command),
                    code_span(&scope_label(c)),
                );
            }
        }
        let names: Vec<String> = left_out_of_combined(outcome)
            .iter()
            .map(|n| code_span(n))
            .collect();
        if !names.is_empty() {
            let _ = writeln!(
                s,
                "\nCombined checks leave out {}, which conflicted.",
                names.join(", ")
            );
        }
    }
    if !p.reasons.is_empty() {
        let _ = writeln!(s, "\nReasons:");
        for r in &p.reasons {
            let _ = writeln!(s, "- {}", code_span(r));
        }
    }
    if verbose {
        let _ = writeln!(s, "\nReceipt `{}` · digest `{}`", p.id, p.content_digest);
    }
    let _ = writeln!(s, "\n</details>\n\n<sub>{INSTALL_HINT}</sub>");
    // Test output can print a tag of its own; only the last line holds one.
    let s = s.replace("pytxo-verdict", "pytxo-\u{200B}verdict");
    format!(
        "{s}\n{}\n",
        verdict_tag(p, outcome.cause.as_ref(), verdict.label)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pytxo_core::{Receipt, ReceiptInput, ReceiptSharedFile};

    fn outcome(p: ReceiptPredicate) -> LandOutcome {
        LandOutcome {
            receipt: Receipt::new(p, "git:main", None),
            receipt_path: Default::default(),
            combined_ref: None,
            insights: Vec::new(),
            out_of_time: false,
            submodules: false,
            target_conflicts: Vec::new(),
            cause: None,
        }
    }

    fn input(reference: &str, status: &str) -> ReceiptInput {
        ReceiptInput {
            reference: reference.into(),
            agent: None,
            commit: "0".repeat(40),
            status: status.into(),
            files: vec![format!("{reference}.txt")],
            conflict_files: Vec::new(),
        }
    }

    fn check(scope: &str, passed: bool) -> ReceiptCheck {
        ReceiptCheck {
            scope: scope.into(),
            command: "npm test".into(),
            exit_code: Some(if passed { 0 } else { 1 }),
            passed,
            duration_ms: 1,
            timed_out: false,
            output_tail: Vec::new(),
            rerun: false,
        }
    }

    fn rerun(passed: bool) -> ReceiptCheck {
        ReceiptCheck {
            rerun: true,
            ..check("combined", passed)
        }
    }

    /// Inputs a, b and c, the third one as `c_status`.
    fn blocked(c_status: &str, checks: Vec<ReceiptCheck>) -> ReceiptPredicate {
        ReceiptPredicate {
            id: "rcpt_20261028T120005Z_ab12cd34".into(),
            kind: "land".into(),
            created_at: String::new(),
            pytxo_version: String::new(),
            decision: "blocked".into(),
            mode: "shadow".into(),
            target: Some("main".into()),
            base: "0".repeat(40),
            result: None,
            inputs: vec![
                input("a", "merged"),
                input("b", "merged"),
                input("c", c_status),
            ],
            files_changed: 3,
            shared_files: Vec::new(),
            checks,
            reasons: vec!["blocked".into()],
            duration_ms: 1,
            content_digest: String::new(),
        }
    }

    #[test]
    fn only_a_blocked_verdict_implicates_anyone() {
        let p = blocked(
            "conflict",
            vec![check("input:a", false), check("combined", false)],
        );
        for label in ["READY", "LANDED", "FLAKY", "NO TESTS", "STALE"] {
            assert_eq!(implicated(&p, None, label), Vec::<usize>::new(), "{label}");
        }
    }

    #[test]
    fn an_input_that_does_not_merge_is_implicated() {
        let p = blocked("conflict", vec![check("combined", true)]);
        assert_eq!(implicated(&p, None, "BLOCKED"), [2]);
    }

    #[test]
    fn an_input_that_fails_alone_is_implicated() {
        let checks = vec![
            check("combined", false),
            check("input:a", true),
            check("input:b", false),
            check("input:c", true),
        ];
        assert_eq!(implicated(&blocked("merged", checks), None, "BLOCKED"), [1]);
        // Not merging onto the target alone counts as failing alone.
        let checks = vec![
            check("input:a", true),
            ReceiptCheck {
                command: "(merge onto target)".into(),
                ..check("input:c", false)
            },
        ];
        assert_eq!(implicated(&blocked("merged", checks), None, "BLOCKED"), [2]);
    }

    #[test]
    fn a_combination_that_fails_while_each_input_passes_alone_implicates_all_of_it() {
        let alone_ok = || ["input:a", "input:b", "input:c"].map(|s| check(s, true));
        let mut checks = vec![check("combined", false)];
        checks.extend(alone_ok());
        checks.push(rerun(false));
        assert_eq!(
            implicated(&blocked("merged", checks), None, "BLOCKED"),
            [0, 1, 2]
        );
        // A conflicting input is not in the combination, but is blamed itself.
        let mut checks = vec![check("combined", false)];
        checks.extend(alone_ok());
        checks.push(rerun(false));
        assert_eq!(
            implicated(&blocked("conflict", checks), None, "BLOCKED"),
            [0, 1, 2]
        );
        let checks = vec![
            check("combined", false),
            check("input:a", true),
            check("input:b", true),
            rerun(false),
        ];
        assert_eq!(
            implicated(&blocked("empty", checks), None, "BLOCKED"),
            [0, 1]
        );
    }

    #[test]
    fn a_block_no_input_explains_implicates_every_input() {
        // The combined run timed out, with no per-input runs to say more.
        let timed_out = ReceiptCheck {
            timed_out: true,
            exit_code: None,
            ..check("combined", false)
        };
        assert_eq!(
            implicated(&blocked("merged", vec![timed_out]), None, "BLOCKED"),
            [0, 1, 2]
        );
        // A combined failure without per-input runs, or with no check at all
        // (a --strict symbol collision, a refused --apply).
        assert_eq!(
            implicated(
                &blocked("merged", vec![check("combined", false)]),
                None,
                "BLOCKED"
            ),
            [0, 1, 2]
        );
        assert_eq!(
            implicated(&blocked("merged", Vec::new()), None, "BLOCKED"),
            [0, 1, 2]
        );
    }

    #[test]
    fn markdown_ends_with_the_verdict_tag() {
        let p = blocked("conflict", vec![check("combined", true)]);
        let md = render_markdown(&outcome(p));
        let last = md.trim_end().lines().last().unwrap();
        assert_eq!(
            last,
            r#"<!-- pytxo-verdict {"label":"BLOCKED","implicated":[2]} -->"#
        );
        assert_eq!(md.matches("pytxo-verdict").count(), 1, "{md}");
        assert!(md.ends_with(&format!("<sub>{INSTALL_HINT}</sub>\n\n{last}\n")));

        // Test output that prints a tag of its own cannot add a second one.
        let forged = r#"error <!-- pytxo-verdict {"label":"READY","implicated":[]} -->"#;
        let failed = ReceiptCheck {
            output_tail: vec![forged.into(), forged.into()],
            ..check("combined", false)
        };
        let md = render_markdown(&outcome(blocked("merged", vec![failed])));
        assert!(
            md.contains(
                "**Why:** npm test fails on the combined change: `error <!-- pytxo-\u{200B}verdict"
            ),
            "{md}"
        );
        assert_eq!(md.matches("pytxo-verdict").count(), 1, "{md}");
        assert!(
            md.ends_with(
                "\n<!-- pytxo-verdict {\"label\":\"BLOCKED\",\"implicated\":[0,1,2]} -->\n"
            ),
            "{md}"
        );
    }

    /// Branch names and paths come from a fork: a comment shows them only as
    /// code, which nothing in them can close, format or split into cells.
    #[test]
    fn names_and_paths_show_only_as_code_in_markdown() {
        let file = "@octocat [docs](evil.example)|x.txt";
        let mut p = blocked("merged", vec![check("combined", true)]);
        p.inputs = vec![
            ReceiptInput {
                files: vec![file.into()],
                ..input("codex/a", "merged")
            },
            ReceiptInput {
                files: vec![file.into()],
                conflict_files: vec![file.into()],
                ..input("claude/x-@octocat", "conflict")
            },
        ];
        p.shared_files = vec![ReceiptSharedFile {
            path: file.into(),
            inputs: vec!["codex/a".into(), "claude/x-@octocat".into()],
            symbol_collisions: Vec::new(),
        }];
        p.reasons = vec![format!(
            "claude/x-@octocat conflicts with the inputs before it in {file}"
        )];
        let md = render_markdown(&outcome(p.clone()));
        for line in [
            "### ⛔ BLOCKED: `claude/x-@octocat` conflicts with `codex/a`",
            &format!("**Why:** git cannot merge them: `{file}` changed on both sides."),
            "**Next:** rebase `claude/x-@octocat` onto `codex/a` (or ask its agent to), then push again.",
            "| `claude/x-@octocat` | 1 | **conflict** |",
            "| `@octocat [docs](evil.example)\\|x.txt` | `codex/a`, `claude/x-@octocat` | — |",
            "Combined checks leave out `claude/x-@octocat`, which conflicted.",
            &format!("- `claude/x-@octocat conflicts with the inputs before it in {file}`"),
        ] {
            assert!(md.lines().any(|l| l == line), "{line}\n{md}");
        }
        // Outside code nothing mentions or links.
        for line in md.lines() {
            let outside: String = line.split('`').step_by(2).collect();
            assert!(!outside.contains('@') && !outside.contains("]("), "{line}");
        }
        // The terminal shows them as they are.
        let v = verdict(&outcome(p), "again");
        assert_eq!(v.what, "claude/x-@octocat conflicts with codex/a.");
    }

    /// The verdict names as many branches as the progress line does.
    #[test]
    fn names_up_to_four_branches_then_counts_them() {
        let names = |n: usize| (1..=n).map(|i| format!("b{i}")).collect::<Vec<_>>();
        assert_eq!(and_list(&names(1)), "b1");
        assert_eq!(and_list(&names(2)), "b1 and b2");
        assert_eq!(and_list(&names(4)), "b1, b2, b3 and b4");
        assert_eq!(and_list(&names(5)), "5 branches");
    }

    /// GitHub ends a line, and so a code span, at a carriage return too.
    #[test]
    fn a_code_span_stays_on_one_line() {
        assert_eq!(code_span("0%\r# @x\nb"), "`0% # @x b`");
    }

    #[test]
    fn one_branch_merges_cleanly() {
        let mut p = blocked("merged", Vec::new());
        p.decision = "ready".into();
        p.inputs.truncate(1);
        assert_eq!(
            verdict(&outcome(p), "again").what,
            "a merges cleanly onto main, but no tests ran."
        );
    }

    /// A target run that timed out is no evidence: the verdict says only what
    /// the branches showed.
    #[test]
    fn a_target_run_that_timed_out_is_not_blamed() {
        let timed_out = ReceiptCheck {
            timed_out: true,
            exit_code: None,
            ..check("target", false)
        };
        let checks = vec![
            check("combined", false),
            check("input:a", false),
            check("input:b", false),
            check("input:c", false),
            timed_out,
        ];
        let v = verdict(&outcome(blocked("merged", checks)), "again");
        assert_eq!(
            v.what,
            "npm test fails on every branch alone too, so the combination is not the problem."
        );
        assert_eq!(v.why, "npm test exits 1 on each branch.");
    }

    #[test]
    fn details_count_one_input_and_one_file() {
        let mut p = blocked("merged", vec![check("combined", false)]);
        let md = render_markdown(&outcome(p.clone()));
        assert!(md.contains(" · 3 inputs · 3 files changed · "), "{md}");
        p.inputs.truncate(1);
        p.files_changed = 1;
        let md = render_markdown(&outcome(p));
        assert!(md.contains(" · 1 input · 1 file changed · "), "{md}");
    }

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_string).collect()
    }

    /// The first line `why_lines` picks from runs, local paths shortened: real
    /// ones, and pytest's written out in its format.
    #[test]
    fn the_why_line_is_the_error_not_the_count() {
        // node:test's spec reporter, Node 22 and later: the counts, `ℹ fail
        // 1` among them, come before the error. Node 26.
        let node_spec = "✔ total adds up the prices (0.7425ms)
✖ discounted takes the percentage off (0.8154ms)
ℹ tests 2
ℹ suites 0
ℹ pass 1
ℹ fail 1
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 150.0703
✖ failing tests:
test at test\\discount.test.js:5:1
✖ discounted takes the percentage off (0.8154ms)
  TypeError [Error]: price is not a function
      at discounted (C:\\work\\shop\\discount.js:4:10)
      at TestContext.<anonymous> (C:\\work\\shop\\test\\discount.test.js:6:22)
      at Test.runInAsyncScope (node:async_hooks:226:14)";
        // The same run with `--test-reporter=tap`.
        let node_tap = "not ok 2 - discounted takes the percentage off
  ---
  duration_ms: 0.7017
  type: 'test'
  location: 'C:\\\\work\\\\shop\\\\test\\\\discount.test.js:5:1'
  failureType: 'testCodeFailure'
  error: 'price is not a function'
  code: 'ERR_TEST_FAILURE'
  name: 'TypeError'
  stack: |-
    discounted (C:\\work\\shop\\discount.js:4:10)
  ...
1..2
# tests 2
# suites 0
# pass 1
# fail 1";
        // Jest 30, colors removed as the output tail removes them.
        let jest = "FAIL src/discount.test.js
  ● discount takes ten percent
    TypeError: price is not a function
      3 | function discounted(item) {
    > 4 |   return price(item) * 0.9;
        |          ^
      at discounted (src/discount.js:4:10)
      at Object.<anonymous> (src/discount.test.js:4:10)
Test Suites: 1 failed, 1 passed, 2 total
Tests:       1 failed, 1 passed, 2 total
Snapshots:   0 total
Time:        1.547 s
Ran all test suites.";
        // pytest 8, written out as it prints a failed assert.
        let pytest =
            "tests/test_cart.py .                                                     [ 50%]
tests/test_discount.py F                                                 [100%]
================================== FAILURES ===================================
________________________________ test_discount ________________________________
    def test_discount():
>       assert discounted({\"cost\": 10}) == 9
tests/test_discount.py:4:
item = {'cost': 10}
    def discounted(item):
>       return price(item) * 0.9
E       NameError: name 'price' is not defined
src/discount.py:4: NameError
=========================== short test summary info ===========================
FAILED tests/test_discount.py::test_discount - NameError: name 'price' is not defined
========================= 1 failed, 1 passed in 0.03s =========================";
        // cargo test, a failed assertion, then a call to a missing function.
        let cargo_panic = "running 2 tests
test tests::total_adds_up ... ok
test tests::discount_takes_ten_percent ... FAILED
failures:
---- tests::discount_takes_ten_percent stdout ----
thread 'tests::discount_takes_ten_percent' (57716) panicked at src\\lib.rs:20:9:
assertion `left == right` failed
  left: 180
 right: 190
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
failures:
    tests::discount_takes_ten_percent
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: test failed, to rerun pass `--lib`";
        let cargo_build = "   Compiling shop v0.1.0 (C:\\work\\shop)
error[E0425]: cannot find function `price` in this scope
 --> src\\lib.rs:6:5
  |
6 |     price(cents) * 9 / 10
  |     ^^^^^ not found in this scope
For more information about this error, try `rustc --explain E0425`.
error: could not compile `shop` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...";
        // go test 1.26: a failed test, then a call that does not compile.
        let go_test = "--- FAIL: TestTotalOfOne (0.00s)
    total_test.go:7: Total([7]) = 700, want 7
FAIL
FAIL\texample.com/shop\t0.248s
FAIL";
        let go_build = "# example.com/shop [example.com/shop.test]
.\\discount.go:3:46: not enough arguments in call to Price
\thave (int)
\twant (int, int)
FAIL\texample.com/shop [build failed]
FAIL";
        for (output, first) in [
            (node_spec, "TypeError [Error]: price is not a function"),
            (node_tap, "error: 'price is not a function'"),
            (jest, "TypeError: price is not a function"),
            (pytest, "E       NameError: name 'price' is not defined"),
            (
                cargo_panic,
                "thread 'tests::discount_takes_ten_percent' (57716) panicked at src\\lib.rs:20:9:",
            ),
            (
                cargo_build,
                "error[E0425]: cannot find function `price` in this scope",
            ),
            (go_test, "total_test.go:7: Total([7]) = 700, want 7"),
            (
                go_build,
                ".\\discount.go:3:46: not enough arguments in call to Price",
            ),
        ] {
            let why = why_lines(&lines(output));
            assert_eq!(why.first().map(String::as_str), Some(first), "{output}");
            assert!(
                !why.iter().any(|l| counts_failures(&l.to_lowercase())),
                "{why:?}"
            );
        }
        // Then a failing file, then other error lines; never a count.
        assert_eq!(
            why_lines(&lines(jest)),
            [
                "TypeError: price is not a function",
                "FAIL src/discount.test.js"
            ]
        );
        for count in [
            "ℹ fail 1",
            "# fail 1",
            "Tests:       1 failed, 1 passed, 2 total",
            "========================= 1 failed, 1 passed in 0.03s ====",
            "test result: FAILED. 1 passed; 1 failed; 0 ignored",
            "  1 failing",
        ] {
            assert!(counts_failures(&count.to_lowercase()), "{count}");
            assert_eq!(rank(count), None, "{count}");
        }
        // The demo's own runner names the file and the error on one line.
        let demo = lines("ok   cart.test.js\nFAIL discount.test.js: price is not a function");
        assert_eq!(
            why_lines(&demo),
            ["FAIL discount.test.js: price is not a function"]
        );
        // Go's test, then the place, before its package; a package is no
        // file, and its tabs read as spaces.
        assert_eq!(
            why_lines(&lines(go_test)),
            [
                "total_test.go:7: Total([7]) = 700, want 7",
                "--- FAIL: TestTotalOfOne (0.00s)",
                "FAIL"
            ]
        );
        assert_eq!(
            why_lines(&lines("FAIL\texample.com/shop\t0.248s")),
            ["FAIL example.com/shop 0.248s"]
        );
        for package in ["FAIL\texample.com/shop\t0.2s", "FAIL gopkg.in/yaml.v3"] {
            assert!(!fails_a_file(package), "{package}");
        }
        // A place with what is wrong there, not a frame or a bare place.
        for place in [
            "src/discount.py:4: NameError",
            r"C:\work\shop\a.go:3:5: undefined: Price",
            "src/a.ts:3:5 - error TS2304: Cannot find name 'price'.",
        ] {
            assert_eq!(rank(place), Some(1), "{place}");
        }
        for not in [
            "at discounted (src/discount.js:4:10)",
            "tests/test_discount.py:4:",
            "test at test\\discount.test.js:5:1",
            "location: 'test/discount.test.js:5:1'",
        ] {
            assert!(!says_where(not), "{not}");
        }
    }

    fn input_named(reference: &str) -> ReceiptInput {
        input(reference, "merged")
    }

    /// The demo's runs: each branch passes alone, the combination fails
    /// twice, and `cause` read from them.
    fn together(cause: Option<Cause>) -> LandOutcome {
        let failed = |rerun| ReceiptCheck {
            rerun,
            output_tail: lines("ok   cart.test.js\nFAIL discount.test.js: price is not a function"),
            ..check("combined", false)
        };
        let mut p = blocked("merged", Vec::new());
        p.inputs = vec![
            input_named("claude/rename-price"),
            input_named("codex/add-discount"),
        ];
        p.checks = vec![
            failed(false),
            check("input:claude/rename-price", true),
            check("input:codex/add-discount", true),
            failed(true),
        ];
        LandOutcome {
            cause,
            ..outcome(p)
        }
    }

    fn renamed(confirmed: bool) -> Cause {
        let at = |line| Use {
            path: "src/discount.js".into(),
            line,
        };
        Cause {
            by: 0,
            user: 1,
            name: "price".into(),
            renamed_to: Some("unitPrice".into()),
            uses: vec![at(1), at(4)],
            confirmed,
        }
    }

    #[test]
    fn a_confirmed_rename_is_the_why_and_the_next() {
        let again = "run npx pytxo again";
        let v = verdict(&together(Some(renamed(true))), again);
        assert_eq!((v.kind, v.label), ("together", "BLOCKED"));
        assert_eq!(
            v.to_string(),
            "BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.
why      claude/rename-price renamed price to unitPrice; codex/add-discount still uses price (src/discount.js:1).
next     rebase codex/add-discount onto claude/rename-price and use unitPrice, then run npx pytxo again.
"
        );
        assert_eq!(v.note, None);
        assert_eq!(
            v.fix_prompt.as_deref(),
            Some("Pytxo merged claude/rename-price and codex/add-discount and ran npm test: each passes alone, together they fail.
claude/rename-price renamed price to unitPrice. codex/add-discount still uses price at src/discount.js:1, src/discount.js:4.
Fix it on codex/add-discount: rebase it onto claude/rename-price, replace price with unitPrice, and run npm test until it passes. Leave claude/rename-price's changes as they are.
Failing output:
ok   cart.test.js
FAIL discount.test.js: price is not a function")
        );
        let json = serde_json::to_value(&v).unwrap();
        assert_eq!(
            json["cause"],
            serde_json::json!({
                "by": "claude/rename-price",
                "user": "codex/add-discount",
                "name": "price",
                "renamed_to": "unitPrice",
                "uses": [{"path": "src/discount.js", "line": 1}, {"path": "src/discount.js", "line": 4}],
                "confirmed": true,
            })
        );
        assert!(json.get("note").is_none(), "{json}");

        // Without a cause the verdict is as before, and says so in JSON.
        let v = verdict(&together(None), again);
        assert_eq!(
            v.why,
            "npm test fails on the combined change: FAIL discount.test.js: price is not a function"
        );
        assert_eq!(
            v.next,
            "fix one branch so they agree, then run npx pytxo again."
        );
        let json = serde_json::to_value(&v).unwrap();
        assert!(
            json["cause"].is_null() && json["fix_prompt"].is_null(),
            "{json}"
        );
    }

    #[test]
    fn a_confirmed_removal_says_to_stop_using_it_or_keep_it() {
        let removed = Cause {
            renamed_to: None,
            ..renamed(true)
        };
        let v = verdict(&together(Some(removed)), "run npx pytxo again");
        assert_eq!(
            v.why,
            "claude/rename-price removed price; codex/add-discount still uses it (src/discount.js:1)."
        );
        assert_eq!(
            v.next,
            "rebase codex/add-discount onto claude/rename-price and stop using price, or keep price on claude/rename-price, then run npx pytxo again."
        );
        let prompt = v.fix_prompt.unwrap();
        assert!(
            prompt.contains("\nclaude/rename-price removed price. codex/add-discount still uses price at src/discount.js:1, src/discount.js:4.\nFix it on codex/add-discount: rebase it onto claude/rename-price, stop using price, and run npm test until it passes. If codex/add-discount needs price, ask for it back on claude/rename-price instead.\n"),
            "{prompt}"
        );
    }

    /// The output does not name it: the why line stays the error, and the
    /// cause is a note, with no prompt to act on.
    #[test]
    fn an_unconfirmed_cause_is_a_note_under_the_error() {
        let out = together(Some(renamed(false)));
        let v = verdict(&out, "run npx pytxo again");
        assert_eq!(
            v.why,
            "npm test fails on the combined change: FAIL discount.test.js: price is not a function"
        );
        assert_eq!(
            v.next,
            "fix one branch so they agree, then run npx pytxo again."
        );
        assert_eq!(
            v.note.as_deref(),
            Some("codex/add-discount uses price, which claude/rename-price renamed to unitPrice (src/discount.js:1).")
        );
        assert_eq!(v.fix_prompt, None);
        assert!(!v.cause.unwrap().confirmed);
        let md = render_markdown(&out);
        assert!(
            md.contains("\n**Note:** `codex/add-discount` uses `price`, which `claude/rename-price` renamed to `unitPrice` (`src/discount.js:1`).\n"),
            "{md}"
        );
        assert!(!md.contains("Fix prompt"), "{md}");
        let removed = Cause {
            renamed_to: None,
            ..renamed(false)
        };
        let v = verdict(&together(Some(removed)), "again");
        assert_eq!(
            v.note.as_deref(),
            Some("codex/add-discount uses price, which claude/rename-price removed (src/discount.js:1).")
        );
    }

    #[test]
    fn markdown_names_the_cause_as_code_and_folds_the_prompt() {
        let md = render_markdown(&together(Some(renamed(true))));
        for line in [
            "### ⛔ BLOCKED: `claude/rename-price` and `codex/add-discount` pass npm test alone but fail together",
            "**Why:** `claude/rename-price` renamed `price` to `unitPrice`; `codex/add-discount` still uses `price` (`src/discount.js:1`).",
            "**Next:** rebase `codex/add-discount` onto `claude/rename-price` and use `unitPrice`, then push again.",
            "<details><summary>Fix prompt for <code>codex/add-discount</code></summary>",
            "```text",
            "Pytxo merged claude/rename-price and codex/add-discount and ran npm test: each passes alone, together they fail.",
        ] {
            assert!(md.lines().any(|l| l == line), "{line}\n{md}");
        }
        assert!(
            md.contains("FAIL discount.test.js: price is not a function\n```\n\n</details>\n\n<details><summary>Details</summary>"),
            "{md}"
        );

        // Branch names and output a fork controls: code, a fence, or HTML
        // text in `<code>`, never a mention, link or tag.
        let mut out = together(Some(renamed(true)));
        let p = &mut out.receipt.predicate;
        p.inputs[1].reference = "codex/x-@octocat<b>".into();
        p.checks[1].scope = "input:codex/x-@octocat<b>".into();
        p.checks[3]
            .output_tail
            .push("```\n@octocat [docs](evil.example)".into());
        let md = render_markdown(&out);
        assert!(
            md.contains(
                "<summary>Fix prompt for <code>codex/x-&#64;octocat&lt;b&gt;</code></summary>"
            ),
            "{md}"
        );
        assert!(md.contains("\n````text\n"), "{md}");
        let mut fence: Option<String> = None;
        for line in md.lines() {
            let ticks: String = line.chars().take_while(|&c| c == '`').collect();
            match &fence {
                Some(open) if line == open.as_str() => fence = None,
                Some(_) => continue,
                None if ticks.len() >= 3 => {
                    fence = Some(ticks);
                    continue;
                }
                None => {}
            }
            let outside: String = line.split('`').step_by(2).collect();
            assert!(!outside.contains('@') && !outside.contains("]("), "{line}");
        }
    }

    /// The hook hands the prompt to the agent whose worktree it runs in.
    #[test]
    fn the_prompt_speaks_to_the_branch_that_reads_it() {
        let out = together(Some(renamed(true)));
        let user = fix_prompt(&out, Reader::User).unwrap();
        assert!(
            user.starts_with("Pytxo merged claude/rename-price and your branch codex/add-discount and ran npm test: each passes alone, together they fail.\nclaude/rename-price renamed price to unitPrice. Your branch codex/add-discount still uses price at src/discount.js:1, src/discount.js:4.\nFix it on your branch: rebase it onto claude/rename-price, replace price with unitPrice, and run npm test until it passes."),
            "{user}"
        );
        let by = fix_prompt(&out, Reader::By).unwrap();
        assert!(
            by.starts_with("Pytxo merged your branch claude/rename-price and codex/add-discount and ran npm test: each passes alone, together they fail.\nYour branch claude/rename-price renamed price to unitPrice. codex/add-discount still uses price at src/discount.js:1, src/discount.js:4.\nThe fix belongs on codex/add-discount: rebase it onto claude/rename-price, replace price with unitPrice, and run npm test until it passes. The other way out is to keep price on your branch as an alias of unitPrice.\n"),
            "{by}"
        );
        assert_eq!(
            fix_prompt(&out, Reader::Anyone),
            verdict(&out, "x").fix_prompt
        );
        // None for a cause the output does not confirm, or another verdict.
        assert_eq!(
            fix_prompt(&together(Some(renamed(false))), Reader::User),
            None
        );
        let mut conflict = together(Some(renamed(true)));
        conflict.receipt.predicate.inputs[1].status = "conflict".into();
        assert_eq!(fix_prompt(&conflict, Reader::User), None);
        assert_eq!(verdict(&conflict, "x").cause, None);
    }

    /// What the runs on the cause's two branches and on the others say, as
    /// the verdict and the Action's tag read them.
    #[test]
    fn the_runs_on_the_pair_and_the_rest_decide_who_is_blamed() {
        let failed = |scope: &str| ReceiptCheck {
            output_tail: lines("TypeError: price is not a function"),
            ..check(scope, false)
        };
        // a, b, c and d each pass alone and fail together; c renamed what a uses.
        let with = |culprits: Vec<ReceiptCheck>| {
            let mut p = blocked("merged", vec![failed("combined")]);
            p.inputs.push(input("d", "merged"));
            p.checks
                .extend(["a", "b", "c", "d"].map(|i| check(&format!("input:{i}"), true)));
            p.checks.push(ReceiptCheck {
                rerun: true,
                ..failed("combined")
            });
            p.checks.extend(culprits);
            let cause = Cause {
                by: 2,
                user: 0,
                ..renamed(true)
            };
            LandOutcome {
                cause: Some(cause),
                ..outcome(p)
            }
        };
        let blamed =
            |out: &LandOutcome| implicated(&out.receipt.predicate, out.cause.as_ref(), "BLOCKED");

        let out = with(vec![failed("pair"), check("rest", true)]);
        assert_eq!(
            verdict(&out, "again").what,
            "a and c fail together; the other 2 branches pass without them."
        );
        assert_eq!(blamed(&out), [0, 2]);
        // No run on the rest: the two are proven, but no one is cleared.
        let out = with(vec![failed("pair")]);
        assert_eq!(verdict(&out, "again").what, "a and c fail together.");
        assert_eq!(blamed(&out), [0, 1, 2, 3]);
        // A run on the pair that did not finish proves nothing.
        let cut = ReceiptCheck {
            timed_out: true,
            exit_code: None,
            ..failed("pair")
        };
        let out = with(vec![cut]);
        assert_eq!(
            verdict(&out, "again").what,
            "a, b, c and d pass npm test alone but fail together."
        );
        assert_eq!(blamed(&out), [0, 1, 2, 3]);
        // Another check that fails together, with no pair of its own, still
        // blames them all.
        let mut out = with(vec![failed("pair"), check("rest", true)]);
        let lint = |c: ReceiptCheck| ReceiptCheck {
            command: "npm run lint".into(),
            ..c
        };
        let p = &mut out.receipt.predicate;
        p.checks.push(lint(failed("combined")));
        p.checks
            .extend(["a", "b", "c", "d"].map(|i| lint(check(&format!("input:{i}"), true))));
        p.checks.push(lint(rerun(false)));
        assert_eq!(blamed(&out), [0, 1, 2, 3]);
    }
}
