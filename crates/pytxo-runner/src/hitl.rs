//! Galaxy human-in-the-loop (HITL) approval queue ([[race-shield]], [[permission-profile-engine]]).
//!
//! High-risk actions under a `Galaxy` permission profile submit a request and block
//! until orchestration resolves it (approve/deny) over IPC. The queue is per
//! execution domain, mirroring Race Shield's per-domain scope.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A pending or resolved approval request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HitlRequest {
    pub id: String,
    pub agent_key: String,
    pub action: String,
    pub reason: String,
    /// Milliseconds since the Unix epoch.
    pub created_at_ms: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HitlDecision {
    Pending,
    Approved,
    Denied,
}

#[derive(Default, Serialize, Deserialize)]
struct HitlPersistFile {
    pending: Vec<HitlRequest>,
    decisions: HashMap<String, HitlDecision>,
}

#[derive(Default)]
struct HitlState {
    pending: HashMap<String, HitlRequest>,
    decisions: HashMap<String, HitlDecision>,
}

type WalAudit = Arc<dyn Fn(&str, bool) + Send + Sync>;

/// Thread-safe approval queue shared between the runner and orchestration/IPC.
#[derive(Clone)]
pub struct HitlQueue {
    inner: Arc<Mutex<HitlState>>,
    persist_path: Option<PathBuf>,
    wal_audit: Option<WalAudit>,
}

static HITL_SEQ: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

impl Default for HitlQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl HitlQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HitlState::default())),
            persist_path: None,
            wal_audit: None,
        }
    }

    /// Queue with JSON persistence under `{data_dir}/hitl.json` (Phase 24).
    pub fn with_persistence(data_dir: &Path) -> Self {
        let path = data_dir.join("hitl.json");
        let mut state = HitlState::default();
        if path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&path) {
                if let Ok(file) = serde_json::from_str::<HitlPersistFile>(&raw) {
                    for req in file.pending {
                        state.pending.insert(req.id.clone(), req);
                    }
                    state.decisions = file.decisions;
                }
            }
        }
        Self {
            inner: Arc::new(Mutex::new(state)),
            persist_path: Some(path),
            wal_audit: None,
        }
    }

    /// Append a `hitl-resolve` WAL row when `resolve()` succeeds.
    pub fn with_wal_audit(mut self, audit: WalAudit) -> Self {
        self.wal_audit = Some(audit);
        self
    }

    fn persist(&self) {
        let Some(path) = self.persist_path.as_ref() else {
            return;
        };
        let Ok(guard) = self.inner.lock() else {
            return;
        };
        let file = HitlPersistFile {
            pending: guard.pending.values().cloned().collect(),
            decisions: guard.decisions.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&file) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn persist_path(&self) -> Option<&Path> {
        self.persist_path.as_deref()
    }

    /// Submit a request for human approval; returns its id.
    pub fn submit(&self, agent_key: &str, action: &str, reason: &str) -> String {
        let seq = HITL_SEQ.fetch_add(1, Ordering::Relaxed);
        let id = format!("hitl-{seq}");
        let req = HitlRequest {
            id: id.clone(),
            agent_key: agent_key.to_string(),
            action: action.to_string(),
            reason: reason.to_string(),
            created_at_ms: now_ms(),
        };
        if let Ok(mut g) = self.inner.lock() {
            g.pending.insert(id.clone(), req);
            g.decisions.insert(id.clone(), HitlDecision::Pending);
        }
        self.persist();
        id
    }

    /// Requests still awaiting a human decision.
    pub fn pending(&self) -> Vec<HitlRequest> {
        self.inner
            .lock()
            .map(|g| g.pending.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Resolve a request. Returns `true` if the id was pending.
    pub fn resolve(&self, id: &str, approved: bool) -> bool {
        let resolved = {
            let Ok(mut g) = self.inner.lock() else {
                return false;
            };
            if g.pending.remove(id).is_some() {
                g.decisions.insert(
                    id.to_string(),
                    if approved {
                        HitlDecision::Approved
                    } else {
                        HitlDecision::Denied
                    },
                );
                true
            } else {
                false
            }
        };
        if resolved {
            if let Some(audit) = &self.wal_audit {
                audit(id, approved);
            }
            self.persist();
        }
        resolved
    }

    /// Current decision for a request (Pending if unknown or unresolved).
    pub fn decision(&self, id: &str) -> HitlDecision {
        self.inner
            .lock()
            .ok()
            .and_then(|g| g.decisions.get(id).copied())
            .unwrap_or(HitlDecision::Pending)
    }

    /// Block until resolved or `timeout` elapses. On timeout returns `Pending`.
    pub fn wait_blocking(&self, id: &str, timeout: Duration) -> HitlDecision {
        let start = Instant::now();
        loop {
            let d = self.decision(id);
            if d != HitlDecision::Pending {
                return d;
            }
            if start.elapsed() >= timeout {
                return HitlDecision::Pending;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_lists_pending_then_resolve_clears() {
        let q = HitlQueue::new();
        let id = q.submit("run:agent-0", "fs.delete", "rm -rf build");
        assert_eq!(q.pending().len(), 1);
        assert_eq!(q.decision(&id), HitlDecision::Pending);
        assert!(q.resolve(&id, true));
        assert_eq!(q.decision(&id), HitlDecision::Approved);
        assert!(q.pending().is_empty());
        // resolving an unknown id is a no-op
        assert!(!q.resolve("missing", true));
    }

    #[test]
    fn wait_returns_after_resolution() {
        let q = HitlQueue::new();
        let id = q.submit("run:agent-1", "net.egress", "curl evil.test");
        let q2 = q.clone();
        let id2 = id.clone();
        let h = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            q2.resolve(&id2, false);
        });
        let decision = q.wait_blocking(&id, Duration::from_secs(2));
        h.join().unwrap();
        assert_eq!(decision, HitlDecision::Denied);
    }
}
