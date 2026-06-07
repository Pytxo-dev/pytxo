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

    // Pre-stage stdin (same as subprocess path); pump drains within one poll tick.
    swarm.enqueue_stdin(agent_key, b"hello\n").unwrap();

    let cmd = r#"IFS= read -r line && printf 'got:%s' "$line""#;
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
