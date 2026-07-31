use serde::{Deserialize, Serialize};

use crate::{AgentId, TaskId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub task_id: TaskId,
    pub agent: String,
    pub paths: Vec<String>,
    /// Explicit upstream task ids whose isolated outputs must be composed into
    /// this task's workspace before its agent starts.
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub wave: u32,
    /// Modular project root label ([[ADR-0011-modular-project-manifest]]); `None` = primary root.
    #[serde(default)]
    pub root: Option<String>,
    /// Per-task Signal Core fidelity override ([[closed-loop-fidelity]]).
    #[serde(default)]
    pub signal_fidelity: Option<crate::moat::FidelityTier>,
    /// Post-task verify shell commands.
    #[serde(default)]
    pub verify: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentAssignment {
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub wave: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictPair {
    pub task_a: TaskId,
    pub task_b: TaskId,
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub waves: Vec<Vec<ScheduledTask>>,
    pub conflicts: Vec<ConflictPair>,
    pub max_agents: usize,
    /// Scheduler dry-run hints (e.g. cross-root path overlap).
    #[serde(default)]
    pub warnings: Vec<String>,
}
