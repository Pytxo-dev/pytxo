//! The `pytxo` binary end to end: bare check, JSON, hook, demo, exit codes.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn git(dir: &Path, args: &[&str]) {
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
}

/// main plus the given agent branches, each adding one file.
fn repo(branches: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    git(d, &["init", "-q", "-b", "main"]);
    git(d, &["config", "user.email", "t@example.com"]);
    git(d, &["config", "user.name", "t"]);
    git(d, &["config", "core.autocrlf", "false"]);
    std::fs::write(d.join("README.md"), "# shop\n").unwrap();
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "base"]);
    for (i, b) in branches.iter().enumerate() {
        git(d, &["checkout", "-q", "-b", b, "main"]);
        std::fs::write(d.join(format!("f{i}.txt")), format!("{i}\n")).unwrap();
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", b]);
        git(d, &["checkout", "-q", "main"]);
    }
    dir
}

fn pytxo(dir: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    pytxo_env(dir, args, stdin, &[])
}

fn pytxo_env(dir: &Path, args: &[&str], stdin: Option<&str>, envs: &[(&str, &str)]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pytxo"))
        .args(args)
        .current_dir(dir)
        .env_remove("PYTXO_RUN_AS")
        .env_remove("PYTXO_BRANCHES")
        .env_remove("PYTXO_CHECK")
        .env_remove("PYTXO_DAYS")
        .env_remove("PYTXO_INTO")
        .envs(envs.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut pipe = child.stdin.take().unwrap();
    pipe.write_all(stdin.unwrap_or("").as_bytes()).unwrap();
    drop(pipe);
    child.wait_with_output().unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn no_agent_branches_is_one_line_and_exit_zero() {
    let dir = repo(&["matt/readme"]);
    let out = pytxo(dir.path(), &[], None);
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    assert_eq!(text.lines().count(), 1, "{text}");
    assert!(
        text.starts_with("No agent branches to check against main"),
        "{text}"
    );
    assert!(
        text.trim_end().ends_with("See it work: pytxo demo"),
        "{text}"
    );

    // An old agent branch, merged long ago: main has its commit, so there is
    // nothing to say about it.
    let d = dir.path();
    git(d, &["checkout", "-q", "-b", "claude/old", "main"]);
    std::fs::write(d.join("old.txt"), "old\n").unwrap();
    git(d, &["add", "-A"]);
    let old = Command::new("git")
        .args(["commit", "-q", "-m", "old"])
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .current_dir(d)
        .output()
        .unwrap();
    assert!(old.status.success());
    git(d, &["checkout", "-q", "main"]);
    git(d, &["merge", "-q", "--no-ff", "-m", "merge", "claude/old"]);
    let text = stdout(&pytxo(d, &[], None));
    assert!(!text.contains("not looked at"), "{text}");
    let text = stdout(&pytxo_env(d, &[], None, &[("PYTXO_DAYS", "0")]));
    assert!(text.starts_with("No agent branches"), "{text}");
    // One nobody merged is counted, with the flag that looks at it.
    git(d, &["checkout", "-q", "-b", "codex/stale", "main"]);
    std::fs::write(d.join("stale.txt"), "stale\n").unwrap();
    git(d, &["add", "-A"]);
    let stale = Command::new("git")
        .args(["commit", "-q", "-m", "stale"])
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .current_dir(d)
        .output()
        .unwrap();
    assert!(stale.status.success());
    git(d, &["checkout", "-q", "main"]);
    let text = stdout(&pytxo(d, &[], None));
    assert!(
        text.contains(" 1 with no commit in 14 days not looked at (--days 0 looks at them). "),
        "{text}"
    );
    git(d, &["branch", "-q", "-D", "codex/stale"]);
    // Cloud agents' branches arrive by fetch.
    assert!(
        text.ends_with(
            " If your agents push to GitHub, run git fetch first. See it work: pytxo demo\n"
        ),
        "{text}"
    );

    // A branch with no history in common is left out and counted, not an error.
    git(d, &["checkout", "-q", "--orphan", "claude/pages"]);
    git(d, &["rm", "-rfq", "."]);
    std::fs::write(d.join("index.html"), "hi\n").unwrap();
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "pages"]);
    git(d, &["checkout", "-q", "main"]);
    let out = pytxo(d, &[], None);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout(&out).contains(" 1 sharing no history with main left out. "),
        "{}",
        stdout(&out)
    );
}

#[test]
fn outside_a_repository_points_at_the_demo() {
    let elsewhere = tempfile::tempdir().unwrap();
    let out = pytxo(elsewhere.path(), &[], None);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "error: not a git repository: run pytxo inside one, or see it work: pytxo demo\n"
    );
    // No git at all says so.
    let empty = tempfile::tempdir().unwrap();
    let path = empty.path().to_string_lossy().into_owned();
    let out = pytxo_env(elsewhere.path(), &[], None, &[("PATH", &path)]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "error: git is not on PATH: install it from https://git-scm.com\n"
    );
}

/// A global pytxo.check runs instead of nothing, never beside the
/// repository's; a global pytxo.days never beats it.
#[test]
fn global_settings_give_way_to_the_repository_ones() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    git(d, &["checkout", "-q", "-b", "claude/old", "main"]);
    std::fs::write(d.join("old.txt"), "old\n").unwrap();
    git(d, &["add", "-A"]);
    let old = Command::new("git")
        .args(["commit", "-q", "-m", "old"])
        .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
        .current_dir(d)
        .output()
        .unwrap();
    assert!(old.status.success());
    git(d, &["checkout", "-q", "main"]);
    git(d, &["config", "pytxo.check", "git --version"]);
    git(d, &["config", "pytxo.days", "14"]);
    let home = tempfile::tempdir().unwrap();
    let global = home.path().join("gitconfig");
    std::fs::write(&global, "[pytxo]\n\tcheck = exit 1\n\tdays = 0\n").unwrap();
    let global = global.to_string_lossy().into_owned();
    let out = pytxo_env(d, &["--json"], None, &[("GIT_CONFIG_GLOBAL", &global)]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["checks"], serde_json::json!(["git --version"]));
    let names: Vec<&str> = v["branches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["claude/a", "codex/b"]);
}

