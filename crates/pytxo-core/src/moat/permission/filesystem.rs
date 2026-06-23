//! Filesystem policy trait ([[permission-profile-engine]], Phase 30).

use std::path::Path;

use super::PermissionProfile;

/// Gates read/write/flush before Blast Shield or MCP materialization.
pub trait FilesystemPolicy {
    fn profile(&self) -> PermissionProfile;

    /// Whether a path under `repo_root` may be read for agent context.
    fn may_read(&self, repo_root: &Path, path: &Path, agent_cwd: &Path) -> bool;

    /// Whether physical flush to disk is allowed for this profile.
    fn may_flush(&self) -> bool;

    fn flush_requires_approval(&self) -> bool;

    fn use_worktree_isolation(&self) -> bool;
}

/// Resolve `path` relative to `repo_root` and test DeepSpace cwd-only reads.
pub fn path_under_cwd(agent_cwd: &Path, path: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let cwd = canon(agent_cwd);
    let target = if path.is_absolute() {
        canon(path)
    } else {
        canon(&agent_cwd.join(path))
    };
    target.starts_with(&cwd)
}

pub struct FilesystemPolicyEngine {
    profile: PermissionProfile,
}

impl FilesystemPolicyEngine {
    pub fn new(profile: PermissionProfile) -> Self {
        Self { profile }
    }
}

impl FilesystemPolicy for FilesystemPolicyEngine {
    fn profile(&self) -> PermissionProfile {
        self.profile
    }

    fn may_read(&self, repo_root: &Path, path: &Path, agent_cwd: &Path) -> bool {
        match self.profile {
            PermissionProfile::DeepSpace => path_under_cwd(agent_cwd, path),
            _ => {
                let _ = repo_root;
                true
            }
        }
    }

    fn may_flush(&self) -> bool {
        !matches!(self.profile, PermissionProfile::DeepSpace)
    }

    fn flush_requires_approval(&self) -> bool {
        matches!(
            self.profile,
            PermissionProfile::DeepSpace | PermissionProfile::Orbit | PermissionProfile::Galaxy
        )
    }

    fn use_worktree_isolation(&self) -> bool {
        !matches!(self.profile, PermissionProfile::Supernova)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn deep_space_denies_read_outside_cwd() {
        let engine = FilesystemPolicyEngine::new(PermissionProfile::DeepSpace);
        let cwd = PathBuf::from("/tmp/agent");
        let inside = PathBuf::from("/tmp/agent/src/lib.rs");
        let outside = PathBuf::from("/tmp/other/lib.rs");
        // Without real dirs canonicalize falls back to prefix logic on joined paths
        assert!(engine.may_read(Path::new("/tmp"), &inside, &cwd));
        assert!(!engine.may_read(Path::new("/tmp"), &outside, &cwd));
    }
}
