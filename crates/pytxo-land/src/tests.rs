use std::path::Path;
use std::process::Command;

use super::*;

pub(crate) fn git(dir: &Path, args: &[&str]) -> String {
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

pub(crate) fn write(dir: &Path, files: &[(&str, &str)]) {
    for (path, body) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
}

pub(crate) const CART: &str = "function price(item) {\n  return item.cost;\n}\n\nfunction total(items) {\n  let sum = 0;\n  for (const i of items) sum += price(i);\n  return sum;\n}\n\nfunction label(n) {\n  return 'total: ' + n;\n}\n\nmodule.exports = { price, total, label };\n";

pub(crate) fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    git(d, &["init", "-q", "-b", "main"]);
    git(d, &["config", "user.email", "test@pytxo.invalid"]);
    git(d, &["config", "user.name", "Test"]);
    git(d, &["config", "core.autocrlf", "false"]);
    write(d, &[("src/cart.js", CART), ("README.md", "# shop\n")]);
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "base"]);
    dir
}

/// Commit `files` on a new branch cut from `from`, then return to main.
pub(crate) fn branch(d: &Path, from: &str, name: &str, files: &[(&str, &str)]) {
    git(d, &["checkout", "-q", "-b", name, from]);
    write(d, files);
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", name]);
    git(d, &["checkout", "-q", "main"]);
}

fn opts(d: &Path, inputs: &[&str]) -> LandOptions {
    let mut o = LandOptions::new(d, inputs.iter().map(|i| LandInput::parse(i)).collect());
    o.receipts_dir = Some(d.join(".receipts-test"));
    o
}

#[test]
fn input_labels_parse() {
    assert_eq!(
        LandInput::parse("codex:agent/a").label.as_deref(),
        Some("codex")
    );
    assert_eq!(LandInput::parse("agent/a").label, None);
    assert_eq!(LandInput::parse("weird label:x").label, None);
}

#[test]
fn shadow_mode_combines_disjoint_inputs_and_changes_nothing() {
    let dir = repo();
    let d = dir.path();
    branch(
        d,
        "main",
        "agent/a",
        &[("src/a.js", "module.exports = 1;\n")],
    );
    branch(d, "main", "agent/b", &[("docs/b.md", "b\n")]);
    let before = git(d, &["rev-parse", "main"]);

    let mut o = opts(d, &["codex:agent/a", "claude:agent/b"]);
    o.checks = vec!["exit 0".into()];
    let out = land(&o).unwrap();

    assert_eq!(out.decision(), "ready", "{}", render_text(&out));
    assert_eq!(out.exit_code(), 0);
    assert_eq!(
        git(d, &["rev-parse", "main"]),
        before,
        "shadow must not move main"
    );
    assert!(
        !d.join("src/a.js").exists(),
        "shadow must not touch the checkout"
    );
    let p = &out.receipt.predicate;
    assert_eq!(p.files_changed, 2);
    assert!(p.shared_files.is_empty());
    assert_eq!(p.inputs[0].agent.as_deref(), Some("codex"));
    assert!(out.receipt.digest_matches());
    assert!(out.receipt_path.exists());
    let combined = out.combined_ref.as_deref().unwrap();
    assert_eq!(git(d, &["rev-parse", combined]), p.result.clone().unwrap());
    // The combined commit contains both inputs.
    git(d, &["cat-file", "-e", &format!("{combined}:src/a.js")]);
    git(d, &["cat-file", "-e", &format!("{combined}:docs/b.md")]);
    assert!(render_markdown(&out)
        .starts_with("### ✅ READY: `agent/a` and `agent/b` pass exit 0 together\n"));
}

#[test]
fn same_file_separate_functions_versus_same_function() {
    let dir = repo();
    let d = dir.path();
    let a = CART.replace("return item.cost;", "return item.cost * 1;");
    let b = CART.replace("return 'total: ' + n;", "return 'Total: ' + n;");
    let c = CART.replace("let sum = 0;", "let sum = 0; // start");
    branch(d, "main", "agent/a", &[("src/cart.js", &a)]);
    branch(d, "main", "agent/b", &[("src/cart.js", &b)]);
    branch(d, "main", "agent/c", &[("src/cart.js", &c)]);

    let separate = land(&opts(d, &["agent/a", "agent/b"])).unwrap();
    assert_eq!(separate.decision(), "ready");
    let shared = &separate.receipt.predicate.shared_files;
    assert_eq!(shared.len(), 1);
    assert!(shared[0].symbol_collisions.is_empty(), "{shared:?}");
    // Claims only what was found: edits outside any named symbol are possible.
    assert!(render_text(&separate).contains("no shared symbol"));

    // b edits label(); c edits total(): still separate. a + c both inside different
    // functions too, so build a real same-symbol case: two edits inside total().
    let c2 = CART.replace("return sum;", "return Math.round(sum);");
    branch(d, "main", "agent/c2", &[("src/cart.js", &c2)]);
    let same = land(&opts(d, &["agent/c", "agent/c2"])).unwrap();
    assert_eq!(same.decision(), "ready", "collisions only warn by default");
    let col = &same.receipt.predicate.shared_files[0].symbol_collisions;
    assert_eq!(col.len(), 1);
    assert_eq!(col[0].symbol, "total");
    assert!(same.insights.iter().any(|i| i.contains("total")));

    let mut strict = opts(d, &["agent/c", "agent/c2"]);
    strict.strict = true;
    let strict = land(&strict).unwrap();
    assert_eq!(strict.decision(), "blocked");
    assert_eq!(strict.exit_code(), 2);
}

#[test]
fn textual_conflict_blocks() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("README.md", "# shop A\n")]);
    branch(d, "main", "agent/b", &[("README.md", "# shop B\n")]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec!["echo ok".into()];
    let out = land(&o).unwrap();
    assert_eq!(out.decision(), "blocked");
    let p = &out.receipt.predicate;
    assert_eq!(p.inputs[1].status, "conflict");
    assert_eq!(p.inputs[1].conflict_files, ["README.md"]);
    assert!(p.reasons[0].contains("agent/b conflicts"));
    assert!(render_text(&out).contains("combined leaves out agent/b (conflict)"));
}

#[test]
fn every_input_passes_alone_but_the_combination_fails() {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("node not found; skipping semantic-break test");
        return;
    }
    let dir = repo();
    let d = dir.path();
    let test = "const c = require('./src/cart.js');\nconst extra = require('./src/extra.js');\nif (extra() !== 2) process.exit(1);\n";
    write(
        d,
        &[
            ("check.js", test),
            ("src/extra.js", "module.exports = () => 2;\n"),
        ],
    );
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "check"]);
    // a renames price -> unitPrice everywhere it knows about.
    let renamed = CART.replace("price", "unitPrice");
    branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
    // b adds a new caller of price() in another file.
    branch(
        d,
        "main",
        "agent/caller",
        &[(
            "src/extra.js",
            "const { price } = require('./cart.js');\nmodule.exports = () => price({ cost: 2 });\n",
        )],
    );

    let mut o = opts(d, &["agent/rename", "agent/caller"]);
    o.checks = vec!["node check.js".into()];
    o.per_input = PerInput::Always;
    let out = land(&o).unwrap();
    assert_eq!(out.decision(), "blocked", "{}", render_text(&out));
    let checks = &out.receipt.predicate.checks;
    assert!(
        checks
            .iter()
            .filter(|c| c.scope.starts_with("input:"))
            .all(|c| c.passed),
        "{checks:?}"
    );
    assert!(checks.iter().any(|c| c.scope == "combined" && !c.passed));
    assert!(out.insights[0].contains("Every input passed"));
    let cause = out.cause.expect("the rename is the cause");
    assert_eq!((cause.by, cause.user, cause.name.as_str()), (0, 1, "price"));
    assert!(cause.confirmed, "node names price: {checks:?}");
}

/// A check that fails, in cmd and sh alike, only where src/cart.js says
/// `unitPrice` and src/extra.js still says `price`, the way Node fails there.
const RENAME_BREAKS: &str = "git grep -q unitPrice -- src/cart.js && git grep -q -w price -- src/extra.js && echo TypeError: price is not a function && exit 1 || exit 0";

/// A new caller of `price()`, which `agent/rename` renames.
const CALLER: &str =
    "const { price } = require('./cart.js');\nmodule.exports = () => price({ cost: 2 });\n";

/// `agent/rename` renames `price`, `agent/caller` adds a caller of it, and
/// `agent/docs` only adds notes.
fn rename_caller_and_docs(d: &Path) {
    branch(d, "main", "agent/docs", &[("docs/notes.md", "notes\n")]);
    let renamed = CART.replace("price", "unitPrice");
    branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
    branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
}

/// The runs on the cause's pair and on the rest, and whether each passed.
fn culprit_runs(out: &LandOutcome) -> Vec<(&str, bool)> {
    out.receipt
        .predicate
        .checks
        .iter()
        .filter_map(|c| {
            let kind = c.scope.split_once(':')?.0;
            ["pair", "rest"].contains(&kind).then_some((kind, c.passed))
        })
        .collect()
}

