use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
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

    let buf = read_rx
        .recv_timeout(Duration::from_secs(5))
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
    env.for_each(|k, v| {
        builder.env(k, v);
    });

    let mut child = pair
        .slave
        .spawn_command(builder)
        .map_err(|e| PytxoError::Runner(format!("pty spawn: {e}")))?;

    let pid = child.process_id();

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

    let status = child
        .wait()
        .map_err(|e| PytxoError::Runner(format!("pty wait: {e}")))?;

    active.store(false, Ordering::Relaxed);
    let _ = pump_handle.join();
    let stdout_acc = read_rx
        .recv_timeout(Duration::from_secs(120))
        .unwrap_or_default();

    Ok(SingleResult {
        worktree_path: worktree.to_path_buf(),
        exit_code: Some(status.exit_code() as i32),
        stdout: stdout_acc,
        stderr: String::new(),
        pid,
    })
}
