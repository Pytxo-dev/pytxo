//! Agent-drafted task split for a free-form request.
//!
//! Scope: read-only drafting inside one execution domain's repository root,
//! outside any permission-profile run. The selected vendor CLI runs in its own
//! read-only/plan mode with Pytxo's clean child environment. Nothing is
//! registered, flushed or applied, and the vendor's read-only mode is advisory,
//! so the project is checked for changes afterwards. The result is editable
//! text: the deterministic planner still decides ownership, order and checks.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{bail, Context};
use pytxo_core::ChildLaunchEnv;
use serde::Serialize;

pub const SPLIT_MAX_TASKS: usize = 8;
const REQUEST_MAX_CHARS: usize = 8_000;
const TASK_MAX_CHARS: usize = 600;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SplitTask {
    pub text: String,
    pub files: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SplitDraft {
    pub ade_id: String,
    pub tasks: Vec<SplitTask>,
    /// One `<task> | files: a, b` line per task, ready for the planner.
    pub mission_text: String,
    /// `None` when the project is not a Git repository and could not be compared.
    pub project_changed: Option<bool>,
    pub elapsed_ms: u64,
}

/// Vendor CLIs whose documented headless mode can be held read-only.
pub fn split_supported(ade_id: &str) -> bool {
    split_args(ade_id, Path::new("out.txt")).is_some()
}

fn split_args(ade_id: &str, last_message: &Path) -> Option<(&'static str, Vec<String>)> {
    match ade_id {
        "codex" => Some((
            "codex",
            [
                "exec",
                // A split is a short planning read; a user default such as
                // "max" effort would outrun the split timeout.
                "-c",
                "model_reasoning_effort=\"medium\"",
                "--sandbox",
                "read-only",
                "--skip-git-repo-check",
                "--ephemeral",
                "--color",
                "never",
                "-o",
            ]
            .into_iter()
            .map(str::to_owned)
            .chain([last_message.to_string_lossy().into_owned(), "-".to_owned()])
            .collect(),
        )),
        "claude" => Some((
            "claude",
            [
                "-p",
                "--permission-mode",
                "plan",
                "--allowedTools",
                "Read,Glob,Grep",
                "--output-format",
                "text",
                "--no-session-persistence",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )),
        _ => None,
    }
}

pub fn split_prompt(request: &str) -> String {
    format!(
        "You are planning work for several coding agents that will run in parallel, each in its own copy of this repository. Do not edit, create or delete any file and do not run commands that change anything.

Request:
{request}

Read the repository, then split the request into 1-{SPLIT_MAX_TASKS} tasks that different agents can do at the same time. Rules:
- Each task owns the files it may change. Two tasks must not own the same file.
- Name only repository-relative paths with forward slashes. Existing files must exist; a new file must sit in an existing directory.
- Write each task as one plain sentence an engineer could act on. Do not use semicolons.
- If the request is small, one task is the right answer.

Reply with ONLY the task lines, one per line, in this exact form:
<task sentence> | files: <path>, <path>"
    )
}

/// Parse agent output into tasks whose every file is a safe ownership path.
pub fn parse_split(output: &str, repo: &Path) -> Vec<SplitTask> {
    let mut tasks: Vec<SplitTask> = Vec::new();
    for raw in output.lines() {
        let line = raw
            .trim()
            .trim_start_matches(|c: char| matches!(c, '-' | '*' | '•' | '>') || c.is_whitespace());
        let line = strip_ordinal(line);
        let Some((sentence, tail)) = line.rsplit_once('|') else {
            continue;
        };
        let tail = tail.trim_start();
        let Some(list) = tail
            .get(..6)
            .filter(|head| head.eq_ignore_ascii_case("files:"))
            .map(|_| &tail[6..])
        else {
            continue;
        };
        let text = sentence.replace(';', ",").trim().to_owned();
        if text.is_empty() || text.chars().count() > TASK_MAX_CHARS {
            continue;
        }
        let mut files = Vec::new();
        for item in list.split(',') {
            let item = item
                .trim()
                .trim_end_matches('.')
                .trim_matches(|c: char| matches!(c, '`' | '"' | '\''))
                .trim_end_matches('.')
                .trim_start_matches("./");
            if let Ok(path) = pytxo_planner::validate_planner_path(repo, item) {
                if !files.contains(&path) {
                    files.push(path);
                }
            }
        }
        if files.is_empty() {
            continue;
        }
        let task = SplitTask { text, files };
        if !tasks.contains(&task) {
            tasks.push(task);
        }
        if tasks.len() == SPLIT_MAX_TASKS {
            break;
        }
    }
    tasks
}

fn strip_ordinal(line: &str) -> &str {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 {
        if let Some(rest) = line[digits..].strip_prefix(['.', ')']) {
            return rest.trim_start();
        }
    }
    line
}

pub fn mission_text(tasks: &[SplitTask]) -> String {
    tasks
        .iter()
        .map(|task| format!("{} | files: {}", task.text, task.files.join(", ")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Ask `ade_id` to split `request` for the repository at `repo`.
pub fn split_request(
    repo: &Path,
    request: &str,
    ade_id: &str,
    timeout: Duration,
    cancel: &AtomicBool,
) -> anyhow::Result<SplitDraft> {
    let request = request.trim();
    if request.is_empty() {
        bail!("Describe the job before asking an agent to split it.");
    }
    if request.chars().count() > REQUEST_MAX_CHARS {
        bail!("The request is longer than {REQUEST_MAX_CHARS} characters. Shorten it, then split again.");
    }
    let repo = repo
        .canonicalize()
        .with_context(|| format!("project folder is unavailable: {}", repo.display()))?;
    let scratch = std::env::temp_dir().join(format!("pytxo-split-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&scratch).context("create split scratch folder")?;
    let last_message = scratch.join("last-message.txt");
    let result = run_split(&repo, request, ade_id, timeout, cancel, &last_message);
    let _ = std::fs::remove_dir_all(&scratch);
    result
}

fn run_split(
    repo: &Path,
    request: &str,
    ade_id: &str,
    timeout: Duration,
    cancel: &AtomicBool,
    last_message: &Path,
) -> anyhow::Result<SplitDraft> {
    let Some((name, args)) = split_args(ade_id, last_message) else {
        bail!("Splitting a request is available with Codex and Claude Code.");
    };
    let program =
        find_program(name).with_context(|| format!("{name} is not installed or not on PATH"))?;
    let before = project_fingerprint(repo);
    let started = Instant::now();

    let mut command = Command::new(&program);
    command
        .args(&args)
        .current_dir(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    ChildLaunchEnv::new().apply_command(&mut command);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("could not start {name}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(split_prompt(request).as_bytes())
            .with_context(|| format!("could not send the request to {name}"))?;
    }
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());

    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        let cancelled = cancel.load(Ordering::Relaxed);
        if cancelled || started.elapsed() >= timeout {
            stop_tree(&mut child);
            if cancelled {
                bail!("Split cancelled. Your request was not changed.");
            }
            let label = if name == "codex" { "OpenAI Codex" } else { "Claude Code" };
            bail!(
                "{label} did not finish within {} minutes. Your request was not changed.",
                timeout.as_secs() / 60
            );
        }
        std::thread::sleep(Duration::from_millis(200));
    };
    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();
    let output = std::fs::read_to_string(last_message)
        .ok()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or(stdout);
    let tasks = parse_split(&output, repo);
    let project_changed = match (before, project_fingerprint(repo)) {
        (Some(before), Some(after)) => Some(before != after),
        _ => None,
    };
    if tasks.is_empty() {
        let detail = if status.success() { &output } else { &stderr };
        let tail: String = detail
            .lines()
            .rev()
            .filter(|line| !line.trim().is_empty())
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|line| sanitize(line.trim()))
            .collect::<Vec<_>>()
            .join(" / ");
        bail!(
            "{name} returned no usable tasks{}. Try naming the main files, or write one task per line yourself.",
            if tail.is_empty() {
                String::new()
            } else {
                format!(": {}", truncate(&tail, 240))
            }
        );
    }
    Ok(SplitDraft {
        ade_id: ade_id.to_owned(),
        mission_text: mission_text(&tasks),
        tasks,
        project_changed,
        elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

fn drain<R: Read + Send + 'static>(source: Option<R>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut source) = source {
            let _ = source.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

fn stop_tree(child: &mut std::process::Child) {
    // The child handle is still open, so its PID cannot be reused yet.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .creation_flags(0x0800_0000)
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// HEAD plus porcelain status; equal fingerprints mean no visible project change.
fn project_fingerprint(repo: &Path) -> Option<String> {
    let git = |args: &[&str]| -> Option<String> {
        let mut command = Command::new("git");
        command.arg("-C").arg(repo).args(args);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let output = command.output().ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
    };
    let head = git(&["rev-parse", "--verify", "-q", "HEAD"]).unwrap_or_default();
    let status = git(&["status", "--porcelain=v1", "--untracked-files=all"])?;
    Some(format!("{head}\n{status}"))
}

fn find_program(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let names: Vec<String> = if cfg!(windows) {
        ["exe", "cmd", "bat"]
            .iter()
            .map(|extension| format!("{name}.{extension}"))
            .collect()
    } else {
        vec![name.to_owned()]
    };
    std::env::split_paths(&path).find_map(|dir| {
        names
            .iter()
            .map(|candidate| dir.join(candidate))
            .find(|candidate| candidate.is_file())
    })
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    format!("{}…", text.chars().take(max).collect::<String>())
}

fn sanitize(line: &str) -> String {
    #[cfg(feature = "sanitize")]
    {
        pytxo_sanitize::sanitize_line(line)
    }
    #[cfg(not(feature = "sanitize"))]
    {
        line.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join("test")).unwrap();
        std::fs::write(dir.path().join("src/risk-policy.mjs"), "export {}\n").unwrap();
        std::fs::write(dir.path().join("README.md"), "# x\n").unwrap();
        dir
    }

    #[test]
    fn parses_task_lines_and_drops_unsafe_or_unowned_tasks() {
        let dir = repo();
        let output = "Here is the plan:\n\
            1. Add risk summaries; keep the API | files: `src/risk-policy.mjs`\n\
            - Add regression tests | files: test/risk-policy.test.mjs\n\
            Add regression tests | files: test/risk-policy.test.mjs\n\
            Escape the repo | files: ../outside.txt, C:/Windows/x.txt\n\
            Write into a missing folder | files: lib/missing/x.js\n\
            Document two examples. | Files: ./README.md.\n\
            No ownership marker here src/risk-policy.mjs\n";
        let tasks = parse_split(output, dir.path());
        assert_eq!(
            tasks,
            vec![
                SplitTask {
                    text: "Add risk summaries, keep the API".into(),
                    files: vec!["src/risk-policy.mjs".into()]
                },
                SplitTask {
                    text: "Add regression tests".into(),
                    files: vec!["test/risk-policy.test.mjs".into()]
                },
                SplitTask {
                    text: "Document two examples.".into(),
                    files: vec!["README.md".into()]
                },
            ]
        );
        assert_eq!(
            mission_text(&tasks).lines().next(),
            Some("Add risk summaries, keep the API | files: src/risk-policy.mjs")
        );
    }

    #[test]
    fn caps_task_count() {
        let dir = repo();
        let output: String = (0..12)
            .map(|index| format!("Task {index} | files: src/f{index}.mjs\n"))
            .collect();
        assert_eq!(parse_split(&output, dir.path()).len(), SPLIT_MAX_TASKS);
    }

    #[test]
    fn only_read_only_adapters_are_offered() {
        assert!(split_supported("codex"));
        assert!(split_supported("claude"));
        assert!(!split_supported("opencode"));
        let (_, codex) = split_args("codex", Path::new("m.txt")).unwrap();
        assert!(codex
            .windows(2)
            .any(|pair| pair == ["--sandbox", "read-only"]));
        assert!(codex
            .windows(2)
            .any(|pair| pair == ["-c", "model_reasoning_effort=\"medium\""]));
        let (_, claude) = split_args("claude", Path::new("m.txt")).unwrap();
        assert!(claude
            .windows(2)
            .any(|pair| pair == ["--permission-mode", "plan"]));
        assert!(!claude
            .iter()
            .any(|arg| arg.contains("Edit") || arg.contains("Bash")));
    }

    #[test]
    #[ignore = "runs a signed-in vendor CLI; set PYTXO_SPLIT_LIVE_REPO and PYTXO_SPLIT_LIVE_ADE"]
    fn live_split_drafts_tasks_and_leaves_the_project_unchanged() {
        let repo = PathBuf::from(std::env::var("PYTXO_SPLIT_LIVE_REPO").unwrap());
        let ade = std::env::var("PYTXO_SPLIT_LIVE_ADE").unwrap();
        let draft = split_request(
            &repo,
            "Make risky changes easier to review: summarize network and destructive command risks, cover them with tests, and document examples.",
            &ade,
            Duration::from_secs(300),
            &AtomicBool::new(false),
        )
        .unwrap();
        eprintln!("{:#?}", draft);
        assert!(!draft.tasks.is_empty());
        assert_eq!(draft.project_changed, Some(false));
    }

    #[test]
    fn rejects_empty_and_unsupported_requests() {
        let dir = repo();
        let cancel = AtomicBool::new(false);
        let empty = split_request(dir.path(), "  ", "codex", Duration::from_secs(1), &cancel);
        assert!(empty.unwrap_err().to_string().contains("Describe the job"));
        let other = split_request(
            dir.path(),
            "Fix it",
            "opencode",
            Duration::from_secs(1),
            &cancel,
        );
        assert!(other
            .unwrap_err()
            .to_string()
            .contains("Codex and Claude Code"));
    }
}
