use pytxo_core::{DomainId, IsolationMode, PermissionProfile};
use pytxo_runner::permission_enforcement_receipt_for_mechanism;

#[test]
fn deepspace_fails_closed_when_socket_isolation_is_only_a_stub() {
    let error = permission_enforcement_receipt_for_mechanism(
        PermissionProfile::DeepSpace,
        PermissionProfile::DeepSpace,
        &DomainId("C:/repo".into()),
        IsolationMode::Worktree,
        &[],
        "windows-wfp-stub",
    )
    .expect_err("DeepSpace must not run with policy-only socket isolation");

    assert!(
        error.to_string().contains("DeepSpace") && error.to_string().contains("windows-wfp-stub"),
        "unexpected error: {error}"
    );
}

#[test]
fn orbit_receipt_is_honest_about_advisory_network_gating() {
    let receipt = permission_enforcement_receipt_for_mechanism(
        PermissionProfile::Orbit,
        PermissionProfile::Orbit,
        &DomainId("C:/repo".into()),
        IsolationMode::Overlay,
        &["target".into()],
        "windows-wfp-stub",
    )
    .expect("Orbit can use its documented policy gate");

    assert_eq!(receipt.effective_profile, "orbit");
    assert_eq!(receipt.network.status, "advisory");
    assert_eq!(receipt.apply_boundary.status, "enforced");
    assert!(
        receipt
            .workspace_isolation
            .mechanism
            .starts_with("overlay-")
            || receipt.workspace_isolation.mechanism.starts_with("projfs-")
    );
}
