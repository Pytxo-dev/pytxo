use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use pytxo_core::{
    conflict_error, paths_claim_overlap, LiveAgent, PytxoError, RaceShield, Result, StdinBuffer,
};

#[derive(Default)]
struct RegistryState {
    agents: HashMap<String, LiveAgent>,
    path_claims: HashMap<String, String>,
    stdin: StdinBuffer,
}

/// Race Shield runtime registry — path claims + stdin serialization.
#[derive(Clone, Default)]
pub struct SwarmRegistry {
    inner: Arc<RwLock<RegistryState>>,
}

impl SwarmRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list_live(&self) -> Vec<LiveAgent> {
        RaceShield::list_live(self)
    }
}

impl RaceShield for SwarmRegistry {
    fn try_claim_paths(&self, agent_key: &str, paths: &[String]) -> Result<()> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| PytxoError::Runner("race shield lock poisoned".into()))?;

        for path in paths {
            for (held_path, holder) in &guard.path_claims {
                if paths_claim_overlap(path, held_path) {
                    return Err(conflict_error(agent_key, path, holder));
                }
            }
        }

        for path in paths {
            guard.path_claims.insert(
                pytxo_core::normalize_claim_path(path),
                agent_key.to_string(),
            );
        }

        guard.agents.insert(
            agent_key.to_string(),
            LiveAgent {
                agent_key: agent_key.to_string(),
                paths: paths.to_vec(),
                pid: None,
            },
        );
        Ok(())
    }

    fn release(&self, agent_key: &str) {
        let Ok(mut guard) = self.inner.write() else {
            return;
        };
        if let Some(agent) = guard.agents.remove(agent_key) {
            for path in &agent.paths {
                let key = pytxo_core::normalize_claim_path(path);
                if guard.path_claims.get(&key).is_some_and(|h| h == agent_key) {
                    guard.path_claims.remove(&key);
                }
            }
        }
        guard.stdin.drain(agent_key);
    }

    fn register_pid(&self, agent_key: &str, pid: u32) {
        if let Ok(mut guard) = self.inner.write() {
            if let Some(agent) = guard.agents.get_mut(agent_key) {
                agent.pid = Some(pid);
            }
        }
    }

    fn enqueue_stdin(&self, agent_key: &str, data: &[u8]) -> Result<()> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| PytxoError::Runner("race shield lock poisoned".into()))?;
        // Allow pre-staging before path claim (subprocess spawn-time stdin drain).
        guard.stdin.enqueue(agent_key, data);
        Ok(())
    }

    fn list_live(&self) -> Vec<LiveAgent> {
        self.inner
            .read()
            .map(|g| g.agents.values().cloned().collect())
            .unwrap_or_default()
    }
}

impl SwarmRegistry {
    pub fn enqueue_stdin(&self, agent_key: &str, data: &[u8]) -> Result<()> {
        RaceShield::enqueue_stdin(self, agent_key, data)
    }

    pub fn drain_stdin(&self, agent_key: &str) -> Vec<u8> {
        self.inner
            .write()
            .map(|mut g| g.stdin.drain(agent_key))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_claims_rejected() {
        let reg = SwarmRegistry::new();
        reg.try_claim_paths("a:agent-0", &["src/**".into()])
            .unwrap();
        let err = reg
            .try_claim_paths("a:agent-1", &["src/foo.ts".into()])
            .unwrap_err();
        assert!(err.to_string().contains("race shield"));
    }

    #[test]
    fn release_frees_paths() {
        let reg = SwarmRegistry::new();
        reg.try_claim_paths("a:agent-0", &["src/a.ts".into()])
            .unwrap();
        reg.release("a:agent-0");
        reg.try_claim_paths("a:agent-1", &["src/a.ts".into()])
            .unwrap();
    }
}
