//! Git worktree lifecycle and agent process execution.

mod arbitrage;
mod blast;
mod candidate_verification;
mod change_set;
mod context;
mod enforcement;
mod failure;
mod git;
mod hitl;
mod hitl_gate;
mod kill;
mod mcp_hub;
mod network_isolation;
#[cfg(all(feature = "overlay-fuse-kernel", target_os = "linux"))]
mod overlay_fuse_linux;
#[cfg(all(feature = "overlay-fuse-macos", target_os = "macos"))]
mod overlay_fuse_macos;
#[cfg(target_os = "windows")]
mod overlay_projfs;
pub mod owned_launch;
mod process;
mod process_registry_file;
mod pty;
mod race;
mod run;

pub use arbitrage::ArbitrageProfiler;
pub use blast::{
    effective_isolation_mode, isolation_backend_label, isolation_for_mode, OverlayIsolation,
    WorktreeIsolation,
};
pub use candidate_verification::{
    refresh_frozen_review_package, require_candidate_verification, CandidateVerification,
};
pub use change_set::{
    apply_attempt_ids, apply_prepared_review, apply_prepared_review_under_lease,
    apply_prepared_review_with_fault, apply_prepared_review_with_fault_under_lease,
    load_review_manifest, load_review_package, prepare_review_package, prepare_run_change_set,
    read_review_content_chunk, read_review_content_chunk_with_fault,
    read_review_content_chunk_with_metrics, reconcile_apply_journals,
    reconcile_apply_journals_under_lease, AgentWorkspaceInput, AppliedRunChange, ApplyFaultPoint,
    ExecutionDomainMutationLease, PreparedRunChangeSet, RecoveryOutcome, ReviewContentChunk,
    ReviewContentSide, ReviewReadFaultPoint, ReviewReadMetrics, RunApplyManifest, RunChange,
    RunChangeKind,
};
pub use context::{prepare_agent_context, prepare_agent_context_for_root, ContextBundle};
pub use enforcement::{
    permission_enforcement_receipt, permission_enforcement_receipt_for_mechanism,
    verification_enforcement_receipt, EnforcementSurfaceReceipt, PermissionEnforcementReceipt,
    VerificationEnforcementReceipt,
};
pub use failure::implicated_paths;
pub use git::{
    branch_name, capture_reviewed_inputs, capture_sealed_output,
    capture_sealed_output_after_dependency, create_plain_verification_view, create_worktree,
    materialize_dependency_output, materialize_reviewed_inputs, materialize_sealed_output,
    merge_agent_branch, prepare_reviewed_worktree, read_reviewed_claim_text,
    remove_reviewed_worktree_if_owned, remove_routed_verification_views, remove_worktree,
    require_exact_reviewed_checkout, seal_one_existing_claimed_text_proposal,
    verify_materialized_dependency_output, verify_materialized_reviewed_inputs,
    verify_sealed_output_view, ReviewedInputFile, ReviewedInputManifest, SealedOutputFile,
    SealedOutputSnapshot,
};
pub use hitl::{HitlDecision, HitlQueue, HitlRequest};
pub use hitl_gate::{
    classify_mcp_proxy, classify_risky_command, gate_hitl_action, gate_mcp_proxy,
    gate_spawn_command, workspace_writes_outside_root,
};
pub use kill::{
    file_identity, kill_pid, process_matches, process_start_identity, FileIdentityGuard,
};
pub use mcp_hub::{spawn_test_mcp_child, ChildMcpSession, McpHub};
pub use network_isolation::{
    isolate_deepspace_network, isolation_mechanism, wrap_deepspace_shell_cmd,
};
pub use process::{ActiveRunHandle, ProcessRegistry};
pub use process_registry_file::{registry_path, ProcessEntry, ProcessRegistryFile};
pub use pty::{doctor_pty_smoke, run_pty_session};

/// Doctor probe for DeepSpace network isolation (Phase 45, ADR-0026).
pub fn doctor_network_isolation_probe() -> String {
    use pytxo_core::{NetworkPolicy, NetworkPolicyEngine, PermissionProfile};
    let deepspace = NetworkPolicyEngine::new(PermissionProfile::DeepSpace);
    let tcp_blocked = !deepspace.egress_allowed("1.1.1.1", 443);
    let (socket_blocked, socket_detail) = crate::network_isolation::doctor_deepspace_socket_probe();
    format!(
        "policy_tcp_blocked={tcp_blocked}; {socket_detail}; socket_probe_blocked={socket_blocked}"
    )
}

/// Whether the doctor socket probe observed blocked egress (ADR-0026).
pub fn doctor_deepspace_socket_blocked() -> bool {
    crate::network_isolation::doctor_deepspace_socket_probe().0
}

/// Doctor probe for Blast overlay isolation (Phase 26/69).
///
/// Always succeeds when a sparse copy-layer can materialize (git or non-git).
/// Kernel FUSE / ProjFS labels are preferred when those backends are present.
pub fn doctor_overlay_probe() -> pytxo_core::Result<String> {
    #[cfg(all(feature = "overlay-fuse-kernel", target_os = "linux"))]
    {
        if crate::overlay_fuse_linux::fuse_available() {
            return Ok("overlay-fuse-kernel mount available".into());
        }
    }
    #[cfg(all(feature = "overlay-fuse-macos", target_os = "macos"))]
    {
        if crate::overlay_fuse_macos::fuse_available() {
            return Ok("overlay-fuse-macos mount available".into());
        }
    }
    #[cfg(target_os = "windows")]
    {
        if crate::overlay_projfs::projfs_supported() {
            return Ok("overlay-projfs sparse-copy-v2 available".into());
        }
    }
    // Phase 69: copy-layer is always available — prefer_kernel_overlay can upgrade Orbit.
    Ok("overlay-copy-layer available".into())
}
pub use race::SwarmRegistry;
pub use run::{
    cleanup_worktrees, commit_workspace, execute_plan, publish_run_cancellation,
    run_candidate_check, stop_all, stop_run, terminate_published_run, AgentRunOutcome,
    AgentRunResult, CandidateCheckContext, EventCallback, PublishedRunCancellation, RootExec,
    RunContext,
};