/// Whether Pytxo 2.0 says the digest of the receipt `out` wrote matches.
/// 2.0 parses a receipt into its structs, copied here from cdcf19d, and
/// hashes what they kept, so it matches only when they keep every field: a
/// field 2.1 adds would read as an edit.
fn digest_matches_in_2_0(out: &LandOutcome) -> bool {
    use serde::{Deserialize, Serialize};
    #[derive(Deserialize, Serialize)]
    struct Predicate {
        id: String,
        kind: String,
        created_at: String,
        pytxo_version: String,
        decision: String,
        mode: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        base: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result: Option<String>,
        #[serde(default)]
        inputs: Vec<Input>,
        #[serde(default)]
        files_changed: usize,
        #[serde(default)]
        shared_files: Vec<SharedFile>,
        #[serde(default)]
        checks: Vec<Check>,
        #[serde(default)]
        reasons: Vec<String>,
        #[serde(default)]
        duration_ms: u64,
        #[serde(default)]
        content_digest: String,
    }
    #[derive(Deserialize, Serialize)]
    struct Input {
        #[serde(rename = "ref")]
        reference: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent: Option<String>,
        commit: String,
        status: String,
        files: Vec<String>,
        #[serde(default)]
        conflict_files: Vec<String>,
    }
    #[derive(Deserialize, Serialize)]
    struct SharedFile {
        path: String,
        inputs: Vec<String>,
        #[serde(default)]
        symbol_collisions: Vec<SymbolCollision>,
    }
    #[derive(Deserialize, Serialize)]
    struct SymbolCollision {
        symbol: String,
        inputs: Vec<String>,
    }
    #[derive(Deserialize, Serialize)]
    struct Check {
        scope: String,
        command: String,
        exit_code: Option<i32>,
        passed: bool,
        duration_ms: u64,
        #[serde(default)]
        timed_out: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        output_tail: Vec<String>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        rerun: bool,
    }
    let written: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&out.receipt_path).unwrap()).unwrap();
    let predicate = &written["predicate"];
    let kept: Predicate = serde_json::from_value(predicate.clone()).unwrap();
    &serde_json::to_value(kept).unwrap() == predicate
}

/// The last line of the pull-request comment, which the Action reads.
fn verdict_line(out: &LandOutcome) -> String {
    let md = render_markdown(out);
    md.trim_end().lines().last().unwrap().to_string()
}

/// A frame the check prints with the scratch worktree's full path reads
/// from its root, as in the branch's own checkout, and its place still
/// confirms the cause.
#[test]
fn a_frame_in_the_scratch_worktree_reads_from_its_root_and_confirms() {
    let dir = repo();
    let d = dir.path();
    let renamed = CART.replace("price", "unitPrice");
    branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
    branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
    let mut o = opts(d, &["agent/rename", "agent/caller"]);
    // An error that leaves the name out: only the frame's place confirms.
    #[cfg(windows)]
    let frame = r"echo     at module.exports (%CD%\src\extra.js:2:24)";
    #[cfg(not(windows))]
    let frame = r#"echo "    at module.exports ($PWD/src/extra.js:2:24)""#;
    let said = format!("echo TypeError: not a function && {frame}");
    o.checks = vec![RENAME_BREAKS.replace("echo TypeError: price is not a function", &said)];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert!(out.cause.as_ref().is_some_and(|c| c.confirmed), "{text}");
    let checks = &out.receipt.predicate.checks;
    let combined = checks.iter().find(|c| c.scope == "combined").unwrap();
    let tail = &combined.output_tail;
    let frame = tail.iter().find(|l| l.contains("module.exports")).unwrap();
    assert_eq!(
        frame.trim().replace('\\', "/"),
        "at module.exports (src/extra.js:2:24)"
    );
}

#[test]
fn a_failure_together_names_the_rename_behind_it() {
    let dir = repo();
    let d = dir.path();
    let renamed = CART.replace("price", "unitPrice");
    branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
    branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
    let mut o = opts(d, &["agent/rename", "agent/caller"]);
    o.checks = vec![RENAME_BREAKS.into()];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert_eq!(
        combined_runs(&out),
        [(false, false), (false, true)],
        "{text}"
    );
    // Two branches are the pair already: nothing more runs.
    assert!(culprit_runs(&out).is_empty(), "{text}");
    assert_eq!(
        out.cause,
        Some(Cause {
            by: 0,
            user: 1,
            name: "price".into(),
            renamed_to: Some("unitPrice".into()),
            uses: [1, 2]
                .map(|line| Use {
                    path: "src/extra.js".into(),
                    line,
                })
                .to_vec(),
            confirmed: true,
            also: vec![],
            involved: vec![],
        })
    );
    let v = verdict(&out, "run npx pytxo again");
    assert_eq!(
        v.why,
        "agent/rename renamed price to unitPrice; agent/caller still uses price (src/extra.js:1)."
    );
    // A reading of the runs, not a record: the receipt is as before.
    let written: Receipt =
        serde_json::from_slice(&std::fs::read(&out.receipt_path).unwrap()).unwrap();
    assert!(written.digest_matches());
    assert_eq!(written.predicate, out.receipt.predicate);

    // Without the runs alone nothing shows they pass alone: no cause.
    o.per_input = PerInput::Never;
    assert_eq!(land(&o).unwrap().cause, None);

    // A check the caller fails alone, run first, is the one the verdict
    // speaks of, so the one that fails only together has no cause either.
    o.per_input = PerInput::OnFailure;
    o.checks = vec![
        "git grep -q -w price -- src/extra.js && exit 1 || exit 0".into(),
        RENAME_BREAKS.into(),
    ];
    let out = land(&o).unwrap();
    assert_eq!(
        verdict(&out, "again").kind,
        "alone",
        "{}",
        render_text(&out)
    );
    assert_eq!(out.cause, None);
}

/// The error a long run prints above its last 40 lines, such as a Go
/// module's one failed package among many, still confirms the cause and is
/// the why, while the receipt keeps only the last 40 lines.
#[test]
fn an_error_above_a_long_runs_tail_still_confirms_the_cause() {
    let dir = repo();
    let d = dir.path();
    let packages: String = (1..=45)
        .map(|n| format!("ok   shop/pkg{n} 0.1s\n"))
        .collect();
    write(d, &[("packages.txt", &packages)]);
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "packages"]);
    let renamed = CART.replace("price", "unitPrice");
    branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
    branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
    let mut o = opts(d, &["agent/rename", "agent/caller"]);
    o.checks = vec![RENAME_BREAKS.replace("&& exit 1", "&& git show HEAD:packages.txt && exit 1")];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert!(out.cause.as_ref().is_some_and(|c| c.confirmed), "{text}");
    let checks = &out.receipt.predicate.checks;
    let combined = checks.iter().find(|c| c.scope == "combined").unwrap();
    assert_eq!(combined.output_tail.len(), 40, "{text}");
    assert!(
        !combined.output_tail.concat().contains("TypeError"),
        "{text}"
    );
    let failed = verdict(&out, "again");
    assert!(
        failed.why.starts_with("agent/rename renamed price"),
        "{text}"
    );
}

/// With a third branch, the two the cause names fail on the target by
/// themselves and the third passes there without them: only the two are
/// blamed, and the verdict clears the third. Its run alone is that run, so
/// it does not run again.
#[test]
fn the_pair_a_cause_names_is_proven_and_the_others_cleared() {
    let dir = repo();
    let d = dir.path();
    rename_caller_and_docs(d);
    let mut o = opts(d, &["agent/docs", "agent/rename", "agent/caller"]);
    o.checks = vec![RENAME_BREAKS.into()];
    o.per_input = PerInput::OnFailure;
    // A deadline with room for them changes nothing.
    o.deadline = Some(Instant::now() + Duration::from_secs(600));
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert_eq!(culprit_runs(&out), [("pair", false)], "{text}");
    assert!(!out.out_of_time);
    let cause = out.cause.as_ref().unwrap();
    assert_eq!((cause.by, cause.user, cause.confirmed), (1, 2, true));
    let v = verdict(&out, "run npx pytxo again");
    assert_eq!(
        v.to_string(),
        "BLOCKED  agent/rename and agent/caller fail together; agent/docs passes without them.
why      agent/rename renamed price to unitPrice; agent/caller still uses price (src/extra.js:1).
next     rebase agent/caller onto agent/rename and use unitPrice, then run npx pytxo again.
"
    );
    assert_eq!(
        verdict_line(&out),
        r#"<!-- pytxo-verdict {"label":"BLOCKED","implicated":[1,2]} -->"#
    );
    // The pair's run says what it merged, and the tables name them.
    let pair = out.receipt.predicate.checks.last().unwrap();
    assert_eq!(pair.scope, "pair:1,2");
    let md = render_markdown(&out);
    assert!(
        md.contains("| `pair (agent/rename, agent/caller)` | **fail** |") && !md.contains("`rest"),
        "{md}"
    );
    assert!(
        text.contains("\n  pair (agent/rename, agent/caller) ") && !text.contains("\n  rest"),
        "{text}"
    );
    // The pair's run is in the written receipt, and its digest verifies, in
    // 2.0 too.
    let written: Receipt =
        serde_json::from_slice(&std::fs::read(&out.receipt_path).unwrap()).unwrap();
    assert!(written.digest_matches());
    assert_eq!(written.predicate.checks, out.receipt.predicate.checks);
    assert!(digest_matches_in_2_0(&out), "{text}");
}