#[test]
fn work_nobody_committed_is_named_not_dropped() {
    let dir = repo(&[]);
    let d = dir.path();
    git(d, &["config", "pytxo.check", "git --version"]);
    // main's last commit is weeks old, as in a quiet repository; the
    // worktrees on it have no commit of their own to be old.
    let amend = Command::new("git")
        .args(["commit", "-q", "--amend", "--no-edit"])
        .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
        .current_dir(d)
        .output()
        .unwrap();
    assert!(amend.status.success());
    // Two agents at their turn's end: worktrees on their branches, nothing committed.
    let worktrees = tempfile::tempdir().unwrap();
    for name in ["claude/one", "claude/two"] {
        let wt = worktrees.path().join(name.replace('/', "-"));
        git(
            d,
            &["worktree", "add", "-q", "-b", name, &wt.to_string_lossy()],
        );
        std::fs::write(wt.join("draft.txt"), "draft\n").unwrap();
    }
    let note = "claude/one and claude/two have only uncommitted changes; commit them to check them together.";
    let text = stdout(&pytxo(d, &[], None));
    assert!(text.contains(&format!(" {note} ")), "{text}");
    assert!(!text.contains("not looked at"), "{text}");
    // The hook says it once.
    let event = stop_event(d);
    let v = hook_reply(&pytxo(d, &["hook"], Some(&event)));
    assert_eq!(
        v["systemMessage"],
        format!("Pytxo did not check the agent branches: {note}")
    );
    let out = pytxo(d, &["hook"], Some(&event));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));
}

#[test]
fn the_hook_carries_the_uncommitted_note_the_terminal_prints() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    git(d, &["config", "pytxo.check", "git --version"]);
    let worktrees = tempfile::tempdir().unwrap();
    let b = worktrees.path().join("b");
    git(
        d,
        &["worktree", "add", "-q", &b.to_string_lossy(), "codex/b"],
    );
    std::fs::write(b.join("f1.txt"), "edited\n").unwrap();
    let note = "codex/b has uncommitted changes; only its commits are checked.";
    let out = pytxo(d, &[], None);
    let progress = String::from_utf8_lossy(&out.stderr);
    assert!(progress.contains(&format!("\n  {note}\n")), "{progress}");

    let v = hook_reply(&pytxo(d, &["hook"], Some(&stop_event(d))));
    let message = v["systemMessage"].as_str().unwrap();
    assert!(message.contains("\nREADY    "), "{message}");
    assert!(
        message.ends_with(&format!("\nnote     {note}\n")),
        "{message}"
    );
    git(d, &["config", "pytxo.check", "exit 1"]);
    let v = hook_reply(&pytxo(d, &["hook", "--block"], Some(&stop_event(d))));
    assert_eq!(v["decision"], "block", "{v}");
    let reason = v["reason"].as_str().unwrap();
    assert!(
        reason.ends_with(&format!("\nnote     {note}\n")),
        "{reason}"
    );
}

#[test]
fn notes_under_the_verdict_say_what_it_left_out() {
    let names: Vec<String> = (0..11).map(|i| format!("codex/b{i:02}")).collect();
    let dir = repo(&names.iter().map(String::as_str).collect::<Vec<_>>());
    let d = dir.path();
    // GitHub has a commit the local main has not pulled.
    git(d, &["checkout", "-q", "-b", "upstream", "main"]);
    git(d, &["commit", "-q", "--allow-empty", "-m", "on GitHub"]);
    git(d, &["update-ref", "refs/remotes/origin/main", "upstream"]);
    git(d, &["checkout", "-q", "main"]);
    let notes = [
        "main is 1 commit behind origin/main; pull it to check against that too.",
        "1 older agent branch left out (at most 10 at once).",
    ];
    let out = pytxo(d, &["-c", "git --version"], None);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    let text = stdout(&out);
    assert!(text.starts_with("READY    10 branches pass"), "{text}");
    assert!(
        text.ends_with(&format!("\nnote     {}\nnote     {}\n", notes[0], notes[1])),
        "{text}"
    );
    let out = pytxo(d, &["--json", "-c", "git --version"], None);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["notes"], serde_json::json!(notes));
    assert_eq!(v["branches"].as_array().unwrap().len(), 10);
}

