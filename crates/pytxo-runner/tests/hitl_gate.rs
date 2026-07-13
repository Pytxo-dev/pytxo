//! Galaxy HITL blocks risky spawn commands until approved.

use std::time::Duration;

use pytxo_core::PermissionProfile;
use pytxo_runner::{classify_risky_command, gate_spawn_command, HitlQueue};
use tempfile::TempDir;

#[test]
fn risky_command_classified() {
    assert!(classify_risky_command("rm -rf node_modules").is_some());
    assert!(classify_risky_command("echo ok").is_none());
}

#[test]
fn galaxy_spawn_gate_blocks_until_approved() {
    let hitl = HitlQueue::new();
    let agent_key = "run:agent-0";
    let cmd = "rm -rf build";

    let reviewer = {
        let hitl = hitl.clone();
        std::thread::spawn(move || loop {
            if let Some(req) = hitl.pending().into_iter().next() {
                assert_eq!(req.action, "fs.delete");
                hitl.resolve(&req.id, true);
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        })
    };

    gate_spawn_command(Some(&hitl), PermissionProfile::Galaxy, agent_key, cmd).unwrap();
    reviewer.join().unwrap();
}

#[test]
fn orbit_denies_network_spawn() {
    use pytxo_core::{PermissionEngine, PermissionProfile};
    let orbit = PermissionEngine::new(PermissionProfile::Orbit);
    assert!(!orbit.spawn_egress_allowed("curl https://example.com"));
    let galaxy = PermissionEngine::new(PermissionProfile::Galaxy);
    assert!(galaxy.spawn_egress_allowed("curl https://example.com"));
}

#[test]
fn hitl_persistence_round_trip() {
    let dir = TempDir::new().unwrap();
    let data = dir.path().join(".pytxo/data");
    std::fs::create_dir_all(&data).unwrap();
    let q1 = HitlQueue::with_persistence(&data);
    let id = q1.submit("a", "net.egress", "curl example.com");
    assert_eq!(q1.pending().len(), 1);

    let q2 = HitlQueue::with_persistence(&data);
    assert_eq!(q2.pending().len(), 1);
    assert_eq!(q2.pending()[0].id, id);
    assert!(q2.resolve(&id, true));
    assert!(q2.pending().is_empty());

    let q3 = HitlQueue::with_persistence(&data);
    assert!(q3.pending().is_empty());
}

#[test]
fn orbit_skips_spawn_gate() {
    let hitl = HitlQueue::new();
    gate_spawn_command(
        Some(&hitl),
        PermissionProfile::Orbit,
        "run:agent-0",
        "rm -rf x",
    )
    .unwrap();
    assert!(hitl.pending().is_empty());
}
