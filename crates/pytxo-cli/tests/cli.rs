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
    let v = hook_reply(&pytxo(d, &["hook", "--block"], Some(&stop_event(d))));
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
    for command in ["shell", "doctor"] {
        let out = pytxo(d, &[command], None);
        assert_eq!(out.status.code(), Some(1), "{command}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(err.lines().count(), 1, "{err}");
        assert!(
            err.contains(&format!("{command} is a Pytxo 1.x command"))
                && err.contains("a source build with default features has it"),
            "{err}"
        );
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
    // The user's own target setting names no branch of the demo's.
    let out = pytxo_env(dir.path(), &["demo"], None, &[("PYTXO_INTO", "develop")]);
    let text = stdout(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.starts_with("BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together."),
        "{text}"
    );
    assert!(text.contains("price is not a function"), "{text}");
}
