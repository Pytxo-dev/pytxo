use std::thread;
use std::time::Duration;

use pytxo_core::{ChildLaunchEnv, RaceShield};
use pytxo_runner::{run_pty_session, SwarmRegistry};

#[test]
fn pty_echo_smoke() {
    let dir = tempfile::tempdir().unwrap();
    let swarm = SwarmRegistry::new();
    let agent_key = "run:agent-0";
    swarm.try_claim_paths(agent_key, &[".".into()]).unwrap();

    let result = run_pty_session(
        dir.path(),
        "echo pytxo-pty",
        ChildLaunchEnv::new(),
        24,
        80,
        None,
        agent_key,
        &swarm,
    )
    .unwrap();

    assert_eq!(result.exit_code, Some(0));
    assert!(result.stdout.contains("pytxo-pty"));
    swarm.release(agent_key);
}

#[test]
fn pty_stdin_round_trip() {
    if cfg!(windows) {
        // ConPTY stdin timing varies; covered on Unix CI.
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let swarm = SwarmRegistry::new();
    let agent_key = "run:agent-0";
    swarm.try_claim_paths(agent_key, &[".".into()]).unwrap();

    let swarm_enqueue = swarm.clone();
    let key = agent_key.to_string();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(80));
        let _ = swarm_enqueue.enqueue_stdin(&key, b"hello\n");
    });

    let cmd = "sh -c 'IFS= read -r line; printf got:%s' \"$line\"'";
    let result = run_pty_session(
        dir.path(),
        cmd,
        ChildLaunchEnv::new(),
        24,
        80,
        None,
        agent_key,
        &swarm,
    )
    .unwrap();

    assert_eq!(result.exit_code, Some(0));
    assert!(
        result.stdout.contains("got:hello"),
        "stdout was {:?}",
        result.stdout
    );
    swarm.release(agent_key);
}

#[test]
fn doctor_pty_smoke_ok() {
    pytxo_runner::doctor_pty_smoke().expect("doctor pty smoke");
}