#[test]
fn squash_merged_branches_are_skipped_without_merge_tree() {
    let dir = repo(&["claude/a", "codex/b", "codex/squashed"]);
    let d = dir.path();
    git(d, &["merge", "-q", "--squash", "codex/squashed"]);
    git(d, &["commit", "-q", "-m", "squash"]);
    // Git before 2.38 has no merge-tree --write-tree; this takes that path.
    let out = pytxo_env(
        d,
        &["--json", "-c", "git --version"],
        None,
        &[("PYTXO_LAND_WORKTREE_MERGE", "1")],
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let names: Vec<&str> = v["branches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["claude/a", "codex/b"]);
}

#[test]
fn bare_pytxo_exits_two_when_blocked_and_three_when_stale() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    let out = pytxo(d, &["-c", "exit 1"], None);
    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    assert!(stdout(&out).starts_with("BLOCKED  "), "{}", stdout(&out));
    // The test command moves main while the check runs.
    let out = pytxo(d, &["-c", "git update-ref refs/heads/main HEAD"], None);
    assert_eq!(out.status.code(), Some(3), "{}", stdout(&out));
    assert!(
        stdout(&out).starts_with("STALE    main moved while the check ran.\n"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn strict_blocks_two_branches_that_change_one_function() {
    let dir = repo(&[]);
    let d = dir.path();
    let cart = "function total(items) {\n  let sum = 0;\n  for (const i of items) sum += i;\n  return sum;\n}\n";
    std::fs::write(d.join("cart.js"), cart).unwrap();
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "cart"]);
    for (name, from, to) in [
        ("claude/c", "let sum = 0;", "let sum = 0; // start"),
        ("codex/c2", "return sum;", "return Math.round(sum);"),
    ] {
        git(d, &["checkout", "-q", "-b", name, "main"]);
        std::fs::write(d.join("cart.js"), cart.replace(from, to)).unwrap();
        git(d, &["commit", "-q", "-am", name]);
        git(d, &["checkout", "-q", "main"]);
    }
    let out = pytxo(d, &["--strict", "-c", "git --version"], None);
    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    assert!(
        stdout(&out).starts_with("BLOCKED  claude/c and codex/c2 both changed total in cart.js.\n"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn bare_pytxo_finds_the_agent_branches_and_prints_three_lines() {
    let dir = repo(&["claude/a", "codex/b", "matt/c"]);
    let out = pytxo(dir.path(), &["-c", "git --version"], None);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    assert_eq!(text.lines().count(), 3, "{text}");
    assert!(
        text.starts_with("READY    claude/a and codex/b pass git --version together."),
        "{text}"
    );
    let progress = String::from_utf8_lossy(&out.stderr);
    assert!(
        progress.contains("Checking claude/a and codex/b against main"),
        "{progress}"
    );

    // Labelled, as the land guides write them: both lines name the branches.
    let out = pytxo(
        dir.path(),
        &["-c", "git --version", "claude:claude/a", "codex:codex/b"],
        None,
    );
    let progress = String::from_utf8_lossy(&out.stderr);
    assert!(
        progress.starts_with("Checking claude/a and codex/b against main"),
        "{progress}"
    );
    assert!(
        stdout(&out).starts_with("READY    claude/a and codex/b pass"),
        "{}",
        stdout(&out)
    );

    // Only --verbose keeps a ref to the combined commit.
    let refs = || {
        let out = Command::new("git")
            .args(["for-each-ref", "refs/pytxo"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).lines().count()
    };
    assert_eq!(refs(), 0);
    pytxo(dir.path(), &["-v", "-c", "git --version"], None);
    assert_eq!(refs(), 1);
}

/// The Action's `check:` input arrives as PYTXO_CHECK, over the repository's
/// own setting.
#[test]
fn pytxo_check_comes_before_the_repository_setting() {
    let dir = repo(&["claude/a", "codex/b"]);
    git(dir.path(), &["config", "pytxo.check", "exit 1"]);
    let out = pytxo_env(
        dir.path(),
        &["--json"],
        None,
        &[("PYTXO_CHECK", "git --version")],
    );
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["checks"], serde_json::json!(["git --version"]));
    assert_eq!(v["check_source"], "PYTXO_CHECK");
}

/// --apply lands only what a test passed: with no test command it changes
/// nothing.
/// A lint or build script runs when package.json has no test script, but it is
/// not a test suite, so --apply lands nothing on it.
#[test]
fn apply_on_a_script_that_is_not_a_test_suite_lands_nothing() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    std::fs::write(
        d.join("package.json"),
        r#"{"name":"x","version":"1.0.0","scripts":{"lint":"node -e 0"}}"#,
    )
    .unwrap();
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", "package.json"]);
    let main = || {
        let out = Command::new("git")
            .args(["rev-parse", "main"])
            .current_dir(d)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let before = main();
    let out = pytxo(d, &["--apply"], None);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(
        err.contains("--apply lands only a tested result") && err.contains("lint script"),
        "{err}"
    );
    assert_eq!(main(), before);
}

/// One of four branches fails alone: without --apply the note names the other
/// three, and --apply lands those three.
#[test]
fn the_note_and_apply_name_the_branches_that_pass_together() {
    let dir = repo(&["claude/a", "codex/b", "cursor/c", "devin/d"]);
    let d = dir.path();
    let fails_b = "git grep -q 1 -- f1.txt && exit 1 || exit 0";
    let seen = pytxo(d, &["--check", fails_b], None);
    let text = String::from_utf8_lossy(&seen.stdout);
    assert_eq!(seen.status.code(), Some(2), "{text}");
    assert!(
        text.contains(
            "3 of 4 branches pass the tests together: run this again with --apply to land them and leave out the rest."
        ),
        "{text}"
    );
    assert!(text.contains("\ncan land claude/a\n"), "{text}");
    assert!(
        text.contains("\nleft out codex/b   fails your check"),
        "{text}"
    );
    let out = pytxo(d, &["--apply", "--check", fails_b], None);
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(
        text.contains("3 of 4 branches are on main now.\nlanded   claude/a\n"),
        "{text}"
    );
    assert!(
        text.contains("\nleft out codex/b   fails your check"),
        "{text}"
    );
    assert!(d.join("f0.txt").exists() && !d.join("f1.txt").exists());
}

#[test]
fn apply_without_a_test_command_lands_nothing() {
    let dir = repo(&["claude/a", "codex/b"]);
    let main = || {
        let out = Command::new("git")
            .args(["rev-parse", "main"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let before = main();
    let out = pytxo(dir.path(), &["--apply"], None);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(
        err.contains(
            "--apply lands only a tested result, and no test command was found at the root of main"
        ),
        "{err}"
    );
    assert_eq!(main(), before);
}

#[test]
fn json_carries_the_verdict_and_what_was_found() {
    let dir = repo(&["claude/a", "codex/b"]);
    let out = pytxo(dir.path(), &["--json", "-c", "git --version"], None);
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["decision"], "ready");
    assert_eq!(v["verdict"]["label"], "READY");
    assert_eq!(v["target"], "main");
    assert_eq!(v["branches"][0]["name"], "claude/a");
    assert_eq!(v["check_source"], "--check");
    assert!(out.stderr.is_empty(), "--json keeps stderr quiet");

    // A label from the command line comes along; the name stays the branch.
    let args = [
        "--json",
        "-c",
        "git --version",
        "claude:claude/a",
        "codex/b",
    ];
    let out = pytxo(dir.path(), &args, None);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["branches"][0]["name"], "claude/a");
    assert_eq!(v["branches"][0]["label"], "claude");
    assert!(v["branches"][1].get("label").is_none(), "{v}");
}

/// Commits `files` on a new branch `name` cut from main, then returns to main.
fn commit_branch(d: &Path, name: &str, files: &[(&str, &str)]) {
    git(d, &["checkout", "-q", "-b", name, "main"]);
    for (path, body) in files {
        let path = d.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
    git(d, &["add", "-A"]);
    git(d, &["commit", "-q", "-m", name]);
    git(d, &["checkout", "-q", "main"]);
}

/// The demo's break, without Node: `claude/rename-price` renames `price` in
/// src/cart.js and `codex/add-discount` adds a caller of it.
fn rename_repo() -> tempfile::TempDir {
    let dir = repo(&[]);
    let d = dir.path();
    let cart = "function price(item) {\n  return item.cost;\n}\n\nmodule.exports = { price };\n";
    commit_branch(d, "base", &[("src/cart.js", cart)]);
    git(d, &["merge", "-q", "base"]);
    git(d, &["branch", "-q", "-D", "base"]);
    let renamed = cart.replace("price", "unitPrice");
    commit_branch(d, "claude/rename-price", &[("src/cart.js", &renamed)]);
    let discount = "const { price } = require('./cart.js');\n\nfunction discounted(item) {\n  return price(item) * 0.9;\n}\n\nmodule.exports = { discounted };\n";
    commit_branch(d, "codex/add-discount", &[("src/discount.js", discount)]);
    dir
}

/// Fails, in cmd and sh alike, only where src/cart.js says `unitPrice` and
/// src/discount.js still says `price`, the way Node fails there.
const RENAME_BREAKS: &str = "git grep -q unitPrice -- src/cart.js && git grep -q -w price -- src/discount.js && echo TypeError: price is not a function && exit 1 || exit 0";

#[test]
fn json_names_the_cause_and_the_fix_prompt() {
    let dir = rename_repo();
    let out = pytxo(dir.path(), &["--json", "-c", RENAME_BREAKS], None);
    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let verdict = &v["verdict"];
    assert_eq!(verdict["kind"], "together", "{v}");
    assert_eq!(
        verdict["cause"],
        serde_json::json!({
            "by": "claude/rename-price",
            "user": "codex/add-discount",
            "name": "price",
            "renamed_to": "unitPrice",
            "uses": [
                {"path": "src/discount.js", "line": 1},
                {"path": "src/discount.js", "line": 4},
            ],
            "confirmed": true,
        })
    );
    let prompt = verdict["fix_prompt"].as_str().unwrap();
    assert!(
        prompt.starts_with(&format!("Pytxo merged claude/rename-price and codex/add-discount and ran {RENAME_BREAKS}: each passes alone, together they fail.\nclaude/rename-price renamed price to unitPrice. codex/add-discount still uses price at src/discount.js:1, src/discount.js:4.\nFix it on codex/add-discount: rebase it onto claude/rename-price, replace price with unitPrice, and run ")),
        "{prompt}"
    );
    assert!(
        prompt.ends_with("\nFailing output:\nTypeError: price is not a function"),
        "{prompt}"
    );
    // Confirmed: no note on the cause, only what --apply would land.
    assert_eq!(
        v["notes"],
        serde_json::json!([
            "claude/rename-price passes the tests on top of main by itself: run this again with --apply to land it and leave out codex/add-discount."
        ])
    );
    // A reading of the runs, not part of the receipt.
    assert!(v["receipt"]["predicate"].get("cause").is_none(), "{v}");
}

/// The line before the tests and the verdict name the branches alike: up to
/// four by name, then how many.
#[test]
fn progress_and_verdict_name_the_branches_alike() {
    let dir = repo(&["claude/a", "claude/b", "codex/c", "codex/d", "codex/e"]);
    let four = [
        "-c",
        "git --version",
        "claude/a",
        "claude/b",
        "codex/c",
        "codex/d",
    ];
    let five = ["-c", "git --version"];
    for (args, names) in [
        (&four[..], "claude/a, claude/b, codex/c and codex/d"),
        (&five[..], "5 branches"),
    ] {
        let out = pytxo(dir.path(), args, None);
        assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
        let progress = String::from_utf8_lossy(&out.stderr);
        assert!(
            progress.starts_with(&format!("Checking {names} against main")),
            "{progress}"
        );
        assert!(
            stdout(&out).starts_with(&format!("READY    {names} pass")),
            "{}",
            stdout(&out)
        );
    }
}

#[test]
fn named_branches_land_on_the_checked_out_branch_unless_pytxo_into_says() {
    let dir = repo(&["codex/a", "codex/b"]);
    let d = dir.path();
    git(d, &["checkout", "-q", "-b", "develop"]);
    let args = ["--json", "-c", "git --version", "codex/a", "codex/b"];
    let target = |envs: &[(&str, &str)]| {
        let out = pytxo_env(d, &args, None, envs);
        assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        v["target"].as_str().unwrap().to_string()
    };
    assert_eq!(target(&[]), "develop");
    git(d, &["config", "pytxo.into", "main"]);
    assert_eq!(target(&[]), "main");
    // The environment beats git config, as for PYTXO_BRANCHES and PYTXO_CHECK.
    assert_eq!(target(&[("PYTXO_INTO", "develop")]), "develop");
    // A target that does not exist is one clear error, branches named or not.
    let bare: &[&str] = &["-c", "git --version"];
    for args in [&args[..], bare] {
        let out = pytxo_env(d, args, None, &[("PYTXO_INTO", "nope")]);
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(
            String::from_utf8_lossy(&out.stderr),
            "error: target branch does not exist: nope (from PYTXO_INTO)\n"
        );
    }

    // --apply moves the checked-out branch, not main.
    git(d, &["config", "--unset", "pytxo.into"]);
    let main = Command::new("git")
        .args(["rev-parse", "main"])
        .current_dir(d)
        .output()
        .unwrap()
        .stdout;
    let out = pytxo(
        d,
        &["--apply", "-c", "git --version", "codex/a", "codex/b"],
        None,
    );
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(stdout(&out).starts_with("LANDED"), "{}", stdout(&out));
    assert!(d.join("f0.txt").exists() && d.join("f1.txt").exists());
    let after = Command::new("git")
        .args(["rev-parse", "main"])
        .current_dir(d)
        .output()
        .unwrap()
        .stdout;
    assert_eq!(after, main, "main did not move");
}

/// Git reads a tag, or a file in .git such as ORIG_HEAD, before the branch
/// that shares its name: neither changes which commit is checked, found or
/// named, nor makes a named branch the target.
#[test]
fn a_tag_named_like_a_branch_does_not_change_what_is_checked() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    let commit = |rev: &str| {
        let out = Command::new("git")
            .args(["rev-parse", rev])
            .current_dir(d)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    let (a, b) = (commit("claude/a"), commit("codex/b"));
    // Both on main, which neither branch is.
    git(d, &["tag", "claude/a", "main"]);
    git(d, &["branch", "ORIG_HEAD", "codex/b"]);
    git(d, &["update-ref", "--no-deref", "ORIG_HEAD", "main"]);
    let checked = |args: &[&str]| {
        let out = pytxo(d, args, None);
        assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        for (i, commit) in [&a, &b].into_iter().enumerate() {
            assert_eq!(v["branches"][i]["commit"], *commit, "{v}");
            assert_eq!(
                v["receipt"]["predicate"]["inputs"][i]["commit"], *commit,
                "{v}"
            );
        }
        v["target"].as_str().unwrap().to_string()
    };
    let check = ["--json", "-c", "git --version"];
    assert_eq!(checked(&check), "main");
    let named = [&check[..], &["claude/a", "ORIG_HEAD"]].concat();
    assert_eq!(checked(&named), "main");
    // Checked out, claude/a is still one of the branches, not the target.
    git(d, &["checkout", "-q", "claude/a"]);
    assert_eq!(checked(&named), "main");
}

#[test]
fn hook_answers_once_per_change_and_waits_for_background_agents() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    git(d, &["config", "pytxo.check", "git --version"]);
    let event = stop_event(d);
    let busy = event.replace(
        r#""Stop","#,
        r#""Stop","background_tasks":[{"type":"subagent","id":"x"}],"#,
    );
    let out = pytxo(d, &["hook"], Some(&busy));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));

    let out = pytxo(d, &["hook"], Some(&event));
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let message = v["systemMessage"].as_str().unwrap();
    assert!(
        message.contains("READY    claude/a and codex/b pass git --version together."),
        "{message}"
    );

    // Nothing changed: nothing to say.
    let out = pytxo(d, &["hook"], Some(&event));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));

    // Not a repository: quiet too.
    let elsewhere = tempfile::tempdir().unwrap();
    let out = pytxo(elsewhere.path(), &["hook"], Some("{}"));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));
}

