//! Run one check command in a worktree with a timeout, keeping the output tail.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use pytxo_core::ReceiptCheck;

const TAIL_LINES: usize = 40;

pub fn run_check(scope: &str, command: &str, cwd: &Path, timeout: Duration) -> ReceiptCheck {
    let started = Instant::now();
    let finish = |exit_code: Option<i32>, timed_out: bool, tail: Vec<String>| ReceiptCheck {
        scope: scope.to_string(),
        command: command.to_string(),
        exit_code,
        passed: exit_code == Some(0) && !timed_out,
        duration_ms: started.elapsed().as_millis() as u64,
        timed_out,
        output_tail: tail,
        rerun: false,
    };

    let log_path =
        std::env::temp_dir().join(format!("pytxo-check-{}.log", uuid::Uuid::new_v4().simple()));
    let spawned = File::create(&log_path).and_then(|log| {
        let err = log.try_clone()?;
        shell(command)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(err))
            .spawn()
    });
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            let _ = std::fs::remove_file(&log_path);
            return finish(None, false, vec![format!("could not start check: {error}")]);
        }
    };

    let (exit_code, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) if started.elapsed() >= timeout => {
                kill_tree(&mut child);
                break (None, true);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
            Err(_) => break (None, false),
        }
    };
    let tail = read_tail(&log_path);
    let _ = std::fs::remove_file(&log_path);
    finish(exit_code, timed_out, tail)
}

/// A check command for the project at `root`, to print as a suggestion; never
/// run unasked. Checks run in a fresh worktree, so Node suggestions install first.
pub fn suggest_check(root: &Path) -> Option<String> {
    let has = |file: &str| root.join(file).exists();
    let npm_test = std::fs::read_to_string(root.join("package.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|pkg| pkg["scripts"]["test"].as_str().map(str::to_string))
        .is_some_and(|test| !test.contains("no test specified"));
    let command = if npm_test {
        if has("pnpm-lock.yaml") {
            "pnpm install --frozen-lockfile && pnpm test"
        } else if has("yarn.lock") {
            "yarn install && yarn test"
        } else if has("package-lock.json") || has("npm-shrinkwrap.json") {
            "npm ci && npm test"
        } else {
            "npm install && npm test"
        }
    } else if has("Cargo.toml") {
        "cargo test"
    } else if has("go.mod") {
        "go test ./..."
    } else if [
        "pyproject.toml",
        "pytest.ini",
        "setup.py",
        "setup.cfg",
        "tox.ini",
    ]
    .iter()
    .any(|f| has(f))
    {
        "python -m pytest"
    } else {
        return None;
    };
    Some(command.into())
}

/// Whether `command` installs dependencies before testing.
pub fn installs(command: &str) -> bool {
    let padded = format!("{command} ");
    [
        "npm install",
        "npm i ",
        "npm ci",
        "yarn install",
        "bun install",
    ]
    .iter()
    .any(|step| padded.contains(step))
}

fn shell(command: &str) -> std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = pytxo_core::background_command("cmd");
        cmd.arg("/d")
            .arg("/s")
            .arg("/c")
            .raw_arg(format!("\"{command}\""));
        cmd
    }
    #[cfg(not(windows))]
    {
        let mut cmd = pytxo_core::background_command("sh");
        cmd.arg("-c").arg(command);
        cmd
    }
}

fn kill_tree(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let _ = pytxo_core::background_command("taskkill")
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn read_tail(path: &Path) -> Vec<String> {
    let mut bytes = Vec::new();
    if File::open(path)
        .and_then(|mut f| f.read_to_end(&mut bytes))
        .is_err()
    {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(TAIL_LINES)..]
        .iter()
        .map(|l| l.chars().take(400).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_fails_and_times_out() {
        let dir = std::env::temp_dir();
        let ok = run_check("combined", "echo hello", &dir, Duration::from_secs(30));
        assert!(ok.passed, "{ok:?}");
        assert_eq!(ok.output_tail, ["hello"]);

        let fail = run_check("combined", "exit 3", &dir, Duration::from_secs(30));
        assert!(!fail.passed);
        assert_eq!(fail.exit_code, Some(3));

        #[cfg(windows)]
        let slow = "ping -n 30 127.0.0.1 > nul";
        #[cfg(not(windows))]
        let slow = "sleep 30";
        let timeout = run_check("combined", slow, &dir, Duration::from_millis(300));
        assert!(timeout.timed_out && !timeout.passed, "{timeout:?}");
    }

    /// Test runners on Windows print `√` as UTF-8; the receipt must keep it,
    /// not the cp1252 reading `âˆš`.
    #[test]
    fn keeps_utf8_check_output() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("out.txt"), "    √ should be equal\n").unwrap();
        #[cfg(windows)]
        let print = "type out.txt";
        #[cfg(not(windows))]
        let print = "cat out.txt";
        let check = run_check("combined", print, dir.path(), Duration::from_secs(30));
        assert_eq!(check.output_tail, ["    √ should be equal"]);
    }

    #[test]
    fn suggests_a_check_from_the_project_files() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        let suggest = || suggest_check(d);
        assert_eq!(suggest(), None);

        std::fs::write(d.join("pyproject.toml"), "").unwrap();
        assert_eq!(suggest().as_deref(), Some("python -m pytest"));
        std::fs::write(d.join("Cargo.toml"), "").unwrap();
        assert_eq!(suggest().as_deref(), Some("cargo test"));

        // npm's placeholder test script is not a test suite.
        let placeholder = r#"{"scripts":{"test":"echo \"Error: no test specified\" && exit 1"}}"#;
        std::fs::write(d.join("package.json"), placeholder).unwrap();
        assert_eq!(suggest().as_deref(), Some("cargo test"));

        std::fs::write(
            d.join("package.json"),
            r#"{"scripts":{"test":"tape test/*.js"}}"#,
        )
        .unwrap();
        assert_eq!(suggest().as_deref(), Some("npm install && npm test"));
        std::fs::write(d.join("package-lock.json"), "{}").unwrap();
        assert_eq!(suggest().as_deref(), Some("npm ci && npm test"));
        std::fs::write(d.join("pnpm-lock.yaml"), "").unwrap();
        assert_eq!(
            suggest().as_deref(),
            Some("pnpm install --frozen-lockfile && pnpm test")
        );

        assert!(installs("npm ci && npm test"));
        assert!(installs("pnpm install --frozen-lockfile && pnpm test"));
        assert!(installs("pnpm i && pnpm test"));
        assert!(!installs("npm test"));
        assert!(!installs("echo tape is not installed && exit 1"));
    }
}
