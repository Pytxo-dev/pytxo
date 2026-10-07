//! Cooperative, cross-process upgrade admission for one user catalog.
//! Shared work leases coexist. An exclusive upgrade lease prevents new work;
//! OS ownership releases on process exit, without stale lock-file deletion.
use crate::{PytxoError, Result};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct UpgradeGuard(File);

impl UpgradeGuard {
    fn path() -> Result<PathBuf> {
        std::env::var_os("PYTXO_HOME")
            .or_else(|| std::env::var_os("HOME"))
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(|home| PathBuf::from(home).join(".pytxo").join("upgrade.lock"))
            .ok_or_else(|| PytxoError::Runner("cannot resolve upgrade lock directory".into()))
    }

    pub fn work() -> Result<Self> {
        Self::acquire(&Self::path()?, false)
    }
    pub fn upgrade() -> Result<Self> {
        Self::acquire(&Self::path()?, true)
    }

    fn acquire(path: &Path, exclusive: bool) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        let locked = if exclusive {
            fs2::FileExt::try_lock_exclusive(&file)
        } else {
            fs2::FileExt::try_lock_shared(&file)
        };
        locked.map_err(|error| {
            PytxoError::Runner(format!(
                "{}: {error}",
                if exclusive {
                    "Work is still active; finish it before updating"
                } else {
                    "An update is in progress; retry after it finishes"
                }
            ))
        })?;
        Ok(Self(file))
    }
}

impl Drop for UpgradeGuard {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_guard_child() {
        let Some(path) = std::env::var_os("PYTXO_TEST_UPGRADE_LOCK") else {
            return;
        };
        let mode = std::env::var("PYTXO_TEST_UPGRADE_MODE").unwrap();
        let exclusive = mode == "exclusive" || mode == "exit";
        let expected = std::env::var("PYTXO_TEST_UPGRADE_EXPECT").unwrap() == "allow";
        let acquired = UpgradeGuard::acquire(Path::new(&path), exclusive);
        assert_eq!(acquired.is_ok(), expected);
        if mode == "exit" {
            std::process::exit(0);
        } // bypass Rust Drop
    }

    #[test]
    fn upgrade_guard_excludes_other_processes_and_releases_without_deleting_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("upgrade.lock");
        let child = |mode: &str, expected: &str| {
            assert!(crate::background_command(std::env::current_exe().unwrap())
                .args(["--exact", "upgrade_guard::tests::upgrade_guard_child"])
                .env("PYTXO_TEST_UPGRADE_LOCK", &path)
                .env("PYTXO_TEST_UPGRADE_MODE", mode)
                .env("PYTXO_TEST_UPGRADE_EXPECT", expected)
                .status()
                .unwrap()
                .success());
        };
        let worker = UpgradeGuard::acquire(&path, false).unwrap();
        child("shared", "allow");
        child("exclusive", "deny");
        drop(worker);
        let installer = UpgradeGuard::acquire(&path, true).unwrap();
        child("shared", "deny");
        child("exclusive", "deny");
        drop(installer);
        assert!(path.exists());
        child("shared", "allow");
        child("exclusive", "allow");
        child("exit", "allow");
        assert!(UpgradeGuard::acquire(&path, false).is_ok());
    }
}
