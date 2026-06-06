use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{PytxoError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathConflict {
    pub agent_key: String,
    pub path: String,
    pub held_by: String,
}

#[derive(Clone, Debug)]
pub struct LiveAgent {
    pub agent_key: String,
    pub paths: Vec<String>,
    pub pid: Option<u32>,
}

/// Race Shield contract: runtime path claims and serialized stdin per agent.
pub trait RaceShield: Send + Sync {
    fn try_claim_paths(&self, agent_key: &str, paths: &[String]) -> Result<()>;

    fn release(&self, agent_key: &str);

    fn register_pid(&self, agent_key: &str, pid: u32);

    fn enqueue_stdin(&self, agent_key: &str, data: &[u8]) -> Result<()>;

    fn list_live(&self) -> Vec<LiveAgent>;
}

/// Normalize a path pattern for claim comparison (matches scheduler overlap rules).
pub fn normalize_claim_path(p: &str) -> String {
    p.replace('\\', "/").trim_start_matches("./").to_string()
}

/// Namespace a claim path by its modular-project root label so claims in
/// different roots never overlap ([[ADR-0011-modular-project-manifest]]).
/// The unit-separator prefix is preserved by [`normalize_claim_path`] and
/// [`paths_claim_overlap`], keeping within-root prefix semantics intact.
pub fn root_scoped_claim(root: Option<&str>, path: &str) -> String {
    match root {
        Some(label) if !label.is_empty() => format!("{label}\u{1f}{path}"),
        _ => path.to_string(),
    }
}

pub fn paths_claim_overlap(a: &str, b: &str) -> bool {
    let a = normalize_claim_path(a);
    let b = normalize_claim_path(b);
    if a == b {
        return true;
    }
    if a.starts_with(&b) || b.starts_with(&a) {
        return true;
    }
    let strip_glob = |p: &str| {
        p.trim_end_matches("/**")
            .trim_end_matches("/*")
            .trim_end_matches('*')
            .to_string()
    };
    let a_base = strip_glob(&a);
    let b_base = strip_glob(&b);
    a_base == b_base || a_base.starts_with(&b_base) || b_base.starts_with(&a_base)
}

pub fn conflict_error(agent_key: &str, path: &str, held_by: &str) -> PytxoError {
    PytxoError::Runner(format!(
        "race shield: agent {agent_key} cannot claim {path} (held by {held_by})"
    ))
}

/// In-memory stdin queue keyed by agent.
#[derive(Default)]
pub struct StdinBuffer {
    queues: HashMap<String, Vec<u8>>,
}

impl StdinBuffer {
    pub fn enqueue(&mut self, agent_key: &str, data: &[u8]) {
        self.queues
            .entry(agent_key.to_string())
            .or_default()
            .extend_from_slice(data);
    }

    pub fn drain(&mut self, agent_key: &str) -> Vec<u8> {
        self.queues.remove(agent_key).unwrap_or_default()
    }

    pub fn pending_len(&self, agent_key: &str) -> usize {
        self.queues.get(agent_key).map(|q| q.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_scoped_claims_do_not_overlap_across_roots() {
        let api = root_scoped_claim(Some("api"), "README.md");
        let web = root_scoped_claim(Some("web"), "README.md");
        assert_ne!(api, web);
        assert!(!paths_claim_overlap(&api, &web));
    }

    #[test]
    fn root_scoped_claim_preserves_within_root_overlap() {
        let a = root_scoped_claim(Some("api"), "src/**");
        let b = root_scoped_claim(Some("api"), "src/lib.rs");
        assert!(paths_claim_overlap(&a, &b));
    }
}
