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

    // An old agent branch, merged long ago: counted as not looked at, which
    // is all the age test knows; PYTXO_DAYS=0 looks and finds it merged.
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
    assert!(
        text.contains(" 1 with no commit in 14 days not looked at (PYTXO_DAYS=0 looks at them). "),
        "{text}"
    );
    let text = stdout(&pytxo_env(d, &[], None, &[("PYTXO_DAYS", "0")]));
    assert!(text.starts_with("No agent branches"), "{text}");
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

#[test]
fn one_hook_checks_a_repository_at_a_time() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    git(d, &["config", "pytxo.check", "git --version"]);
    let event = stop_event(d);
    // Another session's hook holds a fresh lock: quiet, and its lock stays.
    let lock = d.join(".git").join("pytxo").join("hook.lock");
    std::fs::create_dir_all(lock.parent().unwrap()).unwrap();
    std::fs::write(&lock, "4242 other").unwrap();
    let out = pytxo(d, &["hook"], Some(&event));
    assert_eq!((out.status.code(), stdout(&out)), (Some(0), String::new()));
    assert_eq!(std::fs::read_to_string(&lock).unwrap(), "4242 other");

    // Older than the hook timeout: a killed hook's, so it is taken over.
    let killed = std::time::SystemTime::now() - std::time::Duration::from_secs(120);
    let file = std::fs::File::options().write(true).open(&lock).unwrap();
    file.set_modified(killed).unwrap();
    drop(file);
    let v = hook_reply(&pytxo(d, &["hook", "--timeout", "60"], Some(&event)));
    assert!(
        v["systemMessage"].as_str().unwrap().contains("READY"),
        "{v}"
    );
    assert!(!lock.exists(), "released after a verdict");

    // Released when nothing changed, and when the check fails to run.
    let out = pytxo(d, &["hook"], Some(&event));
    assert_eq!(stdout(&out), "");
    assert!(!lock.exists(), "released when quiet");
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

#[test]
fn a_hook_out_of_time_says_so_and_leaves_no_worktree() {
    let dir = repo(&["claude/a", "codex/b"]);
    let d = dir.path();
    #[cfg(windows)]
    let slow = "ping -n 30 127.0.0.1 > nul";
    #[cfg(not(windows))]
    let slow = "sleep 30";
    git(d, &["config", "pytxo.check", slow]);
    let started = std::time::Instant::now();
    // A 10-second hook answers after about 5; the test alone takes 30.
    let out = pytxo(
        d,
        &["hook", "--block", "--timeout", "10"],
        Some(&stop_event(d)),
    );
    let took = started.elapsed();
    let v = hook_reply(&out);
    assert!(took < std::time::Duration::from_secs(10), "{took:?}");
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
    assert!(
        text.starts_with("BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together."),
        "{text}"
    );
    assert!(text.contains("price is not a function"), "{text}");
    assert!(
        err.contains("against main with npm test (from package.json)..."),
        "{err}"
    );
    assert!(
        err.contains("That repo was temporary and is gone."),
        "{err}"
    );
    // A first look takes under a minute.
    assert!(took < std::time::Duration::from_secs(60), "{took:?}");
    let left: Vec<String> = std::fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.starts_with("pybench-") || n.starts_with("pytxo-"))
        .collect();
    assert!(left.is_empty(), "left behind: {left:?}");
}