#[test]
fn a_hook_says_once_when_a_setting_stops_the_check() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    git(d, &["config", "pytxo.check", "git --version"]);
    git(d, &["config", "pytxo.into", "gone"]);
    let event = stop_event(d);
    let v = hook_reply(&pytxo(d, &["hook"], Some(&event)));
    assert_eq!(
        v["systemMessage"],
        "Pytxo could not check the agent branches: target branch does not exist: gone (from git config pytxo.into)"
    );
    let out = pytxo(d, &["hook"], Some(&event));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));
    // Fixed: the next stop checks.
    git(d, &["config", "--unset", "pytxo.into"]);
    let v = hook_reply(&pytxo(d, &["hook"], Some(&event)));
    assert!(
        v["systemMessage"].as_str().unwrap().contains("READY"),
        "{v}"
    );

    // A new repository with no commit yet has nothing to check.
    let empty = tempfile::tempdir().unwrap();
    git(empty.path(), &["init", "-q", "-b", "main"]);
    let out = pytxo(empty.path(), &["hook"], Some(&stop_event(empty.path())));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));
    let out = pytxo(empty.path(), &[], None);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).starts_with("No agent branches to check against main"));
}

/// The JSON a `Stop` hook gets on stdin.
fn stop_event(cwd: &Path) -> String {
    format!(
        r#"{{"hook_event_name":"Stop","cwd":{}}}"#,
        serde_json::to_string(&cwd.display().to_string()).unwrap()
    )
}

