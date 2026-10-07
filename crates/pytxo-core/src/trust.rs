//! Per-folder trust records ([[ADR-0013-folder-trust-tier-picker]]).

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use fs2::FileExt;
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
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let lock_path = sibling_path(path, ".lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(PytxoError::Io)?;
        FileExt::lock_shared(&lock).map_err(PytxoError::Io)?;
        Ok(Self {
            path: path.to_path_buf(),
            data: read_trust_data(path)?,
        })
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
        self.mutate(|data| {
            data.domains.insert(
                id.as_str().to_string(),
                TrustedDomain {
                    permission_profile: profile,
                    trusted_at: Utc::now(),
                    label,
                },
            );
        })
    }

    pub fn revoke(&mut self, repo: &Path) -> crate::Result<bool> {
        let id = DomainId::from_repo_root(repo)?;
        let mut removed = false;
        self.mutate(|data| {
            removed = data.domains.remove(id.as_str()).is_some();
        })?;
        Ok(removed)
    }

    pub fn list(&self) -> Vec<(String, TrustedDomain)> {
        self.data
            .domains
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn mutate(&mut self, update: impl FnOnce(&mut TrustedDomainFile)) -> crate::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let lock_path = sibling_path(&self.path, ".lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(PytxoError::Io)?;
        lock.lock_exclusive().map_err(PytxoError::Io)?;

        self.data = read_trust_data(&self.path)?;
        update(&mut self.data);
        self.flush()
    }

    fn flush(&self) -> crate::Result<()> {
        let raw = serde_json::to_string_pretty(&self.data)
            .map_err(|e| PytxoError::Config(format!("serialize trust: {e}")))?;
        let backup_path = sibling_path(&self.path, ".bak");
        if read_trust_file(&self.path).is_ok_and(|data| data.is_some()) {
            fs::copy(&self.path, &backup_path).map_err(PytxoError::Io)?;
        }

        let temp_path = sibling_path(&self.path, &format!(".{}.tmp", uuid::Uuid::new_v4()));
        let write_result = (|| -> crate::Result<()> {
            let mut temp = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp_path)
                .map_err(PytxoError::Io)?;
            temp.write_all(raw.as_bytes()).map_err(PytxoError::Io)?;
            temp.sync_all().map_err(PytxoError::Io)?;
            drop(temp);

            #[cfg(windows)]
            if self.path.exists() {
                fs::remove_file(&self.path).map_err(PytxoError::Io)?;
            }
            fs::rename(&temp_path, &self.path).map_err(PytxoError::Io)
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(temp_path);
        }
        write_result
    }
}

fn read_trust_data(path: &Path) -> crate::Result<TrustedDomainFile> {
    match read_trust_file(path) {
        Ok(Some(data)) => Ok(data),
        Ok(None) => {
            read_trust_file(&sibling_path(path, ".bak")).map(|data| data.unwrap_or_default())
        }
        Err(primary_error) => match read_trust_file(&sibling_path(path, ".bak")) {
            Ok(Some(data)) => Ok(data),
            _ => Err(primary_error),
        },
    }
}

fn read_trust_file(path: &Path) -> crate::Result<Option<TrustedDomainFile>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(PytxoError::Io)?;
    serde_json::from_str(&raw)
        .map(Some)
        .map_err(|error| PytxoError::Config(format!("parse trust file: {error}")))
}

fn sibling_path(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("trusted-domains.json");
    path.with_file_name(format!("{name}{suffix}"))
}

pub fn default_trust_path() -> crate::Result<PathBuf> {
    if let Ok(path) = std::env::var("PYTXO_TRUST_STORE") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    let home = dirs_home().ok_or_else(|| {
        PytxoError::Config("cannot resolve home directory for trust store".into())
    })?;
    Ok(home.join(".pytxo").join("trusted-domains.json"))
}

/// `PYTXO_HOME` isolates every Pytxo store (catalog, projects, trust), so an
/// isolated home can never trust folders in the operator's real store.
fn dirs_home() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("PYTXO_HOME") {
        return Some(PathBuf::from(home));
    }
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

    #[test]
    fn concurrent_trust_writes_merge_instead_of_corrupting() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trusted.json");
        let repo_a = dir.path().join("repo-a");
        let repo_b = dir.path().join("repo-b");
        fs::create_dir_all(&repo_a).unwrap();
        fs::create_dir_all(&repo_b).unwrap();

        let path_a = path.clone();
        let repo_a_thread = repo_a.clone();
        let first = std::thread::spawn(move || {
            TrustedDomainStore::open(&path_a)
                .unwrap()
                .trust(&repo_a_thread, PermissionProfile::Orbit, None)
                .unwrap();
        });
        let path_b = path.clone();
        let repo_b_thread = repo_b.clone();
        let second = std::thread::spawn(move || {
            TrustedDomainStore::open(&path_b)
                .unwrap()
                .trust(&repo_b_thread, PermissionProfile::Galaxy, None)
                .unwrap();
        });
        first.join().unwrap();
        second.join().unwrap();

        let store = TrustedDomainStore::open(&path).unwrap();
        assert_eq!(
            store.permission_for(&repo_a),
            Some(PermissionProfile::Orbit)
        );
        assert_eq!(
            store.permission_for(&repo_b),
            Some(PermissionProfile::Galaxy)
        );
    }

    #[test]
    fn falls_back_to_last_valid_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trusted.json");
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let mut store = TrustedDomainStore::open(&path).unwrap();
        store.trust(&repo, PermissionProfile::Orbit, None).unwrap();
        store.trust(&repo, PermissionProfile::Galaxy, None).unwrap();
        fs::write(&path, "{\"domains\": {}} trailing").unwrap();

        let recovered = TrustedDomainStore::open(&path).unwrap();
        assert_eq!(
            recovered.permission_for(&repo),
            Some(PermissionProfile::Orbit)
        );
    }
}