/// The two the cause names pass by themselves; the failure takes the third
/// as well. The guess was wrong: the verdict is the one without a cause,
/// and a note says the two passed.
#[test]
fn a_pair_that_passes_by_itself_leaves_the_verdict_as_it_was() {
    let dir = repo();
    let d = dir.path();
    rename_caller_and_docs(d);
    // Breaks only where the docs branch's notes are too.
    let check = RENAME_BREAKS.replacen(
        " && echo",
        " && git grep -q notes -- docs/notes.md && echo",
        1,
    );
    let mut o = opts(d, &["agent/docs", "agent/rename", "agent/caller"]);
    o.checks = vec![check.clone()];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert_eq!(culprit_runs(&out), [("pair", true)], "{text}");
    assert!(out.cause.is_some(), "{text}");
    let v = verdict(&out, "run npx pytxo again");
    assert_eq!(
        v.what,
        format!("agent/docs, agent/rename and agent/caller pass {check} alone but fail together.")
    );
    assert_eq!(
        v.why,
        format!("{check} fails on the combined change: TypeError: price is not a function")
    );
    assert_eq!(
        v.next,
        "fix one branch so they agree, then run npx pytxo again."
    );
    assert_eq!(
        v.note,
        Some(format!(
            "agent/rename and agent/caller pass {check} together without the other branches."
        ))
    );
    assert_eq!((v.cause, v.fix_prompt), (None, None));
    assert_eq!(
        verdict_line(&out),
        r#"<!-- pytxo-verdict {"label":"BLOCKED","implicated":[0,1,2]} -->"#
    );
}

/// The two the cause names fail by themselves, and the others fail without
/// them too: everyone stays blamed, and the verdict names the two.
#[test]
fn others_that_fail_without_the_pair_stay_blamed() {
    let dir = repo();
    let d = dir.path();
    let renamed = CART.replace("price", "unitPrice");
    branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
    branch(d, "main", "agent/c", &[("c.txt", "c\n")]);
    branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
    branch(d, "main", "agent/d", &[("d.txt", "d\n")]);
    // Breaks where c.txt and d.txt both are, too.
    let check = RENAME_BREAKS.replacen(
        "|| exit 0",
        "|| git grep -q c -- c.txt && git grep -q d -- d.txt && echo boom && exit 1 || exit 0",
        1,
    );
    let mut o = opts(d, &["agent/rename", "agent/c", "agent/caller", "agent/d"]);
    o.checks = vec![check];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert_eq!(
        culprit_runs(&out),
        [("pair", false), ("rest", false)],
        "{text}"
    );
    // The others' own failure, not the pair's. (cmd echoes the space before `&&`.)
    let rest = out.receipt.predicate.checks.last().unwrap();
    assert_eq!(rest.output_tail.concat().trim(), "boom", "{text}");
    assert_eq!(rest.scope, "rest:1,3");
    assert!(text.contains("\n  rest (agent/c, agent/d) "), "{text}");
    assert!(digest_matches_in_2_0(&out), "{text}");
    let cause = out.cause.as_ref().unwrap();
    assert_eq!((cause.by, cause.user), (0, 2));
    let v = verdict(&out, "run npx pytxo again");
    assert_eq!(
        v.what,
        "agent/rename and agent/caller fail together; the other 2 branches fail without them too."
    );
    assert_eq!(
        v.why,
        "agent/rename renamed price to unitPrice; agent/caller still uses price (src/extra.js:1)."
    );
    assert_eq!(
        verdict_line(&out),
        r#"<!-- pytxo-verdict {"label":"BLOCKED","implicated":[0,1,2,3]} -->"#
    );
}