fn hook_reply(out: &Output) -> serde_json::Value {
    assert_eq!(out.status.code(), Some(0), "the hook never fails the turn");
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {}", stdout(out)))
}

#[test]
fn hook_block_turns_blocked_into_a_block_decision() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    git(d, &["config", "pytxo.check", "exit 1"]);
    // A timeout no clock can count is no deadline, not a crash.
    let forever = u64::MAX.to_string();
    let args = ["hook", "--block", "--timeout", &forever];
    let v = hook_reply(&pytxo(d, &args, Some(&stop_event(d))));
    assert_eq!(v["decision"], "block", "{v}");
    let reason = v["reason"].as_str().unwrap();
    assert!(reason.contains("\nBLOCKED  "), "{reason}");
    assert!(v.get("systemMessage").is_none(), "{v}");
}

/// With a confirmed cause, `--block` hands the fix prompt to the agent whose
/// worktree the hook runs in: the one that has to change its branch, the
/// one that renamed the name, or anyone.
#[test]
fn hook_block_hands_the_fix_to_the_agent_whose_branch_it_is() {
    let dir = rename_repo();
    let d = dir.path();
    git(d, &["config", "pytxo.check", RENAME_BREAKS]);
    let trees = tempfile::tempdir().unwrap();
    let user_tree = trees.path().join("discount");
    let by_tree = trees.path().join("rename");
    for (tree, branch) in [
        (&user_tree, "codex/add-discount"),
        (&by_tree, "claude/rename-price"),
    ] {
        git(
            d,
            &["worktree", "add", "-q", tree.to_str().unwrap(), branch],
        );
    }
    let pytxo_dir = d.join(".git").join("pytxo");
    let memo = pytxo_dir.join("hook-last");
    let stop = |cwd: &Path, args: &[&str]| pytxo(cwd, args, Some(&stop_event(cwd)));
    let reply = |cwd: &Path, args: &[&str]| hook_reply(&stop(cwd, args));
    let reason = |cwd: &Path| {
        let v = reply(cwd, &["hook", "--block"]);
        assert_eq!(v["decision"], "block", "{v}");
        v["reason"].as_str().unwrap().to_string()
    };
    let receipts = || {
        std::fs::read_dir(pytxo_dir.join("receipts"))
            .unwrap()
            .count()
    };
    // The agent that stops first gets the check and its own prompt; the
    // other gets its own at its next stop, from the same check, and each
    // hears it once.
    let user = reason(&user_tree);
    assert!(
        user.starts_with(
            "Pytxo merged claude/rename-price and your branch codex/add-discount and ran "
        ),
        "{user}"
    );
    assert!(
        user.contains("\nclaude/rename-price renamed price to unitPrice. Your branch codex/add-discount still uses price at src/discount.js:1, src/discount.js:4.\nFix it on your branch: rebase it onto claude/rename-price, replace price with unitPrice, and run "),
        "{user}"
    );
    let checked = receipts();
    let by = reason(&by_tree);
    assert_eq!(receipts(), checked);
    for tree in [&by_tree, &user_tree] {
        let again = stop(tree, &["hook", "--block"]);
        assert_eq!(stdout(&again).trim(), "", "said once per agent");
    }
    assert!(
        by.starts_with(
            "Pytxo merged your branch claude/rename-price and codex/add-discount and ran "
        ),
        "{by}"
    );
    assert!(
        by.contains("\nYour branch claude/rename-price renamed price to unitPrice. codex/add-discount still uses price at src/discount.js:1, src/discount.js:4.\nThe fix belongs on codex/add-discount: rebase it onto claude/rename-price, "),
        "{by}"
    );
    assert!(
        by.contains(
            " The other way out is to keep price on your branch as an alias of unitPrice.\n"
        ),
        "{by}"
    );
    // Same tips, same verdict: each stop below is a new check.
    let _ = std::fs::remove_file(&memo);
    let anyone = reason(d);
    assert!(
        anyone.starts_with("Pytxo merged claude/rename-price and codex/add-discount and ran "),
        "{anyone}"
    );
    // Without --block the message stays the three lines.
    let _ = std::fs::remove_file(&memo);
    let v = reply(&user_tree, &["hook"]);
    let message = v["systemMessage"].as_str().unwrap();
    assert!(
        message.contains("\nwhy      claude/rename-price renamed price to unitPrice; codex/add-discount still uses price (src/discount.js:1).\nnext     rebase codex/add-discount onto claude/rename-price and use unitPrice, then Pytxo checks again when an agent stops.\n"),
        "{message}"
    );
}

