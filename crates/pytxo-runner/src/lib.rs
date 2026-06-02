//! Git worktree lifecycle and agent process execution.

mod blast;
mod context;
mod git;
mod kill;
mod process;
mod process_registry_file;
mod race;
mod run;

pub use blast::{isolation_for_mode, OverlayIsolation, WorktreeIsolation};
pub use context::prepare_agent_context;
pub use git::{create_worktree, remove_worktree};
pub use kill::kill_pid;
pub use process::{ActiveRunHandle, ProcessRegistry};
pub use process_registry_file::{registry_path, ProcessEntry, ProcessRegistryFile};
pub use race::SwarmRegistry;
pub use run::{
    cleanup_worktrees, execute_plan, stop_all, stop_run, AgentRunResult, EventCallback, RunContext,
};
