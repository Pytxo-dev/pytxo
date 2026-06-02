//! Git worktree lifecycle and agent process execution.

mod git;
mod kill;
mod process;
mod process_registry_file;
mod run;

pub use git::{create_worktree, remove_worktree};
pub use kill::kill_pid;
pub use process::{ActiveRunHandle, ProcessRegistry};
pub use process_registry_file::{registry_path, ProcessEntry, ProcessRegistryFile};
pub use run::{
    cleanup_worktrees, execute_plan, stop_all, stop_run, AgentRunResult, EventCallback, RunContext,
};
