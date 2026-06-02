use std::fs;
use std::path::Path;

use pytxo_core::{PytxoError, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProcessRegistryFile {
    pub entries: Vec<ProcessEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub run_id: String,
    pub repo_root: String,
    pub agent_key: String,
    pub pid: u32,
    pub worktree_path: String,
    pub branch: String,
}

impl ProcessRegistryFile {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path).map_err(PytxoError::Io)?;
        serde_json::from_str(&raw).map_err(|e| PytxoError::Runner(format!("process registry: {e}")))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let raw = serde_json::to_string_pretty(self)
            .map_err(|e| PytxoError::Runner(format!("serialize registry: {e}")))?;
        fs::write(path, raw).map_err(PytxoError::Io)?;
        Ok(())
    }

    pub fn push(&mut self, entry: ProcessEntry) {
        self.entries.retain(|e| e.agent_key != entry.agent_key);
        self.entries.push(entry);
    }

    pub fn for_run(&self, run_id: &str) -> Vec<&ProcessEntry> {
        self.entries.iter().filter(|e| e.run_id == run_id).collect()
    }

    pub fn all_pids(&self) -> Vec<u32> {
        self.entries.iter().map(|e| e.pid).collect()
    }

    pub fn remove_run(&mut self, run_id: &str) {
        self.entries.retain(|e| e.run_id != run_id);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

pub fn registry_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("process_registry.json")
}
