//! Modular project manifest: a named workspace over one or more path roots.
//! See [[modular-projects]] and [[ADR-0011-modular-project-manifest]].

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{canonical_repo_root, PermissionProfile, PytxoError, Result};

/// One folder/repo on a project's allowlist.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectRoot {
    pub path: PathBuf,
    /// Short stable handle used by tasks (`root = "api"`). Defaults to the directory name.
    #[serde(default)]
    pub label: Option<String>,
    /// Exactly one root should be primary; the first root is primary if none is marked.
    #[serde(default)]
    pub primary: bool,
    /// Read/scaffold only — never receives a Blast Shield flush.
    #[serde(default)]
    pub read_only: bool,
    /// Optional per-root capability override ([[permission-profile-engine]], Phase 25).
    #[serde(default)]
    pub permission_profile: Option<PermissionProfile>,
}

impl ProjectRoot {
    /// Effective label: explicit `label`, else the final path component.
    pub fn effective_label(&self) -> String {
        if let Some(l) = &self.label {
            if !l.trim().is_empty() {
                return l.trim().to_string();
            }
        }
        self.path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "root".to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Parsed project manifest (`[project]` + `[[roots]]`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectMeta,
    #[serde(default)]
    pub roots: Vec<ProjectRoot>,
}

impl ProjectManifest {
    pub fn parse(raw: &str) -> Result<Self> {
        let manifest: ProjectManifest = toml::from_str(raw)
            .map_err(|e| PytxoError::Config(format!("parse project toml: {e}")))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| PytxoError::Config(format!("read {}: {e}", path.display())))?;
        Self::parse(&raw)
    }

    fn validate(&self) -> Result<()> {
        if self.project.id.trim().is_empty() {
            return Err(PytxoError::Config("project.id must not be empty".into()));
        }
        if self.roots.is_empty() {
            return Err(PytxoError::Config(
                "project must declare at least one [[roots]] entry".into(),
            ));
        }
        let primary_count = self.roots.iter().filter(|r| r.primary).count();
        if primary_count > 1 {
            return Err(PytxoError::Config(
                "only one root may be marked primary".into(),
            ));
        }
        for root in &self.roots {
            let label = root.effective_label();
            let normalized = label.replace('\\', "/");
            if normalized.contains(':')
                || normalized
                    .split('/')
                    .any(|part| part.is_empty() || part == "." || part == "..")
            {
                return Err(PytxoError::Config(format!(
                    "root label must be a safe relative context path: {label}"
                )));
            }
        }
        Ok(())
    }

    /// Primary root: the one marked `primary`, else the first declared root.
    pub fn primary_root(&self) -> &ProjectRoot {
        self.roots
            .iter()
            .find(|r| r.primary)
            .unwrap_or(&self.roots[0])
    }

    /// Resolve a task's `root` label to its declared root.
    pub fn root_by_label(&self, label: &str) -> Option<&ProjectRoot> {
        self.roots.iter().find(|r| r.effective_label() == label)
    }

    /// Canonical `DomainId` source for this project (its primary root).
    pub fn primary_repo_root(&self) -> Result<PathBuf> {
        canonical_repo_root(&self.primary_root().path).map_err(PytxoError::Io)
    }

    /// Standard manifest location for a project id under the user config dir.
    pub fn user_manifest_path(id: &str) -> Option<PathBuf> {
        dirs_home().map(|h| h.join(".pytxo").join("projects").join(format!("{id}.toml")))
    }

    /// Project-scoped telemetry directory (`~/.pytxo/projects/<id>/`, Phase 25).
    pub fn project_data_dir(id: &str) -> Option<PathBuf> {
        dirs_home().map(|h| h.join(".pytxo").join("projects").join(id))
    }

    /// Project-scoped telemetry DB path.
    pub fn project_db_path(id: &str) -> Option<PathBuf> {
        Self::project_data_dir(id).map(|d| d.join("pytxo.db"))
    }

    /// Discover a manifest: explicit path, else `~/.pytxo/projects/<id>.toml`,
    /// else `.pytxo/project.toml` under `cwd`.
    pub fn discover(explicit: Option<&Path>, id: Option<&str>, cwd: &Path) -> Option<PathBuf> {
        if let Some(p) = explicit {
            return Some(p.to_path_buf());
        }
        if let Some(id) = id {
            if let Some(p) = Self::user_manifest_path(id) {
                if p.exists() {
                    return Some(p);
                }
            }
        }
        let local = cwd.join(".pytxo").join("project.toml");
        if local.exists() {
            return Some(local);
        }
        None
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[project]
id = "acme-platform"
name = "Acme Platform"

[[roots]]
path = "/home/dev/acme-api"
label = "api"
primary = true

[[roots]]
path = "/home/dev/acme-web"

[[roots]]
path = "/home/dev/shared-protos"
label = "protos"
read_only = true
"#;

    #[test]
    fn parses_and_resolves_primary() {
        let m = ProjectManifest::parse(SAMPLE).unwrap();
        assert_eq!(m.project.id, "acme-platform");
        assert_eq!(m.roots.len(), 3);
        assert_eq!(m.primary_root().effective_label(), "api");
    }

    #[test]
    fn label_defaults_to_dir_name() {
        let m = ProjectManifest::parse(SAMPLE).unwrap();
        // second root has no label → uses directory name
        let web = &m.roots[1];
        assert_eq!(web.effective_label(), "acme-web");
        assert!(m.root_by_label("acme-web").is_some());
        assert!(m.root_by_label("protos").unwrap().read_only);
    }

    #[test]
    fn rejects_empty_roots() {
        let bad = r#"
[project]
id = "x"
"#;
        assert!(ProjectManifest::parse(bad).is_err());
    }

    #[test]
    fn rejects_multiple_primary() {
        let bad = r#"
[project]
id = "x"
[[roots]]
path = "/a"
primary = true
[[roots]]
path = "/b"
primary = true
"#;
        assert!(ProjectManifest::parse(bad).is_err());
    }

    #[test]
    fn context_boundary_rejects_escaping_root_labels() {
        for label in [
            "../escape",
            "/absolute",
            "C:/escape",
            "\\\\server\\share",
            "safe/../escape",
            "safe\\..\\escape",
        ] {
            let mut manifest = ProjectManifest::parse(SAMPLE).unwrap();
            manifest.roots[2].label = Some(label.into());
            let raw = toml::to_string(&manifest).unwrap();
            assert!(
                ProjectManifest::parse(&raw).is_err(),
                "unsafe context root label accepted: {label}"
            );
        }
    }

    #[test]
    fn context_boundary_preserves_safe_nested_root_labels() {
        let mut manifest = ProjectManifest::parse(SAMPLE).unwrap();
        manifest.roots[2].label = Some("shared/protos".into());
        let parsed = ProjectManifest::parse(&toml::to_string(&manifest).unwrap()).unwrap();
        assert_eq!(parsed.roots[2].effective_label(), "shared/protos");
    }
}