/// Two branches rename `price` alike and a third still calls it: the pair
/// the cause names fails by itself and the other renamer passes without it,
/// yet that one breaks the caller just the same, so it stays blamed, in
/// either order. A branch that only adds notes is still cleared.
#[test]
fn a_second_branch_that_renamed_the_name_stays_blamed() {
    let dir = repo();
    let d = dir.path();
    rename_caller_and_docs(d);
    let renamed = CART.replace("price", "unitPrice");
    let also = [("src/cart.js", renamed.as_str()), ("b.txt", "b\n")];
    branch(d, "main", "agent/rename2", &also);
    let tag = |implicated: &str| {
        format!(r#"<!-- pytxo-verdict {{"label":"BLOCKED","implicated":[{implicated}]}} -->"#)
    };
    for (inputs, implicated) in [
        (
            &["agent/rename", "agent/rename2", "agent/caller"][..],
            "0,1,2",
        ),
        (&["agent/rename2", "agent/rename", "agent/caller"], "0,1,2"),
        (
            &[
                "agent/docs",
                "agent/rename",
                "agent/rename2",
                "agent/caller",
            ],
            "1,2,3",
        ),
    ] {
        let mut o = opts(d, inputs);
        o.checks = vec![RENAME_BREAKS.into()];
        o.per_input = PerInput::OnFailure;
        let out = land(&o).unwrap();
        let text = render_text(&out);
        assert_eq!(verdict_line(&out), tag(implicated), "{text}");
        let v = verdict(&out, "run npx pytxo again");
        assert!(!v.what.contains("without them"), "{text}");
    }
}

/// Two branches use the name a third renamed, and the check breaks only on
/// one of them: the other passes without the pair, but it uses the name as
/// much, so it stays blamed, and nothing says it passes.
#[test]
fn another_branch_that_uses_the_name_stays_blamed() {
    let dir = repo();
    let d = dir.path();
    rename_caller_and_docs(d);
    let tax = CALLER.replace("cost: 2", "cost: 3");
    branch(d, "main", "agent/tax", &[("src/tax.js", &tax)]);
    let mut o = opts(
        d,
        &["agent/rename", "agent/caller", "agent/tax", "agent/docs"],
    );
    o.checks = vec![RENAME_BREAKS.into()];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert_eq!(
        culprit_runs(&out),
        [("pair", false), ("rest", true)],
        "{text}"
    );
    let cause = out.cause.as_ref().unwrap();
    assert_eq!((cause.by, cause.user, &cause.also), (0, 1, &vec![2]));
    let v = verdict(&out, "run npx pytxo again");
    assert_eq!(
        v.to_string(),
        format!("BLOCKED  agent/rename and agent/caller fail {RENAME_BREAKS} together.
why      agent/rename renamed price to unitPrice; agent/caller still uses price (src/extra.js:1), and so does agent/tax.
next     rebase agent/caller and agent/tax onto agent/rename and use unitPrice, then run npx pytxo again.
")
    );
    assert_eq!(v.cause.unwrap().also, ["agent/tax"]);
    assert!(
        v.fix_prompt
            .unwrap()
            .contains("at src/extra.js:1, src/extra.js:2. agent/tax uses price too.\n"),
        "{text}"
    );
    assert_eq!(
        verdict_line(&out),
        r#"<!-- pytxo-verdict {"label":"BLOCKED","implicated":[0,1,2]} -->"#
    );
}

/// `RENAME_BREAKS`, `secs` seconds slower the first time it fails: the
/// combined run takes that long, and its re-run does not.
fn rename_breaks_slowly_once(marker: &Path, secs: u32) -> String {
    #[cfg(windows)]
    let wait = format!(
        "(type \"{0}\" > nul 2>&1 || (type nul > \"{0}\" & ping -n {1} 127.0.0.1 > nul))",
        marker.display(),
        secs + 1
    );
    #[cfg(not(windows))]
    let wait = format!(
        "{{ test -f '{0}' || {{ touch '{0}' && sleep {secs}; }}; }}",
        marker.display()
    );
    RENAME_BREAKS.replacen(" && echo", &format!(" && {wait} && echo"), 1)
}

/// Two more runs as long as the combined one would end past the deadline:
/// neither starts, and the verdict is the one the runs before gave, in time.
#[test]
fn no_pair_runs_start_when_the_deadline_would_pass_first() {
    let dir = repo();
    let d = dir.path();
    rename_caller_and_docs(d);
    // Two others, so the rest is no run alone and both runs are needed.
    branch(
        d,
        "main",
        "agent/notes",
        &[(
            "docs/more.md",
            "more
",
        )],
    );
    let scratch = tempfile::tempdir().unwrap();
    let four = ["agent/docs", "agent/notes", "agent/rename", "agent/caller"];
    let mut o = opts(d, &four);
    o.checks = vec![rename_breaks_slowly_once(&scratch.path().join("slow"), 10)];
    o.per_input = PerInput::OnFailure;
    // Three times the slow run from the start: once it has run, two more of
    // it never fit, and the git calls, the quick runs and the search for the
    // cause around it have twice its length, which a loaded machine needs.
    o.deadline = Some(Instant::now() + Duration::from_secs(30));
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert!(!out.out_of_time, "{text}");
    assert!(culprit_runs(&out).is_empty(), "{text}");
    assert_eq!(
        combined_runs(&out),
        [(false, false), (false, true)],
        "{text}"
    );
    assert!(out.cause.is_some(), "{text}");
    let v = verdict(&out, "again");
    assert_eq!(
        v.what,
        format!(
            "agent/docs, agent/notes, agent/rename and agent/caller pass {} alone but fail together.",
            o.checks[0]
        )
    );
    assert_eq!(
        verdict_line(&out),
        r#"<!-- pytxo-verdict {"label":"BLOCKED","implicated":[0,1,2,3]} -->"#
    );
}

/// With one other branch its run alone stands for the run on the rest, so
/// a deadline with room for the one run on the pair is enough to clear it.
#[test]
fn one_run_on_the_pair_is_enough_with_one_other_branch() {
    let dir = repo();
    let d = dir.path();
    rename_caller_and_docs(d);
    let scratch = tempfile::tempdir().unwrap();
    let mut o = opts(d, &["agent/docs", "agent/rename", "agent/caller"]);
    o.checks = vec![rename_breaks_slowly_once(&scratch.path().join("slow"), 20)];
    o.per_input = PerInput::OnFailure;
    // Three times the slow run, as above: one more of it fits, two do not.
    // Twenty seconds, for the room a loaded machine needs around it.
    o.deadline = Some(Instant::now() + Duration::from_secs(60));
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert!(!out.out_of_time, "{text}");
    assert_eq!(culprit_runs(&out), [("pair", false)], "{text}");
    assert_eq!(
        verdict_line(&out),
        r#"<!-- pytxo-verdict {"label":"BLOCKED","implicated":[1,2]} -->"#
    );
}

/// The combined run fails at once, but the docs branch's run alone, which
/// passes, takes ten seconds: a run on the pair or the rest can take that
/// long too, so neither starts when the two of them, with two other
/// branches, would end past the deadline.
#[test]
fn no_pair_runs_start_when_a_passing_run_would_end_past_the_deadline() {
    let dir = repo();
    let d = dir.path();
    rename_caller_and_docs(d);
    #[cfg(windows)]
    let wait = "ping -n 11 127.0.0.1 > nul";
    #[cfg(not(windows))]
    let wait = "sleep 10";
    let check = RENAME_BREAKS.replacen(
        "|| exit 0",
        &format!("|| git grep -q notes -- docs/notes.md && {wait} || exit 0"),
        1,
    );
    branch(
        d,
        "main",
        "agent/notes",
        &[(
            "docs/more.md",
            "more
",
        )],
    );
    let four = ["agent/docs", "agent/notes", "agent/rename", "agent/caller"];
    let mut o = opts(d, &four);
    o.checks = vec![check];
    o.per_input = PerInput::OnFailure;
    // Three times the slow run from the start, as above.
    o.deadline = Some(Instant::now() + Duration::from_secs(30));
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert!(!out.out_of_time, "{text}");
    assert!(culprit_runs(&out).is_empty(), "{text}");
    assert_eq!(
        combined_runs(&out),
        [(false, false), (false, true)],
        "{text}"
    );
    let slow = &out.receipt.predicate.checks;
    assert!(
        slow.iter()
            .any(|c| c.scope == "input:agent/docs" && c.passed && c.duration_ms >= 9000),
        "{text}"
    );
    assert!(out.cause.is_some(), "{text}");
}

/// A check that fails only where both `a.txt` and `b.txt` exist, so only on
/// the combination; with `marker`, only until it has failed once.
fn combined_only_failure(marker: Option<&Path>) -> String {
    #[cfg(windows)]
    let check = match marker {
        Some(m) => format!(
            "if not exist a.txt (exit 0) else if not exist b.txt (exit 0) else if exist \"{0}\" (exit 0) else (type nul > \"{0}\" & exit 1)",
            m.display()
        ),
        None => "if not exist a.txt (exit 0) else if not exist b.txt (exit 0) else (exit 1)".into(),
    };
    #[cfg(not(windows))]
    let check = match marker {
        Some(m) => format!(
            "test -f a.txt && test -f b.txt || exit 0; test -f '{0}' && exit 0; touch '{0}'; exit 1",
            m.display()
        ),
        None => "test -f a.txt && test -f b.txt && exit 1; exit 0".into(),
    };
    check
}

fn combined_runs(out: &LandOutcome) -> Vec<(bool, bool)> {
    out.receipt
        .predicate
        .checks
        .iter()
        .filter(|c| c.scope == "combined")
        .map(|c| (c.passed, c.rerun))
        .collect()
}

#[test]
fn a_flaky_combined_check_runs_again_and_is_not_blamed_on_the_combination() {
    // Logflare scan, 2026-10-20: both PRs passed `mix test` alone and the
    // combination failed once, yet a plain merge of the same PRs passed.
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    let scratch = tempfile::tempdir().unwrap();
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec![combined_only_failure(Some(&scratch.path().join("failed")))];
    o.per_input = PerInput::Always;
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert_eq!(out.decision(), "blocked", "{text}");
    assert_eq!(
        combined_runs(&out),
        [(false, false), (true, true)],
        "{text}"
    );
    let said = out.insights.join("\n");
    assert!(!said.contains("The combined change failed"), "{said}");
    assert!(said.contains("the check is flaky"), "{said}");
    let p = &out.receipt.predicate;
    assert!(
        p.reasons[0].starts_with("combined check flaky:"),
        "{:?}",
        p.reasons
    );
    assert!(text.contains("  combined, run 2 "), "{text}");
    assert!(
        text.contains("\nnext      the combined check is flaky"),
        "{text}"
    );
    let markdown = render_markdown(&out);
    assert!(
        markdown.contains("| `combined, run 2` | pass |"),
        "{markdown}"
    );

    // Both runs are in the written receipt, and its digest still verifies.
    let written: Receipt =
        serde_json::from_slice(&std::fs::read(&out.receipt_path).unwrap()).unwrap();
    assert!(written.digest_matches());
    assert_eq!(written.predicate.checks, p.checks);
}

#[test]
fn a_combined_failure_that_repeats_is_blamed_on_the_combination() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec![combined_only_failure(None)];

    // Without --per-input nothing shows the inputs pass alone: one run, no blame.
    let out = land(&o).unwrap();
    assert_eq!(combined_runs(&out), [(false, false)]);
    assert!(out.insights.is_empty(), "{:?}", out.insights);

    o.per_input = PerInput::Always;
    let out = land(&o).unwrap();
    let text = render_text(&out);
    assert_eq!(out.decision(), "blocked", "{text}");
    assert_eq!(
        combined_runs(&out),
        [(false, false), (false, true)],
        "{text}"
    );
    assert_eq!(
        out.insights,
        ["Every input passed its checks alone. The combined change failed both times it ran."]
    );
    let reason = &out.receipt.predicate.reasons[0];
    assert!(
        reason.starts_with("combined check failed twice:")
            && reason.ends_with("(exit 1, then exit 1)"),
        "{reason}"
    );
    assert!(text.contains("  combined, run 2 "), "{text}");
    assert!(text.contains("\nnext      fix the failure"), "{text}");
}

/// An npm project whose test script exists, committed on main.
fn npm_project(d: &Path) {
    write(
        d,
        &[(
            "package.json",
            r#"{"scripts":{"test":"tape test/*.js"},"devDependencies":{"tape":"^5"}}"#,
        )],
    );
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "package"]);
}

