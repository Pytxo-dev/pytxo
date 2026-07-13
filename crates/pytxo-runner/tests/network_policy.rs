//! Network policy and DeepSpace isolation contract tests (Phase 45).

use pytxo_core::{NetworkPolicy, NetworkPolicyEngine, PermissionEngine, PermissionProfile};
use pytxo_runner::{isolate_deepspace_network, isolation_mechanism, wrap_deepspace_shell_cmd};

#[test]
fn deepspace_blocks_tcp_egress_policy() {
    let net = NetworkPolicyEngine::new(PermissionProfile::DeepSpace);
    assert!(!net.egress_allowed("1.1.1.1", 443));
    assert!(!net.spawn_egress_allowed("curl https://example.com"));
}

#[test]
fn orbit_denies_spawn_egress_but_allows_non_network_cmds() {
    let orbit = PermissionEngine::new(PermissionProfile::Orbit);
    assert!(!orbit.spawn_egress_allowed("curl https://example.com"));
    assert!(orbit.spawn_egress_allowed("cargo test"));
}

#[test]
fn galaxy_allows_fetch_spawn_but_ports_are_scoped() {
    let net = NetworkPolicyEngine::new(PermissionProfile::Galaxy);
    assert!(net.spawn_egress_allowed("curl https://example.com"));
    assert!(!net.egress_allowed("1.1.1.1", 443));
    assert!(net.egress_allowed("localhost", 5432));
}

#[test]
fn deepspace_isolation_hook_sets_platform_marker() {
    let mut cmd = std::process::Command::new("echo");
    cmd.arg("probe");
    isolate_deepspace_network(&mut cmd).unwrap();
    assert!(!isolation_mechanism().is_empty());
    let _ = wrap_deepspace_shell_cmd("echo probe");
    let _blocked = pytxo_runner::doctor_deepspace_socket_blocked();
}
