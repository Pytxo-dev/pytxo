//! Per-folder trust records ([[ADR-0013-folder-trust-tier-picker]]).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::moat::{DomainId, PermissionProfile};
use crate::PytxoError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustedDomain {
    pub permission_profile: PermissionProfile,
    pub trusted_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct TrustedDomainFile {
    #[serde(default)]
    domains: HashMap<String, TrustedDomain>,
}

/// Persisted trust store at `~/.pytxo/trusted-domains.json`.
#[derive(Clone, Debug)]
pub struct TrustedDomainStore {
    path: PathBuf,
    data: TrustedDomainFile,
}

impl TrustedDomainStore {
    pub fn open_default() -> crate::Result<Self> {
        let path = default_trust_path()?;
        Self::open(&path)
    }

    pub fn open(path: &Path) -> crate::Result<Self> {
        if path.exists() {
            let raw = fs::read_to_string(path).map_err(PytxoError::Io)?;
            let data: TrustedDomainFile = serde_json::from_str(&raw)
                .map_err(|e| PytxoError::Config(format!("parse trust file: {e}")))?;
            Ok(Self {
                path: path.to_path_buf(),
                data,
            })
        } else {
            Ok(Self {
                path: path.to_path_buf(),
                data: TrustedDomainFile::default(),
            })
        }
    }

    pub fn is_trusted(&self, repo: &Path) -> bool {
        self.lookup(repo).is_some()
    }

    pub fn lookup(&self, repo: &Path) -> Option<&TrustedDomain> {
        let id = DomainId::from_repo_root(repo).ok()?;
        self.data.domains.get(id.as_str())
    }

    pub fn permission_for(&self, repo: &Path) -> Option<PermissionProfile> {
        self.lookup(repo).map(|d| d.permission_profile)
    }

    pub fn trust(
        &mut self,
        repo: &Path,
        profile: PermissionProfile,
        label: Option<String>,
    ) -> crate::Result<()> {
        let id = DomainId::from_repo_root(repo)?;
        self.data.domains.insert(
            id.as_str().to_string(),
            TrustedDomain {
                permission_profile: profile,
                trusted_at: Utc::now(),
                label,
            },
        );
        self.flush()
    }

    pub fn revoke(&mut self, repo: &Path) -> crate::Result<bool> {
        let id = DomainId::from_repo_root(repo)?;
        let removed = self.data.domains.remove(id.as_str()).is_some();
        if removed {
            self.flush()?;
        }
        Ok(removed)
    }

    pub fn list(&self) -> Vec<(String, TrustedDomain)> {
        self.data
            .domains
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn flush(&self) -> crate::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let raw = serde_json::to_string_pretty(&self.data)
            .map_err(|e| PytxoError::Config(format!("serialize trust: {e}")))?;
        fs::write(&self.path, raw).map_err(PytxoError::Io)
    }
}

pub fn default_trust_path() -> crate::Result<PathBuf> {
    let home = dirs_home().ok_or_else(|| {
        PytxoError::Config("cannot resolve home directory for trust store".into())
    })?;
    Ok(home.join(".pytxo").join("trusted-domains.json"))
}

fn dirs_home() -> Option<PathBuf> {
    if let Ok(h) = std::env::var("USERPROFILE") {
        return Some(PathBuf::from(h));
    }
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_roundtrip() {
        let dir = std::env::temp_dir().join(format!("pytxo-trust-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("trusted.json");
        let mut store = TrustedDomainStore::open(&path).unwrap();
        assert!(!store.is_trusted(&dir));
        store.trust(&dir, PermissionProfile::Galaxy, None).unwrap();
        let store2 = TrustedDomainStore::open(&path).unwrap();
        assert_eq!(store2.permission_for(&dir), Some(PermissionProfile::Galaxy));
        let _ = fs::remove_dir_all(dir);
    }
}