#[test]
fn a_check_that_fails_everywhere_points_at_the_check() {
    // First-use run 1 (mri): `--check "npm test"` failed on every input and on
    // the combination because the fresh worktree had no node_modules, and the
    // output only said "FAIL exit 1".
    let dir = repo();
    let d = dir.path();
    npm_project(d);
    branch(
        d,
        "main",
        "agent/a",
        &[("src/a.js", "module.exports = 1;\n")],
    );
    branch(d, "main", "agent/b", &[("docs/b.md", "b\n")]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec!["echo Error: tape is not installed && exit 1".into()];
    o.per_input = PerInput::Always;
    let out = land(&o).unwrap();
    assert_eq!(out.decision(), "blocked");

    let hint = out.insights.join("\n");
    assert!(
        hint.contains("failed on every input alone and on main too"),
        "{hint}"
    );
    assert!(
        hint.contains("--check \"npm install && npm test\""),
        "{hint}"
    );
    let text = render_text(&out);
    assert!(
        text.contains("\n      Error: tape is not installed"),
        "{text}"
    );
    assert!(text.contains("\nnext      fix the failure"), "{text}");
    assert!(
        text.contains(&format!(
            "output    pytxo receipts show {}",
            out.receipt.predicate.id
        )),
        "{text}"
    );
    assert!(render_markdown(&out).contains("**Next:**"));

    // A check that installs first gets no install hint.
    o.checks = vec!["echo npm ci && exit 1".into()];
    let out = land(&o).unwrap();
    assert!(!out
        .insights
        .join("\n")
        .contains("dependencies are not installed"));
}

#[test]
fn a_check_that_passes_alone_gets_no_install_hint() {
    // Show HN prep: `--check "npm test" --per-input` on a dependency-free npm
    // project passed on both inputs and failed twice on the combination, yet
    // the output still blamed missing dependencies.
    let dir = repo();
    let d = dir.path();
    npm_project(d);
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec![combined_only_failure(None)];

    // With no solo runs nothing shows the set-up works, so the hint stays.
    let out = land(&o).unwrap();
    let said = out.insights.join("\n");
    assert!(said.contains("dependencies are not installed"), "{said}");

    // The solo passes prove it does, so only the combination is blamed.
    o.per_input = PerInput::Always;
    let out = land(&o).unwrap();
    assert_eq!(
        out.insights,
        ["Every input passed its checks alone. The combined change failed both times it ran."]
    );
}

#[test]
fn without_checks_the_project_test_command_is_suggested_not_run() {
    let dir = repo();
    let d = dir.path();
    npm_project(d);
    branch(
        d,
        "main",
        "agent/a",
        &[("src/a.js", "module.exports = 1;\n")],
    );
    let out = land(&opts(d, &["agent/a"])).unwrap();
    assert_eq!(out.decision(), "ready");
    assert!(out.receipt.predicate.checks.is_empty(), "nothing ran");
    assert!(
        out.insights
            .iter()
            .any(|i| i.contains("Pass --check \"npm install && npm test\"")),
        "{:?}",
        out.insights
    );
}

#[test]
fn apply_fast_forwards_the_checked_out_branch() {
    let dir = repo();
    let d = dir.path();
    branch(
        d,
        "main",
        "agent/a",
        &[("src/a.js", "module.exports = 1;\n")],
    );
    branch(d, "main", "agent/b", &[("docs/b.md", "b\n")]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.apply = true;
    let out = land(&o).unwrap();
    assert_eq!(out.decision(), "landed", "{}", render_text(&out));
    assert_eq!(
        git(d, &["rev-parse", "main"]),
        out.receipt.predicate.result.clone().unwrap()
    );
    assert!(d.join("src/a.js").exists() && d.join("docs/b.md").exists());
}

#[test]
fn apply_updates_a_branch_that_is_not_checked_out() {
    let dir = repo();
    let d = dir.path();
    git(d, &["branch", "release"]);
    branch(
        d,
        "release",
        "agent/a",
        &[("src/a.js", "module.exports = 1;\n")],
    );
    let mut o = opts(d, &["agent/a"]);
    o.into = Some("release".into());
    o.apply = true;
    let out = land(&o).unwrap();
    assert_eq!(out.decision(), "landed", "{}", render_text(&out));
    assert_eq!(
        git(d, &["rev-parse", "release"]),
        out.receipt.predicate.result.clone().unwrap()
    );
    assert!(!d.join("src/a.js").exists(), "main's checkout is untouched");
}

#[test]
fn a_target_that_moves_during_the_gate_is_refused() {
    let dir = repo();
    let d = dir.path();
    branch(
        d,
        "main",
        "agent/a",
        &[("src/a.js", "module.exports = 1;\n")],
    );
    let before_tip = git(d, &["rev-parse", "main"]);
    let mut o = opts(d, &["agent/a"]);
    o.apply = true;
    // The check commits to main in the user's checkout while the gate runs.
    o.checks = vec![format!(
        "git -C \"{}\" commit -q --allow-empty -m moved",
        d.display().to_string().replace('\\', "/")
    )];
    let out = land(&o).unwrap();
    assert_eq!(out.decision(), "stale", "{}", render_text(&out));
    assert_eq!(out.exit_code(), 3);
    let tip = git(d, &["rev-parse", "main"]);
    assert_ne!(tip, before_tip, "the concurrent commit happened");
    assert_ne!(
        tip,
        out.receipt.predicate.result.clone().unwrap(),
        "and the gate did not overwrite it"
    );
    assert!(!d.join("src/a.js").exists());
}

#[test]
fn scratch_worktrees_are_named_the_way_detection_recognizes() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    let seen = d.join(".cwd-test");
    let mut o = opts(d, &["agent/a"]);
    // Each run appends the worktree it ran in.
    #[cfg(windows)]
    let record = format!("cd >> \"{}\"", seen.display());
    #[cfg(not(windows))]
    let record = format!("pwd >> '{}'", seen.display());
    o.checks = vec![record];
    o.per_input = PerInput::Always;
    assert_eq!(land(&o).unwrap().decision(), "ready");
    let paths = std::fs::read_to_string(&seen).unwrap();
    let paths: Vec<&str> = paths.lines().map(str::trim).collect();
    assert_eq!(
        paths.len(),
        2,
        "one solo run and one combined run: {paths:?}"
    );
    for p in paths {
        assert!(is_scratch_worktree(Path::new(p)), "{p}");
    }
    assert!(!is_scratch_worktree(Path::new("/tmp/pytxo-land-fix")));
    assert!(!is_scratch_worktree(d));
}

#[test]
fn a_deadline_cuts_the_checks_short_and_says_so() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    #[cfg(windows)]
    let slow = "ping -n 30 127.0.0.1 > nul";
    #[cfg(not(windows))]
    let slow = "sleep 30";
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec![slow.into(), "echo never".into()];
    o.per_input = PerInput::OnFailure;
    let started = Instant::now();
    o.deadline = Some(started + Duration::from_secs(4));
    let out = land(&o).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "the run was cut"
    );
    assert!(out.out_of_time);
    assert_eq!(out.decision(), "blocked");
    // Nothing starts after the cut: no second command, no input alone, no
    // re-run. (On a loaded machine the deadline can pass before the first.)
    let checks = &out.receipt.predicate.checks;
    assert!(checks.len() <= 1, "{checks:?}");
    assert!(
        checks
            .iter()
            .all(|c| c.scope == "combined" && c.command == slow && c.timed_out),
        "{checks:?}"
    );
    let reasons = &out.receipt.predicate.reasons;
    assert!(
        reasons.iter().any(|r| r.starts_with("out of time")),
        "{reasons:?}"
    );

    // A deadline that leaves room changes nothing.
    o.checks = vec!["echo ok".into()];
    o.deadline = Some(Instant::now() + Duration::from_secs(600));
    let out = land(&o).unwrap();
    assert!(!out.out_of_time);
    assert_eq!(out.decision(), "ready");
}

#[test]
fn empty_inputs_land_nothing() {
    let dir = repo();
    let d = dir.path();
    git(d, &["branch", "agent/noop"]);
    let out = land(&opts(d, &["agent/noop"])).unwrap();
    assert_eq!(out.decision(), "ready");
    assert_eq!(out.receipt.predicate.files_changed, 0);
    assert_eq!(out.receipt.predicate.inputs[0].status, "empty");
    assert!(out.combined_ref.is_none());
}

/// The three plain lines for each kind of outcome, and the order `OnFailure`
/// runs things in: the combination first, each branch alone only after a failure.
#[test]
fn plain_verdicts_say_what_why_and_next() {
    let again = "run npx pytxo again";
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.per_input = PerInput::OnFailure;

    // No checks: merged, but nothing proved it works.
    let v = verdict(&land(&o).unwrap(), again);
    assert_eq!((v.kind, v.label), ("no_tests", "NO TESTS"));
    assert_eq!(
        v.what,
        "agent/a and agent/b merge cleanly onto main, but no tests ran."
    );
    assert!(v.next.contains("--check"), "{v:?}");

    // A passing combination runs once: no solo runs on a pass.
    o.checks = vec!["echo ok".into()];
    let out = land(&o).unwrap();
    assert_eq!(out.receipt.predicate.checks.len(), 1);
    let v = verdict(&out, again);
    assert_eq!((v.kind, v.label), ("ready", "READY"));
    assert_eq!(v.what, "agent/a and agent/b pass echo ok together.");
    assert!(
        v.next.contains("Nothing in your repository changed"),
        "{v:?}"
    );

    // Fails only together: combined, then each alone, then the combined re-run.
    o.checks = vec![combined_only_failure(None)];
    let out = land(&o).unwrap();
    let scopes: Vec<&str> = out
        .receipt
        .predicate
        .checks
        .iter()
        .map(|c| c.scope.as_str())
        .collect();
    assert_eq!(
        scopes,
        ["input:agent/a", "input:agent/b", "combined", "combined"]
    );
    assert_eq!(combined_runs(&out), [(false, false), (false, true)]);
    let v = verdict(&out, again);
    assert_eq!((v.kind, v.label), ("together", "BLOCKED"));
    assert!(
        v.what.starts_with("agent/a and agent/b pass ")
            && v.what.ends_with(" alone but fail together."),
        "{v:?}"
    );
    assert_eq!(
        v.next,
        "fix one branch so they agree, then run npx pytxo again."
    );
    let text = v.to_string();
    assert_eq!(text.lines().count(), 3, "{text}");
    assert!(
        text.starts_with("BLOCKED  agent/a and agent/b pass "),
        "{text}"
    );
    assert!(text.contains("\nwhy      "), "{text}");
    assert!(text.ends_with("\nnext     fix one branch so they agree, then run npx pytxo again.\n"));

    // Flaky: fails once on the combination, passes when run again.
    let scratch = tempfile::tempdir().unwrap();
    o.checks = vec![combined_only_failure(Some(&scratch.path().join("once")))];
    let out = land(&o).unwrap();
    assert_eq!(out.exit_code(), 2);
    let v = verdict(&out, again);
    assert_eq!((v.kind, v.label), ("flaky", "FLAKY"));

    // Fails everywhere, the target too: not the branches' fault.
    o.checks = vec!["exit 1".into()];
    let out = land(&o).unwrap();
    let scopes: Vec<&str> = out
        .receipt
        .predicate
        .checks
        .iter()
        .map(|c| c.scope.as_str())
        .collect();
    assert_eq!(
        scopes,
        ["input:agent/a", "input:agent/b", "combined", "target"]
    );
    let v = verdict(&out, again);
    assert_eq!(v.kind, "alone");
    assert_eq!(
        v.what,
        "exit 1 fails on main as well, so the check or the environment is broken, not the branches."
    );
    assert_eq!(v.why, "exit 1 exits 1 on main.");
    assert!(render_markdown(&out)
        .ends_with("\n<!-- pytxo-verdict {\"label\":\"BLOCKED\",\"implicated\":[0,1]} -->\n"));

    // Every branch fails alone but the target passes: the branches did it.
    o.checks = vec!["git diff --quiet main HEAD".into()];
    let out = land(&o).unwrap();
    let v = verdict(&out, again);
    assert_eq!(v.kind, "alone");
    assert_eq!(
        v.what,
        "agent/a and agent/b fail git diff --quiet main HEAD on their own, so the combination fails too."
    );
    assert_eq!(
        v.next,
        "fix agent/a and agent/b first, then run npx pytxo again."
    );
    assert!(render_markdown(&out)
        .ends_with("\n<!-- pytxo-verdict {\"label\":\"BLOCKED\",\"implicated\":[0,1]} -->\n"));

    // Without solo runs, the verdict says only what it knows.
    o.per_input = PerInput::Never;
    let v = verdict(&land(&o).unwrap(), again);
    assert_eq!(v.kind, "failed");
    assert!(v.next.contains("--per-input"), "{v:?}");

    // A conflict names both sides and the file.
    branch(d, "main", "agent/c", &[("README.md", "# shop C\n")]);
    branch(d, "main", "agent/d", &[("README.md", "# shop D\n")]);
    let mut o = opts(d, &["agent/c", "agent/d"]);
    o.checks = vec!["echo ok".into()];
    let out = land(&o).unwrap();
    let v = verdict(&out, again);
    assert_eq!((v.kind, v.label), ("conflict", "BLOCKED"));
    assert_eq!(v.what, "agent/d conflicts with agent/c.");
    assert_eq!(
        v.why,
        "git cannot merge them: README.md changed on both sides."
    );

    // Markdown leads with the verdict and ends with the install line.
    let md = render_markdown(&out);
    assert!(
        md.starts_with("### ⛔ BLOCKED: `agent/d` conflicts with `agent/c`\n"),
        "{md}"
    );
    assert!(
        md.contains("**Next:** rebase `agent/d` onto `agent/c`"),
        "{md}"
    );
    assert!(
        md.ends_with(&format!(
            "<sub>{INSTALL_HINT}</sub>\n\n<!-- pytxo-verdict {{\"label\":\"BLOCKED\",\"implicated\":[1]}} -->\n"
        )),
        "{md}"
    );
    assert!(
        !md.contains(&out.receipt.predicate.content_digest),
        "digest only with --verbose"
    );
    assert!(render_markdown_with(&out, true).contains(&out.receipt.predicate.content_digest));
}

