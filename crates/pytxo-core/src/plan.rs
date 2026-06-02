use serde::{Deserialize, Serialize};

use crate::{AgentId, TaskId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub task_id: TaskId,
    pub agent: String,
    pub paths: Vec<String>,
    pub wave: u32,
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
}
