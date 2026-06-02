use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use pytxo_core::{AgentId, RunId};

#[derive(Clone, Debug)]
pub struct ChildRecord {
    pub run_id: RunId,
    pub agent_id: AgentId,
    pub worktree_path: PathBuf,
    pub branch: String,
    pub pid: Option<u32>,
}

#[derive(Default, Clone)]
pub struct ProcessRegistry {
    inner: Arc<Mutex<RegistryInner>>,
}

#[derive(Default)]
struct RegistryInner {
    children: HashMap<String, ChildRecord>,
}

impl ProcessRegistry {
    pub fn register(&self, record: ChildRecord) {
        let key = format!("{}:{}", record.run_id, record.agent_id);
        self.inner.lock().expect("registry lock").children.insert(key, record);
    }

    pub fn list(&self) -> Vec<ChildRecord> {
        self.inner
            .lock()
            .expect("registry lock")
            .children
            .values()
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.inner.lock().expect("registry lock").children.clear();
    }
}

#[derive(Clone)]
pub struct ActiveRunHandle {
    pub run_id: RunId,
    pub registry: ProcessRegistry,
}