/// The pull-request comment gives advice in the GitHub Action's terms; the
/// terminal keeps its flags.
#[test]
fn markdown_advice_fits_a_pull_request() {
    let again = "run npx pytxo again";
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    let next = |out: &LandOutcome| {
        let md = render_markdown(out);
        // The Action has no input for these, so the comment never names them.
        for flag in ["--check", "--per-input", "--timeout", "--apply"] {
            assert!(!md.contains(flag), "{flag} in {md}");
        }
        let line = md.lines().find_map(|l| l.strip_prefix("**Next:** "));
        line.unwrap().to_string()
    };

    let out = land(&o).unwrap();
    assert_eq!(
        next(&out),
        "set the Action's `check:` input to your test command."
    );
    assert!(verdict(&out, again).next.contains("git config pytxo.check"));

    o.checks = vec!["echo ok".into()];
    let out = land(&o).unwrap();
    assert_eq!(
        next(&out),
        "merge them as usual. Nothing in your repository changed."
    );
    assert!(verdict(&out, again).next.contains("--apply"));

    #[cfg(windows)]
    let slow = "ping -n 30 127.0.0.1 > nul";
    #[cfg(not(windows))]
    let slow = "sleep 30";
    o.checks = vec![slow.into()];
    o.check_timeout = Duration::from_millis(300);
    let out = land(&o).unwrap();
    assert!(
        next(&out).contains("or raise the Action's `timeout:` input, then push again."),
        "{}",
        next(&out)
    );
    assert!(verdict(&out, again).next.contains("raise --timeout"));

    // Time to fail: on a loaded machine `exit 1` can take 300 ms to start.
    o.check_timeout = DEFAULT_CHECK_TIMEOUT;
    o.checks = vec!["exit 1".into()];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    assert!(
        next(&out).contains("or set the Action's `check:` input to the right command"),
        "{}",
        next(&out)
    );
    assert!(verdict(&out, again).next.contains("with --check"));

    // A failure with no input run alone: the Action cannot pass --per-input.
    o.per_input = PerInput::Never;
    let out = land(&o).unwrap();
    assert_eq!(next(&out), "fix the failure, then push again.");
    assert!(verdict(&out, again).next.contains("--per-input"));
    // ...and one that needs its dependencies installed says how.
    npm_project(d);
    let out = land(&o).unwrap();
    assert_eq!(
        next(&out),
        "checks run in a fresh worktree, so install first: set the Action's `check:` input to `npm install && npm test`. Then push again."
    );
}

/// Test output in a pull-request comment must not mention, link or format:
/// it shows only as code, which nothing in the output can close.
#[test]
fn test_output_shows_only_as_code_in_markdown() {
    let dir = repo();
    let d = dir.path();
    let token = format!("ghp_{}", "x7".repeat(18));
    let line = "Error: login failed for @octocat at https://evil.example <b>x</b> ```x";
    let printed = format!("{line} {token}\n");
    // The token is masked once, where the output is captured.
    let line = &format!("{line} [REDACTED]");
    branch(d, "main", "agent/a", &[("out.txt", &printed)]);
    let mut o = opts(d, &["agent/a"]);
    #[cfg(windows)]
    let check = "type out.txt & exit 1";
    #[cfg(not(windows))]
    let check = "cat out.txt; exit 1";
    o.checks = vec![check.into()];
    let out = land(&o).unwrap();
    assert_eq!(out.decision(), "blocked");
    let md = render_markdown(&out);
    // A span and a fence one backtick longer than the output's own ```.
    assert!(
        md.contains(&format!(
            "**Why:** {check} fails on the combined change: ````{line}````\n"
        )),
        "{md}"
    );
    assert!(md.contains(&format!("\n````\n{line}\n````\n")), "{md}");
    assert_eq!(md.matches("@octocat").count(), 2, "{md}");
    assert!(!md.contains(&token), "{md}");
    // Plain output shows the same line, not as code.
    assert!(verdict(&out, "again").why.ends_with(&format!(": {line}")));
}

/// Colors, progress redraws and other control characters print nothing in
/// a terminal; in a comment a carriage return would end a code span.
#[test]
fn check_output_shows_as_a_terminal_would() {
    let dir = repo();
    let d = dir.path();
    // The second line reads as an error, so it reaches the comment; the
    // first is a count of failures, which says nothing about why.
    let printed = "\u{1b}[31mFAIL\u{1b}[0m 2 tests\n0%\r# @octocat failed it\n";
    branch(d, "main", "agent/a", &[("out.txt", printed)]);
    let mut o = opts(d, &["agent/a"]);
    #[cfg(windows)]
    let check = "type out.txt & exit 1";
    #[cfg(not(windows))]
    let check = "cat out.txt; exit 1";
    o.checks = vec![check.into()];
    let out = land(&o).unwrap();
    assert_eq!(
        out.receipt.predicate.checks[0].output_tail,
        ["FAIL 2 tests", "# @octocat failed it"]
    );
    let md = render_markdown(&out);
    assert!(!md.contains(['\r', '\u{1b}']), "{md:?}");
    let mut fenced = false;
    for line in md.lines().skip(1) {
        if line.starts_with("```") {
            fenced = !fenced;
        }
        assert!(fenced || !line.starts_with('#'), "{line}\n{md}");
    }
}

/// What the target changed since a branch was cut is named as the target,
/// not as the branches before it, even one that changed the same file
/// elsewhere: rebasing onto that one would not help.
#[test]
fn a_conflict_with_the_target_names_the_target() {
    let again = "run npx pytxo again";
    let dir = repo();
    let d = dir.path();
    let readme = "# shop\n\nSells things.\n\nOpen daily.\n\nMIT\n";
    write(d, &[("README.md", readme)]);
    git(d, &["commit", "-q", "-am", "readme"]);
    let last = readme.replace("MIT", "Apache-2.0");
    branch(d, "main", "agent/a", &[("README.md", &last)]);
    let first = readme.replace("# shop", "# shop B");
    branch(d, "main", "agent/b", &[("README.md", &first)]);
    write(
        d,
        &[("README.md", &readme.replace("# shop", "# shop main"))],
    );
    git(d, &["commit", "-q", "-am", "main moves on"]);

    for inputs in [&["agent/b"][..], &["agent/a", "agent/b"]] {
        let mut o = opts(d, inputs);
        o.checks = vec!["echo ok".into()];
        let out = land(&o).unwrap();
        assert_eq!(
            out.receipt.predicate.reasons,
            ["agent/b conflicts with main in README.md"]
        );
        let v = verdict(&out, again);
        assert_eq!(
            (v.kind, v.what.as_str()),
            ("conflict", "agent/b conflicts with main.")
        );
        assert_eq!(
            v.next,
            "rebase agent/b onto main (or ask its agent to), then run npx pytxo again."
        );
        assert!(
            render_markdown(&out).starts_with("### ⛔ BLOCKED: `agent/b` conflicts with `main`\n")
        );
    }
}

