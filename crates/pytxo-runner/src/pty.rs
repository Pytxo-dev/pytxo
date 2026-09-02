use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, CommandBuilder, ExitStatus, PtySize};
use pytxo_core::{ChildLaunchEnv, PytxoError, Result};

use crate::race::SwarmRegistry;
use crate::run::{shell_command, EventCallback, SingleResult};

/// Smoke-test PTY spawn for `pytxo doctor`.
pub fn doctor_pty_smoke() -> Result<()> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 8,
            cols: 40,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| PytxoError::Runner(format!("doctor pty open: {e}")))?;

    let (shell, args) = shell_command();
    let mut builder = CommandBuilder::new(&shell);
    for arg in &args {
        builder.arg(arg);
    }
    builder.arg("echo pytxo-pty-ok");

    let mut child = pair
        .slave
        .spawn_command(builder)
        .map_err(|e| PytxoError::Runner(format!("doctor pty spawn: {e}")))?;
    // Master read EOF requires dropping our slave handle after spawn (portable-pty).
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| PytxoError::Runner(format!("doctor pty reader: {e}")))?;

    let (read_tx, read_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut acc = String::new();
        let _ = reader.read_to_string(&mut acc);
        let _ = read_tx.send(acc);
    });

    let status = child
        .wait()
        .map_err(|e| PytxoError::Runner(format!("doctor pty wait: {e}")))?;

    drop(pair.master);
    let buf = read_rx
        .recv_timeout(Duration::from_secs(10))
        .unwrap_or_default();

    if !status.success() {
        return Err(PytxoError::Runner(format!(
            "doctor pty child failed: {status:?}"
        )));
    }
    if cfg!(windows) {
        // ConPTY capture timing varies; exit success is sufficient on Windows.
        return Ok(());
    }
    if !buf.contains("pytxo-pty-ok") {
        return Err(PytxoError::Runner(format!(
            "doctor pty missing expected output (got {:?})",
            buf.chars().take(80).collect::<String>()
        )));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn run_pty_session(
    worktree: &Path,
    cmd: &str,
    env: ChildLaunchEnv,
    rows: u16,
    cols: u16,
    on_event: Option<&EventCallback>,
    agent_key: &str,
    swarm: &SwarmRegistry,
) -> Result<SingleResult> {
    run_pty_session_with_spawn(
        worktree, cmd, env, rows, cols, on_event, agent_key, swarm, None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_pty_session_with_spawn(
    worktree: &Path,
    cmd: &str,
    env: ChildLaunchEnv,
    rows: u16,
    cols: u16,
    on_event: Option<&EventCallback>,
    agent_key: &str,
    swarm: &SwarmRegistry,
    on_spawn: Option<&dyn Fn(u32) -> Result<()>>,
) -> Result<SingleResult> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| PytxoError::Runner(format!("pty open: {e}")))?;

    let (shell, args) = shell_command();
    let mut builder = CommandBuilder::new(&shell);
    for arg in &args {
        builder.arg(arg);
    }
    builder.arg(cmd);
    builder.cwd(worktree);
    builder.env_clear();
    for (key, value) in ChildLaunchEnv::inherited_baseline() {
        builder.env(key, value);
    }
    env.for_each(|k, v| {
        if k.eq_ignore_ascii_case("PYTXO_ULTRA_SESSION") {
            return;
        }
        builder.env(k, v);
    });

    let mut child = pair
        .slave
        .spawn_command(builder)
        .map_err(|e| PytxoError::Runner(format!("pty spawn: {e}")))?;
    drop(pair.slave);

    let pid = child.process_id();
    if let (Some(pid), Some(on_spawn)) = (pid, on_spawn) {
        if let Err(error) = on_spawn(pid) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    }
    let process_identity = match pid {
        Some(pid) => crate::kill::process_start_identity(pid)?,
        None => None,
    };

    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| PytxoError::Runner(format!("pty reader: {e}")))?;

    let mut writer = pair
        .master
        .take_writer()
        .map_err(|e| PytxoError::Runner(format!("pty writer: {e}")))?;

    let active = Arc::new(AtomicBool::new(true));
    let active_pump = Arc::clone(&active);
    let agent_key_pump = agent_key.to_string();
    let swarm_pump = swarm.clone();

    let pump_handle = thread::spawn(move || {
        while active_pump.load(Ordering::Relaxed) {
            let chunk = swarm_pump.drain_stdin(&agent_key_pump);
            if !chunk.is_empty() {
                let _ = writer.write_all(&chunk);
                let _ = writer.flush();
            }
            thread::sleep(Duration::from_millis(15));
        }
    });

    let on_event_read = on_event.cloned();
    let agent_key_read = agent_key.to_string();
    let active_read = Arc::clone(&active);
    let (read_tx, read_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut acc = String::new();
        let buf_reader = BufReader::new(reader);
        for line in buf_reader.lines().map_while(|l| l.ok()) {
            if let Some(cb) = &on_event_read {
                cb(&agent_key_read, "stdout", &line);
            }
            acc.push_str(&line);
            acc.push('\n');
        }
        active_read.store(false, Ordering::Relaxed);
        let _ = read_tx.send(acc);
    });

    // portable-pty's blocking wait can remain parked behind ConPTY teardown when
    // Stop terminates the process tree from another thread. Polling the process
    // handle observes the OS exit without coupling settlement to output EOF.
    let mut external_stop_started = None;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| PytxoError::Runner(format!("pty wait: {e}")))?
        {
            break status;
        }
        if swarm.stop_requested(agent_key) {
            let stopped_at = external_stop_started.get_or_insert_with(|| {
                let _ = child.kill();
                Instant::now()
            });
            if stopped_at.elapsed() >= Duration::from_secs(2) {
                if let Some(cb) = on_event {
                    cb(
                        agent_key,
                        "pty-exit-confirm-timeout",
                        "Stop completed but portable-pty did not report process completion within 2 seconds",
                    );
                }
                break ExitStatus::with_exit_code(1);
            }
        }
        if let (Some(pid), Some(identity)) = (pid, process_identity.as_deref()) {
            if !crate::kill::process_matches(pid, identity)? {
                let stopped_at = external_stop_started.get_or_insert_with(|| {
                    let _ = child.kill();
                    Instant::now()
                });
                if stopped_at.elapsed() >= Duration::from_secs(2) {
                    if let Some(cb) = on_event {
                        cb(
                            agent_key,
                            "pty-exit-confirm-timeout",
                            "OS process tree exited but portable-pty did not report completion within 2 seconds",
                        );
                    }
                    break ExitStatus::with_exit_code(1);
                }
            }
        }
        thread::sleep(Duration::from_millis(15));
    };

    active.store(false, Ordering::Relaxed);
    let _ = pump_handle.join();
    // Once the process and input pump are finished, close the MasterPty owner so
    // ConPTY signals EOF to the cloned reader instead of delaying settlement.
    drop(pair.master);
    let stdout_acc = match read_rx.recv_timeout(Duration::from_secs(2)) {
        Ok(output) => output,
        Err(_) => {
            if let Some(cb) = on_event {
                cb(
                    agent_key,
                    "pty-output-drain-timeout",
                    "PTY process exited but the output pipe did not close within 2 seconds",
                );
            }
            String::new()
        }
    };

    Ok(SingleResult {
        worktree_path: worktree.to_path_buf(),
        exit_code: Some(status.exit_code() as i32),
        stdout: stdout_acc,
        stderr: String::new(),
        pid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pty_does_not_inherit_parent_sentinel() {
        const SENTINEL: &str = "PYTXO_SENTINEL_PTY_SECRET";
        std::env::set_var(SENTINEL, "must-not-leak");
        let worktree = tempfile::tempdir().expect("temp worktree");
        let command = if cfg!(windows) {
            "if defined PYTXO_SENTINEL_PTY_SECRET (exit /b 9) else (exit /b 0)"
        } else {
            "test -z \"$PYTXO_SENTINEL_PTY_SECRET\""
        };
        let result = run_pty_session(
            worktree.path(),
            command,
            ChildLaunchEnv::new(),
            12,
            80,
            None,
            "sentinel",
            &SwarmRegistry::new(),
        )
        .expect("run pty sentinel probe");
        std::env::remove_var(SENTINEL);
        assert_eq!(
            result.exit_code,
            Some(0),
            "unrelated parent secret reached PTY child"
        );
    }
}