#[test]
fn one_hook_checks_a_repository_at_a_time() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    git(d, &["config", "pytxo.check", "git --version"]);
    let event = stop_event(d);
    // Another session's hook holds a fresh lock: this one waits for it until
    // its own deadline, half of a short timeout, less a last look that would
    // pass it, then has nothing to add, and the lock stays.
    let lock = d.join(".git").join("pytxo").join("hook.lock");
    std::fs::create_dir_all(lock.parent().unwrap()).unwrap();
    let holder = format!("{} other", std::process::id());
    std::fs::write(&lock, &holder).unwrap();
    let started = std::time::Instant::now();
    let out = pytxo(d, &["hook", "--timeout", "10"], Some(&event));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));
    let waited = started.elapsed();
    assert!(waited >= std::time::Duration::from_secs(4), "{waited:?}");
    assert_eq!(std::fs::read_to_string(&lock).unwrap(), holder);

    // Its hook is gone, killed mid-check: taken over at once. No system hands
    // out this process id.
    std::fs::write(&lock, "1073741823 other").unwrap();
    let v = hook_reply(&pytxo(d, &["hook"], Some(&event)));
    assert!(
        v["systemMessage"].as_str().unwrap().contains("READY"),
        "{v}"
    );
    assert!(!lock.exists(), "released after a verdict");

    // Older than the hook timeout, though a process with its id runs: taken
    // over, and released when nothing changed.
    std::fs::write(&lock, &holder).unwrap();
    let killed = std::time::SystemTime::now() - std::time::Duration::from_secs(120);
    let file = std::fs::File::options().write(true).open(&lock).unwrap();
    file.set_modified(killed).unwrap();
    drop(file);
    let out = pytxo(d, &["hook", "--timeout", "60"], Some(&event));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));
    assert!(!lock.exists(), "released when quiet");

    // Released when the check fails to run.
    git(d, &["commit", "-q", "--allow-empty", "-m", "moved"]);
    let receipts = d.join(".git").join("pytxo").join("receipts");
    std::fs::remove_dir_all(&receipts).unwrap();
    std::fs::write(&receipts, "not a directory").unwrap();
    let v = hook_reply(&pytxo(d, &["hook"], Some(&event)));
    assert!(
        v["systemMessage"]
            .as_str()
            .unwrap()
            .contains("could not check"),
        "{v}"
    );
    assert!(!lock.exists(), "released after an error");
    // Said once, like a verdict: the next stop with the same tips is quiet.
    let out = pytxo(d, &["hook"], Some(&event));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));
}

