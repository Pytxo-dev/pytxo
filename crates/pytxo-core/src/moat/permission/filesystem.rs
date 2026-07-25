//! Filesystem policy trait ([[permission-profile-engine]], Phase 30).

use std::path::{Component, Path, PathBuf};

use super::PermissionProfile;

/// Gates read/write/flush before Blast Shield or MCP materialization.
pub trait FilesystemPolicy {
    /// Whether a path under `repo_root` may be read for agent context.
    fn may_read(&self, repo_root: &Path, path: &Path, agent_cwd: &Path) -> bool;

    /// Whether physical flush to disk is allowed for this profile.
    fn may_flush(&self) -> bool;

    fn flush_requires_approval(&self) -> bool;

    fn use_worktree_isolation(&self) -> bool;
}

/// Lexically resolve `.` / `..` without requiring the path to exist.
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Resolve `path` relative to `repo_root` and test DeepSpace cwd-only reads.
pub fn path_under_cwd(agent_cwd: &Path, path: &Path) -> bool {
    let cwd = std::fs::canonicalize(agent_cwd).unwrap_or_else(|_| normalize_lexically(agent_cwd));
    let target = if path.is_absolute() {
        std::fs::canonicalize(path).unwrap_or_else(|_| normalize_lexically(path))
    } else {
        std::fs::canonicalize(agent_cwd.join(path))
            .unwrap_or_else(|_| normalize_lexically(&agent_cwd.join(path)))
    };
    target.starts_with(&cwd)
}

/// Resolve `path` and require it stays under `repo_root` (Orbit+ / default).
pub fn path_under_repo(repo_root: &Path, path: &Path) -> bool {
    let root = std::fs::canonicalize(repo_root).unwrap_or_else(|_| normalize_lexically(repo_root));
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    let target =
        std::fs::canonicalize(&candidate).unwrap_or_else(|_| normalize_lexically(&candidate));
    target.starts_with(&root)
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
    fn may_read(&self, repo_root: &Path, path: &Path, agent_cwd: &Path) -> bool {
        match self.profile {
            PermissionProfile::DeepSpace => path_under_cwd(agent_cwd, path),
            _ => path_under_repo(repo_root, path),
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
        assert!(engine.may_read(Path::new("/tmp"), &inside, &cwd));
        assert!(!engine.may_read(Path::new("/tmp"), &outside, &cwd));
    }

    #[test]
    fn orbit_denies_read_outside_repo() {
        let engine = FilesystemPolicyEngine::new(PermissionProfile::Orbit);
        let root = PathBuf::from("/tmp/repo");
        let inside = PathBuf::from("/tmp/repo/src/a.rs");
        let outside = PathBuf::from("/tmp/other/secret");
        assert!(engine.may_read(&root, &inside, &root));
        assert!(!engine.may_read(&root, &outside, &root));
        assert!(!engine.may_read(&root, Path::new("../../../etc/passwd"), &root));
    }
}
