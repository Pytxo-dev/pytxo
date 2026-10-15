//! Terminal and pull-request (Markdown) summaries of a landing outcome.

use std::fmt::Write as _;

use crate::{short, LandOutcome};

fn seconds(ms: u64) -> String {
    format!("{:.1}s", ms as f64 / 1000.0)
}

fn input_name(reference: &str, agent: Option<&str>) -> String {
    agent.map_or_else(|| reference.to_string(), |a| format!("{a}:{reference}"))
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
                c.scope,
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

pub fn render_markdown(outcome: &LandOutcome) -> String {
    let p = &outcome.receipt.predicate;
    let mark = match p.decision.as_str() {
        "ready" | "landed" => "✅",
        "stale" => "⏸️",
        _ => "⛔",
    };
    let mut s = String::new();
    let _ = writeln!(
        s,
        "### {mark} Pytxo land: **{}** ({})\n",
        p.decision,
        if p.mode == "shadow" {
            "shadow, nothing changed"
        } else {
            "applied"
        }
    );
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
                c.scope,
                result,
                seconds(c.duration_ms)
            );
        }
        for c in p.checks.iter().filter(|c| !c.passed) {
            let lines = why_lines(&c.output_tail);
            if !lines.is_empty() {
                let _ = writeln!(
                    s,
                    "\n`{}` ({}):\n```\n{}\n```",
                    c.command,
                    c.scope,
                    lines.join("\n")
                );
            }
        }
        if let Some(names) = left_out_of_combined(outcome) {
            let _ = writeln!(s, "\nCombined checks leave out {names}, which conflicted.");
        }
    }
    if !p.reasons.is_empty() {
        let _ = writeln!(s, "\n**Why:**");
        for r in &p.reasons {
            let _ = writeln!(s, "- {r}");
        }
    }
    if let Some(next) = next_step(outcome) {
        let _ = writeln!(s, "\n**Next:** {next}.");
    }
    let _ = writeln!(
        s,
        "\n<sub>Receipt `{}` · digest `{}`</sub>",
        p.id, p.content_digest
    );
    s
}