/// A kept prompt is for the agent on its branch, not for whatever the
/// worktree has checked out by its next stop.
#[test]
fn a_kept_prompt_waits_for_its_branch_not_its_worktree() {
    let dir = rename_repo();
    let d = dir.path();
    git(d, &["config", "pytxo.check", RENAME_BREAKS]);
    let trees = tempfile::tempdir().unwrap();
    let user_tree = trees.path().join("discount");
    let by_tree = trees.path().join("rename");
    for (tree, branch) in [
        (&user_tree, "codex/add-discount"),
        (&by_tree, "claude/rename-price"),
    ] {
        git(
            d,
            &["worktree", "add", "-q", tree.to_str().unwrap(), branch],
        );
    }
    let stop = |cwd: &Path| pytxo(cwd, &["hook", "--block"], Some(&stop_event(cwd)));
    // The renaming agent stops first and gets the check.
    assert_eq!(hook_reply(&stop(&by_tree))["decision"], "block");
    // The other worktree moves on to a new task: a branch from main with no
    // commits yet, which leaves every branch and tip as the check found them.
    git(&user_tree, &["switch", "-q", "-c", "claude/next", "main"]);
    assert_eq!(stdout(&stop(&user_tree)), "", "not this agent's branch");
    // Back on its branch, its agent hears it.
    git(&user_tree, &["switch", "-q", "codex/add-discount"]);
    let v = hook_reply(&stop(&user_tree));
    let reason = v["reason"].as_str().unwrap();
    assert!(
        reason.starts_with(
            "Pytxo merged claude/rename-price and your branch codex/add-discount and ran "
        ),
        "{reason}"
    );
}

/// A kept prompt is said the way the hook that says it answers: Claude
/// Code's and Codex's hooks are set up apart, so one may block while the
/// other only tells.
#[test]
fn a_kept_prompt_answers_as_the_hook_that_says_it() {
    let dir = rename_repo();
    let d = dir.path();
    git(d, &["config", "pytxo.check", RENAME_BREAKS]);
    let trees = tempfile::tempdir().unwrap();
    let user_tree = trees.path().join("discount");
    let by_tree = trees.path().join("rename");
    for (tree, branch) in [
        (&user_tree, "codex/add-discount"),
        (&by_tree, "claude/rename-price"),
    ] {
        git(
            d,
            &["worktree", "add", "-q", tree.to_str().unwrap(), branch],
        );
    }
    let stop = |cwd: &Path, args: &[&str]| hook_reply(&pytxo(cwd, args, Some(&stop_event(cwd))));
    // The renaming agent's hook blocks; the other agent's only tells.
    assert_eq!(stop(&by_tree, &["hook", "--block"])["decision"], "block");
    let v = stop(&user_tree, &["hook"]);
    assert!(v.get("decision").is_none(), "{v}");
    let message = v["systemMessage"].as_str().unwrap();
    assert!(
        message.starts_with("Pytxo checked the agent branches together.\nBLOCKED  "),
        "{message}"
    );
    // The other way round, for a new check.
    std::fs::remove_file(d.join(".git").join("pytxo").join("hook-last")).unwrap();
    assert!(stop(&by_tree, &["hook"])["systemMessage"].is_string());
    let v = stop(&user_tree, &["hook", "--block"]);
    let reason = v["reason"].as_str().unwrap();
    assert!(
        reason.starts_with(
            "Pytxo merged claude/rename-price and your branch codex/add-discount and ran "
        ),
        "{reason}"
    );
}

