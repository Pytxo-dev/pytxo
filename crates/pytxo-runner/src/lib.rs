//! Git worktree lifecycle and agent process execution.

mod arbitrage;
mod blast;
mod context;
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
#[cfg(all(feature = "overlay-projfs", target_os = "windows"))]
mod overlay_projfs;
mod process;
mod process_registry_file;
mod pty;
mod race;
mod run;

pub use arbitrage::ArbitrageProfiler;
pub use blast::{
    isolation_backend_label, isolation_for_mode, OverlayIsolation, WorktreeIsolation,
};
pub use context::{prepare_agent_context, prepare_agent_context_for_root, ContextBundle};
pub use failure::implicated_paths;
pub use git::{branch_name, create_worktree, merge_agent_branch, remove_worktree};
pub use hitl::{HitlDecision, HitlQueue, HitlRequest};
pub use hitl_gate::{
    classify_mcp_proxy, classify_risky_command, gate_hitl_action, gate_mcp_proxy,
    gate_spawn_command, workspace_writes_outside_root,
};
pub use kill::kill_pid;
pub use mcp_hub::{spawn_test_mcp_child, ChildMcpSession, McpHub};
pub use network_isolation::{
    isolate_deepspace_network, isolation_mechanism, wrap_deepspace_shell_cmd,
};
pub use process::{ActiveRunHandle, ProcessRegistry};
pub use process_registry_file::{registry_path, ProcessEntry, ProcessRegistryFile};
pub use pty::{doctor_pty_smoke, run_pty_session};

/// Doctor probe for DeepSpace network isolation (Phase 45).
pub fn doctor_network_isolation_probe() -> String {
    use pytxo_core::{NetworkPolicy, NetworkPolicyEngine, PermissionProfile};
    let deepspace = NetworkPolicyEngine::new(PermissionProfile::DeepSpace);
    let tcp_blocked = !deepspace.egress_allowed("1.1.1.1", 443);
    format!(
        "mechanism={}; tcp_probe_expect_blocked={tcp_blocked}",
        isolation_mechanism()
    )
}

/// Doctor probe for Blast overlay isolation (Phase 26).
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
    #[cfg(all(feature = "overlay-projfs", target_os = "windows"))]
    {
        if crate::overlay_projfs::projfs_supported() {
            return Ok("overlay-projfs copy-layer available".into());
        }
    }
    #[cfg(feature = "overlay-fuse")]
    {
        return Ok("overlay-fuse copy-layer flush available".into());
    }
    #[cfg(not(feature = "overlay-fuse"))]
    Err(pytxo_core::PytxoError::Runner(
        "overlay-fuse not enabled; Orbit uses worktree fallback".into(),
    ))
}
pub use race::SwarmRegistry;
pub use run::{
    cleanup_worktrees, commit_workspace, execute_plan, stop_all, stop_run, AgentRunResult,
    EventCallback, RootExec, RunContext,
};
