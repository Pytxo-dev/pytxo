use std::path::PathBuf;

use crate::{AgentId, PytxoError, Result, RunId};

/// Isolation backend selection ([[ADR-0005-worktree-isolation-for-mvp]], [[sparse-overlay-fs]]).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IsolationMode {
    #[default]
    Worktree,
    Overlay,
}

impl IsolationMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "worktree" => Some(Self::Worktree),
            "overlay" => Some(Self::Overlay),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Worktree => "worktree",
            Self::Overlay => "overlay",
        }
    }
}

#[derive(Clone, Debug)]
pub struct IsolationCtx {
    pub run_id: RunId,
    pub agent_id: AgentId,
    pub repo_root: PathBuf,
    pub worktree_base: PathBuf,
    /// Top-level names skipped in overlay sparse copy / lowerdir (from `[blast].sparse_exclude`).
    pub sparse_exclude: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct WorkspaceHandle {
    pub cwd: PathBuf,
    pub branch: String,
    pub backend: IsolationMode,
}

/// Blast Shield contract: prepare an isolated execution bubble, rollback or flush on demand.
pub trait IsolationBackend: Send + Sync {
    fn mode(&self) -> IsolationMode;

    fn prepare(&self, ctx: &IsolationCtx) -> Result<WorkspaceHandle>;

    fn rollback(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()>;

    /// Persist approved agent mutations to physical disk.
    fn flush(&self, ctx: &IsolationCtx, handle: &WorkspaceHandle) -> Result<()> {
        let _ = (ctx, handle);
        Err(PytxoError::Runner(
            "flush not supported for this isolation backend".into(),
        ))
    }
}
