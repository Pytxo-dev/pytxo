//! Git worktree lifecycle and agent process execution.

mod arbitrage;
mod blast;
mod context;
mod failure;
mod git;
mod hitl;
mod kill;
mod mcp_hub;
#[cfg(all(feature = "overlay-fuse-kernel", target_os = "linux"))]
mod overlay_fuse_linux;
#[cfg(all(feature = "overlay-projfs", target_os = "windows"))]
mod overlay_projfs;
mod process;
mod process_registry_file;
mod pty;
mod race;
mod run;

pub use arbitrage::ArbitrageProfiler;
pub use blast::{isolation_for_mode, OverlayIsolation, WorktreeIsolation};
pub use context::{prepare_agent_context, prepare_agent_context_for_root, ContextBundle};
pub use failure::implicated_paths;
pub use git::{branch_name, create_worktree, merge_agent_branch, remove_worktree};
pub use hitl::{HitlDecision, HitlQueue, HitlRequest};
pub use kill::kill_pid;
pub use mcp_hub::{spawn_test_mcp_child, ChildMcpSession, McpHub};
pub use process::{ActiveRunHandle, ProcessRegistry};
pub use process_registry_file::{registry_path, ProcessEntry, ProcessRegistryFile};
pub use pty::{doctor_pty_smoke, run_pty_session};
pub use race::SwarmRegistry;
pub use run::{
    cleanup_worktrees, commit_workspace, execute_plan, stop_all, stop_run, AgentRunResult,
    EventCallback, RootExec, RunContext,
};
