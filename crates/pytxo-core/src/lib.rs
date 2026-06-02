//! Shared domain types and configuration for Pytxo.

mod config;
mod error;
mod ids;
mod path_util;
mod plan;
mod task;

pub use config::PytxoConfig;
pub use error::{PytxoError, Result};
pub use ids::{AgentId, RunId, TaskId};
pub use path_util::{canonical_repo_root, strip_extended_path};
pub use plan::{AgentAssignment, ConflictPair, ExecutionPlan, ScheduledTask};
pub use task::{AgentSpec, Task};

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let id = super::RunId::new();
        assert!(!id.0.is_empty());
    }
}
