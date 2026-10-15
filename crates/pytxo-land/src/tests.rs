use std::path::Path;
use std::process::Command;

use super::*;

fn git(dir: &Path, args: &[&str]) -> String {
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

fn write(dir: &Path, files: &[(&str, &str)]) {
    for (path, body) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
}

const CART: &str = "function price(item) {\n  return item.cost;\n}\n\nfunction total(items) {\n  let sum = 0;\n  for (const i of items) sum += price(i);\n  return sum;\n}\n\nfunction label(n) {\n  return 'total: ' + n;\n}\n\nmodule.exports = { price, total, label };\n";

fn repo() -> tempfile::TempDir {
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
fn branch(d: &Path, from: &str, name: &str, files: &[(&str, &str)]) {
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
    assert!(render_markdown(&out).contains("Pytxo land: **ready**"));
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
    o.per_input = true;
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
}

/// An npm project whose test script exists, committed on main.
fn npm_project(d: &Path) {
    write(
        d,
        &[("package.json", r#"{"scripts":{"test":"tape test/*.js"}}"#)],
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
    o.per_input = true;
    let out = land(&o).unwrap();
    assert_eq!(out.decision(), "blocked");

    let hint = out.insights.join("\n");
    assert!(hint.contains("failed on every input alone too"), "{hint}");
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
