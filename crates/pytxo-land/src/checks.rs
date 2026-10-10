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
}
