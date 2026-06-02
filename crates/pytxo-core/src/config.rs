use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{AgentSpec, PytxoError, Result, Task, TaskId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PytxoConfig {
    #[serde(default = "default_max_agents")]
    pub max_agents: usize,
    #[serde(default = "default_worktree_dir")]
    pub worktree_dir: PathBuf,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub fail_fast: bool,
    #[serde(default)]
    pub agent: Vec<AgentSpec>,
    #[serde(default)]
    pub task: Vec<TaskConfig>,
    #[serde(default = "default_true")]
    pub sanitize: bool,
    #[serde(default)]
    pub dag_explicit_deps: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskConfig {
    pub id: String,
    pub agent: String,
    pub paths: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

fn default_max_agents() -> usize {
    3
}

fn default_worktree_dir() -> PathBuf {
    PathBuf::from(".pytxo/worktrees")
}

fn default_data_dir() -> PathBuf {
    PathBuf::from(".pytxo/data")
}

impl Default for PytxoConfig {
    fn default() -> Self {
        Self {
            max_agents: default_max_agents(),
            worktree_dir: default_worktree_dir(),
            data_dir: default_data_dir(),
            fail_fast: true,
            agent: Vec::new(),
            task: Vec::new(),
            sanitize: true,
            dag_explicit_deps: false,
        }
    }
}

impl PytxoConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| PytxoError::Config(format!("read {}: {e}", path.display())))?;
        toml::from_str(&raw).map_err(|e| PytxoError::Config(format!("parse toml: {e}")))
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.task
            .iter()
            .map(|t| Task {
                id: TaskId(t.id.clone()),
                agent: t.agent.clone(),
                paths: t.paths.clone(),
                depends_on: t.depends_on.clone(),
            })
            .collect()
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("pytxo.db")
    }

    pub fn state_path(&self) -> PathBuf {
        self.data_dir.join("active_run.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_max_agents_is_three() {
        let cfg = PytxoConfig::default();
        assert_eq!(cfg.max_agents, 3);
    }
}