/// A run on the target that times out says nothing about the target: the
/// check is not blamed on it.
#[test]
fn a_target_run_that_times_out_blames_nothing() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.bad", "a\n")]);
    branch(d, "main", "agent/b", &[("b.bad", "b\n")]);
    // Fails at once where a branch is, and runs until stopped on main.
    #[cfg(windows)]
    let check =
        "if exist a.bad (exit 1) else if exist b.bad (exit 1) else (ping -n 30 127.0.0.1 > nul)";
    #[cfg(not(windows))]
    let check = "test -f a.bad && exit 1; test -f b.bad && exit 1; sleep 30";
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec![check.into()];
    o.per_input = PerInput::OnFailure;
    o.check_timeout = Duration::from_secs(3);
    let out = land(&o).unwrap();
    let checks = &out.receipt.predicate.checks;
    assert!(
        checks.iter().any(|c| c.scope == "target" && c.timed_out),
        "{checks:?}"
    );
    let said = out.insights.join("\n");
    assert!(said.contains("failed on every input alone too"), "{said}");
    assert!(!said.contains("on main"), "{said}");
    assert_eq!(
        verdict(&out, "again").what,
        format!("{check} fails on every branch alone too, so the combination is not the problem.")
    );
}

/// A combined run that times out is the answer: each further run could take
/// as long again to say the same.
#[test]
fn nothing_runs_after_a_timeout() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    #[cfg(windows)]
    let slow = "ping -n 30 127.0.0.1 > nul";
    #[cfg(not(windows))]
    let slow = "sleep 30";
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec![slow.into(), "echo never".into()];
    o.per_input = PerInput::OnFailure;
    o.check_timeout = Duration::from_millis(500);
    let out = land(&o).unwrap();
    let checks = &out.receipt.predicate.checks;
    assert_eq!(checks.len(), 1, "{checks:?}");
    assert!(checks[0].scope == "combined" && checks[0].timed_out);
    assert_eq!(verdict(&out, "again").kind, "timeout");

    // Asked for, each input still runs alone first; the target never does.
    o.checks = vec![slow.into()];
    o.per_input = PerInput::Always;
    let out = land(&o).unwrap();
    let scopes: Vec<&str> = out
        .receipt
        .predicate
        .checks
        .iter()
        .map(|c| c.scope.as_str())
        .collect();
    assert_eq!(scopes, ["input:agent/a", "input:agent/b", "combined"]);
}

/// A killed run's scratch worktrees stay registered; the next run removes
/// them, but not a concurrent run's.
#[test]
fn a_run_removes_what_killed_runs_left() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    let scratch = tempfile::tempdir().unwrap();
    let left = |tag: &str, pid: u32| {
        let path = scratch.path().join(format!("pytxo-land-{tag}"));
        let shown = path.to_str().unwrap();
        git(d, &["worktree", "add", "-q", "--detach", shown, "main"]);
        let reason = format!("pytxo {pid}");
        git(d, &["worktree", "lock", "--reason", &reason, shown]);
        path
    };
    let now = chrono::Utc::now().format("%Y%m%dt%H%M%Sz");
    let me = std::process::id();
    let old = left("20250101t000000z_0123abcd", me);
    let dead = left(&format!("{now}_0123abce-solo"), 99_999_999);
    let running = left(&format!("{now}_0123abcf"), me);

    let mut o = opts(d, &["agent/a"]);
    o.checks = vec!["echo ok".into()];
    assert_eq!(land(&o).unwrap().decision(), "ready");
    let listed = git(d, &["worktree", "list", "--porcelain"]);
    let registered = |p: &Path| listed.contains(p.file_name().unwrap().to_str().unwrap());
    assert!(!old.exists() && !registered(&old), "{listed}");
    assert!(!dead.exists() && !registered(&dead), "{listed}");
    assert!(running.exists() && registered(&running), "{listed}");
}

/// Git's own spelling of a path, not `"caf\303\251.txt"`.
#[test]
fn paths_are_recorded_as_they_are() {
    let dir = repo();
    let d = dir.path();
    git(d, &["config", "core.quotePath", "true"]);
    branch(d, "main", "agent/a", &[("café.txt", "a\n")]);
    let out = land(&opts(d, &["agent/a"])).unwrap();
    assert_eq!(out.receipt.predicate.inputs[0].files, ["café.txt"]);
}

/// Checks run in a worktree without submodules; a failure says how to
/// include them.
#[test]
fn a_failure_says_how_to_include_submodules() {
    let dir = repo();
    let d = dir.path();
    let modules = "[submodule \"lib\"]\n\tpath = lib\n\turl = https://example.invalid/lib.git\n";
    write(d, &[(".gitmodules", modules)]);
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "submodules"]);
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    let mut o = opts(d, &["agent/a"]);
    o.checks = vec!["exit 1".into()];
    let out = land(&o).unwrap();
    assert!(out.submodules);
    let v = verdict(&out, "run npx pytxo again");
    assert!(
        v.next.ends_with(" Checks run in a fresh worktree without submodules: to include them, pass --check \"git submodule update --init --recursive && exit 1\"."),
        "{v:?}"
    );
    let md = render_markdown(&out);
    assert!(
        md.contains(" to include them, set the Action's `check:` input to `git submodule update --init --recursive && exit 1`."),
        "{md}"
    );
    // A pass has nothing to explain.
    o.checks = vec!["echo ok".into()];
    let v = verdict(&land(&o).unwrap(), "again");
    assert!(!v.next.contains("submodule"), "{v:?}");
}

