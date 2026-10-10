//! `pytxo land`, `pytxo review`, `pytxo apply`, `pytxo receipts`.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use pytxo_land::{land, render_markdown, render_text, LandInput, LandOptions};

pub struct LandArgs {
    pub inputs: Vec<String>,
    pub base: Option<String>,
    pub into: Option<String>,
    pub checks: Vec<String>,
    pub per_input: bool,
    pub apply: bool,
    pub strict: bool,
    pub timeout_secs: u64,
    pub json: bool,
    pub markdown: bool,
    pub repo: Option<PathBuf>,
}

/// Runs the gate and returns the process exit code (0 ready/landed, 2 blocked, 3 stale).
pub fn run_land(args: LandArgs) -> Result<i32> {
    let repo = args.repo.unwrap_or(std::env::current_dir()?);
    let inputs = expand_inputs(&repo, &args.inputs)?;
    let mut opts = LandOptions::new(repo, inputs);
    opts.base = args.base;
    opts.into = args.into;
    opts.checks = args.checks;
    opts.per_input = args.per_input;
    opts.apply = args.apply;
    opts.strict = args.strict;
    opts.check_timeout = Duration::from_secs(args.timeout_secs.max(1));
    let outcome = land(&opts)?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "decision": outcome.decision(),
                "exit_code": outcome.exit_code(),
                "insights": outcome.insights,
                "combined_ref": outcome.combined_ref,
                "receipt_path": outcome.receipt_path,
                "receipt": outcome.receipt,
            }))?
        );
    } else if args.markdown {
        print!("{}", render_markdown(&outcome));
    } else {
        print!("{}", render_text(&outcome));
    }
    Ok(outcome.exit_code())
}

/// `agent/*`-style patterns expand to matching local branches, keeping any
/// `label:` prefix; everything else passes through unchanged.
fn expand_inputs(repo: &std::path::Path, raw: &[String]) -> Result<Vec<LandInput>> {
    let git = pytxo_land::Git::user(repo);
    let mut out = Vec::new();
    for arg in raw {
        let parsed = LandInput::parse(arg);
        if !parsed.reference.contains(['*', '?', '[']) {
            out.push(parsed);
            continue;
        }
        let pattern = format!("refs/heads/{}", parsed.reference);
        let names = git.lines(&["for-each-ref", "--format=%(refname:short)", &pattern])?;
        if names.is_empty() {
            bail!("no branches match {}", parsed.reference);
        }
        out.extend(names.into_iter().map(|reference| LandInput {
            label: parsed.label.clone(),
            reference,
        }));
    }
    Ok(out)
}

pub fn run_review(
    run_id: &str,
    refresh: bool,
    discard: bool,
    json: bool,
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
) -> Result<()> {
    if refresh && discard {
        bail!("choose --refresh or --discard, not both");
    }
    if discard {
        pytxo_orchestrate::discard_run_review(config, repo, run_id)?;
        println!("Discarded the prepared change for {run_id}. Nothing was written to the project.");
        return Ok(());
    }
    if refresh {
        let manifest = pytxo_orchestrate::refresh_run_review(config.clone(), repo.clone(), run_id)?;
        println!(
            "Refreshed review for {run_id}: new digest {}",
            manifest.package_digest
        );
    }
    let (_, store) = pytxo_orchestrate::open_store(config, repo)?;
    let contract = store.get_run_contract(run_id)?.with_context(|| {
        format!("run {run_id} has no review (only completed Orbit/Galaxy runs do)")
    })?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": contract.run_id,
                "apply_status": contract.apply_status,
                "base_revision": contract.base_revision,
                "package_digest": contract.prepared_digest,
                "prepared_at": contract.prepared_at,
                "manifest": contract.prepared_manifest,
                "last_apply_error": contract.last_apply_error,
            }))?
        );
        return Ok(());
    }
    println!("run      {}", contract.run_id);
    println!("status   {}", contract.apply_status);
    if let Some(base) = &contract.base_revision {
        println!("base     {base}");
    }
    let Some(manifest) = &contract.prepared_manifest else {
        println!(
            "No prepared change. Status {} has nothing to review.",
            contract.apply_status
        );
        return Ok(());
    };
    let s = &manifest.summary;
    println!(
        "change   {} added · {} modified · {} deleted · {} bytes",
        s.added, s.modified, s.deleted, s.bytes
    );
    println!("\nfiles");
    for f in &manifest.files {
        println!(
            "  {:<9} {}  ({})",
            format!("{:?}", f.kind).to_lowercase(),
            f.path,
            f.task_id
        );
    }
    if let Some(v) = &manifest.candidate_verification {
        println!("\nchecks on the combined change");
        for c in &v.checks {
            println!(
                "  {}  {}",
                if c.passed { "pass" } else { "FAIL" },
                c.command
            );
        }
    } else {
        println!("\nchecks on the combined change: none recorded");
    }
    if let Some(e) = &contract.last_apply_error {
        println!("\nlast Apply error: {} ({})", e.message, e.code);
    }
    if let Some(digest) = &contract.prepared_digest {
        println!("\ndigest   {digest}");
        if contract.apply_status == "ready" {
            println!("apply    pytxo apply {run_id} --digest {digest}");
        } else if matches!(contract.apply_status.as_str(), "stale" | "review_failed") {
            println!("refresh  pytxo review {run_id} --refresh");
        }
    }
    Ok(())
}

pub fn run_apply(
    run_id: &str,
    digest: Option<String>,
    yes: bool,
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
) -> Result<()> {
    let digest = match digest {
        Some(d) => d,
        None => {
            // Authorize exactly the package shown here, never "whatever is latest".
            let (_, store) = pytxo_orchestrate::open_store(config.clone(), repo.clone())?;
            let contract = store
                .get_run_contract(run_id)?
                .with_context(|| format!("run {run_id} has no review"))?;
            let shown = contract
                .prepared_digest
                .clone()
                .context("run has no prepared change to apply")?;
            let files = contract
                .prepared_manifest
                .as_ref()
                .map_or(0, |m| m.files.len());
            println!("Apply {files} reviewed file(s) from {run_id}\ndigest {shown}");
            if !yes && !confirm("Write exactly these files to the project?")? {
                println!("Nothing written.");
                return Ok(());
            }
            shown
        }
    };
    let applied = pytxo_orchestrate::apply_run_changes(config, repo, run_id, &digest)?;
    println!(
        "Applied {} file(s) from {run_id} (transaction {}).",
        applied.changes.len(),
        applied.transaction_id
    );
    Ok(())
}

fn confirm(question: &str) -> Result<bool> {
    print!("{question} [y/N] ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(matches!(line.trim(), "y" | "Y" | "yes" | "YES"))
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
        println!("No receipts yet. `pytxo land <branch>...` and every reviewed Apply write one.");
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