/// An agent that stops while another agent's hook checks waits for that
/// check and hears its own prompt from it at this stop, not at the next one,
/// which only comes when a person prompts it again.
#[test]
fn a_hook_that_stops_during_another_check_waits_for_its_prompt() {
    let dir = rename_repo();
    let d = dir.path();
    // Two seconds a run, so the second agent stops while the first checks.
    #[cfg(windows)]
    let slow = "ping -n 3 127.0.0.1 > nul";
    #[cfg(not(windows))]
    let slow = "sleep 2";
    let check = format!("{slow} && {RENAME_BREAKS}");
    git(d, &["config", "pytxo.check", &check]);
    let trees = tempfile::tempdir().unwrap();
    let user_tree = trees.path().join("discount");
    let by_tree = trees.path().join("rename");
    for (tree, branch) in [
        (&user_tree, "codex/add-discount"),
        (&by_tree, "claude/rename-price"),
    ] {
        git(
            d,
            &["worktree", "add", "-q", tree.to_str().unwrap(), branch],
        );
    }
    let pytxo_dir = d.join(".git").join("pytxo");
    let lock = pytxo_dir.join("hook.lock");
    let reason = |cwd: &Path| {
        let v = hook_reply(&pytxo(cwd, &["hook", "--block"], Some(&stop_event(cwd))));
        assert_eq!(v["decision"], "block", "{v}");
        v["reason"].as_str().unwrap().to_string()
    };
    let (by, user) = std::thread::scope(|s| {
        let first = s.spawn(|| reason(&by_tree));
        while !lock.exists() {
            assert!(!first.is_finished(), "the first hook never checked");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let user = reason(&user_tree);
        (first.join().unwrap(), user)
    });
    assert!(
        by.starts_with(
            "Pytxo merged your branch claude/rename-price and codex/add-discount and ran "
        ),
        "{by}"
    );
    assert!(
        user.starts_with(
            "Pytxo merged claude/rename-price and your branch codex/add-discount and ran "
        ),
        "{user}"
    );
    let receipts = std::fs::read_dir(pytxo_dir.join("receipts")).unwrap();
    assert_eq!(receipts.count(), 1, "one check for both");
}

#[test]
fn a_hook_out_of_time_says_so_and_leaves_no_worktree() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    #[cfg(windows)]
    let slow = "ping -n 60 127.0.0.1 > nul";
    #[cfg(not(windows))]
    let slow = "sleep 60";
    git(d, &["config", "pytxo.check", slow]);
    let started = std::time::Instant::now();
    // A 20-second hook answers after about 10; the test alone takes 60. On
    // Windows the git calls before the test starts take seconds, more so
    // with the other tests running.
    let out = pytxo(
        d,
        &["hook", "--block", "--timeout", "20"],
        Some(&stop_event(d)),
    );
    let took = started.elapsed();
    let v = hook_reply(&out);
    assert!(took < std::time::Duration::from_secs(20), "{took:?}");
    let message = v["systemMessage"].as_str().unwrap();
    assert!(
        message.contains("could not finish") && message.contains("Run pytxo in a terminal"),
        "{message}"
    );
    assert!(
        v.get("decision").is_none(),
        "running out of time never blocks"
    );
    let list = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(d)
        .output()
        .unwrap();
    let list = String::from_utf8_lossy(&list.stdout);
    assert_eq!(list.matches("worktree ").count(), 1, "{list}");
}

#[cfg(not(feature = "full"))]
#[test]
fn pytxo_1x_commands_say_this_build_leaves_them_out() {
    let dir = repo(&["claude/a"]);
    let d = dir.path();
    // With the 1.x flags too, which this build's parser does not know, and
    // asking for its help.
    let lines: [(&[&str], &str); 9] = [
        (&["shell"], "shell"),
        (&["doctor"], "doctor"),
        (&["shell", "--eval", "x"], "shell"),
        (&["doctor", "--quick"], "doctor"),
        (&["logs", "--agent", "a"], "logs"),
        (&["help", "shell"], "shell"),
        (&["shell", "--help"], "shell"),
        (&["receipts", "share", "rcpt_x"], "receipts share"),
        (&["bench", "--submit"], "bench --submit"),
    ];
    for (args, name) in lines {
        let out = pytxo(d, args, None);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            err,
            format!("error: {name} is a Pytxo 1.x command that is not in this build; a source build with default features has it\n"),
            "{args:?}"
        );
    }
    // Help for what this build has is still help.
    for args in [
        &["--help"][..],
        &["help"],
        &["help", "check"],
        &["hook", "--help"],
    ] {
        assert_eq!(pytxo(d, args, None).status.code(), Some(0), "{args:?}");
    }
    // Outside a repository too.
    let elsewhere = tempfile::tempdir().unwrap();
    let out = pytxo(elsewhere.path(), &["mission", "fix it"], None);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("mission is a Pytxo 1.x command"));
    // A branch of that name is checked like any other.
    git(d, &["branch", "run", "claude/a"]);
    let out = pytxo(d, &["run", "-c", "git --version"], None);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
}

#[test]
fn usage_errors_exit_one_not_two() {
    let dir = repo(&[]);
    assert_eq!(
        pytxo(dir.path(), &["--no-such-flag"], None).status.code(),
        Some(1)
    );
    assert_eq!(pytxo(dir.path(), &["--help"], None).status.code(), Some(0));
}

#[test]
fn the_demo_catches_the_break() {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("skipped: the demo needs Node.js");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    // A temp dir of its own, to see the demo's repo and worktrees gone after.
    let temp = tempfile::tempdir().unwrap();
    let t = temp.path().to_string_lossy().into_owned();
    // The user's own settings name no branch, test or target of the demo's.
    let started = std::time::Instant::now();
    let out = pytxo_env(
        dir.path(),
        &["demo"],
        None,
        &[
            ("PYTXO_INTO", "develop"),
            ("PYTXO_BRANCHES", "team/*"),
            ("PYTXO_CHECK", "exit 1"),
            ("PYTXO_DAYS", "1"),
            ("TEMP", &t),
            ("TMP", &t),
            ("TMPDIR", &t),
        ],
    );
    let took = started.elapsed();
    let text = stdout(&out);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{text}{err}");
    assert_eq!(
        text,
        "BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.
why      claude/rename-price renamed price to unitPrice; codex/add-discount still uses price (src/discount.js:1).
next     rebase codex/add-discount onto claude/rename-price and use unitPrice, then run pytxo again.
"
    );
    assert!(
        err.contains("against main with npm test (from package.json)..."),
        "{err}"
    );
    assert!(
        err.contains("That repo was temporary and is gone."),
        "{err}"
    );
    // The demo's budget is 30 seconds, npx included.
    assert!(took < std::time::Duration::from_secs(30), "{took:?}");
    let left: Vec<String> = std::fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.starts_with("pybench-") || n.starts_with("pytxo-"))
        .collect();
    assert!(left.is_empty(), "left behind: {left:?}");
}