/// The gate's own git commands run no hooks: not the repository's, and on
/// Unix none another user could plant in the shared temp directory, where
/// the hooks path once pointed.
#[test]
fn scratch_checkouts_run_no_hooks() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    // Here whatever the user's global config says.
    let hooks = d.join(".git/hooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let hooks_path = hooks.display().to_string().replace('\\', "/");
    git(d, &["config", "core.hooksPath", &hooks_path]);
    let ran = d.join(".hook-ran");
    let hook = hooks.join("post-checkout");
    let touch = ran.display().to_string().replace('\\', "/");
    std::fs::write(&hook, format!("#!/bin/sh\ntouch '{touch}'\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    // Planted only when this test makes the directory, and removed before
    // anything can fail.
    #[cfg(unix)]
    let planted = {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join("pytxo-land-no-hooks-dir-does-not-exist");
        std::fs::create_dir(&dir).is_ok().then(|| {
            let hook = dir.join("post-checkout");
            std::fs::write(&hook, format!("#!/bin/sh\ntouch '{touch}'\n")).unwrap();
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
            dir
        })
    };
    let mut o = opts(d, &["agent/a"]);
    o.checks = vec!["echo ok".into()];
    o.per_input = PerInput::Always;
    let decision = land(&o).map(|out| out.decision().to_string());
    #[cfg(unix)]
    if let Some(dir) = planted {
        let _ = std::fs::remove_dir_all(dir);
    }
    assert_eq!(decision.unwrap(), "ready");
    assert!(!ran.exists());
    // The hook runs when the user's own git checks out.
    git(d, &["checkout", "-q", "agent/a"]);
    assert!(ran.exists());
}

/// Checks whose working directory reads `C:\Users\RUNNER~1\...` see it unlike
/// its real path, so it is spelled in full unless that has a space or the
/// like (`C:\Users\Jane Doe\...`).
#[cfg(windows)]
#[test]
fn the_temp_dir_is_spelled_in_full() {
    let temp = long_temp_dir();
    let shown = temp.to_string_lossy();
    let full = std::fs::canonicalize(std::env::temp_dir()).unwrap();
    assert!(!shown.starts_with(r"\\?\"), "{shown}");
    if plain_drive_path(&full.to_string_lossy()).is_some() {
        assert!(!shown.contains('~'), "{shown}");
    }
    assert_eq!(std::fs::canonicalize(&temp).unwrap(), full);
}

/// Only a plain drive path replaces the temp directory's own spelling.
#[test]
fn the_temp_dir_is_spelled_in_full_only_when_plain() {
    assert_eq!(
        plain_drive_path(r"\\?\C:\Users\runneradmin\AppData\Local\Temp"),
        Some(PathBuf::from(r"C:\Users\runneradmin\AppData\Local\Temp"))
    );
    for kept in [
        r"\\?\C:\Users\Jane Doe\AppData\Local\Temp",
        r"\\?\C:\Users\José\AppData\Local\Temp",
        r"\\?\UNC\fileserver\home\jane\Temp",
        r"\\?\Volume{0b7d4f8e-1c2a-4e5b-9a3d-6f1e2d3c4b5a}\Temp",
        r"C:\Users\jane\AppData\Local\Temp",
    ] {
        assert_eq!(plain_drive_path(kept), None, "{kept}");
    }
}

/// A run's worktrees and check logs sit in a directory of its own, which
/// goes with the run, or with the next run after a kill.
#[test]
fn a_run_keeps_its_worktrees_and_logs_in_a_directory_of_its_own() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    let mut o = opts(d, &["agent/a"]);
    // From the run's directory: a path into the worktree reads from its root.
    #[cfg(windows)]
    let list = "cd .. && cd && dir /b";
    #[cfg(not(windows))]
    let list = "cd .. && pwd && ls";
    o.checks = vec![list.into()];
    let out = land(&o).unwrap();
    let p = &out.receipt.predicate;
    let shown = &p.checks[0].output_tail;
    let run_dir = Path::new(&shown[0]);
    let tag = p.id.trim_start_matches("rcpt_").to_lowercase();
    assert!(run_dir.ends_with(format!("pytxo-{tag}")), "{shown:?}");
    assert!(
        shown
            .iter()
            .any(|l| l.starts_with("pytxo-check-") && l.ends_with(".log")),
        "{shown:?}"
    );
    assert!(!run_dir.exists(), "{shown:?}");
}

/// A detached worktree's commit shows as its label, not as a hash, in a
/// --strict collision too.
#[test]
fn a_collision_names_a_detached_worktree_by_its_label() {
    let dir = repo();
    let d = dir.path();
    let c = CART.replace("let sum = 0;", "let sum = 0; // start");
    let c2 = CART.replace("return sum;", "return Math.round(sum);");
    branch(d, "main", "agent/c", &[("src/cart.js", &c)]);
    branch(d, "main", "agent/c2", &[("src/cart.js", &c2)]);
    let codex = format!("codex:{}", git(d, &["rev-parse", "agent/c"]));
    let claude = format!("claude:{}", git(d, &["rev-parse", "agent/c2"]));
    let mut o = opts(d, &[codex.as_str(), claude.as_str()]);
    o.strict = true;
    let out = land(&o).unwrap();
    let shared = &out.receipt.predicate.shared_files[0];
    assert_eq!(shared.inputs, ["codex", "claude"]);
    assert_eq!(shared.symbol_collisions[0].inputs, ["codex", "claude"]);
    assert_eq!(
        verdict(&out, "again").what,
        "codex and claude both changed total in src/cart.js."
    );
}

/// Ctrl+C says what it does, and a second one quits at once while the
/// clean-up is still slow; on Unix a signal ignored at start (`nohup pytxo`,
/// the binary itself) stays ignored. In a child process, whose handler this test installs.
#[test]
fn a_second_ctrl_c_quits_at_once() {
    const CHILD: &str = "PYTXO_LAND_CTRL_C_TEST";
    if std::env::var_os(CHILD).is_some() {
        // SAFETY: ignoring a signal runs no code.
        #[cfg(unix)]
        unsafe {
            libc::signal(libc::SIGHUP, libc::SIG_IGN);
        }
        // The clean-up waits for this lock, as for a slow removal.
        let _busy = active();
        #[cfg(unix)]
        {
            // SAFETY: raise signals only this process.
            unsafe { libc::raise(libc::SIGHUP) };
            std::thread::sleep(Duration::from_millis(500));
            assert!(!STOPPING.load(Ordering::SeqCst), "SIGHUP was handled");
        }
        // As ctrlc's thread would, once for each Ctrl+C.
        std::thread::spawn(interrupted).join().unwrap();
        interrupted();
        unreachable!("the second Ctrl+C exits");
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "tests::a_second_ctrl_c_quits_at_once",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if started.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            break None;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let mut said = String::new();
    std::io::Read::read_to_string(&mut child.stderr.take().unwrap(), &mut said).unwrap();
    assert_eq!(status.and_then(|s| s.code()), Some(130), "{said}");
    assert!(
        said.contains("Press Ctrl+C again to quit at once."),
        "{said}"
    );
}

/// On Unix a run's worktrees and logs sit in a directory only the user can
/// enter, gone with the run.
#[cfg(unix)]
#[test]
fn the_run_directory_is_private() {
    use std::os::unix::fs::PermissionsExt;
    // Unique, so no other run's directory is in the way.
    let tag = new_receipt_id().trim_start_matches("rcpt_").to_lowercase();
    let run = RunDir::new(&tag).unwrap();
    let mode = std::fs::metadata(&run.0).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
    let path = run.0.clone();
    drop(run);
    assert!(!path.exists());
}

/// A check that fails where `file` exists.
fn fails_with(file: &str) -> String {
    if cfg!(windows) {
        format!("if exist {file} (exit 1) else (exit 0)")
    } else {
        format!("test ! -f {file}")
    }
}

/// Each run in a reused worktree starts from its branch's files only: build
/// output an earlier run left in an ignored directory is gone, so a test
/// command that runs that output blames the right branch.
#[test]
fn ignored_build_output_does_not_carry_over_between_runs() {
    let dir = repo();
    let d = dir.path();
    write(d, &[(".gitignore", "out/\n"), ("src/base.txt", "ok\n")]);
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "build"]);
    branch(d, "main", "agent/bad", &[("src/bad1.txt", "bad\n")]);
    branch(d, "main", "agent/good", &[("src/good.txt", "good\n")]);
    branch(d, "main", "agent/worse", &[("src/bad2.txt", "bad\n")]);
    // Builds src/ into the ignored out/, then fails on any bad file there.
    let check = if cfg!(windows) {
        r"(if not exist out mkdir out) & copy /y src\*.txt out >nul & if exist out\bad*.txt (exit 1) else (exit 0)"
    } else {
        "mkdir -p out && cp src/*.txt out/ && ! ls out/bad* >/dev/null 2>&1"
    };

    let mut o = opts(d, &["agent/bad", "agent/good"]);
    o.checks = vec![check.into()];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let checks = &out.receipt.predicate.checks;
    let alone = |r: &str| checks.iter().find(|c| c.scope == format!("input:{r}"));
    assert!(alone("agent/good").is_some_and(|c| c.passed), "{checks:?}");
    assert!(
        verdict(&out, "again").what.starts_with("agent/bad fails "),
        "{checks:?}"
    );

    // Both fail alone: main runs without the combination's output.
    o.inputs = ["agent/bad", "agent/worse"].map(LandInput::parse).to_vec();
    let out = land(&o).unwrap();
    let checks = &out.receipt.predicate.checks;
    assert!(
        checks.iter().any(|c| c.scope == "target" && c.passed),
        "{checks:?}"
    );
    assert!(
        verdict(&out, "again")
            .what
            .starts_with("agent/bad and agent/worse fail "),
        "{checks:?}"
    );
}

/// A user's `submodule.recurse` does not reach the scratch checkouts, which
/// leave submodules out.
#[test]
fn submodule_recurse_does_not_break_the_scratch_checkouts() {
    let sub = repo();
    let dir = repo();
    let d = dir.path();
    let url = sub.path().to_string_lossy().replace('\\', "/");
    git(
        d,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &url,
            "lib",
        ],
    );
    git(d, &["commit", "-q", "-m", "submodule"]);
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    git(d, &["config", "submodule.recurse", "true"]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec![fails_with("a.txt")];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let checks = &out.receipt.predicate.checks;
    assert!(
        checks
            .iter()
            .any(|c| c.scope == "input:agent/b" && c.passed),
        "{checks:?}"
    );
    assert!(
        verdict(&out, "again").what.starts_with("agent/a fails "),
        "{checks:?}"
    );
}

/// The second run that tells a flaky combination from a real break starts
/// from the combined commit's files, as the first did, after the checks before
/// it: not from what the first runs left, nor without what they build.
#[test]
fn the_flaky_rerun_starts_from_the_combined_files() {
    let dir = repo();
    let d = dir.path();
    write(d, &[(".gitignore", "out/\n")]);
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "ignore out"]);
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.per_input = PerInput::OnFailure;

    // Fails over its own output, as `mkdir out` would, and once on the
    // combination.
    let leftover = if cfg!(windows) {
        r"(if exist out\ran exit 1) & (if not exist out mkdir out) & type nul > out\ran & "
    } else {
        "test -f out/ran && exit 1; mkdir -p out && touch out/ran; "
    };
    let marker = d.join(".git").join("flaky-once");
    o.checks = vec![format!(
        "{leftover}{}",
        combined_only_failure(Some(&marker))
    )];
    let out = land(&o).unwrap();
    let checks = &out.receipt.predicate.checks;
    assert_eq!(
        combined_runs(&out),
        [(false, false), (true, true)],
        "{checks:?}"
    );
    assert_eq!(verdict(&out, "again").label, "FLAKY", "{checks:?}");

    // Needs what the check before it built.
    let (build, built) = if cfg!(windows) {
        (
            r"(if not exist out mkdir out) & type nul > out\built",
            r"(if not exist out\built exit 1) & ",
        )
    } else {
        (
            "mkdir -p out && touch out/built",
            "test -f out/built || exit 1; ",
        )
    };
    let marker = d.join(".git").join("flaky-twice");
    o.checks = vec![
        build.into(),
        format!("{built}{}", combined_only_failure(Some(&marker))),
    ];
    let out = land(&o).unwrap();
    let checks = &out.receipt.predicate.checks;
    assert_eq!(
        combined_runs(&out),
        [(true, false), (false, false), (true, true)],
        "{checks:?}"
    );
    assert_eq!(verdict(&out, "again").label, "FLAKY", "{checks:?}");
}

/// A check that fails again on the combination speaks over a flaky one.
#[test]
fn a_flaky_check_does_not_hide_a_real_failure() {
    let dir = repo();
    let d = dir.path();
    branch(d, "main", "agent/a", &[("a.txt", "a\n")]);
    branch(d, "main", "agent/b", &[("b.txt", "b\n")]);
    let marker = d.join(".git").join("flaky-ran");
    let mut o = opts(d, &["agent/a", "agent/b"]);
    o.checks = vec![combined_only_failure(Some(&marker)), fails_with("a.txt")];
    o.per_input = PerInput::OnFailure;
    let out = land(&o).unwrap();
    let v = verdict(&out, "again");
    assert_eq!(v.label, "BLOCKED", "{v:?}");
    assert!(v.what.starts_with("agent/a fails "), "{v:?}");
    let md = render_markdown(&out);
    assert!(
        md.contains(r#"{"label":"BLOCKED","implicated":[0]}"#),
        "{md}"
    );
}
