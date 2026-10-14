//! Terminal and pull-request (Markdown) summaries of a landing outcome.

use std::fmt::Write as _;

use crate::{short, LandOutcome};

fn seconds(ms: u64) -> String {
    format!("{:.1}s", ms as f64 / 1000.0)
}

fn input_name(reference: &str, agent: Option<&str>) -> String {
    agent.map_or_else(|| reference.to_string(), |a| format!("{a}:{reference}"))
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
                "separate symbols".to_string()
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
        }
    }
    for insight in &outcome.insights {
        let _ = writeln!(s, "\n! {insight}");
    }
    let _ = writeln!(s, "\ndecision  {}", p.decision.to_uppercase());
    for r in &p.reasons {
        let _ = writeln!(s, "  - {r}");
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
    }
    if !p.reasons.is_empty() {
        let _ = writeln!(s, "\n**Why:**");
        for r in &p.reasons {
            let _ = writeln!(s, "- {r}");
        }
    }
    let _ = writeln!(
        s,
        "\n<sub>Receipt `{}` · digest `{}`</sub>",
        p.id, p.content_digest
    );
    s
}
